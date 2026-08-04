# Sprint Gate — Phase 2 / Sprint 3–4

**Date:** 2026-08-04  
**Branch:** `phase/2-ingest-http`  
**Base:** `main`  
**PR title:** Phase 2 Sprint 3–4 — Ingestion, collector, companion (E0–E3)

## Scope closed on branch
- **P2-E1** — Local ingest API (loopback HTTP, pairing token, persist, host wire + `/v1/status`)
- **P2-E2** — macOS context collector (active window, opt-in keystroke aggregates, idle/integration tests)
- **P2-E3** — Companion sample + pairing UX (copy/QR)
- **P2-E0** — Sanitize IPC/HTTP `dbError` paths (`StorageError::public_message`)

## Gate actions
1. Commit this marker
2. `git push -u origin HEAD`
3. `gh pr create --base main`
4. Merge after CI green (do not force-push `main`)

## Manual smokes (pre-merge checklist)
- [ ] Companion Show / Copy / QR in Desktop shell
- [ ] Active window stream → `context_window` Observations
- [ ] Companion CLI sample → local ingest (`biofocus-companion-sample`)
- [ ] CI: rust-core + desktop green
