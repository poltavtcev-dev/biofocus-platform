# BioFocus — Sprint Roadmap & Kanban Matrix

> **Phase 5: Wearable dogfood** (Sprint 9–10) — **Opened** 2026-08-05.  
> Phase 1–4 Done. Goal: physical-phone path — **opt-in LAN ingest** → pairing shows LAN endpoint → **iOS HealthKit companion** posts HR `Observation`s to Desktop on the same LAN.

**Phase 5 goal:** Dogfood-ready wearable bridge without cloud. Default bind stays loopback; LAN is **explicit opt-in**. Same Observation HTTP contract (`POST /v1/ingest` + Bearer). No new SQLite tables without ADR + approve.

**Platform vision (accepted):** Personal Pattern Discovery · L1–L5 analysis stack · horizon P6–P12+ — `/docs/00-vision.md`. **Do not** pull P6+ Features/plugins into this Kanban until PM opens that phase.

**Global DoD (каждая задача):**
- [ ] Freeze `/docs/ARCHITECTURE_STATUS.md` + Ubiquitous Language (`Observation` / `Signal` / `Feature` / `Insight`)
- [ ] Нет `unwrap()` / `expect()` в production
- [ ] UI ↛ SQLite (только IPC); Features/Insights считаются в Core
- [ ] Тесты зелёные; `cargo check` / релевантный CI
- [ ] **Idle footprint:** нет busy-loop; poll/refresh по событию или редкому таймеру
- [ ] **Нет новой SQLite-схемы** без ADR + approve
- [ ] **Commit / PR по связанному кластеру** — `docs/12-development.md`
- [ ] Copy спокойный, неоценочный (не «ты выгорел» / clinical claims)
- [ ] **LAN / companion:** Bearer обязателен; нет cloud telemetry; default = loopback

---

## Kanban Overview (Phase 5 active)

| Status | IDs |
| :--- | :--- |
| **Ready** | **P5-E3-T2** |
| **In Progress** | — |
| **Blocked** | — |
| **Done** | Phase 0 · **Phase 1** · **Phase 2** ([PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2)) · **Phase 3** (E1–E3) · **Phase 4** (E1–E3) · **P5-E1** (T1–T2) · **P5-E2** (T1) · **P5-E3-T1** |

**Epic status:** P5-E1 ✅ · P5-E2 ✅ · P5-E3 ⬜ (T1 Done · T2 Ready)

**Phase 5 on `/docs/14-roadmap.md`:** opened 2026-08-05

**Рекомендуемый порядок:**  
~~P5-E1-T1~~ → ~~P5-E1-T2~~ → ~~P5-E2-T1~~ → ~~P5-E3-T1~~ → **P5-E3-T2**

**Live board:** [`biofocus-execution-board.canvas.tsx`](/Users/maksimpoltavcev/.cursor/projects/Users-maksimpoltavcev-Desktop-BioFocus/canvases/biofocus-execution-board.canvas.tsx)

**Agent pipeline:** Dev|UX → QA → PM. См. `/docs/17-agent-workflow.md`. Git: **related work → PR** (`docs/12-development.md`).  
**Suggested branch:** `phase/5-wearable-dogfood` (base on latest `main`; Phase 4 cluster PR may land separately on `phase/4-dashboard-ai`).

### Active assignment

**Ready now:** **P5-E3-T2** — Dogfood runbook + contract docs (Dev).  
Brief: `docs/handoffs/P5-E3-T2-pm-brief.md`.

**Closed:** P5-E3-T1 (QA Pass with notes, 2026-08-05) — runnable `BioFocusCompanion.xcodeproj` + HealthKit one-shot → ingest. Evidence: `docs/handoffs/P5-E3-T1-qa-to-pm.md`. Live Simulator/device E2E left for operator (iOS platform component / signing).

**Previously closed:** P5-E2-T1 — Companion LAN base URL + token/QR; Epic **P5-E2** ✅ (`docs/handoffs/P5-E2-T1-qa-to-pm.md`). P5-E1-T2 — `bind_mode` / `base_url_hints`; Epic **P5-E1** ✅ (`docs/handoffs/P5-E1-T2-qa-to-pm.md`). P5-E1-T1 — opt-in LAN bind; ADR-005 (`docs/handoffs/P5-E1-T1-qa-to-pm.md`).

**Ops note:** Phase 4 cluster PR on `phase/4-dashboard-ai` remains optional parallel ops — does not block Phase 5.

---

## Epic P5-E1 — Opt-in LAN ingest ✅ Done

**Цель:** Physical phone on the same Wi-Fi can reach Desktop `/v1/ingest`. Default remains **loopback-only**; LAN bind is opt-in and documented.

