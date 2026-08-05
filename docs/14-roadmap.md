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
- [x] **Phase 3: Pipeline & Features (Sprint 5-6)** — closed 2026-08-05 (E1–E3 Done)
  - Pipeline logic (Deduplication -> Normalization -> Feature Calculation).
  - Real-time alerts (Menubar color change 🟢/🟡/🔴).
- [ ] **Phase 4: Dashboard UI & Local AI Insights (Sprint 7-8)** — **opened 2026-08-05** (Ready: **P4-E2-T2**)
  - Local React Dashboard with Recharts.
  - Knowledge Insights (deterministic) + Local LLM Prompt Generator (Ollama / OpenAI-compatible, opt-in).

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2:** merged [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)  
**Phase 3:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` archive · handoffs `P3-*-qa-to-pm.md`  
**Phase 4:** active — `docs/SPRINT_ROADMAP.md` · Ready **P4-E2-T2** · brief `docs/handoffs/P4-E2-T2-pm-brief.md` · P4-E1 Done · P4-E2-T1 Done  
**Git:** related-work branches → PR (classic); see `docs/12-development.md`.
