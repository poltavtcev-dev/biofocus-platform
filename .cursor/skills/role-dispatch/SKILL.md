---
name: role-dispatch
description: >-
  BioFocus multi-role chat dispatch: pick mode (pm-brief, build-qa, build-only,
  design-only, qa-only, pm-close, full-pipeline), emit pasteable next-chat
  commands, keep one mode per chat for token budget. Use when user says как PM,
  как Dev, как UX, как QA, прогони pipeline, build→QA, только дизайн, закрой
  задачу, or asks how to split agent chats.
---

# Role Dispatch

Read also: `.cursor/rules/05-agent-workflow.mdc`, `.cursor/rules/07-role-dispatch.mdc`.

## 1. Pick mode

| User signal | Mode |
| :--- | :--- |
| `как PM:` выдай / Ready / brief | **pm-brief** |
| `прогони build→QA` / `собери и проверь` / PM дал команду build-qa | **build-qa** |
| `как Dev:` / `как UX:` без QA | **build-only** |
| `только дизайн` / `UX без QA` / exploration | **design-only** |
| `как QA:` + handoff path | **qa-only** |
| `как PM: закрой` + `*-qa-to-pm.md` | **pm-close** |
| `прогони pipeline` / `сделай сам` **и** задача маленькая | **full-pipeline** |
| `прогони pipeline` на крупной задаче | Откажись от full; предложи **build-qa** → новый чат **pm-close** |

Default after a Ready code/UI task: recommend **build-qa**, not full-pipeline.

## 2. Playbooks

### pm-brief
1. Write/update `docs/handoffs/{TASK_ID}-pm-brief.md` (AC, role, out of scope, branch).
2. Point Kanban Ready if acting as PM open (do not implement).
3. End with **Next chat** block (copy-paste). Prefer `build-qa` for shippable work; `design-only` only if user asked exploration; `build-only` if they want QA in a separate chat.

```markdown
## Next chat (скопируй в новый чат)
как агент: режим build-qa для {TASK_ID}.
Brief: docs/handoffs/{TASK_ID}-pm-brief.md
1) Как Dev|UX — собери по AC, создай docs/handoffs/{TASK_ID}-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/{TASK_ID}-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце напиши: «Передай PM» + путь к qa-to-pm.
```

### build-qa
1. Role **Dev** or **UX** (from brief) → implement → `docs/handoffs/{TASK_ID}-dev-to-qa.md`.
2. Same chat → role **QA** → verify → `docs/handoffs/{TASK_ID}-qa-to-pm.md`.
3. Stop. Message must include: verdict, path to qa-to-pm, pasteable **pm-close** command.
4. Do **not** update SPRINT_ROADMAP Done or execution canvas.

```markdown
## Next chat (PM)
как PM: закрой {TASK_ID} по docs/handoffs/{TASK_ID}-qa-to-pm.md
```

### build-only
Implement + `*-dev-to-qa.md` → «Передаю QA» + pasteable qa-only command. No QA in this chat.

### design-only
UI/copy/layout only. No AC gate, no `qa-to-pm`, no Done. Optional note file OK. If later shipping: open **build-qa** or **qa-only** on the real task.

### qa-only
Read `*-dev-to-qa.md` + AC → tests → `*-qa-to-pm.md` → «Передаю PM» + pm-close paste.

### pm-close
Follow `.cursor/rules/01-pm-skill.mdc`: roadmap, canvas, status docs, next Ready + **pm-brief** / next-chat command for the following task.

### full-pipeline
Only if user explicit **and** change is small (few files). Else switch to build-qa + pm-close. Still write both handoffs on disk.

## 3. Context budget
- One mode per chat; handoffs are the handoff.
- Prefer summarizing diffs in handoff over pasting huge code into chat.
- Mid-chat blow-up: finish current handoff file → stop → give next-chat paste.
- Do not ask the user to paste the entire previous transcript.

## 4. Role boundaries (unchanged)
- Only **pm-close** (or full-pipeline’s PM step) marks Done + canvas.
- Dev/UX/QA never mark Kanban Done.

## More
- Phrase table: [modes.md](modes.md)
