# Dev → QA: P4-E3-T1

## Meta
- **Task ID:** P4-E3-T1
- **Title:** report-engine prompt / markdown builder
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P4-E3 / P4-E3-T1; brief `docs/handoffs/P4-E3-T1-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## What changed
- `crates/report-engine` is no longer stub-only: public `build_report(&[Feature], &[Insight]) → Result<ReportDocument>`.
- `ReportDocument { markdown, llm_prompt }` — deterministic offline markdown + interpret-only LLM prompt wrapper (no HTTP).
- Empty inputs → calm minimal report; non-empty → sorted Features table + Insights sections (stable across calls).
- `thiserror` via `ReportEngineError`; no production `unwrap` / `expect`.
- Docs: short format notes in `docs/09-api.md` (§ report-engine) and `docs/12-development.md`.

### Crates / files touched
- `crates/report-engine/Cargo.toml` — deps: `bio-spec`, `serde_json`, `thiserror`; dev: `uuid`
- `crates/report-engine/src/lib.rs` — public re-exports + crate docs
- `crates/report-engine/src/builder.rs` — `build_report`, `ReportDocument`, unit tests
- `crates/report-engine/src/error.rs` — `ReportEngineError` / `ReportResult`
- `docs/09-api.md` — crate API + output format
- `docs/12-development.md` — Phase 4 bullet + `cargo test -p report-engine`

## Output format (for QA / PM)

### `markdown`
```text
# BioFocus report

_Offline summary from Features and Insights. Not a medical assessment._

## Features
| Feature | Window (UTC s) | Value |
| :--- | :--- | :--- |
| FocusScore | 0–900 | 72.0000 |

## Insights
### <title>
<description>
- **Category:** …
- **Evidence:** feature:FocusScore | signal:<uuid>
- **Suggestion:** …   (optional)
```

Empty both slices:
```text
# BioFocus report
…
## Summary
Nothing to summarize for this period yet.
```

Determinism: Features sorted by `(feature_id, window.start, window.end)`; Insights by UUID; scalars `:.4`.

### `llm_prompt`
Instructions (interpret only / no invented metrics / calm tone) + `---` + same markdown + `---` + short summary ask. No network in this crate.

## How to verify (commands)
```bash
cargo test -p report-engine
# optional compile sanity
cargo check -p report-engine
```

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Public API Features (+ Insights) → deterministic markdown and/or LLM prompt (`Result`, offline)
- [ ] AC2: No network calls (no Ollama/OpenAI HTTP) in this crate path
- [ ] AC3: Unit tests — non-empty → stable; empty/minimal → calm empty/minimal (format documented)
- [ ] AC4: Errors via `thiserror`; no production `unwrap` / `expect`
- [ ] AC5: Format documented in handoff + short note in `docs/12-development.md` or `09-api.md`
- [ ] AC6: Handoff includes `cargo test -p report-engine`
- [ ] Global DoD: UI↛DB; glossary (`Feature` / `Insight`); no Feature math in report-engine; out of scope (T2 HTTP, T3 UX, SQLite) untouched

## Risks / not covered
- Not wired to desktop IPC / Dashboard yet (→ P4-E3-T3).
- No local LLM HTTP adapter (→ P4-E3-T2); `llm_prompt` is a string only.
- `ReportEngineError::BuildFailed` is reserved (object JSON serialize failure); happy path always `Ok` today.

## Notes for QA
- Do **not** mark Done / touch canvas (PM after your report).
- Confirm Cargo.toml has no HTTP client deps.
- `expect` only under `#[cfg(test)]` in `builder.rs`.
