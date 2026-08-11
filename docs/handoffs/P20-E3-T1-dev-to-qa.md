# Dev|UX → QA: P20-E3-T1

## Meta
- **Task ID:** P20-E3-T1
- **Title:** Dogfood notes + optional calm Dashboard surface for `CognitiveLoad`
- **Role that built:** Dev|UX
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P20-E3-T1-pm-brief.md` · ADR-021

## What changed
- Dogfood runbook in `docs/12-development.md` § **CognitiveLoad dogfood**: leaf inputs (MeetingDensity / CSR / optional NotificationPressure); partial emit when notifications opt-in off; unit fixture (`cargo test -p feature-engine cognitive_load`); live IPC `get_feature_snapshot` / `get_feature_series`; calm framing; UI ↛ SQLite.
- **Optional Dashboard surface (thin reuse):**
  - Snapshot list already shows any catalog Feature from IPC (including `CognitiveLoad`) — no gap.
  - Chart allowlist extended: `CognitiveLoad` with calm label **Combined demand** (0–100); omit → quiet (series absent).
  - `?mockSnapshot=ready` includes synthetic CognitiveLoad series for layout smoke.
- Glossary + desktop README notes; Phase 20 status bullet points at E2–E3.
- **No** Feature/leaf formula rewrite; **no** migration; **no** new Observation; **no** Insights/Recommendations rule; **no** Dashboard redesign.

## Paths (state explicitly for AC3)
| Path | Status |
| :--- | :--- |
| IPC Snapshot list (`get_feature_snapshot`) | Already surfaces `CognitiveLoad` when present |
| IPC series (`get_feature_series`) → chart | Now includes calm **Combined demand** series when present |
| Docs dogfood | § CognitiveLoad dogfood |

## Crates / apps / files touched
- `apps/desktop/src/featureChart.ts` — chart allowlist + calm label
- `apps/desktop/src/featureSnapshot.ts` — mock ready series
- `apps/desktop/README.md`
- `docs/12-development.md`, `docs/16-glossary.md`
- Branch: `phase/20-cognitive-load`
- **Not touched:** `crates/feature-engine/` (math stays E2)

## How to verify (commands)
```bash
# Dogfood + calm framing
rg -n "CognitiveLoad dogfood|Combined demand|combined demand in this window" \
  docs/12-development.md apps/desktop/src/featureChart.ts docs/16-glossary.md

# Clinical ban in UI chrome
rg -n "Combined demand" apps/desktop/src/featureChart.ts
# expect: label only — no overloaded/burnout/ADHD

# No formula rewrite this task
git diff --name-only -- crates/feature-engine/
# expect: empty

# Unit fixture still green
cargo test -p feature-engine cognitive_load

# Desktop typecheck
cd apps/desktop && npx tsc --noEmit

# Browser smoke (manual): ?view=dashboard&mockSnapshot=ready → Combined demand series
#                         ?view=dashboard&mockSnapshot=empty → calm empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood notes in `12-development` — verify paths + partial emit note
- [ ] AC2: Calm framing only (docs + UI chrome)
- [ ] AC3: Optional Dashboard — CognitiveLoad visible with calm label when present; omit quiet; paths stated
- [ ] AC4: Smoke notes in handoff; UI ↛ SQLite
- [ ] AC5: No formula rewrite / migration / new Observation / Insights rule
- [ ] AC6: Handoff present
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB

## Risks / not covered
- Physical-device live dogfood with all three leaves not re-run in this chat (unit + mock + docs cover AC).
- Chart legend density grows with more series (pre-existing P17 note).

## Notes for QA
- Unrelated dirty PM-close docs (roadmap/canvas status from E2) may exist — out of AC unless they contradict E3.
- Kanban Done / canvas / Phase close are PM-only after QA Pass.
