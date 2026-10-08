//! HRV samples stay in one method. RMSSD and SDNN are never averaged together.
//!
//! Absolute maps in StressIndex, RecoveryScore, and Focus read RMSSD only.
//! SDNN is compared with the person's own SDNN days in `HrvVsBaseline`.

use bio_spec::Observation;

/// Which HRV number a sample carries. The two are not interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum HrvMethod {
    Rmssd,
    Sdnn,
}

impl HrvMethod {
    #[must_use]
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Rmssd => "rmssd",
            Self::Sdnn => "sdnn",
        }
    }

    #[must_use]
    pub(crate) fn field(self) -> &'static str {
        match self {
            Self::Rmssd => "rmssd_ms",
            Self::Sdnn => "sdnn_ms",
        }
    }
}

/// Method of one sample.
///
/// An explicit `method` wins when its field is present. Otherwise a sample with
/// only one of the two fields takes that method. Both fields and no method, or
/// a method whose field is missing, is skipped so the numbers cannot be mixed.
#[must_use]
pub(crate) fn sample_method(obs: &Observation) -> Option<HrvMethod> {
    let has_rmssd = finite_field(obs, "rmssd_ms");
    let has_sdnn = finite_field(obs, "sdnn_ms");
    match obs.payload.get("method").and_then(|v| v.as_str()) {
        Some("rmssd") if has_rmssd => Some(HrvMethod::Rmssd),
        Some("sdnn") if has_sdnn => Some(HrvMethod::Sdnn),
        Some("rmssd" | "sdnn") => None,
        _ => match (has_rmssd, has_sdnn) {
            (true, false) => Some(HrvMethod::Rmssd),
            (false, true) => Some(HrvMethod::Sdnn),
            _ => None,
        },
    }
}

/// Mean of one method. Samples of the other method are ignored.
pub(crate) fn mean_method(obs: &[&Observation], method: HrvMethod) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if sample_method(o) != Some(method) {
            continue;
        }
        if let Some(v) = o
            .payload
            .get(method.field())
            .and_then(|v| v.as_f64())
            .filter(|r| r.is_finite() && *r >= 0.0)
        {
            sum += v;
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f64)
    }
}

/// Mean RMSSD (ms). SDNN does not fill this slot.
pub(crate) fn mean_hrv_ms(obs: &[&Observation]) -> Option<f64> {
    mean_method(obs, HrvMethod::Rmssd)
}

fn finite_field(obs: &Observation, key: &str) -> bool {
    obs.payload
        .get(key)
        .and_then(|v| v.as_f64())
        .is_some_and(|r| r.is_finite() && r >= 0.0)
}

#[cfg(test)]
mod tests {
    use bio_spec::{Observation, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn obs(id: u128, payload: serde_json::Value) -> Observation {
        Observation::try_new(
            Uuid::from_u128(id),
            UnixTimestamp::from_secs(1_000),
            "test.provider",
            "hrv",
            payload,
            1.0,
        )
        .expect("obs")
    }

    #[test]
    fn sdnn_and_rmssd_are_not_averaged() {
        let rmssd = obs(1, json!({ "method": "rmssd", "rmssd_ms": 80.0 }));
        let sdnn = obs(2, json!({ "method": "sdnn", "sdnn_ms": 20.0 }));
        let both = obs(3, json!({ "rmssd_ms": 10.0, "sdnn_ms": 90.0 }));
        let samples = [&rmssd, &sdnn, &both];
        assert_eq!(mean_hrv_ms(&samples), Some(80.0));
        assert_eq!(mean_method(&samples, HrvMethod::Sdnn), Some(20.0));
        assert!(sample_method(&both).is_none());
    }
}
