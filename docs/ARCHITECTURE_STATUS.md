# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Epic P2-E1 Done**; **P2-E2-T1/T2 Done** (2026-08-04) — active window + opt-in keystroke aggregates. Next Ready: **P2-E2-T3** collector tests/pause idle. Branch `phase/2-ingest-http` (commit after build; **sprint PR → `main` at sprint gate**).
- **Open follow-up:** sanitize IPC `dbError` → **P2-E0-T1** (optionally HTTP `db_error` too).
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