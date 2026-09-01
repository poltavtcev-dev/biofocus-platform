# QA → PM: P27-E1-T1

## Meta
- **Task ID:** P27-E1-T1
- **Title:** ADR-028: lock Pattern Discovery / Recommendations expansion scope (which rules, Feature inputs, non-goals)
- **Date:** 2026-08-12
- **Dev/UX handoff:** `docs/handoffs/P27-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
| Check | Result |
| :--- | :--- |
| `rg ADR-028` in decision-log / 12-development / glossary | **Pass** |
| Locked slate ids + CognitiveLoad / SustainedLoadIndicator | **Pass** |
| Rejected alts (Feature math / TypingRhythm / DeepFocusLikelihood / IDE / weather / App Store / GPS / LLM-authored / workplace / PR freeze / migration / public launch Done / parallel Coach) | **Pass** |
| `rg` slate ids in `crates/knowledge-engine` | **Pass** — no matches (impl deferred to E2) |
| `git diff --name-only -- crates/ apps/` | **Pass** — empty |
| Evaluate-on-read + no Insight/Rec SQLite + schema none + E2/E3 sketch | **Pass** |
| Handoff present | **Pass** |

### AC results
| AC | Result | Notes |
| :--- | :--- | :--- |
| AC1 ADR-028 primary = rule expansion; evaluate-on-read; no store; personal framing | **Pass** | |
| AC2 Explicit E2 slate (3 rules) + inputs/trigger/omit/Evidence/calm; omit others justified | **Pass** | |
| AC3 Rejected alternatives | **Pass** | |
| AC4 Schema none | **Pass** | |
| AC5 E2 register helpers; E3 dogfood sketched | **Pass** | |
| AC6 Docs planned/ADR notes | **Pass** | `12` + `16` name slate |
| AC7 Handoff | **Pass** | |
| Global DoD | **Pass** | no E1 impl; PR freeze; public launch not Done |

### Extra checks
- Slate stays within 1–3 new rules (exactly 3).
- Distinctness of SustainedLoad vs CognitiveLoad / High_Stress documented.

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P27-E1-T1** to Done; Ready **P27-E2-T1** (ship locked rules)
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: note ADR-028 locked (vision/roadmap already mention Phase 27)
- [x] Do **not** open a PR; do **not** claim public launch Done

## Suggested next Ready task
- **P27-E2-T1** — Ship locked rules in `knowledge-engine`: `cognitive_load_elevated_v1`, `sustained_load_elevated_v1`, `combined_demand_pace_hint_v1` per ADR-028 (register via existing helpers; unit tests; no Feature math / no migration).

## Notes for PM
- Branch: `phase/27-pattern-rules`.
- E2 must not add rules outside the locked slate without ADR amend + approve.
- Thresholds sketched at ≥60 with ±10 E2 tune band.
