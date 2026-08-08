# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 9: Deterministic Recommendations** (Sprint 17–18) — **Opened** 2026-08-08.  
> Phase 0–8 Done. Goal: **L4 Recommendations** — deterministic suggested actions with Evidence (not LLM coaching, not clinical advice). ADR first; evolve from thin `Insight.actionRecommendation` only after decision.

**Phase 9 goal:** User sees calm, evidence-backed action suggestions derived from Features / Insights — Personal Pattern Discovery stays the north star; Recommendations are optional next-step hints, not diagnoses or automation.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 · horizon P10–P12+ — `/docs/00-vision.md`. **Do not** pull P10+ into this Kanban until PM opens that phase.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight` / Phase-9 `Recommendation` per ADR)
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

---

## Kanban Overview (Phase 9 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P9-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **P9-E1-T1** · **P9-E2-T1** |

**Epic status:** P9-E1 ✅ · P9-E2 ✅ · P9-E3 ⬜ (T1 Ready)

**Phase 9 on `/docs/14-roadmap.md`:** opened 2026-08-08 · ADR-009 + engine path shipped

**Рекомендуемый порядок:**  
~~P9-E1-T1~~ → ~~P9-E2-T1~~ → **P9-E3-T1**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).  
**Suggested branch:** `phase/9-recommendations`.

### Active assignment

**Ready now:** **P9-E3-T1** — Recommendations IPC / UX (Dev + UX).  
Brief: `docs/handoffs/P9-E3-T1-pm-brief.md`.

**Closed:** P9-E2-T1 (QA Pass, 2026-08-08) — `Recommendation` + `EvidenceRef::Insight` + `focus_dip_pace_hint_v1` + `evaluate_recommendations`; Epic **P9-E2** ✅. Evidence: `docs/handoffs/P9-E2-T1-qa-to-pm.md`. Branch: `phase/9-recommendations`.

**Closed:** P9-E1-T1 (QA Pass, 2026-08-08) — ADR-009; Epic **P9-E1** ✅. Evidence: `docs/handoffs/P9-E1-T1-qa-to-pm.md`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 9 cluster on `phase/9-recommendations`; local commits OK; one cluster PR **after** freeze (or when user lifts it).

---

## Epic P9-E1 — Recommendations ADR & domain contract ✅

**Цель:** Decide how L4 Recommendations relate to existing `Insight.actionRecommendation`, Evidence, and engines — ADR before new types/crates/schema.

### P9-E1-T1 — ADR-009: Recommendations domain / engine shape ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `docs/decision-log.md`; domain / glossary / API notes |
| **Shipped** | ADR-009 = first-class `Recommendation` + `RecommendationRule` in `knowledge-engine` (evaluate-on-read); thin `actionRecommendation` stays optional hint; no Recommendation SQLite; E2 sketch `focus_dip_pace_hint_v1`. QA Pass 2026-08-08. |
| **Evidence** | `docs/handoffs/P9-E1-T1-qa-to-pm.md` |

---

## Epic P9-E2 — Deterministic Recommendation path ✅

**Цель:** First evaluate-on-read Recommendation(s) with Evidence using ADR-009 decision.

### P9-E2-T1 — Recommendations v1 engine path ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/bio-spec`, `crates/knowledge-engine` |
| **Shipped** | `Recommendation` + `EvidenceRef::Insight`; `RecommendationRule` + `register_recommendations_v1` + `focus_dip_pace_hint_v1` (Focus-below-baseline pattern + FocusScore confidence ≥ 0.4); `evaluate_recommendations` / `evaluate_insights_and_recommendations`. QA Pass 2026-08-08. |
| **Evidence** | `docs/handoffs/P9-E2-T1-qa-to-pm.md` |

---

## Epic P9-E3 — Surface Recommendations (IPC / Dashboard)

**Цель:** Expose Recommendations via Core IPC + calm Dashboard (or Insights-adjacent) presentation — UI ↛ DB.

### P9-E3-T1 — Recommendations IPC / UX ✅ Ready
| Field | Value |
| :--- | :--- |
| **Role** | Dev (+ UX if UI copy) |
| **Modules** | desktop + API (`09-api`); host wiring per ADR-009 |
| **Depends on** | P9-E2-T1 (Done) |
| **AC** | (1) Dedicated IPC `get_recommendations` (per ADR-009 / `09-api`) — host registers `register_recommendations_v1`, evaluate-on-read after Insights; empty → calm `[]`. (2) Dashboard calm surface (Insights-adjacent OK) showing title/suggestion/category/Evidence; no clinical chrome. (3) Mock/dev path for UI without rich history (e.g. `?mockRecommendations=…`). (4) Smoke notes in handoff; UI ↛ SQLite. (5) Handoff: `docs/handoffs/P9-E3-T1-dev-to-qa.md`. |
| **Out of scope** | New “Coach” product window redesign; push notifications; cloud sync; AI interpret layer changes (P11) |

