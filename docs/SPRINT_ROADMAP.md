# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 17: Wearable depth + chart ranges** (Sprint 33–34) — Active.  
> Phase 0–16 Done. Goal: Mi Fitness / HealthKit max Observations + Dashboard ranges **1h / 8h / 12h / 1d / 1w** + deterministic analysis first (ADR-017 sequencing; **ADR-018** contracts).

**Phase 17 goal:** Lock contracts (**ADR-018** / E1) → expand Companion HealthKit ingest (E2) → chart ranges + Features without requiring LLM (E3). Soft-optional HRV; no Mi Cloud.

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
- [ ] **Ambient light:** opt-in; coarse light labels/levels only — no camera frames / screen contents / precise geo
- [ ] **Wearable depth:** HealthKit only (no Mi Cloud); soft-optional HRV; new types only per Phase 17 contract ADR
- [ ] **Chart ranges:** recompute-on-read; UI ↛ SQLite; Snapshot = latest (series on chart)

---

## Kanban Overview (Phase 17 Active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P17-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–16 · **PM-GATE-POST-P14** · **PM-GATE-POST-P15** · **P16-E1-T1** · **P16-E2-T1** · **P17-E1-T1** · **P17-E2-T1** |

**Epic status:** P17-E1 ✅ · P17-E2 ✅ · P17-E3 ○ · Phase 16 ✅

**Phase 17 on `/docs/14-roadmap.md`:** opened 2026-08-11 · ADR-017 · **ADR-018** ✅ · E1+E2 Done · Ready E3 charts/Features

**Рекомендуемый порядок (Phase 17):**  
P17-E1-T1 (ADR-018) ✅ → P17-E2-T1 (Companion emit) ✅ → P17-E3-T1 (ranges + Features)

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P17-E3-T1** — Chart ranges IPC/UI + Features from wearable Observations (**ADR-018**). Brief: `docs/handoffs/P17-E3-T1-pm-brief.md`. Role: **Dev|UX**.

**Just closed:** **P17-E2-T1** (2026-08-11) — Companion HealthKit expand per ADR-018 (QA Pass). Branch: `phase/17-wearable-charts`. Evidence: `docs/handoffs/P17-E2-T1-qa-to-pm.md`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 17 on `phase/17-wearable-charts`; local commits OK; no PR. Physical-device dogfood still recommended for Companion emit.

---

## Phase 17 — Wearable depth + chart ranges (Active)

### Epic P17-E1 — Contracts ADR (**ADR-018**) ✅
**Goal:** Lock Observation payloads + chart-range IPC stance before Companion/UI code. SoT: `docs/decision-log.md` ADR-018 + `07-contracts` / `09-api`.  
**Status:** Done 2026-08-11 (QA Pass). Evidence: `docs/handoffs/P17-E1-T1-qa-to-pm.md`.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P17-E1-T1** ✅ | Lock HealthKit + chart-range contracts (**ADR-018**) | Dev | docs + decision-log | See `P17-E1-T1-pm-brief.md` | Phase 16 Done · ADR-017 |

### Epic P17-E2 — Companion HealthKit expand ✅
**Goal:** Emit ADR-018 Observation types (`step_count` / `active_energy` / `sleep_interval`; soft-optional `oxygen_saturation`; keep HR/HRV) via existing queue → ingest autonomy path.  
**Status:** Done 2026-08-11 (QA Pass). Evidence: `docs/handoffs/P17-E2-T1-qa-to-pm.md`.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P17-E2-T1** ✅ | Expand iOS Companion / HealthKit ingest per ADR-018 | Dev | apps/companion + bio-spec/pipeline | See `P17-E2-T1-pm-brief.md` | P17-E1-T1 ✅ |

### Epic P17-E3 — Dashboard ranges + Features
**Goal:** Range picker 1h/8h/12h/1d/1w via `get_feature_series` (recompute-on-read) + Snapshot latest UX + catalog Features from new Observations (no LLM required).

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P17-E3-T1** | Chart ranges IPC/UI + Features from wearable Observations | Dev\|UX | desktop + feature-engine | See `P17-E3-T1-pm-brief.md` | P17-E2-T1 ✅ |

**Out of scope:** Mi Cloud / unofficial API; Feature history SQLite v1; clinical claims; weather/IDE/App Store/NotificationPressure as this phase primary; PR during freeze.

---

## Phase 16 archive (Done)

<details>
<summary>Phase 16 Kanban & epics (closed 2026-08-11 — ambient light)</summary>

**Done:** P16-E1 (T1) · P16-E2 (T1).  
ADR-015 collector + `AmbientLightShare`. Branch: `phase/16-ambient-light` (cluster PR after freeze).

Evidence: `docs/handoffs/P16-*-qa-to-pm.md`.

</details>

## Phase 15 archive (Done)

<details>
<summary>Phase 15 Kanban & epics (closed 2026-08-11 — companion HRV)</summary>

**Done:** P15-E1 (T1) · P15-E2 (T1) · P15-E3 (T1).  
ADR-016 HealthKit HRV + Auto-sync. Branch: `phase/15-companion-hrv-autonomy` (cluster PR after freeze).

Evidence: `docs/handoffs/P15-*-qa-to-pm.md`.

</details>

---

## Queue (Phase 17)

1. P17-E1-T1 — Lock HealthKit + chart-range contracts (**ADR-018**) ← **Done**  
2. P17-E2-T1 — Companion HealthKit expand ← **Done**  
3. **P17-E3-T1** — Chart ranges + Features ← **Ready**  

**Git:** `phase/17-wearable-charts` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P17-E3-T1-pm-brief.md`.  
**Deferred (later / other tracks):** IDE · weather ambient · App Store packaging · NotificationPressure.

**Prior intent note:** `docs/handoffs/PARKED-P17-wearable-dashboard-intent.md` — **contract-locked** (ADR-018); Companion emit Done; charts/Features = this Ready task.
