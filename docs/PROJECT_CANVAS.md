# BioFocus — Project Canvas

> One-page snapshot. Sources: `/docs/00–16`, `ARCHITECTURE_STATUS.md`, `decision-log.md`.  
> **Status:** MVP 1.0 Architecture Frozen · **License:** AGPLv3 (ADR-004) · **OSS public prep:** [`docs/19-oss-public-launch.md`](19-oss-public-launch.md)

---

## Problem

Знаниеработники не видят связь между **физиологией** (пульс, HRV, восстановление) и **реальным рабочим контекстом** (приложения, ввод, созвоны). Существующие трекеры либо облачные и непрозрачные, либо дают метрики без доказательной цепочки «факт → вывод».

## Value Proposition

**BioFocus** — Local-First open-source **Personal Performance OS**: помогает понимать продуктивность, концентрацию, стресс и восстановление **на устройстве пользователя**.

1. Собирает биометрию и контекст работы как неизменяемые факты (`Observation`).
2. Детерминированно считает метрики фокуса / стресса / усталости (`Feature`).
3. Выдаёт объяснимые инсайты с Evidence (`Insight`) — опционально через LLM только как интерпретатор, не как движок метрик.

**North star:** *Personal Pattern Discovery* (персональные закономерности) — не универсальные советы и не AI-обёртка.  
**Для кого:** персональный самоконтроль ритма работы (не меддиагностика, не employee surveillance, не облако по умолчанию).  
**Full vision + sequencing:** `/docs/00-vision.md`.

### Analysis levels (L1–L5)

| L | Layer | Engine today |
| :--- | :--- | :--- |
| 1 | Observations | ingest / collectors / SQLite |
| 2 | Features | `feature-engine` |
| 3 | Knowledge (Insights) | `knowledge-engine` |
| 4 | Recommendations | ADR-009 + `focus_dip_pace_hint_v1` + `get_recommendations` / Suggestions (Phase 9 Done) |
| 5 | Coaching (AI interpret) | `report-engine` + opt-in LLM |

UI (Menubar / Dashboard window) = **presentation** over IPC — not a Core layer name.

### Wearable / companion (Phase 5 — Done)

- **Shipped:** opt-in LAN bind (ADR-005) + advertise hints + Companion Base URL / token/QR + runnable **iOS HealthKit** companion posts HR `Observation`s to Desktop ingest (Simulator loopback or physical phone on LAN).
- **Dogfood:** end-to-end operator runbook in `docs/12-development.md` § Wearable dogfood runbook. Companion READMEs mirror the same steps.
- Today: HealthKit / CLI sample sources + loopback/LAN ingest; Desktop Companion shows copyable base URL + token; no personal device inventory in git.
- **Phase 6 (Done):** Life Events (ADR-006) + Desktop quick-log + Calendar ICS → `MeetingDensity` / `RecoveryBetweenMeetings`. Wearable dogfood verified: iOS Companion → LAN ingest with HealthKit HR (Mi Band via Apple Health).
- **Phase 7 (Done):** Trust layer — ADR-007 confidence → Explanation factors → `RecoveryScore`.
- **Phase 8 (Done 2026-08-08):** Pattern Discovery v1 — ADR-008 **recompute-on-read** → `focus_vs_recent_baseline_v1` → calm Dashboard Insights surface.
- **Phase 9 (Done 2026-08-08):** Deterministic Recommendations — ADR-009 → `focus_dip_pace_hint_v1` → `get_recommendations` + Dashboard Suggestions.
- **Phase 10 (Done 2026-08-10):** Plugin wave-1 — ADR-010 Browser → collector → `DistractionScore`. Later: IDE/Git plugins.
- **Phase 11 (Done 2026-08-10):** AI coaching polish — ADR-011 → packs (`build_report_with_pack`) → provider UX (`get_local_llm_status` + pack-aware Report). L5 interpret-only.
- **Phase 12 (Done 2026-08-10):** Ambient + packaging — ADR-012 → Now Playing → `AmbientMediaShare` + `docs/18-packaging-runbook.md`.
- **Phase 13 (Done 2026-08-10):** Plugin wave-2 — ADR-013 → Git plugin → `GitActivityRate`.
- **Phase 14 (Done 2026-08-10):** Git path-allowlist — ADR-014 → live probe → dogfood + Menubar **Git folders** IPC.
- **Phase 15 (Done 2026-08-11):** Companion HRV + autonomy — ADR-016.
- **Phase 16 (Done 2026-08-11):** Ambient light — ADR-015; collector + `AmbientLightShare`.
- **Phase 17 (Done 2026-08-11):** Wearable depth + chart ranges — ADR-017 · ADR-018 → Companion emit → `get_feature_series` + wearable Features.
- **Phase 18 (Done 2026-08-11):** Notification pressure — ADR-019 → collector → `NotificationPressure`.
- **Phase 19 (Done 2026-08-11):** Live NC OS mapping — ADR-020 → usernoted live probe → dogfood.
- **Phase 20 (Done 2026-08-11):** CognitiveLoad — **ADR-021** ✅; Feature + dogfood + Dashboard **Combined demand**.
- **Phase 21 (Done 2026-08-11):** DeepWorkScore — **ADR-022** ✅; Feature + dogfood + Dashboard **Sustained focus**.
- **Phase 22 (Done 2026-08-11):** AttentionStability — **ADR-023** ✅; Feature + dogfood + Dashboard **Focus stability**.
- **Phase 23 (Done 2026-08-11):** Personal Context Layer — **ADR-024** ✅; `DeskAwayPresence` + health→prompt + dogfood / **Away from desk**.
- **Phase 24 (Done 2026-08-12):** CircadianOffset — **ADR-025** ✅; Feature + dogfood / **Schedule alignment**.
- **Next:** Phase 25 **SustainedLoadIndicator** — **ADR-026** ✅ · Ready **P25-E2-T1** (ship Feature). Gate **PM-GATE-POST-P24** ✅.
- **Git:** **PR freeze until 2026-09-01** — local branch commits OK; no PRs (`docs/12-development.md`).
- Menubar alert colors: Phase 3 E3 · Dashboard/Insights: Phase 4 · Suggestions: Phase 9.

