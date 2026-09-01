# Dev → QA: P13-E2-T1

## Meta
- **Task ID:** P13-E2-T1
- **Title:** Implement Git activity plugin (ADR-013)
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P13-E2-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` Epic P13-E2
- **Branch:** `phase/13-plugin-wave-2`

## What changed
- `GitActivityPlugin` (`com.biofocus.macos.git`, Capability `git_activity`) in `macos-collector` — poll ≥5s, emit on identity change, `stop_stream` joins.
- Injectable `ScriptedGitActivityProbe` + soft-fail `SystemGitActivityProbe` (no path-allowlist in v1 → idle / no emit).
- Desktop `ingest_host` starts plugin **only** when `BIOFOCUS_GIT_ACTIVITY=1` (default off); same Observation channel → persist.
- `bio_spec::validate_git_activity_payload` + ingest reject `invalid_git_activity`; pipeline normalize strips paths/remotes/branch/SHA/message/diff/author.
- Docs finalized as **shipped**: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`.

## Crates / apps / files
- `crates/macos-collector/src/git_activity_{plugin,probe,stream}.rs` (+ `lib.rs`, `payload.rs`)
- `crates/macos-collector/tests/collector_integration.rs`
- `crates/bio-spec/src/git_activity.rs` (+ `error.rs`, `lib.rs`, `life_event.rs`, `tests/contracts.rs`)
- `crates/pipeline/src/normalize.rs`
- `crates/ingest/src/routes.rs`
- `apps/desktop/src-tauri/src/ingest_host.rs`
- Docs: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`

## How to verify (commands)
```bash
cargo test -p bio-spec -p macos-collector -p pipeline -p ingest --tests
cargo check -p desktop
rg -n "BIOFOCUS_GIT_ACTIVITY|com.biofocus.macos.git|validate_git_activity_payload|GitActivityPlugin" \
  crates/macos-collector crates/bio-spec crates/pipeline crates/ingest apps/desktop/src-tauri docs/07-contracts.md docs/08-plugin-sdk.md
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: `BioFocusPlugin` id `com.biofocus.macos.git` with Capability covering `git_activity` in `macos-collector`
- [ ] AC2: `ingest_host` starts only when `BIOFOCUS_GIT_ACTIVITY=1`; same channel → persist; UI↛SQLite
- [ ] AC3: Observations: required `activity_kind` closed-set + optional `event_count` ≥ 1; never paths/remotes/branch/SHA/message/diff/author
- [ ] AC4: Scripted probe for tests; system probe soft-fails idle; no busy-loop; `stop_stream` freezes polls; no path-allowlist table
- [ ] AC5: Docs shipped (`07`/`08`/`10`/`12`); `validate_git_activity_payload` present
- [ ] AC6: Unit/integration: emit → channel → persist; stop freezes counts; forbidden keys stripped on normalize
- [ ] AC7: This handoff
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Live OS git mapping is intentionally soft-fail (no ADR for watched-roots allowlist) — dogfood emits only via scripted/mock until a future ADR.
- Feature `GitActivityRate` is **out of scope** (P13-E3).

## Notes for QA
- Default env off: without `BIOFOCUS_GIT_ACTIVITY=1` host must not start the plugin.
- Production path compiles and idles safely when enabled.
