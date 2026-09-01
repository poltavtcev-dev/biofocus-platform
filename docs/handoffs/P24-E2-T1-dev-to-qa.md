# Dev → QA: P24-E2-T1

**From:** Dev  
**To:** QA  
**Role that built:** Dev  
**Date:** 2026-08-12  
**AC source:** `docs/handoffs/P24-E2-T1-pm-brief.md` · `/docs/SPRINT_ROADMAP.md` Phase 24 E2  
**Branch:** `phase/24-circadian-offset`

## What changed

- Shipped catalog Feature **`CircadianOffset`** in `feature-engine` per **ADR-025**:
  - Observation-level timing only (sleep midpoint vs work/activity centroid)
  - Cadence **15m / 1m**; timing math **24h lookback**; output **0–100**
  - Omit unless **both** sleep + work/activity slots present
  - Prefer desk (`keystrokes` / `context_window`); reinforce with steps / active_energy / workout when desk thin
  - Map via `100 × (1 − offset_hours / 6)` clamped; circular hours from `sleep_mid + 12h`
  - ADR-007 `expected_slots = 2`; factors `sleep_timing` / `work_timing` (+ optional `activity_timing`)
- Registered via **`register_circadian_v1`** + wired into **`register_catalog_v1`**
- Finalized catalog §1.20 + ship notes in `12-development` / `16-glossary`
- Unit tests cover emit / omit / confidence / factors / no Feature-level magnitude proxy

## Crates / files touched

| Path | Change |
| :--- | :--- |
| `crates/feature-engine/src/catalog/circadian_offset.rs` | **new** — node + formula + tests |
| `crates/feature-engine/src/catalog/mod.rs` | module + exports + `register_circadian_v1` / catalog wire |
| `crates/feature-engine/src/lib.rs` | re-exports |
| `docs/06-feature-catalog.md` | §1.20 stub → shipped formula |
| `docs/12-development.md` | Phase 24 ship note |
| `docs/16-glossary.md` | CircadianOffset → shipped |

## How to verify (commands)

```bash
cargo test -p feature-engine circadian
cargo check -p feature-engine

# Scope guards
rg -n "CircadianOffset|register_circadian_v1" crates/feature-engine docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md
rg -n "SleepDebtNode|EnergyScoreNode|ActivityBalanceNode|FocusScoreNode|DeskAwayPresenceNode" crates/feature-engine/src/catalog/circadian_offset.rs
# (should only appear in the negative proxy test assertions, not as deps)

# No UI / migration / new data_type in this task
rg -n "circadian|CircadianOffset" apps/ desktop/ 2>/dev/null || true
rg -n "data_type.*circadian|CREATE TABLE|migration" crates/feature-engine/src/catalog/circadian_offset.rs || true
```

## Acceptance Criteria checklist (for QA)

- [ ] AC1: `CircadianOffset` in `feature-engine` per ADR-025 — Observation-level timing; 15m/1m; 24h lookback; 0–100
- [ ] AC2: Composition — sleep midpoint from qualifying `sleep_interval`; work/activity centroid (desk preferred + optional reinforcement); circular map `100×(1−offset/6)`; omit unless both slots; no Feature-level magnitude proxies
- [ ] AC3: ADR-007 confidence `expected_slots = 2`; emit only when both present
- [ ] AC4: Explanation factors calm (`sleep_timing` / `work_timing` / optional `activity_timing`) — not chronotype / night owl / disorder
- [ ] AC5: Registered in catalog/DAG (`register_circadian_v1` / `register_catalog_v1`); §1.20 finalized
- [ ] AC6: Unit tests — both slots emit · omit without sleep · omit without work/activity · confidence · factors · no magnitude proxy path
- [ ] AC7: Docs ship notes — `12-development` / `16-glossary`
- [ ] AC8: No rewrite of SleepDebt / EnergyScore / ActivityBalance / FocusScore / DeskAwayPresence; no new Observation `data_type`; no migration; no UI/Dashboard; no parallel engine crate
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered

- Exact desk vs activity blend weights when both present: v1 **prefers desk only** when desk evidence exists (activity reinforcement only when desk thin) — matches ADR “when desk signals thin”.
- Multi-bout sleep midpoint uses union span `(min_start + max_end) / 2` (not rest-second center-of-mass) — ADR “midpoint of union”.
- Dashboard / dogfood calm surface deferred to **P24-E3**.

## Notes for QA

- Sibling Feature files must be untouched by this task (only CircadianOffset + catalog wire + docs).
- PR freeze still active — no PR expected.
