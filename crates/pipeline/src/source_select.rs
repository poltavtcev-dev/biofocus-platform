//! Read-side source choice (ADR-030). SQLite keeps every row.
//!
//! Before features, each time bucket keeps one `src.kind`. Default order:
//! Apple Watch, then Mi Fitness, Zepp Life, iPhone, other apps, manual entry.
//! Cumulative types are not summed. Sleep stages stay with the source that wrote them.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bio_spec::{
    DATA_TYPE_ACTIVE_ENERGY, DATA_TYPE_BASAL_ENERGY, DATA_TYPE_DISTANCE_WALKING_RUNNING,
    DATA_TYPE_EXERCISE_TIME, DATA_TYPE_HEART_RATE, DATA_TYPE_HRV, DATA_TYPE_OXYGEN_SATURATION,
    DATA_TYPE_RESPIRATORY_RATE, DATA_TYPE_RESTING_HEART_RATE, DATA_TYPE_SLEEP_INTERVAL,
    DATA_TYPE_SLEEPING_WRIST_TEMPERATURE, DATA_TYPE_STAND_TIME, DATA_TYPE_STEP_COUNT,
    DATA_TYPE_VO2_MAX, DATA_TYPE_WALKING_HEART_RATE_AVERAGE, Observation, SRC_KIND_APPLE_WATCH,
    SRC_KIND_IPHONE, SRC_KIND_MANUAL, SRC_KIND_OTHER_APP, SRC_KIND_XIAOMI_MI_FITNESS,
    SRC_KIND_ZEPP_LIFE, apply_source_deletions,
};

const SECS_PER_DAY: i64 = 86_400;
const DISCRETE_BUCKET_SECS: i64 = 900;
const SLEEP_NIGHT_SHIFT_SECS: i64 = 6 * 3_600;

/// Higher entries win. Unknown kinds sort after this list and tie with each other.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePriority {
    order: Vec<String>,
}

impl Default for SourcePriority {
    fn default() -> Self {
        Self {
            order: vec![
                SRC_KIND_APPLE_WATCH.to_owned(),
                SRC_KIND_XIAOMI_MI_FITNESS.to_owned(),
                SRC_KIND_ZEPP_LIFE.to_owned(),
                SRC_KIND_IPHONE.to_owned(),
                SRC_KIND_OTHER_APP.to_owned(),
                SRC_KIND_MANUAL.to_owned(),
            ],
        }
    }
}

impl SourcePriority {
    /// First name is the winner. Unknown names are ignored. Omitted defaults are appended.
    #[must_use]
    pub fn from_order(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut order = Vec::new();
        for name in names {
            let name = name.into();
            if bio_spec::is_src_kind(&name) && !order.iter().any(|existing| existing == &name) {
                order.push(name);
            }
        }
        for fallback in Self::default().order {
            if !order.iter().any(|existing| existing == &fallback) {
                order.push(fallback);
            }
        }
        Self { order }
    }

    /// `order = ["apple_watch", ...]` in a TOML file. Invalid or missing → [`Default`].
    #[must_use]
    pub fn from_toml_str(text: &str) -> Self {
        let Ok(value) = text.parse::<toml::Value>() else {
            return Self::default();
        };
        let Some(items) = value.get("order").and_then(toml::Value::as_array) else {
            return Self::default();
        };
        let names = items.iter().filter_map(toml::Value::as_str);
        Self::from_order(names)
    }

    /// `BIOFOCUS_HOME/source-priority.toml`, else `~/.biofocus/source-priority.toml`.
    #[must_use]
    pub fn load_installed() -> Self {
        let Some(path) = default_priority_path() else {
            return Self::default();
        };
        Self::load_path(&path)
    }

