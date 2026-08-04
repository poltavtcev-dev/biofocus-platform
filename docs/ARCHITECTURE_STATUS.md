# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Epic P2-E1 Done**; **Epic P2-E2 Done**; **P2-E3-T1 Done** (2026-08-04) — companion sample `heart_rate` → ingest. Next Ready: **P2-E3-T2** pairing UX. Branch `phase/2-ingest-http` (commit after build; **sprint PR → `main` at sprint gate**).
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