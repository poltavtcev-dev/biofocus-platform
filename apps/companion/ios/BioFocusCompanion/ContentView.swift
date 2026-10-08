import SwiftUI

/// Pairing + Auto-sync (ADR-016 / ADR-018): wearable HealthKit → queue → Desktop ingest.
struct ContentView: View {
    @AppStorage("biofocus.ingestBaseURL") private var baseURLText = "http://127.0.0.1:8787"
    @AppStorage("biofocus.pairingToken") private var token = ""
    @AppStorage("biofocus.autoSync") private var autoSync = true

    @StateObject private var sync = HealthKitSyncCoordinator.shared
    @State private var isBusy = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Base URL", text: $baseURLText)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                    SecureField("Pairing token", text: $token)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                } header: {
                    Text("Desktop pairing")
                } footer: {
                    Text("Copy Base URL and token from Desktop → Companion. Simulator: loopback. Physical iPhone: enable LAN on Desktop, then use the LAN Base URL — not 127.0.0.1.")
                }

                Section {
                    Button("Test connection") {
                        Task { await testConnection() }
                    }
                    .disabled(isBusy || baseURLText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    Toggle("Auto-sync", isOn: $autoSync)
                        .onChange(of: autoSync) { _, enabled in
                            sync.isAutoSyncEnabled = enabled
                            if enabled {
                                Task { await sync.startAutoSyncIfNeeded() }
                            }
                        }
                    if let last = sync.lastFlushDate() {
                        Text("Last flush: \(last.formatted(date: .abbreviated, time: .shortened))")
                            .foregroundStyle(.secondary)
                    }
                    Text("Pending queue: \(sync.pendingCount)")
                        .foregroundStyle(.secondary)
                } header: {
                    Text("Autonomy")
                } footer: {
                    Text("When on, new HealthKit samples enqueue and flush when Desktop is reachable. Test connection checks reachability in under 5 seconds.")
                }

                Section {
                    Button {
                        Task { await sendNow() }
                    } label: {
                        if isBusy {
                            ProgressView()
                                .frame(maxWidth: .infinity)
                        } else {
                            Text("Синхронизировать сейчас")
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .disabled(isBusy || baseURLText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                        || token.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    Button("Flush queue only") {
                        Task { await flushOnly() }
                    }
                    .disabled(isBusy || sync.pendingCount == 0
                        || baseURLText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                        || token.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }

                Section("Данные Health") {
                    Text(sync.backfillLine)
                        .font(.subheadline)
                    ForEach(sync.typeRows) { row in
                        HStack {
                            Text(row.label)
                            Spacer()
                            Text(row.status)
                                .foregroundStyle(row.status == "OK" ? Color.green : Color.secondary)
                        }
                    }
                }

                Section("Status") {
                    Text(sync.lastStatus)
                        .foregroundStyle(sync.lastWasError ? .red : .primary)
                        .font(.body)
                }
            }
            .navigationTitle("BioFocus Companion")
            .navigationBarTitleDisplayMode(.inline)
            .task {
                sync.isAutoSyncEnabled = autoSync
                if autoSync {
                    await sync.startAutoSyncIfNeeded()
                }
                sync.refreshPendingCount()
                sync.refreshTypeBoard()
            }
        }
    }

    @MainActor
    private func testConnection() async {
        let trimmedURL = baseURLText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: trimmedURL), url.scheme == "http" || url.scheme == "https" else {
            sync.reportPairingError("Base URL looks invalid — use http://… from Desktop Companion.")
            return
        }
        isBusy = true
        defer { isBusy = false }
        await sync.testConnection(baseURL: url, token: token)
    }

    @MainActor
    private func sendNow() async {
        let trimmedURL = baseURLText.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedToken = token.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: trimmedURL), url.scheme == "http" || url.scheme == "https" else {
            sync.reportPairingError("Base URL looks invalid — use http://… from Desktop Companion.")
            return
        }
        isBusy = true
        defer { isBusy = false }
        do {
            try await sync.preflightConnection(baseURL: url, token: trimmedToken)
        } catch let err as IngestClientError {
            sync.reportPairingError(err.localizedDescription ?? "Connection check failed.")
            return
        } catch {
            sync.reportPairingError(error.localizedDescription)
            return
        }
        await sync.sendLatestNow(baseURL: url, token: trimmedToken)
    }

    @MainActor
    private func flushOnly() async {
        let trimmedURL = baseURLText.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedToken = token.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: trimmedURL), url.scheme == "http" || url.scheme == "https" else {
            sync.reportPairingError("Base URL looks invalid — use http://… from Desktop Companion.")
            return
        }
        isBusy = true
        defer { isBusy = false }
        do {
            try await sync.preflightConnection(baseURL: url, token: trimmedToken)
        } catch let err as IngestClientError {
            sync.reportPairingError(err.localizedDescription ?? "Connection check failed.")
            return
        } catch {
            sync.reportPairingError(error.localizedDescription)
            return
        }
        do {
            try await sync.flush(baseURL: url, token: trimmedToken)
        } catch {
            // Status already set in coordinator.
        }
    }
}
