# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 7: Trust layer** (Sprint 13–14) — **Opened** 2026-08-06.  
> Phase 0–6 Done. Goal: **Feature confidence** + **Explanation factors** + a few bio-backed catalog Features — calm, evidence-first Trust layer. No parallel registries; no clinical claims.

**Phase 7 goal:** Features carry trustworthy confidence (and later factor breakdowns) so Dashboard / Insights can down-weight thin data; extend catalog only where Observation inputs already exist. No new SQLite tables without ADR + approve.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 analysis stack · horizon P8–P12+ — `/docs/00-vision.md`. **Do not** pull P8+ into this Kanban until PM opens that phase.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight`)
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); Features/Insights считаются в Core
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; poll/refresh по событию или редкому таймеру
- [ ] **Нет новой SQLite-схемы** без ADR + approve
- [ ] **Commit по связанному кластеру** (локально / feature-ветка) — `docs/12-development.md`  
- [ ] **PR freeze до 2026-09-01** — не открывать PR / не мержить в `main` через PR (`06-git-agent-policy.mdc`)
- [ ] Copy спокойный, неоценочный (не «ты выгорел» / clinical claims)
- [ ] **LAN / companion:** Bearer обязателен; нет cloud telemetry; default = loopback

---

## Kanban Overview (Phase 7 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P7-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **P7-E1-T1** · **P7-E2-T1** |

**Epic status:** P7-E1 ✅ · P7-E2 ✅ · P7-E3 ⬜ (T1 Ready)

**Phase 7 on `/docs/14-roadmap.md`:** opened 2026-08-06

**Рекомендуемый порядок:**  
~~P7-E1-T1~~ → ~~P7-E2-T1~~ → **P7-E3-T1**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).  
**Suggested branch:** `phase/7-trust-layer`.

### Active assignment

**Ready now:** **P7-E3-T1** — First bio-backed Trust Features (Dev).  
Brief: `docs/handoffs/P7-E3-T1-pm-brief.md`.

**Closed:** P7-E2-T1 (QA Pass, 2026-08-06) — `ExplanationFactor` + `FocusScore` factors on snapshot IPC; Epic **P7-E2** ✅. Evidence: `docs/handoffs/P7-E2-T1-qa-to-pm.md`. Branch: `phase/7-trust-layer`.

**Closed:** P7-E1-T1 (QA Pass, 2026-08-06) — ADR-007 Feature confidence + domain/IPC wire; Epic **P7-E1** ✅. Evidence: `docs/handoffs/P7-E1-T1-qa-to-pm.md`.

**Closed (Phase 6):** P6-E3-T2 (QA Pass, 2026-08-06) — `MeetingDensity` + `RecoveryBetweenMeetings`; Epic **P6-E3** ✅ · Phase 6 Kanban complete.

**Ops note:** **PR freeze until 2026-09-01** — keep work on `phase/7-trust-layer` (commits OK). Fold confidence + factors + E3 Features into one Phase 7 cluster PR **after** the freeze (or when user lifts it). Phase 4/5/6 cluster PRs deferred the same way.

---

## Epic P7-E1 — Feature confidence ✅ Done

**Цель:** Feature-level confidence (0–1) derived from input coverage / Observation confidence — not a parallel registry. ADR if domain/IPC shape grows; wire into `Feature` + snapshot consumers.

### P7-E1-T1 — Feature confidence contract + ADR + wire ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/bio-spec`, `crates/feature-engine`, `docs/decision-log.md`, `docs/02-domain-model.md`, `docs/07-contracts.md` / `docs/09-api.md` as needed |
| **Depends on** | Phase 6 Done (real Features + calendar/bio Observations); vision Trust layer |
| **AC** | (1) Record **ADR-007**: Feature-level confidence (vs Observation.confidence only); v1 formula strategy documented (coverage / mean Observation confidence / missing-input policy — exact rule in ADR + catalog note). (2) Domain + contracts: `Feature` (and snapshot IPC JSON) expose confidence in `[0.0, 1.0]` without breaking Ubiquitous Language; no new SQLite schema unless ADR + user approve. (3) At least Focus/Stress (or catalog_v1 path) **compute** confidence; empty/thin windows → low or omitted per ADR (explicit, tested). (4) Unit tests: full inputs → high confidence; missing HRV/context → lower; idle-safe. (5) Docs: domain model + catalog rule updated. (6) Handoff: `docs/handoffs/P7-E1-T1-dev-to-qa.md`. |
| **Out of scope** | Explanation factor breakdown (→ **E2-T1**), new bio Features (→ **E3**), Pattern Discovery, Dashboard redesign, Action/automation framework |
| **Shipped** | ADR-007 (`coverage × mean_obs`); `Feature.confidence` + snapshot IPC; all `register_catalog_v1` nodes; QA Pass 2026-08-06. Epic **P7-E1** closed. |

