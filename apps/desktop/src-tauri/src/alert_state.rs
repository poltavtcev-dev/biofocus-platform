//! Shared Menubar alert level updated by the Feature Worker (P3-E3-T2).
//!
//! Latest [`AlertLevel`](feature_engine::AlertLevel) is computed in-process from
//! normalized Observation batches (catalog Features + Signals). Exposed to the
//! UI only via IPC `get_status` — never Observation payloads.

use std::sync::{Arc, Mutex};

use feature_engine::{
    map_alert_level, register_catalog_v1, AlertLevel, FeatureEngine, WINDOW_SECS,
};
use pipeline::NormalizedBatch;
use runtime::{FeatureHook, Observation};
use tracing::{debug, warn};

/// Process-wide latest alert level for IPC (default Green = idle / no evidence).
#[derive(Debug, Clone)]
pub struct AlertState {
    inner: Arc<Mutex<AlertLevel>>,
}

impl AlertState {
    /// Starts at [`AlertLevel::Green`].
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AlertLevel::Green)),
        }
    }

    /// Cloneable handle for the Feature Worker hook.
    #[must_use]
    pub fn share(&self) -> Arc<Mutex<AlertLevel>> {
        Arc::clone(&self.inner)
    }

    /// Current level for `get_status` (Green if lock poisoned).
    #[must_use]
    pub fn current(&self) -> AlertLevel {
        self.inner
            .lock()
            .map(|g| *g)
            .unwrap_or(AlertLevel::Green)
    }
}

impl Default for AlertState {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature Worker hook: rolling Observation snapshot → catalog → alert map.
pub struct CatalogAlertHook {
    level: Arc<Mutex<AlertLevel>>,
    recent: Vec<Observation>,
    engine: FeatureEngine,
}

impl CatalogAlertHook {
    /// Builds a catalog-registered engine. Soft-fails registration (empty engine).
    #[must_use]
    pub fn new(level: Arc<Mutex<AlertLevel>>) -> Self {
        let mut engine = FeatureEngine::new();
        if let Err(err) = register_catalog_v1(&mut engine) {
            warn!(error = %err, "alert hook: catalog register failed; alerts stay green");
        }
        Self {
            level,
            recent: Vec::new(),
            engine,
        }
    }

    fn prune_recent(&mut self) {
        let Some(max_ts) = self.recent.iter().map(|o| o.timestamp.as_secs()).max() else {
            return;
        };
        // Keep slightly more than one Feature window for sliding ends.
        let cutoff = max_ts.saturating_sub(WINDOW_SECS + 120);
        self.recent.retain(|o| o.timestamp.as_secs() >= cutoff);

        const HARD_CAP: usize = 4096;
        if self.recent.len() > HARD_CAP {
            let excess = self.recent.len() - HARD_CAP;
            self.recent.drain(0..excess);
        }
    }
}

impl FeatureHook for CatalogAlertHook {
    fn on_normalized(&mut self, batch: &NormalizedBatch) {
        if !batch.is_empty() {
            self.recent
                .extend(batch.observations().iter().cloned());
            self.prune_recent();
        }

        match self.engine.run(&self.recent) {
            Ok(output) => {
                let next = map_alert_level(&output);
                if let Ok(mut guard) = self.level.lock() {
                    if *guard != next {
                        debug!(alert = next.as_str(), "alert level updated");
                    }
                    *guard = next;
                }
            }
            Err(err) => {
                warn!(error = %err, "alert hook: feature engine run failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bio_spec::UnixTimestamp;
    use feature_engine::HIGH_STRESS_SIGNAL_TYPE;
    use pipeline::{run_quality_pipeline, DedupeState};
    use serde_json::json;
    use uuid::Uuid;

    fn hrv(id: u128, ts: i64, rmssd: f64) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.test",
            "hrv",
            json!({ "rmssd_ms": rmssd }),
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn default_state_is_green() {
        assert_eq!(AlertState::new().current(), AlertLevel::Green);
    }

    #[test]
    fn hook_elevated_stress_becomes_yellow() {
        let state = AlertState::new();
        let mut hook = CatalogAlertHook::new(state.share());
        let mut dedupe = DedupeState::new();
        let batch = run_quality_pipeline(
            vec![hrv(1, 1500, 25.0), hrv(2, 1800, 25.0)],
            &mut dedupe,
        )
        .expect("pipeline");
        // RMSSD 25 → stress ≈ (70-25)/(70-15)*100 ≈ 81.8 > 60 → Yellow (no High_Stress span).
        hook.on_normalized(&batch);
        assert_eq!(state.current(), AlertLevel::Yellow);
    }

    #[test]
    fn hook_high_stress_signal_becomes_red() {
        let state = AlertState::new();
        let mut hook = CatalogAlertHook::new(state.share());
        let mut dedupe = DedupeState::new();
        let mut raw = Vec::new();
        for (i, ts) in (900..=1260).step_by(60).enumerate() {
            raw.push(hrv(100 + i as u128, ts, 15.0));
        }
        let batch = run_quality_pipeline(raw, &mut dedupe).expect("pipeline");
        hook.on_normalized(&batch);
        assert_eq!(state.current(), AlertLevel::Red);

        // Sanity: engine would have emitted High_Stress (covered in feature-engine).
        let mut engine = FeatureEngine::new();
        register_catalog_v1(&mut engine).expect("reg");
        let out = engine.run(batch.observations()).expect("run");
        assert!(out
            .signals
            .iter()
            .any(|s| s.signal_type == HIGH_STRESS_SIGNAL_TYPE));
    }
}
