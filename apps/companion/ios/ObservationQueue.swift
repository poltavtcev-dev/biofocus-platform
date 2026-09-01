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
        // Cap queue to avoid unbounded growth offline.
        if entries.count > 200 {
            entries = Array(entries.suffix(200))
        }
        save(entries)
    }

    static func enqueueEncodable<T: Encodable>(dedupeKey: String, _ value: T) throws {
        let data = try JSONEncoder().encode(value)
        enqueue(dedupeKey: dedupeKey, observationJSON: data)
    }

    static var pendingCount: Int { load().count }

    /// Build JSON array body for ingest; returns nil if empty.
    static func encodeBatchBody() -> Data? {
        let entries = load()
        guard !entries.isEmpty else { return nil }
        var chunks: [String] = []
        chunks.reserveCapacity(entries.count)
        for e in entries {
            guard let s = String(data: e.jsonUTF8, encoding: .utf8) else { continue }
            chunks.append(s)
        }
        guard !chunks.isEmpty else { return nil }
        let joined = "[" + chunks.joined(separator: ",") + "]"
        return joined.data(using: .utf8)
    }

    static func clearAll() {
        save([])
    }
}
