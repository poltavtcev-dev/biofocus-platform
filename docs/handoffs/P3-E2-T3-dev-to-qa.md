# Dev → QA: P3-E2-T3

## Meta
- **Task ID:** P3-E2-T3
- **Title:** `StressIndex` + `FatigueIndex` (v1) + High_Stress Signal
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E2 / P3-E2-T3; brief `docs/handoffs/P3-E2-T3-pm-brief.md`
- **Branch:** `phase/3-pipeline-features`

## What changed
- Summary:
  - Catalog nodes: `StressIndexNode` (Features + transient `High_Stress` Signal) and `FatigueIndexNode` (depends on `FocusScore`).
  - Helpers: `register_stress_v1` (Stress → Fatigue), `register_catalog_v1` (Focus + Stress/Fatigue).
  - High_Stress: StressIndex > 75 for longer than 5 minutes of consecutive 1m samples → `Signal { type: High_Stress, severity: High }`.
  - Нет UI / Menubar / новой SQLite-схемы / ADR / E2E suite / FeatureHook wire.
- Crates / files:
  - `crates/feature-engine/Cargo.toml` — `uuid` (Signal ids)
  - `crates/feature-engine/src/catalog/stress_index.rs` — StressIndex + High_Stress + tests
  - `crates/feature-engine/src/catalog/fatigue_index.rs` — FatigueIndex + tests
  - `crates/feature-engine/src/catalog/mod.rs` — register helpers / re-exports
  - `crates/feature-engine/src/lib.rs` — public API / docs
  - `Cargo.lock` — uuid link for feature-engine (if changed)

## v1 formulas (simplifications)

| Feature / Signal | Rule |
| :--- | :--- |
| **StressIndex** | Window 15m / step 1m. From `hrv`: RMSSD map 100 @ ≤15 ms → 0 @ ≥70 ms; optional SDNN same map + `pnn50` as `100 - pnn50`; average present components. Provenance = HRV IDs in window. |
| **High_Stress** | Contiguous minute-aligned StressIndex samples all `> 75`; emit if `(last_end - first_end) > 300` s. `Severity::High`. |
| **FatigueIndex** | Depends on FocusScore. Weighted (renormalized): `100 - FocusScore` (0.50) + active minutes since snapshot min vs 8h day (0.30) + HR rise vs early-15m baseline / 20 bpm (0.20). Provenance = keystrokes/HR/context in window. |

## How to verify (commands)
```bash
cargo check -p feature-engine
cargo test -p feature-engine
```

Ожидание: **16** unit-тестов зелёные (5 skeleton + 4 focus catalog + 5 stress + 2 fatigue).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: v1 `StressIndex` / `FatigueIndex`; упрощения задокументированы (этот handoff + module rustdoc)
- [ ] AC2: StressIndex > 75 дольше 5 мин → `Signal` type `High_Stress`, severity `bio-spec::Severity::High`
- [ ] AC3: Features + provenance Observation IDs; Signals в `EngineOutput.signals`
- [ ] AC4: Unit-тесты на порог High_Stress + базовые Stress/Fatigue (`docs/11-testing.md`)
- [ ] AC5: Nodes через DAG API; нет UI / новой SQLite-схемы / ADR
- [ ] AC6: handoff этот файл
- [ ] Global DoD: no `unwrap`/`expect` в production paths; UI↛DB; glossary terms; idle-safe (sync compute, no spin)

## Entrypoint
| Symbol | Role |
| :--- | :--- |
| `feature_engine::register_stress_v1` | Register StressIndex + FatigueIndex (needs FocusScore already registered) |
| `feature_engine::register_catalog_v1` | Focus (T2) + Stress/Fatigue (T3) |
| `feature_engine::StressIndexNode` | Feature id `StressIndex` + optional High_Stress |
| `feature_engine::FatigueIndexNode` | Feature id `FatigueIndex` (depends on FocusScore) |
| `feature_engine::HIGH_STRESS_SIGNAL_TYPE` | `"High_Stress"` |
| `feature_engine::HIGH_STRESS_THRESHOLD` | `75.0` |
| `feature_engine::HIGH_STRESS_MIN_DURATION_SECS` | `300` (strict `>` for emit) |

## Risks / not covered
- Baevsky SI / full RR-interval math не реализованы — RMSSD linear map only.
- Fatigue «active hours» считаются от min timestamp snapshot, не от calendar day / midnight.
- Strict `>` 5 minutes: span == 300 s **не** эмитит Signal (покрыто тестом).
- E2E Observation → Feature/Signal → **P3-E2-T4**.
- Alert mapping / Menubar → **P3-E3**.
- `FeatureHook` desktop wire — вне scope.

## Notes for QA
- Branch already contains merge of `main` (PR #19 T2 PM close).
- `FatigueIndex` alone without `FocusScore` → `UnknownDependency` on `run` (тест `fatigue_requires_focus_score_node`).
- Prefer `register_catalog_v1` in integration smoke; unit tests use `register_focus_v1` + `register_stress_v1`.
