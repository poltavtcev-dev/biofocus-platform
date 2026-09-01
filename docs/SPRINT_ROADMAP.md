# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 27: Pattern Discovery rule expansion** (Sprint 53–54) — **Active**.  
> Phase 0–26 Done. Gate **PM-GATE-POST-P26** ✅ · **ADR-028** ✅ · E2 rules ✅ · Ready **P27-E3-T1**.  
> **Public launch not Done** (OSS layers 2–3 after PR freeze). Deferred: park-until-freeze-lift as primary · IDE · weather · App Store · Companion polish-as-primary · TypingRhythm · DeepFocusLikelihood · precise GPS · Feature-math expansion.

**Phase 27 outcome (target):** **ADR-028** locks Insight/Recommendation rule slate on **shipped** Features → register rules in `knowledge-engine` → optional dogfood / Dashboard Insights·Suggestions. Extend ADR-008 / ADR-009. **No** new Observation / Feature math. **Not** App Store. **Not** public launch Done.

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
- [ ] **Companion autonomy:** HealthKit event → local queue → flush; no busy-loop; no cloud relay; no clinical claims on HRV/SDNN
- [ ] **Ambient light:** opt-in; coarse light labels/levels only — no camera frames / screen contents / precise geo
- [ ] **Wearable depth:** HealthKit only (no Mi Cloud); soft-optional HRV; new types only per Phase 17 contract ADR
- [ ] **Chart ranges:** recompute-on-read; UI ↛ SQLite; Snapshot = latest (series on chart)
- [ ] **Notifications:** opt-in; coarse counts/cadence only — **no** notification body/title/content; personal self-tracking only
- [ ] **Personal context:** Variant B framing; user-declared health only; no precise GPS desk-away; no diagnosis from biometrics
- [ ] **Circadian:** schedule-alignment framing only — no chronotype / sleep-disorder diagnosis
- [ ] **Sustained load:** prolonged-load framing only — no burnout / clinical diagnosis
- [ ] **OSS launch:** AGPLv3 Core open; no App Store productization as Phase 26 primary; no cloud accounts / telemetry by default; packaging ≠ secret Feature math
- [ ] **Pattern rules:** evaluate-on-read only; Evidence-backed; no LLM-authored Insights/Recommendations; no new Feature math this phase

---

## Kanban Overview (Phase 27)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P27-E3-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–26 · **PM-GATE-POST-P14…P26** · **P26-E1–E3** · **P25-E1–E3** · **P27-E1–E2** (see archives) |

**Epic status:** Phase 27 Active (E1–E2 ✅ · E3 Ready) · Phase 26 ✅

**Phase 27 on `/docs/14-roadmap.md`:** opened 2026-08-12 · **ADR-028** ✅ · E2 rules shipped · Ready E3 dogfood

**Live board (maintainers):** Cursor canvas `biofocus-execution-board.canvas.tsx` in the local Cursor projects `canvases/` directory (not required for external contributors).

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P27-E3-T1** — Dogfood notes + optional calm Insights/Suggestions surface for ADR-028 rules. Brief: `docs/handoffs/P27-E3-T1-pm-brief.md`. Role: **Dev|UX** (docs + optional Dashboard).

**Just closed:** **P27-E2-T1** (2026-08-12) — shipped `cognitive_load_elevated_v1`, `sustained_load_elevated_v1`, `combined_demand_pace_hint_v1` in `knowledge-engine` (QA Pass · 46/46).

**Ops note:** **PR freeze until 2026-09-01** — no PR. Phase 27 on `phase/27-pattern-rules`. Phase 26 cluster stays on `phase/26-oss-public-launch`. **Public launch not Done** (OSS layers 2–3 parked after freeze).

---

## Phase 27 — Pattern Discovery rule expansion (Active)

### Epic P27-E1 — Contracts ADR (**ADR-028**) ✅

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P27-E1-T1** ✅ | Lock Pattern Discovery / Recommendations expansion (**ADR-028**) | Dev | docs + decision-log (+ knowledge-engine contracts) |

