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

- [ ] **Phase 9: Recommendations** (Sprint 17–18) — opened 2026-08-08 (PM gate)
  - ADR-009 Recommendations domain / engine shape — **P9-E1 Done** (2026-08-08)
  - Deterministic recommendation path (`focus_dip_pace_hint_v1`) — **P9-E2 Done** (2026-08-08)
  - Recommendations IPC / UX — **P9-E3 Ready**

### Horizon (accepted vision ladder — not Kanban-Ready yet)

Product philosophy + sequencing: `/docs/00-vision.md`. Open each phase via PM gate only.

- [ ] **Phase 10: Plugin wave-1** — IDE/Git or Browser categories (dogfood-driven)
- [ ] **Phase 11: AI coaching polish** — prompt packs / provider UX (interpret-only)
- [ ] **Phase 12+: Ambient + commercial packaging** — music/weather/light; signed builds / updates; optional user sync

**Evidence Phase 1:** `docs/handoffs/P1-E4-T1-acceptance.md` · `P1-E4-T2-qa-to-pm.md`  
**Phase 2:** merged [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)  
**Phase 3:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` archive · handoffs `P3-*-qa-to-pm.md`  
**Phase 4:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai` (cluster PR when ready)  
**Phase 5:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P5-*-qa-to-pm.md` · branch `phase/5-wearable-dogfood` (cluster PR when ready)  
**Phase 6:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P6-*-qa-to-pm.md` · tip `phase/6-dogfood-fixes` (PR after freeze)  
**Phase 7:** E1–E3 Done — `docs/SPRINT_ROADMAP.md` · handoffs `P7-*-qa-to-pm.md` · branch `phase/7-trust-layer` (PR after freeze)  
**Phase 8:** Done — `docs/SPRINT_ROADMAP.md` · handoffs `P8-*-qa-to-pm.md` · branch `phase/8-pattern-discovery` (PR after freeze)  
**Phase 9:** Open — P9-E1–E2 Done · Ready **P9-E3-T1** · `docs/SPRINT_ROADMAP.md` · brief `docs/handoffs/P9-E3-T1-pm-brief.md` · branch `phase/9-recommendations`  
**Next Kanban:** P9-E3-T1 (closes Phase 9) · **PR freeze until 2026-09-01**  
**Vision:** `/docs/00-vision.md` · canvas snapshot `PROJECT_CANVAS.md`  
**Git:** related-work branches → local commits; **PR freeze until 2026-09-01** — see `docs/12-development.md` / `.cursor/rules/06-git-agent-policy.mdc`.
