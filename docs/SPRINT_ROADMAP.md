# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 6: Life context** (Sprint 11–12) — **Opened** 2026-08-05.  
> Phase 1–5 Done. Goal: **Life Events** as `Observation` kinds + **Calendar** → meeting-density Features — no parallel Life Events DB, no cloud.

**Phase 6 goal:** Dogfood-ready life context: user can log Life Events (Coffee / Walk / Lunch / Workout, …) as Observations; Calendar-derived Observations feed `MeetingDensity` / `RecoveryBetweenMeetings`. Calm, non-clinical copy. No new SQLite tables without ADR + approve.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 analysis stack · horizon P7–P12+ — `/docs/00-vision.md`. **Do not** pull P7+ Features/plugins into this Kanban until PM opens that phase.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight`)
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); Features/Insights считаются в Core
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; poll/refresh по событию или редкому таймеру
- [ ] **Нет новой SQLite-схемы** без ADR + approve
- [ ] **Commit / PR по связанному кластеру** — `docs/12-development.md`
- [ ] Copy спокойный, неоценочный (не «ты выгорел» / clinical claims)
- [ ] **LAN / companion:** Bearer обязателен; нет cloud telemetry; default = loopback

---

## Kanban Overview (Phase 6 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P6-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **P6-E1-T1** · **P6-E2-T1** |

**Epic status:** P6-E1 ✅ · P6-E2 ✅ · P6-E3 ⬜

**Phase 6 on `/docs/14-roadmap.md`:** opened 2026-08-05

**Рекомендуемый порядок:**  
~~P6-E1-T1~~ → ~~P6-E2-T1~~ → **P6-E3-T1** → P6-E3-T2

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Desktop-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **related work → PR** (`docs/12-development.md`).  
**Suggested branch:** `phase/6-life-context` (base on latest `main`; Phase 5 cluster PR may land separately on `phase/5-wearable-dogfood`).

### Active assignment

**Ready now:** **P6-E3-T1** — Calendar → Observations (dogfood source) (Dev).  
Brief: `docs/handoffs/P6-E3-T1-pm-brief.md`.

**Closed:** P6-E2-T1 (QA Pass with notes, 2026-08-05) — Desktop Menubar quick-log Life Events via IPC; Epic **P6-E2** ✅. Evidence: `docs/handoffs/P6-E2-T1-qa-to-pm.md`.

**Closed:** P6-E1-T1 (QA Pass, 2026-08-05) — Life Events as Observation kinds + ADR-006; Epic **P6-E1** ✅. Evidence: `docs/handoffs/P6-E1-T1-qa-to-pm.md`.

**Closed (Phase 5):** P5-E3-T2 (2026-08-05) — dogfood runbook + contract docs; Epic **P5-E3** ✅ · Phase 5 Kanban complete. Branch: `phase/5-wearable-dogfood`.

**Ops note:** Phase 5 cluster PR on `phase/5-wearable-dogfood` remains optional parallel ops — does not block Phase 6 if `main` already has ingest + companion contracts. Phase 6 code cluster: fold ADR/contracts + Life Events validation + quick-log IPC into PR on `phase/6-life-context` when ready.

---

## Epic P6-E1 — Life Events Observation contract ✅ Done

**Цель:** Life Events (Coffee, Walk, Lunch, Workout, …) as first-class `Observation` kinds — `data_type` + payload — not a parallel DB. ADR + contracts + ingest accept path.

### P6-E1-T1 — Life Events Observation kinds + ADR-006 ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/bio-spec` (and/or ingest validation), `docs/07-contracts.md`, `docs/09-api.md`, `docs/decision-log.md`, glossary/catalog notes as needed |
| **Depends on** | Phase 2 Observation ingest; vision §4 rule 6 |
| **AC** | (1) Record **ADR-006**: Life Events as Observation kinds (not a parallel table/store); list v1 kinds (at least Coffee / Walk / Lunch / Workout — exact `data_type` / payload shape documented). (2) Contracts (`07-contracts` / `09-api`) document the JSON shape + examples. (3) Ingest (or bio-spec validation) **accepts** valid Life Event Observations and rejects malformed ones with explicit errors (no `unwrap` in production). (4) Round-trip test: ingest → storage read (existing Observation repository) for ≥1 Life Event kind. (5) No new SQLite schema unless ADR + user approve (prefer existing Observation store). (6) Idle-safe. (7) Handoff: `docs/handoffs/P6-E1-T1-dev-to-qa.md`. |
| **Out of scope** | Desktop quick-log UI (→ **E2-T1**), Calendar sync (→ **E3**), `MeetingDensity` Feature compute, ActivityBalance Feature |
| **Shipped** | ADR-006; `bio-spec` `validate_*` + v1 kinds `coffee`/`walk`/`lunch`/`workout`; ingest `400 invalid_life_event`; round-trip via `ObservationRepository`; contracts/API/glossary; QA Pass 2026-08-05. Epic **P6-E1** closed. |

---

## Epic P6-E2 — Manual Life Event capture ✅ Done

**Цель:** Desktop UX to log a Life Event → Core as Observation (IPC only).

