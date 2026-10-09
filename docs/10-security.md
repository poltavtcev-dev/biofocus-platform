# 10. Security, Privacy & Local Isolation

## 1. Rules
1. **Network Binding:** Встроенный сервер (Axum) по умолчанию слушает только loopback (`127.0.0.1`). LAN-reachable bind — **явный opt-in** (см. §1.1). Даже при LAN bind доступ к `/v1/ingest` только с Bearer pairing token.
2. **Pairing Token:** Доступ к `/v1/ingest` разрешён только по секретному Bearer-токену.
3. **No External Telemetry:** Отправка анонимной аналитики или телеметрии по умолчанию отключена.

### 1.1 Ingest bind (opt-in LAN)

| Knob | Default | Effect |
| :--- | :--- | :--- |
| *(none)* | `127.0.0.1:8787` | Same-machine / Simulator path; unchanged from Phase 2 |
| `BIOFOCUS_INGEST_LAN=1` | off | Bind `0.0.0.0:<port>` so a phone on the same Wi-Fi can reach Desktop ingest. Truthy: `1` / `true` / `yes` / `on` (case-insensitive) |
| `BIOFOCUS_INGEST_BIND_HOST=<ipv4>` | unset | Explicit IPv4 bind override (e.g. `0.0.0.0` or a NIC address). **Wins over** `BIOFOCUS_INGEST_LAN` |

**ADR-005:** opt-in LAN for companion dogfood; default remains loopback. Restart Desktop after changing knobs. Bind mode + base URL hints are exposed via `GET /v1/status` and IPC `get_pairing_token` (no Observation / DB paths / tokens in status). Companion UI shows copyable Base URL + token/QR (P5-E2-T1). No anonymous ingest on LAN — Bearer still required.

**ADR-031 (wearables P7):** loopback stays plain HTTP. A non-loopback bind serves TLS with a self-signed certificate stored as `ingest_cert.pem` / `ingest_key.pem` under `~/.biofocus/` (key mode `0600`). The fingerprint is SHA-256 of the certificate DER. The pairing QR is four lines (`biofocus:1`, URL, token, fingerprint). The companion pins that fingerprint and rejects a mismatch. `rotate_pairing_token` writes a new bearer and the running server drops the old one. The certificate is not rotated with the token.

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

### Wearable depth + chart ranges (ADR-018 / Phase 17)
Phase 17 expands Companion HealthKit Observations Mi Fitness writes into Apple Health (`step_count` / `active_energy` / `sleep_interval`; soft-optional `oxygen_saturation`; keep `heart_rate` + soft-optional `hrv`). **HealthKit only** — no Mi Cloud / unofficial API. Same local queue → Desktop ingest (ADR-016); no busy-loop HK poll; no clinical SpO2/sleep claims. Dashboard long ranges use **recompute-on-read** Feature series (`get_feature_series`) — **no** Feature-history SQLite in v1; UI ↛ SQLite; Snapshot lists show **latest** Features. Contracts locked; Companion/UI implementation → E2/E3.

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

### Local Calendar / ICS (P6-E3-T1)
Calendar collector is **opt-in** (`BIOFOCUS_CALENDAR=1`, default off) and reads a **local** `.ics` file (`BIOFOCUS_CALENDAR_ICS`). No Google/Outlook OAuth. Payload stores schedule metadata only (`uid`, `start`, `end`, `all_day`, `busy`) — **not** titles, descriptions, locations, or attendees. Logs on channel pressure use Observation `id` only.

| Item | Value |
| :--- | :--- |
| Source | Local ICS file path |
| Poll | Rare (≥60s); emit-once per `(uid,start,end)` |
| Deny / missing path | Soft-fail: collector not started (warn log); no panic |
| Disable | Unset env flags and restart Desktop |

### Browser categories (P10-E2-T1 / ADR-010)
Browser category collector is **opt-in** (`BIOFOCUS_BROWSER_CATEGORIES=1`, default off). Payload stores a **coarse** `category` label (+ optional `browser_bundle_id`) only — **never** full URLs, page titles, form content, or keystrokes. Personal self-tracking only — not workplace monitoring. Logs on channel pressure use Observation `id` only.

| Item | Value |
| :--- | :--- |
| Source | Frontmost known-browser detection (v1 OS probe emits `unknown` without URL mapping); mocks supply closed-set labels in tests |
| Poll | ≥5s; emit on category identity change |
| Deny / unavailable | Soft-fail idle (`None`); no panic |
| Disable | Unset env flag and restart Desktop |

### Optional local LLM reports (P4-E3-T2)
Local LLM interpret is **opt-in** (`BIOFOCUS_LOCAL_LLM=1`, default off). When disabled, `report-engine` opens no sockets for LLM. When enabled, only `ReportDocument::llm_prompt` is POSTed to a user-configured OpenAI-compatible endpoint (default localhost Ollama). Not invoked on startup — explicit host call only. Prefer `127.0.0.1`; pointing the base URL off-machine is operator-controlled. See `docs/12-development.md`.

