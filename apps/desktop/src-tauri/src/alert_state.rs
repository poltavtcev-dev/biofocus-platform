//! Shared Menubar alert level + Feature snapshot cache (P3-E3-T2 / P4-E1-T1).
//!
//! Latest [`AlertLevel`](feature_engine::AlertLevel) and
//! [`FeatureSnapshot`](feature_engine::FeatureSnapshot) are computed in-process
//! from normalized Observation batches (catalog Features + Signals). Exposed to
//! the UI only via IPC (`get_status` / `get_feature_snapshot` / `get_insights` /
//! `get_recommendations`) — never Observation
//! payloads.

use std::sync::{Arc, Mutex};

use feature_engine::{
    map_alert_level, register_catalog_v1, register_wearable_vitals_v1, AlertLevel, FeatureEngine,
    FeatureSnapshot, WINDOW_SECS,
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
        self.inner.lock().map(|g| *g).unwrap_or(AlertLevel::Green)
    }
}

impl Default for AlertState {
    fn default() -> Self {
        Self::new()
    }
}

/// Process-wide latest Feature snapshot for IPC (default empty = idle).
#[derive(Debug, Clone)]
pub struct SnapshotState {
    inner: Arc<Mutex<FeatureSnapshot>>,
}

impl SnapshotState {
    /// Starts empty (idle / no Feature evidence yet).
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(FeatureSnapshot::empty())),
        }
    }

    /// Cloneable handle for the Feature Worker hook.
    #[must_use]
    pub fn share(&self) -> Arc<Mutex<FeatureSnapshot>> {
        Arc::clone(&self.inner)
    }

    /// Current snapshot for `get_feature_snapshot` (empty if lock poisoned).
    #[must_use]
    pub fn current(&self) -> FeatureSnapshot {
        self.inner
            .lock()
            .map(|g| g.clone())
            .unwrap_or_else(|_| FeatureSnapshot::empty())
    }
}

impl Default for SnapshotState {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature Worker hook: rolling Observation snapshot → catalog → alert + cache.
///
/// Wearable vitals keep a separate 28-day buffer so a personal median does not
/// stretch the 15-minute window used by the rest of the catalog.
pub struct CatalogAlertHook {
    level: Arc<Mutex<AlertLevel>>,
    snapshot: Arc<Mutex<FeatureSnapshot>>,
    recent: Vec<Observation>,
    vitals: Vec<Observation>,
    engine: FeatureEngine,
    vital_engine: FeatureEngine,
}

impl CatalogAlertHook {
    /// Builds a catalog-registered engine. Soft-fails registration (empty engine).
    #[must_use]
    pub fn new(level: Arc<Mutex<AlertLevel>>, snapshot: Arc<Mutex<FeatureSnapshot>>) -> Self {
        let mut engine = FeatureEngine::new();
        if let Err(err) = register_catalog_v1(&mut engine) {
            warn!(error = %err, "alert hook: catalog register failed; alerts stay green");
        }
        let mut vital_engine = FeatureEngine::new();
        if let Err(err) = register_wearable_vitals_v1(&mut vital_engine) {
            warn!(error = %err, "alert hook: wearable vitals register failed");
        }
        Self {
            level,
            snapshot,
            recent: Vec::new(),
            vitals: Vec::new(),
            engine,
            vital_engine,
        }
    }

