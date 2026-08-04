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

## Git / PR

1. Ветка под **связанный кластер** (`phase/…`, `epic/…`, `feat/…`) — не `main`.  
2. **Commit** когда единица работы готова (batch Task IDs ок); handoff-файлы всё равно пишем на каждый шаг роли.  
3. **Push + PR → `main`** когда связанные задачи кластера Done (или «открой PR») — не ждать конец всего спринта.  
4. Merge после зелёного CI; предпочтительно **squash**.  
5. Прямой push в `main` — запрещён (исключение: Phase 1 foundation уже в истории).

Команды: `docs/12-development.md` § Git workflow · правило `.cursor/rules/06-git-agent-policy.mdc` · hook `.cursor/hooks/git-policy-stop.sh`.

## Оркестрация «от разных лиц»

Фраза вроде `прогони pipeline` / `сделай сам от разных лиц` разрешает одному чату пройти Dev→QA→PM подряд. Handoff-файлы обязательны; git — мало коммитов на кластер, PR когда кластер готов.

## Сейчас в очереди

См. Kanban в `/docs/SPRINT_ROADMAP.md` и canvas execution board.
