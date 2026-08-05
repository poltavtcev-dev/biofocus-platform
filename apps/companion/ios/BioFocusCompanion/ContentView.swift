import SwiftUI

/// Minimal dogfood UI: paste Desktop Companion base URL + token, send one HR sample.
struct ContentView: View {
    @AppStorage("biofocus.ingestBaseURL") private var baseURLText = "http://127.0.0.1:8787"
    @AppStorage("biofocus.pairingToken") private var token = ""

    @State private var statusMessage = "Idle — paste Base URL and token from Desktop Companion, then send once."
    @State private var isBusy = false
    @State private var lastWasError = false

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
                    Text("Copy Base URL and token from Desktop → Companion. Simulator: loopback. Phone: enable BIOFOCUS_INGEST_LAN=1 on Desktop, then use the LAN Base URL.")
                }

                Section {
                    Button {
                        Task { await sendOnce() }
                    } label: {
                        if isBusy {
                            ProgressView()
                                .frame(maxWidth: .infinity)
                        } else {
                            Text("Send one heart-rate sample")
                                .frame(maxWidth: .infinity)
                        }
                    }
                    .disabled(isBusy || baseURLText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                        || token.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }

                Section("Status") {
                    Text(statusMessage)
                        .foregroundStyle(lastWasError ? .red : .primary)
                        .font(.body)
                }
            }
            .navigationTitle("BioFocus Companion")
            .navigationBarTitleDisplayMode(.inline)
        }
    }

    @MainActor
    private func sendOnce() async {
        let trimmedURL = baseURLText.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedToken = token.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: trimmedURL), url.scheme == "http" || url.scheme == "https" else {
            lastWasError = true
            statusMessage = "Base URL looks invalid — use http://… from Desktop Companion."
            return
        }

        isBusy = true
        lastWasError = false
        statusMessage = "Reading one heart-rate sample…"
        defer { isBusy = false }

        do {
            try await SamplePost.postLatestHeartRate(baseURL: url, token: trimmedToken)
            lastWasError = false
            statusMessage = "Sample sent. Desktop ingest accepted the Observation."
        } catch let error as IngestClientError {
            lastWasError = true
            switch error {
            case .unauthorized:
                statusMessage = "Unauthorized (401) — check the pairing token from Desktop Companion."
            case .network(let underlying):
                statusMessage = "Network error — is Desktop running and reachable? \(underlying.localizedDescription)"
            case .http(let status, let body):
                let snippet = body.isEmpty ? "" : " \(body)"
                statusMessage = "Ingest HTTP \(status).\(snippet)"
            case .badResponse(let message):
                statusMessage = "Unexpected response: \(message)"
            }
        } catch {
            lastWasError = true
            statusMessage = error.localizedDescription
        }
    }
}

#Preview {
    ContentView()
}
