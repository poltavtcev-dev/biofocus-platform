# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Done on `main`** via [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2) (2026-08-04) — E0–E3 (ingest, collector, companion/pairing, `dbError` hygiene).
- **Phase 3:** **Done** (2026-08-05) — Pipeline & Features; E1–E3 (quality pipeline → Feature DAG → Menubar AlertLevel). Branch cluster: `phase/3-pipeline-features` / follow-ups.
- **Phase 4:** **Open** (Sprint 7–8) — Dashboard UI & Local AI Insights; Ready = **P4-E3-T2** (optional local LLM adapter). Done: Epic **P4-E1** · Epic **P4-E2** · **P4-E3-T1** (`build_report` → offline markdown + `llm_prompt`). Branch: `phase/4-dashboard-ai`.
- Phase 5+ not started. Git = classic related-work PRs (`docs/12-development.md`).

## Core Decisions
- Local First Architecture
- Rust Runtime (Tokio)
- SQLite Storage (WAL Mode)
- Tauri v2 (Desktop UI)
- Feature Pipeline & Knowledge Engine
- Capability Plugin Model

## Modification Policy
Все изменения архитектуры выполняются только через ADR (Architecture Decision Records). 
AI-помощникам (Cursor / Claude Code) запрещено самостоятельно изменять архитектурные границы и сущности проекта.
