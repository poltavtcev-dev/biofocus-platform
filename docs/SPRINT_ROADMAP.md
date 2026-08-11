# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 19: Live NC OS mapping** (Sprint 37–38) — **Active**.  
> Phase 0–18 Done. Gate **PM-GATE-POST-P18** ✅ · **ADR-020** ✅ · Ready **P19-E2-T1**.

**Phase 19 outcome (target):** **ADR-020** locks privacy-safe OS mapping → live `SystemNotificationEventProbe` emits → existing `NotificationPressure` dogfoods. No body/title content; soft-fail still OK when mapping unavailable.

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

## Kanban Overview (Phase 19)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P19-E2-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–18 · **PM-GATE-POST-P14** · **PM-GATE-POST-P15** · **PM-GATE-POST-P17** · **PM-GATE-POST-P18** · **P16-E1-T1** · **P16-E2-T1** · **P17-E1-T1** · **P17-E2-T1** · **P17-E3-T1** · **P18-E1-T1** · **P18-E2-T1** · **P18-E3-T1** · **P19-E1-T1** |

**Epic status:** Phase 19 Active (E1 ✅ · E2 Ready) · Phase 18 ✅

**Phase 19 on `/docs/14-roadmap.md`:** **ADR-020** ✅ · E1 Done · Ready **P19-E2-T1**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P19-E2-T1** — Implement live `SystemNotificationEventProbe` mapping per ADR-020. Brief: `docs/handoffs/P19-E2-T1-pm-brief.md`. Role: **Dev**.

**Just closed:** **P19-E1-T1** (2026-08-11) — **ADR-020** hybrid NC OS mapping + field allowlist (QA Pass). Evidence: `docs/handoffs/P19-E1-T1-qa-to-pm.md`.

**Ops note:** **PR freeze until 2026-09-01** — no PR. Branch: `phase/19-live-nc-mapping`. No schema approve for E2.

---

## Phase 19 — Live NC OS mapping (Active)

### Epic P19-E1 — Contracts ADR (**ADR-020**) ✅
| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P19-E1-T1** ✅ | Lock privacy-safe NC OS mapping + live probe boundaries (**ADR-020**) | Dev | docs + decision-log |

### Epic P19-E2 — Live probe
| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P19-E2-T1** ← Ready | Implement live `SystemNotificationEventProbe` mapping per ADR-020 | Dev | macos-collector + bio-spec/pipeline |

### Epic P19-E3 — Dogfood
| ID | Task | Role | Modules |
| :--- | :--- | :--- | :--- |
| **P19-E3-T1** | Dogfood runbook + verify `NotificationPressure` on live emits | Dev | docs (+ optional calm status) |

---

## Phase 18 archive (Done)

<details>
<summary>Phase 18 Kanban & epics (closed 2026-08-11 — notification pressure)</summary>

**Done:** P18-E1 (T1) · P18-E2 (T1) · P18-E3 (T1).  
**ADR-019** contracts · `notification_event` collector · `NotificationPressure`.  
Branch: `phase/18-notification-pressure` (cluster PR after freeze).

Evidence: `docs/handoffs/P18-*-qa-to-pm.md`.

</details>

## Phase 17 archive (Done)

<details>
<summary>Phase 17 Kanban & epics (closed 2026-08-11 — wearable depth + chart ranges)</summary>

**Done:** P17-E1 (T1) · P17-E2 (T1) · P17-E3 (T1).  
ADR-017 sequencing · **ADR-018** contracts · Companion HealthKit emit · `get_feature_series` + wearable Features.  
Branch: `phase/17-wearable-charts` (cluster PR after freeze).

Evidence: `docs/handoffs/P17-*-qa-to-pm.md`.

</details>

## Phase 16 archive (Done)

<details>
<summary>Phase 16 Kanban & epics (closed 2026-08-11 — ambient light)</summary>

**Done:** P16-E1 (T1) · P16-E2 (T1).  
ADR-015 collector + `AmbientLightShare`. Branch: `phase/16-ambient-light` (cluster PR after freeze).

Evidence: `docs/handoffs/P16-*-qa-to-pm.md`.

</details>

---

## Queue (Phase 19)

1. **P19-E1-T1** — ADR-020 NC OS mapping ← **Done**  
2. **P19-E2-T1** — Live SystemNotificationEventProbe ← **Ready**  
3. **P19-E3-T1** — Dogfood runbook / Feature verify  
4. Deferred (later gates): IDE · weather ambient · App Store packaging · `CognitiveLoad` · Companion polish  

**Git:** `phase/19-live-nc-mapping` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P19-E2-T1-pm-brief.md`.
