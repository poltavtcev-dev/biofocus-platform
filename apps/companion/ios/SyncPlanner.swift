import Foundation

struct TypeCursor: Codable, Equatable {
    var phase: String
    /// Next descending page keeps samples with `end` strictly below this instant.
    var cursorEnd: Int64
    var forwardAnchor: String?

    static func fresh(now: Int64) -> TypeCursor {
        TypeCursor(phase: "recent", cursorEnd: now + 1, forwardAnchor: nil)
    }
}

struct SyncState: Codable, Equatable {
    var installTime: Int64
    var historyDays: Int
    var types: [String: TypeCursor]

    static func fresh(now: Int64) -> SyncState {
        SyncState(installTime: now, historyDays: SyncPlanner.defaultHistoryDays, types: [:])
    }
}

struct BackfillPage: Equatable {
    var identifier: String
    var start: Int64?
    var endExclusive: Int64
    var limit: Int
}

struct DatedSample: Equatable {
    var id: String
    var start: Int64
    var end: Int64
}

enum SyncPlanner {
    static let pageSize = 500
    static let recentSeconds: Int64 = 30 * 24 * 60 * 60
    static let defaultHistoryDays = 365

    static func forwardCutoff(installTime: Int64) -> Int64 {
        installTime - recentSeconds
    }

    /// A sample written late still belongs on the forward channel when its start is inside the cutoff.
    static func includedInForward(sampleStart: Int64, cutoff: Int64) -> Bool {
        sampleStart >= cutoff
    }

    static func cursor(for identifier: String, state: SyncState, now: Int64) -> TypeCursor {
        state.types[identifier] ?? TypeCursor.fresh(now: now)
    }

    /// One newest-first page per type that still has backfill left, in registry priority.
    static func nextBackfillPages(now: Int64, state: SyncState) -> [BackfillPage] {
        HealthTypeRegistry.syncPriority.compactMap { identifier in
            let cursor = cursor(for: identifier, state: state, now: now)
            guard cursor.phase != "done" else { return nil }
            let start: Int64?
            if cursor.phase == "recent" {
                start = now - recentSeconds
            } else if state.historyDays > 0 {
                start = now - Int64(state.historyDays) * 24 * 60 * 60
            } else {
                start = nil
            }
            return BackfillPage(
                identifier: identifier,
                start: start,
                endExclusive: cursor.cursorEnd,
                limit: pageSize
            )
        }
    }

    /// Newest end dates first. This is what the first launch must send, not the oldest history.
    static func selectDescending(
        samples: [DatedSample],
        start: Int64?,
        endExclusive: Int64,
        limit: Int
    ) -> [DatedSample] {
        samples
            .filter { sample in
                if let start, sample.end < start {
                    return false
                }
                return sample.end < endExclusive
            }
            .sorted { lhs, rhs in
                if lhs.end != rhs.end {
                    return lhs.end > rhs.end
                }
                return lhs.id > rhs.id
            }
            .prefix(limit)
            .map { $0 }
    }

    static func advance(
        cursor: inout TypeCursor,
        now: Int64,
        historyDays: Int,
        returnedEnds: [Int64],
        pageLimit: Int
    ) {
        guard cursor.phase != "done" else { return }
        if returnedEnds.isEmpty || returnedEnds.count < pageLimit {
            if cursor.phase == "recent" {
                cursor.phase = "history"
                cursor.cursorEnd = now - recentSeconds
                return
            }
            cursor.phase = "done"
            return
        }
        let oldest = returnedEnds.min() ?? cursor.cursorEnd
        if oldest < cursor.cursorEnd {
            cursor.cursorEnd = oldest
        } else {
            cursor.cursorEnd = oldest - 1
        }
        _ = historyDays
    }
}

enum IngestBatches {
    static let maxCount = 500
    static let maxBytes = 1_048_576

    static func split(_ objects: [Data], maxCount: Int = maxCount, maxBytes: Int = maxBytes) -> [[Data]] {
        var batches: [[Data]] = []
        var current: [Data] = []
        var size = 2
        for object in objects {
            let extra = object.count + (current.isEmpty ? 0 : 1)
            let overflows = !current.isEmpty && (current.count >= maxCount || size + extra > maxBytes)
            if overflows {
                batches.append(current)
                current = [object]
                size = 2 + object.count
            } else {
                current.append(object)
                size += extra
            }
        }
        if !current.isEmpty {
            batches.append(current)
        }
        return batches
    }

    static func body(from objects: [Data]) -> Data {
        var data = Data("[".utf8)
        for (index, object) in objects.enumerated() {
            if index > 0 {
                data.append(contentsOf: Data(",".utf8))
            }
            data.append(object)
        }
        data.append(contentsOf: Data("]".utf8))
        return data
    }
}

enum ObserverFinish {
    /// HealthKit requires `completion` on every observer callback, including failures.
    static func callAlways(
        completion: @escaping () -> Void,
        work: () async throws -> Void
    ) async rethrows {
        defer { completion() }
        try await work()
    }
}

enum SyncStateStore {
    static func fileURL(in directory: URL) -> URL {
        directory.appendingPathComponent("biofocus-sync-state.json")
    }

    static func load(from url: URL, now: Int64) -> SyncState {
        guard let data = try? Data(contentsOf: url),
              let state = try? JSONDecoder().decode(SyncState.self, from: data)
        else {
            return SyncState.fresh(now: now)
        }
        return state
    }

    static func save(_ state: SyncState, to url: URL) {
        let directory = url.deletingLastPathComponent()
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        guard let data = try? JSONEncoder().encode(state) else { return }
        try? data.write(to: url, options: .atomic)
    }
}
