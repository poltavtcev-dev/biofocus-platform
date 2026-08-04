# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **P2-E1-T1** + **P2-E1-T2** Done (Pass with notes) — `crates/ingest` loopback ingest + `~/.biofocus/pairing_token` (or `BIOFOCUS_HOME` / env override). Next Ready: **P2-E1-T3** persist + mid-batch 503. **Branch `phase/2-ingest-http` still needs commit+PR.**
- **Open follow-up:** sanitize IPC `dbError` → **P2-E0-T1**; host `IngestConfig::load` → **P2-E1-T4**.
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