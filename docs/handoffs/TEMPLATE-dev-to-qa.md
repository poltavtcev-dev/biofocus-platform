# TEMPLATE: Dev|UX → QA

> Скопировать в `docs/handoffs/{TASK_ID}-dev-to-qa.md` и заполнить.  
> После заполнения Dev/UX **останавливается** и передаёт ход QA.

## Meta
- **Task ID:** P1-E?-T?
- **Title:**
- **Role that built:** Dev | UX
- **Date:**
- **AC source:** `/docs/SPRINT_ROADMAP.md` → section …

## What changed
- Summary (2–5 bullets):
- Crates / apps / files touched:

## How to verify (commands)
```bash
# примеры
cargo check
cargo test -p <crate>
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1 …
- [ ] AC2 …
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; glossary terms

## Risks / not covered
- …

## Notes for QA
- …
