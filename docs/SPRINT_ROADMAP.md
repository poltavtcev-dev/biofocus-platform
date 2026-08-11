# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 15: Companion HRV + autonomy** (Sprint 29–30) — **Done** (2026-08-11).  
> Phase 0–15 Done. ADR-016 HealthKit HRV + Auto-sync. ADR-015 ambient light **parked**. Next: **PM-GATE-POST-P15**.

**Phase 15 goal (shipped):** Autonomous wearable companion — HR + HRV (SDNN) → local queue → Desktop ingest; Core accepts SDNN-or-RMSSD. Sleep/steps out of scope.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 · `/docs/00-vision.md`.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight` / `Recommendation`)
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); Features/Insights/Recommendations считаются в Core
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; poll/refresh по событию или редкому таймеру
- [ ] **Нет новой SQLite-схемы** без ADR + approve
- [ ] **Commit по связанному кластеру** (локально / feature-ветка) — `docs/12-development.md`  
- [ ] **PR freeze до 2026-09-01** — не открывать PR / не мержить в `main` через PR (`06-git-agent-policy.mdc`)
- [ ] Copy спокойный, неоценочный (не «ты выгорел» / clinical claims)
- [ ] **LAN / companion:** Bearer обязателен; нет cloud telemetry; default = loopback
- [ ] **LLM не считает** Recommendations / Features / Evidence (interpret-only stays L5)
- [ ] **Plugins:** opt-in; no full URL / keystroke content / employee-surveillance framing; stop joins background work
- [ ] **Coaching:** never auto-invoke LLM on app / Dashboard open; opt-in local provider; no cloud LLM by default
- [ ] **Ambient / packaging:** opt-in ambient capture; no always-on mic/geo dumps; commercial packaging ≠ closed Feature math; optional sync off by default
- [ ] **Git allowlist:** user-chosen roots only; never widen `git_activity` Observation payloads with paths/remotes/diffs
- [ ] **Companion autonomy:** HealthKit event → local queue → flush; no busy-loop; no cloud relay; no clinical claims on HRV/SDNN

---

## Kanban Overview (Post–Phase 15)

| Status | IDs |
| :--- | :--- |
| **Ready** | **PM-GATE-POST-P15** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–14 · **PM-GATE-POST-P14** · **P15-E1-T1** · **P15-E2-T1** · **P15-E3-T1** · **Phase 15** (companion HRV) · ADR-015 ambient light parked |

**Epic status:** P15-E1 ✅ · P15-E2 ✅ · P15-E3 ✅ · **Phase 15** ✅

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **PM-GATE-POST-P15** — Choose next Phase 16+ slice. Brief: `docs/handoffs/PM-GATE-POST-P15-pm-brief.md`. Role: **PM**.

**Closed:** Phase 15 companion (2026-08-11) — ADR-016 → Core SDNN-or-RMSSD + iOS Auto-sync → dogfood. Branch: `phase/15-companion-hrv-autonomy`. Evidence: `docs/handoffs/P15-E*-qa-to-pm.md`.

**Parked:** ADR-015 ambient light — collector/Feature deferred.

**Ops note:** **PR freeze until 2026-09-01** — cluster on `phase/15-companion-hrv-autonomy`.

---

## Phase 15 — Companion HRV + autonomy (Done)

### Epic P15-E1 — ADR-016 companion scope
**P15-E1 shipped:** ADR-016 = HealthKit HRV (SDNN) + existing HR; event → queue → flush; Core accepts `rmssd_ms` OR `sdnn_ms`; no new SQLite; ambient light E2/E3 deferred. QA Pass 2026-08-11.

### Epic P15-E2 — Core + iOS autonomy
**P15-E2 shipped:** `normalize_hrv` SDNN-only; Focus/Recovery prefer rmssd then sdnn; iOS HRV + ObservationQueue + HKObserver background delivery + Auto-sync UI. QA Pass 2026-08-11.

### Epic P15-E3 — Dogfood companion
**P15-E3 shipped:** `docs/12-development.md` companion autonomy runbook; iOS README; Auto-sync / last sync / calm empty HRV. QA Pass 2026-08-11.

**Out of scope:** sleep/steps/SpO2/ECG; ambient light collector; weather; IDE; App Store; cloud relay; PR during freeze.

---

## Queue (Post–Phase 15)

1. **PM-GATE-POST-P15** — choose next slice (ambient light resume · IDE · weather · other)

**Branch:** `phase/15-companion-hrv-autonomy` (PR after freeze).
