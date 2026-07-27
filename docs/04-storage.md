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