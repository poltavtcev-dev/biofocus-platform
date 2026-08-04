# Agent Team — как работать

Команда ролей: **PM**, **Dev**, **QA**, **UX**. Конвейер зафиксирован в `.cursor/rules/05-agent-workflow.mdc`.

## Вызов в чате

| Фраза | Кто работает | Результат шага |
| :--- | :--- | :--- |
| `как PM: …` | Product Manager | задача/AC или закрытие Done + docs/canvas |
| `как Dev: …` | Rust/Tauri Engineer | код + `docs/handoffs/{ID}-dev-to-qa.md` |
| `как UX: …` | Desktop UI | UI + handoff для QA |
| `как QA: …` | Lead QA | проверка + `docs/handoffs/{ID}-qa-to-pm.md` |

## Порядок на задачу

1. PM (или Ready из roadmap) → берём Task ID.  
2. Dev/UX собирает → **обязательный** handoff QA.  
3. QA проверяет → **обязательный** отчёт PM.  
4. PM обновляет `SPRINT_ROADMAP.md` + canvas → Done.

Шаблоны: `docs/handoffs/TEMPLATE-*.md`.

## Сейчас в очереди

См. Kanban в `/docs/SPRINT_ROADMAP.md` и canvas execution board.
