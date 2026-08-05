# BioFocus — Sprint Roadmap & Kanban Matrix

> Active: **Phase 4: Dashboard UI & Local AI Insights** (Sprint 7–8) from `/docs/14-roadmap.md`.  
> Phase 1 Foundation · Phase 2 Ingestion · Phase 3 Pipeline & Features: **Done**.  
> Out of scope until later / ADR: LAN ingest, wearable companion bridges, **new SQLite tables** (Insights stay derived/in-memory in Phase 4 v1).

**Phase 4 goal:** Local React Dashboard (Recharts) over Feature snapshots via IPC + deterministic Knowledge Insights + optional local LLM reports (Ollama / OpenAI-compatible). LLM **interprets** only — never computes Features.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight`)
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); Features/Insights считаются в Core
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; poll/refresh по событию или редкому таймеру
- [ ] **Нет новой SQLite-схемы** без ADR + approve (Insights/reports — in-memory / derived)
- [ ] **Commit / PR по связанному кластеру** — `docs/12-development.md`
- [ ] Copy спокойный, неоценочный (не «ты выгорел» / clinical claims)

---

## Kanban Overview (Phase 4 = Sprint 7–8)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P4-E2-T3** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3; Menubar via [PR #24](https://github.com/poltavtcev-dev/biofocus-platform/pull/24)) · **P4-E1** (T1–T3) · **P4-E2-T1** · **P4-E2-T2** |

**Epic status:** P4-E1 ✅ · P4-E2 ◐ · P4-E3 ⬜

**Phase 4 on `/docs/14-roadmap.md`:** opened 2026-08-05

**Рекомендуемый порядок:**  
~~P4-E1~~ → ~~P4-E2-T1~~ → ~~T2~~ → **T3** → P4-E3-T1 → T2 → T3

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Desktop-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **related work → PR** (`docs/12-development.md`).  
**Suggested branch:** `phase/4-dashboard-ai`

### Active assignment

**Ready now:** **P4-E2-T3** — Insights IPC + Dashboard list — role **Dev + UX**.  
Brief: `docs/handoffs/P4-E2-T3-pm-brief.md`.

**Closed:** P4-E2-T2 (QA Pass, 2026-08-05) — `register_insights_v1` (`high_stress_period_v1` + `context_switch_elevated_v1`); IPC/UI → T3.

---

## Epic P4-E1 — Feature IPC + Dashboard charts

**Цель:** UI получает windowed Feature snapshot только через IPC; Recharts dashboard без LLM.

### P4-E1-T1 — Feature snapshot API + IPC
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `runtime` / `feature-engine`, `apps/desktop/src-tauri`, optionally `bio-spec` DTO |
| **Depends on** | Phase 3 (Feature Worker + DAG + AlertLevel) |
| **AC** | Публичный Core API отдаёт недавний snapshot Features (+ optional Signals) с `provenance` / ids; отдельная IPC-команда (предпочтительно `get_feature_snapshot`, не раздувать `get_status`); payload без raw Observation biometrics / absolute paths; unit-тесты на empty + non-empty; idle-safe (pure read / cached snapshot). Нет Recharts, нет LLM, нет новой SQLite-таблицы. Документировать контракт в handoff + кратко в `docs/09-api.md` / `12-development.md`. |
| **Out of scope** | Dashboard UI (→ T2/T3), Insights (→ E2), LLM (→ E3) |

### P4-E1-T2 — Dashboard shell (window / route)
| Field | Value |
| :--- | :--- |
| **Role** | UX + Dev |
| **Modules** | `apps/desktop` (React + Tauri) |
| **Depends on** | P4-E1-T1 (IPC may be stubbed with mocks for UX layout if documented) |
| **AC** | Открываемый Dashboard view из Menubar (отдельное окно или route); спокойный shell: loading / empty / error; данные только через IPC; Menubar alert UX не ломается. Нет Recharts series ещё (layout slots / empty chart area OK). Нет evaluative copy. Handoff с manual smoke. |
| **Out of scope** | Полные charts (→ T3), Insights list (→ E2-T3), LLM report UI (→ E3-T3) |

### P4-E1-T3 — Recharts Feature series
| Field | Value |
| :--- | :--- |
| **Role** | UX (+ Dev IPC glue if needed) |
| **Modules** | `apps/desktop` (+ Recharts) |
| **Depends on** | P4-E1-T1, P4-E1-T2 |
| **AC** | Графики v1 по snapshot: как минимум `FocusScore`, `StressIndex`, `FatigueIndex` (и `ContextSwitchRate` если есть в snapshot); спокойные labels/units; refresh on open или редкий timer (нет busy-loop); нет medical claims. Smoke + typecheck. |
| **Out of scope** | Insight cards, LLM, persistence |

---

## Epic P4-E2 — Knowledge Insights (deterministic)

**Цель:** Rule-based `Insight` + Evidence из Features/Signals. LLM не участвует в генерации метрик и не обязателен для Insights.

### P4-E2-T1 — knowledge-engine skeleton + Insight types
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/knowledge-engine`, `bio-spec` |
| **Depends on** | P4-E1-T1 (Feature snapshot shape helpful) |
| **AC** | Crate больше не stub-only: типы `Insight` / Evidence (ids Feature/Signal); API `Features + Signals → Result<Vec<Insight>>` (может вернуть empty); `thiserror`; unit-тест happy path + empty. Нет SQLite, нет UI, нет LLM. |
| **Out of scope** | Реальные product rules (→ T2), IPC/UI (→ T3) |

