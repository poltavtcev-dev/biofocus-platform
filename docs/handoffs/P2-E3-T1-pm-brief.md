# PM Brief → Dev: P2-E3-T1

**From:** PM  
**To:** Dev (iOS)  
**Status:** Ready (assigned)  
**Date:** 2026-08-04  
**Closed previous:** P2-E2-T3 — QA Pass with notes (`docs/handoffs/P2-E2-T3-qa-to-pm.md`); Epic **P2-E2** ✅

## Task
**P2-E3-T1 — Companion contract + minimal HealthKit sample path**

## Why
Desktop ingest + pairing are live. Need a minimal iOS/companion path that posts a sample `heart_rate` Observation to local `POST /v1/ingest` before pairing UX (T2).

## Acceptance Criteria
1. Minimal path: sample `heart_rate` Observation → HTTP ingest on a reachable host (loopback / LAN as documented).
2. Request body = JSON array of Observation (same contract as desktop ingest).
3. Network / 401 errors handled without crash; no silent swallow of auth failure.
4. No Feature math, no dashboard, no cloud account.
5. Handoff: `docs/handoffs/P2-E3-T1-dev-to-qa.md`.

## Out of scope
- Pairing QR / copy UX → **P2-E3-T2**
- Phase 3 pipeline / Features / Menubar alerts
- Changing ingest HTTP contract or E2 collector payloads

## Constraints
- Local-only; Bearer pairing token from desktop (`docs/10-security.md` / `12-development.md`)
- Idle / battery: no busy-loop HealthKit polling beyond what sample path needs
- No unwrap/expect in production paths

## Hygiene
- Commit after build; push only at sprint gate (`SPRINT-GATE.md`).
- Epic E2 closed on branch — sprint PR still deferred until Sprint 3–4 gate (or explicit «закрой спринт»).

## After QA Pass
PM → Ready **P2-E3-T2** (pairing UX); consider sprint gate PR if E3 near-complete or user requests.