---

## Phase 8 archive (Done)

<details>
<summary>Phase 8 Kanban & epics (closed 2026-08-08 — E1–E3)</summary>

**Done:** P8-E1 (T1) · P8-E2 (T1) · P8-E3 (T1).  
ADR-008 recompute-on-read → `focus_vs_recent_baseline_v1` → Dashboard Insights surface.

Evidence: `docs/handoffs/P8-*-qa-to-pm.md` · branch `phase/8-pattern-discovery` (cluster PR after freeze).

**P8-E1 shipped:** ADR-008 = recompute-on-read; no Feature-history table; Knowledge sketch. QA Pass 2026-08-07.  
**P8-E2 shipped:** baseline rule + `feature_engine::baseline` + `pattern_host` memo; Core `get_insights`. QA Pass with notes 2026-08-07.  
**P8-E3 shipped:** calm Dashboard list + `mockInsights=pattern`; category labels. QA Pass 2026-08-08.

</details>

---

## Phase 7 archive (Done)

<details>
<summary>Phase 7 Kanban & epics (closed 2026-08-06 — E1–E3)</summary>

**Done:** P7-E1 (T1) · P7-E2 (T1) · P7-E3 (T1).  
Feature confidence ADR-007 → Explanation factors (`FocusScore`) → `RecoveryScore` bio-backed Feature.

Evidence: `docs/handoffs/P7-*-qa-to-pm.md` · branch `phase/7-trust-layer` (cluster PR after freeze).

**P7-E3-T1 shipped:** `RecoveryScore` §1.7; `register_recovery_v1` in `register_catalog_v1`; confidence + optional factors; QA Pass 2026-08-06.

</details>

---

## Phase 6 archive (Done)

<details>
<summary>Phase 6 Kanban & epics (closed 2026-08-06 — E1–E3)</summary>

**Done:** P6-E1 (T1) · P6-E2 (T1) · P6-E3 (T1–T2).  
Life Events ADR-006 + quick-log → Calendar ICS Observations → `MeetingDensity` / `RecoveryBetweenMeetings`.

Evidence: `docs/handoffs/P6-*-qa-to-pm.md` · branch tip `phase/6-dogfood-fixes` (cluster PR after freeze).

</details>

---

## Phase 5 archive (Done)

<details>
<summary>Phase 5 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P5-E1 (T1–T2) · P5-E2 (T1) · P5-E3 (T1–T2).  
Opt-in LAN ingest (ADR-005) → Companion LAN UI → iOS HealthKit companion → dogfood runbook.

Evidence: `docs/handoffs/P5-*-qa-to-pm.md` · branch `phase/5-wearable-dogfood` (cluster PR after freeze).

</details>

---

## Phase 4 archive (Done)

<details>
<summary>Phase 4 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P4-E1 (T1–T3) · P4-E2 (T1–T3) · P4-E3 (T1–T3).  
Feature snapshot IPC → Dashboard + Recharts → Knowledge Insights → report-engine + optional local LLM + Report UX.

Evidence: `docs/handoffs/P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai` (cluster PR after freeze).

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

</details>

---

## Phase 1 archive (Done)

<details>
<summary>Phase 1 Kanban & epics (closed 2026-08-04)</summary>

**Done:** Epic E1–E4. Evidence: `docs/handoffs/P1-E4-T1-acceptance.md`, `P1-E4-T2-qa-to-pm.md`.

</details>

---

## Role × Module Matrix (Phase 9)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P9-E1-T1 ADR-009 Recommendations shape | | ● | ○ | | docs + decision-log |
| P9-E2-T1 Recommendation engine path | | ● | ○ | | knowledge-engine / per ADR |
| P9-E3-T1 Recommendations IPC / UX | | ● | ○ | ○ | desktop + API |

● = owner · ○ = collaborator

---

## Sprint 17–18 — Queue

1. ~~P9-E1-T1 — ADR-009 Recommendations domain / engine shape~~ ✅ Done  
2. ~~P9-E2-T1 — Recommendations v1 engine path~~ ✅ Done  
3. **P9-E3-T1 — Recommendations IPC / UX** ← **Ready**  

**Git:** `phase/9-recommendations` → local commits → **one cluster PR after 2026-09-01**.
