# PM Brief → UX + Dev: P2-E3-T2

**From:** PM  
**To:** UX (lead) + Dev  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E3-T1 — QA Pass with notes (`docs/handoffs/P2-E3-T1-qa-to-pm.md`)

## Task
**P2-E3-T2 — Pairing UX (token share)**

## Why
Companion can POST with a Bearer token, but users have no in-app way to move `~/.biofocus/pairing_token` to the phone. Close the E3 bridge with copy/QR (local only).

## Acceptance Criteria
1. Desktop shell exposes a way to view/copy (and optionally QR) the pairing token for companion use.
2. No cloud account; token remains local secret (`docs/10-security.md`).
3. UI still does not talk to SQLite directly (IPC only).
4. Document the flow in `docs/12-development.md` / companion README.
5. Handoff: `docs/handoffs/P2-E3-T2-dev-to-qa.md` (UX may author).

## Out of scope
- Changing ingest HTTP contract
- LAN bind changes (note/document if needed for physical device — separate decision)
- Phase 3 Features / dashboard
- Full App Store iOS app polish beyond what's needed to paste/use the token

## Constraints
- Apple HIG; calm non-judgmental UI
- Idle footprint; no unwrap/expect in prod

## Hygiene
- Commit after build; push only at sprint gate (`SPRINT-GATE.md`).

## After QA Pass
PM may Ready **P2-E0-T1** (hygiene) or declare Sprint 3–4 gate / PR if E3 epic complete enough.
