# QA → PM: P27-E2-T1

## Meta
- **Task ID:** P27-E2-T1
- **Title:** Ship locked Insight/Recommendation rules in `knowledge-engine` per ADR-028
- **Date:** 2026-08-12
- **Dev/UX handoff:** `docs/handoffs/P27-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
| Check | Result |
| :--- | :--- |
| `cargo test -p knowledge-engine` | **Pass** — 46/46 |
| `rg` rule ids in crate + `12-development` / `16-glossary` / `09-api` | **Pass** — all three ids present as shipped |
| `rg` FeatureEngine / catalog register helpers in `knowledge-engine` | **Pass** — none (Feature ids as string inputs only) |
| `unwrap`/`expect` in new rule prod paths | **Pass** — only in `#[cfg(test)]` helpers |
| Existing v1 rule modules still present / ids unchanged | **Pass** |
| `feature-engine` / migration in E2 commit `a7f5971` | **Pass** — no Feature-engine diff; no migration/sql |
| Extra rules outside ADR-028 slate | **Pass** — only the three locked ids added |

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 Exactly three rules via existing register helpers; triggers / Evidence / calm copy | **Pass** | Insights ≥60 CognitiveLoad / SustainedLoadIndicator; Rec after demand Insight |
| AC2 No rule outside locked slate | **Pass** | |
| AC3 Existing v1 rules not rewritten beyond registration wiring | **Pass** | high_stress / context_switch / focus_baseline / focus_dip_pace body untouched |
| AC4 Unit tests emit/omit/Evidence/calm/threshold/Rec gating | **Pass** | per-rule + integration in `rules/mod.rs` |
| AC5 No migration / Observation / Feature math / Insight-Rec store | **Pass** | |
| AC6 Docs 12 / 16 (+ 09-api knowledge notes) E2 ship | **Pass** | |
| Global DoD / Pattern rules DoD | **Pass** | evaluate-on-read; personal calm framing |

### Extra checks
- Categories: `demand` / `prolonged_load` / Rec `pace` — distinct from High_Stress `stress`.
- Recommendation omits on prolonged-load-only Insight (unit covered).
- Host already registers helpers → new rules auto-pick up; no desktop IPC change required for AC.

## Defects (if any)
- None blocking.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — P27-E2-T1 → Done; Ready **P27-E3-T1** (dogfood + optional calm Insights/Suggestions surface)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: `ARCHITECTURE_STATUS` / `14-roadmap` / `PROJECT_CANVAS` — Phase 27 E2 Done · E3 Ready; do **not** claim public launch Done
- [x] Optional: confirm `09-api` / glossary ship notes already on branch (Dev updated)
- [x] Note: working tree may already show Done wording from a prior docs pass — reconcile + still refresh canvas if not synced

**PM close (2026-08-12):** QA Pass accepted. Kanban / canvas / status docs reconciled (E2 Done · E3 Ready). Brief for next: `docs/handoffs/P27-E3-T1-pm-brief.md`. No PR (freeze). Public launch not Done.

## Suggested next Ready task
- **P27-E3-T1** — Dogfood notes + optional calm Dashboard Insights/Suggestions surface for the new rule categories (reuse existing IPC; omit stays quiet). Brief may already exist: `docs/handoffs/P27-E3-T1-pm-brief.md`.

## Notes for PM
- Branch: `phase/27-pattern-rules` (PR freeze — no PR).
- Thresholds shipped at **60** (within ADR ±10).
- Dashboard may show raw `demand` / `prolonged_load` until E3 label polish — not a Fail for E2.
- This chat did **not** mark Kanban Done / canvas (build-qa stop).
