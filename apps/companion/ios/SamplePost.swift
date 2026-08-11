import Foundation

/// Manual / shared post helpers — prefer `HealthKitSyncCoordinator` for Auto-sync.
enum SamplePost {
    @MainActor
    static func postLatestHeartRate(
        baseURL: URL,
        token: String
    ) async throws {
        let sync = HealthKitSyncCoordinator.shared
        await sync.sendLatestNow(baseURL: baseURL, token: token)
        if sync.lastWasError {
            throw IngestClientError.badResponse(sync.lastStatus)
        }
    }
}
