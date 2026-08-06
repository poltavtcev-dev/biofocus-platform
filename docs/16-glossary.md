# 16. Ubiquitous Language & Glossary

- **Observation:** Исходный неизменяемый факт биометрии или контекста.
- **Life Event:** Пользовательский контекстный факт (Coffee / Walk / Lunch / Workout, …), моделируемый как `Observation` с `data_type: "life_event"` и `payload.kind` — не отдельная сущность БД (ADR-006).
- **Calendar Event:** Локальный календарный/meeting факт как `Observation` с `data_type: "calendar_event"` (uid/start/end/busy; без title/body) — dogfood через opt-in ICS (`BIOFOCUS_CALENDAR`), без cloud OAuth (P6-E3-T1).
- **Signal:** Зафиксированная в реальном времени аномалия или переход состояния (например, всплеск пульса).
- **Feature:** Рассчитанный за временной интервал метрический показатель (например, FocusScore).
- **Insight:** Готовый аналитический вывод с фактами-доказательствами (Evidence).
- **Provider / Plugin:** Источник данных (модуль), отправляющий `Observation`.
- **Core Daemon:** Фоновый процесс на Rust, обрабатывающий пайплайн данных.