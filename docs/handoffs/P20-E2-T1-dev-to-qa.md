# Dev → QA: P20-E2-T1

## Meta
- **Task ID:** P20-E2-T1
- **Title:** Ship catalog Feature `CognitiveLoad` per ADR-021
- **Role that built:** Dev
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P20-E2-T1-pm-brief.md` · ADR-021

## What changed
- New catalog node **`CognitiveLoadNode`** (`crates/feature-engine/src/catalog/cognitive_load.rs`):
  - Feature-level inputs: `MeetingDensity` + `ContextSwitchRate` + `NotificationPressure` (same window)
  - Normalize: `density×100`, `clamp(CSR×50,0,100)`, `NotificationPressure` as-is
  - Equal thirds; **renormalize** when ≥1 present; **omit** when none
  - ADR-007: expected slots = 3; `coverage × mean(upstream Feature.confidence)`
  - Explanation factors: `meeting` / `switches` / `notifications` (calm labels)
  - Windows driven from unique upstream Feature window ends (calendar meetings can span beyond Observation.timestamp)
- `register_cognitive_v1` + wired into `register_catalog_v1` (after notification)
- Catalog §1.16 finalized (shipped); glossary + `12-development` notes updated
- Unit tests: rich all-three · partial notify · partial meeting · empty omit · confidence · catalog register
- **No** leaf Feature formula changes; **no** migration; **no** UI/Dashboard

## Crates / apps / files touched
- `crates/feature-engine/src/catalog/cognitive_load.rs` (new)
- `crates/feature-engine/src/catalog/mod.rs`
- `crates/feature-engine/src/lib.rs`
- `docs/06-feature-catalog.md`, `docs/12-development.md`, `docs/16-glossary.md`
- Branch: `phase/20-cognitive-load`

## How to verify (commands)
```bash
cargo test -p feature-engine cognitive_load
cargo test -p feature-engine --lib

# Registration + catalog
rg -n "CognitiveLoad|register_cognitive_v1" \
  crates/feature-engine/src/catalog/mod.rs \
  crates/feature-engine/src/lib.rs \
  docs/06-feature-catalog.md

# Leaf formulas untouched
git diff --name-only -- \
  crates/feature-engine/src/catalog/meeting_density.rs \
  crates/feature-engine/src/catalog/context_switch_rate.rs \
  crates/feature-engine/src/catalog/notification_pressure.rs
# expect: empty

# No migration / no new data_type
rg -n "CognitiveLoad" crates/bio-spec || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: CognitiveLoad in feature-engine; Feature-level inputs; 15m/1m; 0–100
- [ ] AC2: Normalization + equal thirds; renormalize ≥1; omit none
- [ ] AC3: ADR-007 slots=3; coverage × mean(upstream.confidence)
- [ ] AC4: Factors meeting/switches/notifications; calm labels (no clinical copy)
- [ ] AC5: Registered after calendar+focus+notification; catalog §1.16 shipped
- [ ] AC6: Unit tests rich / partial / omit / confidence / factors
- [ ] AC7: No leaf rewrite; no new Observation; no migration; no UI
- [ ] AC8: Handoff present
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; PR freeze

## Risks / not covered
- Dashboard / dogfood surface deferred to **P20-E3**.
- Weight tuning stays equal thirds per ADR-021 (not reopened here).

## Notes for QA
- Partial NotificationPressure-only emit is intentional (opt-in often off).
- Kanban Done / canvas are PM-only after QA Pass.
