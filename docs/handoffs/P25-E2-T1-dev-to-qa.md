# Dev → QA: P25-E2-T1

## Meta
- **Task ID:** P25-E2-T1
- **Title:** Ship catalog Feature `SustainedLoadIndicator` per ADR-026
- **Role that built:** Dev
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P25-E2-T1-pm-brief.md` · ADR-026
- **Branch:** `phase/25-sustained-load`

## What changed
- Shipped catalog Feature **`SustainedLoadIndicator`** in `feature-engine` per **ADR-026**:
  - Feature-level persistence: mean StressIndex / FatigueIndex / (MeetingDensity×100) over **4h lookback**
  - Cadence **15m / 1m**; weights **0.40 / 0.40 / 0.20**; renormalize present terms
  - Omit when both Stress and Fatigue absent (MeetingDensity alone never emits; windows driven from Stress|Fatigue ends)
  - ADR-007 `expected_slots = 3`; factors `stress` / `fatigue` / `meeting`
  - Does **not** use CognitiveLoad / FocusScore / CircadianOffset as inputs
- Registered via **`register_sustained_load_v1`** + wired into **`register_catalog_v1`**
- Finalized catalog §1.21 + ship notes in `12-development` / `16-glossary`
- Unit tests cover emit / omit / confidence / factors / no CognitiveLoad-as-input

## Crates / files touched
| Path | Change |
| :--- | :--- |
| `crates/feature-engine/src/catalog/sustained_load_indicator.rs` | **new** — node + formula + tests |
| `crates/feature-engine/src/catalog/mod.rs` | module + exports + `register_sustained_load_v1` / catalog wire |
| `crates/feature-engine/src/lib.rs` | re-exports |
| `docs/06-feature-catalog.md` | §1.21 stub → shipped formula |
| `docs/12-development.md` | Phase 25 ship note |
| `docs/16-glossary.md` | SustainedLoadIndicator → shipped |

## How to verify (commands)
```bash
cargo test -p feature-engine sustained_load
cargo check -p feature-engine

rg -n "SustainedLoadIndicator|register_sustained_load_v1" \
  crates/feature-engine docs/06-feature-catalog.md docs/12-development.md docs/16-glossary.md

# No CognitiveLoad-as-input / leaf rewrite this task
rg -n "CognitiveLoadNode|\"CognitiveLoad\"" crates/feature-engine/src/catalog/sustained_load_indicator.rs
git diff --name-only -- crates/feature-engine/src/catalog/stress_index.rs \
  crates/feature-engine/src/catalog/fatigue_index.rs \
  crates/feature-engine/src/catalog/meeting_density.rs \
  crates/feature-engine/src/catalog/cognitive_load.rs
# expect: empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: SustainedLoadIndicator Feature-level; 15m/1m; 4h lookback; 0–100
- [ ] AC2: Composition — means + weights 0.40/0.40/0.20; omit when both Stress&Fatigue absent; no CognitiveLoad/Focus/Circadian inputs
- [ ] AC3: ADR-007 slots=3; coverage × mean upstream confidence
- [ ] AC4: Calm factors (`stress` / `fatigue` / `meeting`) — not burnout copy
- [ ] AC5: `register_sustained_load_v1` / `register_catalog_v1`; §1.21 finalized
- [ ] AC6: Unit tests — all-three · Stress+Fatigue · meetings-alone omit · empty omit · confidence · factors · no CognitiveLoad path
- [ ] AC7: Docs `12-development` / `16-glossary` ship notes
- [ ] AC8: No leaf / CognitiveLoad rewrite; no migration; no UI; no parallel crate
- [ ] Global DoD

## Risks / not covered
- Dashboard / dogfood calm surface deferred to **P25-E3** (label **Prolonged load**).
- Exact MeetingDensity unit already 0–1 density → ×100 (matches CognitiveLoad mapping).

## Notes for QA
- Sibling Feature formula files must be untouched.
- PR freeze still active — no PR expected.
