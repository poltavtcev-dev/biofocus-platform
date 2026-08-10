# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 11: AI coaching polish** (Sprint 21–22) — **Opened** 2026-08-10.  
> Phase 0–10 Done. Goal: polish **L5 Coaching (AI)** — versioned **prompt packs** + calmer **provider UX** — while LLM stays **interpret-only** (never computes Features / Recommendations / Evidence).

**Phase 11 goal:** Turn the Phase 4 `report-engine` + opt-in local LLM path into a dogfood-ready coaching polish: named prompt packs that wrap already-computed Evidence (Features / Insights / Recommendations), plus a calm Dashboard provider surface — still local-first, explicit user action, no auto-send.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 · horizon P12+ — `/docs/00-vision.md`.

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

---

## Kanban Overview (Phase 11 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P11-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **Phase 9** (E1–E3) · **Phase 10** (E1–E3) · **P11-E1-T1** · **P11-E2-T1** |

**Epic status:** P11-E1 ✅ · P11-E2 ✅ · P11-E3 ⬜ (T1 Ready)

**Phase 11 on `/docs/14-roadmap.md`:** opened 2026-08-10 · ADR-011 + prompt packs shipped

**Рекомендуемый порядок:**  
~~P11-E1-T1~~ → ~~P11-E2-T1~~ → **P11-E3-T1**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).

### Active assignment

**Ready now:** **P11-E3-T1** — Local LLM provider UX + pack-aware Report flow. Brief: `docs/handoffs/P11-E3-T1-pm-brief.md`. Role: **UX + Dev**.

**Closed:** P11-E2-T1 (QA Pass, 2026-08-10) — `build_report_with_pack` + `biofocus.default` @ `1`; Epic **P11-E2** ✅. Evidence: `docs/handoffs/P11-E2-T1-qa-to-pm.md`.

**Closed:** P11-E1-T1 (QA Pass, 2026-08-10) — ADR-011; Epic **P11-E1** ✅. Evidence: `docs/handoffs/P11-E1-T1-qa-to-pm.md`.

**Next after Pass:** close Epic **P11-E3** + **Phase 11** Kanban (then Phase 12+ via separate PM gate).

**Closed previous:** Phase 10 (E1–E3 Done, 2026-08-10) — ADR-010 Browser → `BrowserCategoryPlugin` → `DistractionScore`. Evidence: `docs/handoffs/P10-*-qa-to-pm.md`. Branch: `phase/10-plugin-wave-1`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 11 cluster on `phase/11-ai-coaching-polish`; local commits OK; one cluster PR **after** freeze (or when user lifts it). E3 wires host to `build_report_with_pack` (default pack); LLM still env opt-in.

---

## Phase 11 — Epics & Tasks (Sprint 21–22)

### Epic P11-E1 — ADR-011 Coaching polish scope
**Goal:** Decide how prompt packs and provider UX evolve L5 without breaking interpret-only / Local-First.

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P11-E1-T1** | ADR-011: prompt packs + provider UX boundaries | Dev | docs + decision-log | ADR-011 recorded | Phase 10 Done |

**P11-E1 shipped:** ADR-011 = named/versioned prompt packs in `report-engine` + calm Dashboard provider UX; interpret-only; no chat SQLite; no Coach Engine. QA Pass 2026-08-10.

**Out of scope (E1):** implementing packs code (→ E2); Dashboard provider UI (→ E3); ambient plugins / commercial packaging (Phase 12+); opening a PR during freeze.

---

### Epic P11-E2 — Prompt packs (`report-engine`)
**Goal:** Ship ≥1 versioned prompt pack that wraps deterministic report facts for interpret-only LLM use.

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P11-E2-T1** | Versioned prompt packs in report-engine | Dev | report-engine (+ bio-spec types as needed) | `build_report_with_pack` + default pack | P11-E1-T1 |

**P11-E2 shipped:** `build_report_with_pack(id, version, …)`; default `biofocus.default` @ `1`; empty/partial soft Ok; no SQLite/network/UI. QA Pass 2026-08-10.

**Out of scope (E2):** in-app provider toggle / Dashboard UX (→ E3); cloud providers; auto-interpret; on-disk user pack overrides (future ADR).

---

### Epic P11-E3 — Provider UX (Dashboard)
**Goal:** Calm Dashboard surface so users understand local LLM opt-in status and can generate/interpret without env archaeology — still explicit action only.

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P11-E3-T1** | Local LLM provider UX + pack-aware Report flow | UX + Dev | apps/desktop (+ src-tauri IPC as needed) | See below | P11-E2-T1 |

**P11-E3-T1 AC:**
1. Dashboard shows calm **provider status** for local LLM (e.g. disabled / ready / error) via IPC — UI ↛ SQLite; no secrets in UI logs; status reflects host env/config (Phase 4 `BIOFOCUS_LOCAL_LLM` stance).
2. Report / coaching flow uses **`build_report_with_pack`** (default `biofocus.default` @ `1`, or documented pack pick) behind **explicit** user action; never auto-invoke on open/poll; include Recommendations Evidence when available.
3. When LLM off: offline markdown + prompt still available; calm copy that local AI is optional.
4. When LLM on (host config): optional interpretation soft-fails without losing markdown (`llmStatus` pattern from Phase 4).
5. Copy non-clinical; no employee-surveillance / diagnosis framing; no cloud marketplace.
6. Mock/dev path documented for browser QA if needed; handoff with smoke steps: `docs/handoffs/P11-E3-T1-dev-to-qa.md`.

