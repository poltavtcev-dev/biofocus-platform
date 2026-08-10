# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 13: Plugin wave-2 (Git activity)** (Sprint 25–26) — **Opened** 2026-08-10.  
> Phase 0–12 Done. Goal: ADR-013 → Git activity Observations → `GitActivityRate` — Local-First, opt-in. IDE deferred (no additive privacy-safe signal beyond `context_window`).

**Phase 13 goal:** Dogfood **plugin wave-2** — **Git activity aggregates** (ADR-013) via Capability Plugin Model, then catalog Feature **`GitActivityRate`**. IDE, weather/light, and App Store packaging product remain deferred.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 · Phase 13 finishes plugin ladder after Browser — `/docs/00-vision.md`.

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

---

## Kanban Overview (Phase 13 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P13-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **Phase 9** (E1–E3) · **Phase 10** (E1–E3) · **Phase 11** (E1–E3) · **Phase 12** (E1–E3) · **PM-GATE-POST-P12** · **P13-E1-T1** · **P13-E2-T1** |

**Epic status:** P13-E1 ✅ · P13-E2 ✅ · P13-E3 ⬜ (T1 Ready)

**Phase 13 on `/docs/14-roadmap.md`:** opened 2026-08-10 · ADR-013 + Git plugin shipped · Ready `GitActivityRate`

**Рекомендуемый порядок:**  
~~P13-E1-T1~~ → ~~P13-E2-T1~~ → **P13-E3-T1**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).

### Active assignment

**Ready now:** **P13-E3-T1** — Catalog Feature `GitActivityRate`. Brief: `docs/handoffs/P13-E3-T1-pm-brief.md`. Role: **Dev**.

**Closed:** P13-E2-T1 (QA Pass, 2026-08-10) — `GitActivityPlugin` / `BIOFOCUS_GIT_ACTIVITY` / `validate_git_activity_payload`; production OS probe soft-fails idle without path-allowlist ADR (by design); scripted probe covers emit→persist. Epic **P13-E2** ✅. Evidence: `docs/handoffs/P13-E2-T1-qa-to-pm.md`.

**Closed:** P13-E1-T1 (QA Pass, 2026-08-10) — ADR-013: wave-2 = **Git activity aggregates**; E3 Feature = **`GitActivityRate`**; no schema. Epic **P13-E1** ✅. Evidence: `docs/handoffs/P13-E1-T1-qa-to-pm.md`.

**Closed gate:** PM-GATE-POST-P12 (2026-08-10) — chose plugin wave-2 over weather/light and App Store packaging. Evidence: `docs/handoffs/PM-GATE-POST-P12-pm-brief.md`.

**Next after Pass:** close Epic **P13-E3** + **Phase 13** Kanban (then next horizon via separate PM gate).

**Ops note:** **PR freeze until 2026-09-01** — Phase 13 cluster on `phase/13-plugin-wave-2`; local commits OK; one cluster PR **after** freeze. Public surface: `GitActivityPlugin`, `BIOFOCUS_GIT_ACTIVITY`, `validate_git_activity_payload`, ingest/normalize `git_activity`.

---

## Phase 13 — Epics & Tasks (Sprint 25–26)

### Epic P13-E1 — ADR-013 Plugin wave-2 scope
**Goal:** Decide IDE vs Git (exactly one v1 primary) + Observation contract + E2/E3 sketch without breaking Local-First / Capability Model / privacy bar from ADR-010.

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P13-E1-T1** | ADR-013: IDE/Git wave-2 boundaries | Dev | docs + decision-log | ADR-013 recorded | Phase 12 Done · PM-GATE-POST-P12 |

**P13-E1 shipped:** ADR-013 = wave-2 primary **Git activity aggregates** (`git_activity`); IDE deferred (no additive privacy-safe signal beyond `context_window`); E2 plugin → E3 **`GitActivityRate`**; existing `observations` only; no migration. QA Pass 2026-08-10.

**Out of scope (E1):** implementing collector / Feature / Git path indexing; weather/light; App Store product; NotificationPressure unless ADR notes deferral; PR during freeze.

---

### Epic P13-E2 — Git activity collector plugin
**Goal:** Ship opt-in Git activity `BioFocusPlugin` → Observation channel → persist (ADR-013).

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P13-E2-T1** | Git activity plugin | Dev | macos-collector / plugin-sdk + host | See `P13-E2-T1-pm-brief.md` | P13-E1-T1 |

**P13-E2 shipped:** `GitActivityPlugin` (`com.biofocus.macos.git`); opt-in `BIOFOCUS_GIT_ACTIVITY`; `validate_git_activity_payload`; pipeline normalize strips forbidden keys; idle-safe + stop joins; production OS probe soft-fails idle without path-allowlist ADR (by design) — scripted probe covers emit→persist. QA Pass 2026-08-10.

**Out of scope (E2):** `GitActivityRate` / Feature DAG (→ **P13-E3**); IDE collector; new SQLite schema / path allowlist table; PR during freeze.

---