    #[must_use]
    pub fn load_path(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };
        Self::from_toml_str(&text)
    }

    /// Highest priority first. Always the full closed set.
    #[must_use]
    pub fn order(&self) -> &[String] {
        &self.order
    }

    /// `order = ["apple_watch", ...]` for `source-priority.toml`.
    #[must_use]
    pub fn to_toml(&self) -> String {
        let mut out = String::from("order = [");
        for (index, name) in self.order.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            out.push('"');
            out.push_str(name);
            out.push('"');
        }
        out.push_str("]\n");
        out
    }

    /// Writes [`Self::to_toml`] and creates the parent directory when needed.
    pub fn write_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, self.to_toml())
    }

    /// Path the desktop host reads and writes. `None` when home is unset.
    #[must_use]
    pub fn installed_path() -> Option<PathBuf> {
        default_priority_path()
    }

    fn rank(&self, kind: Option<&str>) -> usize {
        let Some(kind) = kind else {
            return self.order.len();
        };
        self.order
            .iter()
            .position(|name| name == kind)
            .unwrap_or(self.order.len())
    }
}

/// Drop lower-priority sources that share a bucket with a better one.
///
/// Rows without a wearable `data_type` are kept. `source_deletion` markers hide
/// their target when the provider matches, and the markers themselves are removed.
#[must_use]
pub fn select_sources(
    observations: Vec<Observation>,
    priority: &SourcePriority,
) -> Vec<Observation> {
    let observations = apply_source_deletions(observations);
    let mut groups: HashMap<(String, i64), Vec<usize>> = HashMap::new();
    let mut keep = vec![false; observations.len()];

    for (index, obs) in observations.iter().enumerate() {
        match bucket(obs) {
            Some(key) => groups
                .entry((obs.data_type.clone(), key))
                .or_default()
                .push(index),
            None => keep[index] = true,
        }
    }

    for indexes in groups.values() {
        let Some(best) = indexes
            .iter()
            .map(|&index| priority.rank(src_kind(&observations[index])))
            .min()
        else {
            continue;
        };
        for &index in indexes {
            if priority.rank(src_kind(&observations[index])) == best {
                keep[index] = true;
            }
        }
    }

    observations
        .into_iter()
        .enumerate()
        .filter(|(index, _)| keep[*index])
        .map(|(_, obs)| obs)
        .collect()
}

fn default_priority_path() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("BIOFOCUS_HOME") {
        if !home.is_empty() {
            return Some(PathBuf::from(home).join("source-priority.toml"));
        }
    }
    let home = std::env::var_os("HOME")?;
    Some(
        PathBuf::from(home)
            .join(".biofocus")
            .join("source-priority.toml"),
    )
}

fn src_kind(obs: &Observation) -> Option<&str> {
    obs.payload.get("src")?.get("kind")?.as_str()
}

