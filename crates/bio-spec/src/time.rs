//! Time primitives: Unix UTC timestamps and closed time windows.

use serde::{Deserialize, Serialize};

use crate::{SpecError, SpecResult};

/// Unix timestamp in **seconds UTC** (`i64`).
///
/// Documented contract for all BioFocus temporal fields (`docs/02-domain-model.md`,
/// `docs/07-contracts.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixTimestamp(pub i64);

impl UnixTimestamp {
    /// Creates a timestamp from Unix seconds UTC.
    #[must_use]
    pub const fn from_secs(secs: i64) -> Self {
        Self(secs)
    }

    /// Returns the inner Unix seconds value.
    #[must_use]
    pub const fn as_secs(self) -> i64 {
        self.0
    }
}

/// Inclusive time window `[start, end]` in Unix seconds UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimeWindow {
    pub start: UnixTimestamp,
    pub end: UnixTimestamp,
}

impl TimeWindow {
    /// Builds a window, rejecting `end < start`.
    pub fn try_new(start: UnixTimestamp, end: UnixTimestamp) -> SpecResult<Self> {
        if end.as_secs() < start.as_secs() {
            return Err(SpecError::InvalidTimeWindow {
                start: start.as_secs(),
                end: end.as_secs(),
            });
        }
        Ok(Self { start, end })
    }
}
