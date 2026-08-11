# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 22: AttentionStability** — **Done** (2026-08-11).  
> Phase 0–22 Done. Gate **PM-GATE-POST-P22** ← **Ready** (choose Phase 23+).

**Phase 22 outcome:** **ADR-023** ✅ → `AttentionStability` Feature shipped → dogfood + Dashboard **Focus stability**. No new Observation family; calm non-clinical framing; distinct from DeepWorkScore.

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

---

## Kanban Overview (Post–Phase 22)

| Status | IDs |
| :--- | :--- |
| **Ready** | **PM-GATE-POST-P22** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–22 · **PM-GATE-POST-P14** · **PM-GATE-POST-P15** · **PM-GATE-POST-P17** · **PM-GATE-POST-P18** · **PM-GATE-POST-P19** · **PM-GATE-POST-P20** · **PM-GATE-POST-P21** · **P22-E1–E3** (see archive) |

**Epic status:** Phase 22 ✅ · Gate Ready

**Phase 22 on `/docs/14-roadmap.md`:** closed 2026-08-11 · **ADR-023** ✅ · Feature + dogfood + Focus stability

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **PM-GATE-POST-P22** — Choose Phase 23+ primary slice. Brief: `docs/handoffs/PM-GATE-POST-P22-pm-brief.md`. Role: **PM**.

**Just closed:** **P22-E3-T1** (2026-08-11) — dogfood + Focus stability chart; **Phase 22** closed.

**Ops note:** **PR freeze until 2026-09-01** — no PR. Phase 22 remains on `phase/22-attention-stability`.

---

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

## Phase 19 archive (Done)

<details>
<summary>Phase 19 Kanban & epics (closed 2026-08-11 — live NC OS mapping)</summary>

**Done:** P19-E1 (T1) · P19-E2 (T1) · P19-E3 (T1).  
**ADR-020** · usernoted live probe · dogfood runbook for `NotificationPressure`.  
Branch: `phase/19-live-nc-mapping` (cluster PR after freeze).

Evidence: `docs/handoffs/P19-*-qa-to-pm.md`.

</details>

---

## Queue (Post–Phase 22)

1. **PM-GATE-POST-P22** — Choose Phase 23+ ← **Ready**  
   Candidates: IDE · weather · App Store · Companion polish · CircadianOffset · other  

**Git:** Phase 22 on `phase/22-attention-stability` → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/PM-GATE-POST-P22-pm-brief.md`.
