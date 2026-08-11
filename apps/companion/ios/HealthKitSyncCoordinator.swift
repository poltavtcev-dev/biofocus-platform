import Foundation
import HealthKit

/// HealthKit observers → local queue → Desktop ingest flush (ADR-016).
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

    /// Manual: enqueue latest HR (+ HRV if present) and flush.
    func sendLatestNow(baseURL: URL, token: String) async {
        lastWasError = false
        lastStatus = "Reading HealthKit…"
        do {
            try await requestAuthorization()
            var enqueued = 0
            if let hr = try? await HeartRateSample.latestObservation(store: store) {
                try ObservationQueue.enqueueEncodable(dedupeKey: "hr-\(hr.id)", hr)
                enqueued += 1
            }
            var hrvNote: String?
            do {
                let hrv = try await HrvSample.latestObservation(store: store)
                try ObservationQueue.enqueueEncodable(dedupeKey: "hrv-\(hrv.id)", hrv)
                enqueued += 1
            } catch {
                hrvNote = error.localizedDescription
            }
            refreshPendingCount()
            guard enqueued > 0 else {
                lastWasError = true
                lastStatus = hrvNote ?? HeartRateSampleError.noSample.localizedDescription
                return
            }
            try await flush(baseURL: baseURL, token: token)
            if let hrvNote, !lastWasError {
                lastStatus = "Sent. Note: \(hrvNote)"
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

    func lastFlushDate() -> Date? {
        let t = UserDefaults.standard.double(forKey: "biofocus.lastFlushSuccess")
        guard t > 0 else { return nil }
        return Date(timeIntervalSince1970: t)
    }

    // MARK: - Background / observers

    private func enableBackgroundDelivery() async throws {
        if let hr = HKQuantityType.quantityType(forIdentifier: .heartRate) {
            try await store.enableBackgroundDelivery(for: hr, frequency: .hourly)
        }
        if let hrv = HrvSample.hrvType {
            try await store.enableBackgroundDelivery(for: hrv, frequency: .hourly)
        }
    }

    private func startObservers() {
        guard !observersStarted else { return }
        observersStarted = true

        if let hr = HKQuantityType.quantityType(forIdentifier: .heartRate) {
            let q = HKObserverQuery(sampleType: hr, predicate: nil) { [weak self] _, completion, error in
                Task { @MainActor in
                    if let error {
                        self?.lastWasError = true
                        self?.lastStatus = "HR observer: \(error.localizedDescription)"
                    } else if self?.isAutoSyncEnabled == true {
                        await self?.ingestNewSamplesAndFlush()
                    }
                    completion()
                }
            }
            store.execute(q)
        }

        if let hrv = HrvSample.hrvType {
            let q = HKObserverQuery(sampleType: hrv, predicate: nil) { [weak self] _, completion, error in
                Task { @MainActor in
                    if let error {
                        self?.lastWasError = true
                        self?.lastStatus = "HRV observer: \(error.localizedDescription)"
                    } else if self?.isAutoSyncEnabled == true {
                        await self?.ingestNewSamplesAndFlush()
                    }
                    completion()
                }
            }
            store.execute(q)
        }
    }

    private func ingestNewSamplesAndFlush() async {
        guard isAutoSyncEnabled else { return }
        let baseText = UserDefaults.standard.string(forKey: "biofocus.ingestBaseURL") ?? ""
        let token = UserDefaults.standard.string(forKey: "biofocus.pairingToken") ?? ""
        guard let baseURL = URL(string: baseText), !token.isEmpty else {
            lastStatus = "Auto-sync armed — set Base URL + token to flush."
            return
        }

        let hrSince = anchorDate(anchorHRKey)
        let hrvSince = anchorDate(anchorHRVKey)

        do {
            let hrSamples = try await HeartRateSample.observations(since: hrSince, store: store)
            for dto in hrSamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "hr-\(dto.id)", dto)
                setAnchor(anchorHRKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
            }
            let hrvSamples = try await HrvSample.observations(since: hrvSince, store: store)
            for dto in hrvSamples {
                try ObservationQueue.enqueueEncodable(dedupeKey: "hrv-\(dto.id)", dto)
                setAnchor(anchorHRVKey, date: Date(timeIntervalSince1970: TimeInterval(dto.timestamp)))
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