# QA → PM: P12-E2-T1

**From:** QA  
**To:** PM  
**Date:** 2026-08-10  
**Dev/UX handoff:** `docs/handoffs/P12-E2-T1-dev-to-qa.md`  
**Verdict:** Pass with notes

## What was verified

### Commands
```text
cargo test -p bio-spec now_playing
→ 5 unit + 1 contract pass

cargo test -p macos-collector --test collector_integration now_playing
→ emit→persist + stop-halts pass

cargo check -p desktop → ok
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 Plugin id `com.biofocus.macos.now_playing` + Capability | **Pass** |
| AC2 ingest_host only when `BIOFOCUS_NOW_PLAYING=1`; same channel → persist | **Pass** |
| AC3 Payload media_kind + is_playing; no titles/artists/lyrics/playlists | **Pass** — payload helper + integration asserts forbidden keys absent |
| AC4 Injectable probe; system soft-fail; ≥5s/on-change; stop joins | **Pass** |
| AC5 Docs shipped (`07`/`08`/`10`/`12`) | **Pass** |
| AC6 Tests emit→persist + stop freezes | **Pass** |
| AC7 Handoff | **Pass** |
| Global DoD | **Pass** — no prod unwrap; no schema; Feature OOS |

### Extra checks
- Ingest maps `InvalidNowPlayingPayload` → `invalid_now_playing`.
- `AmbientMediaShare` absent from `feature-engine` (correct E3 OOS).
- Personal self-tracking framing; no workplace surveillance.

## Defects
None blocking.

## Notes
- **Production `SystemNowPlayingProbe` intentionally returns `None`** (idle soft-fail) — no content-bearing MediaRemote/AppleScript in v1. Scripted probe covers emit→persist. Live OS mapping is a future privacy-safe refinement, not an AC fail (brief allows soft-fail when OS mapping unavailable).

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — mark **P12-E2-T1** Done; Ready **P12-E3-T1** (`AmbientMediaShare` + packaging runbook)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx`
- [ ] Note public surface: `NowPlayingPlugin`, `BIOFOCUS_NOW_PLAYING`, `validate_now_playing_payload`, `invalid_now_playing`
- [ ] **Do not open a PR** (PR freeze until 2026-09-01)

## Suggested next Ready task
- **P12-E3-T1** — `AmbientMediaShare` catalog Feature from `now_playing` Observations + packaging signed-build/notarization runbook companion. Branch: `phase/12-ambient-packaging`.

## Notes for PM
- Branch: `phase/12-ambient-packaging`.
- Default remains off; dogfood with scripted probes or future privacy-safe OS probe.
