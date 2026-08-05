# Handoff: Dev → QA

## Meta
- **Task ID:** P4-E1-T1
- **Title:** Feature snapshot API + IPC
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P4-E1-T1; brief `docs/handoffs/P4-E1-T1-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai` (from `feat/p3-e3-t3-menubar-alert` — Menubar prerequisite)

## What changed
- Public Core type `feature_engine::FeatureSnapshot` — Features + optional Signals with provenance Observation ids; built from last `EngineOutput` (no Observation payloads).
- Desktop `SnapshotState` + `CatalogAlertHook` caches the latest snapshot alongside `AlertState` on each Feature Worker tick.
- New IPC command `get_feature_snapshot` (camelCase DTO); `get_status` stays lean (version / dbStatus / alertLevel only).
- Contract documented in `docs/09-api.md`; note in `docs/12-development.md`.
- Unit tests: empty snapshot + non-empty synthetic Features/Signals (no biometrics / paths in JSON); hook caches StressIndex + High_Stress.

### Crates / apps / files touched
| Path | Change |
| :--- | :--- |
| `crates/feature-engine/src/snapshot.rs` | **new** — `FeatureSnapshot` API + unit tests |
| `crates/feature-engine/src/lib.rs` | export `FeatureSnapshot`, `FeatureValue` |
| `apps/desktop/src-tauri/src/alert_state.rs` | `SnapshotState`; hook writes snapshot cache |
| `apps/desktop/src-tauri/src/feature_host.rs` | manage `SnapshotState` |
| `apps/desktop/src-tauri/src/lib.rs` | `get_feature_snapshot` + DTO + tests |
| `docs/09-api.md` | IPC contract |
| `docs/12-development.md` | short invoke note |

## How to verify (commands)
```bash
cargo test -p feature-engine
cargo test -p desktop --lib
cargo check -p desktop
```

Dev ran all three — green (2026-08-05).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Public Core API `FeatureSnapshot` returns Features (+ optional Signals) with ids / names / values / time windows / provenance Observation ids
- [ ] AC2: IPC `get_feature_snapshot` registered; `get_status` not bloated with Feature series
- [ ] AC3: IPC payload has **no** raw Observation biometric fields and **no** absolute filesystem paths
- [ ] AC4: Unit tests cover empty (idle) + non-empty synthetic snapshot
- [ ] AC5: Idle-safe — command is pure cache read (no busy-loop / spin poll inside invoke)
- [ ] AC6: Contract in `docs/09-api.md` (+ handoff); brief note in `docs/12-development.md`
- [ ] Global DoD: no `unwrap`/`expect` in production paths; UI↛DB; glossary Feature/Signal; no Recharts / LLM / new SQLite table

### Example JSON (`get_feature_snapshot`)
```json
{
  "features": [
    {
      "featureId": "FocusScore",
      "timeWindow": { "start": 100, "end": 1000 },
      "value": 72.5,
      "provenance": ["00000000-0000-0000-0000-000000000001"]
    }
  ],
  "signals": []
}
```
Empty idle: `{ "features": [], "signals": [] }`.

### Core API
```rust
use feature_engine::{FeatureSnapshot, EngineOutput};
let snap = FeatureSnapshot::from_engine_output(&output);
// or FeatureSnapshot::empty()
```

## Risks / not covered
- Live Tauri `invoke` smoke (Dashboard UI → **P4-E1-T2**); worker must have run at least once for non-empty cache in a real session.
- If Feature Worker fails to start (DB soft-fail), snapshot stays empty and `get_feature_snapshot` still returns `[]` — expected idle.
- Time-series may include multiple windows per Feature id (engine step); chart aggregation is T3.
- Branch includes unmerged Menubar (`P3-E3-T3`) as base — merge/PR ordering for `main` is PM/git cluster concern.

## Notes for QA
- No frontend React consumer yet — verify Rust unit tests + that `get_status` JSON still has no `features` key.
- Focus on serde shape (camelCase) and absence of `payload` / `rmssd` / path strings in snapshot JSON.
