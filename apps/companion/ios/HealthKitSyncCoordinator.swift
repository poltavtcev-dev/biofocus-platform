import Foundation
import HealthKit

/// HealthKit observers → local queue → Desktop ingest flush (ADR-016 / ADR-018).
/// Event / background-delivery driven — no busy-loop polling.
@MainActor
final class HealthKitSyncCoordinator: ObservableObject {
    static let shared = HealthKitSyncCoordinator()

    @Published private(set) var lastStatus: String = "Idle"
    @Published private(set) var lastWasError: Bool = false
    @Published private(set) var pendingCount: Int = 0

    private let store = HKHealthStore()
    private var observersStarted = false
    private let anchorHRKey = "biofocus.hk.anchor.hr"
    private let anchorHRVKey = "biofocus.hk.anchor.hrv"
    private let anchorStepsKey = "biofocus.hk.anchor.steps"
    private let anchorEnergyKey = "biofocus.hk.anchor.energy"
    private let anchorSleepKey = "biofocus.hk.anchor.sleep"
    private let anchorSpO2Key = "biofocus.hk.anchor.spo2"

    private init() {
        pendingCount = ObservationQueue.pendingCount
    }

    var isAutoSyncEnabled: Bool {
        get { UserDefaults.standard.object(forKey: "biofocus.autoSync") as? Bool ?? true }
        set {
            UserDefaults.standard.set(newValue, forKey: "biofocus.autoSync")
            if newValue {
                Task { await startAutoSyncIfNeeded() }
            }
        }
    }

    func refreshPendingCount() {
        pendingCount = ObservationQueue.pendingCount
    }

    func requestAuthorization() async throws {
        guard HKHealthStore.isHealthDataAvailable() else {
            throw HeartRateSampleError.healthDataUnavailable
        }
        var types: Set<HKObjectType> = []
        if let hr = HKQuantityType.quantityType(forIdentifier: .heartRate) {
            types.insert(hr)
        }
        if let hrv = HrvSample.hrvType {
            types.insert(hrv)
        }
        if let steps = StepCountSample.quantityType {
            types.insert(steps)
        }
        if let energy = ActiveEnergySample.quantityType {
            types.insert(energy)
        }
        if let sleep = SleepIntervalSample.categoryType {
            types.insert(sleep)
        }
        if let spo2 = OxygenSaturationSample.quantityType {
            types.insert(spo2)
        }
        try await store.requestAuthorization(toShare: [], read: types)
    }

    func startAutoSyncIfNeeded() async {
        guard isAutoSyncEnabled else { return }
        do {
            try await requestAuthorization()
            try await enableBackgroundDelivery()
            startObservers()
            await ingestNewSamplesAndFlush()
        } catch {
            lastWasError = true
            lastStatus = "Auto-sync setup: \(error.localizedDescription)"
        }
    }

    /// Manual: enqueue latest wearable samples (soft-omit sparse types) and flush.
    func sendLatestNow(baseURL: URL, token: String) async {
        lastWasError = false
        lastStatus = "Reading HealthKit…"
        do {
            try await requestAuthorization()
            var enqueued = 0
            var notes: [String] = []

            if let hr = try? await HeartRateSample.latestObservation(store: store) {
                try ObservationQueue.enqueueEncodable(dedupeKey: "hr-\(hr.id)", hr)
                enqueued += 1
            }
            do {
                let hrv = try await HrvSample.latestObservation(store: store)
                try ObservationQueue.enqueueEncodable(dedupeKey: "hrv-\(hrv.id)", hrv)
                enqueued += 1
            } catch {
                notes.append(error.localizedDescription)
            }
            if let steps = try? await StepCountSample.latestObservation(store: store) {
                try ObservationQueue.enqueueEncodable(dedupeKey: "steps-\(steps.id)", steps)
                enqueued += 1
            }
            if let energy = try? await ActiveEnergySample.latestObservation(store: store) {
                try ObservationQueue.enqueueEncodable(dedupeKey: "energy-\(energy.id)", energy)
                enqueued += 1
            }
            if let sleep = try? await SleepIntervalSample.latestObservation(store: store) {
                try ObservationQueue.enqueueEncodable(dedupeKey: "sleep-\(sleep.id)", sleep)
                enqueued += 1
            }
            // Soft-optional SpO2 — never invent when absent.
            if let spo2 = try? await OxygenSaturationSample.latestObservation(store: store) {
                try ObservationQueue.enqueueEncodable(dedupeKey: "spo2-\(spo2.id)", spo2)
                enqueued += 1
            }

            refreshPendingCount()
            guard enqueued > 0 else {
                lastWasError = true
                lastStatus = notes.first ?? HeartRateSampleError.noSample.localizedDescription
                return
            }
            try await flush(baseURL: baseURL, token: token)
            if !notes.isEmpty, !lastWasError {
                lastStatus = "Sent \(enqueued) sample(s). Note: \(notes.joined(separator: "; "))"
            } else if !lastWasError {
                lastStatus = "Sent \(enqueued) sample(s)."
            }
        } catch {
            lastWasError = true
            lastStatus = error.localizedDescription
        }
        refreshPendingCount()
    }

    func flush(baseURL: URL, token: String) async throws {
        guard let body = ObservationQueue.encodeBatchBody() else {
            lastWasError = false
            lastStatus = "Nothing pending to send."
            return
        }
        do {
            let client = IngestClient(baseURL: baseURL, token: token)
            try await client.postObservations(body)
            ObservationQueue.clearAll()
            refreshPendingCount()
            UserDefaults.standard.set(Date().timeIntervalSince1970, forKey: "biofocus.lastFlushSuccess")
            lastWasError = false
            lastStatus = "Flush ok — queue empty."
        } catch {
            lastWasError = true
            lastStatus = error.localizedDescription
            refreshPendingCount()
            throw error
        }
    }