---

## Epic P7-E2 — Explanation factors ✅ Done

**Цель:** Calm “why this value” factor breakdown on Features (weights / contributions) — builds on provenance + confidence; no clinical tone.

### P7-E2-T1 — Explanation factors on Features ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/bio-spec` (if shape needs field), `crates/feature-engine`, contracts / API docs (`07` / `09`), catalog notes |
| **Depends on** | P7-E1-T1 |
| **AC** | (1) Document factor shape (id/label + contribution weight or share; calm non-clinical names) in domain/contracts/catalog — ADR only if IPC/domain boundary grows beyond additive optional field. (2) ≥1 catalog Feature (prefer `FocusScore` or `StressIndex`) **emits** factors alongside value + confidence + provenance. (3) Snapshot IPC exposes factors (or documented omit-until-present policy) without UI→DB. (4) Unit tests: factors sum/policy documented; empty/thin windows idle-safe. (5) Handoff: `docs/handoffs/P7-E2-T1-dev-to-qa.md`. |
| **Out of scope** | LLM-generated explanations; Pattern Discovery; new SQLite history store; full Dashboard “Why?” redesign (read-only display optional, not required) |
| **Shipped** | `ExplanationFactor { id, label, share }`; `FocusScore` factors (typing/stability/hrv); snapshot omit-until-present; QA Pass 2026-08-06. Epic **P7-E2** closed. |

---

## Epic P7-E3 — Bio-backed catalog Features

**Цель:** Ship a small set of catalog Features that already have Observation inputs (e.g. `RecoveryScore` / `DeepWorkScore` / `AttentionStability`) with confidence (and factors if E2 Done).

### P7-E3-T1 — First bio-backed Trust Features ✅ Ready
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/feature-engine`, `docs/06-feature-catalog.md` |
| **Depends on** | P7-E1-T1 (confidence); P7-E2-T1 (factors helpful) |
| **AC** | (1) Move ≥1 Feature from planned backlog → catalog §1 with formula / window / units / deps / provenance (+ ADR-007 confidence). Prefer **`RecoveryScore`** (HRV/HR) or **`DeepWorkScore`** / **`AttentionStability`** (Focus+CSR) — pick one with real Observation inputs today. (2) Register in `register_catalog_v1` (or focused register helper wired into catalog_v1). (3) Unit tests with synthetic Observations (empty / rich / thin → confidence). (4) Calm non-clinical copy; idle-safe. (5) Optional: emit factors for the new Feature if formula has clear weighted components (reuse E2 shape). (6) Handoff: `docs/handoffs/P7-E3-T1-dev-to-qa.md`. |
| **Out of scope** | CognitiveLoad (needs richer schedule+notify), SleepDebt without sleep Observations, CircadianOffset (P8), Dashboard redesign, Pattern Discovery |

---

## Phase 6 archive (Done)

<details>
<summary>Phase 6 Kanban & epics (closed 2026-08-06 — E1–E3)</summary>

**Done:** P6-E1 (T1) · P6-E2 (T1) · P6-E3 (T1–T2).  
Life Events ADR-006 + quick-log → Calendar ICS Observations → `MeetingDensity` / `RecoveryBetweenMeetings`.

Evidence: `docs/handoffs/P6-*-qa-to-pm.md` · branch tip `phase/6-dogfood-fixes` (cluster PR when ready).

**P6-E3-T2 shipped:** `MeetingDensityNode` / `RecoveryBetweenMeetingsNode`; `register_calendar_v1` in `register_catalog_v1`; catalog §1.5 / §1.6; QA Pass 2026-08-06.

</details>

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

## Role × Module Matrix (Phase 7)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P7-E1-T1 Feature confidence + ADR | | ● | ○ | | bio-spec / feature-engine + docs |
| P7-E2-T1 Explanation factors | | ● | ○ | | feature-engine + contracts |
| P7-E3-T1 Bio-backed catalog Features | | ● | ○ | | feature-engine + catalog |

● = owner · ○ = collaborator

---

## Sprint 13–14 — Queue

1. ~~P7-E1-T1 — Feature confidence contract + ADR + wire~~ **Done** (QA Pass) · Epic **P7-E1** ✅  
2. ~~P7-E2-T1 — Explanation factors on Features~~ **Done** (QA Pass) · Epic **P7-E2** ✅  
3. **P7-E3-T1 — First bio-backed Trust Features** ← **Ready**  

**Git:** `phase/7-trust-layer` → related commits locally → **one cluster PR after 2026-09-01** (PR freeze; see `06-git-agent-policy.mdc`).
