# Dev → QA: P16-E1-T1

## Meta
- **Task ID:** P16-E1-T1
- **Title:** Implement ambient light plugin (ADR-015)
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P16-E1-T1; brief `docs/handoffs/P16-E1-T1-pm-brief.md`
- **Branch:** `phase/16-ambient-light`

## What changed
- Resumed / finalized ADR-015 ambient light collector as **shipped** for Phase 16 (code already present from earlier ambient-light commit; host + plugin path verified).
- `AmbientLightPlugin` (`com.biofocus.macos.ambient_light`) in `macos-collector` — Capability `ambient_light` / `data_type: "ambient_light"`.
- Opt-in `BIOFOCUS_AMBIENT_LIGHT=1` (default off); Desktop `ingest_host` starts plugin only when set; same Observation channel → persist worker.
- Payload: required `light_kind` (`dark`\|`dim`\|`moderate`\|`bright`\|`unknown`) + optional `level` 0–100 — **no** camera frames / screenshots / geo / mic / cloud light telemetry.
- Injectable `ScriptedAmbientLightProbe`; production `SystemAmbientLightProbe` soft-fails idle (`None`); poll ≥5s; emit on light-band change; `stop_stream` joins.
- Unit coverage: plugin id/capability + default poll ≥5s; integration: emit→channel→persist, stop freezes polls, soft-fail emits nothing.
- Docs finalized **shipped** (not parked-only): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary` (+ storage / catalog notes). Feature still → P16-E2.
- **No** Feature formula / `AmbientLightShare`; **no** SQLite migration; **no** new ADR.

## Crates / apps / files
- `crates/macos-collector/src/ambient_light_{plugin,probe,stream}.rs` (+ unit tests on plugin)
- `crates/macos-collector/src/payload.rs` / `lib.rs` (existing exports)
- `crates/macos-collector/tests/collector_integration.rs` (ambient_light_* tests)
- `crates/bio-spec/src/ambient_light.rs` (module status comment)
- `apps/desktop/src-tauri/src/ingest_host.rs` (already wired; verified)
- Docs: `04-storage`, `06-feature-catalog`, `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`

## How to verify (commands)
```bash
cargo test -p bio-spec ambient_light
cargo test -p macos-collector ambient_light
cargo test -p macos-collector --test collector_integration ambient_light
cargo check -p desktop

# Optional privacy grep on collector helpers
rg -n "camera_frame|screenshot|latitude|longitude|mic_waveform|geo" \
  crates/macos-collector/src/ambient_light_*.rs crates/macos-collector/src/payload.rs || true
```

### Manual smoke
1. Default (env unset): Desktop logs `ambient_light collector off`.
2. `export BIOFOCUS_AMBIENT_LIGHT=1` + restart: collector armed; system probe idle (no frames) until a privacy-safe OS mapping lands.
3. Integration path covered by scripted probe → channel → SQLite.

## Acceptance Criteria checklist (for QA)
- [ ] AC1 Plugin id `com.biofocus.macos.ambient_light` + Capability `ambient_light` in macos-collector / plugin-sdk
- [ ] AC2 ingest_host only when `BIOFOCUS_AMBIENT_LIGHT=1`; same channel → persist; UI↛SQLite
- [ ] AC3 Payload contract: `light_kind` + optional `level` 0–100; never camera/screen/geo/mic/cloud light
- [ ] AC4 Soft-fail OS probe; scripted probe; ≥5s / on-change; `stop_stream` joins; no busy-loop
- [ ] AC5 Docs shipped (`07` / `08` / `10` / `12` / glossary as needed); Phase 16 collector active; Feature → P16-E2
- [ ] AC6 Tests: emit→channel→persist; stop freezes; soft-fail no busy-loop
- [ ] AC7 No Feature / migration / new ADR
- [ ] AC8 Handoff present
- [ ] Global DoD: no unwrap in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Production OS ambient-light mapping intentionally soft-fails (`None`) — no camera / private screen sampling in v1 (privacy). Live dogfood of real brightness needs a future privacy-safe probe or scripted injection.
- `AmbientLightShare` not implemented (→ P16-E2-T1).

## Notes for QA
- Mirror of Now Playing / Git activity pattern; ambient_light clones `collector_tx` in `ingest_host`.
- Do **not** start P16-E2 or Phase 17 in this chat; PM closes Done.
