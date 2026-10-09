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
    @Published private(set) var typeRows: [CompanionTypeRow] = []
    @Published private(set) var backfillLine: String = "Синхронизация ещё не начиналась"

    private let store = HKHealthStore()
    private var observersStarted = false
    private var backgroundRegistered = false
    private var retryScheduled = false
    private var flushFailures = 0
    private var startTask: Task<Void, Never>?
    private var pullInFlight = false
    private var pullFollowUp = false
    private var followUpBackfill = false
    private var followUpAllTypes = false
    private var followUpBaseURL: URL?
    private var followUpToken: String?
    private var followUpTypes: Set<String>?
    private var coalescedPull: Task<Void, Never>?
    private var dirtyTypes: Set<String> = []

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
        if observersStarted {
            try? await pullAndFlush(rounds: 1)
            return
        }
        if let startTask {
            await startTask.value
            return
        }
        let task = Task { @MainActor in
            await self.beginAutoSync()
        }
        startTask = task
        await task.value
    }

    private func beginAutoSync() async {
        do {
            try await requestAuthorization()
            try await enableBackgroundDelivery()
            startObservers()
            try await pullAndFlush(rounds: 3)
        } catch {
            startTask = nil
            lastWasError = true
            lastStatus = "Автосинхронизация: \(error.localizedDescription)"
        }
    }

    /// Manual sync uses the same newest-first pull as launch, not a single latest sample.
    func sendLatestNow(baseURL: URL, token: String) async {
        lastWasError = false
        lastStatus = "Читаем Health…"
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
            lastStatus = sent == 0 ? "В очереди ничего нет." : "Отправлено: \(sent)."
        } catch {
            lastWasError = true
            lastStatus = "\(error.localizedDescription) — осталось в очереди (\(ObservationQueue.pendingCount))."
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
        lastStatus = "Проверяем связь…"
        do {
            let client = IngestClient(baseURL: baseURL, token: token)
            let status = try await client.fetchStatus()
            lastWasError = false
            lastStatus = "Связь есть. Режим: \(status.bind_mode), база: \(status.db_status)."
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

    private func pullAndFlush(
        rounds: Int,
        includeBackfill: Bool = true,
        only types: Set<String>? = nil,
        baseURL: URL? = nil,
        token: String? = nil
    ) async throws {
        if pullInFlight {
            pullFollowUp = true
            if includeBackfill {
                followUpBackfill = true
            }
            if let baseURL { followUpBaseURL = baseURL }
            if let token { followUpToken = token }
            if let types, !followUpAllTypes {
                followUpTypes = followUpTypes.map { $0.union(types) } ?? types
            } else if types == nil {
                followUpAllTypes = true
                followUpTypes = nil
            }
            return
        }
        pullInFlight = true
        defer { pullInFlight = false }
        var nextRounds = rounds
        var nextBackfill = includeBackfill
        var nextTypes = types
        var nextBase = baseURL
        var nextToken = token
        while true {
            try await performPull(
                rounds: nextRounds,
                includeBackfill: nextBackfill,
                only: nextTypes,
                baseURL: nextBase,
                token: nextToken
            )
            guard pullFollowUp else { return }
            pullFollowUp = false
            nextRounds = 1
            nextBackfill = followUpBackfill
            nextTypes = followUpAllTypes ? nil : followUpTypes
            nextBase = followUpBaseURL ?? nextBase
            nextToken = followUpToken ?? nextToken
            followUpBackfill = false
            followUpAllTypes = false
            followUpTypes = nil
            followUpBaseURL = nil
            followUpToken = nil
        }
    }

    private func performPull(
        rounds: Int,
        includeBackfill: Bool,
        only types: Set<String>?,
        baseURL: URL?,
        token: String?
    ) async throws {
        let salt = DeviceSalt.loadOrCreate()
        do {
            for _ in 0..<rounds {
                let progressed = try await pullOneRound(salt: salt, includeBackfill: includeBackfill, only: types)
                if !progressed {
                    break
                }
            }
        } catch {
            if Self.isAuthDenied(error) {
                recordAuthDenied()
            }
            throw error
        }
        refreshPendingCount()
        await publishProgress(baseURL: baseURL, token: token)
        let base = baseURL ?? configuredBaseURL()
        let auth = token ?? (UserDefaults.standard.string(forKey: "biofocus.pairingToken") ?? "")
        guard let base, !auth.isEmpty else {
            lastWasError = false
            lastStatus = "Автосинхронизация включена. Укажите адрес и токен, чтобы отправить. В очереди \(ObservationQueue.pendingCount)."
            return
        }
        if ObservationQueue.pendingCount > 0 {
            try await flush(baseURL: base, token: auth)
        } else if !lastWasError {
            lastStatus = "За недавнее окно Health уже актуален."
        }
    }

    private func pullOneRound(salt: Data, includeBackfill: Bool, only types: Set<String>?) async throws -> Bool {
        let now = Int64(Date().timeIntervalSince1970)
        let url = SyncStateStore.fileURL(in: DeviceSalt.supportDirectory())
        var state = SyncStateStore.load(from: url, now: now)
        if state.installTime == 0 {
            state.installTime = now
        }
        var progressed = false
        var marks = marks()
        let cutoff = Date(timeIntervalSince1970: TimeInterval(SyncPlanner.forwardCutoff(installTime: state.installTime)))
        let forwardPredicate = HKQuery.predicateForSamples(withStart: cutoff, end: nil, options: .strictStartDate)

        for identifier in HealthTypeRegistry.syncPriority {
            if let types, !types.contains(identifier) { continue }
            guard let sampleType = HealthTypeRegistry.sampleType(for: identifier) else { continue }
            var cursor = SyncPlanner.cursor(for: identifier, state: state, now: now)
            let anchor = cursor.forwardAnchor.flatMap(AnchoredCursor.unarchive)
            let forward = try await anchoredPage(type: sampleType, predicate: forwardPredicate, anchor: anchor)
            if !forward.samples.isEmpty || !forward.deleted.isEmpty {
                progressed = true
            }
            var queued: [(dedupeKey: String, observationJSON: Data)] = []
            queued.reserveCapacity(forward.samples.count + forward.deleted.count)
            for sample in forward.samples {
                if let item = queuedObservation(sample: sample, identifier: identifier, salt: salt) {
                    queued.append(item)
                }
            }
            for deleted in forward.deleted {
                if let item = queuedDeletion(id: deleted.uuid.uuidString, now: now) {
                    queued.append(item)
                }
            }
            if let newAnchor = forward.anchor {
                cursor.forwardAnchor = AnchoredCursor.archive(newAnchor)
            }

            var backfillCount = 0
            if includeBackfill && cursor.phase != "done" {
                let page = BackfillPage(
                    identifier: identifier,
                    start: cursor.phase == "recent"
                        ? now - SyncPlanner.recentSeconds
                        : (state.historyDays > 0 ? now - Int64(state.historyDays) * 86_400 : nil),
                    endExclusive: cursor.cursorEnd,
                    limit: SyncPlanner.pageSize
                )
                let samples = try await descendingPage(type: sampleType, page: page)
                backfillCount = samples.count
                if !samples.isEmpty {
                    progressed = true
                }
                for sample in samples {
                    if let item = queuedObservation(sample: sample, identifier: identifier, salt: salt) {
                        queued.append(item)
                    }
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
            ObservationQueue.enqueueMany(queued)
            let hadSamples = !forward.samples.isEmpty || backfillCount > 0
            let next = CompanionStatusBoard.note(hadSamples: hadSamples, authDenied: false)
            marks[identifier] = CompanionStatusBoard.merge(previous: marks[identifier], next: next)
            await Task.yield()
        }
        setMarks(marks)
        SyncStateStore.save(state, to: url)
        return progressed
    }

    private func queuedObservation(sample: HKSample, identifier: String, salt: Data) -> (dedupeKey: String, observationJSON: Data)? {
        let record: HealthRecord
        if let quantity = sample as? HKQuantitySample {
            record = HKRecordAdapter.quantity(quantity, identifier: identifier)
        } else if let category = sample as? HKCategorySample {
            record = HKRecordAdapter.category(category, identifier: identifier)
        } else if let workout = sample as? HKWorkout {
            record = HKRecordAdapter.workout(workout)
        } else {
            return nil
        }
        guard let data = WearableMapper.jsonData(for: record, salt: salt) else { return nil }
        return (record.uuid.lowercased(), data)
    }

    private func queuedDeletion(id: String, now: Int64) -> (dedupeKey: String, observationJSON: Data)? {
        guard let data = WearableMapper.deletionJSON(targetId: id, timestamp: now) else { return nil }
        return ("del-\(id.lowercased())", data)
    }

    func refreshTypeBoard() {
        let stored = marks()
        typeRows = CompanionStatusBoard.rows(
            identifiers: HealthTypeRegistry.syncPriority,
            marks: stored
        )
        let now = Int64(Date().timeIntervalSince1970)
        let url = SyncStateStore.fileURL(in: DeviceSalt.supportDirectory())
        let state = SyncStateStore.load(from: url, now: now)
        var cursors: [String: String] = [:]
        for identifier in HealthTypeRegistry.syncPriority {
            cursors[identifier] = state.types[identifier]?.phase ?? "recent"
        }
        let done = cursors.values.filter { $0 == "done" }.count
        backfillLine = "Прогресс: \(Self.phaseLine(CompanionStatusBoard.phase(cursors: cursors))). Готово \(done) из \(HealthTypeRegistry.syncPriority.count)."
    }

    private func publishProgress(baseURL: URL?, token: String?) async {
        refreshTypeBoard()
        let base = baseURL ?? configuredBaseURL()
        let auth = token ?? (UserDefaults.standard.string(forKey: "biofocus.pairingToken") ?? "")
        guard let base, !auth.isEmpty else { return }
        let now = Int64(Date().timeIntervalSince1970)
        let url = SyncStateStore.fileURL(in: DeviceSalt.supportDirectory())
        let state = SyncStateStore.load(from: url, now: now)
        var cursors: [String: String] = [:]
        for identifier in HealthTypeRegistry.syncPriority {
            if let phase = state.types[identifier]?.phase {
                cursors[identifier] = phase
            }
        }
        let object = CompanionStatusBoard.payload(
            identifiers: HealthTypeRegistry.syncPriority,
            marks: marks(),
            cursors: cursors,
            pending: ObservationQueue.pendingCount
        )
        guard let data = try? JSONSerialization.data(withJSONObject: object, options: [.sortedKeys]) else {
            return
        }
        let client = IngestClient(baseURL: base, token: auth)
        try? await client.postCompanionStatus(data)
    }

    private func marks() -> [String: String] {
        UserDefaults.standard.dictionary(forKey: "biofocus.typeMarks") as? [String: String] ?? [:]
    }

    private func setMarks(_ next: [String: String]) {
        UserDefaults.standard.set(next, forKey: "biofocus.typeMarks")
        refreshTypeBoard()
    }

    private func recordAuthDenied() {
        var stored = marks()
        for identifier in HealthTypeRegistry.syncPriority {
            stored[identifier] = TypeAvailability.noPermission
        }
        setMarks(stored)
    }

    private static func phaseLine(_ phase: String) -> String {
        switch phase {
        case "recent": return "сначала последние 30 дней"
        case "history": return "история"
        case "done": return "окно дочитано"
        default: return "ожидание"
        }
    }

    private static func isAuthDenied(_ error: Error) -> Bool {
        let ns = error as NSError
        guard ns.domain == HKError.errorDomain else { return false }
        return ns.code == HKError.Code.errorAuthorizationDenied.rawValue
            || ns.code == HKError.Code.errorAuthorizationNotDetermined.rawValue
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
                completion()
                let failed = error
                Task { @MainActor in
                    if let failed {
                        self.lastWasError = true
                        self.lastStatus = "Наблюдатель \(identifier): \(failed.localizedDescription)"
                        return
                    }
                    self.scheduleCoalescedPull(identifier: identifier)
                }
            }
            store.execute(query)
        }
    }

    /// Heart-rate delivery is immediate. Fold a burst into one forward pull of the types that changed.
    private func scheduleCoalescedPull(identifier: String) {
        guard isAutoSyncEnabled else { return }
        dirtyTypes.insert(identifier)
        guard coalescedPull == nil else { return }
        coalescedPull = Task { @MainActor in
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            self.coalescedPull = nil
            let types = self.dirtyTypes
            self.dirtyTypes = []
            guard self.isAutoSyncEnabled, !types.isEmpty, !Task.isCancelled else { return }
            try? await self.pullAndFlush(rounds: 1, includeBackfill: false, only: types)
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
