# PM Brief → Dev|UX: P28-E3-T1

**From:** PM  
**To:** Dev + UX  
**Status:** Queued (after P28-E2-T1 Pass)  
**Date:** 2026-08-31  
**Phase:** Phase 28 — Local device reliability **(A)** — **ADR-029**  
**Branch:** `phase/28-local-reliability`

## Task
**P28-E3-T1 — Mac always-on: Login Item opt-in + hide≠quit lifecycle**

## Why
ADR-029 pillar **(A):** ingest, collectors и Feature Worker живут **только пока процесс Desktop запущен**. Закрытие окна не должно убивать сбор; юзер может включить «запускать при входе».

## Acceptance Criteria
1. **Hide on close** (default): закрытие Dashboard window → menubar/tray остаётся; ingest + collectors + Feature Worker продолжают.
2. **Explicit Quit** в меню tray: полный shutdown (как сейчас on exit).
3. **Login Item opt-in** (macOS): calm toggle в UI или Settings — «Launch BioFocus at login»; persists locally (`~/.biofocus/` or app prefs); **off by default**.
4. При login start: open DB, ingest, collectors, feature worker — same as today’s cold start.
5. Docs: `docs/12-development.md` § always-on / Login Item.
6. Handoff: `docs/handoffs/P28-E3-T1-dev-to-qa.md`.

## Out of scope
- LaunchDaemon as root; iOS changes; catch-up replay (P28-E4)
- Silent LAN bind

## Constraints
- Idle-safe; no busy-loop
- UI ↛ SQLite
- User must explicitly opt into Login Item

## Next chat (после P28-E2 Pass)
```
как агент: режим build-qa для P28-E3-T1.
Brief: docs/handoffs/P28-E3-T1-pm-brief.md
1) Как Dev|UX — собери по AC, создай docs/handoffs/P28-E3-T1-dev-to-qa.md
2) Сразу как QA — проверь handoff + AC, создай docs/handoffs/P28-E3-T1-qa-to-pm.md
3) Не закрывай Done / не трогай canvas. В конце: «Передай PM» + путь к qa-to-pm.
```
