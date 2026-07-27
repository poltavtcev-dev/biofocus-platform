---

### `docs/08-plugin-sdk.md`

```markdown
# 08. Plugin SDK & Capability Model

## 1. Plugin Interface (Rust Trait)

```rust
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Capability {
    pub name: String,
    pub data_types: Vec<String>,
}

#[async_trait]
pub trait BioFocusPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<Capability>;
    
    async fn start_stream(&self, tx: tokio::sync::mpsc::Sender<Observation>) -> Result<(), PluginError>;
    async fn stop_stream(&self) -> Result<(), PluginError>;
}