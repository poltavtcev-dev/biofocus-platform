# 14. MVP Roadmap

- [x] **Phase 0: Architecture Freeze & Spec Design**
- [x] **Phase 1: Foundation (Sprint 1-2)** — closed 2026-08-03 (user approve)
  - Workspace setup (`bio-spec`, `runtime`, `storage`).
  - SQLite WAL migration script & Repositories.
  - Basic Menubar UI via Tauri v2.
- [ ] **Phase 2: Ingestion & Context Collector (Sprint 3-4)** — Epics **P2-E1** + **P2-E2** Done; E3 next
  - Local HTTP Server (`/v1/ingest`) + host wire + `/v1/status` — **shipped (E1)**.
  - macOS collectors + integration/idle tests — **shipped (E2)**.
  - iOS Companion app (HealthKit → ingest) — **next (E3)**.
- [ ] **Phase 3: Pipeline & Features (Sprint 5-6)**
  - Pipeline logic (Deduplication -> Normalization -> Feature Calculation).
  - Real-time alerts (Menubar color change 🟢/🟡/🔴).
- [ ] **Phase 4: Dashboard UI & Local AI Insights (Sprint 7-8)**
  - Local React Dashboard with Recharts.
  - Local LLM Prompt Generator (Ollama / OpenAI API).

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2:** `docs/SPRINT_ROADMAP.md` — E1–E2 Done; Ready **P2-E3-T1** (`docs/handoffs/P2-E3-T1-pm-brief.md`)  
**Git:** branch `phase/2-ingest-http`; **commit after build**; **PR → `main` per sprint**.
