# BioFocus — Agent roles

Этот репозиторий ведётся **мультиагентно**. Перед работой прочитай:

1. `.cursor/rules/05-agent-workflow.mdc` — конвейер Dev|UX → QA → PM  
2. `.cursor/rules/00-core.mdc` — архитектурные запреты  
3. `docs/17-agent-workflow.md` — как вызывать роли в чате  
4. `docs/SPRINT_ROADMAP.md` — очередь задач  

Handoffs: `docs/handoffs/`.

## Git / PR

С Phase 2+: работа в ветке по **связанному кластеру** (`phase/…`, `epic/…`, `feat/…`).  
- **Commit** — когда единица работы готова (можно batch связанных Task ID); не обязательно после каждого handoff.  
- **Push + PR → `main`** — когда кластер Done / пользователь просит PR; предпочтительно squash.  
Прямой push в `main` запрещён. Детали: `docs/12-development.md`, `.cursor/rules/06-git-agent-policy.mdc`.  
Hook: `.cursor/hooks/git-policy-stop.sh` мягко напоминает при сильно dirty дереве / PR-gate.
