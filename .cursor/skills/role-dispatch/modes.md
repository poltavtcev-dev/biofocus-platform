# Role dispatch — phrase cheat sheet

## Paste templates

### After PM Ready (default shippable)
```text
как агент: режим build-qa для {TASK_ID}.
Brief: docs/handoffs/{TASK_ID}-pm-brief.md
Собери (Dev|UX) → handoff dev-to-qa → сразу QA → qa-to-pm.
Не Done / не canvas. В конце: «Передай PM» + путь к отчёту.
```

### Design exploration (no QA)
```text
как UX: режим design-only для {TASK_ID или темы}.
Только UI/copy/layout. Без QA-отчёта и без Done.
```

### Build without QA in same chat
```text
как Dev: режим build-only для {TASK_ID}.
Brief: docs/handoffs/{TASK_ID}-pm-brief.md
Handoff: docs/handoffs/{TASK_ID}-dev-to-qa.md — затем стоп «Передаю QA».
```

### QA alone
```text
как QA: режим qa-only для {TASK_ID}.
Handoff: docs/handoffs/{TASK_ID}-dev-to-qa.md
Отчёт: docs/handoffs/{TASK_ID}-qa-to-pm.md — «Передаю PM».
```

### PM close
```text
как PM: закрой {TASK_ID} по docs/handoffs/{TASK_ID}-qa-to-pm.md
```

### Full pipeline (small tasks only)
```text
прогони full-pipeline для {TASK_ID} (маленькая задача): Dev|UX → QA → PM close.
Handoffs на диск. Если раздуется контекст — остановись после qa-to-pm.
```

## When NOT to combine
| Avoid in one chat | Prefer |
| :--- | :--- |
| PM brief + implementation + QA + close | pm-brief → build-qa → pm-close (3 chats) |
| Large UI + full Recharts + QA + PM | build-qa, then pm-close |
| design-only + mark Done | design-only, then later build-qa |

## Signals that context is too fat
- Agent re-reads the same large files repeatedly
- Failures from lost earlier AC
- User correcting “you forgot the handoff”

→ Write/update handoff → stop → new chat with paste template.
