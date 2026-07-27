---

### `docs/14-roadmap.md`

```markdown
# 14. MVP Roadmap

- [x] **Phase 0: Architecture Freeze & Spec Design**
- [ ] **Phase 1: Foundation (Sprint 1-2)**
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