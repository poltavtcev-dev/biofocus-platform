# PM Brief: P1-E4-T1 — Phase 1 acceptance checklist

**From:** PM  
**To:** PM (lead) + QA (verify evidence)  
**Status:** In Progress (assigned)  
**Date:** 2026-08-03  
**Closed previous:** Epic E3 — P1-E3-T4 Pass with notes (`docs/handoffs/P1-E3-T4-qa-to-pm.md`)

## Task
**P1-E4-T1 — Phase 1 acceptance checklist**

## Acceptance Criteria
1. Evidence pack Phase 1 green:
   - `cargo check` (workspace)
   - `cargo test -p bio-spec -p runtime -p storage -p desktop` (or full `cargo test`)
   - desktop: `pnpm build` / prior Menubar smoke (T4)
2. Checklist document written (this handoff completion or `docs/handoffs/P1-E4-T1-qa-to-pm.md` / acceptance note): all E1–E3 Done IDs listed.
3. Ready list for **Phase 2** captured (ingest HTTP, collectors, iOS) — **без** старта реализации.
4. **Do not** flip `/docs/14-roadmap.md` Phase 1 → done until **explicit user approve**.
5. Record open follow-ups:
   - `dbError` may include absolute paths (`CreateDir`) — sanitize on host (future small task / ADR).

## Out of scope
- Implementing Phase 2
- Changing architecture freeze
- Notarization / store packaging

## After
PM sync docs + canvas → assign **P1-E4-T2** (dev guide final sync) if still needed, then ask user to approve Phase 1 close on `14-roadmap.md`.
