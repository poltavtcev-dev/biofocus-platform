# Dev → QA: P12-E2-T1

## Meta
- **Task ID:** P12-E2-T1
- **Title:** Implement Now Playing ambient plugin (ADR-012)
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P12-E2-T1; brief `docs/handoffs/P12-E2-T1-pm-brief.md`
- **Branch:** `phase/12-ambient-packaging`

## What changed
- `NowPlayingPlugin` (`com.biofocus.macos.now_playing`) in `macos-collector` — Capability `now_playing` / `data_type: "now_playing"`.
- Opt-in `BIOFOCUS_NOW_PLAYING=1` (default off); Desktop `ingest_host` starts plugin only when set; same Observation channel → persist.
- Payload: required `media_kind` (`music`\|`podcast`\|`other`\|`none`\|`unknown`) + `is_playing` bool — **no** titles/artists/lyrics/playlists.
- Injectable `ScriptedNowPlayingProbe`; production `SystemNowPlayingProbe` soft-fails idle (`None`) — compile + idle-safe; poll ≥5s; emit on change; `stop_stream` joins.
- `bio_spec::validate_now_playing_payload` + ingest reject `invalid_now_playing`.
- Docs finalized shipped (not planned-only): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, glossary.
- **No** Feature DAG / packaging installer / schema migration (OOS → E3 / future ADR).

## Crates / apps / files
- `crates/bio-spec/src/now_playing.rs` (+ error / routing / exports / contracts test)
- `crates/ingest/src/routes.rs` (`invalid_now_playing`)
- `crates/macos-collector/src/now_playing_{plugin,probe,stream}.rs` + `payload.rs` / `lib.rs`
- `crates/macos-collector/tests/collector_integration.rs`
- `apps/desktop/src-tauri/src/ingest_host.rs`
- Docs: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`

## How to verify (commands)
```bash
cargo test -p bio-spec now_playing
cargo test -p macos-collector --test collector_integration now_playing
cargo check -p desktop

# Optional privacy grep on emitted payload helpers
rg -n "title|artist|lyrics|playlist" crates/macos-collector/src/now_playing_*.rs crates/macos-collector/src/payload.rs || true
```

### Manual smoke
1. Default (env unset): Desktop logs `now_playing collector off`.
2. `export BIOFOCUS_NOW_PLAYING=1` + restart: collector armed; system probe idle (no titles) until a privacy-safe OS mapping lands.
3. Integration path covered by scripted probe → channel → SQLite.

## Acceptance Criteria checklist (for QA)
- [ ] AC1 Plugin id + Capability `now_playing` in macos-collector / plugin-sdk
- [ ] AC2 ingest_host only when `BIOFOCUS_NOW_PLAYING=1`; same channel → persist; UI↛SQLite
- [ ] AC3 Payload contract: media_kind + is_playing; never titles/artists/lyrics/playlists/mic/geo
- [ ] AC4 Injectable probe; system soft-fail; ≥5s / on-change; stop joins; no busy-loop
- [ ] AC5 Docs shipped (`07` / `08` / `10` / `12`)
- [ ] AC6 Tests: emit→persist + stop freezes polls
- [ ] AC7 Handoff present
- [ ] Global DoD: no unwrap in prod paths; no new schema; opt-in default off

## Risks / not covered
- Production OS Now Playing mapping intentionally soft-fails (`None`) — no MediaRemote/private API in v1 (privacy). Live dogfood of real media state needs a future privacy-safe probe or scripted injection.
- `AmbientMediaShare` not implemented (E3).

## Notes for QA
- Mirror of Browser categories pattern; browser + now_playing both clone `collector_tx`.
