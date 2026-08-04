# Dev → QA: P3-E1-T3

## Meta
- **Task ID:** P3-E1-T3
- **Title:** Normalization & calibration
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P3-E1 / P3-E1-T3; brief `docs/handoffs/P3-E1-T3-pm-brief.md`

## What changed
- Added normalize stage in `crates/pipeline`: canonical payload / unit calibration for known `data_type`s.
- New `PipelineStage::Normalized`; wire `normalize_observations` / `normalize_owned` / `normalize_deduped`.
- Unknown `data_type` → **pass-through** (kept unchanged). Unparseable known type → **skip** (`skipped_count`).
- Unit tests per known type + unknown contract + empty batch + intake→dedupe→normalize chain.
- No Feature scores, Menubar/UI, SQLite schema, or runtime worker.

### Entrypoint
```rust
use pipeline::{
    accept_observations, dedupe_accepted, normalize_deduped, DedupeState, PipelineStage,
};

let accepted = accept_observations(&batch)?;
let mut state = DedupeState::new();
let deduped = dedupe_accepted(&mut state, accepted)?;
let normalized = normalize_deduped(deduped)?;
assert_eq!(normalized.stage(), PipelineStage::Normalized);
```

Or slice-only: `normalize_observations(&[Observation])` → `NormalizedBatch`.

### Canonical rules (explicit)

| `data_type` | Canonical payload | Notes |
| :--- | :--- | :--- |
| `heart_rate` | `{ "bpm": f64, "source"?: string }` | aliases `hr` / `heart_rate` / `beats_per_minute`; `unit: "hz"` → ×60 |
| `hrv` | `{ "rmssd_ms": f64, "sdnn_ms"?: f64, "pnn50"?: f64 }` | aliases `rmssd` / `hrv_ms` / `hrv`; `unit: "s"` → ×1000 |
| `context_window` | `{ "bundle_id", "app_name" }` | aliases `bundleId` / `appName` / `name`; extras stripped |
| `keystrokes` | `{ "count", "window_secs", "rate_per_min" }` | default `window_secs=60`; rate recomputed |
| *(other)* | unchanged | **pass-through** |

SQLite Observation rows are never rewritten (in-memory pipeline view only).

## How to verify (commands)
```bash
cargo test -p pipeline
cargo check -p pipeline
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: stage normalizes known types (`heart_rate`, `hrv`, `context_window`, `keystrokes`) with explicit rules in code + tests
- [ ] AC2: unknown `data_type` → **pass-through** (documented here + module docs); unparseable known → skip
- [ ] AC3: unit tests for each known type + unknown contract
- [ ] AC4: idle-safe sync stage (no busy-loop / threads)
- [ ] AC5: no Feature scores / DAG, Menubar/UI, new SQLite table, runtime worker
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary (`Observation` payload only)

## Risks / not covered
- HRV payload shape not frozen in `docs/07-contracts.md` yet — T3 defines `rmssd_ms` canon for Feature Engine; may need docs sync later (PM).
- Alias / unit matrix is v1 (hz→bpm, s→ms only); exotic provider shapes may skip until extended.
- T2 notes still open: payload fingerprint key-order; seen-set bounds → T4/later.
- No commit in this handoff (cluster commit / PR later per git policy).

## Notes for QA
- Branch: `phase/3-pipeline-features` (from current `main`).
- Files: `crates/pipeline/src/{normalize,lib,error}.rs`, `crates/pipeline/Cargo.toml` (`serde_json` dep), this handoff.
- `expect` only in `#[cfg(test)]` helpers (same pattern as T1/T2).
- Worker wire → **P3-E1-T4**; Feature DAG → **P3-E2**.
