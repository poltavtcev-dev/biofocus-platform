# BioFocus — Sprint Roadmap & Kanban Matrix

> Technical PM decomposition of **Phase 1: Foundation** from `/docs/14-roadmap.md`.  
> Out of scope here: Phase 2+ (ingest HTTP, collectors, iOS, pipeline DAG, dashboard, LLM).

**Phase goal:** поднятый Cargo workspace, канонические типы, SQLite WAL + repositories, минимальный Tauri Menubar shell с IPC-границей.

**Global DoD (каждая задача):**
- [ ] Соответствует freeze в `/docs/ARCHITECTURE_STATUS.md`
- [ ] Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight`)
- [ ] Нет `unwrap()` / `expect()` в production-путях
- [ ] UI не обращается к SQLite напрямую
- [ ] Релевантные тесты зелёные; `cargo check` по workspace проходит

---

## Kanban Overview (Phase 1 = Sprint 1–2)

| Status | IDs |
| :--- | :--- |
| **Ready** | — *(next: PM Phase 2 decomposition — no code)* |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Epic E1** · **Epic E2** · **Epic E3** · **Epic E4** (T1 2026-08-03 · **T2 2026-08-04**) |

**Epic status:** E1 ✅ · E2 ✅ · E3 ✅ · E4 ✅

**Phase 1 on `/docs/14-roadmap.md`:** ✅ Done (user approve 2026-08-03) · Sprint Kanban fully closed 2026-08-04

**Рекомендуемый порядок:**  
Phase 1 closed → push CI → PM decomposes Phase 2 (ingest / collectors; idle footprint constraint; no code until assigned)

**Live board:** open the Cursor Canvas [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Desktop-BioFocus/canvases/biofocus-execution-board.canvas.tsx) beside chat.

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`.

### Active assignment

**None** — Phase 1 complete.  
**Close note:** `docs/handoffs/P1-E4-T2-qa-to-pm.md`

---

## Epic P1-E1 — Cargo Workspace & Spec Crate

**Цель:** воспроизводимый monorepo skeleton и канонические типы из контрактов.

### P1-E1-T1 — Bootstrap Cargo workspace
| Field | Value |
| :--- | :--- |
| **Role** | Dev (lead), PM (scope check) |
| **Modules** | root `Cargo.toml`, `crates/*` stubs |
| **Depends on** | — |
| **AC** | Корневой workspace включает members: как минимум `bio-spec`, `runtime`, `storage`. `cargo check` на пустых/stub crates проходит. Структура совпадает с `/docs/13-project-structure.md` (без обязательного создания всех Phase 2+ crates в Sprint 1 — допускается stub-only для остальных). |

### P1-E1-T2 — `bio-spec`: domain types & serialization
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/bio-spec` |
| **Depends on** | P1-E1-T1 |
| **AC** | Типы `Observation`, `Signal`, `Feature`, `Insight` (минимальные поля по `/docs/02-domain-model.md` + `/docs/07-contracts.md`). Serde JSON round-trip для sample Observation из контракта. UUIDv7 / timestamp UTC задокументированы в API типов. Публичный API crate стабилен для `storage`/`runtime`. |

### P1-E1-T3 — `runtime`: Tokio host skeleton
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/runtime` |
| **Depends on** | P1-E1-T1, P1-E1-T2 |
| **AC** | Crate поднимает multi-thread Tokio runtime; есть bounded channel placeholder для будущих Observation; `tracing` subscriber инициализируется. Нет зависимости от Tauri. Нет ingest HTTP (Phase 2). |

### P1-E1-T4 — Workspace hygiene & LICENSE
| Field | Value |
| :--- | :--- |
| **Role** | PM + Dev |
| **Modules** | root, `.gitignore`, `LICENSE` (AGPLv3) |
| **Depends on** | P1-E1-T1 |
| **AC** | `LICENSE` = AGPLv3 (ADR-004). `.gitignore` покрывает `target/`, Node/Tauri артефакты. README или `docs/12-development.md` отражает команды `cargo check` / будущий `pnpm tauri dev`. |

---

## Epic P1-E2 — Storage: SQLite WAL & Repositories

**Цель:** локальная БД и репозиторий Observations по схеме `/docs/04-storage.md`.

### P1-E2-T1 — DB open + WAL pragmas
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/storage` |
| **Depends on** | P1-E1-T1 |
| **AC** | Открытие DB по пути `~/.biofocus/data/biofocus_main.db` (или test path). Pragmas: `journal_mode=WAL`, `synchronous=NORMAL`. Ошибки — через typed `Result`, без panic. |

### P1-E2-T2 — Migration: `observations` table
| Field | Value |
| :--- | :--- |
| **Role** | Dev, QA (review) |
| **Modules** | `crates/storage` (migrations) |
| **Depends on** | P1-E2-T1 |
| **AC** | Миграция создаёт таблицу и индексы из `/docs/04-storage.md`. Повторный запуск идемпотентен (`IF NOT EXISTS` / versioned migrations). Schema change только через будущий ADR — текущая схема freeze. |

### P1-E2-T3 — `ObservationRepository`
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/storage`, depends on `bio-spec` |
| **Depends on** | P1-E2-T2, P1-E1-T2 |
| **AC** | `insert` (immutable), `get_by_id`, `list_by_time_range` / `list_by_data_type`. Insert дубликата PK → явная ошибка (не silent overwrite). Payload хранится как JSON. |

