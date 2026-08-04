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

Window title / keystroke aggregates → **P2-E2-T2** (may require Accessibility — document then).
