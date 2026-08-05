# Dev → QA: P5-E3-T2

## Meta
- **Task ID:** P5-E3-T2
- **Title:** Dogfood runbook + contract docs
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P5-E3-T2; brief `docs/handoffs/P5-E3-T2-pm-brief.md`
- **Branch:** `phase/5-wearable-dogfood`

## What changed
- End-to-end **Wearable dogfood runbook** in `docs/12-development.md` (LAN → Base URL → pair token → open/run iOS → one-shot → confirm Observation via Status / optional SQLite / Dashboard note).
- Companion READMEs (`apps/companion/README.md`, `apps/companion/ios/README.md`) point at `BioFocusCompanion.xcodeproj` and the runbook.
- `docs/PROJECT_CANVAS.md` wearable § reflects runnable iOS path + runbook (no personal device inventory).
- Contract docs: `docs/07-contracts.md` no longer labels iOS as “stub”; `docs/09-api.md` documents runnable Xcode target + Bearer / Observation contract + runbook link.
- Docs-only; no ingest auth / schema / product Feature changes.

## How to verify (commands)
```bash
# Runbook section present
rg -n "Wearable dogfood runbook" docs/12-development.md

# Companion READMEs → xcodeproj + runbook
rg -n "BioFocusCompanion.xcodeproj|Wearable dogfood runbook" apps/companion/README.md apps/companion/ios/README.md

# No stub-only iOS path in live contract docs
rg -n "HealthKit stub|iOS stub|stub-only" docs/07-contracts.md docs/09-api.md docs/12-development.md || true

# Canvas wearable §
rg -n "dogfood runbook|BioFocusCompanion" docs/PROJECT_CANVAS.md

# No personal secrets / serials in touched docs (spot-check)
rg -n "BIOFOCUS_INGEST_TOKEN=[A-Za-z0-9]{8,}|serial|iPhone [0-9]" \
  docs/12-development.md docs/07-contracts.md docs/09-api.md docs/PROJECT_CANVAS.md \
  apps/companion/README.md apps/companion/ios/README.md || true

# Optional: contract client still green (unchanged code)
cargo test -p companion
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: E2E runbook in `docs/12-development.md` (or clearly linked): LAN → Base URL → pair → iOS run → one-shot → Observation visible (status / Dashboard / storage as appropriate)
- [ ] AC2: Companion READMEs align with runbook and point at `BioFocusCompanion.xcodeproj`
- [ ] AC3: `PROJECT_CANVAS` wearable § reflects runnable iOS path + runbook (no personal device inventory)
- [ ] AC4: Contract docs (`09-api` / related) no longer call iOS stub-only; Observation / Bearer accurate
- [ ] AC5: No secrets, personal hostnames, or device serials committed
- [ ] AC6: Handoff `docs/handoffs/P5-E3-T2-dev-to-qa.md`
- [ ] Global DoD: docs-only; UI↛DB; glossary `Observation`; no schema/auth change

## Risks / not covered
- Live Desktop + Simulator/device dogfood not re-run in this session (operator path is documented; T1 noted missing Simulator runtime on some hosts).
- Dashboard Feature charts are explicitly **not** the primary proof for a single HR sample (catalog inputs differ) — storage / Companion Status are.

## Notes for QA
- Branch already had T1 close docs pointing at T2; this task replaces the “→ P5-E3-T2” placeholder with the actual runbook.
- Do not mark Kanban Done / canvas (PM after qa-to-pm).
