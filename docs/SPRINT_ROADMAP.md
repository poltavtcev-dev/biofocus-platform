# BioFocus — Sprint Roadmap & Kanban Matrix

> Active: **Phase 2: Ingestion & Context Collector** (Sprint 3–4) from `/docs/14-roadmap.md`.  
> Phase 1 Foundation: **Done** (2026-08-04). Out of scope until Phase 3: pipeline DAG, Features, dashboard, LLM.

**Phase 2 goal:** локальный приём `Observation` (HTTP ingest + pairing), macOS context collector, мост iOS/HealthKit → ingest. Ядро математики Features — **не** в этой фазе.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); ingest пишет через Core/`storage`
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; sleep/wake по событиям или редкому таймеру; при простое CPU ≈ idle OS
- [ ] Ветка → **PR → `main`** после зелёного CI (`docs/12-development.md`)

---

## Kanban Overview (Phase 2 = Sprint 3–4)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P2-E1-T3** (persist + mid-batch 503) |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** (E1–E4) · **P2-E1-T1** · **P2-E1-T2** (2026-08-04, Pass with notes) |

**Epic status:** P2-E1 ▶️ (T1 ✅ T2 ✅) · P2-E2 ○ · P2-E3 ○ · P2-E0 ○ (hygiene)

**Phase 2 on `/docs/14-roadmap.md`:** ☐ in progress (ingest + pairing on branch; **PR pending**)

