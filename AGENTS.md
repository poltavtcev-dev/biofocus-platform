# BioFocus — Agent roles

Этот репозиторий ведётся **мультиагентно**. Перед работой прочитай:

1. `.cursor/rules/05-agent-workflow.mdc` — конвейер Dev|UX → QA → PM  
2. `.cursor/rules/00-core.mdc` — архитектурные запреты  
3. `docs/17-agent-workflow.md` — как вызывать роли в чате  
4. `docs/SPRINT_ROADMAP.md` — очередь задач  

Handoffs: `docs/handoffs/`.

## Git / PR

С Phase 2+: ветка под **код-кластер** (`phase/…`, `epic/…`, `feat/…`).  
- **Commit** — когда код-единица готова (batch Task ID ок); handoffs на диске, не обязательный коммит/PR на каждый шаг.  
- **Push + PR → `main`** — только substantive code + (кластер готов **или** явный «PR»); один PR на ветку; squash.  
- **Не PR** для handoffs / roadmap / canvas alone — docs можно обновить отдельно позже.  
Прямой push в `main` запрещён. Детали: `docs/12-development.md`, `.cursor/rules/06-git-agent-policy.mdc`.