### Epic P13-E3 — GitActivityRate catalog Feature
**Goal:** Catalog Feature **`GitActivityRate`** from `git_activity` Observations + pipeline normalize (ADR-013).

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P13-E3-T1** | `GitActivityRate` catalog Feature | Dev | feature-engine + pipeline + docs | See `P13-E3-T1-pm-brief.md` | P13-E2-T1 |

**Out of scope (E3):** Dashboard redesign; Insights/Recommendations for GitActivityRate unless separately scoped; workplace surveillance framing; path-allowlist ADR; PR during freeze.

---

## Phase 12 archive (Done)

<details>
<summary>Phase 12 Kanban & epics (closed 2026-08-10 — E1–E3)</summary>

**Done:** P12-E1 (T1) · P12-E2 (T1) · P12-E3 (T1).  
ADR-012 Now Playing ambient → `NowPlayingPlugin` → `AmbientMediaShare` + packaging runbook.

Evidence: `docs/handoffs/P12-*-qa-to-pm.md` · branch `phase/12-ambient-packaging` (cluster PR after freeze).

### Epic P12-E1 — ADR-012 Phase 12 scope
**P12-E1 shipped:** ADR-012 = primary **Now Playing / music** ambient (`now_playing`); secondary packaging as signed-build / notarization / update **runbook**; E2 plugin → E3 `AmbientMediaShare` + runbook; existing `observations` only; no migration. QA Pass 2026-08-10.

### Epic P12-E2 — Now Playing ambient plugin
**P12-E2 shipped:** `NowPlayingPlugin` (`com.biofocus.macos.now_playing`); opt-in `BIOFOCUS_NOW_PLAYING`; `validate_now_playing_payload`; ingest `invalid_now_playing`; idle-safe + stop joins; production OS probe soft-fails (`None`) by design — scripted probe covers emit→persist. QA Pass with notes 2026-08-10.

### Epic P12-E3 — AmbientMediaShare + packaging runbook
**P12-E3 shipped:** Catalog §1.9 `AmbientMediaShare` via `register_ambient_v1`; share 0–100 from `now_playing`; **omit** empty / only-`none` / only-`unknown`; ADR-007 confidence + kind factors; pipeline strips forbidden content keys; `docs/18-packaging-runbook.md` (signed `.app`/`.dmg`, notarization, update stance; sync off by default; AGPLv3 Core open). QA Pass with notes 2026-08-10.

**Out of scope (Phase 12):** weather/light; App Store product; sync product; live MediaRemote content mapping; IDE/Git wave; workplace surveillance framing; PR during freeze.

</details>

---

## Phase 11 archive (Done)

<details>
<summary>Phase 11 Kanban & epics (closed 2026-08-10 — E1–E3)</summary>

**Done:** P11-E1 (T1) · P11-E2 (T1) · P11-E3 (T1).  
ADR-011 → `build_report_with_pack` / `biofocus.default` @ `1` → `get_local_llm_status` + pack-aware Report UX.

Evidence: `docs/handoffs/P11-*-qa-to-pm.md` · branch `phase/11-ai-coaching-polish` (cluster PR after freeze).

### Epic P11-E1 — ADR-011 Coaching polish scope
**P11-E1 shipped:** ADR-011 = named/versioned prompt packs in `report-engine` + calm Dashboard provider UX; interpret-only; no chat SQLite; no Coach Engine. QA Pass 2026-08-10.

### Epic P11-E2 — Prompt packs (`report-engine`)
**P11-E2 shipped:** `build_report_with_pack(id, version, …)`; default `biofocus.default` @ `1`; empty/partial soft Ok; no SQLite/network/UI. QA Pass 2026-08-10.

### Epic P11-E3 — Provider UX (Dashboard)
**P11-E3 shipped:** Calm Local AI status via `get_local_llm_status` (`disabled`/`ready`/`error`); `generate_report` uses `build_report_with_pack` + Recommendations Evidence; explicit Generate only; soft-fail retains markdown. QA Pass with notes 2026-08-10 (live Ollama smoke optional / deferred).

**Out of scope (Phase 11):** cloud LLM marketplace; chat history store; ambient plugins; commercial packaging; on-disk pack overrides; PR during freeze.

</details>

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

## Role × Module Matrix (Phase 13)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P13-E1-T1 ADR-013 Git wave-2 scope | | ● | ○ | | docs + decision-log |
| P13-E2-T1 Git activity plugin | | ● | ○ | | macos-collector / plugin-sdk + host |
| P13-E3-T1 GitActivityRate catalog Feature | | ● | ○ | | feature-engine + pipeline + docs |

● = owner · ○ = collaborator

---

## Sprint 25–26 — Queue

1. ~~P13-E1-T1 — ADR-013 Plugin wave-2 (Git)~~ ✅ Done  
2. ~~P13-E2-T1 — Git activity plugin~~ ✅ Done  
3. **P13-E3-T1** — `GitActivityRate` catalog Feature ← **Ready**

**Git:** `phase/13-plugin-wave-2` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P13-E3-T1-pm-brief.md`.  
**Deferred (not Phase 13):** IDE collector · weather/light ambient · App Store packaging product · NotificationPressure · path-allowlist table.