### P6-E2-T1 — Desktop quick-log Life Events ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | UX + Dev |
| **Modules** | `apps/desktop` (+ IPC / host glue), Core path that creates Observation |
| **Depends on** | P6-E1-T1 |
| **AC** | User can log a v1 Life Event from Desktop with calm copy; event becomes an Observation via IPC (UI ↛ SQLite); visible via existing status / Dashboard / storage path as appropriate; idle-safe (no busy-loop); smoke steps in handoff. |
| **Out of scope** | Calendar import, wearable auto-detect of workouts, new Insight rules |
| **Shipped** | Menubar Life events (Coffee / Walk / Lunch / Workout) + Recent; IPC `log_life_event` / `list_recent_life_events` → `bio_spec` + `ObservationRepository`; no poll for logging; QA Pass with notes 2026-08-05. Epic **P6-E2** closed. |

---

## Epic P6-E3 — Calendar → Meeting Features

**Цель:** Calendar-derived Observations feed `MeetingDensity` and `RecoveryBetweenMeetings` in Core.

### P6-E3-T1 — Calendar → Observations (dogfood source) ✅ Ready
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | collector / host plugin path, contracts docs |
| **Depends on** | P6-E1-T1 (Observation contract discipline); E2 helpful but not required |
| **AC** | Opt-in local Calendar source produces Calendar/meeting Observations (shape documented); no cloud calendar sync required for dogfood; idle-safe polling or event-driven refresh; privacy: no event titles/bodies leaked to logs beyond what’s needed; tests with fixtures. Handoff with operator smoke notes. |
| **Out of scope** | Google/Outlook cloud OAuth, MeetingDensity formula (→ **T2**), Life Event UI |

### P6-E3-T2 — MeetingDensity + RecoveryBetweenMeetings Features
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/feature-engine`, `docs/06-feature-catalog.md`, snapshot IPC consumers as needed |
| **Depends on** | P6-E3-T1 |
| **AC** | `MeetingDensity` and `RecoveryBetweenMeetings` registered + computed from Calendar Observations; catalog docs (formula / units / deps / provenance); unit tests with synthetic calendar Observations; idle-safe; calm naming (no clinical claims). Handoff: `docs/handoffs/P6-E3-T2-dev-to-qa.md`. |
| **Out of scope** | Feature-level confidence / explanation factors (→ Phase 7), CognitiveLoad, Pattern Discovery |

---

## Phase 5 archive (Done)

<details>
<summary>Phase 5 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P5-E1 (T1–T2) · P5-E2 (T1) · P5-E3 (T1–T2).  
Opt-in LAN ingest (ADR-005) → advertise URL hints → Companion LAN Base URL / token/QR → runnable iOS HealthKit companion → dogfood runbook.

Evidence: `docs/handoffs/P5-*-qa-to-pm.md` · branch `phase/5-wearable-dogfood` (cluster PR when ready).

</details>

---

## Phase 4 archive (Done)

<details>
<summary>Phase 4 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P4-E1 (T1–T3) · P4-E2 (T1–T3) · P4-E3 (T1–T3).  
Feature snapshot IPC → Dashboard + Recharts → Knowledge Insights → report-engine + optional local LLM + Report UX.

Evidence: `docs/handoffs/P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai` (cluster PR when ready).

</details>

---

## Phase 3 archive (Done)

<details>
<summary>Phase 3 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P3-E1 (T1–T4) · P3-E2 (T1–T4) · P3-E3 (T1–T3).  
Pipeline quality → Feature DAG → Menubar AlertLevel IPC + UX.

Evidence: `docs/handoffs/P3-*-qa-to-pm.md` · PRs #5–#23 (cluster) · Menubar via [PR #24](https://github.com/poltavtcev-dev/biofocus-platform/pull/24).

</details>

---

## Phase 2 archive (Done)

<details>
<summary>Phase 2 Kanban & epics (closed 2026-08-04, PR #2)</summary>

**Done:** P2-E0 · P2-E1 (T1–T4) · P2-E2 (T1–T3) · P2-E3 (T1–T2).  
Evidence: `docs/handoffs/P2-*-qa-to-pm.md` · [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2).

Ingest loopback + pairing · macOS collector · companion sample + Copy/QR · `dbError` sanitize.

</details>

---

## Phase 1 archive (Done)

<details>
<summary>Phase 1 Kanban & epics (closed 2026-08-04)</summary>

**Done:** Epic E1–E4. Evidence: `docs/handoffs/P1-E4-T1-acceptance.md`, `P1-E4-T2-qa-to-pm.md`.

Epics: workspace/`bio-spec`/`runtime` → SQLite WAL + `ObservationRepository` → Tauri Menubar + `get_status` → exit docs.

</details>

---

## Role × Module Matrix (Phase 6)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P6-E1-T1 Life Events kinds + ADR | | ● | ○ | | bio-spec / ingest + docs |
| P6-E2-T1 Desktop quick-log | ○ | ○ | ○ | ● | apps/desktop |
| P6-E3-T1 Calendar → Observations | | ● | ○ | | collector / host |
| P6-E3-T2 MeetingDensity Features | | ● | ○ | | feature-engine |

● = owner · ○ = collaborator

---

## Sprint 11–12 — Queue

1. ~~P6-E1-T1 — Life Events Observation kinds + ADR-006~~ **Done** (QA Pass) · Epic **P6-E1** ✅  
2. ~~P6-E2-T1 — Desktop quick-log Life Events~~ **Done** (QA Pass with notes) · Epic **P6-E2** ✅  
3. **P6-E3-T1 — Calendar → Observations (dogfood source)** ← **Ready**  
4. P6-E3-T2 — MeetingDensity + RecoveryBetweenMeetings Features  

**Git:** `phase/6-life-context` → related commits → **one cluster PR** when E1–E3 (or coherent subset) is Ready to ship.