**Рекомендуемый порядок:**  
`P2-E1-T3` → `T4` → `P2-E2-*` → `P2-E3-*`  
Параллельно: `P2-E0-T1` sanitize `dbError` (∥)

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Desktop-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`.

### Active assignment (PM → Dev)

**Task:** `P2-E1-T3` — Persist ingest → `ObservationRepository` (+ mid-batch contract)  
**Assignee:** Dev  
**Brief:** `docs/handoffs/P2-E1-T3-pm-brief.md`  
**Branch:** `phase/2-ingest-http` → **commit + PR urgently** (T1+T2 still uncommitted)  
**Carry notes:** host `IngestConfig::load()` → **T4**; mid-batch 503 → **T3** (this task).

---

## Epic P2-E0 — Host hygiene (carry-over)

### P2-E0-T1 — Sanitize IPC `dbError` paths
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `apps/desktop/src-tauri`, optionally `crates/storage` error mapping |
| **Depends on** | Phase 1 |
| **AC** | UI/`get_status` не показывает абсолютный filesystem path в `dbError`. Сообщение короткое и безопасное. Тест(ы) на mapping. |
| **Note** | Из E3-T4 follow-up. Можно ∥ после P2-E1-T1. |

---

## Epic P2-E1 — Local Ingest API

**Цель:** дверь для Observation — Axum на loopback, Bearer pairing, запись в SQLite через Core.

### P2-E1-T1 — Ingest HTTP skeleton
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | новый crate или `runtime`/`ingest` module; без Tauri UI |
| **Depends on** | Phase 1 (`bio-spec`, `runtime`) |
| **AC** | Сервер слушает **только** `127.0.0.1` (порт из конфига/константы). `POST /v1/ingest` принимает JSON array `Observation`, требует `Authorization: Bearer …` (токен пока из env/const для скелета). Успех → `202` + `{"status":"queued","count":N}`. Без токена / битый JSON → 4xx. Idle: после bind нет spin-loop (tokio accept). Unit/integration тест на bind + auth reject + 202. Нет Feature/pipeline. |
| **Out of scope** | Wi‑Fi LAN bind, QR pairing UI, iOS, macOS collector, persist (→ T3) |

### P2-E1-T2 — Pairing token persistence
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | storage or local config under `~/.biofocus/` |
| **Depends on** | P2-E1-T1 |
| **AC** | Токен генерируется при первом старте, хранится локально (не в git). Ingest отклоняет неверный Bearer. Документ: где лежит секрет (`docs/10-security.md` / `12-development`). Нет QR UI (можно позже E3). |

### P2-E1-T3 — Persist ingest → `ObservationRepository`
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | ingest path + `storage` + bounded channel |
| **Depends on** | P2-E1-T1, P1-E2-T3 |
| **AC** | Принятые Observation асинхронно пишутся через `ObservationRepository::insert` (immutable). Duplicate PK → явная ошибка/ответ без silent overwrite. Backpressure: bounded channel; при full — предсказуемый **503**. **Формализовать mid-batch:** если часть batch уже в канале, а дальше `queue_full` — зафиксировать контракт (напр. `202` с `count` + warning / или атомарный all-or-nothing / или `207`/`503` с `accepted`/`rejected` counts) в API + тестах (из T1 QA note). Тесты round-trip ingest→DB. |

### P2-E1-T4 — Host wire + `GET /v1/status` + idle DoD
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `apps/desktop/src-tauri`, ingest lifecycle |
| **Depends on** | P2-E1-T2, P2-E1-T3 |
| **AC** | Desktop стартует/останавливает ingest с приложением через `IngestConfig::load()` (pairing file / env из T2). `GET /v1/status` (local) отдаёт version + db ok\|error **без** Observation payload. Нет busy-loop в idle. UI по-прежнему только IPC для статуса shell (HTTP status — для companion/debug, не сырой биометрии). |

---

## Epic P2-E2 — macOS Context Collector

**Цель:** локальный контекст работы → Observation (`context_window` / агрегаты ввода). Не keylogger содержимого.

### P2-E2-T1 — Active window Observation stream
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `plugin-sdk` + macOS collector adapter |
| **Depends on** | P2-E1-T3 (куда писать) или channel→repo |
| **AC** | Смена активного приложения/окна → Observation с `data_type` согласованным со схемой (`docs/04-storage.md`). Polling/event interval не чаще разумного (напр. ≥1s или event-driven). Stop/pause останавливает работу. Idle без лишней нагрузки. |

### P2-E2-T2 — Keystroke / input aggregates (privacy-safe)
| Field | Value |
| :--- | :--- |
| **Role** | Dev (+ PM privacy check) |
| **Modules** | macOS collector |
| **Depends on** | P2-E2-T1 |
| **AC** | Только **агрегаты** (counts / rates в окне), **не** символы/текст. Документированы разрешения Accessibility. Отключение collector — одна настройка/флаг. |

### P2-E2-T3 — Collector integration tests + pause idle
| Field | Value |
| :--- | :--- |
| **Role** | QA (lead), Dev |
| **Modules** | collector + storage |
| **Depends on** | P2-E2-T1 (T2 если готов) |
| **AC** | Тесты на emit Observation / pause. Подтверждение: paused collector ≈ нет периодической работы. |

---

## Epic P2-E3 — iOS Companion → Ingest

**Цель:** HealthKit факты уходят на desktop `POST /v1/ingest` с pairing token. Не облако.

### P2-E3-T1 — Companion contract + minimal HealthKit sample path
| Field | Value |
| :--- | :--- |
| **Role** | Dev (iOS) |
| **Modules** | `apps/` companion (новое) или stub + docs |
| **Depends on** | P2-E1-T2, P2-E1-T3 |
| **AC** | Минимальный путь: sample `heart_rate` Observation → HTTP ingest на reachable host. Контракт body = array Observation. Ошибки сети/401 обработаны. Нет Feature math. |

### P2-E3-T2 — Pairing UX (token share)
| Field | Value |
| :--- | :--- |
| **Role** | UX + Dev |
| **Modules** | Desktop shell + companion |
| **Depends on** | P2-E1-T2, P2-E3-T1 |
| **AC** | Пользователь может перенести pairing token на телефон (QR или copy). Без облачного аккаунта. |

---

## Phase 1 archive (Done)

<details>
<summary>Phase 1 Kanban & epics (closed 2026-08-04)</summary>

**Done:** Epic E1–E4. Evidence: `docs/handoffs/P1-E4-T1-acceptance.md`, `P1-E4-T2-qa-to-pm.md`.

Epics: workspace/`bio-spec`/`runtime` → SQLite WAL + `ObservationRepository` → Tauri Menubar + `get_status` → exit docs.

</details>

---

## Role × Module Matrix (Phase 2)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P2-E0-T1 sanitize dbError | | ● | ○ | | desktop / storage |
| P2-E1-T1 ingest HTTP skeleton | ○ | ● | ○ | | runtime / ingest |
| P2-E1-T2 pairing token | | ● | ○ | | ~/.biofocus config |
| P2-E1-T3 persist Observations | | ● | ○ | | storage + channel |
| P2-E1-T4 host wire + status | | ● | ○ | | desktop + ingest |
| P2-E2-T1 active window | | ● | | | plugin-sdk / collector |
| P2-E2-T2 keystroke aggregates | ● | ● | | | collector |
| P2-E2-T3 collector tests | | ○ | ● | | collector |
| P2-E3-T1 HealthKit → ingest | | ● | ○ | | iOS companion |
| P2-E3-T2 pairing UX | ○ | ○ | | ● | desktop + iOS |

● = owner · ○ = collaborator

---

## Sprint 3 — Ready Now

1. **P2-E1-T3** — Persist + mid-batch 503 ← **берите сейчас**  
2. Затем **P2-E1-T4** (host `IngestConfig::load`)  
3. Опционально ∥ **P2-E0-T1** sanitize `dbError`  
4. ~~T1~~ ~~T2~~ Done — **сначала commit+PR `phase/2-ingest-http`**
