# 19. OSS Public Launch (Phase 26 / ADR-027)

> **Maintainer SoT** for honest public-beta / visibility gates — **not** App Store productization and **not** Feature math.
> Status: SoT **authored** (P26-E2) · dry-run checklist **added** (P26-E3). **Public launch is not Done** until layers (2) and (3) complete after PR freeze.
> Scope lock: **ADR-027** in [`docs/decision-log.md`](decision-log.md).

## Purpose

Give maintainers a single checklist for making BioFocus **honestly public** without:

- claiming “release ready” while phase clusters sit only on feature branches;
- flipping repo / Release visibility before a **notarized** GitHub Release exists;
- inventing cloud accounts, default telemetry, workplace-monitoring marketing, or secret Core Feature formulas.

BioFocus remains a **personal self-tracking** tool (Local-First). Public copy must not frame the product as employer / workplace surveillance or clinical diagnosis.

## Relationship to packaging (`docs/18-packaging-runbook.md`)

| Doc | Answers | Does **not** contain |
| :--- | :--- | :--- |
| **This file (`19-oss…`)** | When / how to flip **visibility**; repo & docs honesty; contributor path; AGPLv3 framing; Release-before-public gate | Codesign command recipes, notarization credentials, or signing key material |
| **[`18-packaging-runbook.md`](18-packaging-runbook.md)** | How to **build / codesign / notarize / staple** a macOS `.app`/`.dmg`; update-channel stance | Visibility flip policy, contributor marketing copy |

**Do not merge the two files.** Packaging ops stay in 18. This doc **references** 18 for layer (3) only. Never commit signing secrets, notarization passwords, or Keychain material to the repo.

**App Store listing** stays **deferred** (beyond Phase 12 runbook; not Phase 26 primary). Future App Store productization needs a **new ADR + approve**.

## License & Core openness

- Core license: **[AGPLv3](../LICENSE)** (ADR-004).
- Commercial packaging (signed builds, notarization, optional future update channel) ≠ closed / proprietary Feature formulas.
- Catalog Feature math, pipeline, and collectors remain open for audit.

## Framing (public copy)

| Do | Don’t |
| :--- | :--- |
| Personal productivity / stress / recovery self-tracking | Workplace / manager dashboards or employee monitoring |
| Local-First; zero telemetry by default | Cloud accounts or Observation sync by default |
| Calm, non-clinical language | Clinical diagnosis / “you are burned out” marketing |
| LLM = L5 interpret-only (explicit user action) | LLM as source of Features / Recommendations / Evidence |

## Maintainer checklist — three locked layers (ADR-027)

### (1) Repo / docs honesty & hygiene

- [x] SoT file exists at `docs/19-oss-public-launch.md` (this document — **P26-E2**).
- [ ] README / CONTRIBUTING / vision pointers resolve here and do **not** claim “public launch Done”.
- [ ] Personal self-tracking framing in public-facing docs; no workplace-surveillance marketing.
- [ ] AGPLv3 Core called out honestly; no implication of secret Feature math.
- [ ] Broken or aspirational links that promise missing product surfaces are fixed or clearly marked deferred.
- [ ] No pairing tokens, `~/.biofocus` secrets, or notarization credentials in git.

**During PR freeze (through 2026-09-01 inclusive):** layer (1) docs work and local branch commits are allowed. Opening a PR is **not**.

### (2) Post–2026-09-01 `main` catch-up / related cluster merges

- [ ] PR freeze has ended (after **2026-09-01**, or earlier if explicitly lifted).
- [ ] Related phase clusters merged to `main` via normal PR flow (see [`12-development.md`](12-development.md) § Git workflow).
- [ ] Docs do **not** claim “public release ready” while substantive clusters remain only on feature branches.
- [ ] CI expectations for `main` are understood (`.github/workflows/ci.yml`).

**During freeze:** prepare notes / branch hygiene only — **do not** merge to `main` via PR.

### (3) Notarized GitHub Release **before** flipping public visibility

- [ ] Build and notarize a distribution artifact using [`18-packaging-runbook.md`](18-packaging-runbook.md) (Developer ID + `notarytool` + staple).
- [ ] Publish that artifact as a **GitHub Release** (or equivalent maintainer Release channel).
- [ ] **Only then** flip repo / Release visibility to public (or widen audience).
- [ ] Unsigned / local / ad-hoc builds and dry-run notes **do not** satisfy this gate.

**During freeze:** local unsigned dry-run **notes** and checklist text may land in **P26-E3** — they do **not** authorize a public Release flip or visibility change.

## Freeze vs after-freeze (quick reference)

| When | Allowed | Not allowed |
| :--- | :--- | :--- |
| **During PR freeze** (≤ **2026-09-01**) | Docs / ADR / handoffs; this SoT; local unsigned dry-run **notes** (E3); commits on `phase/26-oss-public-launch` | PR to `main`; public notarized Release as visibility flip; claiming public launch Done |
| **After freeze** (≥ **2026-09-02**, or earlier if freeze lifted) | Cluster PRs → `main` (layer 2); notarized Release then visibility flip (layer 3) | Skipping notarization before public visibility; default cloud accounts / telemetry; App Store productization without a new ADR |