### P4-E2-T2 — Rule Insights v1
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `knowledge-engine` |
| **Depends on** | P4-E2-T1 |
| **AC** | ≥2 детерминированных rules (напр. `High_Stress` Signal → Insight; elevated Focus/ContextSwitch pattern); `evidence_list` ссылается на Feature/Signal ids; copy спокойный; unit-тесты на trigger + no-trigger. |
| **Out of scope** | Persistence ADR, LLM rewrite of Insights |

### P4-E2-T3 — Insights IPC + Dashboard list
| Field | Value |
| :--- | :--- |
| **Role** | Dev + UX |
| **Modules** | desktop host + React Dashboard |
| **Depends on** | P4-E2-T2, P4-E1-T2 |
| **AC** | IPC `get_insights` (или эквивалент); Dashboard показывает список Insights + evidence refs; UI↛DB; без новой SQLite-схемы. Handoff + smoke. |
| **Out of scope** | Report/LLM (→ E3) |

---

## Epic P4-E3 — Reports & optional local LLM

**Цель:** Детерминированный report/prompt builder + opt-in local LLM (Ollama / OpenAI-compatible). Default OFF; данные не уходят без явного user action.

### P4-E3-T1 — report-engine prompt / markdown builder
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/report-engine` |
| **Depends on** | P4-E2-T1 (Insights shape); Features from E1 |
| **AC** | Crate API: Features (+ Insights) → deterministic markdown и/или LLM prompt string; offline; unit-тесты; нет network calls. Документировать формат. |
| **Out of scope** | HTTP к Ollama/OpenAI (→ T2), Dashboard button (→ T3) |

### P4-E3-T2 — Optional local LLM adapter
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `report-engine` and/or desktop host |
| **Depends on** | P4-E3-T1 |
| **AC** | Opt-in (env/flag/config); default **OFF**; предпочтительно localhost Ollama / OpenAI-compatible base URL; `Result` errors; timeout; never auto-send on startup; docs privacy note в `12-development.md`. Нет Feature math в LLM. |
| **Out of scope** | Cloud account UX, mandatory AI |

### P4-E3-T3 — Report UX in Dashboard
| Field | Value |
| :--- | :--- |
| **Role** | UX + Dev |
| **Modules** | `apps/desktop` |
| **Depends on** | P4-E3-T1 (T2 optional for LLM path) |
| **AC** | Кнопка/flow «Generate report»: показывает deterministic markdown/prompt; если LLM enabled — optional local output; явный copy «local / optional AI»; спокойный тон; IPC-only. Smoke steps в handoff. |
| **Out of scope** | Insight persistence, LAN ingest |

---

## Phase 3 archive (Done)

<details>
<summary>Phase 3 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P3-E1 (T1–T4) · P3-E2 (T1–T4) · P3-E3 (T1–T3).  
Pipeline quality → Feature DAG → Menubar AlertLevel IPC + UX.

Evidence: `docs/handoffs/P3-*-qa-to-pm.md` · PRs #5–#23 (cluster) · Menubar T3 may land via follow-up PR if not yet on `main`.

</details>

## Phase 2 archive (Done)

<details>
<summary>Phase 2 Kanban & epics (closed 2026-08-04, PR #2)</summary>

**Done:** P2-E0 · P2-E1 (T1–T4) · P2-E2 (T1–T3) · P2-E3 (T1–T2).  
Evidence: `docs/handoffs/P2-*-qa-to-pm.md` · [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2).

Ingest loopback + pairing · macOS collector · companion sample + Copy/QR · `dbError` sanitize.

</details>

## Phase 1 archive (Done)

<details>
<summary>Phase 1 Kanban & epics (closed 2026-08-04)</summary>

**Done:** Epic E1–E4. Evidence: `docs/handoffs/P1-E4-T1-acceptance.md`, `P1-E4-T2-qa-to-pm.md`.

Epics: workspace/`bio-spec`/`runtime` → SQLite WAL + `ObservationRepository` → Tauri Menubar + `get_status` → exit docs.

</details>

---

## Role × Module Matrix (Phase 4)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P4-E1-T1 Feature snapshot IPC | ○ | ● | ○ | | runtime / feature-engine + src-tauri |
| P4-E1-T2 Dashboard shell | | ○ | ○ | ● | apps/desktop |
| P4-E1-T3 Recharts series | | ○ | ○ | ● | apps/desktop |
| P4-E2-T1 knowledge-engine skeleton | | ● | ○ | | knowledge-engine + bio-spec |
| P4-E2-T2 Rule Insights v1 | | ● | ○ | | knowledge-engine |
| P4-E2-T3 Insights IPC + list UI | | ● | ○ | ● | desktop + knowledge-engine |
| P4-E3-T1 report-engine builder | | ● | ○ | | report-engine |
| P4-E3-T2 Optional local LLM | | ● | ○ | | report-engine / host |
| P4-E3-T3 Report UX | ○ | ○ | | ● | apps/desktop |

● = owner · ○ = collaborator

---

## Sprint 7 — Ready Now

1. ~~P4-E1-T1 — Feature snapshot API + IPC~~ **Done**  
2. ~~P4-E1-T2 — Dashboard shell~~ **Done**  
3. ~~P4-E1-T3 — Recharts Feature series~~ **Done** (QA Pass with notes) — Epic **P4-E1** ✅  

## Sprint 8 (after E1)

4. ~~P4-E2-T1 — knowledge-engine + Insight types~~ **Done**  
5. ~~P4-E2-T2 — Rule Insights v1~~ **Done**  
6. **P4-E2-T3** — Insights IPC + Dashboard list ← **Ready (Dev + UX)**  
7. P4-E3-T1 — report-engine builder  
8. P4-E3-T2 — Optional local LLM adapter  
9. P4-E3-T3 — Report UX  

**Git:** `phase/4-dashboard-ai` → related commits → PR when cluster ready.

**Wearables (later):** phone companion bridges + LAN ingest; HealthKit/CLI sample stays the contract path for now — see `docs/PROJECT_CANVAS.md` § Wearable / companion.
