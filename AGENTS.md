# BioFocus — Agent roles

Этот репозиторий ведётся **мультиагентно**. Перед работой:

1. `.cursor/rules/05-agent-workflow.mdc` — конвейер Dev|UX → QA → PM  
2. `.cursor/rules/07-role-dispatch.mdc` — режимы чатов + бюджет контекста  
3. `.cursor/skills/role-dispatch/` — playbooks и copy-paste команд  
4. `.cursor/rules/00-core.mdc` — архитектурные запреты  
5. `docs/17-agent-workflow.md` — как вызывать роли  
6. `docs/SPRINT_ROADMAP.md` — очередь задач  

Handoffs: `docs/handoffs/`.

## Рекомендуемый ритм чатов

`pm-brief` → новый чат `build-qa` → новый чат `pm-close`.  
Не смешивать brief + большой билд + close в одном треде.  
`design-only` — отдельно, без QA/Done.

## Git / PR

С Phase 2+: ветка под **код-кластер** (`phase/…`, `epic/…`, `feat/…`).  
- **Commit** — когда код-единица готова (batch Task ID ок); handoffs на диске, не обязательный коммит/PR на каждый шаг.  
- **Push + PR → `main`** — substantive code + (кластер готов **или** явный «PR»); один PR на ветку; squash.  
- **Не PR** для handoffs / roadmap / canvas alone — docs можно обновить отдельно позже.  
Прямой push в `main` запрещён. Детали: `docs/12-development.md`, `.cursor/rules/06-git-agent-policy.mdc`.
