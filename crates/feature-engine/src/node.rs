//! DAG node contract and per-run compute context.

use bio_spec::{Feature, Observation, Signal};

use crate::FeatureEngineResult;

/// Stable id of a Feature Engine DAG node (e.g. `FocusScore`, `stub_a`).
pub type NodeId = String;

/// Read-only inputs available to a node during a single DAG run.
///
/// Observations are an in-memory snapshot (typically post-normalize).
/// Features / Signals already produced by upstream nodes appear in topo order.
#[derive(Debug)]
pub struct ComputeContext<'a> {
    observations: &'a [Observation],
    features: &'a [Feature],
    signals: &'a [Signal],
}

impl<'a> ComputeContext<'a> {
    pub(crate) fn new(
        observations: &'a [Observation],
        features: &'a [Feature],
        signals: &'a [Signal],
    ) -> Self {
        Self {
            observations,
            features,
            signals,
        }
    }

    /// Normalized Observation snapshot for this run.
    #[must_use]
    pub fn observations(&self) -> &'a [Observation] {
        self.observations
    }

    /// Features produced by nodes that already ran (topo predecessors).
    #[must_use]
    pub fn features(&self) -> &'a [Feature] {
        self.features
    }

    /// Signals produced by nodes that already ran.
    #[must_use]
    pub fn signals(&self) -> &'a [Signal] {
        self.signals
    }

    /// First Feature with matching `feature_id`, if any.
    #[must_use]
    pub fn feature_by_id(&self, feature_id: &str) -> Option<&'a Feature> {
        self.features
            .iter()
            .find(|f| f.feature_id == feature_id)
    }
}

/// Outputs contributed by one node during a run.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct NodeOutput {
    /// Derived Features (may be empty).
    pub features: Vec<Feature>,
    /// Optional transient Signals (may be empty).
    pub signals: Vec<Signal>,
}

impl NodeOutput {
    /// No Features and no Signals.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Features only.
    #[must_use]
    pub fn features(features: Vec<Feature>) -> Self {
        Self {
            features,
            signals: Vec::new(),
        }
    }

    /// Signals only.
    #[must_use]
    pub fn signals(signals: Vec<Signal>) -> Self {
        Self {
            features: Vec::new(),
            signals,
        }
    }
}

/// One node in the Feature calculation DAG.
///
/// Implementors compute Features / Signals from the Observation snapshot and
/// upstream outputs. Catalog v1 nodes live in [`crate::catalog`].
pub trait FeatureNode: Send {
    /// Unique node id within one [`crate::FeatureEngine`].
    fn id(&self) -> &str;

    /// Node ids that must finish before this node runs.
    fn depends_on(&self) -> &[NodeId];

    /// Deterministic compute step for this node.
    fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput>;
}
