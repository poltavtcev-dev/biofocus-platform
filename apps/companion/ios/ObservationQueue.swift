import Foundation

/// Durable pending Observation JSON objects (UserDefaults). Dedupe by stable key.
enum ObservationQueue {
    private static let storageKey = "biofocus.pendingObservations.v1"

    struct Entry: Codable, Equatable {
        let dedupeKey: String
        let jsonUTF8: Data
    }

    static func load() -> [Entry] {
        guard let data = UserDefaults.standard.data(forKey: storageKey) else { return [] }
        return (try? JSONDecoder().decode([Entry].self, from: data)) ?? []
    }

    static func save(_ entries: [Entry]) {
        guard let data = try? JSONEncoder().encode(entries) else { return }
        UserDefaults.standard.set(data, forKey: storageKey)
    }

    static func enqueue(dedupeKey: String, observationJSON: Data) {
        var entries = load()
        if entries.contains(where: { $0.dedupeKey == dedupeKey }) {
            return
        }
        entries.append(Entry(dedupeKey: dedupeKey, jsonUTF8: observationJSON))
        // Sync appends newest samples first. Keep that head and drop the tail.
        if entries.count > 8_000 {
            entries = Array(entries.prefix(8_000))
        }
        save(entries)
    }

    static func enqueueEncodable<T: Encodable>(dedupeKey: String, _ value: T) throws {
        let data = try JSONEncoder().encode(value)
        enqueue(dedupeKey: dedupeKey, observationJSON: data)
    }

    static var pendingCount: Int { load().count }

    /// One ingest body, capped at 500 observations or 1 MB.
    static func nextBatch() -> (body: Data, keys: [String])? {
        let entries = load()
        guard !entries.isEmpty else { return nil }
        let objects = entries.map(\.jsonUTF8)
        guard let first = IngestBatches.split(objects).first else { return nil }
        let keys = Array(entries.prefix(first.count).map(\.dedupeKey))
        return (IngestBatches.body(from: first), keys)
    }

    static func remove(keys: [String]) {
        let drop = Set(keys)
        save(load().filter { !drop.contains($0.dedupeKey) })
    }

    static func clearAll() {
        save([])
    }
}
