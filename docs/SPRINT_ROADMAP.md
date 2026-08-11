# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 21: DeepWorkScore** (Sprint 41–42) — **Active**.  
> Phase 0–20 Done. Gate **PM-GATE-POST-P20** ✅ chose catalog Feature **`DeepWorkScore`** from FocusScore + ContextSwitchRate.

**Phase 21 outcome (target):** **ADR-022** ✅ locks sustained-focus Feature scope → ship `DeepWorkScore` in `feature-engine` → optional dogfood / calm Dashboard surface. No new Observation family; calm non-clinical framing.

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

## Kanban Overview (Phase 21)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P21-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–20 · **PM-GATE-POST-P14** · **PM-GATE-POST-P15** · **PM-GATE-POST-P17** · **PM-GATE-POST-P18** · **PM-GATE-POST-P19** · **PM-GATE-POST-P20** · **P20-E1-T1** · **P20-E2-T1** · **P20-E3-T1** · **P21-E1-T1** |

**Epic status:** Phase 21 Active (E1 ✅ · E2 Ready) · Phase 20 ✅

**Phase 21 on `/docs/14-roadmap.md`:** opened 2026-08-11 · **ADR-022** ✅ · E1 Done · E2 Ready

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P21-E2-T1** — Ship catalog Feature `DeepWorkScore` per ADR-022. Brief: `docs/handoffs/P21-E2-T1-pm-brief.md`. Role: **Dev**.

**Just closed:** **P21-E1-T1** (2026-08-11) — ADR-022 locked Focus required + CSR optional; idle dropped; omit-without-Focus / renormalize-without-CSR. Evidence: `docs/handoffs/P21-E1-T1-qa-to-pm.md`.

**Ops note:** **PR freeze until 2026-09-01** — no PR. Phase 20 remains on `phase/20-cognitive-load`; Phase 21 on `phase/21-deep-work-score`.

---

## Phase 21 — DeepWorkScore (Active)

### Epic P21-E1 — Contracts ADR (**ADR-022**) ✅
| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P21-E1-T1** ✅ | Lock `DeepWorkScore` Feature scope (**ADR-022**) | Dev | docs + decision-log |

### Epic P21-E2 — Feature
| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P21-E2-T1** ← Ready | Ship catalog Feature `DeepWorkScore` per ADR-022 | Dev | feature-engine + docs |

### Epic P21-E3 — Dogfood / surface
| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P21-E3-T1** | Dogfood notes + optional calm Dashboard surface | Dev | docs (+ optional UI) |

---

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

## Phase 18 archive (Done)

<details>
<summary>Phase 18 Kanban & epics (closed 2026-08-11 — notification pressure)</summary>

**Done:** P18-E1 (T1) · P18-E2 (T1) · P18-E3 (T1).  
**ADR-019** contracts · `notification_event` collector · `NotificationPressure`.  
Branch: `phase/18-notification-pressure` (cluster PR after freeze).

Evidence: `docs/handoffs/P18-*-qa-to-pm.md`.

</details>

---

## Queue (Phase 21)

1. **P21-E1-T1** — ADR-022 DeepWorkScore scope ← **Done**  
2. **P21-E2-T1** — Ship `DeepWorkScore` Feature ← **Ready**  
3. **P21-E3-T1** — Dogfood / optional Dashboard surface  
4. Deferred (later gates): IDE · weather ambient · App Store packaging · Companion polish · AttentionStability · CircadianOffset  

**Git:** `phase/21-deep-work-score` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P21-E2-T1-pm-brief.md`.