    /// Loads already-stored vital Observations (source choice already applied).
    pub(crate) fn seed_vitals(&mut self, observations: Vec<Observation>) {
        if observations.is_empty() {
            return;
        }
        self.vitals = observations;
        self.prune_vitals();
        self.refresh();
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

    fn prune_vitals(&mut self) {
        let Some(max_ts) = self.vitals.iter().map(|o| o.timestamp.as_secs()).max() else {
            return;
        };
        let cutoff = max_ts.saturating_sub(VITAL_HISTORY_SECS);
        self.vitals
            .retain(|o| is_vital_history(&o.data_type) && o.timestamp.as_secs() >= cutoff);
        const VITAL_CAP: usize = 12_000;
        if self.vitals.len() > VITAL_CAP {
            self.vitals.sort_by_key(|o| o.timestamp.as_secs());
            let excess = self.vitals.len() - VITAL_CAP;
            self.vitals.drain(0..excess);
        }
    }

    fn refresh(&mut self) {
        let mut output = match self.engine.run(&self.recent) {
            Ok(output) => output,
            Err(err) => {
                warn!(error = %err, "alert hook: feature engine run failed");
                return;
            }
        };
        match self.vital_engine.run(&self.vitals) {
            Ok(vital) => output.features.extend(vital.features),
            Err(err) => warn!(error = %err, "alert hook: wearable vitals run failed"),
        }
        let next = map_alert_level(&output);
        let snap = FeatureSnapshot::from_engine_output(&output);
        if let Ok(mut guard) = self.level.lock() {
            if *guard != next {
                debug!(alert = next.as_str(), "alert level updated");
            }
            *guard = next;
        }
        if let Ok(mut guard) = self.snapshot.lock() {
            *guard = snap;
        }
    }
}

const VITAL_HISTORY_SECS: i64 = 28 * 24 * 60 * 60;

fn is_vital_history(data_type: &str) -> bool {
    matches!(
        data_type,
        bio_spec::DATA_TYPE_RESTING_HEART_RATE
            | bio_spec::DATA_TYPE_HRV
            | bio_spec::DATA_TYPE_SLEEP_INTERVAL
            | bio_spec::DATA_TYPE_OXYGEN_SATURATION
            | bio_spec::DATA_TYPE_RESPIRATORY_RATE
            | bio_spec::DATA_TYPE_SLEEPING_WRIST_TEMPERATURE
    )
}

fn absorb_vitals(vitals: &mut Vec<Observation>, batch: &[Observation]) {
    for obs in batch.iter().filter(|o| is_vital_history(&o.data_type)) {
        if let Some(existing) = vitals.iter_mut().find(|kept| kept.id == obs.id) {
            *existing = obs.clone();
        } else {
            vitals.push(obs.clone());
        }
    }
}

/// Appends a normalized batch to the in-memory window, honouring
/// `life_event_retraction` markers: the target leaves the window and the
/// marker itself is never fed to the Feature Engine.
pub(crate) fn absorb_batch(recent: &mut Vec<Observation>, batch: &[Observation]) {
    let retracted: std::collections::HashSet<uuid::Uuid> = batch
        .iter()
        .filter_map(bio_spec::life_event_retraction_target)
        .collect();
    recent.extend(
        batch
            .iter()
            .filter(|o| o.data_type != bio_spec::DATA_TYPE_LIFE_EVENT_RETRACTION)
            .cloned(),
    );
    if !retracted.is_empty() {
        recent.retain(|o| !retracted.contains(&o.id));
    }
}

impl FeatureHook for CatalogAlertHook {
    fn on_source_deletions(&mut self, target_ids: &[bio_spec::ObservationId]) {
        if target_ids.is_empty() {
            return;
        }
        self.vitals.retain(|o| !target_ids.contains(&o.id));
        self.recent.retain(|o| !target_ids.contains(&o.id));
    }

