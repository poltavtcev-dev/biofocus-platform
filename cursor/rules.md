# Cursor Rules for BioFocus Platform

## Core Directives
1. **Architecture Compliance:** Before implementing any code, read the specifications in `/docs/`. You MUST strictly follow the boundaries, data contracts, and crate architecture defined in `/docs/`.
2. **Domain Terms:** Use Ubiquitous Language from `/docs/16-glossary.md` (`Observation`, `Signal`, `Feature`, `Insight`).
3. **No Architecture Breaking:** You are NOT allowed to change database schemas, API contracts, or domain entities without explicit user approval via an ADR (Architecture Decision Record).
4. **Error Handling:** Do not use `unwrap()` or `expect()` in production code. Use `thiserror` and explicit `Result<T, E>`.
5. **No Direct UI-DB Access:** The UI layer (`apps/desktop`) must communicate with the Rust backend strictly via Tauri IPC commands.