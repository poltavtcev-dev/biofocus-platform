# 00. Vision & Product Strategy

## 1. System Purpose
**BioFocus Platform** — локальная (Local-First) open-source платформа анализа состояния и рабочего ритма человека на основе биометрических и контекстных данных.

Платформа связывает данные с носимых устройств (смарт-часы, фитнес-браслеты, кольца) с реальным контекстом работы за компьютером (активные приложения, ввод, созвоны), помогает понимать продуктивность, концентрацию, стресс и восстановление — **без облака по умолчанию и без обязательного AI**.

**North-star value:** *Personal Pattern Discovery* — персональные закономерности («когда у меня глубже фокус», «что обычно предшествует росту стресса»), а не универсальные советы и не «магия графиков».

**Позиционирование:** Personal Performance Operating System (локальная платформа самопонимания) — не AI-обёртка над биометрией и не медицинский продукт.

## 2. Core Engineering Principles
1. **Local-First & Privacy First:** Все данные, вычисления и локальная БД хранятся строго на устройстве пользователя. Передача биометрии в третьи сервисы без явного согласия запрещена.
2. **AI-Optional Architecture:** AI/LLM используется только как генератор отчетов и интерпретатор естественного языка. Вычисление фич, стресса и метрик выполняется строго алгоритмически (Deterministic Engine).
3. **Explainability & Evidence:** Каждый выведенный инсайт или рекомендация должны иметь прямую ссылку на исходные данные (Provenance & Evidence).
4. **Extensibility & Capability Model:** Подключение новых устройств и источников данных выполняется через плагины с явным описанием возможностей (Capabilities).

### Privacy thesis
> BioFocus не защищает ваши данные на своих серверах. У BioFocus нет ваших данных. Они остаются только у вас.

Zero telemetry by default · AI only on explicit user action · separate permissions · user can export / delete / inspect local data · source open for audit (AGPLv3).

## 3. Analysis stack (5 levels)

Сохраняем Ubiquitous Language: `Observation` → `Signal` → `Feature` → `Insight` → `Recommendation`. UI (в т.ч. Dashboard window) — presentation; Core не называется «Dashboard».

| Level | Name | Role | Implementation today |
| :--- | :--- | :--- | :--- |
| **L1** | Observations | Immutable raw facts (user rarely inspects) | SQLite + ingest + macOS collectors + sample HR |
| **L2** | Features | Deterministic metrics + provenance + confidence (ADR-007) + optional explanation factors (P7-E2) | `feature-engine` DAG (`FeatureNode`) |
| **L3** | Knowledge | Patterns / Insights from Features (+ Evidence) | `knowledge-engine` `InsightRule` (evaluate-on-read) |
| **L4** | Recommendations | Deterministic suggested actions with Evidence | ADR-009 + `focus_dip_pace_hint_v1` + IPC `get_recommendations` / Dashboard Suggestions (Phase 9 Done); thin `Insight.actionRecommendation` remains optional hint only |
| **L5** | Coaching (AI) | NL explanation only — never computes Features / Recommendations | `report-engine` + opt-in local LLM |

```text
Observation → Pipeline → Signal / Feature → Knowledge (Insight) → Recommendation
                                              ↓
                                    Report / Prompt → optional LLM
```

## 4. Sequencing rules (accepted 2026-08-05)

Не выбрасываем идеи из vision — **ставим в очередь и смягчаем формулировки**:

1. **Feature backlog** живёт в `/docs/06-feature-catalog.md` § Planned. В спринт попадают только Features с реальными Observation-входами.
2. **Personal Pattern Discovery** (окна дня / базовые линии) — Phase **8**; нужен ADR (recompute vs Feature history). Не обещать в Phase 6.
3. **Load / Deep-focus style metrics** — ок как детерминированные Features; **без** clinical / burnout diagnosis tone (Global DoD).
4. **Source priority:** Wearables (HealthKit) → Life Events + Calendar → IDE/Git/Browser plugins → ambient (music / weather / light).
5. **Commercial split** (signed builds, updates, optional user-opt-in sync, support) — горизонт Phase **12+**; алгоритмы остаются open-source. Не двигает текущий Core Kanban.
6. **Life Events** (Coffee, Walk, Lunch, Workout, …) — предпочтительно как `Observation` kinds (`data_type` + payload), не параллельная БД; открывать через ADR при Phase 6.

## 5. Dual audience (horizon)

| Audience | Offer |
| :--- | :--- |
| **Power users (OSS)** | Build from source, plugins, custom Features/rules, local LLM, editable prompts (later), full data control |
| **Everyday users (commercial apps, later)** | Packaged installers, guided device setup, calm UI/reports — same open Core, no secret metric math |

## 6. Non-Goals (Scope Limits)
- Платформа **не является** медицинским диагностическим средством / clinical diagnosis.
- Платформа **не предоставляет** централизованное облачное хранилище по умолчанию.
- Платформа **не выполняет** скрытый мониторинг сотрудников (персональный инструмент для самоконтроля).
- LLM **не** считает Features и **не** подменяет Evidence.

## 7. Horizon phases (product ladder)

Immediate Kanban = **P19-E2-T1** (live `SystemNotificationEventProbe` per ADR-020). Below is the accepted ladder — open later slices via PM gate, not all at once.

| Phase | Focus |
| :--- | :--- |
| **0–7** | Done — foundation → ingest → pipeline → Dashboard/Insights → wearable dogfood → Life Events + Calendar → Trust layer (confidence / factors / RecoveryScore) |
| **8** | Done — Pattern Discovery v1 (ADR-008 recompute-on-read → baseline Insight → Dashboard surface) |
| **9** | Done — Deterministic Recommendations (ADR-009 → `focus_dip_pace_hint_v1` → `get_recommendations` + Suggestions) |
| **10** | Done — Plugin wave-1 (ADR-010 Browser categories → collector → `DistractionScore`) |
| **11** | Done — AI coaching polish (ADR-011 → packs → provider UX; interpret-only) |
| **12** | Done — Ambient + commercial packaging (ADR-012 → Now Playing → `AmbientMediaShare` + packaging runbook) |
| **13** | Done — Plugin wave-2 (ADR-013 → Git plugin → `GitActivityRate`) |
| **14** | Done — Git path-allowlist (ADR-014 → live probe → dogfood / Menubar Git folders) |
| **15** | Done — Companion HRV + autonomy (ADR-016) |
| **16** | Done — Ambient light resume (ADR-015 → collector → `AmbientLightShare`); weather deferred |
| **17** | Done — Wearable depth + chart ranges (ADR-017 · ADR-018 → Companion emit → `get_feature_series` + wearable Features) |
| **18** | Done — Notification pressure (ADR-019 → collector → `NotificationPressure`) |
| **19** | Active — Live NC OS mapping (**ADR-020** ✅ → live probe Ready; unlock `NotificationPressure` dogfood) |
| **20+** | Open via later PM gate — IDE · weather · App Store packaging · dogfood polish · `CognitiveLoad` |

Sources: PM triage 2026-08-05 · canvases `platform-vision-triage` · `phase5-architecture-triage`.
