# Dev → QA: P7-E3-T1

## Meta
- **Task ID:** P7-E3-T1
- **Title:** First bio-backed Trust Features (`RecoveryScore`)
- **Role that built:** Dev
- **Date:** 2026-08-06
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P7-E3-T1; brief `docs/handoffs/P7-E3-T1-pm-brief.md`
- **Branch:** `phase/7-trust-layer`

## What changed
- Shipped **`RecoveryScore`** from planned backlog → catalog §1.7: short-term physiological recovery proxy from `hrv` (`rmssd_ms`) + optional `heart_rate` (`bpm`). No sleep required for v1.
- Registered via `register_recovery_v1` and wired into `register_catalog_v1` (Feature Worker / snapshot path picks it up automatically).
- ADR-007 confidence: expected slots = 2 (HRV / HR); HRV-only → coverage `0.5`; rich both → `1.0` (× mean obs confidence).
- Optional ExplanationFactors (E2 shape): `hrv` / `heart_rate` with renormalized shares (0.70 / 0.30 when both present).
- Calm non-clinical copy; distinct from schedule `RecoveryBetweenMeetings`.
- Unit tests: empty/idle, HR-only omit, high/low RMSSD, rich confidence+factors, thin confidence, elevated HR blends down, `register_catalog_v1` includes node.

### Formula (v1)
| Component | Weight | Map |
| :--- | :--- | :--- |
| HRV | 0.70 | RMSSD 0 @ ≤15 ms → 100 @ ≥70 ms (inverse StressIndex anchors) |
| HR (optional) | 0.30 | `100 - clamp((mean_bpm - early_baseline) / 20 * 100, 0, 100)` |

No usable `rmssd_ms` → omit Feature (HR alone insufficient).

## Crates / files touched
- `crates/feature-engine/src/catalog/recovery_score.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs` — `register_recovery_v1` + catalog_v1 wire
- `crates/feature-engine/src/lib.rs` — re-exports
- `docs/06-feature-catalog.md` — §1.7; removed `RecoveryScore` from backlog table
- `docs/12-development.md` — entrypoint note
- `docs/09-api.md` — Feature list mention

## How to verify (commands)
```bash
cargo test -p feature-engine
cargo test -p pipeline --test pipeline_e2e
cargo check -p feature-engine
```

Dev ran all three — pass (56 feature-engine tests; 3 pipeline e2e).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ≥1 Feature moved backlog → catalog §1 with formula / window / units / deps / provenance + ADR-007 confidence (`RecoveryScore`)
- [ ] AC2: Registered in `register_catalog_v1` (via `register_recovery_v1`)
- [ ] AC3: Unit tests — empty/thin/rich; confidence per ADR-007; idle-safe
- [ ] AC4: Calm non-clinical copy (no burnout / clinical diagnosis)
- [ ] AC5 (optional): ExplanationFactors when weighted components present
- [ ] AC6: This handoff exists
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- `DeepWorkScore` / `AttentionStability` not shipped (picked RecoveryScore as bio-backed preference).
- SleepDebt / CognitiveLoad / CircadianOffset still backlog (out of scope).
- Dashboard chart series for `RecoveryScore` not added (snapshot path only — out of scope).
- Alert mapping unchanged (still Stress/Fatigue/`High_Stress`).
- No new SQLite schema.

## Notes for QA
- Prefer focused tests via `RecoveryScoreNode` / `register_recovery_v1`; production path = `register_catalog_v1`.
- HRV-only confidence ≈ `0.5`; rich HRV+HR ≈ `1.0` (obs confidence 1.0).
- Factors: both present → shares `0.70` / `0.30`; HRV-only → single factor share `1.0`.
- Do not confuse with `RecoveryBetweenMeetings` (calendar gaps).
