//! Catalog Feature nodes (Phase 3 E2) — deterministic v1 formulas.
//!
//! | Node | Feature id | Task |
//! | :--- | :--- | :--- |
//! | [`ContextSwitchRateNode`] | `ContextSwitchRate` | P3-E2-T2 |
//! | [`FocusScoreNode`] | `FocusScore` | P3-E2-T2 |
//!
//! Register both via [`register_focus_v1`].

mod context_switch_rate;
mod focus_score;
mod window;

pub use context_switch_rate::{ContextSwitchRateNode, FEATURE_ID as CONTEXT_SWITCH_RATE_ID};
pub use focus_score::{FocusScoreNode, FEATURE_ID as FOCUS_SCORE_ID};
pub use window::{STEP_SECS, WINDOW_SECS};

use crate::{FeatureEngine, FeatureEngineResult};

/// Registers `ContextSwitchRate` then `FocusScore` (DAG dependency order).
pub fn register_focus_v1(engine: &mut FeatureEngine) -> FeatureEngineResult<()> {
    engine.register(ContextSwitchRateNode::new())?;
    engine.register(FocusScoreNode::new())?;
    Ok(())
}
