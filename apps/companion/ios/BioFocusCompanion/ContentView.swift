import SwiftUI

/// Pairing + Auto-sync (ADR-016 / ADR-018): wearable HealthKit → queue → Desktop ingest.
struct ContentView: View {
    @AppStorage("biofocus.ingestBaseURL") private var baseURLText = "http://127.0.0.1:8787"
    @AppStorage("biofocus.pairingToken") private var token = ""
    @AppStorage("biofocus.certPin") private var certPin = ""
    @AppStorage("biofocus.autoSync") private var autoSync = true
    @State private var pairingPaste = ""

    @StateObject private var sync = HealthKitSyncCoordinator.shared
    @State private var isBusy = false

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Адрес", text: $baseURLText)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                    SecureField("Токен", text: $token)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    TextField("Отпечаток сертификата", text: $certPin)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    TextField("Текст QR", text: $pairingPaste, axis: .vertical)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    Button("Применить текст QR") {
                        applyPairingText()
                    }
                    .disabled(pairingPaste.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                } header: {
                    Text("Подключение к Mac")
                } footer: {
                    Text("Симулятор: http://127.0.0.1:8787. Настоящий iPhone берёт адрес https из BioFocus на Mac. Вставьте текст QR — четыре строки. Чужой отпечаток сертификата отклоняется.")
                }

                Section {
                    Button("Проверить связь") {
                        Task { await testConnection() }
                    }
                    .disabled(isBusy || baseURLText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                    Toggle("Автосинхронизация", isOn: $autoSync)
                        .onChange(of: autoSync) { _, enabled in
                            sync.isAutoSyncEnabled = enabled
                            if enabled {
                                Task { await sync.startAutoSyncIfNeeded() }
                            }
                        }
                    if let last = sync.lastFlushDate() {
                        Text("Последняя отправка: \(last.formatted(date: .abbreviated, time: .shortened))")
                            .foregroundStyle(.secondary)
                    }
                    Text("В очереди: \(sync.pendingCount)")
                        .foregroundStyle(.secondary)
                } header: {
                    Text("Автономность")
                } footer: {
                    Text("Когда включено, новые данные Health попадают в очередь и уходят на Mac, как только он доступен. «Проверить связь» ждёт ответ не дольше 5 секунд.")
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

                    Button("Только отправить очередь") {
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
                                .foregroundStyle(row.status == "есть" ? Color.green : Color.secondary)
                        }
                    }
                }

                Section("Статус") {
                    Text(sync.lastStatus)
                        .foregroundStyle(sync.lastWasError ? .red : .primary)
                        .font(.body)
                }
            }
            .navigationTitle("BioFocus Companion")
            .navigationBarTitleDisplayMode(.inline)
            .task {
                sync.isAutoSyncEnabled = autoSync
                sync.refreshPendingCount()
                sync.refreshTypeBoard()
            }
        }
    }

    private func applyPairingText() {
        guard let parsed = PairingQr.parse(pairingPaste) else {
            sync.reportPairingError("Текст QR — четыре строки, первая biofocus:1.")
            return
        }
        baseURLText = parsed.url
        token = parsed.token
        certPin = parsed.pin ?? ""
        pairingPaste = ""
    }

    @MainActor
    private func testConnection() async {
        let trimmedURL = baseURLText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let url = URL(string: trimmedURL), url.scheme == "http" || url.scheme == "https" else {
            sync.reportPairingError("Адрес не похож на ссылку. Возьмите http:// или https:// из BioFocus на Mac.")
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
            sync.reportPairingError("Адрес не похож на ссылку. Возьмите http:// или https:// из BioFocus на Mac.")
            return
        }
        isBusy = true
        defer { isBusy = false }
        do {
            try await sync.preflightConnection(baseURL: url, token: trimmedToken)
        } catch let err as IngestClientError {
            sync.reportPairingError(err.localizedDescription ?? "Проверка связи не удалась.")
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
            sync.reportPairingError("Адрес не похож на ссылку. Возьмите http:// или https:// из BioFocus на Mac.")
            return
        }
        isBusy = true
        defer { isBusy = false }
        do {
            try await sync.preflightConnection(baseURL: url, token: trimmedToken)
        } catch let err as IngestClientError {
            sync.reportPairingError(err.localizedDescription ?? "Проверка связи не удалась.")
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
