import BackgroundTasks
import Foundation
import HealthKit
import Security

/// HealthKit observers → local queue → Desktop ingest (ADR-016 / ADR-030).
/// Forward uses an `HKQueryAnchor` (new, late writes, deletions). Backfill walks newest-first.
@MainActor
final class HealthKitSyncCoordinator: ObservableObject {
    static let shared = HealthKitSyncCoordinator()
    static let backfillTaskId = "com.biofocus.companion.backfill"
    static let refreshTaskId = "com.biofocus.companion.refresh"

    @Published private(set) var lastStatus: String = "Idle"
    @Published private(set) var lastWasError: Bool = false
    @Published private(set) var pendingCount: Int = 0

    private let store = HKHealthStore()
    private var observersStarted = false
    private var backgroundRegistered = false
    private var retryScheduled = false
    private var flushFailures = 0

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

    func registerBackgroundTasks() {
        guard !backgroundRegistered else { return }
        backgroundRegistered = true
        BGTaskScheduler.shared.register(forTaskWithIdentifier: Self.backfillTaskId, using: nil) { task in
            Task { @MainActor in
                await HealthKitSyncCoordinator.shared.handleBackground(task)
            }
        }
        BGTaskScheduler.shared.register(forTaskWithIdentifier: Self.refreshTaskId, using: nil) { task in
            Task { @MainActor in
                await HealthKitSyncCoordinator.shared.handleBackground(task)
            }
        }
    }

    func requestAuthorization() async throws {
        guard HKHealthStore.isHealthDataAvailable() else {
            throw HeartRateSampleError.healthDataUnavailable
        }
        try await store.requestAuthorization(toShare: [], read: HealthTypeRegistry.readTypes)
    }

    /// Observers and background delivery are registered at launch, before the first flush.
    func prepareAtLaunch() async {
        registerBackgroundTasks()
        scheduleBackgroundTasks()
        guard isAutoSyncEnabled else { return }
        await startAutoSyncIfNeeded()
    }

    func startAutoSyncIfNeeded() async {
        guard isAutoSyncEnabled else { return }
        do {
            try await requestAuthorization()
            try await enableBackgroundDelivery()
            startObservers()
            try await pullAndFlush(rounds: 3)
        } catch {
            lastWasError = true
            lastStatus = "Auto-sync setup: \(error.localizedDescription)"
        }
    }

    /// Manual sync uses the same newest-first pull as launch, not a single latest sample.
    func sendLatestNow(baseURL: URL, token: String) async {
        lastWasError = false
        lastStatus = "Reading HealthKit…"
        UserDefaults.standard.set(baseURL.absoluteString, forKey: "biofocus.ingestBaseURL")
        UserDefaults.standard.set(token, forKey: "biofocus.pairingToken")
        do {
            try await requestAuthorization()
            try await enableBackgroundDelivery()
            startObservers()
            try await pullAndFlush(rounds: 3, baseURL: baseURL, token: token)
        } catch {
            lastWasError = true
            lastStatus = error.localizedDescription
        }
        refreshPendingCount()
    }

    func flush(baseURL: URL, token: String) async throws {
        let client = IngestClient(baseURL: baseURL, token: token)
        var sent = 0
        do {
            while let batch = ObservationQueue.nextBatch() {
                try await client.postObservations(batch.body)
                ObservationQueue.remove(keys: batch.keys)
                sent += batch.keys.count
                refreshPendingCount()
            }
            UserDefaults.standard.set(Date().timeIntervalSince1970, forKey: "biofocus.lastFlushSuccess")
            flushFailures = 0
            lastWasError = false
            lastStatus = sent == 0 ? "Nothing pending to send." : "Flush ok — sent \(sent)."
        } catch {
            lastWasError = true
            lastStatus = "\(error.localizedDescription) — kept in queue (\(ObservationQueue.pendingCount))."
            refreshPendingCount()
            scheduleFlushRetry()
            throw error
        }
    }

