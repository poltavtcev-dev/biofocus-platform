# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Done on `main`** via [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2) (2026-08-04) — E0–E3 (ingest, collector, companion/pairing, `dbError` hygiene).
- **Phase 3:** **Done** (2026-08-05) — Pipeline & Features; E1–E3 (quality pipeline → Feature DAG → Menubar AlertLevel). Branch cluster: `phase/3-pipeline-features` / follow-ups.
- **Phase 4:** **Done** (2026-08-05) — Dashboard UI & Local AI Insights; E1–E3 (Feature IPC + Recharts · Knowledge Insights · report-engine + optional local LLM + Dashboard `generate_report`). Branch cluster: `phase/4-dashboard-ai` (PR pending).
- **Phase 5:** **Done** (2026-08-05) — Wearable dogfood; E1–E3 (opt-in LAN ADR-005 + advertise hints + Companion LAN Base URL / token/QR + runnable iOS HealthKit companion + dogfood runbook). Branch cluster: `phase/5-wearable-dogfood` (PR pending).
- **Phase 6:** **Active** (2026-08-05) — Life context: Life Events as `Observation` kinds (ADR-006) → Desktop quick-log → Calendar → `MeetingDensity` / `RecoveryBetweenMeetings`. **P6-E1-T1 Done**; Ready: **P6-E2-T1**. Branch: `phase/6-life-context`. Git = classic related-work PRs (`docs/12-development.md`).
- **Horizon P7–P12+:** accepted in `/docs/00-vision.md` (Trust → Pattern Discovery → Recs → plugins → AI polish → packaging). Not Kanban-Ready until PM opens each phase.

## Core Decisions
- Local First Architecture
- Rust Runtime (Tokio)
- SQLite Storage (WAL Mode)
- Tauri v2 (Desktop UI)
- Feature Pipeline & Knowledge Engine
- Capability Plugin Model
- Platform Vision ladder (Personal Pattern Discovery) — `/docs/00-vision.md`

## Modification Policy
Все изменения архитектуры выполняются только через ADR (Architecture Decision Records). 
AI-помощникам (Cursor / Claude Code) запрещено самостоятельно изменять архитектурные границы и сущности проекта.
