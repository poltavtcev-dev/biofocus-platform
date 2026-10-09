import SwiftUI

@main
struct BioFocusCompanionApp: App {
    @StateObject private var sync = HealthKitSyncCoordinator.shared

    init() {
        HealthKitSyncCoordinator.shared.registerBackgroundTasks()
    }

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(sync)
                .task {
                    await sync.prepareAtLaunch()
                }
        }
    }
}