    func reportPairingError(_ message: String) {
        lastWasError = true
        lastStatus = message
    }

    func testConnection(baseURL: URL, token: String) async {
        lastWasError = false
        lastStatus = "Testing connection…"
        do {
            let client = IngestClient(baseURL: baseURL, token: token)
            let status = try await client.fetchStatus()
            lastWasError = false
            lastStatus = "Connected — bind_mode=\(status.bind_mode), db_status=\(status.db_status)."
        } catch let err as IngestClientError {
            lastWasError = true
            lastStatus = err.localizedDescription
        } catch {
            lastWasError = true
            lastStatus = error.localizedDescription
        }
    }

    func preflightConnection(baseURL: URL, token: String) async throws {
        let client = IngestClient(baseURL: baseURL, token: token)
        _ = try await client.fetchStatus()
    }

    func lastFlushDate() -> Date? {
        let t = UserDefaults.standard.double(forKey: "biofocus.lastFlushSuccess")
        guard t > 0 else { return nil }
        return Date(timeIntervalSince1970: t)
    }

    // MARK: - Pull

    private func pullAndFlush(rounds: Int, baseURL: URL? = nil, token: String? = nil) async throws {
        let salt = DeviceSalt.loadOrCreate()
        for _ in 0..<rounds {
            let progressed = try await pullOneRound(salt: salt)
            if !progressed {
                break
            }
        }
        refreshPendingCount()
        let base = baseURL ?? configuredBaseURL()
        let auth = token ?? (UserDefaults.standard.string(forKey: "biofocus.pairingToken") ?? "")
        guard let base, !auth.isEmpty else {
            lastWasError = false
            lastStatus = "Auto-sync armed — set Base URL + token to flush. Pending \(ObservationQueue.pendingCount)."
            return
        }
        if ObservationQueue.pendingCount > 0 {
            try await flush(baseURL: base, token: auth)
        } else if !lastWasError {
            lastStatus = "Health is up to date for the recent window."
        }
    }

    private func pullOneRound(salt: Data) async throws -> Bool {
        let now = Int64(Date().timeIntervalSince1970)
        let url = SyncStateStore.fileURL(in: DeviceSalt.supportDirectory())
        var state = SyncStateStore.load(from: url, now: now)
        if state.installTime == 0 {
            state.installTime = now
        }
        var progressed = false
        let cutoff = Date(timeIntervalSince1970: TimeInterval(SyncPlanner.forwardCutoff(installTime: state.installTime)))
        let forwardPredicate = HKQuery.predicateForSamples(withStart: cutoff, end: nil, options: .strictStartDate)

        for identifier in HealthTypeRegistry.syncPriority {
            guard let sampleType = HealthTypeRegistry.sampleType(for: identifier) else { continue }
            var cursor = SyncPlanner.cursor(for: identifier, state: state, now: now)
            let anchor = cursor.forwardAnchor.flatMap(AnchoredCursor.unarchive)
            let forward = try await anchoredPage(type: sampleType, predicate: forwardPredicate, anchor: anchor)
            if !forward.samples.isEmpty || !forward.deleted.isEmpty {
                progressed = true
            }
            for sample in forward.samples {
                enqueue(sample: sample, identifier: identifier, salt: salt)
            }
            for deleted in forward.deleted {
                enqueueDeletion(id: deleted.uuid.uuidString, now: now, salt: salt)
            }
            if let newAnchor = forward.anchor {
                cursor.forwardAnchor = AnchoredCursor.archive(newAnchor)
            }

            if cursor.phase != "done" {
                let page = BackfillPage(
                    identifier: identifier,
                    start: cursor.phase == "recent"
                        ? now - SyncPlanner.recentSeconds
                        : (state.historyDays > 0 ? now - Int64(state.historyDays) * 86_400 : nil),
                    endExclusive: cursor.cursorEnd,
                    limit: SyncPlanner.pageSize
                )
                let samples = try await descendingPage(type: sampleType, page: page)
                if !samples.isEmpty {
                    progressed = true
                }
                for sample in samples {
                    enqueue(sample: sample, identifier: identifier, salt: salt)
                }
                let ends = samples.map { Int64($0.endDate.timeIntervalSince1970) }
                SyncPlanner.advance(
                    cursor: &cursor,
                    now: now,
                    historyDays: state.historyDays,
                    returnedEnds: ends,
                    pageLimit: page.limit
                )
            }
            state.types[identifier] = cursor
        }
        SyncStateStore.save(state, to: url)
        return progressed
    }