### Prompt packs + coaching polish (ADR-011 / P11-E2)
Prompt packs are **in-process** templates over already-computed Evidence — they do not open SQLite or network. Default pack `biofocus.default` @ `1` (`build_report_with_pack`) only formats Features / Insights / Recommendations; `llm_prompt` forbids inventing scores / Evidence / Recommendations. Provider UX (P11-E3) surfaces calm local-LLM status via `get_local_llm_status` (config-only; no secrets in UI logs) and pack-aware Generate report; default stays **off**; no auto-invoke on Dashboard open; no chat-history persistence in v1. LLM stays interpret-only. Cloud LLM marketplace is out of Phase 11.

### Ambient Now Playing + packaging (ADR-012 / P12-E2)
Ambient wave-1 is opt-in **Now Playing** Observations (`data_type: "now_playing"`; coarse `media_kind` + `is_playing` only). Default **off** (`BIOFOCUS_NOW_PLAYING`). No song titles / lyrics / playlists, no always-on mic, no precise home geolocation dumps. Persist via existing `observations` store — **no** sync product and **no** migration in v1. Commercial packaging in Phase 12 is signed-build / notarization / update **runbook** ([`docs/18-packaging-runbook.md`](18-packaging-runbook.md)) with optional sync **stance** off by default — AGPLv3 Core stays open; packaging ≠ secret Feature math. Weather deferred; light → ADR-015 / Phase 15.

### Ambient light (ADR-015 / P16-E1–E2 shipped)
ADR-015 locks opt-in **ambient light** Observations (`data_type: "ambient_light"`; coarse `light_kind` + optional bounded `level` 0–100). Default **off** (`BIOFOCUS_AMBIENT_LIGHT`). No camera frames, screen contents, precise geolocation, or always-on mic. No cloud light / weather telemetry. Persist via existing `observations` store — **no** migration in v1. Weather ambient remains deferred. Personal self-tracking only. Collector **shipped** (P16-E1); Feature `AmbientLightShare` **shipped** (P16-E2).

### Notification events (ADR-019 · ADR-020 live probe shipped)
ADR-019 locks opt-in **notification** Observations (`data_type: "notification_event"`; required coarse `count` ≥ 1; optional closed-set `category` / `interruption_level` / `app_kind`). Default **off** (`BIOFOCUS_NOTIFICATION_EVENTS`). **Never** store notification body, title, subtitle, message text, screenshots, attachments, or userInfo dumps. Persist via existing `observations` store — **no** migration in v1. Personal self-tracking only — **not** workplace / employer monitoring. Collector + Feature **shipped** (P18). **ADR-020 / P19-E2:** live `SystemNotificationEventProbe` reads **usernoted** NC SQLite allowlisted columns only (`delivered_date` + `app.identifier`); never `record.data` / content fields; soft-fail when unavailable or TCC denied. Payload unchanged; no content capture even transiently for classification. **Dogfood (P19-E3):** Full Disk Access may be required for `group.com.apple.usernoted`; without it expect idle soft-fail — see `docs/12-development.md` § Notification events dogfood.

### Plugin wave-2 / Git activity (ADR-013 / P13-E2)
Wave-2 is opt-in **Git activity** Observations (`data_type: "git_activity"`; coarse `activity_kind` + optional `event_count` only). Default **off** (`BIOFOCUS_GIT_ACTIVITY`). No repo paths, remotes, branch names, diffs, commit messages, or buffer/keystroke content. Persist via existing `observations` store — **no** migration in v1. IDE session capture deferred (no additive privacy-safe signal beyond `context_window` for v1). Personal self-tracking only — not workplace monitoring. Collector **shipped** (P13-E2); Feature `GitActivityRate` **shipped** (P13-E3).

### Git watched-roots allowlist (ADR-014 / P14-E2–E3)
Live probe roots are **user-chosen absolute paths** in local config `~/.biofocus/git-watched-roots.toml` (ADR-014). When the file is **absent**, optional `BIOFOCUS_GIT_WATCHED_ROOTS` (colon/comma-separated) may supply roots for tests/CI — **file is SoT when present**. Menubar **Git folders** edits the file via IPC (`get_git_watched_roots` / `set_git_watched_roots`) — UI ↛ SQLite; paths may appear on that Settings IPC only. Allowlist is configuration — **never** widen Observation payloads with those paths (or remotes / branch / SHA / message / diff / author). Prefer logs with Observation `id` / counts (`root_count` / `repo_count`); do not dump allowlist roots or discovered repos at default levels. Empty/missing allowlist → soft-fail idle (no whole-disk scan). **No** SQLite allowlist table / **no** migration. Personal self-tracking only — not workplace monitoring. Live probe **shipped** (P14-E2); Settings/dogfood **shipped** (P14-E3). Dogfood: `docs/12-development.md` § Git activity dogfood.