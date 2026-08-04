# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Epic P2-E0 Done**; **Epic P2-E1 Done**; **Epic P2-E2 Done**; **Epic P2-E3 Done** (2026-08-04) — hygiene `dbError` + ingest + collector + companion/pairing. Next: **sprint gate** (push + PR → `main`). Branch `phase/2-ingest-http`.
- **Open follow-up:** none for Phase 2 task queue — open `docs/handoffs/SPRINT-GATE.md` when closing the sprint.
- Phase 3+ not started.

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