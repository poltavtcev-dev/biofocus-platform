//! Feature Engine: register DAG nodes and run topological compute.

use std::collections::{HashMap, VecDeque};

use bio_spec::{Feature, Observation, Signal};

use crate::{
    ComputeContext, FeatureEngineError, FeatureEngineResult, FeatureNode, NodeId, NodeOutput,
};

/// Aggregated result of one DAG run over an Observation snapshot.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct EngineOutput {
    /// Features in topological execution order (nodes may emit zero or more).
    pub features: Vec<Feature>,
    /// Optional Signals in the same execution order.
    pub signals: Vec<Signal>,
}

impl EngineOutput {
    /// `true` when no Features and no Signals were produced.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.features.is_empty() && self.signals.is_empty()
    }
}

/// In-memory Feature DAG scheduler.
///
/// Register nodes, then [`Self::run`] on a normalized Observation snapshot.
/// Features / Signals stay derived in memory (no SQLite schema).
#[derive(Default)]
pub struct FeatureEngine {
    nodes: Vec<Box<dyn FeatureNode>>,
    index_by_id: HashMap<NodeId, usize>,
}

impl FeatureEngine {
    /// Empty engine with no nodes.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of registered nodes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// `true` when no nodes are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Register a DAG node. Fails if `id` is already taken.
    pub fn register(&mut self, node: impl FeatureNode + 'static) -> FeatureEngineResult<()> {
        let id = node.id().to_owned();
        if self.index_by_id.contains_key(&id) {
            return Err(FeatureEngineError::DuplicateNode { id });
        }
        let idx = self.nodes.len();
        self.index_by_id.insert(id, idx);
        self.nodes.push(Box::new(node));
        Ok(())
    }

    /// Topological run over an in-memory Observation snapshot.
    ///
    /// Empty snapshot and/or empty DAG return [`Ok`] with empty output
    /// (idle-friendly). Does not spin or allocate threads.
    pub fn run(&self, observations: &[Observation]) -> FeatureEngineResult<EngineOutput> {
        if self.nodes.is_empty() {
            return Ok(EngineOutput::default());
        }

        let order = self.topological_order()?;
        let mut features: Vec<Feature> = Vec::new();
        let mut signals: Vec<Signal> = Vec::new();

        for idx in order {
            let node = &self.nodes[idx];
            let ctx = ComputeContext::new(observations, &features, &signals);
            let NodeOutput {
                features: node_features,
                signals: node_signals,
            } = node.compute(&ctx).map_err(|err| match err {
                FeatureEngineError::NodeFailed { .. } => err,
                other => FeatureEngineError::NodeFailed {
                    node: node.id().to_owned(),
                    message: other.to_string(),
                },
            })?;
            features.extend(node_features);
            signals.extend(node_signals);
        }

        Ok(EngineOutput { features, signals })
    }

    /// Kahn topological sort; validates dependencies and detects cycles.
    fn topological_order(&self) -> FeatureEngineResult<Vec<usize>> {
        let n = self.nodes.len();
        let mut indegree = vec![0usize; n];
        let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];

        for (idx, node) in self.nodes.iter().enumerate() {
            for dep_id in node.depends_on() {
                let Some(&dep_idx) = self.index_by_id.get(dep_id.as_str()) else {
                    return Err(FeatureEngineError::UnknownDependency {
                        node: node.id().to_owned(),
                        missing: dep_id.clone(),
                    });
                };
                adjacency[dep_idx].push(idx);
                indegree[idx] += 1;
            }
        }

        let mut queue: VecDeque<usize> = indegree
            .iter()
            .enumerate()
            .filter_map(|(i, &d)| if d == 0 { Some(i) } else { None })
            .collect();

        // Stable among equal indegree: registration order via index ascending.
        queue.make_contiguous().sort_unstable();

        let mut order = Vec::with_capacity(n);
        while let Some(idx) = queue.pop_front() {
            order.push(idx);
            let mut unlocked = Vec::new();
            for &next in &adjacency[idx] {
                indegree[next] -= 1;
                if indegree[next] == 0 {
                    unlocked.push(next);
                }
            }
            unlocked.sort_unstable();
            queue.extend(unlocked);
        }

