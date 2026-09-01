# QA → PM: P16-E1-T1

## Meta
- **Task ID:** P16-E1-T1
- **Title:** Implement ambient light plugin (ADR-015)
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P16-E1-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
### Commands run + results
```bash
cargo test -p bio-spec ambient_light
# 6 unit + 1 contracts — ok

cargo test -p macos-collector ambient_light
# plugin/probe units (5) + collector_integration ambient_light_* (3) — ok

cargo check -p desktop
# ok
```

### AC results
| AC | Result |
| :--- | :--- |
| AC1 Plugin id + Capability `ambient_light` | **Pass** — `AmbientLightPlugin` id `com.biofocus.macos.ambient_light`; Capability name/data_types covered; unit test asserts contract |
| AC2 ingest_host only when `BIOFOCUS_AMBIENT_LIGHT=1`; channel → persist; UI↛SQLite | **Pass** — `ingest_host.rs` gates on `ambient_light_enabled()`; same `collector_tx` → persist worker |
| AC3 Payload `light_kind` + optional `level` 0–100; never camera/screen/geo/mic | **Pass** — `ambient_light_payload` + bio-spec validation; integration asserts forbidden keys absent; pipeline strips extras (existing) |
| AC4 Soft-fail OS probe; scripted probe; ≥5s / on-change; stop joins; no busy-loop | **Pass** — `SystemAmbientLightProbe` → `None`; scripted FIFO; default poll ≥5s; stop freezes poll counts |
| AC5 Docs shipped; collector active; Feature → P16-E2 | **Pass** — `07` / `08` / `10` / `12` / `16-glossary` (+ storage/catalog notes) mark collector **shipped**; Feature still planned P16-E2 |
| AC6 Tests emit→persist + stop freeze + soft-fail idle | **Pass** — three integration tests + bio-spec units |
| AC7 No Feature / migration / new ADR | **Pass** — no `AmbientLightShare` / `register_ambient_light` in feature-engine; no schema migration; ADR-015 reused |
| AC8 Handoff | **Pass** — `docs/handoffs/P16-E1-T1-dev-to-qa.md` |
| Global DoD | **Pass** — `unwrap`/`expect` only in `#[cfg(test)]`; UI↛DB; glossary terms |

### Extra checks (edge / security)
- Privacy grep on ambient_light collector helpers: no payload emission of camera/screenshot/geo/mic fields (comments only).
- Soft-fail system probe emits **zero** Observations (integration).
- Opt-in default off documented + host log path when unset.

## Defects (if any)
- None.

## What PM must update
- [ ] `/docs/SPRINT_ROADMAP.md` — move **P16-E1-T1** to Done; Ready **P16-E2-T1** (`AmbientLightShare`)
- [ ] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [ ] Other docs if needed: `ARCHITECTURE_STATUS.md` / `PROJECT_CANVAS.md` / `14-roadmap.md` (collector shipped note); fold into cluster docs when convenient
- [ ] **Do not** open Phase 17 / wearable work (ADR-017 parked until P16 Done)

## Suggested next Ready task
- **P16-E2-T1** — AmbientLightShare catalog Feature (`feature-engine` + docs)

## Notes for PM
- Live OS brightness mapping still soft-fails by design (privacy) — dogfood of real light bands needs a future privacy-safe probe or scripted injection; Feature E2 can still ship against scripted / fixture Observations.
- Branch: `phase/16-ambient-light`. PR freeze until 2026-09-01 — no PR from this handoff.