    private func enqueue(sample: HKSample, identifier: String, salt: Data) {
        let record: HealthRecord
        if let quantity = sample as? HKQuantitySample {
            record = HKRecordAdapter.quantity(quantity, identifier: identifier)
        } else if let category = sample as? HKCategorySample {
            record = HKRecordAdapter.category(category, identifier: identifier)
        } else if let workout = sample as? HKWorkout {
            record = HKRecordAdapter.workout(workout)
        } else {
            return
        }
        guard let data = WearableMapper.jsonData(for: record, salt: salt) else { return }
        ObservationQueue.enqueue(dedupeKey: record.uuid.lowercased(), observationJSON: data)
    }

    private func enqueueDeletion(id: String, now: Int64, salt: Data) {
        _ = salt
        guard let data = WearableMapper.deletionJSON(targetId: id, timestamp: now) else { return }
        ObservationQueue.enqueue(dedupeKey: "del-\(id.lowercased())", observationJSON: data)
    }

    private func configuredBaseURL() -> URL? {
        let text = UserDefaults.standard.string(forKey: "biofocus.ingestBaseURL") ?? ""
        return URL(string: text)
    }

    private func scheduleFlushRetry() {
        flushFailures += 1
        guard !retryScheduled else { return }
        retryScheduled = true
        let shift = UInt64(min(60, 1 << min(flushFailures, 5)))
        Task { @MainActor in
            try? await Task.sleep(nanoseconds: shift * 1_000_000_000)
            self.retryScheduled = false
            guard self.isAutoSyncEnabled, let base = self.configuredBaseURL() else { return }
            let token = UserDefaults.standard.string(forKey: "biofocus.pairingToken") ?? ""
            guard !token.isEmpty else { return }
            try? await self.flush(baseURL: base, token: token)
        }
    }

    // MARK: - HealthKit queries

    private func enableBackgroundDelivery() async throws {
        for identifier in HealthTypeRegistry.syncPriority {
            guard let sampleType = HealthTypeRegistry.sampleType(for: identifier) else { continue }
            let immediate: Set<String> = [
                HealthTypeRegistry.heartRate,
                HealthTypeRegistry.hrvSDNN,
                HealthTypeRegistry.sleep,
            ]
            let frequency: HKUpdateFrequency = immediate.contains(identifier) ? .immediate : .hourly
            do {
                try await store.enableBackgroundDelivery(for: sampleType, frequency: frequency)
            } catch {
                try await store.enableBackgroundDelivery(for: sampleType, frequency: .hourly)
            }
        }
    }

    private func startObservers() {
        guard !observersStarted else { return }
        observersStarted = true
        for identifier in HealthTypeRegistry.syncPriority {
            guard let sampleType = HealthTypeRegistry.sampleType(for: identifier) else { continue }
            let query = HKObserverQuery(sampleType: sampleType, predicate: nil) { _, completion, error in
                Task { @MainActor in
                    try await ObserverFinish.callAlways(completion: completion) {
                        if let error {
                            self.lastWasError = true
                            self.lastStatus = "\(identifier) observer: \(error.localizedDescription)"
                            return
                        }
                        guard self.isAutoSyncEnabled else { return }
                        try await self.pullAndFlush(rounds: 1)
                    }
                }
            }
            store.execute(query)
        }
    }

