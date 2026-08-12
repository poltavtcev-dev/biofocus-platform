# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 25: SustainedLoadIndicator** (Sprint 49–50) — **Active**.  
> Phase 0–24 Done. Gate **PM-GATE-POST-P24** ✅ · **ADR-026** ✅ · Ready **P25-E2-T1**.  
> Deferred: IDE · weather · App Store · Companion polish-as-primary · TypingRhythm · DeepFocusLikelihood · precise GPS.

**Phase 25 outcome (target):** **ADR-026** ✅ locks `SustainedLoadIndicator` → ship catalog Feature from Stress / Fatigue / MeetingDensity → optional dogfood / calm surface. Calm non-clinical framing (“prolonged load in this window”). No burnout diagnosis. No new Observation family.

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

---

## Kanban Overview (Phase 25)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P25-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–24 · **PM-GATE-POST-P14…P24** · **P25-E1-T1** · **P24-E1–E3** · **P23-E1–E3** (see archives) |

**Epic status:** Phase 25 Active (E1 ✅ · E2 Ready) · Phase 24 ✅

**Phase 25 on `/docs/14-roadmap.md`:** opened 2026-08-12 · **ADR-026** ✅ · Ready E2 ship

**Live board (maintainers):** Cursor canvas `biofocus-execution-board.canvas.tsx` in the local Cursor projects `canvases/` directory (not required for external contributors).

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P25-E2-T1** — Ship catalog Feature `SustainedLoadIndicator` per ADR-026. Brief: `docs/handoffs/P25-E2-T1-pm-brief.md`. Role: **Dev**.

**Just closed:** **P25-E1-T1** (2026-08-12) — **ADR-026** locked (QA Pass).

**Ops note:** **PR freeze until 2026-09-01** — no PR. Phase 25 on `phase/25-sustained-load`.

---

## Phase 25 — SustainedLoadIndicator (Active)

### Epic P25-E1 — Contracts ADR (**ADR-026**)

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P25-E1-T1** ✅ Done | Lock `SustainedLoadIndicator` Feature scope (**ADR-026**) | Dev | docs + decision-log |

### Epic P25-E2 — First ship slice

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P25-E2-T1** ← Ready | Ship catalog Feature `SustainedLoadIndicator` | Dev | `feature-engine` + docs |

### Epic P25-E3 — Dogfood / surface (optional)

| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P25-E3-T1** | Dogfood notes + optional calm UI surface | Dev | docs (+ optional UI) |

---

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

## Queue (Phase 25)

1. **P25-E1-T1** — ADR-026 SustainedLoadIndicator ← **Done**  
2. **P25-E2-T1** — Ship catalog Feature ← **Ready**  
3. **P25-E3-T1** — Dogfood / optional surface  

**Deferred:** IDE · weather · App Store · Companion polish-as-primary · TypingRhythm · DeepFocusLikelihood · precise GPS.

**Git:** Phase 25 on `phase/25-sustained-load` → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P25-E2-T1-pm-brief.md`.
