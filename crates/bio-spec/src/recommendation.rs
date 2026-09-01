//! Deterministic suggested actions with Evidence (`Recommendation`, L4 / ADR-009).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::EvidenceRef;

/// Recommendation primary key.
pub type RecommendationId = Uuid;

/// Calm, optional suggested action with provenance Evidence.
///
/// Not clinical advice. Computed deterministically in Core (not by an LLM).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recommendation {
    pub id: RecommendationId,
    pub title: String,
    pub suggestion: String,
    pub category: String,
    pub evidence_list: Vec<EvidenceRef>,
}
