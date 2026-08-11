//! Shared HRV payload helpers (ADR-016: prefer RMSSD, else SDNN proxy).

use bio_spec::Observation;

/// Mean of a finite numeric payload field across Observations.
pub(crate) fn mean_f64_field(obs: &[&Observation], key: &str) -> Option<f64> {
    let mut sum = 0.0;
    let mut n = 0usize;
    for o in obs {
        if let Some(v) = o
            .payload
            .get(key)
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

/// Prefer mean `rmssd_ms`, else mean `sdnn_ms` (Apple HealthKit HRV-proxy).
pub(crate) fn mean_hrv_ms(obs: &[&Observation]) -> Option<f64> {
    mean_f64_field(obs, "rmssd_ms").or_else(|| mean_f64_field(obs, "sdnn_ms"))
}
