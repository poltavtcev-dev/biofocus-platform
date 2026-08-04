# BioFocus — Project Canvas

> One-page snapshot. Sources: `/docs/00–16`, `ARCHITECTURE_STATUS.md`, `decision-log.md`.  
> **Status:** MVP 1.0 Architecture Frozen · **License:** AGPLv3 (ADR-004)

---

## Problem

Знаниеработники не видят связь между **физиологией** (пульс, HRV, восстановление) и **реальным рабочим контекстом** (приложения, ввод, созвоны). Существующие трекеры либо облачные и непрозрачные, либо дают метрики без доказательной цепочки «факт → вывод».

## Value Proposition

**BioFocus** — Local-First open-source платформа, которая на устройстве пользователя:

1. Собирает биометрию и контекст работы как неизменяемые факты (`Observation`).
2. Детерминированно считает метрики фокуса / стресса / усталости (`Feature`).
3. Выдаёт объяснимые инсайты с Evidence (`Insight`) — опционально через LLM только как интерпретатор, не как движок метрик.

**Для кого:** персональный самоконтроль ритма работы (не меддиагностика, не employee surveillance, не облако по умолчанию).

### Owner dogfood stack

| Piece | Device |
| :--- | :--- |
| Wearable | **Xiaomi Mi Band 8** |
| Phone | **iPhone 12 mini** |
| Desktop | **MacBook Pro 14" (M1 Pro)** |

**Primary biometric source to support next:** Mi Band 8 (owner’s only wearable).

- Today’s companion path is an **Apple HealthKit / CLI sample** — useful as the HTTP contract, **not** the owner’s daily Band 8 source.
- Future path (realistic for this stack): Band 8 ↔ phone companion (Mi Fitness / Zepp / Health export or BLE bridge on **iPhone 12 mini**) → Desktop ingest on **M1 Pro MacBook** → Core Features → Menubar.
- Practical deps: **LAN-reachable ingest** on the Mac (today bind is loopback-only — phone on Wi‑Fi cannot reach `127.0.0.1` on the Mac); then a **Mi Band 8 → Observation** bridge with stable `provider_id` (e.g. `com.xiaomi.miband` / Zepp — adapter TBD). Menubar 🟢/🟡/🔴 remains **P3-E3**.
- Constraint: on iOS, continuous third-party BLE to Mi Band is harder than Android; prefer documented viable bridges (Health/export/share) over assuming raw BLE in App Store companion.

Until that lands, dogfood on the Mac: `context_window` / optional `keystrokes` + CLI sample HR into ingest.

---

## Key Architectural Decisions

| Decision | Choice | Why (ADR) |
| :--- | :--- | :--- |
| Topology | **Modular Monolith** (Rust crates + event pipeline) | Низкий RAM, безопасность памяти (ADR-001) |
| Data residency | **Local-First / Privacy-First** | Вся БД и compute на устройстве |
| Storage | **SQLite WAL** (`~/.biofocus/data/…`) | Без внешних серверов (ADR-002) |
| UI shell | **Tauri v2 + React** | Компактный дистрибутив (ADR-003) |
| AI stance | **AI-Optional** | Метрики алгоритмические; LLM — отчёты/NL |
| Extensibility | **Capability Plugin Model** | Новые источники через явные Capabilities |
| License | **AGPLv3** | Защита от закрытых коммерческих форков (ADR-004) |

**Границы IPC:** `Desktop UI` ↔ Tauri IPC ↔ `Core Runtime` ↔ SQLite. UI **не** ходит в БД напрямую.

---

## Domain Entities (Ubiquitous Language)

```text
Observation (immutable fact)
    → Pipeline (validate / dedupe / normalize)
        → Signal (transient change / anomaly)
            → Feature (windowed metric + provenance)
                → Insight (pattern + evidence + optional recommendation)
```

| Entity | Essence | Persistence |
| :--- | :--- | :--- |
| **Observation** | Атомарный факт биометрии/контекста (`id` UUIDv7, `timestamp`, `provider_id`, `data_type`, `payload`, `confidence`) | SQLite `observations` (immutable) |
| **Signal** | Событие изменения/аномалии (`HR_Spike`, `Context_Switch`, …) | Transient (in-memory / alerts) |
| **Feature** | Метрика окна (`FocusScore`, `StressIndex`, `FatigueIndex`) + provenance | Derived / cached as designed |
| **Insight** | Вывод + Evidence + recommendation | Knowledge / Report layer |

---

## Tech Stack & Product Boundaries

### In Scope (MVP path)

| Layer | Stack |
| :--- | :--- |
| Core | Rust 2024, Tokio, tracing, bounded mpsc |
| Specs | `crates/bio-spec` — canonical types & contracts |
| Storage | `rusqlite`, WAL, repositories |
| Pipeline / Engines | `pipeline`, `feature-engine`, `knowledge-engine`, `report-engine` |
| Plugins | `plugin-sdk` traits + adapters |
| Ingest (Phase 2+) | Local HTTP (`127.0.0.1`), Bearer pairing token |
| Desktop | Tauri v2, React/TS, Menubar → later Dashboard |

### Explicit Non-Goals

- Медицинская диагностика / clinical claims  
- Централизованное облако биометрии по умолчанию  
- Скрытый мониторинг сотрудников  
- Обязательный облачный LLM для расчёта метрик  

### Workspace (target)

`apps/desktop` · `crates/{bio-spec, runtime, storage, pipeline, feature-engine, knowledge-engine, report-engine, plugin-sdk}` · `docs/`

---

## North Star for Engineering

> Каждый Insight трассируем до Observation. Каждый crate компилируется без Tauri. Архитектура меняется только через ADR.
