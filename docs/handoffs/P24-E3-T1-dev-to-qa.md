# Dev|UX → QA: P24-E3-T1

## Meta
- **Task ID:** P24-E3-T1
- **Title:** Dogfood notes + optional calm UI surface for `CircadianOffset`
- **Role that built:** Dev|UX
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P24-E3-T1-pm-brief.md` · ADR-025
- **Branch:** `phase/24-circadian-offset`

## What changed
- Dogfood runbook in `docs/12-development.md` § **CircadianOffset dogfood**:
  - Both-slot emit (sleep + work/activity); omit unless both; 15m/1m + 24h lookback
  - Align-near-mid-wake → high; large circular offset → toward 0 (high-level)
  - Unit fixture + live IPC; calm framing; UI ↛ SQLite; distinct from SleepDebt / DeskAwayPresence
- **Optional Dashboard (thin reuse):**
  - Snapshot list already shows catalog Features (including `CircadianOffset`) via IPC — no gap
  - Chart allowlist + mock ready: `CircadianOffset` / calm label **Schedule alignment**; omit → quiet
- Glossary + catalog chart note + desktop README
- **No** Feature/leaf/sibling formula rewrite; **no** migration; **no** new Observation; **no** Insights/Recommendations rule

## Paths (state explicitly for AC3)
| Path | Status |
| :--- | :--- |
| IPC Snapshot list (`get_feature_snapshot`) | Already surfaces `CircadianOffset` when present (E2 DAG) |
| IPC series (`get_feature_series`) → chart | Now includes calm **Schedule alignment** series when present |
| Docs dogfood | § CircadianOffset dogfood |
| Browser mock | `?view=dashboard&mockSnapshot=ready` includes CircadianOffset series |

## Crates / apps / files touched
- `apps/desktop/src/featureChart.ts` — chart allowlist + calm label **Schedule alignment**
- `apps/desktop/src/featureSnapshot.ts` — mock ready series
- `apps/desktop/README.md`
- `docs/12-development.md`, `docs/16-glossary.md`, `docs/06-feature-catalog.md`
- **Not touched:** `crates/feature-engine/` (math stays E2)

## How to verify (commands)
```bash
# Dogfood + calm framing
rg -n "CircadianOffset dogfood|Schedule alignment|schedule alignment in this window" \
  docs/12-development.md apps/desktop/src/featureChart.ts docs/16-glossary.md

# Clinical ban in UI chrome
rg -n "Schedule alignment" apps/desktop/src/featureChart.ts
# expect: calm label only — no chronotype / night owl / disorder

# Distinct sibling chart labels still present
rg -n "Sleep shortfall|Away from desk|Schedule alignment" apps/desktop/src/featureChart.ts

# No formula rewrite this task
git diff --name-only -- crates/feature-engine/
# expect: empty (vs HEAD / unstaged)

cargo test -p feature-engine circadian

cd apps/desktop && npx tsc --noEmit

# Browser smoke: ?view=dashboard&mockSnapshot=ready → Schedule alignment series
#                ?view=dashboard&mockSnapshot=empty → calm empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood notes — emit both slots / omit / 15m+24h / align vs large offset (high-level)
- [ ] AC2: Calm framing only (docs + UI chrome); distinct from SleepDebt / DeskAwayPresence
- [ ] AC3: Optional Dashboard Schedule alignment when present; omit quiet; paths stated
- [ ] AC4: Smoke notes; UI ↛ SQLite
- [ ] AC5: No CircadianOffset / leaf / sibling formula rewrite; no migration; no new Observation; no Insights rule
- [ ] AC6: Handoff present
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB

## Risks / not covered
- Physical-device live dogfood (sleep + desk over 24h) not re-run in this chat — unit + mock + docs cover AC.
- No new CircadianOffset product window / Dashboard redesign (out of scope).

## Notes for QA
- Kanban Done / canvas / Phase 24 close / **PM-GATE-POST-P24** are PM-only after QA Pass.
- Do **not** open a PR (freeze until 2026-09-01).
