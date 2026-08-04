# Dev → QA: P2-E2-T2

## Meta
- **Task ID:** P2-E2-T2
- **Title:** Keystroke / input aggregates (privacy-safe)
- **Role:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E2-T2; brief `docs/handoffs/P2-E2-T2-pm-brief.md`

## What was built
- `KeystrokeAggregatePlugin` in `crates/macos-collector`: windowed key-down **counts/rates** → `data_type=keystrokes`.
- Payload only `{count, window_secs, rate_per_min}` — never characters / key codes / clipboard.
- Opt-in: `BIOFOCUS_INPUT_AGGREGATES=1` (default **off**). Desktop `ingest_host` starts plugin only when set; same channel → persist.
- System probe: listen-only `CGEventTap` when `AXIsProcessTrusted`; else idle counts=0 (no panic).
- Docs: `07-contracts`, `08-plugin-sdk`, `10-security` §3, `12-development`.

## Verify commands
```bash
cargo test -p macos-collector
cargo check -p desktop
cargo test -p desktop
cargo test -p ingest
```

## AC checklist for QA
- [ ] AC1: `data_type=keystrokes`, provider `com.biofocus.macos.input`, schema aligned with `04`/`07`
- [ ] AC2: aggregates only — tests assert no `char`/`text`/`keys`
- [ ] AC3: Accessibility documented; deny → idle no panic
- [ ] AC4: single disable flag (`BIOFOCUS_INPUT_AGGREGATES`, default off)
- [ ] AC5: channel → persist; UI ↛ SQLite; clean stop
- [ ] AC6: mock tests emit + untrusted emits nothing + stop idle
- [ ] AC7: this handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; idle select!/interval

## Risks / not covered
- Live GUI + real Accessibility grant not smoke-tested here (sandbox/CI).
- Window title still deferred.
- `CGEventMaskBit` / tap path is macOS-only FFI; non-macOS builds idle.

## Git
Commit after this handoff per agent git policy (no push until sprint gate).
