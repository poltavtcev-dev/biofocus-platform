# PM Brief → Dev: P23-E2-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P23-E2** ✅ · `DeskAwayPresence` + health→prompt shipped  
**Evidence:** `docs/handoffs/P23-E2-T1-qa-to-pm.md` · `docs/handoffs/P23-E2-T1-dev-to-qa.md`  
**Next:** **P23-E3-T1** — `docs/handoffs/P23-E3-T1-pm-brief.md`  
**Closed previous:** P23-E1-T1 (ADR-024 locked; QA Pass)  
**Phase:** Phase 23 Personal Context Layer — Epic P23-E2  
**Branch:** `phase/23-personal-context`

## Task
**P23-E2-T1 — Ship first slice per ADR-024: `DeskAwayPresence` (+ health→prompt if capacity)**

## Why
ADR-024 locked Personal Context Layer with E2 ship order: **PRIMARY** desk-away Feature from existing secondary signals; **SECONDARY** health-context local config + L5 prompt/report injection; large literature library deferred. E1 stubs are in place (`DeskAwayPresence` §1.19). Implement without GPS, diagnosis, or Feature math branching on health labels.

## Acceptance Criteria
1. **PRIMARY — implement `DeskAwayPresence`** in `feature-engine` per ADR-024:  
   - Window prefer **15m / 1m**; output **0–100** or omit.  
   - Inputs from existing secondary signals (quiet/absent keystrokes; quiet/absent `context_window`; optional `step_count` cadence; optional `life_event` `walk`).  
   - **Omit** when evidence cannot distinguish idle-at-desk vs away (no false presence claims).  
   - **No** precise GPS / continuous geo; **no** new Observation `data_type` unless ADR amend + approve.  
   - ADR-007 confidence + optional calm factors (`input_quiet` / `steps` / `walk_event`).  
   - Calm copy: “away from desk in this window” — not workplace surveillance.  
   - Register in catalog / DAG; finalize `docs/06-feature-catalog.md` §1.19 (stub → shipped).  
   - Unit tests: emit with evidence · omit on insufficient · no GPS path · factors when emitted.
2. **SECONDARY (same task if capacity; else note deferral in handoff with concrete leftover):**  
   - Local opt-in `~/.biofocus/health-context.toml` (or equivalent under `~/.biofocus/`).  
   - Closed-set condition ids + optional free-text note; missing/empty = no injection.  
   - Wire **prompt-pack / report** consume-only (L5); **must not** branch Feature math on condition ids.  
   - No cloud health sync; no SQLite migration.
3. **Defer:** large literature-band library (at most thin Variant B case stub if trivial; do not rebuild ADR-008 personal baselines).
4. Docs: `12-development` / `16-glossary` ship notes for DeskAwayPresence (+ health config if shipped).
5. **Do not** invent clinical diagnosis; **do not** rewrite Focus/Stress/leaf formulas from disease tags; **no** UI/Dashboard surface (→ E3); **no** CircadianOffset; **no** PR during freeze.
6. Handoff: `docs/handoffs/P23-E2-T1-dev-to-qa.md` — state clearly whether secondary health→prompt landed or is leftover for follow-on.

## Out of scope
- Dogfood / Dashboard / health declare UI (→ **P23-E3**)
- Large literature Insight library
- Precise GPS / continuous geo
- Workplace presence / manager dashboards
- Opening a PR (PR freeze until 2026-09-01)
- Migration without approve

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md` (incl. personal-context DoD)
- Prefer extend `feature-engine` (+ `report-engine` / packs for secondary) — no parallel crates
- Branch: `phase/23-personal-context`
- Provider-agnostic Observation contracts (not Apple-only)
- LLM remains L5 interpret-only — may consume declared health context; must not diagnose
- **No migration**

## After QA Pass
PM → mark P23-E2-T1 Done; Ready **P23-E3-T1** (dogfood / optional calm UI).
