# Dev → QA: P2-E3-T1

## Meta
- **Task ID:** P2-E3-T1
- **Title:** Companion contract + minimal HealthKit sample path
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E3-T1; brief `docs/handoffs/P2-E3-T1-pm-brief.md`

## What changed
- New workspace crate `apps/companion`:
  - `CompanionClient` posts Observation JSON array to `POST /v1/ingest`
  - `sample_heart_rate_observation` / `post_sample_heart_rate` (provider `com.biofocus.applehealth`)
  - Explicit `CompanionError::Unauthorized` (401) and `Network` (transport)
  - CLI `biofocus-companion-sample` (exit 2 network / 3 unauthorized)
- Integration tests: happy path against ephemeral loopback ingest, wrong token, connection refused
- iOS Swift stub under `apps/companion/ios/` (HealthKit one-shot → same HTTP contract; not a full Xcode app)
- Docs: `07` / `09` / `10` / `11` / `12` + `apps/companion/README.md`
- CI: `cargo test -p companion` on rust-core job

## How to verify (commands)
```bash
cargo test -p companion
cargo check --workspace --exclude desktop
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: sample `heart_rate` → HTTP ingest on reachable host (loopback documented)
- [ ] AC2: body = JSON array of Observation (same desktop contract)
- [ ] AC3: network / 401 handled without crash; 401 not silently swallowed
- [ ] AC4: no Feature math / dashboard / cloud
- [ ] AC5: this handoff present
- [ ] Global DoD: no unwrap/expect in prod companion lib paths; local-only

## Risks / not covered
- Full Xcode iOS app / App Store target not created — Swift sources are stubs.
- Desktop still binds **loopback only** — physical iPhone on LAN cannot reach ingest until host bind/LAN work (documented).
- Live Desktop + real HealthKit smoke deferred (tests use mock loopback + CLI docs).
- CLI uses UUIDv7; Swift stub uses `UUID()` (v4) until wired into a real app — note for T2 polish.

## Notes for QA
- Package name: `companion` (path `apps/companion`).
- Pairing QR/copy remains **P2-E3-T2**.

## Git
Commit after this handoff (no push until sprint gate).
