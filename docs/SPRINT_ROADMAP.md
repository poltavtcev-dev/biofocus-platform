# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 8: Pattern Discovery v1** (Sprint 15–16) — **Opened** 2026-08-06.  
> Phase 0–7 Done. Goal: multi-day / baseline **Knowledge** (personal patterns) — ADR for recompute vs Feature history first; calm Insights with Evidence. No clinical claims. No new SQLite schema without ADR + approve.

**Phase 8 goal:** User-visible personal patterns over days (e.g. “Focus tends to be higher after recovery windows”) via Knowledge layer — not a new ML crate. Decide persistence/recompute in ADR-008 before shipping history tables or recompute jobs.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 · horizon P9–P12+ — `/docs/00-vision.md`. **Do not** pull P9+ into this Kanban until PM opens that phase.

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

## Kanban Overview (Phase 8 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P8-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **P8-E1-T1** |

**Epic status:** P8-E1 ✅ · P8-E2 ⬜ (T1 Ready) · P8-E3 ⬜

**Phase 8 on `/docs/14-roadmap.md`:** opened 2026-08-06

**Рекомендуемый порядок:**  
~~P8-E1-T1~~ → **P8-E2-T1** → P8-E3-T1

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).  
**Suggested branch:** `phase/8-pattern-discovery` (or continue `phase/7-trust-layer` tip until freeze ends / user asks for branch cut).

### Active assignment

**Ready now:** **P8-E2-T1** — Pattern Discovery v1 Insight path (Dev).  
Brief: `docs/handoffs/P8-E2-T1-pm-brief.md`.

**Closed:** P8-E1-T1 (QA Pass, 2026-08-07) — ADR-008 **recompute-on-read**; no Feature-history schema; Epic **P8-E1** ✅. Evidence: `docs/handoffs/P8-E1-T1-qa-to-pm.md`. No schema approve wait.

**Closed (Phase 7):** P7-E3-T1 (QA Pass, 2026-08-06) — `RecoveryScore` + confidence + factors; Epic **P7-E3** ✅ · Phase 7 Kanban complete. Evidence: `docs/handoffs/P7-E3-T1-qa-to-pm.md`. Branch: `phase/7-trust-layer`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 8 docs/code on `phase/8-pattern-discovery` (or tip); local commits OK; one cluster PR **after** freeze (or when user lifts it).

---

## Epic P8-E1 — Pattern Discovery ADR & storage contract ✅

**Цель:** Decide how multi-day baselines / Feature history work — ADR before schema or background recompute.

### P8-E1-T1 — ADR-008: recompute vs Feature history ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `docs/decision-log.md`, `docs/04-storage.md` / `02-domain-model` / `05-pipeline` notes as needed; **no** schema migration without user approve after ADR |
| **Depends on** | Phase 7 Done (trusted Features + confidence) |
| **AC** | (1) Record **ADR-008**: Pattern Discovery v1 approach — evaluate-on-read recompute over Observations/Features **vs** persisted Feature/baseline history (or hybrid); rejected alternatives; idle/privacy constraints. (2) If schema needed: propose tables/columns in ADR + docs — **do not apply migration** until user approve. (3) Document how Knowledge Insights would consume the chosen approach (contracts sketch OK). (4) Calm non-clinical framing. (5) Handoff: `docs/handoffs/P8-E1-T1-dev-to-qa.md`. |
| **Out of scope** | Implementing history tables / recompute worker (→ **E2** after approve); Recommendations engine (P9); ML training |
| **Shipped** | ADR-008 = **recompute-on-read** (+ optional in-process memo); no Feature/baseline history table; Knowledge sketch `focus_vs_recent_baseline_v1`. QA Pass 2026-08-07. |

---

## Epic P8-E2 — Baseline / multi-day Knowledge path

**Цель:** First evaluate-on-read baseline Insights using ADR-008 (**recompute-on-read**; no new SQLite schema).

### P8-E2-T1 — Pattern Discovery v1 Insight path ✅ Ready
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/knowledge-engine`, contracts / API as needed; Observation load / Feature recompute helpers as required by ADR-008 |
| **Depends on** | P8-E1-T1 Done (no schema approve — ADR chose recompute-on-read) |
| **AC** | ≥1 Insight rule or evaluator that uses multi-window / baseline comparison with Evidence (prefer `focus_vs_recent_baseline_v1` per ADR-008); tests; idle-safe; calm copy. Handoff required. |
| **Out of scope** | LLM pattern generation; CircadianOffset Feature; Recommendations; Feature-history SQLite table |

---

## Epic P8-E3 — Surface patterns (IPC / Dashboard read)

**Цель:** Expose Pattern Discovery Insights via existing Insights IPC (and optional calm Dashboard copy) without UI→DB.

### P8-E3-T1 — Insights IPC / UX for patterns
| Field | Value |
| :--- | :--- |
| **Role** | Dev (+ UX if UI copy) |
| **Modules** | desktop Insights path / `09-api` |
| **Depends on** | P8-E2-T1 |
| **AC** | Patterns visible via existing Insights IPC path; calm copy; smoke notes in handoff. |
| **Out of scope** | New “Pattern Discovery” product window redesign; cloud sync |

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

## Role × Module Matrix (Phase 8)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P8-E1-T1 ADR-008 history/recompute | | ● | ○ | | docs + decision-log |
| P8-E2-T1 Pattern Insight path | | ● | ○ | | knowledge-engine |
| P8-E3-T1 Insights IPC / UX | | ● | ○ | ○ | desktop + API |

● = owner · ○ = collaborator

---

## Sprint 15–16 — Queue

1. ~~P8-E1-T1 — ADR-008 Pattern Discovery history / recompute~~ ✅ Done  
2. **P8-E2-T1 — Pattern Discovery v1 Insight path** ← **Ready**  
3. P8-E3-T1 — Insights IPC / UX for patterns  

**Git:** `phase/8-pattern-discovery` (or `phase/7-trust-layer` tip) → local commits → **one cluster PR after 2026-09-01**.
