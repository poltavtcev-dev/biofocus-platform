# Dev|UX → QA: P25-E3-T1

## Meta
- **Task ID:** P25-E3-T1
- **Title:** Dogfood notes + optional calm UI surface for `SustainedLoadIndicator`
- **Role that built:** Dev|UX
- **Date:** 2026-08-12
- **AC source:** `docs/handoffs/P25-E3-T1-pm-brief.md` · ADR-026
- **Branch:** `phase/25-sustained-load`

## What changed
- Dogfood runbook in `docs/12-development.md` § **SustainedLoadIndicator dogfood**:
  - Stress + Fatigue over **4h lookback**; MeetingDensity optional (renormalize)
  - **Omit** when both Stress and Fatigue absent (meetings-alone must not emit)
  - Cadence **15m / 1m**; unit fixture + live IPC; calm framing; UI ↛ SQLite
  - Distinct from CognitiveLoad (“combined demand”)
- **Optional Dashboard (thin reuse):**
  - Snapshot list already shows catalog Features (including `SustainedLoadIndicator`) via IPC — no gap
  - Chart allowlist + mock ready: `SustainedLoadIndicator` / calm label **Prolonged load**; omit → quiet
- Glossary + catalog chart note + desktop README
- **No** Feature/leaf/sibling/CognitiveLoad formula rewrite; **no** migration; **no** new Observation; **no** Insights/Recommendations rule

## Paths (state explicitly for AC3)
| Path | Status |
| :--- | :--- |
| IPC Snapshot list (`get_feature_snapshot`) | Already surfaces `SustainedLoadIndicator` when present (E2 DAG) |
| IPC series (`get_feature_series`) → chart | Now includes calm **Prolonged load** series when present |
| Docs dogfood | § SustainedLoadIndicator dogfood |
| Browser mock | `?view=dashboard&mockSnapshot=ready` includes SustainedLoadIndicator series |

## Crates / apps / files touched
- `apps/desktop/src/featureChart.ts` — chart allowlist + calm label **Prolonged load**
- `apps/desktop/src/featureSnapshot.ts` — mock ready series
- `apps/desktop/README.md`
- `docs/12-development.md`, `docs/16-glossary.md`, `docs/06-feature-catalog.md`
- **Not touched:** `crates/feature-engine/` (math stays E2)

## How to verify (commands)
```bash
# Dogfood + calm framing
rg -n "SustainedLoadIndicator dogfood|Prolonged load|prolonged load in this window" \
  docs/12-development.md apps/desktop/src/featureChart.ts docs/16-glossary.md

# Clinical ban in UI chrome
rg -n "Prolonged load" apps/desktop/src/featureChart.ts
# expect: calm label only — no burnout / burned out

# Distinct sibling chart labels still present
rg -n "Combined demand|Prolonged load" apps/desktop/src/featureChart.ts

# No formula rewrite this task
git diff --name-only -- crates/feature-engine/
# expect: empty (vs HEAD / unstaged)

cargo test -p feature-engine sustained_load

cd apps/desktop && npx tsc --noEmit

# Browser smoke: ?view=dashboard&mockSnapshot=ready → Prolonged load series
#                ?view=dashboard&mockSnapshot=empty → calm empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood notes — Stress+Fatigue (+ optional MeetingDensity) / 4h lookback / omit when both Stress&Fatigue absent / 15m+1m / renormalize Meeting absent (high-level)
- [ ] AC2: Calm framing only (docs + UI chrome); distinct from CognitiveLoad
- [ ] AC3: Optional Dashboard Prolonged load when present; omit quiet; paths stated
- [ ] AC4: Smoke notes; UI ↛ SQLite
- [ ] AC5: No SustainedLoadIndicator / leaf / sibling / CognitiveLoad formula rewrite; no migration; no new Observation; no Insights rule
- [ ] AC6: Handoff present
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB

## Risks / not covered
- Physical-device live dogfood (Stress+Fatigue over 4h + optional calendar) not re-run in this chat — unit + mock + docs cover AC.
- No new SustainedLoad product window / Dashboard redesign (out of scope).

## Notes for QA
- Kanban Done / canvas / Phase 25 close / **PM-GATE-POST-P25** are PM-only after QA Pass.
- Do **not** open a PR (freeze until 2026-09-01).
