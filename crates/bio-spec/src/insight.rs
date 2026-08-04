//! Analytical conclusions with evidence (`Insight`).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{FeatureId, SignalId};

/// Insight primary key.
pub type InsightId = Uuid;

/// Reference to supporting Feature or Signal evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum EvidenceRef {
    Feature(FeatureId),
    Signal(SignalId),
}

/// Analytical conclusion with provenance evidence and optional action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Insight {
    pub id: InsightId,
    pub title: String,
    pub description: String,
    pub category: String,
    pub evidence_list: Vec<EvidenceRef>,
    pub action_recommendation: Option<String>,
}
