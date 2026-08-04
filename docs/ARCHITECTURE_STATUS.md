# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Epic P2-E0 Done**; **Epic P2-E1 Done**; **Epic P2-E2 Done**; **Epic P2-E3 Done** (2026-08-04). **Sprint gate open** — branch `phase/2-ingest-http` → PR → `main`.
- **Open follow-up:** merge after CI; then mark Phase 2 ☐→☑ on roadmap if product accepts.
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