        if order.len() != n {
            let leftover = indegree
                .iter()
                .enumerate()
                .find_map(|(i, &d)| {
                    if d > 0 {
                        Some(self.nodes[i].id().to_owned())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "<unknown>".to_owned());
            return Err(FeatureEngineError::CycleDetected { node: leftover });
        }

        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bio_spec::{Confidence, Feature, FeatureValue, Observation, TimeWindow, UnixTimestamp};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::NodeOutput;

    struct RecordingNode {
        id: NodeId,
        deps: Vec<NodeId>,
        feature_id: String,
        order: Arc<Mutex<Vec<String>>>,
        /// When set, requires this upstream feature_id to already exist.
        require_upstream_feature: Option<String>,
    }

    impl FeatureNode for RecordingNode {
        fn id(&self) -> &str {
            &self.id
        }

        fn depends_on(&self) -> &[NodeId] {
            &self.deps
        }

        fn compute(&self, ctx: &ComputeContext<'_>) -> FeatureEngineResult<NodeOutput> {
            if let Some(ref need) = self.require_upstream_feature {
                if ctx.feature_by_id(need).is_none() {
                    return Err(FeatureEngineError::NodeFailed {
                        node: self.id.clone(),
                        message: format!("missing upstream feature `{need}`"),
                    });
                }
            }
            if let Ok(mut guard) = self.order.lock() {
                guard.push(self.id.clone());
            }
            let window =
                TimeWindow::try_new(UnixTimestamp::from_secs(0), UnixTimestamp::from_secs(60))
                    .map_err(|e| FeatureEngineError::NodeFailed {
                        node: self.id.clone(),
                        message: e.to_string(),
                    })?;
            Ok(NodeOutput::features(vec![Feature {
                feature_id: self.feature_id.clone(),
                time_window: window,
                value: FeatureValue::Scalar(1.0),
                provenance: ctx.observations().iter().map(|o| o.id).collect(),
            confidence: Confidence::ONE,
            }]))
        }
    }

    fn sample_observation() -> Observation {
        Observation::try_new(
            Uuid::from_u128(1),
            UnixTimestamp::from_secs(1),
            "test.provider",
            "heart_rate",
            json!({ "bpm": 60 }),
            1.0,
        )
        .expect("valid observation")
    }

    #[test]
    fn two_node_dag_runs_in_dependency_order() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut engine = FeatureEngine::new();

        engine
            .register(RecordingNode {
                id: "node_a".into(),
                deps: vec![],
                feature_id: "FeatA".into(),
                order: Arc::clone(&order),
                require_upstream_feature: None,
            })
            .expect("register a");
        engine
            .register(RecordingNode {
                id: "node_b".into(),
                deps: vec!["node_a".into()],
                feature_id: "FeatB".into(),
                order: Arc::clone(&order),
                require_upstream_feature: Some("FeatA".into()),
            })
            .expect("register b");

        let out = engine.run(&[sample_observation()]).expect("run ok");

        let recorded = order.lock().expect("order lock");
        assert_eq!(recorded.as_slice(), ["node_a", "node_b"]);
        assert_eq!(out.features.len(), 2);
        assert_eq!(out.features[0].feature_id, "FeatA");
        assert_eq!(out.features[1].feature_id, "FeatB");
        assert!(out.signals.is_empty());
        assert_eq!(out.features[0].provenance.len(), 1);
    }

    #[test]
    fn empty_engine_and_empty_snapshot_are_ok() {
        let engine = FeatureEngine::new();
        let out = engine.run(&[]).expect("empty ok");
        assert!(out.is_empty());
    }

    #[test]
    fn duplicate_node_id_rejected() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut engine = FeatureEngine::new();
        engine
            .register(RecordingNode {
                id: "same".into(),
                deps: vec![],
                feature_id: "X".into(),
                order: Arc::clone(&order),
                require_upstream_feature: None,
            })
            .expect("first");
        let err = engine
            .register(RecordingNode {
                id: "same".into(),
                deps: vec![],
                feature_id: "Y".into(),
                order,
                require_upstream_feature: None,
            })
            .expect_err("duplicate");
        assert!(matches!(err, FeatureEngineError::DuplicateNode { .. }));
    }

    #[test]
    fn unknown_dependency_rejected() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut engine = FeatureEngine::new();
        engine
            .register(RecordingNode {
                id: "leaf".into(),
                deps: vec!["missing".into()],
                feature_id: "X".into(),
                order,
                require_upstream_feature: None,
            })
            .expect("register");
        let err = engine.run(&[]).expect_err("unknown dep");
        assert!(matches!(
            err,
            FeatureEngineError::UnknownDependency { .. }
        ));
    }

    #[test]
    fn cycle_detected() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let mut engine = FeatureEngine::new();
        engine
            .register(RecordingNode {
                id: "a".into(),
                deps: vec!["b".into()],
                feature_id: "A".into(),
                order: Arc::clone(&order),
                require_upstream_feature: None,
            })
            .expect("a");
        engine
            .register(RecordingNode {
                id: "b".into(),
                deps: vec!["a".into()],
                feature_id: "B".into(),
                order,
                require_upstream_feature: None,
            })
            .expect("b");
        let err = engine.run(&[]).expect_err("cycle");
        assert!(matches!(err, FeatureEngineError::CycleDetected { .. }));
    }
}