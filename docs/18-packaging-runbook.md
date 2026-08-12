# 18. Commercial Packaging Runbook (Phase 12 / ADR-012)

> Companion to ambient Feature work — **process / distribution stance**, not a sync product and not closed Feature math.
> AGPLv3 **Core stays open**. Signed macOS packaging ≠ proprietary catalog formulas.

## Stance (locked)

| Topic | Stance |
| :--- | :--- |
| Core license | AGPLv3 — Feature math, pipeline, collectors remain open source |
| Commercial edge | Signed / notarized distribution + update channel ops — **not** secret metrics |
| Optional sync | **Off by default**; stance only in Phase 12 — **no** sync product, **no** account system |
| Cloud LLM | Unchanged — L5 interpret-only, opt-in local; no marketplace / cloud Feature math |
| UI → DB | Unchanged — UI ↔ Tauri IPC ↔ Core only |
| App Store listing | Out of Phase 12 v1 product scope (may follow later under a new ADR) |

## Signed macOS `.app` / `.dmg` (developer checklist)

Local unsigned / ad-hoc builds remain the default dogfood path (`pnpm tauri build` → `BioFocus.app`). For a **distribution-signed** build:

1. **Apple Developer Program** membership + certificates in Keychain:
   - Developer ID Application (for `.app` outside Mac App Store)
   - Optional: Developer ID Installer if shipping a `.pkg`
2. **Bundle identity** must match the Desktop Tauri / Xcode signing config (`apps/desktop`).
3. **Build** a release app:

```bash
cd apps/desktop
pnpm install
pnpm tauri build
# → target/release/bundle/macos/BioFocus.app (paths may vary by Tauri version)
```

4. **Codesign** the `.app` (and nested helpers/frameworks if required by your tooling):

```bash
codesign --deep --force --options runtime \
  --sign "Developer ID Application: <Your Name> (<TEAM_ID>)" \
  path/to/BioFocus.app
codesign --verify --deep --strict --verbose=2 path/to/BioFocus.app
```

5. **Package** a `.dmg` (or zip) for distribution — prefer a read-only DMG with the signed `.app` as the sole payload. Do not embed pairing tokens, SQLite DBs, or user Observations in the image.

6. **Staple / ship** only after notarization succeeds (next section).

> Tip: keep signing secrets in CI / local Keychain — never commit `.p12`, API keys, or notarization credentials to the repo.

## Notarization

Apple Gatekeeper expects Developer ID–signed software to be **notarized**:

1. Create an app-specific password / API key for `notarytool` (Apple ID with App Store Connect access).
2. Submit the signed archive (`.dmg` or zip of `.app`):

```bash
xcrun notarytool submit path/to/BioFocus.dmg \
  --apple-id "<apple-id>" \
  --team-id "<TEAM_ID>" \
  --password "<app-specific-password>" \
  --wait
```

3. On success, **staple** the ticket:

```bash
xcrun stapler staple path/to/BioFocus.dmg
# or staple the .app before packaging, depending on ship format
spctl --assess --type execute -vv path/to/BioFocus.app
```

4. Re-test on a clean Mac / VM without Developer tools — Gatekeeper should accept the staple.

Failures are usually: missing hardened runtime, unsigned nested binaries, or entitlements mismatches. Fix signing first; do not weaken privacy entitlements to “make notarization pass.”

## Update channel stance

| Channel | Phase 12 stance |
| :--- | :--- |
| Local / TestFlight-like dogfood | Manual install of signed builds from a private link is enough |
| Public auto-update | **Deferred** — no Sparkle/Tauri updater product required in P12 |
| Channel naming | Prefer explicit labels (`dev` / `beta` / `stable`) when an updater lands later |
| Telemetry in updater | Must stay local-first; no Observation / Feature payload exfil |

When an updater is introduced (future ADR): keep it opt-in or clearly disclosed; never require cloud accounts for Core Features to compute.

## Optional sync (explicit non-goals)

- No Observation / Feature sync store in Phase 12.
- No account system, no cloud outbox, no “backup to BioFocus servers.”
- Any future sync requires a **new ADR + user approve** (schema / privacy / UI↛DB unchanged).

## Verification (local)

```bash
# Feature path still works without packaging:
cargo test -p feature-engine ambient_media
cargo test -p pipeline now_playing

# Release build (unsigned dogfood OK):
cd apps/desktop && pnpm tauri build
```

## Related docs

- ADR-012: `docs/decision-log.md`
- Ambient Feature: `docs/06-feature-catalog.md` §1.9 `AmbientMediaShare`
- Security / privacy: `docs/10-security.md`
- Dev commands: `docs/12-development.md`
- OSS public launch / visibility gates (ADR-027): [`docs/19-oss-public-launch.md`](19-oss-public-launch.md) — see **§ Dry-run release checklist**; packaging **ops** stay **here**; do not duplicate secrets into 19
