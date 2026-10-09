import Foundation

/// Durable pending Observation JSON objects (UserDefaults). Dedupe by stable key.
enum ObservationQueue {
    private static let storageKey = "biofocus.pendingObservations.v1"
    private static var cachedCount: Int?

    struct Entry: Codable, Equatable {
        let dedupeKey: String
        let jsonUTF8: Data
    }

    static func load() -> [Entry] {
        guard let data = UserDefaults.standard.data(forKey: storageKey) else {
            cachedCount = 0
            return []
        }
        let entries = (try? JSONDecoder().decode([Entry].self, from: data)) ?? []
        cachedCount = entries.count
        return entries
    }

    static func save(_ entries: [Entry]) {
        cachedCount = entries.count
        guard let data = try? JSONEncoder().encode(entries) else { return }
        UserDefaults.standard.set(data, forKey: storageKey)
    }

    static func enqueue(dedupeKey: String, observationJSON: Data) {
        enqueueMany([(dedupeKey: dedupeKey, observationJSON: observationJSON)])
    }

    /// One read and one write for a whole HealthKit page. Per-sample saves froze the phone.
    static func enqueueMany(_ items: [(dedupeKey: String, observationJSON: Data)]) {
        guard !items.isEmpty else { return }
        var entries = load()
        var seen = Set(entries.map(\.dedupeKey))
        var changed = false
        for item in items {
            if seen.contains(item.dedupeKey) {
                continue
            }
            seen.insert(item.dedupeKey)
            entries.append(Entry(dedupeKey: item.dedupeKey, jsonUTF8: item.observationJSON))
            changed = true
        }
        // Sync appends newest samples first. Keep that head and drop the tail.
        if entries.count > 8_000 {
            entries = Array(entries.prefix(8_000))
            changed = true
        }
        if changed {
            save(entries)
        }
    }

    static func enqueueEncodable<T: Encodable>(dedupeKey: String, _ value: T) throws {
        let data = try JSONEncoder().encode(value)
        enqueue(dedupeKey: dedupeKey, observationJSON: data)
    }

    static var pendingCount: Int {
        if let cachedCount { return cachedCount }
        return load().count
    }

    /// One ingest body, capped at 500 observations or 1 MB.
    static func nextBatch() -> (body: Data, keys: [String])? {
        let entries = load()
        guard !entries.isEmpty else { return nil }
        var chosen: [Data] = []
        var size = 2
        for entry in entries {
            let extra = entry.jsonUTF8.count + (chosen.isEmpty ? 0 : 1)
            if !chosen.isEmpty && (chosen.count >= IngestBatches.maxCount || size + extra > IngestBatches.maxBytes) {
                break
            }
            chosen.append(entry.jsonUTF8)
            size += extra
        }
        guard !chosen.isEmpty else { return nil }
        let keys = Array(entries.prefix(chosen.count).map(\.dedupeKey))
        return (IngestBatches.body(from: chosen), keys)
    }

    static func remove(keys: [String]) {
        let drop = Set(keys)
        save(load().filter { !drop.contains($0.dedupeKey) })
    }

    static func clearAll() {
        save([])
    }
}
