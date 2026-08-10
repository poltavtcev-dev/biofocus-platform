---

### `docs/04-storage.md`

```markdown
# 04. Storage Architecture & Database Schema

## 1. Engine & Configuration
- **Database Engine:** Embedded SQLite (`rusqlite`)
- **Journal Mode:** `WAL` (Write-Ahead Logging)
- **Synchronous Mode:** `NORMAL`
- **Location:** `~/.biofocus/data/biofocus_main.db`

## 2. Core Tables Schema

### Table: `observations`
```sql
CREATE TABLE IF NOT EXISTS observations (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp INTEGER NOT NULL,
    provider_id TEXT NOT NULL,
    data_type TEXT NOT NULL, -- 'heart_rate', 'hrv', 'context_window', 'keystrokes'
    payload JSON NOT NULL,
    confidence REAL DEFAULT 1.0,
    created_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_obs_ts ON observations(timestamp);
CREATE INDEX IF NOT EXISTS idx_obs_type_ts ON observations(data_type, timestamp);
```

### Pattern Discovery / Feature history (ADR-008)

- **v1:** Observations remain the only durable history. Features and baselines are **not** persisted as SQLite rows.
- **No migration** for Feature history / baseline tables under ADR-008.
- Future optional daily rollup table requires a **new ADR + user approve** — not part of Pattern Discovery v1.

### Recommendations (ADR-009)

- **v1:** Recommendations are evaluate-on-read (in-memory / IPC) — **no** `recommendations` table and **no** recommendation-history columns.
- **No migration** under ADR-009.

### Plugin wave-1 / browser categories (ADR-010)

- **v1:** Browser category facts are ordinary rows in `observations` (`data_type = 'browser_category'`). **No** plugin registry table, browsing-history table, or category columns beyond payload JSON.
- **No migration** under ADR-010.
- Future optional host allowlist / config store requires a **new ADR + user approve** — not part of wave-1 v1.

### Ambient Now Playing + packaging (ADR-012)

- **v1:** Now Playing ambient facts are ordinary rows in `observations` (`data_type = 'now_playing'`). **No** ambient media table, sync mirror, or packaging registry.
- **No migration** under ADR-012. Optional sync store / outbox requires a **new ADR + user approve** — out of Phase 12 v1.
- Commercial packaging in Phase 12 is runbook/process only — does not add SQLite schema.

### Plugin wave-2 / Git activity (ADR-013)

- **v1:** Git activity facts are ordinary rows in `observations` (`data_type = 'git_activity'`). **No** plugin registry table, remotes table, or repo-path allowlist columns.
- **No migration** under ADR-013.

### Git watched-roots allowlist (ADR-014 / Phase 14)

- **v1 durable store:** local config file `~/.biofocus/git-watched-roots.toml` (user-chosen absolute roots only) — **not** SQLite. Live load + probe **shipped** (P14-E2).
- **No migration** under ADR-014. Observations remain the only durable git **facts** store.
- Future SQLite allowlist table (if ever needed for CRUD) requires a **new ADR + user approve** — not Phase 14 v1.