Dogfood tip: if `base_url_hints` is empty under `BIOFOCUS_INGEST_LAN=1`, set `BIOFOCUS_INGEST_BIND_HOST=<lan-ipv4>` before pairing.

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
| Ingest (Phase 2+) | Local HTTP; default loopback; opt-in LAN (ADR-005) + Bearer |
| Desktop | Tauri v2, React/TS, Menubar + Dashboard window (presentation) |

### Explicit Non-Goals

- Медицинская диагностика / clinical claims  
- Централизованное облако биометрии по умолчанию  
- Скрытый мониторинг сотрудников  
- Обязательный облачный LLM для расчёта метрик  

### Workspace (target)

`apps/desktop` · `crates/{bio-spec, runtime, storage, pipeline, feature-engine, knowledge-engine, report-engine, plugin-sdk}` · `docs/`

---

## Horizon ladder (accepted)

Immediate Kanban = **P25-E2-T1** (`/docs/SPRINT_ROADMAP.md`). Phase 0–24 Done · Phase 25 Active (E1 ✅).

| Phase | Focus |
| :--- | :--- |
| **5** | Wearable dogfood (LAN → HealthKit) — **done** |
| **6** | Life Events v1 + Calendar Features — **done** |
| **7** | Feature confidence + Explanation factors + RecoveryScore — **done** |
| **8** | Pattern Discovery v1 (ADR-008 + baseline + Dashboard) — **done** |
| **9** | Deterministic Recommendations — **done** |
| **10** | Plugin wave-1 (Browser → DistractionScore) — **done** |
| **11** | AI coaching polish (ADR-011 → packs → provider UX) — **done** |
| **12** | Ambient + packaging (ADR-012 → Now Playing → AmbientMediaShare) — **done** |
| **13** | Plugin wave-2 (Git → GitActivityRate) — **done** |
| **14** | Git path-allowlist (ADR-014 → live probe → dogfood/UX) — **done** |
| **15** | Done — Companion HRV + autonomy (ADR-016) |
| **16** | Done — Ambient light (ADR-015 → plugin → AmbientLightShare); weather deferred |
| **17** | Done — Wearable depth + chart ranges (ADR-017 · ADR-018) |
| **18** | Done — Notification pressure (ADR-019 → collector → NotificationPressure) |
| **19** | Done — Live NC OS mapping (ADR-020 → usernoted probe → dogfood) |
| **20** | Done — CognitiveLoad (**ADR-021** → Feature + dogfood + Combined demand) |
| **21** | Done — DeepWorkScore (**ADR-022** → Feature + dogfood + Sustained focus) |
| **22** | Done — AttentionStability (**ADR-023** → Feature + dogfood + Focus stability) |
| **23** | Done — Personal Context Layer (**ADR-024** → DeskAwayPresence + health→prompt + Away from desk) |
| **24** | Done — CircadianOffset (**ADR-025** → Feature + Schedule alignment) |
| **25** | Active — SustainedLoadIndicator (**ADR-026** ✅ · Ready P25-E2); deferred IDE · weather · App Store · TypingRhythm |

**Sequencing:** Features only with real inputs · calm non-clinical copy · Calendar/Life Events before ambient plugins · commercial ≠ secret Core math.

## North Star for Engineering

> Каждый Insight трассируем до Observation. Каждый crate компилируется без Tauri. Архитектура меняется только через ADR. Personal Pattern Discovery — продуктовый north star (`/docs/00-vision.md`).
