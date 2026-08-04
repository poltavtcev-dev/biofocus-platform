import Foundation

/// One-shot: HealthKit sample → JSON array → Desktop ingest.
/// Call from a button / debug action — do not poll in a tight loop.
enum SamplePost {
    static func postLatestHeartRate(
        baseURL: URL,
        token: String
    ) async throws {
        let observation = try await HeartRateSample.latestObservation()
        let encoder = JSONEncoder()
        let data = try encoder.encode([observation])
        let client = IngestClient(baseURL: baseURL, token: token)
        try await client.postObservations(data)
    }
}
