# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 15: Companion HRV + autonomy** (Sprint 29–30) — Active.  
> Phase 0–14 Done. ADR-016 HealthKit HRV + Auto-sync (ADR-015 ambient light **parked**).

**Phase 15 goal:** Autonomous wearable companion — HR + HRV (SDNN) → local queue → Desktop ingest; Core accepts SDNN-or-RMSSD. Sleep/steps out of scope. Ambient light collector deferred.

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

## Kanban Overview (Phase 15 Active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P15-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–14 · **PM-GATE-POST-P14** · **P15-E1-T1** · **P15-E2-T1** |

**Epic status:** P15-E1 ✅ · P15-E2 ✅ · P15-E3 ○ · Phase 14 ✅ · **PM-GATE-POST-P14** ✅

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P15-E3-T1** — Dogfood companion + Auto-sync UI/status. Brief: `docs/handoffs/P15-E3-T1-pm-brief.md`. Role: **Dev/UX**.  
**Note:** Build + QA Pass already on disk (`P15-E3-T1-*-qa-to-pm.md`) — next chat may **pm-close** directly.

**Closed:** P15-E2-T1 (QA Pass, 2026-08-11) — Core SDNN-or-RMSSD + iOS Auto-sync (queue / observers / background delivery). Epic **P15-E2** ✅. Evidence: `docs/handoffs/P15-E2-T1-qa-to-pm.md`.

**Closed:** P15-E1-T1 (QA Pass, 2026-08-11) — ADR-016 companion scope; ambient light parked. Epic **P15-E1** ✅. Evidence: `docs/handoffs/P15-E1-T1-qa-to-pm.md`.

**Parked:** ADR-015 ambient light — collector / `AmbientLightShare` deferred to a later PM gate.

**Ops note:** **PR freeze until 2026-09-01** — cluster on `phase/15-companion-hrv-autonomy`.

---

## Phase 15 — Companion HRV + autonomy (Active)

### Epic P15-E1 — ADR-016 companion scope
**P15-E1 shipped:** ADR-016 = HealthKit HRV (SDNN) + existing HR; event → queue → flush; Core accepts `rmssd_ms` OR `sdnn_ms`; no new SQLite; ambient light E2/E3 deferred. QA Pass 2026-08-11.

### Epic P15-E2 — Core + iOS autonomy
**P15-E2 shipped:** `normalize_hrv` SDNN-only; Focus/Recovery prefer rmssd then sdnn; iOS HRV + ObservationQueue + HKObserver background delivery + Auto-sync wiring. QA Pass 2026-08-11.

### Epic P15-E3 — Dogfood companion
**Goal:** Operator dogfood runbook + calm Companion Auto-sync UI/status.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P15-E3-T1** | Dogfood + Companion Auto-sync UI | Dev/UX | docs + ios | See `P15-E3-T1-pm-brief.md` | P15-E2-T1 |

**Out of scope:** sleep/steps/SpO2/ECG; ambient light collector; weather; IDE; App Store; cloud relay; PR during freeze.

---

## Queue (Phase 15)

1. ~~P15-E1-T1 — ADR-016 companion scope~~ ✅  
2. ~~P15-E2-T1 — Core SDNN + iOS Auto-sync~~ ✅  
3. **P15-E3-T1** — Dogfood + Companion UI ← **Ready**  

**Branch:** `phase/15-companion-hrv-autonomy` (PR after freeze).  
**Brief:** `docs/handoffs/P15-E3-T1-pm-brief.md`.  
**Deferred:** ambient light resume · IDE · weather · App Store · NotificationPressure.