    fn on_normalized(&mut self, batch: &NormalizedBatch) {
        if !batch.is_empty() {
            absorb_batch(&mut self.recent, batch.observations());
            absorb_vitals(&mut self.vitals, batch.observations());
            self.prune_recent();
            self.prune_vitals();
        }
        self.refresh();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bio_spec::UnixTimestamp;
    use feature_engine::{HIGH_STRESS_SIGNAL_TYPE, STRESS_INDEX_ID};
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
    fn default_snapshot_is_empty() {
        assert!(SnapshotState::new().current().is_empty());
    }

    #[test]
    fn hook_elevated_stress_becomes_yellow_and_caches_features() {
        let state = AlertState::new();
        let snapshots = SnapshotState::new();
        let mut hook = CatalogAlertHook::new(state.share(), snapshots.share());
        let mut dedupe = DedupeState::new();
        let batch = run_quality_pipeline(vec![hrv(1, 1500, 25.0), hrv(2, 1800, 25.0)], &mut dedupe)
            .expect("pipeline");
        // RMSSD 25 → stress ≈ (70-25)/(70-15)*100 ≈ 81.8 > 60 → Yellow (no High_Stress span).
        hook.on_normalized(&batch);
        assert_eq!(state.current(), AlertLevel::Yellow);

        let snap = snapshots.current();
        assert!(!snap.is_empty());
        assert!(snap
            .features
            .iter()
            .any(|f| f.feature_id == STRESS_INDEX_ID));
        let stress = snap
            .features
            .iter()
            .find(|f| f.feature_id == STRESS_INDEX_ID)
            .expect("StressIndex");
        assert!(!stress.provenance.is_empty());
    }

    #[test]
    fn hook_high_stress_signal_becomes_red() {
        let state = AlertState::new();
        let snapshots = SnapshotState::new();
        let mut hook = CatalogAlertHook::new(state.share(), snapshots.share());
        let mut dedupe = DedupeState::new();
        let mut raw = Vec::new();
        for (i, ts) in (900..=1260).step_by(60).enumerate() {
            raw.push(hrv(100 + i as u128, ts, 15.0));
        }
        let batch = run_quality_pipeline(raw, &mut dedupe).expect("pipeline");
        hook.on_normalized(&batch);
        assert_eq!(state.current(), AlertLevel::Red);

        let snap = snapshots.current();
        assert!(snap
            .signals
            .iter()
            .any(|s| s.signal_type == HIGH_STRESS_SIGNAL_TYPE));
    }

    #[test]
    fn absorb_batch_drops_retracted_life_event_and_marker() {
        let walk = Observation::try_new(
            Uuid::from_u128(500),
            UnixTimestamp::from_secs(1_000),
            "com.biofocus.desktop",
            bio_spec::DATA_TYPE_LIFE_EVENT,
            json!({ "kind": "walk" }),
            1.0,
        )
        .expect("walk");
        let mut recent = vec![hrv(1, 990, 40.0)];
        absorb_batch(&mut recent, &[walk]);
        assert_eq!(recent.len(), 2);

        let marker = Observation::try_new(
            Uuid::from_u128(501),
            UnixTimestamp::from_secs(1_010),
            "com.biofocus.desktop",
            bio_spec::DATA_TYPE_LIFE_EVENT_RETRACTION,
            json!({ "target_id": Uuid::from_u128(500).to_string() }),
            1.0,
        )
        .expect("marker");
        absorb_batch(&mut recent, &[marker]);
        assert_eq!(recent.len(), 1, "walk + marker gone, HRV kept");
        assert_eq!(recent[0].id, Uuid::from_u128(1));
    }

    #[test]
    fn seeded_resting_heart_rate_stays_in_the_snapshot() {
        let state = AlertState::new();
        let snapshots = SnapshotState::new();
        let mut hook = CatalogAlertHook::new(state.share(), snapshots.share());
        let sample = Observation::try_new(
            Uuid::from_u128(7),
            UnixTimestamp::from_secs(1_700_000_000),
            "com.biofocus.applehealth",
            bio_spec::DATA_TYPE_RESTING_HEART_RATE,
            json!({ "bpm": 58.0, "src": { "kind": "apple_watch" } }),
            1.0,
        )
        .expect("rhr");
        hook.seed_vitals(vec![sample]);
        assert!(snapshots
            .current()
            .features
            .iter()
            .any(|f| f.feature_id == "RestingHeartRate"));

        hook.on_source_deletions(&[Uuid::from_u128(7)]);
        hook.on_normalized(&pipeline::NormalizedBatch::from_selected(Vec::new(), 0));
        assert!(snapshots
            .current()
            .features
            .iter()
            .all(|f| f.feature_id != "RestingHeartRate"));
    }
}
