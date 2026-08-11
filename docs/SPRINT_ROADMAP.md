# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 16: Ambient light** (Sprint 31–32) — Active.  
> Phase 0–15 Done. Goal: resume ADR-015 ambient light Observation → opt-in collector → `AmbientLightShare` — Local-First, personal self-tracking only.

**Phase 16 goal:** Ship **ambient light** collector + catalog Feature (contract already locked in ADR-015; parked during companion Phase 15). Coarse privacy-safe payload only; Feature only from real Observations. Weather remains deferred.

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

---

## Kanban Overview (Phase 16 Active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P16-E1-T1** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0–15 · **PM-GATE-POST-P14** · **PM-GATE-POST-P15** · **P15-E1-T1** · **P15-E2-T1** · **P15-E3-T1** |

**Epic status:** P16-E1 ○ · P16-E2 ○ · Phase 15 ✅ · **PM-GATE-POST-P15** ✅

**Phase 16 on `/docs/14-roadmap.md`:** opened 2026-08-11 · Ambient light resume (ADR-015) · Ready plugin

**Рекомендуемый порядок (Phase 16):**  
P16-E1-T1 → P16-E2-T1

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Developer-AI-Project-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. Git: **PR freeze до 2026-09-01**.

### Active assignment

**Ready now:** **P16-E1-T1** — Implement ambient light plugin (`macos-collector` + host) per ADR-015. Brief: `docs/handoffs/P16-E1-T1-pm-brief.md`. Role: **Dev**.

**Closed gate:** PM-GATE-POST-P15 (2026-08-11) — chose **resume ambient light** (ADR-015) over IDE / weather / App Store / NotificationPressure. Evidence: `docs/handoffs/PM-GATE-POST-P15-pm-brief.md`.

**Closed:** Phase 15 companion (2026-08-11) — ADR-016 → Core SDNN-or-RMSSD + iOS Auto-sync → dogfood. Branch: `phase/15-companion-hrv-autonomy`.

**Ops note:** **PR freeze until 2026-09-01** — Phase 15 cluster on `phase/15-companion-hrv-autonomy`; Phase 16 on `phase/16-ambient-light`; local commits OK; cluster PRs **after** freeze.

---

## Phase 16 — Ambient light (Active)

### Epic P16-E1 — Ambient light plugin
**Goal:** Opt-in collector → existing Observation channel (payload per ADR-015). Contract already locked — no new ADR unless contract changes.

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P16-E1-T1** | Implement ambient light plugin | Dev | macos-collector + host | See `P16-E1-T1-pm-brief.md` | PM-GATE-POST-P15 |

### Epic P16-E2 — Ambient light Feature
**Goal:** Catalog Feature from ambient light Observations (ADR-007; calm framing).

| ID | Task | Role | Modules | AC | Depends |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P16-E2-T1** | AmbientLightShare catalog Feature | Dev | feature-engine + docs | Shaped by ADR-015 | P16-E1-T1 |

**Out of scope:** weather ambient; IDE; App Store packaging product; NotificationPressure; camera/scene capture; new SQLite migration; PR during freeze.

---

## Phase 15 archive (Done)

<details>
<summary>Phase 15 Kanban & epics (closed 2026-08-11 — companion HRV)</summary>

**Done:** P15-E1 (T1) · P15-E2 (T1) · P15-E3 (T1).  
ADR-016 HealthKit HRV + Auto-sync → Core SDNN-or-RMSSD → dogfood. ADR-015 ambient light parked → resumed in Phase 16.

Evidence: `docs/handoffs/P15-*-qa-to-pm.md` · branch `phase/15-companion-hrv-autonomy` (cluster PR after freeze).

</details>

---

## Queue (Phase 16)

1. **P16-E1-T1** — Ambient light plugin ← **Ready**  
2. P16-E2-T1 — AmbientLightShare catalog Feature  

**Git:** `phase/16-ambient-light` → local commits → **one cluster PR after 2026-09-01**.  
**Brief:** `docs/handoffs/P16-E1-T1-pm-brief.md`.  
**Deferred (later / other tracks):** IDE · weather ambient · App Store packaging · NotificationPressure.

### Parked after Phase 16 — Phase 17 intent (ADR-017)

**User lock (2026-08-11):** option **B** — finish ambient light first; then wearable + chart UX.

| Track | Intent (not Ready yet) |
| :--- | :--- |
| Wearable depth | Max Mi Fitness → HealthKit → Companion Observations (HR + steps/energy/sleep candidates; HRV soft-optional) |
| Dashboard ranges | Chart picker **1h / 8h / 12h / 1d / 1w** via recompute-on-read; Snapshot = latest + series chart |
| Analysis | Deterministic Features/Insights/Recommendations first; LLM interpret-only on top |

Evidence / detail: `docs/decision-log.md` ADR-017 · `docs/handoffs/PARKED-P17-wearable-dashboard-intent.md`.  
**Do not start P17 build** until P16 Done + follow-up contract ADR for new `data_type`s (if any).
