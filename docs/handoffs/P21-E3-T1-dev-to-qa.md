# Dev|UX → QA: P21-E3-T1

## Meta
- **Task ID:** P21-E3-T1
- **Title:** Dogfood notes + optional calm Dashboard surface for `DeepWorkScore`
- **Role that built:** Dev|UX
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P21-E3-T1-pm-brief.md` · ADR-022

## What changed
- Dogfood runbook in `docs/12-development.md` § **DeepWorkScore dogfood**: Focus required + optional CSR; Focus-only emit; omit without Focus; unit fixture; live IPC; calm framing; UI ↛ SQLite.
- **Optional Dashboard surface (thin reuse):**
  - Snapshot list already shows any catalog Feature from IPC (including `DeepWorkScore`) — no gap.
  - Chart allowlist extended: `DeepWorkScore` with calm label **Sustained focus** (0–100); omit → quiet.
  - `?mockSnapshot=ready` includes synthetic DeepWorkScore series for layout smoke.
- Glossary + desktop README notes; Phase 21 status bullet points at E2–E3.
- **No** Feature/leaf formula rewrite; **no** migration; **no** new Observation; **no** Insights/Recommendations rule; **no** Dashboard redesign.

## Paths (state explicitly for AC3)
| Path | Status |
| :--- | :--- |
| IPC Snapshot list (`get_feature_snapshot`) | Already surfaces `DeepWorkScore` when present |
| IPC series (`get_feature_series`) → chart | Now includes calm **Sustained focus** series when present |
| Docs dogfood | § DeepWorkScore dogfood |

## Crates / apps / files touched
- `apps/desktop/src/featureChart.ts` — chart allowlist + calm label
- `apps/desktop/src/featureSnapshot.ts` — mock ready series
- `apps/desktop/README.md`
- `docs/12-development.md`, `docs/16-glossary.md`
- Branch: `phase/21-deep-work-score`
- **Not touched:** `crates/feature-engine/` (math stays E2)

## How to verify (commands)
```bash
# Dogfood + calm framing
rg -n "DeepWorkScore dogfood|Sustained focus|sustained focus in this window" \
  docs/12-development.md apps/desktop/src/featureChart.ts docs/16-glossary.md

# Clinical ban in UI chrome
rg -n "Sustained focus" apps/desktop/src/featureChart.ts
# expect: label only — no flow/burnout/ADHD

# No formula rewrite this task
git diff --name-only -- crates/feature-engine/
# expect: empty

# Unit fixture still green
cargo test -p feature-engine deep_work_score

# Desktop typecheck
cd apps/desktop && npx tsc --noEmit

# Browser smoke (manual): ?view=dashboard&mockSnapshot=ready → Sustained focus series
#                         ?view=dashboard&mockSnapshot=empty → calm empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood notes in `12-development` — verify paths + Focus-only / omit-without-Focus notes
- [ ] AC2: Calm framing only (docs + UI chrome)
- [ ] AC3: Optional Dashboard — DeepWorkScore visible with calm label when present; omit quiet; paths stated
- [ ] AC4: Smoke notes in handoff; UI ↛ SQLite
- [ ] AC5: No formula rewrite / migration / new Observation / Insights rule
- [ ] AC6: Handoff present
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB

## Risks / not covered
- Physical-device live dogfood not re-run in this chat (unit + mock + docs cover AC).
- Chart legend density grows with more series (pre-existing note).

## Notes for QA
- Unrelated dirty PM-close docs may exist — out of AC unless they contradict E3.
- Kanban Done / canvas / Phase close are PM-only after QA Pass.
