# Dev|UX → QA: P28-E1-T1

## Meta
- **Task ID:** P28-E1-T1
- **Title:** Companion LAN connectivity — fail-fast probe, clear errors, operator path
- **Role that built:** Dev + UX
- **Date:** 2026-09-01
- **AC source:** `docs/handoffs/P28-E1-T1-pm-brief.md` · ADR-029

## What changed
- **iOS `IngestClient.swift`:** explicit timeouts (5s status / 10s POST); `GET /v1/status`; loopback guard on physical device; calm errors (timeout / unreachable / unauthorized).
- **iOS `ContentView` + `HealthKitSyncCoordinator`:** **Test connection** button; preflight before Send/Flush.
- **Desktop Companion UX (`App.tsx`, `pairing.ts`, `App.css`):** loopback warning; LAN address error state; Copy URL disabled until LAN URL usable; **Enable LAN ingest** checkbox.
- **Desktop IPC (`lib.rs`):** `get_ingest_lan_preference` / `set_ingest_lan_preference`.
- **Ingest crate:** persisted LAN opt-in `~/.biofocus/ingest_lan_enabled`; `resolve_bind_host` reads it after env knobs.
- **Rust companion:** `get_status`, `classify_network_error`, timeout clients; tests extended.
- **Docs:** `docs/12-development.md` troubleshooting table; `apps/companion/ios/README.md` mirror.

## Files / crates
- `apps/companion/ios/IngestClient.swift`
- `apps/companion/ios/BioFocusCompanion/ContentView.swift`
- `apps/companion/ios/HealthKitSyncCoordinator.swift`
- `apps/companion/src/lib.rs`
- `apps/companion/tests/sample_ingest.rs`
- `apps/desktop/src/App.tsx`, `App.css`, `pairing.ts`, `ingestLan.ts`
- `apps/desktop/src-tauri/src/lib.rs`
- `crates/ingest/src/{config.rs,ingest_prefs.rs,test_lock.rs,token.rs,lib.rs}`

## How to verify
```bash
cargo test -p ingest
cargo test -p companion
cargo check -p desktop
cd apps/desktop && pnpm tauri dev   # Companion section: LAN toggle, warnings
# iOS: open BioFocusCompanion.xcodeproj → Test connection on Simulator (loopback ok)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: iOS Test connection / preflight → `GET /v1/status` ≤5s; loopback blocked on device; calm timeout/unreachable; success shows bind_mode + db_status
- [ ] AC2: iOS POST ≤10s; unauthorized distinct from timeout
- [ ] AC3: Desktop loopback warning; LAN fallback error; Copy URL gated
- [ ] AC4: LAN preference persists to `~/.biofocus/ingest_lan_enabled`; restart message; env still wins
- [ ] AC5: Docs troubleshooting in `12-development.md` + iOS README
- [ ] AC6: `cargo test -p ingest` + `cargo test -p companion` green
- [ ] Global DoD: no unwrap in prod paths; UI ↛ SQLite; Bearer unchanged

## Risks / not covered
- Physical iPhone + LAN dogfood not run in CI (Simulator-only compile path).
- LAN toggle requires **app restart** to rebind ingest (by design — no hot rebind).
- iOS Local Network permission deny — message mentions it but no Settings deep-link.

## Notes for QA
- Operator smoke from brief still needs manual LAN + physical device when available.
- `IngestURLPolicyTests.loopbackHostDetection` available in DEBUG Swift build only.
