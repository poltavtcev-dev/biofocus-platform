# PM Brief → PM: PM-GATE-POST-P26 (closed)

**From:** PM (close of Phase 26 OSS Public Launch Hygiene)  
**To:** PM (this gate)  
**Status:** Done (2026-08-12) — chose **Pattern Discovery rule expansion** → Phase 27 opened; Ready **P27-E1-T1**  
**Date:** 2026-08-12  
**Closed previous:** Phase 26 E1–E3 (**ADR-027** → SoT `docs/19-oss-public-launch.md` → dry-run checklist)  
**Evidence:** `docs/handoffs/P26-E3-T1-qa-to-pm.md` · `docs/handoffs/P26-*-qa-to-pm.md`  
**Outcome:** Deferred park-until-freeze-lift as primary; IDE · weather · App Store · Companion polish-as-primary · TypingRhythm · DeepFocusLikelihood · precise GPS · Feature-math. OSS layers 2–3 remain **parked ops** (after 2026-09-01) — **public launch not Done**. Next brief: `docs/handoffs/P27-E1-T1-pm-brief.md`.  
**Branch note:** Phase 26 stays on `phase/26-oss-public-launch`. Phase 27 on `phase/27-pattern-rules`.

## Task
**PM-GATE-POST-P26 — Choose next Phase 27+ primary track**

## Decision (locked)
**Primary Phase 27 track = Pattern Discovery rule expansion** — lock scope via **ADR-028** → ship additional deterministic **InsightRule** / **RecommendationRule**s in `knowledge-engine` on **already shipped** Features (Phases 10–25 catalog) → optional dogfood / Dashboard Insights·Suggestions surface. Extend ADR-008 / ADR-009 evaluate-on-read path. **No** new Observation / Feature catalog math. **No** App Store. **Public launch still not Done** (OSS layers 2–3 after freeze).

Rationale: Catalog Feature backlog has no safe distinct math left. Plugin/ambient/commercial candidates stay behind prior ADR bars. Parking solely for freeze lift would idle ~3 weeks of productive local work. North star is Personal Pattern Discovery — v1 only has three Insight rules + one Recommendation; many shipped Features (DistractionScore, NotificationPressure, CognitiveLoad, DeepWorkScore, AttentionStability, DeskAwayPresence, CircadianOffset, SustainedLoadIndicator, …) are under-used by Knowledge. Highest leverage during freeze = deepen L3/L4 on existing Evidence.

| Candidate | Verdict |
| :--- | :--- |
| Park / wait for freeze lift (OSS layers 2–3 only) | Deferred as **primary** — layers 2–3 stay parked ops after 2026-09-01; not a Feature/product phase |
| **Pattern Discovery rule expansion** | **Chosen** — north star; uses shipped Features; no new sensors / math |
| IDE plugin | Deferred — ADR-013 bar |
| Weather ambient | Deferred — ADR-015 bar |
| App Store listing / productization | Deferred — vision keeps App Store deferred |
| Companion polish (as primary) | Parallel OK — not Phase 27 theme |
| TypingRhythm | Deferred — surveillance framing risk |
| DeepFocusLikelihood | Deferred — overlaps DeepWorkScore |
| Precise GPS | **Rejected** — ADR-024 bar |
| Feature-math “other” | Deferred — no distinct catalog node |
| New Observation family | Rejected for v1 — prefer zero |

## Universality note
- Core path remains provider-agnostic Observation / Feature contracts.
- Dogfood path today: Mac Desktop + iOS Companion + Apple Health.
- Phase 27 must **not** invent clinical labels, workplace manager dashboards, cloud accounts by default, or secret Core math.
- Do **not** treat unsigned dry-run notes as notarized Release / public launch Done.

## Acceptance Criteria (gate)
1. Pick **one** primary Phase 27 v1 track — **done** (Pattern Discovery rule expansion).
2. Open Phase 27 in `docs/SPRINT_ROADMAP.md` + `docs/14-roadmap.md` — **done**.
3. Write `docs/handoffs/P27-E1-T1-pm-brief.md` — **done**.
4. Update execution canvas to Phase 27 Ready — **done**.
5. Reaffirm **PR freeze until 2026-09-01** and **public launch not Done** — Phase 27 on `phase/27-pattern-rules`.

## After gate
Paste **build-qa** command for **P27-E1-T1**.
