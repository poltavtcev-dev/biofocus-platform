# Dev|UX → QA: P22-E3-T1

## Meta
- **Task ID:** P22-E3-T1
- **Title:** Dogfood notes + optional calm Dashboard surface for `AttentionStability`
- **Role that built:** Dev|UX
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P22-E3-T1-pm-brief.md` · ADR-023
- **Branch:** `phase/22-attention-stability`

## What changed
- Dogfood runbook in `docs/12-development.md` § **AttentionStability dogfood**: Focus required + optional CSR; Focus-only emit; omit without Focus; single-vs-multi Focus sample behavior (range vs 100); unit fixture; live IPC; calm framing; UI ↛ SQLite; distinct from DeepWorkScore.
- **Optional Dashboard surface (thin reuse):**
  - Snapshot list already shows any catalog Feature from IPC (including `AttentionStability`) — no gap.
  - Chart allowlist extended: `AttentionStability` with calm label **Focus stability** (0–100); omit → quiet.
  - `?mockSnapshot=ready` includes synthetic AttentionStability series for layout smoke.
- Glossary + desktop README notes; Phase 22 status bullet points at E2–E3.
- **No** Feature/leaf/DeepWorkScore formula rewrite; **no** migration; **no** new Observation; **no** Insights/Recommendations rule; **no** Dashboard redesign; **no** `09-api` change (no IPC gap).

## Paths (state explicitly for AC3)
| Path | Status |
| :--- | :--- |
| IPC Snapshot list (`get_feature_snapshot`) | Already surfaces `AttentionStability` when present |
| IPC series (`get_feature_series`) → chart | Now includes calm **Focus stability** series when present |
| Docs dogfood | § AttentionStability dogfood |

## Crates / apps / files touched
- `apps/desktop/src/featureChart.ts` — chart allowlist + calm label
- `apps/desktop/src/featureSnapshot.ts` — mock ready series
- `apps/desktop/README.md`
- `docs/12-development.md`, `docs/16-glossary.md`
- **Not touched:** `crates/feature-engine/` (math stays E2)

## How to verify (commands)
```bash
# Dogfood + calm framing
rg -n "AttentionStability dogfood|Focus stability|focus stability in this window" \
  docs/12-development.md apps/desktop/src/featureChart.ts docs/16-glossary.md

# Clinical ban in UI chrome — label only
rg -n "Focus stability" apps/desktop/src/featureChart.ts
# expect: calm label — no ADHD / “can’t focus” / burnout

# Distinct from DeepWork sibling label
rg -n "Sustained focus|Focus stability" apps/desktop/src/featureChart.ts

# No formula rewrite this task
git diff --name-only -- crates/feature-engine/
# expect: empty

# Unit fixture still green
cargo test -p feature-engine attention_stability

# Desktop typecheck
cd apps/desktop && npx tsc --noEmit

# Browser smoke (manual): ?view=dashboard&mockSnapshot=ready → Focus stability series
#                         ?view=dashboard&mockSnapshot=empty → calm empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood notes in `12-development` — verify paths + Focus-only / omit / single-vs-multi notes
- [ ] AC2: Calm framing only (docs + UI chrome); distinct from DeepWork “sustained focus”
- [ ] AC3: Optional Dashboard — AttentionStability visible with calm label when present; omit quiet; paths stated
- [ ] AC4: Smoke notes in handoff; UI ↛ SQLite
- [ ] AC5: No formula rewrite / migration / new Observation / Insights rule
- [ ] AC6: Handoff present
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB

## Risks / not covered
- Physical-device live dogfood not re-run in this chat (unit + mock + docs cover AC).
- Chart legend density grows with more series (pre-existing note).

## Notes for QA
- Kanban Done / canvas / Phase 22 close / **PM-GATE-POST-P22** are PM-only after QA Pass.
- Do **not** open a PR (freeze until 2026-09-01).
