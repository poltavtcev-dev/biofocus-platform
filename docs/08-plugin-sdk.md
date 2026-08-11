# 08. Plugin SDK & Capability Model

## 1. Plugin Interface (Rust Trait)

Implemented in `crates/plugin-sdk` (async fn in trait; no `async-trait` crate).

```rust
pub struct Capability {
    pub name: String,
    pub data_types: Vec<String>,
}

pub trait BioFocusPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<Capability>;

    async fn start_stream(
        &self,
        tx: runtime::ObservationSender,
    ) -> Result<(), PluginError>;

    async fn stop_stream(&self) -> Result<(), PluginError>;
}
```

Idle DoD: plugins must not busy-loop; stop must join background work.

## 2. macOS active window (P2-E2-T1)

| Item | Value |
| :--- | :--- |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.context` |
| `data_type` | `context_window` |
| Probe | `NSWorkspace.frontmostApplication` — **no Accessibility** |
| Payload | `bundle_id`, `app_name` only (see `docs/07-contracts.md`) |
| Poll | ≥1s; emit **on change** only |
| Host wire | Desktop `ingest_host` clones Observation channel → `start_stream` / `stop_stream` on app exit |

Non-macOS builds: probe returns `None` (collector still compiles; no OS emissions).

Window title capture remains deferred.

## 3. macOS input aggregates (P2-E2-T2)

| Item | Value |
| :--- | :--- |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.input` |
| `data_type` | `keystrokes` |
| Probe | Listen-only `CGEventTap` key-down counter — **Accessibility required** |
| Payload | `count`, `window_secs`, `rate_per_min` only (see `docs/07-contracts.md`) |
| Enable | `BIOFOCUS_INPUT_AGGREGATES=1` (default **off**) |
| Window | default 60s; emit when count > 0 |
| Host wire | Desktop `ingest_host` starts only when env set; same Observation channel → persist |

Non-macOS / Accessibility denied: probe returns 0 (idle).

## 4. Local Calendar ICS (P6-E3-T1)

| Item | Value |
| :--- | :--- |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.calendar` |
| `data_type` | `calendar_event` |
| Probe | Local `.ics` file (`IcsFileCalendarProbe`) or scripted fixtures |
| Payload | `uid`, `start`, `end`, `all_day`, `busy` only (see `docs/07-contracts.md`) |
| Enable | `BIOFOCUS_CALENDAR=1` **and** `BIOFOCUS_CALENDAR_ICS=/path/to/file.ics` (default **off**) |
| Poll | rare ≥60s; lookaround past 24h / future 48h; emit once per event key |
| Host wire | Desktop `ingest_host` starts only when env set + ICS path present; same Observation channel → persist |

No cloud calendar OAuth. Missing ICS path → warn and skip start (soft-fail).

Window title capture remains deferred.

## 5. Browser categories (P10 wave-1 — ADR-010 / P10-E2-T1)

| Item | Value |
| :--- | :--- |
| Status | **Shipped** (collector); Feature `DistractionScore` → **P10-E3** |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.browser` |
| `data_type` | `browser_category` |
| Probe | Frontmost known-browser → `unknown` + bundle id (no URL/title); injectable `ScriptedBrowserProbe` in tests |
| Payload | Coarse `category` (+ optional `browser_bundle_id`) only — **no** full URLs / page titles / content (see `docs/07-contracts.md`) |
| Enable | `BIOFOCUS_BROWSER_CATEGORIES=1` (default **off**) |
| Poll | On category change or rare ≥5s; no busy-loop |
| Host wire | Desktop `ingest_host` starts only when env set; same Observation channel → persist |
| E3 Feature | `DistractionScore` (catalog) |

IDE/Git collectors are **deferred** (not wave-1 v1). No plugin marketplace crate. Personal self-tracking only — not employee surveillance.

> **Update (ADR-013):** Plugin wave-2 chooses **Git activity** (`git_activity`) — see §7. IDE remains deferred.

## 6. Now Playing ambient (Phase 12 wave-1 — ADR-012 / P12-E2-T1)

