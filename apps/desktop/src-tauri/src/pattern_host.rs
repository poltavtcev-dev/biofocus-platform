//! Pattern Discovery v1 host helpers (ADR-008 / P8-E2-T1).
//!
//! Loads local Observations and recomputes a bounded FocusScore afternoon
//! baseline on read. Optional in-process memo (TTL + Observation watermark) —
//! not SQLite. Soft-fails to an empty series (idle / privacy safe).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use bio_spec::{Feature, Observation, UnixTimestamp};
use feature_engine::{
    baseline_lookback_start, recompute_focus_afternoon_baseline, BASELINE_MAX_WINDOWS,
};
use storage::{Database, ObservationRepository};
use tracing::warn;

/// Process-local memo TTL for baseline recompute (session cache only).
const BASELINE_MEMO_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Default)]
struct MemoEntry {
    watermark: i64,
    series: Vec<Feature>,
    computed_at: Option<Instant>,
}

/// In-process memo for recomputed baseline series (not persisted).
#[derive(Debug, Default)]
pub struct BaselineMemoState {
    inner: Mutex<MemoEntry>,
}

impl BaselineMemoState {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Return cached series when TTL valid and watermark matches.
    fn get_if_fresh(&self, watermark: i64) -> Option<Vec<Feature>> {
        let guard = self.inner.lock().ok()?;
        let computed_at = guard.computed_at?;
        if guard.watermark != watermark {
            return None;
        }
        if computed_at.elapsed() > BASELINE_MEMO_TTL {
            return None;
        }
        Some(guard.series.clone())
    }

    fn store(&self, watermark: i64, series: Vec<Feature>) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = MemoEntry {
                watermark,
                series,
                computed_at: Some(Instant::now()),
            };
        }
    }
}

/// Load Observations for the ADR-008 lookback and recompute Focus afternoon means.
///
/// Soft-fails (empty vec) on DB / recompute errors — never panics; no busy-loop.
pub fn load_focus_baseline_series(
    memo: Option<&BaselineMemoState>,
    reference_ts: i64,
) -> Vec<Feature> {
    let observations = match load_lookback_observations(reference_ts) {
        Ok(obs) => obs,
        Err(err) => {
            warn!(error = %err, "pattern: observation load failed; empty baseline");
            return Vec::new();
        }
    };

    let watermark = observations
        .iter()
        .map(|o| o.timestamp.as_secs())
        .max()
        .unwrap_or(0);

    if let Some(memo) = memo {
        if let Some(cached) = memo.get_if_fresh(watermark) {
            return cached;
        }
    }

    match recompute_focus_afternoon_baseline(&observations, reference_ts, BASELINE_MAX_WINDOWS) {
        Ok(series) => {
            if let Some(memo) = memo {
                memo.store(watermark, series.clone());
            }
            series
        }
        Err(err) => {
            warn!(error = %err, "pattern: baseline recompute failed; empty series");
            Vec::new()
        }
    }
}

fn load_lookback_observations(reference_ts: i64) -> Result<Vec<Observation>, String> {
    let path = storage::default_db_path().map_err(|err| err.public_message())?;
    let db = Database::open(&path).map_err(|err| err.public_message())?;
    let repo = ObservationRepository::new(&db);
    let start = UnixTimestamp::from_secs(baseline_lookback_start(
        reference_ts,
        BASELINE_MAX_WINDOWS,
    ));
    let end = UnixTimestamp::from_secs(reference_ts);
    repo.list_by_time_range(start, end)
        .map_err(|err| err.public_message())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memo_misses_on_watermark_change() {
        let memo = BaselineMemoState::new();
        memo.store(100, Vec::new());
        assert!(memo.get_if_fresh(100).is_some());
        assert!(memo.get_if_fresh(101).is_none());
    }
}
