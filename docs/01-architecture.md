# 01. System Architecture & Boundaries

## 1. Architectural Style
Modular Monolith / Event-Driven Local Data Pipeline (Rust Crates + Tauri v2 Desktop UI).

## 2. High-Level Data Flow
```text
[ Provider / Plugin ]
        │
        ▼
  Observation (Immutable Fact)
        │
        ▼
  Data Quality & Normalization Pipeline
        │
        ▼
  Signal Engine (Change / Anomaly Detection)
        │
        ▼
  Feature Engine (DAG Compute Runtime)
        │
        ▼
  Knowledge Engine (Pattern & Evidence Rules)
        │
        ▼
  Report Engine / AI Provider (Optional LLM)
        │
        ▼
  Desktop UI (Tauri v2 / React)