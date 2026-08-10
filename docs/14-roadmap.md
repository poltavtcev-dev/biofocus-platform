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
- [x] **Phase 7: Trust layer (Sprint 13–14)** — closed 2026-08-06 (E1–E3 Done; cluster PR after freeze)
  - Feature confidence (ADR-007) — **P7-E1 Done**.
  - Explanation factors — **P7-E2 Done**.
  - `RecoveryScore` bio-backed Feature — **P7-E3 Done**.
- [x] **Phase 8: Pattern Discovery v1 (Sprint 15–16)** — closed 2026-08-08 (E1–E3 Done; cluster PR after freeze)
  - ADR-008 history / recompute (**recompute-on-read**) — **P8-E1 Done** (2026-08-07).
  - Baseline Knowledge Insights (`focus_vs_recent_baseline_v1`) — **P8-E2 Done** (2026-08-07).
  - Insights IPC / UX surface — **P8-E3 Done** (2026-08-08).

- [x] **Phase 9: Recommendations** (Sprint 17–18) — closed 2026-08-08 (E1–E3 Done; cluster PR after freeze)
  - ADR-009 Recommendations domain / engine shape — **P9-E1 Done** (2026-08-08).
  - Deterministic recommendation path (`focus_dip_pace_hint_v1`) — **P9-E2 Done** (2026-08-08).
  - Recommendations IPC / UX (`get_recommendations` + Suggestions) — **P9-E3 Done** (2026-08-08).

- [x] **Phase 10: Plugin wave-1** (Sprint 19–20) — closed 2026-08-10 (E1–E3 Done; cluster PR after freeze)
  - ADR-010 Browser categories + Observation contract — **P10-E1 Done** (2026-08-10).
  - Browser categories collector plugin — **P10-E2 Done** (2026-08-10).
  - `DistractionScore` catalog Feature — **P10-E3 Done** (2026-08-10).

- [x] **Phase 11: AI coaching polish** (Sprint 21–22) — closed 2026-08-10 (E1–E3 Done; cluster PR after freeze)
  - ADR-011 prompt packs + provider UX boundaries — **P11-E1 Done** (2026-08-10).
  - Versioned prompt packs in `report-engine` — **P11-E2 Done** (2026-08-10).
  - Local LLM provider UX + pack-aware Report flow — **P11-E3 Done** (2026-08-10).

- [x] **Phase 12: Ambient + commercial packaging** (Sprint 23–24) — closed 2026-08-10 (E1–E3 Done; cluster PR after freeze)
  - ADR-012 ambient + packaging boundaries — **P12-E1 Done** (2026-08-10).
  - Now Playing ambient plugin — **P12-E2 Done** (2026-08-10).
  - `AmbientMediaShare` + packaging runbook — **P12-E3 Done** (2026-08-10).

- [ ] **Phase 13: Plugin wave-2 (Git activity)** (Sprint 25–26) — opened 2026-08-10
  - ADR-013 Git activity + Observation contract — **P13-E1 Done** (2026-08-10).
  - Git activity collector plugin — **P13-E2 Ready** (T1).
  - `GitActivityRate` catalog Feature — **P13-E3** (after E2).

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2:** merged [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)  
**Phase 3:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` archive · handoffs `P3-*-qa-to-pm.md`  
**Phase 4:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai` (cluster PR when ready)  
**Phase 5:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P5-*-qa-to-pm.md` · branch `phase/5-wearable-dogfood` (cluster PR when ready)  
**Phase 6:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P6-*-qa-to-pm.md` · tip `phase/6-dogfood-fixes` (PR after freeze)  
**Phase 7:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P7-*-qa-to-pm.md` · branch `phase/7-trust-layer` (PR after freeze)  
**Phase 8:** Done — `docs/SPRINT_ROADMAP.md` · handoffs `P8-*-qa-to-pm.md` · branch `phase/8-pattern-discovery` (PR after freeze)  
**Phase 9:** Done — `docs/SPRINT_ROADMAP.md` · handoffs `P9-*-qa-to-pm.md` · branch `phase/9-recommendations` (PR after freeze)  
**Phase 10:** Done — `docs/SPRINT_ROADMAP.md` · handoffs `P10-*-qa-to-pm.md` · branch `phase/10-plugin-wave-1` (PR after freeze)  
**Phase 11:** Done — `docs/SPRINT_ROADMAP.md` · handoffs `P11-*-qa-to-pm.md` · branch `phase/11-ai-coaching-polish` (PR after freeze)  
**Phase 12:** Done 2026-08-10 — ADR-012 → Now Playing → `AmbientMediaShare` + packaging runbook · branch `phase/12-ambient-packaging` (PR after freeze)
**Phase 13:** Opened 2026-08-10 — Plugin wave-2 = **Git** (ADR-013) · Ready **P13-E2-T1** (Git activity plugin) · branch `phase/13-plugin-wave-2` (PR after freeze)
**Next Kanban:** P13-E2-T1 Git activity plugin · **PR freeze until 2026-09-01**
**Vision:** `/docs/00-vision.md` · canvas snapshot `PROJECT_CANVAS.md`  
**Git:** related-work branches → local commits; **PR freeze until 2026-09-01** — see `docs/12-development.md` / `.cursor/rules/06-git-agent-policy.mdc`.
