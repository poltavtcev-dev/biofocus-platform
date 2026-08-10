# Dev|UX → QA: P14-E3-T1

## Meta
- **Task ID:** P14-E3-T1
- **Title:** Dogfood gate + calm Git allowlist Settings/IPC
- **Role that built:** Dev|UX
- **Date:** 2026-08-10
- **AC source:** `docs/handoffs/P14-E3-T1-pm-brief.md` / `/docs/SPRINT_ROADMAP.md` → P14-E3
- **Branch:** `phase/14-git-allowlist`

## What changed
- Dogfood runbook in `docs/12-development.md` § Git activity dogfood; cross-links in `08-plugin-sdk`, `10-security`, glossary/storage notes.
- Crate: `write_watched_roots_file` / `set_settings_watched_roots` / `load_settings_watched_roots`; `BIOFOCUS_HOME` path override (`$BIOFOCUS_HOME/git-watched-roots.toml`).
- Desktop IPC: `get_git_watched_roots` / `set_git_watched_roots` — validate absolute/`~/` roots; empty set → empty file / soft-fail; UI ↛ SQLite; no path dumps in default logs.
- Menubar **Git folders**: list / add / remove / save; calm personal self-tracking copy; no Dashboard chart.
- QA mock: `?mockGitRoots=empty|ready|error`.
- Docs: `09-api`, desktop README; Phase 14 companion marked shipped.
- **No** `GitActivityRate` rewrite; **no** SQLite migration; ADR-013 payload unchanged.

## Crates / apps / files touched
- `crates/macos-collector/src/git_watched_roots.rs`, `lib.rs`
- `apps/desktop/src-tauri/src/git_watched_roots_ipc.rs` (new), `lib.rs`, `Cargo.toml`
- `apps/desktop/src/gitWatchedRoots.ts` (new), `App.tsx`, `App.css`, `README.md`
- Docs: `04-storage`, `08-plugin-sdk`, `09-api`, `10-security`, `12-development`, `16-glossary`
- `docs/handoffs/P14-E3-T1-dev-to-qa.md` (this file)

## How to verify (commands)
```bash
cargo test -p macos-collector git_watched_roots
cargo test -p desktop git_watched_roots_ipc
cd apps/desktop && pnpm exec tsc --noEmit
rg -n "get_git_watched_roots|set_git_watched_roots|mockGitRoots|Git activity dogfood" \
  apps/desktop crates/macos-collector docs/09-api.md docs/12-development.md docs/08-plugin-sdk.md docs/10-security.md
git diff --name-only -- 'crates/feature-engine/' || true
```

Browser smoke (Vite): `?mockGitRoots=ready` / `empty` / `error` on Menubar surface.

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Dogfood runbook in `12-development` + cross-links; enable env + non-empty allowlist; soft-fail empty; privacy reminder
- [ ] AC2: IPC get/set config file only; absolute roots; empty soft-fail; never write roots into Observations/default logs; UI↛SQLite
- [ ] AC3: Calm Menubar Git folders list/add/remove; personal framing; no mandatory Dashboard chart
- [ ] AC4: Tests under `BIOFOCUS_HOME`; mock `?mockGitRoots=`
- [ ] AC5: No Feature rewrite; no SQLite migration; ADR-013 payload unchanged
- [ ] AC6: Docs mark companion shipped
- [ ] AC7: This handoff (Role Dev|UX)
- [ ] Global DoD: no unwrap/expect in prod paths; UI↛DB; personal self-tracking; no PR during freeze

## Risks / not covered
- Live `pnpm tauri dev` click-through not required if IPC unit tests + mock UI pass.
- Probe still needs process restart / already-running collector to pick up newly saved roots on next poll (file re-read each poll — OK without restart).
- Env override is not editable via Settings (by design — file SoT).

## Notes for QA
- PM may have dirty roadmap/canvas docs from opening P14-E3; Dev|UX did not mark Done.
