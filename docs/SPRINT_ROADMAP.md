# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 12: Ambient + commercial packaging** (Sprint 23–24) — **Opened** 2026-08-10.  
> Phase 0–11 Done. Goal: Now Playing ambient Observations → `AmbientMediaShare` + packaging runbook — Local-First, opt-in — locked by ADR-012.

**Phase 12 goal:** Dogfood **Now Playing** ambient (`now_playing` Observations) via Capability Plugin Model, then `AmbientMediaShare` Feature + commercial packaging runbook (signed builds / notarization / update stance) — algorithms remain open-source; sync off by default.

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
- [ ] **Ambient / packaging:** opt-in ambient capture; no always-on mic/geo dumps; commercial packaging ≠ closed Feature math; optional sync off by default

---

## Kanban Overview (Phase 12 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P12-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **Phase 9** (E1–E3) · **Phase 10** (E1–E3) · **Phase 11** (E1–E3) · **P12-E1-T1** |

**Epic status:** P12-E1 ✅ · P12-E2 ⬜ (T1 Ready) · P12-E3 ⬜

**Phase 12 on `/docs/14-roadmap.md`:** opened 2026-08-10 · ADR-012 shipped · Ready Now Playing plugin

**Рекомендуемый порядок:**  
~~P12-E1-T1~~ → **P12-E2-T1** → P12-E3-T1 (`AmbientMediaShare` + packaging runbook)

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).

### Active assignment

**Ready now:** **P12-E2-T1** — Now Playing ambient plugin (`now_playing` / `BIOFOCUS_NOW_PLAYING`). Brief: `docs/handoffs/P12-E2-T1-pm-brief.md`. Role: **Dev**.

**Closed:** P12-E1-T1 (QA Pass, 2026-08-10) — ADR-012: primary Now Playing ambient; secondary packaging runbook; E2/E3 names locked. Epic **P12-E1** ✅. Evidence: `docs/handoffs/P12-E1-T1-qa-to-pm.md`.

**Closed previous:** Phase 11 (E1–E3 Done, 2026-08-10) — ADR-011 → packs → provider UX. Evidence: `docs/handoffs/P11-*-qa-to-pm.md`. Branch: `phase/11-ai-coaching-polish`.

**Next after Pass:** Ready **P12-E3-T1** — `AmbientMediaShare` + packaging runbook.

**Ops note:** **PR freeze until 2026-09-01** — Phase 12 cluster on `phase/12-ambient-packaging`; local commits OK; one cluster PR **after** freeze (or when user lifts it). No schema/sync migration approve needed for v1.

---

## Phase 12 — Epics & Tasks (Sprint 23–24)

### Epic P12-E1 — ADR-012 Phase 12 scope
**Goal:** Decide how ambient sources and commercial packaging enter v1 without breaking Local-First / Capability Model / open Core math.

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P12-E1-T1** | ADR-012: ambient + packaging boundaries | Dev | docs + decision-log | ADR-012 recorded | Phase 11 Done |

**P12-E1 shipped:** ADR-012 = primary **Now Playing / music** ambient (`now_playing`); secondary packaging as signed-build / notarization / update **runbook**; E2 plugin → E3 `AmbientMediaShare` + runbook; existing `observations` only; no migration. QA Pass 2026-08-10.

**Out of scope (E1):** implementing ambient collector / Feature / installer pipeline (→ E2/E3); IDE/Git wave; cloud LLM marketplace; PR during freeze.

---

### Epic P12-E2 — Now Playing ambient plugin
**Goal:** Ship opt-in Now Playing `BioFocusPlugin` → Observation channel → persist (ADR-012).

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P12-E2-T1** | Now Playing ambient plugin | Dev | macos-collector / plugin-sdk + host | See below | P12-E1-T1 |

**P12-E2-T1 AC:**
1. `BioFocusPlugin` id `com.biofocus.macos.now_playing` with Capability covering `now_playing`; implement in `crates/macos-collector` (or thin adapter) per ADR-012 / `docs/08-plugin-sdk.md`.
2. Desktop `ingest_host` starts the plugin **only** when `BIOFOCUS_NOW_PLAYING=1` (default **off**); same bounded Observation channel → persist worker (UI ↛ SQLite).
3. Emitted Observations match ADR-012 / contracts: required `media_kind` (`music` \| `podcast` \| `other` \| `none` \| `unknown`) + `is_playing`; **never** titles/artists/lyrics/playlists/mic/geo dumps.
4. Injectable probe for tests; production probe may soft-fail / emit nothing when OS mapping unavailable — idle-safe (change or rare ≥5s poll; **no** busy-loop); `stop_stream` joins background work.
5. Docs finalized (shipped): `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development` as needed.
6. Tests with mock probe: emit → channel → persist where practical; after `stop_stream`, emissions freeze.
7. Handoff: `docs/handoffs/P12-E2-T1-dev-to-qa.md`.

**Out of scope (E2):** `AmbientMediaShare` / Feature DAG (→ **P12-E3**); packaging installer binary; weather/light; IDE/Git; new SQLite schema; PR during freeze.

---

### Epic P12-E3 — AmbientMediaShare + packaging runbook
**Goal:** Catalog Feature `AmbientMediaShare` from `now_playing` Observations + commercial packaging companion runbook (ADR-012).

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :---: | :--- | :--- | :--- |
| **P12-E3-T1** | `AmbientMediaShare` + packaging runbook | Dev (+ UX if UI) | feature-engine + docs | AC at Ready after E2 | P12-E2-T1 |

**Note:** Exact AC filled when PM Ready's E3 after E2 Pass. Locked names: Feature `AmbientMediaShare`; packaging = signed-build / notarization / update-channel runbook (sync stance only, off by default).

**Out of scope (E3):** weather/light; App Store product; sync product; workplace surveillance framing.

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

## Role × Module Matrix (Phase 12)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P12-E1-T1 ADR-012 ambient + packaging scope | | ● | ○ | | docs + decision-log |
| P12-E2-T1 Now Playing ambient plugin | | ● | ○ | | macos-collector / plugin-sdk + host |
| P12-E3-T1 AmbientMediaShare + packaging runbook | | ● | ○ | ○ | feature-engine + docs |

● = owner · ○ = collaborator

---

## Sprint 23–24 — Queue

1. ~~P12-E1-T1 — ADR-012 Phase 12 scope (ambient + commercial packaging)~~ ✅ Done  
2. **P12-E2-T1** — Now Playing ambient plugin ← **Ready**  
3. P12-E3-T1 — `AmbientMediaShare` + packaging runbook

**Git:** `phase/12-ambient-packaging` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P12-E2-T1-pm-brief.md`.
