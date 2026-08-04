# 14. MVP Roadmap

- [x] **Phase 0: Architecture Freeze & Spec Design**
- [x] **Phase 1: Foundation (Sprint 1-2)** — closed 2026-08-03 (user approve)
  - Workspace setup (`bio-spec`, `runtime`, `storage`).
  - SQLite WAL migration script & Repositories.
  - Basic Menubar UI via Tauri v2.
- [x] **Phase 2: Ingestion & Context Collector (Sprint 3-4)** — merged to `main` ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2), 2026-08-04)
  - Local HTTP Server (`/v1/ingest`) + host wire + `/v1/status` — **shipped (E1)**.
  - macOS collectors + integration/idle tests — **shipped (E2)**.
  - Companion sample + pairing UX (copy/QR) — **shipped (E3)**.
  - Host hygiene (`dbError` / `db_error` sanitize) — **shipped (E0)**.
- [ ] **Phase 3: Pipeline & Features (Sprint 5-6)** — **opened 2026-08-04** (Ready: P3-E1-T1)
  - Pipeline logic (Deduplication -> Normalization -> Feature Calculation).
  - Real-time alerts (Menubar color change 🟢/🟡/🔴).
- [ ] **Phase 4: Dashboard UI & Local AI Insights (Sprint 7-8)**
  - Local React Dashboard with Recharts.
  - Local LLM Prompt Generator (Ollama / OpenAI API).

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2:** merged [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)  
**Phase 3:** active — `docs/SPRINT_ROADMAP.md` · Ready **P3-E1-T4** · brief `docs/handoffs/P3-E1-T4-pm-brief.md`  
**Git:** related-work branches → PR (classic); see `docs/12-development.md`.
