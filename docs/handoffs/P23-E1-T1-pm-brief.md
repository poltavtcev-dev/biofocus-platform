# PM Brief → Dev: P23-E1-T1

**From:** PM  
**To:** Dev  
**Status:** Done (QA Pass 2026-08-11)  
**Closed:** Epic **P23-E1** ✅ · **ADR-024** locked  
**Evidence:** `docs/handoffs/P23-E1-T1-qa-to-pm.md` · `docs/handoffs/P23-E1-T1-dev-to-qa.md`  
**Next:** **P23-E2-T1** — `docs/handoffs/P23-E2-T1-pm-brief.md`  
**Closed previous:** **PM-GATE-POST-P22** (chose Personal Context Layer; supersedes CircadianOffset draft)  
**Phase:** Phase 23 Personal Context Layer — Epic P23-E1  
**Branch:** `phase/23-personal-context`

## Task
**P23-E1-T1 — ADR-024: lock Personal Context Layer (Variant B + health context + desk-away)**

## Why
Focus-ladder Features are shipped. Next product gap is **interpretation context**: how we talk about numbers (personal + literature bands), optional user-declared conditions, and whether the person left the desk — without becoming a medical device or a GPS tracker.

## Acceptance Criteria
1. Record **ADR-024** in `docs/decision-log.md`: Phase 23 v1 primary = **Personal Context Layer** with three locked pillars:
   - **Reference bands = Variant B** — personal baseline primary; literature bands secondary (cited, calm orienting ranges); reject clinical “нельзя / diagnosis” tone.
   - **Health context** — opt-in **user-declared** curated conditions; v1 consumer = **prompt packs / report** (L5); may sketch later Insight soft-framing; **must not** invent disease from HR/HRV; **must not** rewrite Feature formulas from disease labels in v1.
   - **Desk-away / presence** — infer leave-desk / likely break-or-walk from **secondary signals** (prefer existing: quiet `keystrokes` / `context_window`, `step_count` cadence, life_event `walk`; optional thin new Observation only if ADR proves need). **Reject precise GPS / continuous geo** for v1.
2. Document relationships: Observations / Features / Insights / Recommendations / Prompt packs / local profile config (prefer file/config like Git folders over new SQLite unless ADR justifies).
3. Formula / product stance sketches for E2:
   - Case-catalog approach for reference-band Insights (not one mega-rule).
   - Health-context storage shape (local config; closed-set ids + optional free-text local-only).
   - Desk-away Feature or Insight candidate name + inputs + omit policy + calm copy (“away from desk in this window” — not surveillance).
4. Rejected alternatives documented (precise GPS; clinical diagnosis engine; LLM inventing conditions; cloud health records sync by default; workplace presence monitoring; rewriting Focus/Stress math from disease tags; CircadianOffset / IDE / weather / App Store as this-phase primary; PR during freeze; migration without approve).
5. Short epic sketch: E2 ships first implementation slice (ADR must pick order — prefer **desk-away signal path** and/or **health-context → prompt pack** before a large literature library); E3 dogfood / calm UI surface.
6. Update catalog / glossary stubs only as needed for names locked by ADR (no Feature math in E1).
7. Note **provider universality**: Features consume Observation contracts, not Apple-only APIs; Mac+iPhone+Watch is dogfood path, not architectural lock-in.
8. Handoff: `docs/handoffs/P23-E1-T1-dev-to-qa.md`.

## Out of scope
- Implementing Feature/Insight math or UI (→ **P23-E2** / **E3**)
- Android companion / Windows collectors as this-phase primary (universality via contracts only in ADR note)
- CircadianOffset Feature math (deferred)
- App Store / IDE / weather as primary
- Opening a PR (PR freeze until 2026-09-01)

## Constraints
- Global DoD from `docs/SPRINT_ROADMAP.md`
- Calm non-clinical framing; personal self-tracking only
- Local-First; no always-on precise geo
- LLM remains L5 interpret-only — may *consume* declared health context in prompts; must not diagnose
- Prefer no new SQLite schema; if profile needs persistence, prefer `~/.biofocus/…` config file (Git folders pattern) unless ADR + approve says otherwise
- **No migration** without explicit approve
- Branch: `phase/23-personal-context`

## After QA Pass
PM → mark P23-E1-T1 Done; Ready **P23-E2-T1** (first ship slice per ADR-024).
