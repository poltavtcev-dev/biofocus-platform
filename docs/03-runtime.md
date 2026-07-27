# 03. Runtime Architecture & Event Loop

## 1. Core Runtime Stack
- **Language:** Rust (stable, 2024 edition)
- **Async Runtime:** `tokio` (multi-threaded executor)
- **Internal Bus:** `tokio::sync::broadcast` / `tracing`

## 2. Process & Thread Model
```text
┌─────────────────────────────────────────────────────────────┐
│                    Tauri Desktop Process                    │
│                                                             │
│  ┌──────────────────┐          ┌─────────────────────────┐  │
│  │ UI Thread (Web)  │ ◄─ IPC ─►│ Core Async Runtime      │  │
│  └──────────────────┘          │ (Tokio Worker Threads)  │  │
│                                └────────────┬────────────┘  │
└─────────────────────────────────────────────┼───────────────┘
                                              │
               ┌──────────────────────────────┼──────────────────────────────┐
               ▼                              ▼                              ▼
  ┌─────────────────────────┐   ┌──────────────────────────┐   ┌─────────────────────────┐
  │ Local Ingestion Server  │   │ System Context Collector │   │ Feature Engine Worker   │
  │ (Warp/Axum HTTP Server) │   │ (NSWorkspace / OS Loop)  │   │ (DAG Scheduler)         │
  └─────────────────────────┘   └──────────────────────────┘   └─────────────────────────┘