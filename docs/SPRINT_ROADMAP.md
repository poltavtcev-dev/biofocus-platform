# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 14: Git path-allowlist / live probe** (Sprint 27–28) — Active.  
> Phase 0–13 Done. Goal: ADR-014 watched-roots allowlist → live `SystemGitActivityProbe` for already-shipped `git_activity` / `GitActivityRate` — Local-First, personal self-tracking only.

**Phase 14 goal:** Unlock **live** Git activity Observations via a privacy-scoped **path / watched-roots allowlist** (ADR-014). Observation payload contract from ADR-013 stays coarse (`activity_kind` + optional `event_count`). IDE, weather/light, App Store packaging product, and NotificationPressure remain deferred.

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

---

## Kanban Overview (Phase 14 Active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P14-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **Phase 5** (E1–E3) · **Phase 6** (E1–E3) · **Phase 7** (E1–E3) · **Phase 8** (E1–E3) · **Phase 9** (E1–E3) · **Phase 10** (E1–E3) · **Phase 11** (E1–E3) · **Phase 12** (E1–E3) · **Phase 13** (E1–E3) · **PM-GATE-POST-P13** · **P14-E1-T1** |

**Epic status:** P14-E1 ✅ · P14-E2 ⬜ (T1 Ready) · P14-E3 ⬜

**Phase 14 on `/docs/14-roadmap.md`:** opened 2026-08-10 · ADR-014 ✅ config-file allowlist · Ready live probe

**Рекомендуемый порядок (Phase 14):**  
~~P14-E1-T1~~ → **P14-E2-T1** → P14-E3-T1

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **PR freeze до 2026-09-01** — commits OK, no PRs (`docs/12-development.md`).

### Active assignment

**Ready now:** **P14-E2-T1** — Allowlist load (`~/.biofocus/git-watched-roots.toml`) + live `SystemGitActivityProbe`. Brief: `docs/handoffs/P14-E2-T1-pm-brief.md`. Role: **Dev**.

**Closed:** P14-E1-T1 (QA Pass, 2026-08-10) — ADR-014: v1 allowlist = local config file `~/.biofocus/git-watched-roots.toml` (not SQLite / no migration); Observation payload unchanged. Epic **P14-E1** ✅. Evidence: `docs/handoffs/P14-E1-T1-qa-to-pm.md`.

**Closed gate:** PM-GATE-POST-P13 (2026-08-10) — chose **Git path-allowlist**. Evidence: `docs/handoffs/PM-GATE-POST-P13-pm-brief.md`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 14 on `phase/14-git-allowlist`; local commits OK; cluster PR **after** freeze. Locked: `git-watched-roots.toml`, ADR-014, no schema approve before E2.

---

## Phase 14 — Git path-allowlist / live probe (Active)

### Epic P14-E1 — ADR-014 allowlist scope
**P14-E1 shipped:** ADR-014 = v1 allowlist store **local config** `~/.biofocus/git-watched-roots.toml` (`version` + absolute `roots`); **no** SQLite table / **no** migration; Observation payload stays ADR-013 coarse; empty/missing → soft-fail idle; E2 live probe → existing `GitActivityRate`. QA Pass 2026-08-10.

### Epic P14-E2 — Live Git probe + allowlist
**Goal:** Implement ADR-014 allowlist + live `SystemGitActivityProbe` → existing Observation channel (payload unchanged).

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P14-E2-T1** | Allowlist + live git probe | Dev | macos-collector + host | See `P14-E2-T1-pm-brief.md` | P14-E1-T1 |

### Epic P14-E3 — Dogfood / allowlist UX companion
**Goal:** Close Phase 14 with dogfood notes and/or calm Settings/IPC to edit allowlist (no Feature math rewrite).

| ID | Task | Role | Modules | AC (summary) | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P14-E3-T1** | Dogfood gate / allowlist UX | Dev\|UX | docs + optional desktop IPC | Shaped by ADR-014 | P14-E2-T1 |

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

## Role × Module Matrix (Phase 14)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P14-E1-T1 ADR-014 allowlist scope | | ● | ○ | | docs + decision-log |
| P14-E2-T1 live probe + allowlist | | ● | ○ | | macos-collector + host |
| P14-E3-T1 dogfood / allowlist UX | | ● | ○ | ○ | docs + optional desktop |

● = owner · ○ = collaborator

---

## Queue (Phase 14)

1. ~~P14-E1-T1 — ADR-014 Git watched-roots / path-allowlist~~ ✅ Done  
2. **P14-E2-T1** — Allowlist + live git probe ← **Ready**  
3. P14-E3-T1 — Dogfood gate / allowlist UX  

**Git:** `phase/14-git-allowlist` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P14-E2-T1-pm-brief.md`.  
**Deferred:** IDE · weather/light · App Store packaging · NotificationPressure · Phase 13 cluster PR batch after freeze.
