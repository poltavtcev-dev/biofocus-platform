# Dev → QA: P14-E2-T1

## Meta
- **Task ID:** P14-E2-T1
- **Title:** Implement Git watched-roots allowlist + live probe (ADR-014)
- **Role that built:** Dev
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P14-E2-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P14-E2
- **Branch:** `phase/14-git-allowlist`

## What changed
- Allowlist load/validate: `~/.biofocus/git-watched-roots.toml` (`version` + absolute `roots`); missing/empty/unreadable → soft-fail idle (no whole-disk scan).
- Precedence: **file SoT when present**; optional `BIOFOCUS_GIT_WATCHED_ROOTS` (colon/comma) only when file **absent** (tests/CI).
- Live `SystemGitActivityProbe`: discover nested repos under allowlisted roots (depth ≤4, ≤64 repos); fingerprint `.git/HEAD` / reflog length / `FETCH_HEAD` / `index`; emit ADR-013 `activity_kind` + `event_count` — **never** paths/remotes/branch/SHA/message/diff/author in payloads or default logs.
- Keep `ScriptedGitActivityProbe`; `GitActivityPlugin` / `ingest_host` still opt-in `BIOFOCUS_GIT_ACTIVITY=1`; poll ≥5s; `stop_stream` joins.
- Tests: empty allowlist → no emit; fixture root commit → channel → persist; stop freezes polls; unit parse for TOML.
- Docs finalized as **shipped**: `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development` (+ storage/glossary notes).
- **No** `GitActivityRate` formula rewrite; **no** SQLite migration / allowlist table.

## Crates / apps / files touched
- `crates/macos-collector/src/git_watched_roots.rs` (new)
- `crates/macos-collector/src/git_activity_probe.rs` (live probe)
- `crates/macos-collector/src/git_activity_plugin.rs`, `lib.rs`, `Cargo.toml` (`toml`)
- `crates/macos-collector/tests/collector_integration.rs`
- `apps/desktop/src-tauri/src/ingest_host.rs` (log wording)
- Docs: `04-storage`, `07-contracts`, `08-plugin-sdk`, `10-security`, `12-development`, `16-glossary`
- `docs/handoffs/P14-E2-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p macos-collector --tests
cargo test -p pipeline normalize::tests::git_activity
cargo check -p desktop
rg -n "git-watched-roots|BIOFOCUS_GIT_WATCHED_ROOTS|SystemGitActivityProbe|resolve_watched_roots" \
  crates/macos-collector apps/desktop/src-tauri docs/07-contracts.md docs/08-plugin-sdk.md docs/10-security.md docs/12-development.md
# No Feature rewrite / no migration applied
git diff --name-only -- 'crates/feature-engine/' || true
rg -n "CREATE TABLE|ALTER TABLE" crates/macos-collector || true
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Load/validate `~/.biofocus/git-watched-roots.toml`; missing/empty/unreadable → soft-fail idle; expand `~` / home
- [ ] AC2: Live probe only under allowlisted roots (nested OK); ADR-013 payload only; no path dumps in default logs
- [ ] AC3: `ingest_host` only when `BIOFOCUS_GIT_ACTIVITY=1`; same channel → persist; ≥5s / no busy-loop; `stop_stream` joins
- [ ] AC4: Scripted probe kept; tests: empty → no emit; fixture → emit→persist; stop freezes; forbidden keys still stripped/rejected
- [ ] AC5: Docs shipped (not planned-only); env override documented with file-SoT precedence
- [ ] AC6: No Feature rewrite; no SQLite migration / allowlist table
- [ ] AC7: This handoff
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; personal self-tracking; no PR during freeze

## Risks / not covered
- Settings / IPC to edit allowlist → **P14-E3-T1**.
- Heuristic classification from reflog/`FETCH_HEAD` may map some ops to `other` — acceptable for dogfood.
- Deep monorepos beyond depth 4 / 64 repos are truncated by design (idle-safe).
- Env override ignored when a (even empty/broken) config file exists — by ADR precedence.

## Notes for QA
- Dogfood: create `~/.biofocus/git-watched-roots.toml` with absolute roots + set `BIOFOCUS_GIT_ACTIVITY=1`.
- PM open roadmap/canvas docs may already be dirty; Dev did not mark Done.