### P5-E1-T1 — Opt-in LAN bind + config ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/ingest`, desktop host wire (`apps/desktop/src-tauri`), docs (`10-security`, `12-development`, `09-api` as needed) |
| **Depends on** | Phase 2 ingest (loopback serve + Bearer) |
| **AC** | (1) Default bind remains `127.0.0.1` (Simulator / same-machine unchanged). (2) Explicit opt-in enables LAN-reachable bind (e.g. env/config such as `BIOFOCUS_INGEST_LAN=1` and/or bind-host override — document the chosen knobs). (3) Bearer pairing token **still required** for `POST /v1/ingest`; no anonymous LAN ingest. (4) Without opt-in, behavior identical to today (loopback-only). (5) Unit/integration coverage: default loopback + opt-in LAN bind path (ephemeral port OK). (6) Idle-safe (tokio accept; no busy-loop). (7) Update `docs/10-security.md` + short note in `docs/12-development.md` (and `09-api.md` if bind surface changes). (8) Record **ADR-005** in `docs/decision-log.md`: opt-in LAN ingest for companion dogfood; default loopback. (9) No new SQLite schema; no iOS app work in this task. Handoff: `docs/handoffs/P5-E1-T1-dev-to-qa.md`. |
| **Out of scope** | Companion UI LAN URL/QR (→ **E2-T1**), iOS runnable app (→ **E3**), cloud / mDNS discovery, TLS termination, new Observation types |
| **Shipped** | `IngestConfig.bind_host`; `BIOFOCUS_INGEST_LAN` → `0.0.0.0`; `BIOFOCUS_INGEST_BIND_HOST` override; ADR-005; QA Pass with notes 2026-08-05. Pairing `ingestBaseUrl` still loopback → **T2**. |

### P5-E1-T2 — Advertise bind mode + base URL hints ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `crates/ingest`, desktop host, optionally `/v1/status` / IPC |
| **Depends on** | P5-E1-T1 |
| **AC** | When LAN opt-in is on, host exposes enough for dogfood: bind mode + usable base URL hint(s) (e.g. primary LAN IPv4 + port) via documented status/IPC — **no** Observation payloads, **no** absolute DB paths. Loopback mode still reports loopback URL. Tests for status shape. Idle-safe. Docs note how to read the hint. |
| **Out of scope** | Full pairing QR redesign (→ E2), iOS UI |
| **Shipped** | `AdvertiseInfo` / `BindMode`; `/v1/status` + pairing IPC `bindMode` / `baseUrlHints` / primary `ingestBaseUrl`; QA Pass with notes 2026-08-05. Epic **P5-E1** closed. |

---

## Epic P5-E2 — Pairing UX for LAN dogfood ✅ Done

**Цель:** Desktop Companion section makes physical-phone pairing obvious: LAN base URL + token (copy / QR).

### P5-E2-T1 — Companion UI: LAN base URL + token/QR ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | UX + Dev |
| **Modules** | `apps/desktop` (Companion section + IPC glue) |
| **Depends on** | P5-E1-T2 (URL hint); token IPC already exists (`get_pairing_token`) |
| **AC** | Companion UI shows copyable LAN (or loopback) base URL when available; token Show/Copy/QR still works; calm copy that LAN is opt-in / local network only; UI↛DB; smoke steps in handoff. No cloud account. |
| **Out of scope** | iOS app (→ E3), changing ingest auth scheme |
| **Shipped** | Companion Base URL block (`ingestBaseUrl` / bindMode / LAN fallback hint); token Show/Copy/QR preserved; idle-safe (no pairing poll); QA Pass with notes 2026-08-05. Epic **P5-E2** closed. |

---

## Epic P5-E3 — iOS HealthKit companion (dogfood)

**Цель:** Runnable iOS path: HealthKit heart-rate sample → same Observation JSON → `POST /v1/ingest` over LAN (or Simulator loopback).

### P5-E3-T1 — Runnable iOS companion + HealthKit one-shot ✅ Done
| Field | Value |
| :--- | :--- |
| **Role** | Dev (+ UX for minimal UI copy) |
| **Modules** | `apps/companion/ios/` |
| **Depends on** | P5-E1-T1 (LAN); E2-T1 helpful for pairing UX |
| **AC** | Runnable Xcode target (or clearly documented project) using existing Swift contract sources; user can set base URL + paste pairing token; one-shot HealthKit HR → Observation array POST; surface 401 / network errors; **no** busy-loop HealthKit polling; privacy: HR read only per existing stub; local Desktop only. Handoff with Simulator and/or device smoke notes. |
| **Out of scope** | App Store release, background continuous streaming, vendor wearable SDKs beyond HealthKit, new SQLite |
| **Shipped** | `BioFocusCompanion.xcodeproj` + shared scheme; base URL + token UI; one-shot HK → `POST /v1/ingest`; 401/network in Status; QA Pass with notes 2026-08-05 (`swiftc` + `cargo test -p companion`; live Simulator deferred to host platform install). |

