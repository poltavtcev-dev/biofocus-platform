# BioFocus Architecture Status

**Current Version:** MVP 1.0  
**Status:** Frozen (Зафиксировано)

## Implementation snapshot
- **Phase 1:** Done (E1–E4, 2026-08-04).
- **Phase 2:** **Done on `main`** via [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2) (2026-08-04) — E0–E3 (ingest, collector, companion/pairing, `dbError` hygiene).
- **Phase 3:** **Done** (2026-08-05) — Pipeline & Features; E1–E3.
- **Phase 4:** **Done** (2026-08-05) — Dashboard UI & Local AI Insights; E1–E3. Branch: `phase/4-dashboard-ai` (PR after freeze).
- **Phase 5:** **Done** (2026-08-05) — Wearable dogfood; E1–E3. Branch: `phase/5-wearable-dogfood` (PR after freeze).
- **Phase 6:** **Done** (2026-08-06) — Life context; E1–E3. Tip: `phase/6-dogfood-fixes` (PR after freeze).
- **Phase 7:** **Done** (2026-08-06) — Trust layer; E1–E3 (ADR-007 confidence → Explanation factors → `RecoveryScore`). Branch: `phase/7-trust-layer` (PR after freeze).
- **Phase 8:** **Done** (2026-08-08) — Pattern Discovery v1; E1–E3 (ADR-008 recompute-on-read → `focus_vs_recent_baseline_v1` → Dashboard Insights surface). Branch: `phase/8-pattern-discovery` (PR after freeze).
- **Phase 9:** **Done** (2026-08-08) — Deterministic Recommendations; E1–E3 (ADR-009 → `focus_dip_pace_hint_v1` → `get_recommendations` + Suggestions). Branch: `phase/9-recommendations` (PR after freeze).
- **Phase 10:** **Done** (2026-08-10) — Plugin wave-1; E1–E3 (ADR-010 Browser → `BrowserCategoryPlugin` → `DistractionScore`). Branch: `phase/10-plugin-wave-1` (PR after freeze).
- **Phase 11:** **Done** (2026-08-10) — AI coaching polish; E1–E3 (ADR-011 → `build_report_with_pack` / `biofocus.default` → `get_local_llm_status` + pack-aware Report UX). Branch: `phase/11-ai-coaching-polish` (PR after freeze).
- **Phase 12:** **Done** (2026-08-10) — Ambient + commercial packaging; ADR-012 → Now Playing (`BIOFOCUS_NOW_PLAYING`) → `AmbientMediaShare` (`register_ambient_v1`) + `docs/18-packaging-runbook.md`. Branch: `phase/12-ambient-packaging` (PR after freeze).
- **Phase 13:** **Done** (2026-08-10) — Plugin wave-2 = **Git activity**; ADR-013 → `GitActivityPlugin` (`BIOFOCUS_GIT_ACTIVITY`) → `GitActivityRate` (`register_git_v1`). Branch: `phase/13-plugin-wave-2` (PR after freeze).
- **Phase 14:** **Done** (2026-08-10) — Git path-allowlist / live probe; ADR-014 → live `SystemGitActivityProbe` → dogfood + Menubar **Git folders** IPC. Branch: `phase/14-git-allowlist` (PR after freeze).
- **Phase 15:** **Done** (2026-08-11) — Companion HRV + autonomy; **ADR-016**. Branch: `phase/15-companion-hrv-autonomy` (PR after freeze).
- **Phase 16:** **Done** (2026-08-11) — Ambient light (**ADR-015**); collector + `AmbientLightShare`. Branch: `phase/16-ambient-light` (PR after freeze).
- **Phase 17:** **Done** (2026-08-11) — Wearable depth + chart ranges (**ADR-017** · **ADR-018**); E1–E3. Branch: `phase/17-wearable-charts` (PR after freeze).
- **Phase 18:** **Done** (2026-08-11) — Notification pressure (**ADR-019**); E1–E3. Branch: `phase/18-notification-pressure` (PR after freeze).
- **Phase 19:** **Done** (2026-08-11) — Live NC OS mapping (**ADR-020**); E1–E3. Branch: `phase/19-live-nc-mapping` (PR after freeze).
- **Phase 20:** **Done** (2026-08-11) — CognitiveLoad; **ADR-021** ✅; Feature + dogfood + Combined demand chart. Branch: `phase/20-cognitive-load` (PR after freeze).
- **Phase 21:** **Done** (2026-08-11) — DeepWorkScore; **ADR-022** ✅; Feature + dogfood + Sustained focus chart. Branch: `phase/21-deep-work-score` (PR after freeze).
- **Phase 22:** **Done** (2026-08-11) — AttentionStability; **ADR-023** ✅; Feature + dogfood + Focus stability chart. Branch: `phase/22-attention-stability` (PR after freeze).
- **Phase 23:** **Done** (2026-08-11) — Personal Context Layer; **ADR-024** ✅; `DeskAwayPresence` + health→prompt + dogfood / Away from desk. Branch: `phase/23-personal-context` (PR after freeze).
- **Phase 24:** **Active** (2026-08-11) — CircadianOffset; **ADR-025** ✅; Feature shipped (`register_circadian_v1`); Ready **P24-E3-T1**. Branch: `phase/24-circadian-offset` (PR after freeze). Deferred: IDE · weather · App Store · Companion polish · TypingRhythm · precise GPS.
- **Gate:** **PM-GATE-POST-P23** ✅ chose **`CircadianOffset`**. **ADR-025** ✅.
## Core Decisions
- Local First Architecture
- Rust Runtime (Tokio)
- SQLite Storage (WAL Mode)
- Tauri v2 (Desktop UI)
- Feature Pipeline & Knowledge Engine
- Capability Plugin Model
- Platform Vision ladder (Personal Pattern Discovery) — `/docs/00-vision.md`

## Modification Policy
Все изменения архитектуры выполняются только через ADR (Architecture Decision Records). 
AI-помощникам (Cursor / Claude Code) запрещено самостоятельно изменять архитектурные границы и сущности проекта.
