# BioFocus — Agent roles

Этот репозиторий ведётся **мультиагентно**. Перед работой прочитай:

1. `.cursor/rules/05-agent-workflow.mdc` — конвейер Dev|UX → QA → PM  
2. `.cursor/rules/00-core.mdc` — архитектурные запреты  
3. `docs/17-agent-workflow.md` — как вызывать роли в чате  
4. `docs/SPRINT_ROADMAP.md` — очередь задач  

Handoffs: `docs/handoffs/`.

## Git / PR

С Phase 2: работа в ветке (`phase/N-…` или `sprint/N-…`).  
- **Commit сразу после Dev/UX билда** (код + `*-dev-to-qa.md`); ещё один commit после PM Done, если docs dirty.  
- **Push + PR → `main` — раз на спринт / epic gate** (любая роль, которая закрывает спринт).  
Прямой push в `main` запрещён. Детали: `docs/12-development.md`, `.cursor/rules/06-git-agent-policy.mdc`.  
Hook: `.cursor/hooks/git-policy-stop.sh` напоминает о commit/push, если дерево dirty.
