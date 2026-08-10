# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Done on `main`** via [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2) (2026-08-04) — E0–E3 (ingest, collector, companion/pairing, `dbError` hygiene).
- **Phase 3:** **Done** (2026-08-05) — Pipeline & Features; E1–E3.
- **Phase 4:** **Done** (2026-08-05) — Dashboard UI & Local AI Insights; E1–E3. Branch: `phase/4-dashboard-ai` (PR after freeze).
- **Phase 5:** **Done** (2026-08-05) — Wearable dogfood; E1–E3. Branch: `phase/5-wearable-dogfood` (PR after freeze).
- **Phase 6:** **Done** (2026-08-06) — Life context; E1–E3. Tip: `phase/6-dogfood-fixes` (PR after freeze).
- **Phase 7:** **Done** (2026-08-06) — Trust layer; E1–E3 (ADR-007 confidence → Explanation factors → `RecoveryScore`). Branch: `phase/7-trust-layer` (PR after freeze).
- **Phase 8:** **Done** (2026-08-08) — Pattern Discovery v1; E1–E3 (ADR-008 recompute-on-read → `focus_vs_recent_baseline_v1` → Dashboard Insights surface). Branch: `phase/8-pattern-discovery` (PR after freeze).
- **Phase 9:** **Done** (2026-08-08) — Deterministic Recommendations; E1–E3 (ADR-009 → `focus_dip_pace_hint_v1` → `get_recommendations` + Suggestions). Branch: `phase/9-recommendations` (PR after freeze).
- **Phase 10:** **Opened** (2026-08-10) — Plugin wave-1; ADR-010 **Browser categories** Done (P10-E1); Ready **P10-E2-T1** (collector). Branch: `phase/10-plugin-wave-1` (PR after freeze).
- **Horizon P11–P12+:** AI coaching polish → ambient + packaging. Not Kanban-Ready until PM opens each phase. **Next after P10:** Phase 11.

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