### P1-E2-T4 — Storage integration tests
| Field | Value |
| :--- | :--- |
| **Role** | QA (lead), Dev |
| **Modules** | `crates/storage` tests (in-memory / temp dir) |
| **Depends on** | P1-E2-T3 |
| **AC** | Тесты: миграция, insert+read round-trip, duplicate id, range query. Битый JSON / invalid confidence обрабатываются предсказуемо. Соответствует слою Integration из `/docs/11-testing.md`. |

---

## Epic P1-E3 — Tauri v2 Menubar Shell

**Цель:** desktop-процесс с Menubar и IPC-границей к Core (без dashboard).

### P1-E3-T1 — Scaffold `apps/desktop` (Tauri v2 + React)
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `apps/desktop`, `apps/desktop/src-tauri` |
| **Depends on** | P1-E1-T1 (желательно P1-E1-T3) |
| **AC** | `pnpm install` + `pnpm tauri build`/`dev` поднимают окно/tray. `src-tauri` зависит от workspace crates (`runtime`/`storage` stub wiring допустим). Нет прямых SQL-вызовов из frontend. |

### P1-E3-T2 — Menubar status UX (minimal)
| Field | Value |
| :--- | :--- |
| **Role** | UX (lead), Dev (wire) |
| **Modules** | `apps/desktop/src/**` |
| **Depends on** | P1-E3-T1 |
| **AC** | Menubar/tray показывает нейтральный статус Core (например Idle / Ready / Error). Без графиков, без dashboard widgets (Phase 4). Состояния читаемы по HIG; нет оценочных формулировок («ты выгорел»). |

### P1-E3-T3 — IPC: `get_status` command
| Field | Value |
| :--- | :--- |
| **Role** | Dev, UX (consumer) |
| **Modules** | `apps/desktop/src-tauri`, frontend invoke |
| **Depends on** | P1-E3-T1, P1-E1-T3 |
| **AC** | Команда IPC возвращает версию/статус DB (ok|error) без сырых Observation. UI обновляет Menubar только через IPC. Контракт задокументирован кратко в коде или `docs/09-api.md` (internal IPC note). |

### P1-E3-T4 — Smoke: UI ↔ Core boundary
| Field | Value |
| :--- | :--- |
| **Role** | QA, UX |
| **Modules** | `apps/desktop` |
| **Depends on** | P1-E3-T2, P1-E3-T3 |
| **AC** | Чеклист: Menubar виден; статус меняется при mock error; в frontend-бандле нет `rusqlite`/прямых путей к `biofocus_main.db`. |

---

## Epic P1-E4 — Phase 1 Exit & Handoff

### P1-E4-T1 — Phase 1 acceptance checklist
| Field | Value |
| :--- | :--- |
| **Role** | PM (lead), QA |
| **Modules** | docs / CI notes |
| **Depends on** | P1-E1…P1-E3 |
| **AC** | Чеклист Phase 1 закрыт: workspace check, storage tests, Menubar smoke. Обновлён статус в `/docs/14-roadmap.md` (Phase 1 → done) только после явного approve. Зафиксирован список Ready для Phase 2 без старта реализации Phase 2. |

### P1-E4-T2 — Dev environment doc sync
| Field | Value |
| :--- | :--- |
| **Role** | PM, Dev |
| **Modules** | `docs/12-development.md` |
| **Depends on** | P1-E1-T1, P1-E3-T1 |
| **AC** | Команды установки и проверки актуальны (`cargo check`, `cargo test -p storage`, `pnpm tauri dev`). Нет ссылок на несуществующие скрипты. |

---

## Role × Module Matrix (Phase 1)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P1-E1-T1 Workspace bootstrap | ○ | ● | | | root, crates stubs |
| P1-E1-T2 bio-spec types | | ● | ○ | | `bio-spec` |
| P1-E1-T3 runtime skeleton | | ● | | | `runtime` |
| P1-E1-T4 LICENSE / hygiene | ● | ● | | | root |
| P1-E2-T1 WAL open | | ● | | | `storage` |
| P1-E2-T2 migrations | | ● | ○ | | `storage` |
| P1-E2-T3 repository | | ● | | | `storage`, `bio-spec` |
| P1-E2-T4 storage tests | ○ | ○ | ● | | `storage` |
| P1-E3-T1 Tauri scaffold | | ● | | ○ | `apps/desktop` |
| P1-E3-T2 Menubar UX | | ○ | | ● | `apps/desktop/src` |
| P1-E3-T3 IPC get_status | | ● | | ○ | `src-tauri` + UI |
| P1-E3-T4 boundary smoke | ○ | | ● | ● | `apps/desktop` |
| P1-E4-T1 Phase exit | ● | | ● | | docs |
| P1-E4-T2 Dev guide sync | ● | ● | | | `docs/12-development.md` |

● = owner · ○ = collaborator

---

## Sprint 1 — Ready Now (take first)

1. **P1-E1-T1** — Bootstrap Cargo workspace  
2. **P1-E1-T2** — `bio-spec` domain types  
3. **P1-E1-T4** — LICENSE (AGPLv3) + hygiene *(можно параллельно с T2)*  
4. **P1-E1-T3** — `runtime` skeleton  
5. **P1-E2-T1** — SQLite WAL open  

Следующий batch (конец Sprint 1 / Sprint 2): `P1-E2-T2` → `T3` → `T4`, затем `P1-E3-*`.
