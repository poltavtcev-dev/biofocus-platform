import SwiftUI

@main
struct BioFocusCompanionApp: App {
    @StateObject private var sync = HealthKitSyncCoordinator.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(sync)
                .task {
                    if sync.isAutoSyncEnabled {
                        await sync.startAutoSyncIfNeeded()
                    }
                }
        }
    }
}
