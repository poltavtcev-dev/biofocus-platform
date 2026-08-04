# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Done on `main`** via [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2) (2026-08-04) — E0–E3 (ingest, collector, companion/pairing, `dbError` hygiene).
- **Phase 3:** **Open** (Sprint 5–6) — Pipeline & Features; Ready = **P3-E1-T1**. Branch suggestion: `phase/3-pipeline-features`.
- Phase 4+ not started. Git = classic related-work PRs (`docs/12-development.md`).

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
