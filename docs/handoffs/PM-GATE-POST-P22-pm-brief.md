# PM Brief → PM: PM-GATE-POST-P22 (closed)

**From:** PM (close of Phase 22 AttentionStability)  
**To:** PM (this gate)  
**Status:** Done (2026-08-11) — chose **Personal Context Layer** → Phase 23 opened; Ready **P23-E1-T1**  
**Date:** 2026-08-11  
**Closed previous:** Phase 22 E1–E3 (ADR-023 → `AttentionStability` Feature → dogfood + Focus stability chart)  
**Evidence:** `docs/handoffs/P22-E3-T1-qa-to-pm.md` · `docs/handoffs/P22-*-qa-to-pm.md`  
**Outcome:** Deferred IDE, weather ambient, App Store packaging product, Companion polish-as-primary, CircadianOffset-as-primary, TypingRhythm, precise GPS. Next brief: `docs/handoffs/P23-E1-T1-pm-brief.md`.  
**Branch note:** Phase 22 cluster stays on `phase/22-attention-stability` (PR after freeze). Phase 23 on `phase/23-personal-context`.  
**Supersedes:** earlier same-day CircadianOffset gate draft — user-directed Product decision favors Personal Context Layer (Variant B + health context + desk-away).

## Task
**PM-GATE-POST-P22 — Choose next Phase 23+ slice**

## Decision (locked)
**Primary Phase 23 track = Personal Context Layer** — lock via **ADR-024** three pillars (Variant **B** framing throughout):

1. **Reference bands (Variant B)** — personal baseline first; literature bands second as calm orienting ranges with cited sources; **not** “нельзя / диагноз”.
2. **User-declared health context** — opt-in curated conditions the user already knows → primarily **prompt packs / report context** (L5); Insights later only with Evidence; **never** invent diagnosis from biometrics.
3. **Desk-away / leave-computer presence** — infer “away from desk / likely walk or break” from **secondary signals** (idle input, quiet `context_window`, `step_count` cadence, life_event `walk`, optional screen-lock later) — **no precise GPS / continuous geo** in v1.

| Candidate | Verdict |
| :--- | :--- |
| IDE plugin | Deferred — still no additive privacy-safe signal beyond `context_window` |
| Weather ambient | Deferred — cloud / geo vs Local-First |
| App Store packaging product | Deferred — commercial track beyond P12 runbook |
| Companion polish (as primary) | Parallel OK — not Phase 23 theme |
| CircadianOffset | Deferred — chronotype framing riskier; after context layer |
| TypingRhythm | Deferred |
| Precise GPS walk tracking | **Rejected for v1** — precise geo dumps forbidden; secondary signals first |
| **Personal Context Layer** | **Chosen** — user-directed; unlocks interpretation + presence without medical/GPS product |

## Universality note (product stance)
- **Core path is universal:** any source posting valid `Observation` JSON to local ingest works (`provider_id` + `data_type` contracts).
- **Dogfood path today:** Mac Desktop + iOS Companion + Apple Health (Apple Watch; Mi Band via Health OK).
- **Not Mac/iPhone-only forever:** new platforms = new Capability plugins / companions behind the same contracts — Features/Insights stay provider-agnostic.
- Phase 23 must **not** hard-code “only Apple Watch” into Features.

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 23 v1 track — **done** (Personal Context Layer).
2. Open Phase 23 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P23-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 23 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** — Phase 22 on `phase/22-attention-stability`; Phase 23 on `phase/23-personal-context`.

## After gate
Paste **build-qa** command for **P23-E1-T1**.
