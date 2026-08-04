# 14. MVP Roadmap

- [x] **Phase 0: Architecture Freeze & Spec Design**
- [x] **Phase 1: Foundation (Sprint 1-2)** — closed 2026-08-03 (user approve)
  - Workspace setup (`bio-spec`, `runtime`, `storage`).
  - SQLite WAL migration script & Repositories.
  - Basic Menubar UI via Tauri v2.
- [ ] **Phase 2: Ingestion & Context Collector (Sprint 3-4)**
  - Local HTTP Server (`/v1/ingest`).
  - macOS active window & keystroke collector.
  - iOS Companion app (HealthKit webhook).
- [ ] **Phase 3: Pipeline & Features (Sprint 5-6)**
  - Pipeline logic (Deduplication -> Normalization -> Feature Calculation).
  - Real-time alerts (Menubar color change 🟢/🟡/🔴).
- [ ] **Phase 4: Dashboard UI & Local AI Insights (Sprint 7-8)**
  - Local React Dashboard with Recharts.
  - Local LLM Prompt Generator (Ollama / OpenAI API).

**Evidence:** `docs/handoffs/P1-E4-T1-acceptance.md` · T2 close: `docs/handoffs/P1-E4-T2-qa-to-pm.md` · Kanban: `docs/SPRINT_ROADMAP.md`  
**Next (not started):** Phase 2 Ready list only — no implementation until PM assigns. CI: `.github/workflows/ci.yml`.
