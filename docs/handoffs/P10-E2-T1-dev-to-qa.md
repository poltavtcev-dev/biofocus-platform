# Dev → QA: P10-E2-T1

## Meta
- **Task ID:** P10-E2-T1
- **Title:** Implement Browser categories collector plugin (ADR-010)
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P10-E2-T1; brief `docs/handoffs/P10-E2-T1-pm-brief.md`
- **Branch:** `phase/10-plugin-wave-1`

## What changed
- `BrowserCategoryPlugin` (`com.biofocus.macos.browser`) in `macos-collector` with Capability `browser_categories` → `browser_category`.
- Opt-in env `BIOFOCUS_BROWSER_CATEGORIES=1` (default off); Desktop `ingest_host` starts only when set → same Observation channel → persist.
- Payload: required coarse `category` + optional `browser_bundle_id`; **never** URL/title/content.
- Injectable `ScriptedBrowserProbe` + `SystemBrowserProbe` (known browser → `unknown` + bundle; soft-fail otherwise). Poll ≥5s, emit on change; `stop_stream` joins.
- `bio-spec::validate_browser_category_payload` + ingest `invalid_browser_category`; docs finalized (07/08/10/12).
- Integration tests: emit→persist + stop freezes probe polls.

## Crates / apps / files touched
- `crates/macos-collector/src/{browser_plugin,browser_probe,browser_stream,payload,lib}.rs`
- `crates/macos-collector/tests/collector_integration.rs`
- `crates/bio-spec/src/{browser_category,error,lib,life_event}.rs` + `tests/contracts.rs`
- `crates/ingest/src/routes.rs`
- `apps/desktop/src-tauri/src/ingest_host.rs`
- `docs/{07-contracts,08-plugin-sdk,10-security,12-development}.md`
- `docs/handoffs/P10-E2-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p bio-spec browser
cargo test -p macos-collector --test collector_integration
cargo test -p macos-collector browser --lib
cargo check -p macos-collector -p bio-spec -p ingest -p desktop
```

Optional dogfood (macOS):
```bash
export BIOFOCUS_BROWSER_CATEGORIES=1
# restart Desktop; switch to Safari/Chrome → SQLite observations data_type=browser_category (category often "unknown" without URL mapping)
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: BioFocusPlugin id `com.biofocus.macos.browser` + Capability `browser_category` in macos-collector
- [ ] AC2: ingest_host starts only when `BIOFOCUS_BROWSER_CATEGORIES=1`; same channel → persist; UI↛SQLite
- [ ] AC3: Observations match ADR-010 / contracts; no URL/title/keystroke content in payload or tests
- [ ] AC4: Injectable probe; OS soft-fail OK; ≥5s / on-change; stop joins; no busy-loop
- [ ] AC5: Docs shipped (not “planned only”): 07, 08, 10, 12
- [ ] AC6: Unit/integration tests cover emit→persist + stop freezes
- [ ] AC7: This handoff exists
- [ ] Global DoD: no unwrap/expect in prod paths; idle; no new SQLite schema; personal self-tracking framing

## Risks / not covered
- Rich URL→category mapping without persisting URLs — deferred; OS probe emits `unknown` for known browsers.
- Pipeline normalize strip of forbidden keys → can land with P10-E3 if needed (collector never emits them).
- `DistractionScore` Feature DAG → **P10-E3-T1** (out of scope).
- Full GUI dogfood not required for Pass (mock covers AC).

## Notes for QA
- Provider id equals plugin id: `com.biofocus.macos.browser`.
- Confidence 0.5 when OS emits `unknown`; scripted categories use 1.0.
- No PR (freeze).
