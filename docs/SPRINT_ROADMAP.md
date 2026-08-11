# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 18: Notification pressure** (Sprint 35–36) — Active.  
> Phase 0–17 Done. Goal: lock notification Observation contract (**ADR-019**) → opt-in macOS collector → catalog **`NotificationPressure`** (interruption intensity). No notification body/title content.

**Phase 18 goal:** ADR contracts (E1) → collector emit (E2) → `NotificationPressure` Feature (E3). Personal self-tracking; calm non-clinical copy; opt-in default **off**.

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
- [ ] **Notifications:** opt-in; coarse counts/cadence only — **no** notification body/title/content; personal self-tracking only

---

## Kanban Overview (Phase 18 Active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P18-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–17 · **PM-GATE-POST-P14** · **PM-GATE-POST-P15** · **PM-GATE-POST-P17** · **P16-E1-T1** · **P16-E2-T1** · **P17-E1-T1** · **P17-E2-T1** · **P17-E3-T1** · **P18-E1-T1** · **P18-E2-T1** |

**Epic status:** P18-E1 ✅ · P18-E2 ✅ · P18-E3 ○ · Phase 17 ✅

**Phase 18 on `/docs/14-roadmap.md`:** opened 2026-08-11 · **ADR-019** ✅ · E1+E2 Done · Ready E3 Feature

**Рекомендуемый порядок (Phase 18):**  
P18-E1-T1 (ADR-019) ✅ → P18-E2-T1 (collector) ✅ → P18-E3-T1 (`NotificationPressure`)

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P18-E3-T1** — Ship catalog Feature **`NotificationPressure`** per **ADR-019**. Brief: `docs/handoffs/P18-E3-T1-pm-brief.md`. Role: **Dev**.

**Just closed:** **P18-E2-T1** (2026-08-11) — notification collector plugin (QA Pass). Branch: `phase/18-notification-pressure`. Evidence: `docs/handoffs/P18-E2-T1-qa-to-pm.md`. Note: live Notification Center OS mapping still soft-fail idle; Feature works on scripted / fixture / ingest evidence.

**Ops note:** **PR freeze until 2026-09-01** — Phase 18 on `phase/18-notification-pressure`; local commits OK; no PR.

---

## Phase 18 — Notification pressure (Active)

### Epic P18-E1 — Contracts ADR (**ADR-019**) ✅
**Goal:** Lock notification Observation `data_type: "notification_event"` + privacy bar (no body/title) + Feature scope `NotificationPressure` before collector code. SoT: `docs/decision-log.md` ADR-019 + contracts sketches (`07-contracts`, `08-plugin-sdk`, catalog stub).  
**Status:** Done 2026-08-11 (QA Pass). Evidence: `docs/handoffs/P18-E1-T1-qa-to-pm.md`.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P18-E1-T1** ✅ | Lock notification Observation + `NotificationPressure` scope (**ADR-019**) | Dev | docs + decision-log | See `P18-E1-T1-pm-brief.md` | PM-GATE-POST-P17 |

### Epic P18-E2 — Notification collector ✅
**Goal:** Opt-in macOS plugin (`com.biofocus.macos.notifications`, `BIOFOCUS_NOTIFICATION_EVENTS`) emits locked `notification_event` Observations via existing ingest channel (no body/title content).  
**Status:** Done 2026-08-11 (QA Pass). Evidence: `docs/handoffs/P18-E2-T1-qa-to-pm.md`. System probe soft-fails idle until privacy-safe OS mapping exists.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P18-E2-T1** ✅ | Implement notification collector plugin per ADR-019 | Dev | macos-collector + bio-spec/pipeline | See `P18-E2-T1-pm-brief.md` | P18-E1-T1 ✅ |

### Epic P18-E3 — NotificationPressure Feature
**Goal:** Catalog Feature `NotificationPressure` from `notification_event` Observations (omit empty; ADR-007 confidence + factors where practical).

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P18-E3-T1** | Ship `NotificationPressure` catalog Feature | Dev | feature-engine + docs | See `P18-E3-T1-pm-brief.md` | P18-E2-T1 ✅ |

**Out of scope:** IDE plugin; weather ambient; App Store product; notification content capture; workplace surveillance framing; `CognitiveLoad` (later — needs more inputs); PR during freeze.

---

## Phase 17 archive (Done)

<details>
<summary>Phase 17 Kanban & epics (closed 2026-08-11 — wearable depth + chart ranges)</summary>

**Done:** P17-E1 (T1) · P17-E2 (T1) · P17-E3 (T1).  
ADR-017 sequencing · **ADR-018** contracts · Companion HealthKit emit · `get_feature_series` + `ActivityBalance` / `EnergyScore` / `SleepDebt`.  
Branch: `phase/17-wearable-charts` (cluster PR after freeze).

Evidence: `docs/handoffs/P17-*-qa-to-pm.md`.

</details>

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

## Queue (Phase 18)

1. P18-E1-T1 — Lock notification Observation + `NotificationPressure` (**ADR-019**) ← **Done**  
2. P18-E2-T1 — Notification collector plugin ← **Done**  
3. **P18-E3-T1** — `NotificationPressure` Feature ← **Ready**  

**Git:** `phase/18-notification-pressure` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P18-E3-T1-pm-brief.md`.  
**Deferred (later / other tracks):** IDE · weather ambient · App Store packaging · Companion dogfood polish (parallel OK) · `CognitiveLoad` · live NC OS mapping (soft-fail until future probe).

**Gate evidence:** `docs/handoffs/PM-GATE-POST-P17-pm-brief.md` — **Done** (chose NotificationPressure).