| Item | Value |
| :--- | :--- |
| Status | **Shipped** (collector + Feature `AmbientMediaShare`) |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.now_playing` |
| `data_type` | `now_playing` |
| Probe | Soft-fail system probe (no content-bearing OS API in v1); injectable `ScriptedNowPlayingProbe` in tests |
| Payload | Coarse `media_kind` + `is_playing` only — **no** titles / artists / lyrics / playlists (see `docs/07-contracts.md`) |
| Enable | `BIOFOCUS_NOW_PLAYING=1` (default **off**) |
| Poll | On play-state / media-kind change or rare ≥5s; no busy-loop |
| Host wire | Desktop `ingest_host` starts only when env set; same Observation channel → persist |
| E3 Feature | `AmbientMediaShare` (catalog — `register_ambient_v1` / `register_catalog_v1`) |

Weather / light ambient collectors were **deferred** from Phase 12 wave-1. **Update (ADR-015):** Phase 15 primary = ambient **light** — see §8. Weather remains deferred. Packaging signed-build runbook: [`docs/18-packaging-runbook.md`](18-packaging-runbook.md) — secondary Phase 12 track (docs/process); AGPLv3 Core stays open.

## 7. Git activity (Phase 13 wave-2 — ADR-013 / P13-E2 shipped)

| Item | Value |
| :--- | :--- |
| Status | **Shipped** (collector P13-E2; Feature `GitActivityRate` P13-E3) |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.git` |
| `data_type` | `git_activity` |
| Probe | `SystemGitActivityProbe` — ADR-014 allowlist + live FS probe under roots; empty allowlist → soft-fail idle; `ScriptedGitActivityProbe` for tests |
| Payload | Coarse `activity_kind` (+ optional `event_count`) — **no** paths / remotes / diffs / branch names / commit messages (see `docs/07-contracts.md`) |
| Enable | `BIOFOCUS_GIT_ACTIVITY=1` (default **off**) |
| Poll | On activity change or rare ≥5s; no busy-loop; `stop_stream` joins |
| Host wire | Desktop `ingest_host` starts only when env set; same Observation channel → persist |
| Validation | `bio_spec::validate_git_activity_payload`; ingest `invalid_git_activity` |
| Feature | `GitActivityRate` (catalog §1.10; `register_git_v1` / `register_catalog_v1`) — **no** formula rewrite for Phase 14 |

### 7.1 Watched-roots allowlist (ADR-014 / P14-E2 shipped)

| Item | Value |
| :--- | :--- |
| Status | **Shipped** (live probe P14-E2; Settings/IPC + dogfood P14-E3) |
| Store | Local file `~/.biofocus/git-watched-roots.toml` (`version` + `roots` absolute dirs) — **not** SQLite |
| Env override | `BIOFOCUS_GIT_WATCHED_ROOTS` only when config **file is absent** (tests/CI); file remains SoT when present |
| Settings IPC | `get_git_watched_roots` / `set_git_watched_roots` (P14-E3) — Menubar **Git folders**; UI ↛ SQLite |
| Scope | Probe may watch only under listed roots (nested repos ≤ depth 4 / ≤ 64 repos); empty/missing → soft-fail idle |
| Privacy | Roots stay in config / Settings IPC; **never** copy into Observation payloads or default logs |
| Migration | **None** — no schema apply |
| Dogfood | `docs/12-development.md` § Git activity dogfood |

IDE collectors remain **deferred** (no additive privacy-safe session kind beyond `context_window` in v1 — ADR-013). Weather ambient and App Store packaging product remain deferred. Ambient **light** collector = ADR-015 / §8 (**shipped** P16-E1). No plugin marketplace crate. Personal self-tracking only — not employee / workplace git surveillance.

## 8. Ambient light (ADR-015 / P16-E1 shipped)

| Item | Value |
| :--- | :--- |
| Status | **Shipped** (collector P16-E1; Feature `AmbientLightShare` P16-E2) |
| Crate | `crates/macos-collector` |
| Plugin id | `com.biofocus.macos.ambient_light` |
| `data_type` | `ambient_light` |
| Probe | Soft-fail `SystemAmbientLightProbe` when OS mapping unavailable (idle, no emit); injectable `ScriptedAmbientLightProbe` for tests |
| Payload | Coarse `light_kind` (+ optional bounded `level` 0–100) — **no** camera frames / screen contents / precise geo / mic (see `docs/07-contracts.md`) |
| Enable | `BIOFOCUS_AMBIENT_LIGHT=1` (default **off**) |
| Poll | On light-band change or rare ≥5s; no busy-loop; `stop_stream` joins |
| Host wire | Desktop `ingest_host` starts only when env set; same Observation channel → persist |
| Validation | `bio_spec::validate_ambient_light_payload`; ingest `invalid_ambient_light` |
| Feature | `AmbientLightShare` (catalog §1.11; `register_ambient_light_v1` / `register_catalog_v1`) |

Weather ambient collector remains **deferred**. Personal self-tracking only — not workplace light surveillance.