    func reportPairingError(_ message: String) {
        lastWasError = true
        lastStatus = message
    }

    /// Preflight Desktop reachability before Send/Flush (P28-E1-T1).
    func testConnection(baseURL: URL, token: String) async {
        lastWasError = false
        lastStatus = "Testing connection…"
        do {
            let client = IngestClient(baseURL: baseURL, token: token)
            let status = try await client.fetchStatus()
            lastWasError = false
            lastStatus =
                "Connected — bind_mode=\(status.bind_mode), db_status=\(status.db_status)."
        } catch let err as IngestClientError {
            lastWasError = true
            lastStatus = err.localizedDescription
        } catch {
            lastWasError = true
            lastStatus = error.localizedDescription
        }
    }

    /// Preflight `GET /v1/status` before Send/Flush (P28-E1-T1).
    func preflightConnection(baseURL: URL, token: String) async throws {
        let client = IngestClient(baseURL: baseURL, token: token)
        _ = try await client.fetchStatus()
    }

    func lastFlushDate() -> Date? {
        let t = UserDefaults.standard.double(forKey: "biofocus.lastFlushSuccess")
        guard t > 0 else { return nil }
        return Date(timeIntervalSince1970: t)
    }

    // MARK: - Background / observers

    private func enableBackgroundDelivery() async throws {
        let quantityIds: [HKQuantityTypeIdentifier] = [
            .heartRate,
            .heartRateVariabilitySDNN,
            .stepCount,
            .activeEnergyBurned,
            .oxygenSaturation,
        ]
        for id in quantityIds {
            if let t = HKQuantityType.quantityType(forIdentifier: id) {
                try await store.enableBackgroundDelivery(for: t, frequency: .hourly)
            }
        }
        if let sleep = SleepIntervalSample.categoryType {
            try await store.enableBackgroundDelivery(for: sleep, frequency: .hourly)
        }
    }

    private func startObservers() {
        guard !observersStarted else { return }
        observersStarted = true

        let quantityIds: [HKQuantityTypeIdentifier] = [
            .heartRate,
            .heartRateVariabilitySDNN,
            .stepCount,
            .activeEnergyBurned,
            .oxygenSaturation,
        ]
        for id in quantityIds {
            if let t = HKQuantityType.quantityType(forIdentifier: id) {
                attachObserver(sampleType: t, label: id.rawValue)
            }
        }
        if let sleep = SleepIntervalSample.categoryType {
            attachObserver(sampleType: sleep, label: "sleepAnalysis")
        }
    }

    private func attachObserver(sampleType: HKSampleType, label: String) {
        let q = HKObserverQuery(sampleType: sampleType, predicate: nil) { [weak self] _, completion, error in
            Task { @MainActor in
                if let error {
                    self?.lastWasError = true
                    self?.lastStatus = "\(label) observer: \(error.localizedDescription)"
                } else if self?.isAutoSyncEnabled == true {
                    await self?.ingestNewSamplesAndFlush()
                }
                completion()
            }
        }
        store.execute(q)
    }

    private func ingestNewSamplesAndFlush() async {
        guard isAutoSyncEnabled else { return }
        let baseText = UserDefaults.standard.string(forKey: "biofocus.ingestBaseURL") ?? ""
        let token = UserDefaults.standard.string(forKey: "biofocus.pairingToken") ?? ""
        guard let baseURL = URL(string: baseText), !token.isEmpty else {
            lastStatus = "Auto-sync armed — set Base URL + token to flush."
            return
        }

        do {
            let hrSamples = try await HeartRateSample.observations(since: anchorDate(anchorHRKey), store: store)
            for dto in hrSamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "hr-\(dto.id)", dto)
                setAnchor(anchorHRKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }
            let hrvSamples = try await HrvSample.observations(since: anchorDate(anchorHRVKey), store: store)
            for dto in hrvSamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "hrv-\(dto.id)", dto)
                setAnchor(anchorHRVKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }
            let stepSamples = try await StepCountSample.observations(since: anchorDate(anchorStepsKey), store: store)
            for dto in stepSamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "steps-\(dto.id)", dto)
                setAnchor(anchorStepsKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }
            let energySamples = try await ActiveEnergySample.observations(since: anchorDate(anchorEnergyKey), store: store)
            for dto in energySamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "energy-\(dto.id)", dto)
                setAnchor(anchorEnergyKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }
            let sleepSamples = try await SleepIntervalSample.observations(since: anchorDate(anchorSleepKey), store: store)
            for dto in sleepSamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "sleep-\(dto.id)", dto)
                setAnchor(anchorSleepKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }
            // Soft-optional SpO2 — empty result is fine (no invent).
            let spo2Samples = try await OxygenSaturationSample.observations(since: anchorDate(anchorSpO2Key), store: store)
            for dto in spo2Samples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "spo2-\(dto.id)", dto)
                setAnchor(anchorSpO2Key, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }

            refreshPendingCount()
            if ObservationQueue.pendingCount > 0 {
                try await flush(baseURL: baseURL, token: token)
            }
        } catch {
            lastWasError = true
            lastStatus = "Sync: \(error.localizedDescription) — kept in queue (\(ObservationQueue.pendingCount))."
            refreshPendingCount()
        }
    }

    private func anchorDate(_ key: String) -> Date? {
        let t = UserDefaults.standard.double(forKey: key)
        guard t > 0 else { return nil }
        return Date(timeIntervalSince1970: t)
    }

    private func setAnchor(_ key: String, date: Date) {
        let prev = UserDefaults.standard.double(forKey: key)
        let next = date.timeIntervalSince1970
        if next > prev {
            UserDefaults.standard.set(next, forKey: key)
        }
    }
}
