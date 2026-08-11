# Dev|UX → QA: P23-E3-T1

## Meta
- **Task ID:** P23-E3-T1
- **Title:** Dogfood notes + optional calm UI surface for Personal Context Layer
- **Role that built:** Dev|UX
- **Date:** 2026-08-11
- **AC source:** `docs/handoffs/P23-E3-T1-pm-brief.md` · ADR-024
- **Branch:** `phase/23-personal-context`

## What changed
- Dogfood runbook in `docs/12-development.md` § **DeskAwayPresence / health-context dogfood**:
  - DeskAway: emit walk/steps; omit quiet-alone / insufficient; no GPS; unit + live IPC; chart label **Away from desk**
  - Health→prompt: edit `~/.biofocus/health-context.toml`; pack path injects; empty = none; Feature values unchanged; **Phase 4 `build_report` does not auto-inject**
- **Optional Dashboard (thin reuse):**
  - Snapshot list already shows catalog Features (including `DeskAwayPresence`) — no gap
  - Chart allowlist + mock ready: `DeskAwayPresence` / **Away from desk**; omit → quiet
- **Health declare surface:** docs-only — edit config file (path clear in dogfood); no new Dashboard health editor
- Glossary + desktop README; Phase 23 status at E2–E3
- **No** Feature/leaf formula rewrite; **no** migration; **no** GPS; **no** literature library; **no** Insights/Recommendations rule

## Paths (state explicitly for AC3)
| Path | Status |
| :--- | :--- |
| IPC Snapshot list (`get_feature_snapshot`) | Already surfaces `DeskAwayPresence` when present |
| IPC series (`get_feature_series`) → chart | Now includes calm **Away from desk** series when present |
| Health declare | Docs-only: edit `~/.biofocus/health-context.toml` |
| Docs dogfood | § DeskAwayPresence / health-context dogfood |

## Crates / apps / files touched
- `apps/desktop/src/featureChart.ts` — chart allowlist + calm label
- `apps/desktop/src/featureSnapshot.ts` — mock ready series
- `apps/desktop/README.md`
- `docs/12-development.md`, `docs/16-glossary.md`
- **Not touched:** `crates/feature-engine/` / `crates/report-engine/` (math + health→prompt stay E2)

## How to verify (commands)
```bash
# Dogfood + calm framing
rg -n "DeskAwayPresence / health-context dogfood|Away from desk|away from desk in this window" \
  docs/12-development.md apps/desktop/src/featureChart.ts docs/16-glossary.md

# Clinical / GPS ban in UI chrome
rg -n "Away from desk" apps/desktop/src/featureChart.ts
# expect: calm label only — no GPS / surveillance / diagnosis

# Health pack note
rg -n "build_report does not|health-context.toml|User-declared" docs/12-development.md

# No formula rewrite this task
git diff --name-only -- crates/feature-engine/ crates/report-engine/
# expect: empty

cargo test -p feature-engine desk_away
cargo test -p report-engine health_context

cd apps/desktop && npx tsc --noEmit

# Browser smoke: ?view=dashboard&mockSnapshot=ready → Away from desk series
#                ?view=dashboard&mockSnapshot=empty → calm empty
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood notes — DeskAway emit/omit + health→prompt + non-pack caveat
- [ ] AC2: Calm framing only (docs + UI chrome)
- [ ] AC3: Optional Dashboard DeskAwayPresence + docs-only health declare; paths stated
- [ ] AC4: Smoke notes; UI ↛ SQLite
- [ ] AC5: No formula rewrite / migration / GPS / literature / Insights rule
- [ ] AC6: Handoff present
- [ ] Global DoD: personal self-tracking; PR freeze; UI↛DB

## Risks / not covered
- Physical-device live dogfood not re-run in this chat (unit + mock + docs cover AC).
- No in-app health-declare editor (docs-only by design for thin E3).

## Notes for QA
- Kanban Done / canvas / Phase 23 close / **PM-GATE-POST-P23** are PM-only after QA Pass.
- Do **not** open a PR (freeze until 2026-09-01).
