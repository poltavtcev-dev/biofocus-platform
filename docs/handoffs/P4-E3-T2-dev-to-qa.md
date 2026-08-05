# Dev → QA: P4-E3-T2

## Meta
- **Task ID:** P4-E3-T2
- **Title:** Optional local LLM adapter
- **Role that built:** Dev
- **Date:** 2026-08-05
- **AC source:** `/docs/SPRINT_ROADMAP.md` → Epic P4-E3 / P4-E3-T2; brief `docs/handoffs/P4-E3-T2-pm-brief.md`
- **Branch:** `phase/4-dashboard-ai`

## What changed
- Optional opt-in local LLM path in `crates/report-engine`: OpenAI-compatible `POST …/chat/completions` (default Ollama `http://127.0.0.1:11434/v1`).
- Default **OFF** — `LocalLlmConfig::from_env()` / `disabled()`; when disabled returns `ReportEngineError::LocalLlmDisabled` with **no HTTP**.
- Consumes `ReportDocument::llm_prompt` only via `interpret_report` / `interpret_llm_prompt` (async `Result`); no Feature math.
- HTTP timeout via reqwest client timeout; typed errors (`LocalLlmHttp` / `LocalLlmTimeout` / `LocalLlmResponse`).
- Documented: never auto-send on startup — host must call explicitly (Dashboard UX → T3).
- Privacy note in `docs/12-development.md` (+ short note in `docs/10-security.md`, API sketch in `docs/09-api.md`).

### Crates / files touched
- `crates/report-engine/Cargo.toml` — `reqwest`, `serde`; dev: `axum`, `tokio`
- `crates/report-engine/src/llm.rs` — config, interpret API, mock-server unit tests
- `crates/report-engine/src/error.rs` — LLM error variants
- `crates/report-engine/src/lib.rs` — re-exports + crate docs
- `crates/report-engine/src/builder.rs` — doc cross-link
- `docs/12-development.md` — enable + privacy
- `docs/10-security.md` — opt-in local LLM
- `docs/09-api.md` — interpret API

## Public API (for QA)

```rust
use report_engine::{
    build_report, interpret_report, LocalLlmConfig, ReportEngineError,
};

let doc = build_report(&features, &insights)?;
let cfg = LocalLlmConfig::from_env(); // default enabled=false
match interpret_report(&doc, &cfg).await {
    Err(ReportEngineError::LocalLlmDisabled) => { /* expected when OFF */ }
    Ok(text) => { /* interpreted summary */ }
    Err(other) => { /* timeout / HTTP / response */ }
}
```

| Env | Default | Notes |
| :--- | :--- | :--- |
| `BIOFOCUS_LOCAL_LLM` | unset → OFF | `1` / `true` / `yes` / `on` |
| `BIOFOCUS_LOCAL_LLM_BASE_URL` | `http://127.0.0.1:11434/v1` | OpenAI-compatible base |
| `BIOFOCUS_LOCAL_LLM_MODEL` | `llama3.2` | chat model id |
| `BIOFOCUS_LOCAL_LLM_TIMEOUT_SECS` | `30` | HTTP timeout |

## How to verify (commands)
```bash
cargo test -p report-engine
cargo check -p report-engine
```

### Optional manual smoke (real Ollama)
```bash
# terminal A
ollama serve   # if not already
ollama pull llama3.2

# terminal B — tiny Rust/REPL not required; unit tests cover mock path.
# For live call from a one-off, enable:
export BIOFOCUS_LOCAL_LLM=1
# then host code: interpret_report(&doc, &LocalLlmConfig::from_env()).await
```
Unit tests already cover enabled path against an in-process mock OpenAI server (no Ollama required for Pass).

## Acceptance Criteria checklist (for QA)
- [ ] AC1: Opt-in via env/config; default **OFF** (no network when disabled)
- [ ] AC2: Prefer localhost Ollama / OpenAI-compatible base URL; configurable base + model
- [ ] AC3: Consumes `ReportDocument::llm_prompt`; returns interpreted text via `Result`; **no Feature math**
- [ ] AC4: HTTP timeout; never auto-send on startup (documented; no startup call in desktop)
- [ ] AC5: Errors via `thiserror`; no production `unwrap` / `expect`
- [ ] AC6: Privacy note in `docs/12-development.md`
- [ ] AC7: Handoff with tests/smoke + how to enable
- [ ] Global DoD: UI↛DB; glossary; out of scope (T3 UX, SQLite, Feature math) untouched

## Risks / not covered
- Desktop / Dashboard button **not** wired (→ P4-E3-T3). Crate API only.
- Live Ollama not required for automated Pass; mock HTTP covers contract.
- `LocalLlmTimeout.timeout_ms` uses saturating `u64` cast from `Duration::as_millis`.

## Notes for QA
- Placement: **`crates/report-engine`** (not desktop host).
- Confirm disabled path does not hit mock server (`hits == 0` test).
- Confirm request body has `messages` and no `features` field (asserted in mock).
