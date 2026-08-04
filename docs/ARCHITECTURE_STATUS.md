# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot (Phase 1)
- **Epic E1–E3:** Done (2026-08-03) — crates foundation, SQLite WAL + `ObservationRepository`, Tauri v2 Menubar + IPC `get_status`.
- **Epic E4:** Done — T1 acceptance (`docs/handoffs/P1-E4-T1-acceptance.md`) + T2 doc sync (`docs/handoffs/P1-E4-T2-qa-to-pm.md`, 2026-08-04).
- **`docs/14-roadmap.md` Phase 1:** **[x] Done** (user approve 2026-08-03). Phase 2+ not started.
- **Open follow-up (non-architecture):** sanitize IPC `dbError` so absolute DB paths are not shown in UI (from E3-T4 notes).

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