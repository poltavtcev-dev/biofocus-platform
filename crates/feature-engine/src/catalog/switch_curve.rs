//! Shared context-switch → 0–100 curve (tester-ready recalibration).
//!
//! v1 used `csr × 50` clamped, which saturated at **2 switches/min**
//! (30 app changes per 15 min — ordinary for developer work), pinning
//! FocusScore stability at 0 and CognitiveLoad switches at 100.
//!
//! v2 uses a smooth saturating curve:
//!
//! `switch_load(csr) = 100 × (1 − e^(−csr / k))`, `k = 1.5` switches/min.
//!
//! | switches/min | per 15 min | load |
//! |---|---|---|
//! | 0.25 | ~4  | 15 |
//! | 0.5  | ~8  | 28 |
//! | 1    | 15  | 49 |
//! | 2    | 30  | 74 |
//! | 4    | 60  | 93 |
//!
//! It approaches but never quite reaches 100, so very different switching
//! levels stay distinguishable. Stability is `100 − switch_load`.

/// Switch rate (per min) at which load reaches ~63 %.
pub(crate) const SWITCH_CURVE_K: f64 = 1.5;

/// Maps ContextSwitchRate (switches/min) onto a 0–100 "switching load".
#[must_use]
pub(crate) fn switch_load(csr_per_min: f64) -> f64 {
    if !csr_per_min.is_finite() || csr_per_min <= 0.0 {
        return 0.0;
    }
    (100.0 * (1.0 - (-csr_per_min / SWITCH_CURVE_K).exp())).clamp(0.0, 100.0)
}

/// Inverse view used by focus / stability scores.
#[must_use]
pub(crate) fn switch_stability(csr_per_min: f64) -> f64 {
    100.0 - switch_load(csr_per_min)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_and_invalid_rates_are_calm() {
        assert_eq!(switch_load(0.0), 0.0);
        assert_eq!(switch_load(-1.0), 0.0);
        assert_eq!(switch_load(f64::NAN), 0.0);
        assert_eq!(switch_stability(0.0), 100.0);
    }

    #[test]
    fn two_switches_per_min_no_longer_saturates() {
        // v1: 2.0 × 50 = 100 (pinned). v2 must leave headroom.
        let load = switch_load(2.0);
        assert!(load > 65.0 && load < 80.0, "load={load}");
        assert!(switch_stability(2.0) > 20.0);
    }

    #[test]
    fn curve_is_monotonic_and_bounded() {
        let mut prev = -1.0;
        for i in 0..=400 {
            let csr = f64::from(i) * 0.05; // 0 .. 20 switches/min
            let v = switch_load(csr);
            assert!((0.0..=100.0).contains(&v));
            assert!(v >= prev, "not monotonic at {csr}");
            prev = v;
        }
    }

    #[test]
    fn extreme_rates_stay_distinguishable_until_very_high() {
        assert!(switch_load(4.0) - switch_load(2.0) > 10.0);
        assert!(switch_load(1000.0) <= 100.0);
    }
}