fn bucket(obs: &Observation) -> Option<i64> {
    let ts = obs.timestamp.as_secs();
    match obs.data_type.as_str() {
        DATA_TYPE_STEP_COUNT
        | DATA_TYPE_ACTIVE_ENERGY
        | DATA_TYPE_BASAL_ENERGY
        | DATA_TYPE_DISTANCE_WALKING_RUNNING
        | DATA_TYPE_EXERCISE_TIME
        | DATA_TYPE_STAND_TIME => Some(ts.div_euclid(SECS_PER_DAY)),
        DATA_TYPE_HEART_RATE
        | DATA_TYPE_RESTING_HEART_RATE
        | DATA_TYPE_WALKING_HEART_RATE_AVERAGE
        | DATA_TYPE_HRV
        | DATA_TYPE_OXYGEN_SATURATION
        | DATA_TYPE_RESPIRATORY_RATE
        | DATA_TYPE_SLEEPING_WRIST_TEMPERATURE
        | DATA_TYPE_VO2_MAX => Some(ts.div_euclid(DISCRETE_BUCKET_SECS)),
        DATA_TYPE_SLEEP_INTERVAL => {
            let start = obs
                .payload
                .get("start")
                .and_then(serde_json::Value::as_i64)
                .unwrap_or(ts);
            Some(
                start
                    .saturating_sub(SLEEP_NIGHT_SHIFT_SECS)
                    .div_euclid(SECS_PER_DAY),
            )
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn obs(n: u128, ts: i64, data_type: &str, payload: serde_json::Value) -> Observation {
        Observation::try_new(
            Uuid::from_u128(n),
            UnixTimestamp::from_secs(ts),
            "com.biofocus.applehealth",
            data_type,
            payload,
            1.0,
        )
        .expect("observation")
    }

    fn src(kind: &str) -> serde_json::Value {
        json!({ "kind": kind })
    }

    #[test]
    fn steps_in_one_day_are_not_summed() {
        let day = 1_700_000_000;
        let watch = obs(
            1,
            day,
            DATA_TYPE_STEP_COUNT,
            json!({ "count": 1000, "src": src(SRC_KIND_APPLE_WATCH) }),
        );
        let band = obs(
            2,
            day + 30,
            DATA_TYPE_STEP_COUNT,
            json!({ "count": 800, "src": src(SRC_KIND_XIAOMI_MI_FITNESS) }),
        );
        let kept = select_sources(vec![watch, band], &SourcePriority::default());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].payload["count"], 1000);
    }

    #[test]
    fn heart_rate_prefers_apple_watch() {
        let ts = 1_700_000_100;
        let watch = obs(
            3,
            ts,
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": 62, "src": src(SRC_KIND_APPLE_WATCH) }),
        );
        let band = obs(
            4,
            ts + 10,
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": 71, "src": src(SRC_KIND_XIAOMI_MI_FITNESS) }),
        );
        let kept = select_sources(vec![band, watch], &SourcePriority::default());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].payload["bpm"], 62);
    }

    #[test]
    fn priority_change_keeps_the_other_source() {
        let ts = 1_700_000_100;
        let watch = obs(
            5,
            ts,
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": 62, "src": src(SRC_KIND_APPLE_WATCH) }),
        );
        let band = obs(
            6,
            ts + 10,
            DATA_TYPE_HEART_RATE,
            json!({ "bpm": 71, "src": src(SRC_KIND_XIAOMI_MI_FITNESS) }),
        );
        let priority =
            SourcePriority::from_order([SRC_KIND_XIAOMI_MI_FITNESS, SRC_KIND_APPLE_WATCH]);
        let kept = select_sources(vec![watch, band], &priority);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].payload["bpm"], 71);
    }

    #[test]
    fn sleep_stages_stay_with_the_winning_source() {
        let start = 1_700_003_600;
        let watch = obs(
            7,
            start,
            DATA_TYPE_SLEEP_INTERVAL,
            json!({
                "start": start,
                "end": start + 3600,
                "stage": "asleep_deep",
                "src": src(SRC_KIND_APPLE_WATCH)
            }),
        );
        let band = obs(
            8,
            start + 60,
            DATA_TYPE_SLEEP_INTERVAL,
            json!({
                "start": start + 60,
                "end": start + 4000,
                "stage": "asleep",
                "src": src(SRC_KIND_XIAOMI_MI_FITNESS)
            }),
        );
        let kept = select_sources(vec![watch, band], &SourcePriority::default());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].payload["stage"], "asleep_deep");
    }

    #[test]
    fn hrv_methods_are_not_mixed_across_sources() {
        let ts = 1_700_000_100;
        let watch = obs(
            9,
            ts,
            DATA_TYPE_HRV,
            json!({ "method": "sdnn", "sdnn_ms": 48, "src": src(SRC_KIND_APPLE_WATCH) }),
        );
        let other = obs(
            10,
            ts + 5,
            DATA_TYPE_HRV,
            json!({ "method": "rmssd", "rmssd_ms": 30, "src": src(SRC_KIND_OTHER_APP) }),
        );
        let kept = select_sources(vec![watch, other], &SourcePriority::default());
        assert_eq!(kept.len(), 1);
        assert!(kept[0].payload.get("rmssd_ms").is_none());
        assert_eq!(kept[0].payload["method"], "sdnn");
    }

    #[test]
    fn mac_rows_without_src_are_kept() {
        let mac = obs(
            11,
            1_700_000_000,
            "context_window",
            json!({ "bundle_id": "com.example.app", "app_name": "Example" }),
        );
        let kept = select_sources(vec![mac], &SourcePriority::default());
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].data_type, "context_window");
    }

    #[test]
    fn toml_order_overrides_the_default() {
        let priority = SourcePriority::from_toml_str("order = [\"zepp_life\", \"apple_watch\"]\n");
        assert_eq!(priority.rank(Some(SRC_KIND_ZEPP_LIFE)), 0);
        assert_eq!(priority.rank(Some(SRC_KIND_APPLE_WATCH)), 1);
    }
}
