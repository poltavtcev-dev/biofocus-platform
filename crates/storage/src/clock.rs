//! Wall-clock helpers for storage metadata (`created_at`, migration `applied_at`).

use std::time::{SystemTime, UNIX_EPOCH};

use crate::{StorageError, StorageResult};

/// Current Unix UTC seconds, or [`StorageError::Clock`] if the system clock is unavailable.
pub(crate) fn unix_now_secs() -> StorageResult<i64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .map_err(|source| StorageError::Clock { source })
}
