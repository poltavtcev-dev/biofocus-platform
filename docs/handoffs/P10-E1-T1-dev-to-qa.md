# Dev → QA: P10-E1-T1

## Meta
- **Task ID:** P10-E1-T1
- **Title:** ADR-010: Plugin wave-1 source + Observation contract
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P10-E1-T1; brief `docs/handoffs/P10-E1-T1-pm-brief.md`
- **Branch:** `phase/10-plugin-wave-1`

## What changed
- **ADR-010** in `docs/decision-log.md`: wave-1 source = **Browser categories** (IDE/Git deferred); Capability Plugin Model via `plugin-sdk` + `macos-collector` (or thin adapter); persist in existing `observations` store.
- Observation contract sketch: `data_type: "browser_category"`, provider `com.biofocus.macos.browser`, payload coarse `category` (+ optional `browser_bundle_id`), opt-in `BIOFOCUS_BROWSER_CATEGORIES=1` (default off), emit on change / rare ≥5s poll, no busy-loop.
- Explicit forbid: full URLs, page titles, keystroke/content capture, employee-surveillance framing.
- Rejected: both IDE+Browser same wave; always-on; cloud history sync; marketplace crate; parallel plugin SQLite registry; ambient in P10; IDE/Git as v1; URL logging “for accuracy”; NotificationPressure folded here.
- Schema: **none to apply** — no migration; deferred allowlist config only via future ADR + user approve.
- E2/E3 sketch: plugin → shared Observation channel → persist; E3 `DistractionScore` with ADR-007 confidence + calm framing.
- Planned notes: `08-plugin-sdk`, `07-contracts`, `16-glossary`, `04-storage`, `06-feature-catalog`, `12-development`.
- **No code / no migration.**

## Crates / apps / files touched
- `docs/decision-log.md` (ADR-010 summary + detail)
- `docs/08-plugin-sdk.md` (§5 planned browser plugin)
- `docs/07-contracts.md` (`browser_category` payload sketch)
- `docs/16-glossary.md` (Browser Category + DistractionScore + Plugin note)
- `docs/04-storage.md` (ADR-010 — no migration)
- `docs/06-feature-catalog.md` (`DistractionScore` → ADR-010 / P10-E3)
- `docs/12-development.md` (ADR-010 note)
- `docs/handoffs/P10-E1-T1-dev-to-qa.md` (this file)
- (pre-existing Phase 10 PM open docs on tree — not authored in this Dev pass)

## How to verify (commands)
```bash
# Docs-only ADR — confirm ADR-010 present and no migration SQL applied
rg -n "ADR-010" docs/decision-log.md docs/08-plugin-sdk.md docs/07-contracts.md docs/16-glossary.md docs/04-storage.md docs/06-feature-catalog.md docs/12-development.md
rg -n "browser_category|BIOFOCUS_BROWSER_CATEGORIES|DistractionScore" docs/decision-log.md docs/07-contracts.md docs/08-plugin-sdk.md docs/06-feature-catalog.md
rg -n "CREATE TABLE.*(plugin|browser)" docs/decision-log.md docs/04-storage.md || true
# No crate changes expected for this task
git diff --name-only -- 'crates/' 'apps/' || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: ADR-010 in `docs/decision-log.md` — chosen **Browser categories**; rationale (dogfood / CSR complement / catalog `DistractionScore`); Capability Plugin Model relationship
- [ ] AC2: Observation contract sketch — `data_type`, privacy-safe payload, `provider_id`, opt-in env default off, poll/idle; forbids URLs / keystroke content / surveillance framing
- [ ] AC3: Rejected alternatives documented (both IDE+Browser; always-on; cloud sync; marketplace; parallel plugin SQLite; ambient in P10; …)
- [ ] AC4: Schema — none to apply; prefer existing `observations`; no migration from this task
- [ ] AC5: E2 plugin → channel → persist sketch; E3 `DistractionScore` + ADR-007 confidence + calm framing
- [ ] AC6: Docs touch `08` / `07` / `16` (and related) as planned/ADR notes — not deferred without statement
- [ ] AC7: This handoff exists
- [ ] Global DoD: glossary terms; calm non-clinical; UI↛DB; no parallel marketplace crate; LLM not defining payloads/Features; personal self-tracking only

## Risks / not covered
- Implementing the collector plugin → **P10-E2-T1** (out of scope).
- Catalog Feature / normalize beyond sketch → **P10-E3-T1** (out of scope).
- Exact probe (Accessibility vs extension vs allowlist) left to E2 — ADR locks contract + privacy bar only.
- Category enum labels may be refined slightly in E2 contracts without a new ADR if still coarse and closed-set.
- If dogfood later wants host allowlist SQLite config, needs a **new** ADR + approve — not silently added.

## Notes for QA
- Decision is explicitly **Browser categories**, not IDE/Git for wave-1 v1.
- No `cargo test` required (docs-only); spot-check that no `crates/` / `apps/` diffs come from this task.
- Calm framing in E3 sketch must not read as clinical ADHD / “you are distracted” diagnosis.
