# Dev → QA: P2-E1-T2

## Meta
- **Task ID:** P2-E1-T2
- **Title:** Pairing token persistence
- **Role that built:** Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E1-T2; brief `docs/handoffs/P2-E1-T2-pm-brief.md`
- **Branch:** `phase/2-ingest-http`

## What changed
- Pairing token: crypto generate (32 bytes → 64 hex) on first resolve; persist under `~/.biofocus/pairing_token` (or `$BIOFOCUS_HOME/pairing_token`).
- Resolve order: non-empty `BIOFOCUS_INGEST_TOKEN` → else load-or-create file. Unix write mode `0600` (temp + rename).
- `IngestConfig::load()` / `from_env()` now return `IngestResult` (no skeleton hardcode in production path). `DEFAULT_TEST_TOKEN` / `DEFAULT_SKELETON_TOKEN` remain for tests/`with_token`.
- Docs: `docs/10-security.md` §2, `docs/12-development.md`, header note in `docs/09-api.md`. `.gitignore`: `.biofocus/`, `pairing_token`.
- Tests: generate→persist→reload; env override; HTTP accept persisted / reject wrong token.

### Crates / files
- `crates/ingest/src/token.rs` (new)
- `crates/ingest/src/{lib,config,error}.rs`, `Cargo.toml`
- `crates/ingest/tests/{pairing_token,ingest_http}.rs`
- `docs/10-security.md`, `docs/12-development.md`, `docs/09-api.md`
- `.gitignore`
- `Cargo.lock` (getrandom)

## How to verify (commands)
```bash
cargo check -p ingest
cargo test -p ingest
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: First start generates crypto token under `~/.biofocus/` (path documented)
- [ ] AC2: Second start reads same token; ingest accepts only that Bearer
- [ ] AC3: Missing/wrong Bearer → **401**
- [ ] AC4: Secret not in git; `.gitignore` covers `.biofocus/` / `pairing_token`
- [ ] AC5: Docs in `10-security.md` / `12-development.md` (path + `BIOFOCUS_INGEST_TOKEN` override)
- [ ] AC6: Tests: generate→persist→reload; reject wrong token (temp dir / `BIOFOCUS_HOME`)
- [ ] Global DoD: no `unwrap`/`expect` in prod paths; UI↛DB; loopback-only unchanged; idle accept loop unchanged

## Risks / not covered
- Host/Tauri does not yet call `IngestConfig::load` at app start → **P2-E1-T4**.
- QR / copy pairing UX → **P2-E3-T2**.
- Mid-batch 503 contract → **P2-E1-T3** (carry from T1).
- Empty on-disk token file → hard error (no silent regenerate); intentional.
- Env-mutating tests use `unsafe` `set_var` under a mutex (edition 2024); crate forbids `unsafe` outside tests.

## Notes for QA
- Prefer `BIOFOCUS_HOME=<temp>` in manual checks so real `~/.biofocus/pairing_token` is untouched.
- Skeleton const `biofocus-dev-ingest-token` is **not** used by `load()` anymore — only `Default` / `with_token` / tests.
- T1 code still on this branch (not merged to `main` yet); verify together if reviewing the PR.
