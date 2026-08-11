# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 15: Ambient light** (Sprint 29–30) — Active.  
> Phase 0–14 Done. Goal: ADR-015 ambient light Observation → opt-in collector → calm ambient Feature — Local-First, personal self-tracking only.

**Phase 15 goal:** Ship **ambient light** as the next ambient Observation source after Now Playing (weather deferred). Coarse privacy-safe payload only; Feature only from real Observations.

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
- [ ] **Ambient light:** opt-in; coarse light labels/levels only — no camera frames / screen contents / precise geo

---

## Kanban Overview (Phase 15 Active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P15-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtsev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **Phase 9** (E1–E3) · **Phase 10** (E1–E3) · **Phase 11** (E1–E3) · **Phase 12** (E1–E3) · **Phase 13** (E1–E3) · **PM-GATE-POST-P13** · **Phase 14** (E1–E3) · **PM-GATE-POST-P14** · **P14-E1-T1** · **P14-E2-T1** · **P14-E3-T1** · **P15-E1-T1** |

**Epic status:** P15-E1 ✅ · P15-E2 ○ · P15-E3 ○ · Phase 14 ✅ · **PM-GATE-POST-P14** ✅

**Phase 15 on `/docs/14-roadmap.md`:** opened 2026-08-10 · Ambient light · ADR-015 locked · Ready plugin

**Рекомендуемый порядок (Phase 15):**  
P15-E1-T1 → P15-E2-T1 → P15-E3-T1

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).

### Active assignment

**Ready now:** **P15-E2-T1** — Implement ambient light plugin (`macos-collector` + host) per ADR-015. Brief: `docs/handoffs/P15-E2-T1-pm-brief.md`. Role: **Dev**.

**Closed:** P15-E1-T1 (QA Pass, 2026-08-11) — ADR-015 ambient light Observation contract; Epic **P15-E1** ✅. Evidence: `docs/handoffs/P15-E1-T1-qa-to-pm.md`. No schema approve needed (existing `observations` only).

**Closed gate:** PM-GATE-POST-P14 (2026-08-10) — chose **ambient light** (weather/light track) over IDE / weather-as-primary / App Store / NotificationPressure. Evidence: `docs/handoffs/PM-GATE-POST-P14-pm-brief.md`.

**Closed:** P14-E3-T1 (QA Pass, 2026-08-10) — dogfood + Menubar **Git folders** IPC; Epic **P14-E3** ✅ · **Phase 14** ✅. Evidence: `docs/handoffs/P14-E3-T1-qa-to-pm.md`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 14 cluster on `phase/14-git-allowlist`; Phase 15 on `phase/15-ambient-light`; local commits OK; cluster PRs **after** freeze.

---

## Phase 15 — Ambient light (Active)

### Epic P15-E1 — ADR-015 ambient light scope
**Goal:** Lock Observation contract + privacy boundaries before collector/Feature.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P15-E1-T1** | ADR-015: ambient light contract | Dev | docs + decision-log | See `P15-E1-T1-pm-brief.md` | PM-GATE-POST-P14 |

**P15-E1 shipped:** ADR-015 = Phase 15 v1 primary **ambient light** (not weather); `data_type: "ambient_light"`, coarse `light_kind` + optional `level` 0–100; opt-in `BIOFOCUS_AMBIENT_LIGHT` default off; existing `observations` only (**no** migration); E3 Feature name **`AmbientLightShare`**. QA Pass 2026-08-11.

### Epic P15-E2 — Ambient light plugin
**Goal:** Opt-in collector → existing Observation channel (payload per ADR-015).

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P15-E2-T1** | Implement ambient light plugin | Dev | macos-collector + host | See `P15-E2-T1-pm-brief.md` | P15-E1-T1 |

### Epic P15-E3 — Ambient light Feature
**Goal:** Catalog Feature from ambient light Observations (ADR-007; calm framing).

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P15-E3-T1** | Ambient light catalog Feature | Dev | feature-engine + docs | Shaped by ADR-015 | P15-E2-T1 |

---

## Phase 14 archive (Done)

<details>
<summary>Phase 14 Kanban & epics (closed 2026-08-10 — E1–E3)</summary>

**Done:** P14-E1 (T1) · P14-E2 (T1) · P14-E3 (T1).  
ADR-014 config allowlist → live `SystemGitActivityProbe` → dogfood + Menubar **Git folders** IPC.

