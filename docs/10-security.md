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

QR / copy UX for sharing the token with a companion device → **P2-E3-T2**. Host auto-start of ingest → shipped in **P2-E1-T4**.
