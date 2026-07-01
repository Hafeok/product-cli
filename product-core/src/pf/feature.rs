//! Delivery feature — a saved pointer to a subgraph of the event model (§7.1).
//!
//! A feature is a reference to a subgraph of one or more flows, "not a
//! free-floating ticket": this artifact names one or more anchor nodes
//! (typically a flow) and nothing else, so the concrete LLM build-context is
//! *assembled from the model* via the bundle closure rather than restated in
//! the feature. (Containment, §7.1: a feature is a subgraph of flows, a flow is
//! a chain of slices/work-units — the feature is the subgraph, never the atom.)

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::bundle::bundle_many;
use super::model::DomainGraph;
use super::validate::Violation;

/// A pointer to a subgraph of the event model. The closure (the concrete
/// context) is derived from `anchors`, never restated here.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    /// Anchor node ids (a flow, a bounded context, an aggregate, …).
    pub anchors: Vec<String>,
    /// Traversal depth for the assembled context (default 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
}

impl Feature {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid feature YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize feature: {}", e)))
    }

    /// The traversal depth, defaulting to 2 hops.
    pub fn depth(&self) -> usize {
        self.depth.unwrap_or(2)
    }
}

/// Validate a feature: it must name at least one anchor, and every anchor must
/// resolve to a real node in the captured What graph.
pub fn validate_feature(feature: &Feature, graph: &DomainGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    if feature.anchors.is_empty() {
        out.push(v(&feature.id, "anchors",
            "§7.1 A delivery feature must point at a subgraph of the event model (at least one anchor)."));
    }
    for a in &feature.anchors {
        if !graph.contains(a) {
            out.push(v(&feature.id, "anchors",
                &format!("§7.1 anchor '{a}' is not a node in the captured What graph.")));
        }
    }
    out
}

/// Assemble the concrete LLM build-context for a feature — the What subgraph
/// reachable from its anchors, as a markdown bundle. `None` if no anchor
/// resolves.
pub fn feature_context(feature: &Feature, graph: &DomainGraph, depth: usize, product: &str) -> Option<String> {
    bundle_many(graph, &feature.anchors, depth, product)
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    Violation {
        focus: focus.to_string(),
        path: path.to_string(),
        message: message.to_string(),
        severity: "violation".to_string(),
    }
}

#[cfg(test)]
#[path = "feature_tests.rs"]
mod tests;
