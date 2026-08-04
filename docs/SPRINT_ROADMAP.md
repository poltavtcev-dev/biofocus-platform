# BioFocus — Sprint Roadmap & Kanban Matrix

> Active: **Phase 3: Pipeline & Features** (Sprint 5–6) from `/docs/14-roadmap.md`.  
> Phase 1 Foundation · Phase 2 Ingestion: **Done** (PR #2, 2026-08-04).  
> Out of scope until Phase 4: React dashboard (Recharts), local LLM / Ollama reports, Knowledge Insights persistence.

**Phase 3 goal:** детерминированный pipeline `Observation → (dedupe / normalize) → Feature` + transient `Signal` + Menubar alert levels (🟢/🟡/🔴). Математика Features — алгоритмическая; LLM не участвует.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature`)
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); Features/Signals считаются в Core
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; sleep/wake по событиям или редкому таймеру; при простое CPU ≈ idle OS
- [ ] **Нет новой SQLite-схемы** без ADR + approve (Features/Signals — in-memory / derived, пока не утверждено иное)
- [ ] **Commit / PR по связанному кластеру** — `docs/12-development.md`

---

## Kanban Overview (Phase 3 = Sprint 5–6)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P3-E3-T1** (next) |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** (E1–E4) · **Phase 2** (E0–E3, [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **P3-E1-T1** ([PR #5](https://github.com/poltavtcev-dev/biofocus-platform/pull/5)/[#6](https://github.com/poltavtcev-dev/biofocus-platform/pull/6)) · **P3-E1-T2** ([PR #8](https://github.com/poltavtcev-dev/biofocus-platform/pull/8) code · [PR #9](https://github.com/poltavtcev-dev/biofocus-platform/pull/9) QA handoff) · **P3-E1-T3** ([PR #11](https://github.com/poltavtcev-dev/biofocus-platform/pull/11) code · QA Pass with notes) · **P3-E1-T4** ([PR #13](https://github.com/poltavtcev-dev/biofocus-platform/pull/13) code · QA Pass with notes) · **P3-E2-T1** (QA Pass) · **P3-E2-T2** (QA Pass with notes) · **P3-E2-T3** (QA Pass with notes; [PR #20](https://github.com/poltavtcev-dev/biofocus-platform/pull/20)/[#21](https://github.com/poltavtcev-dev/biofocus-platform/pull/21)) · **P3-E2-T4** (QA Pass; pipeline E2E) |

**Epic status:** P3-E1 ✅ (T1–T4 Done) · P3-E2 ✅ (T1–T4 Done) · P3-E3 ⬜

**Phase 3 on `/docs/14-roadmap.md`:** ☐ open (Sprint 5–6)

**Рекомендуемый порядок:**  
~~P3-E1-T1~~ → ~~E1-T2~~ → ~~E1-T3~~ → ~~E1-T4~~ → ~~E2-T1~~ → ~~E2-T2~~ → ~~E2-T3~~ → ~~E2-T4~~ → **E3-T1** → E3-T2 → E3-T3 → cluster PR

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Desktop-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **related work → PR** (`docs/12-development.md`).  
**Suggested branch:** `phase/3-pipeline-features` (или `epic/p3-e3-menubar-alerts` для E3).

### Active assignment (PM → Dev)

**Task:** **P3-E3-T1** — Alert level mapping (Core)  
**Brief:** (see AC in this file · Epic P3-E3)  
**Role:** Dev · **Modules:** `feature-engine` or `runtime` alert module

---

## Epic P3-E1 — Pipeline (Quality → Dedupe → Normalize)

**Цель:** превратить поток/батч `Observation` в чистый нормализованный вход для Feature Engine. Без Feature math в E1 (кроме заглушки вызова на T4).

### P3-E1-T1 — Pipeline crate skeleton + Observation intake
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/pipeline`, optionally `bio-spec` helpers |
| **Depends on** | Phase 2 (Observations in SQLite / channel) |
| **AC** | Crate `pipeline` больше не stub-only: публичный API принимает batch/`&[Observation]` (или iterator) и возвращает `Result` со стадией «accepted for processing» / structured error (`thiserror`). Unit-тесты на happy path + пустой batch. Нет busy-loop. Нет Feature formulas, нет Menubar, нет новой SQLite-таблицы. Документировать entrypoint в handoff. |
| **Out of scope** | Dedupe logic (→ T2), normalize (→ T3), runtime worker wire (→ T4), Feature DAG |

### P3-E1-T2 — Deduplication stage
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/pipeline` |
| **Depends on** | P3-E1-T1 |
| **AC** | Stage удаляет/помечает дубликаты по явному правилу (документировать: напр. same `id` уже seen в окне / same `(provider_id, data_type, timestamp, payload hash)`). Immutable Observations в БД не переписываются. Тесты: duplicate in-batch + cross-batch (in-memory seen-set). Idle-safe. |

### P3-E1-T3 — Normalization & calibration
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/pipeline` |
| **Depends on** | P3-E1-T2 |
| **AC** | Stage приводит payload/единицы к канону для известных `data_type` (`heart_rate`, `hrv`, `context_window`, input aggregates) — явные правила в коде + тестах; неизвестный type → pass-through или явный skip (зафиксировать). Нет Feature scores. |

### P3-E1-T4 — Runtime Feature Worker wire (idle-safe)
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/runtime`, `pipeline`, host/`apps/desktop/src-tauri` |
| **Depends on** | P3-E1-T3 |
| **AC** | Worker в Core периодически или по событию читает новые Observations → прогоняет pipeline stages. При отсутствии новых данных — sleep/event wait (нет spin). Desktop host стартует/останавливает worker с приложением. Заглушка/hook к feature-engine допустима (полный DAG → E2). Тест на idle freeze после stop. |

---

## Epic P3-E2 — Feature Engine (DAG)

**Цель:** детерминированные Features по `/docs/06-feature-catalog.md` + provenance. Signals для порогов (напр. High_Stress).

### P3-E2-T1 — DAG scheduler skeleton
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/feature-engine` |
| **Depends on** | P3-E1-T1 (лучше после E1-T4) |
| **AC** | `feature-engine` API: зарегистрировать узлы DAG, топологический прогон на in-memory snapshot нормализованных Observations → `Vec<Feature>` (+ optional `Vec<Signal>`). Ошибки через `Result`/`thiserror`. Unit-тест на 2-node DAG order. Нет UI. |

### P3-E2-T2 — `ContextSwitchRate` + `FocusScore` (v1)
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `feature-engine` |
| **Depends on** | P3-E2-T1 |
| **AC** | Реализованы v1 по catalog: `FocusScore` window 15m / step 1m (допустимы упрощения inputs, если задокументированы); `ContextSwitchRate` из `context_window`. Output + `provenance` Observation IDs. Unit-тесты на синтетических данных (`docs/11-testing.md`). |

### P3-E2-T3 — `StressIndex` + `FatigueIndex` (v1) + High_Stress Signal
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `feature-engine`, `bio-spec` |
| **Depends on** | P3-E2-T2 |
| **AC** | `StressIndex` / `FatigueIndex` v1 (упрощения OK если documented). Порог catalog: StressIndex > 75 дольше 5 мин → transient `Signal` type `High_Stress` (severity согласован с `bio-spec::Severity`). Тесты на порог. |

### P3-E2-T4 — Pipeline E2E (Observation → Feature / Signal)
| Field | Value |
| :--- | :--- |
| **Role** | QA (lead), Dev |
| **Modules** | pipeline + feature-engine (+ storage fixtures) |
| **Depends on** | P3-E2-T3, P3-E1-T3 |
| **AC** | E2E/integration: mock Observation stream/file → pipeline → features/signals с проверяемыми значениями (`docs/11-testing.md` §5). Документ: как гонять suite в `docs/12-development.md`. |

---

## Epic P3-E3 — Real-time Menubar alerts

**Цель:** уровень алерта из Core → Menubar 🟢/🟡/🔴 без dashboard и без evaluative copy («ты выгорел»).

### P3-E3-T1 — Alert level mapping (Core)
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `feature-engine` or `runtime` alert module |
| **Depends on** | P3-E2-T3 |
| **AC** | Детерминированный map Features/Signals → `AlertLevel` { Green, Yellow, Red } (имена в коде — snake/Pascal OK). Правила документированы; unit-тесты. Нет Tauri UI в этом task. |

### P3-E3-T2 — IPC expose alert level
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `apps/desktop/src-tauri`, IPC `get_status` (или соседнее поле) |
| **Depends on** | P3-E3-T1 |
| **AC** | UI получает alert level только через IPC. Расширение статуса не ломает Idle/Ready/Error. Нет absolute paths / биометрии в payload. Тест mapping на host side если уместно. |

### P3-E3-T3 — Menubar traffic-light UX
| Field | Value |
| :--- | :--- |
| **Role** | UX + Dev |
| **Modules** | desktop React/Menubar |
| **Depends on** | P3-E3-T2 |
| **AC** | Menubar/tray отражает 🟢/🟡/🔴 (цвет иконки/индикатора) по IPC. Copy спокойный, неоценочный. Нет charts/dashboard. Handoff с manual smoke steps. |

---

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

## Role × Module Matrix (Phase 3)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P3-E1-T1 pipeline skeleton | ○ | ● | ○ | | pipeline |
| P3-E1-T2 dedupe | | ● | ○ | | pipeline |
| P3-E1-T3 normalize | | ● | ○ | | pipeline |
| P3-E1-T4 worker wire | | ● | ○ | | runtime + desktop |
| P3-E2-T1 DAG skeleton | | ● | ○ | | feature-engine |
| P3-E2-T2 FocusScore v1 | | ● | ○ | | feature-engine |
| P3-E2-T3 Stress/Fatigue + Signal | | ● | ○ | | feature-engine |
| P3-E2-T4 pipeline E2E | | ○ | ● | | pipeline + feature-engine |
| P3-E3-T1 alert mapping | | ● | ○ | | runtime / feature-engine |
| P3-E3-T2 IPC alert | | ● | ○ | | desktop src-tauri |
| P3-E3-T3 Menubar lights | ○ | ○ | | ● | desktop UI |

● = owner · ○ = collaborator

---

## Sprint 5 — Ready Now

1. ~~P3-E1-T1~~ — Done (PR #5 + #6)  
2. ~~P3-E1-T2~~ — Done (QA Pass with notes; PR #8 + #9)  
3. ~~P3-E1-T3~~ — Done (QA Pass with notes; PR #11 code)  
4. ~~P3-E1-T4~~ — Done (QA Pass with notes; PR #13 code)  
5. ~~P3-E2-T1~~ — Done (QA Pass; DAG skeleton)  
6. ~~P3-E2-T2~~ — Done (QA Pass with notes; CSR + FocusScore v1)  
7. ~~P3-E2-T3~~ — Done (QA Pass with notes; Stress/Fatigue + High_Stress — PR #20/#21)  
8. ~~P3-E2-T4~~ — Done (QA Pass; pipeline E2E Observation → Feature/Signal)  

**Next:** **P3-E3-T1** — Alert level mapping (Core) ← **start here**  
**Git:** branch `phase/3-pipeline-features` → E2-T4 code PR; then E3 cluster.
