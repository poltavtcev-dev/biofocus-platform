# Dev → QA: P3-E3-T1

## Meta
- **Task ID:** P3-E3-T1
- **Title:** Alert level mapping (Core)
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P3-E3-T1; `docs/handoffs/P3-E3-T1-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- `feature_engine::map_alert_level(&EngineOutput) → AlertLevel { Green, Yellow, Red }`.
- v1 rules (documented in rustdoc + below): Red ← `High_Stress`; Yellow ← latest StressIndex or FatigueIndex **> 60**; else Green (empty → Green).
- Unit tests for Green / Yellow / Red (+ threshold edge, latest-window wins, Red overrides).
- Entry note in `docs/12-development.md`. No UI / IPC / SQLite.

## Crates / apps / files touched
- `crates/feature-engine/src/alert.rs` (new)
- `crates/feature-engine/src/lib.rs`
- `docs/12-development.md`
- `docs/handoffs/P3-E3-T1-pm-brief.md`

## How to verify (commands)
```bash
cargo test -p feature-engine
cargo test -p feature-engine alert::
```

## Acceptance Criteria checklist (for QA)
- [x] AC1 — deterministic map EngineOutput → AlertLevel { Green, Yellow, Red }
- [x] AC2 — rules in rustdoc (`alert.rs`) + this handoff + `docs/12-development.md`
- [x] AC3 — unit tests: empty Green; elevated Yellow; High_Stress Red
- [x] AC4 — no Tauri UI / Menubar / IPC / SQLite schema / ADR
- [x] AC5 — this handoff
- [x] Global DoD — no unwrap/expect in prod paths; idle-safe pure fn; UI↛DB

## v1 rules (canonical)
1. **Red** if any Signal with `type == High_Stress`.
2. **Yellow** else if latest (by `time_window.end`) `StressIndex` **or** `FatigueIndex` scalar **>** `YELLOW_FEATURE_THRESHOLD` (60.0). Equality at 60 → Green.
3. **Green** otherwise (including empty `EngineOutput`).

## Risks / not covered
- IPC expose (`get_status`) → **P3-E3-T2**.
- Menubar colors → **P3-E3-T3**.
- Wire live FeatureHook → engine → map (follow-up).
- Serde on `AlertLevel` deferred to T2 if needed (`as_str()` ready).
- Mi Band 8 / LAN — out of scope.

## Notes for QA
- Public API: `map_alert_level`, `AlertLevel`, `YELLOW_FEATURE_THRESHOLD`.
- Catalog High_Stress still requires StressIndex > 75 for > 5m; Yellow fires earlier at > 60 on latest sample.
