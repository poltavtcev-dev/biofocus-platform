# Dev → QA: P11-E1-T1

## Meta
- **Task ID:** P11-E1-T1
- **Title:** ADR-011: AI coaching polish (prompt packs + provider UX)
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P11-E1-T1; brief `docs/handoffs/P11-E1-T1-pm-brief.md`
- **Branch:** `phase/11-ai-coaching-polish`

## What changed
- **ADR-011** in `docs/decision-log.md`: L5 polish v1 = **named/versioned prompt packs** in `report-engine` (templates over already-computed Features / Insights / Recommendations) + **calm Dashboard provider UX** for opt-in local LLM status/config; reaffirm interpret-only / local-first / idle / no auto-invoke on open.
- Schema: **none to apply** — in-process packs + existing env/IPC (Phase 4); no chat-history SQLite; no migration; deferred on-disk pack overrides / feedback history need future ADR + user approve.
- E2/E3 sketch: packs API in `report-engine` → Dashboard provider UX on existing `generate_report` / interpret path (explicit user action).
- Rejected: cloud LLM by default; LLM as SoT for Features/Recommendations; auto-invoke on open; parallel Coach Engine; clinical tone; chat history SQLite without need; shipping packs without ADR; cloud marketplace in P11.
- Planned notes: `09-api`, `10-security`, `16-glossary` (not deferred).
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-011 summary + detail)
- `docs/09-api.md` (planned packs + provider UX)
- `docs/10-security.md` (planned coaching polish security note)
- `docs/16-glossary.md` (Prompt Pack + Provider UX)
- `docs/handoffs/P11-E1-T1-dev-to-qa.md` (this file)
- (pre-existing Phase 11 PM open docs on tree — not authored in this Dev pass)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-011 present and no migration SQL applied
rg -n "ADR-011" docs/decision-log.md docs/09-api.md docs/10-security.md docs/16-glossary.md
rg -n "prompt pack|Prompt Pack|provider UX|Provider UX" docs/decision-log.md docs/09-api.md docs/10-security.md docs/16-glossary.md
rg -n "Coach Engine|auto-invoke|chat history|Cloud LLM by default|shipping packs without ADR" docs/decision-log.md
rg -n "CREATE TABLE.*(pack|chat|coach)" docs/decision-log.md || true
# No crate changes expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-011 in `docs/decision-log.md` — prompt packs (named/versioned over Features / Insights / Recommendations) + provider UX; L5 interpret-only reaffirmed; local-first / idle / no auto-invoke on open
- [ ] AC2: Rejected alternatives documented (cloud LLM by default; LLM as SoT; auto-invoke; parallel Coach Engine; clinical tone; chat history SQLite without need; shipping packs without ADR)
- [ ] AC3: Schema — sketch only / none to apply; prefer in-process + env/IPC; **no migration**
- [ ] AC4: Short sketch E2 packs API in `report-engine` → E3 Dashboard provider UX on `generate_report` / interpret (explicit user action)
- [ ] AC5: Docs touch `09-api` / `10-security` / `16-glossary` as planned/ADR notes (not deferred without statement)
- [ ] AC6: This handoff exists
- [ ] Global DoD: glossary terms; calm non-clinical; UI↛DB; LLM not computing Features/Recommendations/Evidence; no parallel Coach Engine; PR freeze respected

## Risks / not covered
- Implementing prompt packs → **P11-E2-T1** (out of scope).
- Dashboard provider UI / pack picker → **P11-E3-T1** (out of scope).
- Exact pack API names / IPC DTO shapes left to E2/E3 — ADR locks boundary + persistence stance.
- If later dogfood wants on-disk editable packs or chat transcript SQLite, needs a **new** ADR + user approve — not silently added.

## Notes for QA
- Decision evolves `report-engine` + existing Report IPC — not a new Coach Engine crate.
- No `cargo test` required (docs-only); spot-check that no `crates/` / `apps/` diffs come from this task.
- Interpret-only boundary must be explicit: LLM must not invent scores, Evidence, or Recommendations.