### Epic P27-E2 — Rules ship ✅

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P27-E2-T1** ✅ | Ship locked Insight/Recommendation rules | Dev | knowledge-engine (+ tests) |

### Epic P27-E3 — Dogfood / surface

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P27-E3-T1** ← Ready | Dogfood notes + optional calm Insights/Suggestions surface | Dev\|UX | docs (+ optional Dashboard) |

---

## Phase 26 archive (Done)

<details>
<summary>Phase 26 Kanban & epics (closed 2026-08-12 — OSS Public Launch Hygiene)</summary>

**Done:** P26-E1 (T1) · P26-E2 (T1) · P26-E3 (T1).  
**ADR-027** · SoT `docs/19-oss-public-launch.md` · dry-run checklist vs `docs/18-packaging-runbook.md`.  
**Public launch not Done** — layers (2)–(3) after freeze.  
Branch: `phase/26-oss-public-launch` (cluster PR after freeze).

Evidence: `docs/handoffs/P26-*-qa-to-pm.md`.

</details>

## Phase 25 archive (Done)

<details>
<summary>Phase 25 Kanban & epics (closed 2026-08-12 — SustainedLoadIndicator)</summary>

**Done:** P25-E1 (T1) · P25-E2 (T1) · P25-E3 (T1).  
**ADR-026** · `SustainedLoadIndicator` Feature · dogfood + calm Dashboard **Prolonged load**.  
Branch: `phase/25-sustained-load` (cluster PR after freeze).

Evidence: `docs/handoffs/P25-*-qa-to-pm.md`.

</details>

## Phase 24 archive (Done)

<details>
<summary>Phase 24 Kanban & epics (closed 2026-08-12 — CircadianOffset)</summary>

**Done:** P24-E1 (T1) · P24-E2 (T1) · P24-E3 (T1).  
**ADR-025** · `CircadianOffset` Feature · dogfood + calm Dashboard **Schedule alignment**.  
Branch: `phase/24-circadian-offset` (cluster PR after freeze).

Evidence: `docs/handoffs/P24-*-qa-to-pm.md`.

</details>

## Phase 23 archive (Done)

<details>
<summary>Phase 23 Kanban & epics (closed 2026-08-11 — Personal Context Layer)</summary>

**Done:** P23-E1 (T1) · P23-E2 (T1) · P23-E3 (T1).  
**ADR-024** · `DeskAwayPresence` Feature · health→prompt · dogfood + calm Dashboard **Away from desk**.  
Branch: `phase/23-personal-context` (cluster PR after freeze).

Evidence: `docs/handoffs/P23-*-qa-to-pm.md`.

</details>

## Phase 22 archive (Done)

<details>
<summary>Phase 22 Kanban & epics (closed 2026-08-11 — AttentionStability)</summary>

**Done:** P22-E1 (T1) · P22-E2 (T1) · P22-E3 (T1).  
**ADR-023** · `AttentionStability` Feature · dogfood + calm Dashboard Focus stability.  
Branch: `phase/22-attention-stability` (cluster PR after freeze).

Evidence: `docs/handoffs/P22-*-qa-to-pm.md`.

</details>

## Phase 21 archive (Done)

<details>
<summary>Phase 21 Kanban & epics (closed 2026-08-11 — DeepWorkScore)</summary>

**Done:** P21-E1 (T1) · P21-E2 (T1) · P21-E3 (T1).  
**ADR-022** · `DeepWorkScore` Feature · dogfood + calm Dashboard Sustained focus.  
Branch: `phase/21-deep-work-score` (cluster PR after freeze).

Evidence: `docs/handoffs/P21-*-qa-to-pm.md`.

</details>

## Phase 20 archive (Done)

<details>
<summary>Phase 20 Kanban & epics (closed 2026-08-11 — CognitiveLoad)</summary>