### P5-E3-T2 — Dogfood runbook + contract docs ✅ Ready
| Field | Value |
| :--- | :--- |
| **Role** | Dev |
| **Modules** | `docs/` (`12-development`, companion READMEs, `PROJECT_CANVAS` wearable §) |
| **Depends on** | P5-E3-T1 |
| **AC** | End-to-end runbook: enable LAN → note URL → pair token → iPhone/Simulator post → Observation visible (status / Dashboard / storage path as appropriate). Update companion READMEs; no personal device inventory in git. |
| **Out of scope** | New product Features/Insights; non-HealthKit bridges |

---

## Phase 4 archive (Done)

<details>
<summary>Phase 4 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P4-E1 (T1–T3) · P4-E2 (T1–T3) · P4-E3 (T1–T3).  
Feature snapshot IPC → Dashboard + Recharts → Knowledge Insights → report-engine + optional local LLM + Report UX.

Evidence: `docs/handoffs/P4-*-qa-to-pm.md` · branch `phase/4-dashboard-ai` (cluster PR when ready).

</details>

---

## Phase 3 archive (Done)

<details>
<summary>Phase 3 Kanban & epics (closed 2026-08-05 — E1–E3)</summary>

**Done:** P3-E1 (T1–T4) · P3-E2 (T1–T4) · P3-E3 (T1–T3).  
Pipeline quality → Feature DAG → Menubar AlertLevel IPC + UX.

Evidence: `docs/handoffs/P3-*-qa-to-pm.md` · PRs #5–#23 (cluster) · Menubar via [PR #24](https://github.com/poltavtcev-dev/biofocus-platform/pull/24).

</details>

---

## Phase 2 archive (Done)

<details>
<summary>Phase 2 Kanban & epics (closed 2026-08-04, PR #2)</summary>

**Done:** P2-E0 · P2-E1 (T1–T4) · P2-E2 (T1–T3) · P2-E3 (T1–T2).  
Evidence: `docs/handoffs/P2-*-qa-to-pm.md` · [PR #2](https://github.com/poltavtcev-dev/biofocus-platform/pull/2).

Ingest loopback + pairing · macOS collector · companion sample + Copy/QR · `dbError` sanitize.

</details>

---

## Phase 1 archive (Done)

<details>
<summary>Phase 1 Kanban & epics (closed 2026-08-04)</summary>

**Done:** Epic E1–E4. Evidence: `docs/handoffs/P1-E4-T1-acceptance.md`, `P1-E4-T2-qa-to-pm.md`.

Epics: workspace/`bio-spec`/`runtime` → SQLite WAL + `ObservationRepository` → Tauri Menubar + `get_status` → exit docs.

</details>

---

## Role × Module Matrix (Phase 5)

| Task | PM | Dev | QA | UX | Primary modules |
| :--- | :---: | :---: | :---: | :---: | :--- |
| P5-E1-T1 Opt-in LAN bind | | ● | ○ | | ingest + desktop host |
| P5-E1-T2 Advertise URL hints | | ● | ○ | | ingest + host / status |
| P5-E2-T1 Companion LAN UX | ○ | ○ | ○ | ● | apps/desktop |
| P5-E3-T1 iOS HealthKit runnable | | ● | ○ | ○ | apps/companion/ios |
| P5-E3-T2 Dogfood runbook | | ● | ○ | | docs + companion READMEs |

● = owner · ○ = collaborator

---

## Sprint 9–10 — Queue

1. ~~P5-E1-T1 — Opt-in LAN ingest bind + config~~ **Done** (QA Pass with notes)  
2. ~~P5-E1-T2 — Advertise bind mode + base URL hints~~ **Done** (QA Pass with notes) · Epic **P5-E1** ✅  
3. ~~P5-E2-T1 — Companion UI: LAN base URL + token/QR~~ **Done** (QA Pass with notes) · Epic **P5-E2** ✅  
4. ~~P5-E3-T1 — Runnable iOS companion + HealthKit one-shot~~ **Done** (QA Pass with notes)  
5. **P5-E3-T2 — Dogfood runbook + contract docs** ← **Ready**  

**Git:** `phase/5-wearable-dogfood` → related commits → **one cluster PR** when E1–E3 (or coherent subset) is Ready to ship.
