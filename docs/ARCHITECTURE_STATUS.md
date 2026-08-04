# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Epic P2-E1 Done** (T1–T4); **P2-E2-T1 Done** (2026-08-04, Pass with notes) — `macos-collector` → `context_window` (`bundle_id`/`app_name`) on Desktop channel→persist. Next Ready: **P2-E2-T2** keystroke/input aggregates (privacy-safe). Branch `phase/2-ingest-http` (commit per task; **sprint PR → `main` at sprint gate**).
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