## Explicit non-goals (this phase / this doc)

- Cutting a public GitHub Release or flipping visibility in Phase 26 E2/E3 (execution after freeze + layer 2; dry-run notes do not authorize a flip).
- App Store listing / review pipeline.
- New Observation families, Feature formulas, or Pattern Discovery rule expansion.
- IDE plugin, weather ambient, TypingRhythm, DeepFocusLikelihood as launch blockers.
- Cloud accounts / Observation sync / telemetry **by default**.
- Copying signing credentials into the repository.

## Dry-run release checklist (P26-E3 / ADR-027)

> Rehearse packaging readiness **without** treating the rehearsal as layer (3).
> Ops details live in [`18-packaging-runbook.md`](18-packaging-runbook.md) — **do not** paste credential recipes or signing secrets here.

### A. During PR freeze (now — through 2026-09-01 inclusive)

Allowed:

- [ ] Confirm SoT + packaging cross-links resolve (`19-oss…` ↔ `18-packaging…`).
- [ ] Read packaging runbook end-to-end (build → codesign → notarize → staple → verify).
- [ ] Local **unsigned** dogfood build when useful (`cd apps/desktop && pnpm tauri build`) — optional; record notes below.
- [ ] Inventory signing posture honestly (Developer ID Application present? notarization Apple ID / app-specific password configured in Keychain — **do not** commit them).
- [ ] Docs / handoffs / local branch commits on `phase/26-oss-public-launch`.

Forbidden during freeze:

- [ ] Opening a PR / merging to `main` via PR.
- [ ] Cutting a **public** notarized GitHub Release as the visibility flip.
- [ ] Flipping repo / Release visibility to public.
- [ ] Claiming **public launch Done**.
- [ ] Treating an unsigned local `.app` / `.dmg` as satisfying layer (3).

### B. After freeze + layer (2) `main` catch-up

Only when freeze has ended **and** related clusters are on `main`:

- [ ] Follow [`18-packaging-runbook.md`](18-packaging-runbook.md): release build → Developer ID codesign → notarize → staple → Gatekeeper assess.
- [ ] Publish the **notarized** artifact as a GitHub Release (or equivalent maintainer channel).
- [ ] **Then** flip repo / Release visibility (layer 3).
- [ ] Re-check public copy: personal self-tracking framing; AGPLv3; no workplace / clinical claims; no secret-math implication.

### C. Hard rule

**Unsigned / local / ad-hoc dry-runs never satisfy layer (3).** Layer (3) requires a notarized Release artifact **before** visibility flip.

### D. Maintainer dry-run notes (2026-08-12 — P26-E3)

Local probe only — **no** Release cut, **no** visibility flip, **no** secrets recorded.

| Check | Result |
| :--- | :--- |
| Calendar | 2026-08-12 — **inside PR freeze** (through 2026-09-01 inclusive) |
| `apps/desktop` present | Yes |
| Local unsigned release `.app` this pass | **Not found** — full `pnpm tauri build` not re-run in this task (optional; dogfood path remains per packaging runbook) |
| Codesigning identities visible | **Apple Development** identity present; **no Developer ID Application** identity observed in Keychain inventory |
| `notarytool` CLI | Available via `xcrun` |
| Notarization credentials / Apple ID app-specific password | **Not verified / not used** this pass — would be required for layer (3); must stay in Keychain / CI secrets, never git |
| Blockers for layer (3) now | (1) PR freeze + no layer (2) `main` catch-up yet; (2) missing Developer ID Application for distribution signing; (3) notarization credentials not exercised |
| Verdict | Dry-run checklist documented; **public launch not Done**; unsigned rehearsal ≠ notarized Release |

## Related docs

- Scope lock: **ADR-027** — [`docs/decision-log.md`](decision-log.md)
- Packaging ops: [`docs/18-packaging-runbook.md`](18-packaging-runbook.md) (dry-run references this file — ops stay there)
- Vision (OSS audience + Non-Goals): [`docs/00-vision.md`](00-vision.md)
- Dev / Git / PR freeze: [`docs/12-development.md`](12-development.md)
- Contributing: [`CONTRIBUTING.md`](../CONTRIBUTING.md)
- Security / privacy: [`docs/10-security.md`](10-security.md)
- Glossary: [`docs/16-glossary.md`](16-glossary.md)

## Phase 26 epic sketch

```text
E1 — ADR-027 lock (Done)
E2 — THIS SoT + hygiene touchpoints (Done; public launch not Done)
E3 — Dry-run checklist vs packaging runbook (this section)
      unsigned/local notes OK in freeze; notarized public Release only after freeze + layer 2
```
