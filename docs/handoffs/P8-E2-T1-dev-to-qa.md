# Dev → QA: P8-E2-T1

**From:** Dev  
**To:** QA  
**Status:** Ready for QA  
**Date:** 2026-08-07  
**Brief:** `docs/handoffs/P8-E2-T1-pm-brief.md`

## Meta
- **Task ID:** P8-E2-T1
- **Title:** Pattern Discovery v1 Insight path
- **Role that built:** Dev
- **AC source:** `/docs/SPRINT_ROADMAP.md` + `docs/handoffs/P8-E2-T1-pm-brief.md` (ADR-008)

## What changed
- **`feature-engine` baseline helper (ADR-008):** `recompute_focus_afternoon_baseline` — bounded (N ≤ 7) UTC-afternoon FocusScore means from local Observations; thin/empty → `Ok([])`.
- **`knowledge-engine` rule `focus_vs_recent_baseline_v1`:** compares live `FocusScore` (confidence ≥ 0.4) to mean of recomputed series; emit when `|Δ| ≥ 10`; Evidence = Feature `FocusScore`; calm non-clinical copy; category `pattern`.
- **`PatternInputs` + `evaluate_with_pattern`:** snapshot-only rules ignore pattern; host supplies baseline series.
- **Desktop Core wire:** `get_insights` / `generate_report` load Observations via `ObservationRepository`, recompute series, optional in-process memo (TTL 60s + watermark). UI ↛ SQLite. No Feature-history migration / no always-on worker.

## Crates / files touched
- `crates/feature-engine/src/baseline.rs` (new)
- `crates/feature-engine/src/lib.rs`
- `crates/knowledge-engine/src/pattern.rs` (new)
- `crates/knowledge-engine/src/rules/focus_baseline.rs` (new)
- `crates/knowledge-engine/src/{lib,engine,rule}.rs`
- `crates/knowledge-engine/src/rules/{mod,context_switch,high_stress}.rs`
- `apps/desktop/src-tauri/src/pattern_host.rs` (new)
- `apps/desktop/src-tauri/src/lib.rs`
- Branch: `phase/8-pattern-discovery`

## How to verify (commands)
```bash
cargo test -p feature-engine -p knowledge-engine --lib
cargo test -p desktop --lib
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ≥1 baseline Insight rule `focus_vs_recent_baseline_v1` — current FocusScore vs N≤7 afternoon series; emit on `|Δ|≥δ` + confidence gates; Evidence Feature ids; calm copy
- [ ] AC2: Observation load + bounded Feature recompute wired (Core / feature-engine helpers); no Correlation Engine crate; no Feature-history migration
- [ ] AC3: Idle / privacy — no always-on worker; thin history → omit (`Ok([])` / no pattern Insight); local-only
- [ ] AC4: Optional in-process memo only (not SQLite)
- [ ] AC5: Unit tests — thin history (no emit); rich synthetic series (emit + Evidence); calm non-clinical copy
- [ ] AC6: This handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- Afternoon bucket is **UTC 13:00–17:00** (not local TZ) — acceptable v1 simplification; dogfood may want local afternoon later.
- Confidence gate `0.4` and δ `10` are rule constants (ADR sketch).
- Dashboard UX polish for pattern Insights → **P8-E3-T1** (Core path via existing `get_insights` is wired).
- No end-to-end GUI smoke with multi-day live DB.

## Notes for QA
- `register_insights_v1` now registers **3** rules (was 2).
- Snapshot-only `evaluate()` still works (empty pattern → baseline rule omits).
- Prod paths in new modules use `Result` / soft-fail warn — `expect` only in `#[cfg(test)]`.
