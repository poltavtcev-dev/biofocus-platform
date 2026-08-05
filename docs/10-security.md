# 10. Security, Privacy & Local Isolation

## 1. Rules
1. **Network Binding:** Встроенный сервер (Axum) слушает только локальные интерфейсы (`127.0.0.1` или назначенный IP локальной сети Wi-Fi при сопряжении с телефоном).
2. **Pairing Token:** Доступ к `/v1/ingest` разрешён только по секретному Bearer-токену.
3. **No External Telemetry:** Отправка анонимной аналитики или телеметрии по умолчанию отключена.

## 2. Pairing token (local secret)

| Item | Value |
| :--- | :--- |
| **Path** | `~/.biofocus/pairing_token` |
| **Override home (tests/dev)** | `$BIOFOCUS_HOME/pairing_token` when `BIOFOCUS_HOME` is set |
| **Env override** | `BIOFOCUS_INGEST_TOKEN` — if non-empty, used instead of the file (does not rewrite disk) |
| **Generation** | On first resolve (`IngestConfig::load`): 32 random bytes → 64 hex chars (`getrandom`) |
| **Permissions** | Written `0600` on Unix (via exclusive create of temp file + rename) |
| **Git** | Never commit; lives outside the repo under the user home. `.gitignore` also ignores `.biofocus/` and `pairing_token` if copied into the tree |

QR / copy UX for sharing the token with a companion device → shipped in **P2-E3-T2** (Desktop IPC `get_pairing_token` + shell Show/Copy/QR). Host auto-start of ingest → shipped in **P2-E1-T4**.

Token remains a **local secret**: no cloud account; UI receives the value only via Tauri IPC (never reads the token file from the frontend).

### Companion client (P2-E3-T1)
`apps/companion` posts Observations with Bearer auth. Wrong token → explicit unauthorized (CLI exit `3` / Swift `IngestClientError.unauthorized`). Sample path is local-only; no cloud.

## 3. Context collector privacy

### Active window (P2-E2-T1)
Active window collector records **app identity only** (`bundle_id`, `app_name`) via `NSWorkspace`. It does **not** capture window titles, keystrokes, clipboard, or screenshots. No Accessibility permission is required for T1.

### Input aggregates (P2-E2-T2)
Keystroke collector is **opt-in** (`BIOFOCUS_INPUT_AGGREGATES=1`, default off). When enabled it uses a **listen-only** `CGEventTap` that increments a counter on key-down — **no characters or key codes are stored**. Payload: `{count, window_secs, rate_per_min}` only.

| Item | Value |
| :--- | :--- |
| Permission | macOS **Accessibility** (`AXIsProcessTrusted`) |
| Deny / unavailable | No panic; collector idles with count=0 (no Observations) |
| Disable | Unset/remove env flag (or set to `0`) and restart Desktop |

Grant Accessibility to the BioFocus app (System Settings → Privacy & Security → Accessibility) before enabling the flag if you want live aggregates.

### Optional local LLM reports (P4-E3-T2)
Local LLM interpret is **opt-in** (`BIOFOCUS_LOCAL_LLM=1`, default off). When disabled, `report-engine` opens no sockets for LLM. When enabled, only `ReportDocument::llm_prompt` is POSTed to a user-configured OpenAI-compatible endpoint (default localhost Ollama). Not invoked on startup — explicit host call only. Prefer `127.0.0.1`; pointing the base URL off-machine is operator-controlled. See `docs/12-development.md`.
