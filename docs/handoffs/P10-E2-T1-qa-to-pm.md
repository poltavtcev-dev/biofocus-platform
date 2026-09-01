# QA → PM: P10-E2-T1

## Meta
- **Task ID:** P10-E2-T1
- **Title:** Implement Browser categories collector plugin (ADR-010)
- **Date:** 2026-08-10
- **Dev/UX handoff:** `docs/handoffs/P10-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p bio-spec browser` → **5 passed** (4 unit + 1 contracts round-trip)
  - `cargo test -p macos-collector --test collector_integration` → **9 passed** (incl. browser emit→persist + stop freeze)
  - `cargo test -p macos-collector browser --lib` → **2 passed**
  - `cargo check -p macos-collector -p bio-spec -p ingest -p desktop` → **ok**
  - Docs 07/08/10/12 no longer “planned only” for browser section; security opt-in + privacy bar present
  - Privacy asserts in integration test: no `url` / `title` / `href` on emitted Observation
  - Dev handoff present
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Plugin `com.biofocus.macos.browser` + Capability `browser_categories` / `browser_category` in macos-collector | **Pass** |
| AC2 ingest_host starts only when `BIOFOCUS_BROWSER_CATEGORIES=1`; same channel → persist; UI↛SQLite | **Pass** |
| AC3 Contract payload coarse category + optional bundle; never URL/title/content | **Pass** |
| AC4 Injectable probe; OS soft-fail (`unknown` / `None`); ≥5s on-change; stop joins; no busy-loop | **Pass** |
| AC5 Docs finalized: 07-contracts, 08-plugin-sdk, 10-security, 12-development | **Pass** |
| AC6 Tests: emit→persist + stop freezes probe | **Pass** |
| AC7 `docs/handoffs/P10-E2-T1-dev-to-qa.md` | **Pass** |
| Global DoD: no prod unwrap/expect; idle; no new SQLite schema; personal self-tracking | **Pass** |

- Extra checks: Out of scope respected (no DistractionScore DAG, no migration, no PR). OS probe honesty (`unknown` without URL mapping) documented.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move P10-E2-T1 to Done; Ready **P10-E3-T1**
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / `14-roadmap` (collector shipped)
- [ ] Brief for next: `docs/handoffs/P10-E3-T1-pm-brief.md`

## Suggested next Ready task
- **P10-E3-T1** — Catalog Feature `DistractionScore` from `browser_category` (+ optional CSR); ADR-007 confidence; register in feature-engine; pipeline normalize if needed.

## Notes for PM
- Branch: `phase/10-plugin-wave-1`.
- Evidence: this file + `docs/handoffs/P10-E2-T1-dev-to-qa.md` + green collector/bio-spec tests.
- Dogfood: `BIOFOCUS_BROWSER_CATEGORIES=1` — live OS often emits `category: "unknown"` until richer non-persisting mapping exists; mock path covers closed-set labels for E3.
- PR freeze still active — no PR.
