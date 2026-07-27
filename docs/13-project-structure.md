---

### `docs/13-project-structure.md`

```markdown
# 13. Project Workspace Structure

```text
biofocus-platform/
├── apps/
│   └── desktop/            # Tauri v2 Frontend (React / TypeScript / Tailwind)
├── crates/
│   ├── bio-spec/           # Canonical Types & Contracts
│   ├── runtime/            # Tokio Async Runtime & Event Bus
│   ├── storage/            # SQLite Migrations & Repositories
│   ├── pipeline/           # Quality, Deduplication & Normalization
│   ├── feature-engine/     # Feature Calculations (Focus, Stress, Fatigue)
│   ├── knowledge-engine/   # Evidence & Insights Generator
│   ├── report-engine/      # Markdown & AI LLM Prompt Generators
│   └── plugin-sdk/         # Interface Traits for Device Adapters
├── docs/                   # Engineering Documentation
├── ARCHITECTURE_STATUS.md  # Architectural Guardrails
└── Cargo.toml              # Root Workspace Config