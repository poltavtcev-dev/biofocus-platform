# Dev|UX → QA: P2-E3-T2

## Meta
- **Task ID:** P2-E3-T2
- **Title:** Pairing UX (token share)
- **Role that built:** UX + Dev
- **Date:** 2026-08-04
- **AC source:** `/docs/SPRINT_ROADMAP.md` → P2-E3-T2; brief `docs/handoffs/P2-E3-T2-pm-brief.md`

## What changed
- Desktop IPC `get_pairing_token`: token + loopback `ingestBaseUrl` + SVG QR; path-safe error mapping
- Shell **Companion** section: Show / Hide token (masked), Copy, Show QR; calm HIG copy; no cloud
- Window resized to fit pairing UI (`320×420`)
- Docs: `09-api`, `10-security`, `12-development`, desktop + companion READMEs
- Dep: `qrcode` (SVG) in desktop host only

## How to verify (commands)
```bash
cargo test -p desktop
cargo check -p desktop
cd apps/desktop && pnpm exec tsc --noEmit && pnpm build
# Optional live: pnpm tauri dev → Companion Show / Copy / Show QR
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Desktop shell exposes view/copy (and QR) of pairing token
- [ ] AC2: No cloud account; token remains local secret
- [ ] AC3: UI does not talk to SQLite / token file directly (IPC only)
- [ ] AC4: Flow documented in `docs/12-development.md` / companion README
- [ ] AC5: this handoff present
- [ ] Global DoD: no unwrap/expect in prod pairing path; UI↛DB; glossary ok

## Risks / not covered
- Physical iPhone still cannot reach loopback-bound ingest (documented; LAN out of scope).
- Clipboard may fail in non-secure / browser-only Vite preview without Tauri — Copy reports “Could not copy.”
- QR encodes the raw token string (not a deep-link URI scheme).
- Full iOS App Store UI still stub-only — paste/scan documented.

## Notes for QA
- Errors from token I/O must not leak absolute paths (unit test covers mapping).
- `fromEnv: true` when `BIOFOCUS_INGEST_TOKEN` is set.
- Do not log/print the live token in QA notes.

## Git
Commit after this handoff (no push until sprint gate).