**Done:** P20-E1 (T1) · P20-E2 (T1) · P20-E3 (T1).  
**ADR-021** · `CognitiveLoad` Feature · dogfood + calm Dashboard Combined demand.  
Branch: `phase/20-cognitive-load` (cluster PR after freeze).

Evidence: `docs/handoffs/P20-*-qa-to-pm.md`.

</details>

---

## Queue (Phase 27)

1. **P27-E1-T1** — ADR-028 Pattern Discovery / Recommendations expansion ← **Done** (2026-08-12)  
2. **P27-E2-T1** — Ship locked Insight/Recommendation rules ← **Done** (2026-08-12)  
3. **P27-E3-T1** — Dogfood + optional Insights/Suggestions surface ← **Ready**  

**Deferred / parked:** OSS layers 2–3 (after freeze) · IDE · weather · App Store · TypingRhythm · DeepFocusLikelihood · precise GPS · Feature-math.

**Git:** Phase 27 on `phase/27-pattern-rules` → **PR after 2026-09-01**. Phase 28 on `phase/28-local-reliability` (may start parallel after P27-E3 or interrupt for user blocker).  
**Brief:** `docs/handoffs/P27-E3-T1-pm-brief.md`.  
**Evidence E1–E2:** `docs/handoffs/P27-E1-T1-qa-to-pm.md` · `docs/handoffs/P27-E2-T1-qa-to-pm.md`.  
**Note:** **Public launch not Done**.

---

## Phase 28 — Local device reliability (A+B) — **Queued** (user blocker)

> **ADR-029** ✅ (2026-08-31) — Mac always-on **(A)** + iPhone buffer **(B)**; no BioFocus cloud.  
> **Outcome:** данные только у юзера; companion не отваливается; Mac догоняет после offline; Insights/графики начинают приносить пользу на 1–2-й неделе.

**Branch:** `phase/28-local-reliability` (or ride `phase/28-companion-connectivity` for E1 only).

| Status | IDs |
| :--- | :--- |
| **Ready** | **P28-E1-T1** (after PM gate or parallel to P27-E3) |
| **Queued** | P28-E2-T1 · P28-E3-T1 · P28-E4-T1 · P28-E5-T1 (optional) |
| **Done** | — |

### Epic P28-E1 — Companion connectivity (ADR-029 / ADR-005)

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P28-E1-T1** ← Ready | LAN preflight, timeouts, calm errors, Desktop Companion UX | Dev\|UX | `apps/companion/ios`, `apps/desktop`, docs |

Brief: `docs/handoffs/P28-E1-T1-pm-brief.md`.

### Epic P28-E2 — iPhone buffer (B)

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P28-E2-T1** | Retry/backoff flush, reachability + foreground triggers, queue durability | Dev | `apps/companion/ios`, tests |

Brief: `docs/handoffs/P28-E2-T1-pm-brief.md`.

### Epic P28-E3 — Mac always-on (A)

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P28-E3-T1** | Login Item opt-in, hide≠quit, lifecycle on login start | Dev\|UX | `apps/desktop`, docs |

Brief: `docs/handoffs/P28-E3-T1-pm-brief.md`.

### Epic P28-E4 — Catch-up replay

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P28-E4-T1** | Feature Worker drains cursor after offline; snapshot refresh | Dev | `apps/desktop`, `runtime`, `feature_host` |

Brief: `docs/handoffs/P28-E4-T1-pm-brief.md`.

### Epic P28-E5 — Sync health (optional)

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P28-E5-T1** | Calm sync status (last flush, pending, Mac reachable) | Dev\|UX | Desktop Companion UI + iOS status |

Brief: `docs/handoffs/P28-E5-T1-pm-brief.md`.

**Sequencing:** E1 → E2 → E3 → E4 (E5 anytime after E1). **PM-GATE-POST-P27** may promote P28-E1 ahead of P27-E3 (user blocker).