Evidence: `docs/handoffs/P14-*-qa-to-pm.md` · branch `phase/14-git-allowlist` (cluster PR after freeze).

### Epic P14-E1 — ADR-014 allowlist scope
**P14-E1 shipped:** ADR-014 = v1 allowlist store **local config** `~/.biofocus/git-watched-roots.toml` (`version` + absolute `roots`); **no** SQLite table / **no** migration; Observation payload stays ADR-013 coarse; empty/missing → soft-fail idle; E2 live probe → existing `GitActivityRate`. QA Pass 2026-08-10.

### Epic P14-E2 — Live Git probe + allowlist
**P14-E2 shipped:** Load/validate `git-watched-roots.toml`; live `SystemGitActivityProbe` under allowlisted roots only; opt-in `BIOFOCUS_GIT_ACTIVITY`; optional `BIOFOCUS_GIT_WATCHED_ROOTS` when file absent; idle-safe + stop joins; ADR-013 payload unchanged; scripted tests + empty-allowlist soft-fail. QA Pass 2026-08-10.

### Epic P14-E3 — Dogfood / allowlist UX companion
**P14-E3 shipped:** Dogfood runbook in `docs/12-development.md`; Tauri IPC `get_git_watched_roots` / `set_git_watched_roots` (file only, absolute roots); Menubar **Git folders**; `?mockGitRoots=` for browser QA; no Feature rewrite / no SQLite migration. QA Pass 2026-08-10.

**Out of scope (Phase 14):** IDE collector; weather/light; App Store product; NotificationPressure; SQLite allowlist table; workplace surveillance framing; PR during freeze.

</details>

---

## Phase 13 archive (Done)

<details>
<summary>Phase 13 Kanban & epics (closed 2026-08-10 — E1–E3)</summary>

**Done:** P13-E1 (T1) · P13-E2 (T1) · P13-E3 (T1).  
ADR-013 Git activity → `GitActivityPlugin` → `GitActivityRate`.

Evidence: `docs/handoffs/P13-*-qa-to-pm.md` · branch `phase/13-plugin-wave-2` (cluster PR after freeze).

### Epic P13-E1 — ADR-013 Plugin wave-2 scope
**P13-E1 shipped:** ADR-013 = wave-2 primary **Git activity aggregates** (`git_activity`); IDE deferred (no additive privacy-safe signal beyond `context_window`); E2 plugin → E3 **`GitActivityRate`**; existing `observations` only; no migration. QA Pass 2026-08-10.

### Epic P13-E2 — Git activity collector plugin
**P13-E2 shipped:** `GitActivityPlugin` (`com.biofocus.macos.git`); opt-in `BIOFOCUS_GIT_ACTIVITY`; `validate_git_activity_payload`; pipeline normalize strips forbidden keys; idle-safe + stop joins; production OS probe soft-fails idle without path-allowlist ADR (by design) — scripted probe covers emit→persist. QA Pass 2026-08-10.

### Epic P13-E3 — GitActivityRate catalog Feature
**P13-E3 shipped:** Catalog §1.10 `GitActivityRate` via `register_git_v1`; sum `event_count` (default 1) for `commit|checkout|sync|other` → **events per 15m window**; **omit** empty / only-`idle` / only-`unknown`; ADR-007 confidence + kind factors; distinct from `DistractionScore`. QA Pass 2026-08-10.

**Out of scope (Phase 13):** IDE collector; weather/light; App Store product; NotificationPressure; path-allowlist table; workplace surveillance framing; PR during freeze.

</details>

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

## Role × Module Matrix (Phase 15 Active)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P15-E1-T1 ADR-015 ambient light scope | | ● | ○ | | docs + decision-log |
| P15-E2-T1 ambient light plugin | | ● | ○ | | macos-collector + host |
| P15-E3-T1 ambient light Feature | | ● | ○ | | feature-engine + docs |

● = owner · ○ = collaborator

---

## Queue (Phase 15)

1. ~~P15-E1-T1 — ADR-015 Ambient light Observation contract~~ ✅  
2. **P15-E2-T1** — Ambient light plugin ← **Ready**  
3. P15-E3-T1 — Ambient light catalog Feature  

**Git:** `phase/15-ambient-light` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P15-E2-T1-pm-brief.md`.  
**Deferred:** IDE · weather ambient · App Store packaging · NotificationPressure · Phase 14/15 cluster PR batch after freeze.
