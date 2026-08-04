# 14. MVP Roadmap

- [x] **Phase 0: Architecture Freeze & Spec Design**
- [x] **Phase 1: Foundation (Sprint 1-2)** — closed 2026-08-03 (user approve)
  - Workspace setup (`bio-spec`, `runtime`, `storage`).
  - SQLite WAL migration script & Repositories.
  - Basic Menubar UI via Tauri v2.
- [ ] **Phase 2: Ingestion & Context Collector (Sprint 3-4)** — decomposed 2026-08-04; implementation not started
  - Local HTTP Server (`/v1/ingest`).
  - macOS active window & keystroke collector.
  - iOS Companion app (HealthKit webhook).
- [ ] **Phase 3: Pipeline & Features (Sprint 5-6)**
  - Pipeline logic (Deduplication -> Normalization -> Feature Calculation).
  - Real-time alerts (Menubar color change 🟢/🟡/🔴).
- [ ] **Phase 4: Dashboard UI & Local AI Insights (Sprint 7-8)**
  - Local React Dashboard with Recharts.
  - Local LLM Prompt Generator (Ollama / OpenAI API).

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2 plan:** `docs/SPRINT_ROADMAP.md` · first Ready: **P2-E1-T1** (`docs/handoffs/P2-E1-T1-pm-brief.md`)  
**Git:** branch → PR → `main` after CI (`docs/12-development.md`).