**Out of scope (E3):** cloud LLM marketplace; chat history store; ambient Phase 12+; on-disk pack overrides; PR during freeze.

---

## Phase 10 archive (Done)

<details>
<summary>Phase 10 Kanban & epics (closed 2026-08-10 — E1–E3)</summary>

**Done:** P10-E1 (T1) · P10-E2 (T1) · P10-E3 (T1).  
ADR-010 Browser categories → `BrowserCategoryPlugin` → `DistractionScore`.

Evidence: `docs/handoffs/P10-*-qa-to-pm.md` · branch `phase/10-plugin-wave-1` (cluster PR after freeze).

**P10-E1 shipped:** ADR-010 = Browser categories; `browser_category` contract; no schema migration. QA Pass 2026-08-10.  
**P10-E2 shipped:** `BrowserCategoryPlugin` + `BIOFOCUS_BROWSER_CATEGORIES`; emit→persist; idle-safe. QA Pass 2026-08-10.  
**P10-E3 shipped:** `DistractionScore` §1.8; `register_distraction_v1`; pipeline normalize; omit empty/unknown-only; explanation factors. QA Pass 2026-08-10.

</details>

---

## Phase 9 archive (Done)

<details>
<summary>Phase 9 Kanban & epics (closed 2026-08-08 — E1–E3)</summary>

**Done:** P9-E1 (T1) · P9-E2 (T1) · P9-E3 (T1).  
ADR-009 → `focus_dip_pace_hint_v1` → `get_recommendations` + Suggestions UX.

Evidence: `docs/handoffs/P9-*-qa-to-pm.md` · branch `phase/9-recommendations` (cluster PR after freeze).

</details>

---

## Phase 8 archive (Done)

<details>
<summary>Phase 8 Kanban & epics (closed 2026-08-08 — E1–E3)</summary>

**Done:** P8-E1 (T1) · P8-E2 (T1) · P8-E3 (T1).  
ADR-008 recompute-on-read → `focus_vs_recent_baseline_v1` → Dashboard Insights surface.

Evidence: `docs/handoffs/P8-*-qa-to-pm.md` · branch `phase/8-pattern-discovery` (cluster PR after freeze).

</details>

---

## Phase 7 archive (Done)

<details>
<summary>Phase 7 Kanban & epics (closed 2026-08-06 — E1–E3)</summary>

**Done:** P7-E1 (T1) · P7-E2 (T1) · P7-E3 (T1).  
Feature confidence ADR-007 → Explanation factors (`FocusScore`) → `RecoveryScore` bio-backed Feature.

Evidence: `docs/handoffs/P7-*-qa-to-pm.md` · branch `phase/7-trust-layer` (cluster PR after freeze).

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

**Done:** P5-E1 (T1–T2) · P5-E2 (T1) · P5-E3 (T1). Wearable dogfood. Evidence: `docs/handoffs/P5-*-qa-to-pm.md`.

</details>

---

## Phase 4 archive (Done)

<details>
<summary>Phase 4 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** Epic E1–E3. Evidence: `docs/handoffs/P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai`.

</details>

---

## Phase 3 archive (Done)

<details>
<summary>Phase 3 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** Epic E1–E3. Evidence: `docs/handoffs/P3-*-qa-to-pm.md`.

</details>

---

## Phase 2 archive (Done)

<details>
<summary>Phase 2 Kanban (merged to main — PR #2)</summary>

**Done on `main`:** E0–E3 via [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2).

</details>

---

## Phase 1 archive (Done)

<details>
<summary>Phase 1 Kanban & epics (closed 2026-08-04)</summary>

**Done:** Epic E1–E4. Evidence: `docs/handoffs/P1-E4-T1-acceptance.md`, `P1-E4-T2-qa-to-pm.md`.

</details>

---

## Role × Module Matrix (Phase 11)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P11-E1-T1 ADR-011 prompt packs + provider UX | | ● | ○ | | docs + decision-log |
| P11-E2-T1 Versioned prompt packs | | ● | ○ | | report-engine |
| P11-E3-T1 Provider UX + pack-aware Report | | ● | ○ | ● | apps/desktop + IPC |

● = owner · ○ = collaborator

---

## Sprint 21–22 — Queue

1. ~~P11-E1-T1 — ADR-011 AI coaching polish (prompt packs + provider UX)~~ ✅ Done  
2. ~~P11-E2-T1 — Versioned prompt packs in `report-engine`~~ ✅ Done  
3. **P11-E3-T1** — Local LLM provider UX + pack-aware Report flow ← **Ready**

**Git:** `phase/11-ai-coaching-polish` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P11-E3-T1-pm-brief.md`.  
**Next after Phase 11:** PM gate → Phase 12+ ambient + packaging (`docs/14-roadmap.md`).