    private func anchoredPage(
        type: HKSampleType,
        predicate: NSPredicate,
        anchor: HKQueryAnchor?
    ) async throws -> (samples: [HKSample], deleted: [HKDeletedObject], anchor: HKQueryAnchor?) {
        try await withCheckedThrowingContinuation { continuation in
            let query = HKAnchoredObjectQuery(
                type: type,
                predicate: predicate,
                anchor: anchor,
                limit: SyncPlanner.pageSize
            ) { _, samples, deleted, newAnchor, error in
                if let error {
                    continuation.resume(throwing: error)
                    return
                }
                continuation.resume(returning: (samples ?? [], deleted ?? [], newAnchor))
            }
            store.execute(query)
        }
    }

    private func descendingPage(type: HKSampleType, page: BackfillPage) async throws -> [HKSample] {
        try await withCheckedThrowingContinuation { continuation in
            let end = Date(timeIntervalSince1970: TimeInterval(page.endExclusive) - 0.001)
            let start = page.start.map { Date(timeIntervalSince1970: TimeInterval($0)) }
            var options: HKQueryOptions = [.strictEndDate]
            if start != nil {
                options.insert(.strictStartDate)
            }
            let predicate = HKQuery.predicateForSamples(withStart: start, end: end, options: options)
            let sort = NSSortDescriptor(key: HKSampleSortIdentifierEndDate, ascending: false)
            let query = HKSampleQuery(
                sampleType: type,
                predicate: predicate,
                limit: page.limit,
                sortDescriptors: [sort]
            ) { _, samples, error in
                if let error {
                    continuation.resume(throwing: error)
                    return
                }
                continuation.resume(returning: samples ?? [])
            }
            store.execute(query)
        }
    }

    // MARK: - Background

    private func scheduleBackgroundTasks() {
        let processing = BGProcessingTaskRequest(identifier: Self.backfillTaskId)
        processing.requiresNetworkConnectivity = true
        processing.requiresExternalPower = false
        try? BGTaskScheduler.shared.submit(processing)

        let refresh = BGAppRefreshTaskRequest(identifier: Self.refreshTaskId)
        refresh.earliestBeginDate = Date(timeIntervalSinceNow: 15 * 60)
        try? BGTaskScheduler.shared.submit(refresh)
    }

    private func handleBackground(_ task: BGTask) async {
        scheduleBackgroundTasks()
        task.expirationHandler = {}
        do {
            guard isAutoSyncEnabled else {
                task.setTaskCompleted(success: true)
                return
            }
            try await requestAuthorization()
            try await pullAndFlush(rounds: 2)
            task.setTaskCompleted(success: true)
        } catch {
            task.setTaskCompleted(success: false)
        }
    }
}

enum DeviceSalt {
    static func supportDirectory() -> URL {
        let base = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSTemporaryDirectory())
        return base.appendingPathComponent("BioFocus", isDirectory: true)
    }

    static func loadOrCreate() -> Data {
        let directory = supportDirectory()
        let url = directory.appendingPathComponent("biofocus-device-salt")
        if let existing = try? Data(contentsOf: url), existing.count >= 16 {
            return existing
        }
        var bytes = [UInt8](repeating: 0, count: 32)
        _ = SecRandomCopyBytes(kSecRandomDefault, bytes.count, &bytes)
        let data = Data(bytes)
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        try? data.write(to: url, options: [.atomic, .completeFileProtection])
        return data
    }
}

enum AnchoredCursor {
    static func archive(_ anchor: HKQueryAnchor) -> String? {
        guard let data = try? NSKeyedArchiver.archivedData(withRootObject: anchor, requiringSecureCoding: true) else {
            return nil
        }
        return data.base64EncodedString()
    }

    static func unarchive(_ encoded: String) -> HKQueryAnchor? {
        guard let data = Data(base64Encoded: encoded) else { return nil }
        return try? NSKeyedUnarchiver.unarchivedObject(ofClass: HKQueryAnchor.self, from: data)
    }
}
