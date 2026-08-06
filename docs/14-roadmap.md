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
- [x] **Phase 4: Dashboard UI & Local AI Insights (Sprint 7-8)** — closed 2026-08-05 (E1–E3 Done; cluster PR pending)
  - Local React Dashboard with Recharts.
  - Knowledge Insights (deterministic) + Local LLM Prompt Generator (Ollama / OpenAI-compatible, opt-in).
- [x] **Phase 5: Wearable dogfood (Sprint 9–10)** — closed 2026-08-05 (E1–E3 Done; cluster PR pending)
  - Opt-in **LAN-reachable ingest** + URL advertise — **P5-E1 Done** (T1 ADR-005 · T2 `bind_mode` / `base_url_hints`).
  - Pairing UX exposes LAN base URL for physical phone — **P5-E2 Done** (Companion Base URL + token/QR).
  - Runnable **iOS HealthKit companion** → `Observation` → Desktop ingest + dogfood runbook — **P5-E3 Done**.
- [x] **Phase 6: Life context (Sprint 11–12)** — closed 2026-08-06 (E1–E3 Done; cluster PR pending)
  - Life Events as `Observation` kinds (ADR-006) — **P6-E1 Done**.
  - Desktop quick-log — **P6-E2 Done**.
  - Calendar Observations + `MeetingDensity` / `RecoveryBetweenMeetings` — **P6-E3 Done**.
- [ ] **Phase 7: Trust layer (Sprint 13–14)** — **opened 2026-08-06**
  - Feature confidence (ADR-007) + Core/snapshot wire — **P7-E1 Done**.
  - Explanation factors — **P7-E2 Done** → Ready **P7-E3-T1** (bio-backed catalog Features).

### Horizon (accepted vision ladder — not Kanban-Ready yet)

Product philosophy + sequencing: `/docs/00-vision.md`. Open each phase via PM gate only.

- [ ] **Phase 8: Pattern Discovery v1** — multi-day / baseline Knowledge (ADR: recompute vs Feature history)
- [ ] **Phase 9: Recommendations** — deterministic action suggestions with Evidence
- [ ] **Phase 10: Plugin wave-1** — IDE/Git or Browser categories (dogfood-driven)
- [ ] **Phase 11: AI coaching polish** — prompt packs / provider UX (interpret-only)
- [ ] **Phase 12+: Ambient + commercial packaging** — music/weather/light; signed builds / updates; optional user sync

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2:** merged [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)  
**Phase 3:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` archive · handoffs `P3-*-qa-to-pm.md`  
**Phase 4:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai` (cluster PR when ready)  
**Phase 5:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P5-*-qa-to-pm.md` · branch `phase/5-wearable-dogfood` (cluster PR when ready)  
**Phase 6:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P6-*-qa-to-pm.md` · tip `phase/6-dogfood-fixes` (cluster PR when ready)  
**Phase 7:** active — `docs/SPRINT_ROADMAP.md` · P7-E1/E2 Done · Ready **P7-E3-T1** · branch `phase/7-trust-layer` · **PR freeze until 2026-09-01**  
**Vision:** `/docs/00-vision.md` · canvas snapshot `PROJECT_CANVAS.md`  
**Git:** related-work branches → local commits; **PR freeze until 2026-09-01** — see `docs/12-development.md` / `.cursor/rules/06-git-agent-policy.mdc`.
