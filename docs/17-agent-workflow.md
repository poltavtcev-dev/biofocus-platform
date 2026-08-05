# Agent Team — как работать

Команда ролей: **PM**, **Dev**, **QA**, **UX**.  
Конвейер: `.cursor/rules/05-agent-workflow.mdc`.  
Режимы чатов и бюджет контекста: `.cursor/rules/07-role-dispatch.mdc` · skill `.cursor/skills/role-dispatch/`.

## Зачем режимы

Один чат не должен тащить brief + большой билд + QA + закрытие PM — выгорает контекст.  
Память между чатами = файлы в `docs/handoffs/`.

## Режимы

| Режим | Фраза / когда | Стоп |
| :--- | :--- | :--- |
| **pm-brief** | `как PM:` выдай Ready | Brief + **Next chat** команда; без билда |
| **build-qa** | дефолт после Ready; `прогони build→QA` | Dev\|UX → QA → «Передай PM» (без Done) |
| **build-only** | `как Dev:` / `как UX:` | Handoff QA; стоп |
| **design-only** | `только дизайн` / UX без проверки | UI/copy; без QA и без Done |
| **qa-only** | `как QA:` | `qa-to-pm` → «Передай PM» |
| **pm-close** | `как PM: закрой …` | Roadmap + canvas → Done + next |
| **full-pipeline** | явно и задача маленькая | Dev→QA→PM; иначе дробить |

## Типовой поток (рекомендуется)

```text
Чат 1  как PM: brief          →  Next chat = build-qa
Чат 2  build-qa               →  Next chat = pm-close
Чат 3  как PM: закрой по отчёту
```

Шаблоны фраз: `.cursor/skills/role-dispatch/modes.md`.

## Вызов роли (без режима)

| Фраза | Кто | Результат шага |
| :--- | :--- | :--- |
| `как PM: …` | Product Manager | brief или close Done + docs/canvas |
| `как Dev: …` | Rust/Tauri | код + `*-dev-to-qa.md` |
| `как UX: …` | Desktop UI | UI + handoff (или design-only) |
| `как QA: …` | Lead QA | `*-qa-to-pm.md` |

## Git / PR

1. Ветка под **код-кластер** (`phase/…`, `epic/…`, `feat/…`) — не `main`.  
2. Handoffs на диск каждый роль-шаг; коммит/PR на каждый handoff **не нужен**.  
3. **Push + PR → `main`** только при substantive code и (кластер готов **или** явный «PR»).  
4. Docs / roadmap / canvas — отдельно позже или вместе со следующим code PR.  
5. Merge после зелёного CI; предпочтительно **squash**. Прямой push в `main` — запрещён.

Команды: `docs/12-development.md` · `.cursor/rules/06-git-agent-policy.mdc`.

## Сейчас в очереди

См. Kanban в `/docs/SPRINT_ROADMAP.md` и canvas execution board.
