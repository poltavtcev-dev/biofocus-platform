# QA → PM: P22-E2-T1

## Meta
- **Task ID:** P22-E2-T1
- **Title:** Ship catalog Feature `AttentionStability` per ADR-023
- **Date:** 2026-08-11
- **Dev/UX handoff:** `docs/handoffs/P22-E2-T1-dev-to-qa.md`
- **Verdict:** Pass

## What was verified
- Commands run + results:
  - `cargo test -p feature-engine attention_stability` — **10/10 ok**
  - `cargo test -p feature-engine deep_work_score` — **6/6 ok** (sibling untouched)
  - `register_attention_stability_v1` + `register_catalog_v1` wiring present (after DeepWork)
  - Catalog §1.18 marked **shipped** with full formula / factors / DAG names
  - No `AttentionStability` in `bio-spec`; no migration files; no UI/apps dirt
  - No prod `.unwrap()` / `.expect()` in `attention_stability.rs`
  - Leaf files `focus_score` / `context_switch_rate` / `deep_work_score` unmodified
- AC results:

| AC | Result |
| :--- | :--- |
| AC1 Feature-level AttentionStability; 15m/1m; 0–100 | **Pass** |
| AC2 range/single-sample/CSR weights; omit without Focus; renormalize; no CSR-only; not DeepWork intensity | **Pass** |
| AC3 ADR-007 expected_slots=2; coverage × mean upstream conf | **Pass** |
| AC4 Calm factors (`Focus consistency` / `Switch steadiness`) | **Pass** |
| AC5 register helpers + catalog §1.18 shipped | **Pass** |
| AC6 Unit tests (Focus+CSR / Focus-only / single-vs-multi / omit / confidence / factors / sibling distinct) | **Pass** |
| AC7 No leaf/DeepWork rewrite; no new data_type; no migration; no UI | **Pass** |
| Global DoD | **Pass** |

- Extra checks:
  - `distinct_from_deep_work_focus_level_intensity` — single Focus sample → Attention=100 vs DeepWork=Focus level
  - Factor labels reject ADHD / burnout / “can’t focus” phrasing in tests
  - Clinical terms appear only in comments / negative assertions, not user-facing labels

## Defects (if any)
- None.

## What PM must update
- [x] `/docs/SPRINT_ROADMAP.md` — move **P22-E2-T1** to Done; Ready **P22-E3-T1**
- [x] Execution canvas `biofocus-execution-board.canvas.tsx` — QUEUE, todos, stats, callout, DAG
- [x] Other docs if needed: `14-roadmap` / `ARCHITECTURE_STATUS` / `PROJECT_CANVAS` / glossary — AttentionStability **shipped** (E2); E3 dogfood optional
- [x] Optional: confirm `docs/12-development.md` Phase 22 note (already updated by Dev) aligns with Done

## Suggested next Ready task
- **P22-E3-T1** — Dogfood / optional calm Dashboard surface for `AttentionStability` (no formula rewrite; no migration).

## Notes for PM
- Snapshot/IPC picks up `AttentionStability` via existing `register_catalog_v1` — E3 is UX/dogfood.
- Focus-slot confidence = mean of in-window Focus samples (documented in catalog §1.18).
- PR freeze still active until 2026-09-01 — no PR.
