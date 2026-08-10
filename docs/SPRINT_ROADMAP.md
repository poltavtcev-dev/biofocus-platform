# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 10: Plugin wave-1** (Sprint 19–20) — **Opened** 2026-08-10.  
> Phase 0–9 Done. Goal: dogfood **Browser categories** plugin (`browser_category`) + catalog **`DistractionScore`** (ADR-010). IDE/Git deferred.

**Phase 10 goal:** ADR-010 chose **Browser categories** → opt-in `browser_category` Observations → catalog **`DistractionScore`**. Calm, non-surveillance, opt-in. IDE/Git deferred.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 · horizon P11–P12+ — `/docs/00-vision.md`.

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

---

## Kanban Overview (Phase 10 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P10-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **Phase 9** (E1–E3) · **P10-E1-T1** · **P10-E2-T1** |

**Epic status:** P10-E1 ✅ · P10-E2 ✅ · P10-E3 ⬜ (T1 Ready)

**Phase 10 on `/docs/14-roadmap.md`:** opened 2026-08-10 · ADR-010 + Browser collector shipped

**Рекомендуемый порядок:**  
~~P10-E1-T1~~ → ~~P10-E2-T1~~ → **P10-E3-T1**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).

### Active assignment

**Ready now:** **P10-E3-T1** — Catalog Feature `DistractionScore`. Brief: `docs/handoffs/P10-E3-T1-pm-brief.md`. Role: **Dev**.

**Closed:** P10-E2-T1 (QA Pass, 2026-08-10) — `BrowserCategoryPlugin` + opt-in `BIOFOCUS_BROWSER_CATEGORIES`; Epic **P10-E2** ✅. Evidence: `docs/handoffs/P10-E2-T1-qa-to-pm.md`.

**Closed:** P10-E1-T1 (QA Pass, 2026-08-10) — ADR-010 Browser categories; Epic **P10-E1** ✅. Evidence: `docs/handoffs/P10-E1-T1-qa-to-pm.md`.

**Next after Pass:** close Epic **P10-E3** + **Phase 10** Kanban (then Phase 11 via separate PM gate).

**Ops note:** **PR freeze until 2026-09-01** — Phase 10 cluster on `phase/10-plugin-wave-1`; local commits OK; one cluster PR **after** freeze (or when user lifts it). Dogfood note: live OS probe often emits `category: "unknown"` until richer non-persisting mapping exists — Feature must tolerate thin/`unknown` windows.

---

## Phase 10 — Epics & Tasks (Sprint 19–20)

### Epic P10-E1 — Wave-1 ADR (source + contract)
**Goal:** Decide IDE/Git **vs** Browser categories for dogfood wave-1; lock privacy-safe Observation contract before code.

| ID | Task | Role | Modules | Depends | AC (summary) | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **P10-E1-T1** | ADR-010: Plugin wave-1 source + Observation contract | Dev | docs + decision-log | Phase 9 Done | ADR-010 Browser + contract | **Done** (2026-08-10) |

**P10-E1 shipped:** ADR-010 = **Browser categories**; `browser_category` contract; no schema migration; E3 = `DistractionScore`. IDE/Git deferred. QA Pass 2026-08-10.

---

### Epic P10-E2 — Browser categories collector plugin
**Goal:** Opt-in `BioFocusPlugin` emits `browser_category` Observations into the Desktop ingest channel (no UI→SQLite).

| ID | Task | Role | Modules | Depends | AC (summary) | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **P10-E2-T1** | Implement Browser categories plugin (ADR-010) | Dev | plugin-sdk + macos-collector (or adapter) + desktop host wire | P10-E1-T1 | Plugin + opt-in + tests | **Done** (2026-08-10) |

**P10-E2 shipped:** `BrowserCategoryPlugin` (`com.biofocus.macos.browser`); `BIOFOCUS_BROWSER_CATEGORIES=1`; emit→persist; idle-safe; docs 07/08/10/12. QA Pass 2026-08-10.

---

### Epic P10-E3 — DistractionScore catalog Feature
**Goal:** Ship **`DistractionScore`** that requires `browser_category` Observations (vision rule: Features only with real inputs).

| ID | Task | Role | Modules | Depends | AC (summary) | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **P10-E3-T1** | Catalog Feature `DistractionScore` | Dev | feature-engine + pipeline normalize + catalog docs | P10-E2-T1 | See AC below | **Ready** |

**AC — P10-E3-T1:**
1. Move **`DistractionScore`** from `docs/06-feature-catalog.md` § Planned → §1 with formula / window / step / units / inputs / provenance + ADR-007 confidence. Inputs: `browser_category` Observations; optional combine with `ContextSwitchRate` if it improves the calm fragmentation signal (document either way).
2. Pipeline: normalize `browser_category` (aliases / strip forbidden keys `url`/`title`/`href`/content if present) when needed — collector already clean, but Core must not trust UI/ingest extras.
3. Register node in `register_catalog_v1` (or helper wired into it); Feature appears on existing snapshot path when inputs present.
4. Empty / only-`unknown` / thin windows → omit Feature or emit with clearly lower confidence (document policy); no busy-loop; no new SQLite.
5. Unit tests: synthetic rich categories → emit; empty/`unknown`-only → omit or low confidence per policy; calm non-clinical framing (not ADHD / “you are distracted” diagnosis).
6. Optional: `ExplanationFactor`s if weighted components are clear (reuse P7-E2 shape).
7. Handoff: `docs/handoffs/P10-E3-T1-dev-to-qa.md`.

**Out of Phase 10:** NotificationPressure (unless ADR folds a thin path); ambient music/weather/light (P12+); AI coaching polish (P11); second wave-1 source (defer to later sprint); PR before 2026-09-01.

---

## Phase 9 archive (Done)

<details>
<summary>Phase 9 Kanban & epics (closed 2026-08-08 — E1–E3)</summary>

**Done:** P9-E1 (T1) · P9-E2 (T1) · P9-E3 (T1).  
ADR-009 → `focus_dip_pace_hint_v1` → `get_recommendations` + Suggestions UX.

Evidence: `docs/handoffs/P9-*-qa-to-pm.md` · branch `phase/9-recommendations` (cluster PR after freeze).

**P9-E1 shipped:** ADR-009 = first-class `Recommendation` + `RecommendationRule` in `knowledge-engine`; evaluate-on-read; no Recommendation SQLite. QA Pass 2026-08-08.  
**P9-E2 shipped:** `EvidenceRef::Insight` + `focus_dip_pace_hint_v1` + `evaluate_recommendations`. QA Pass 2026-08-08.  
**P9-E3 shipped:** IPC `get_recommendations` + Dashboard Suggestions + `mockRecommendations`. QA Pass 2026-08-08.

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

## Role × Module Matrix (Phase 10)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P10-E1-T1 ADR-010 wave-1 source + contract | | ● | ○ | | docs + decision-log |
| P10-E2-T1 Browser categories collector plugin | | ● | ○ | | plugin-sdk + macos-collector + desktop |
| P10-E3-T1 DistractionScore catalog Feature | | ● | ○ | | feature-engine + pipeline |

● = owner · ○ = collaborator

---

## Sprint 19–20 — Queue

1. ~~P10-E1-T1 — ADR-010 Plugin wave-1 source + Observation contract~~ ✅ Done  
2. ~~P10-E2-T1 — Browser categories collector plugin~~ ✅ Done  
3. **P10-E3-T1** — `DistractionScore` catalog Feature ← **Ready**

**Git:** `phase/10-plugin-wave-1` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P10-E3-T1-pm-brief.md`.
