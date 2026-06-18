//! Delivery slice — a saved pointer to a section of the event model (§7.1).
//!
//! A delivery feature is a subgraph of the What, "not a free-floating ticket":
//! this artifact names one or more anchor nodes (typically a flow) and nothing
//! else, so the concrete LLM build-context is *assembled from the model* via the
//! bundle closure rather than restated in the feature.

use serde::{Deserialize, Serialize};

use crate::error::{ProductError, Result};

use super::bundle::bundle_many;
use super::model::DomainGraph;
use super::validate::Violation;

/// A pointer to a slice of the event model. The closure (the concrete context)
/// is derived from `anchors`, never restated here.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Slice {
    pub id: String,
    /// Anchor node ids (a flow, a bounded context, an aggregate, …).
    pub anchors: Vec<String>,
    /// Traversal depth for the assembled context (default 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<usize>,
}

impl Slice {
    pub fn from_yaml(text: &str) -> Result<Self> {
        serde_yaml::from_str(text)
            .map_err(|e| ProductError::ConfigError(format!("invalid slice YAML: {}", e)))
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self)
            .map_err(|e| ProductError::Internal(format!("serialize slice: {}", e)))
    }

    /// The traversal depth, defaulting to 2 hops.
    pub fn depth(&self) -> usize {
        self.depth.unwrap_or(2)
    }
}

/// Validate a slice: it must name at least one anchor, and every anchor must
/// resolve to a real node in the captured What graph.
pub fn validate_slice(slice: &Slice, graph: &DomainGraph) -> Vec<Violation> {
    let mut out = Vec::new();
    if slice.anchors.is_empty() {
        out.push(v(&slice.id, "anchors",
            "§7.1 A delivery slice must point at a section of the event model (at least one anchor)."));
    }
    for a in &slice.anchors {
        if !graph.contains(a) {
            out.push(v(&slice.id, "anchors",
                &format!("§7.1 anchor '{a}' is not a node in the captured What graph.")));
        }
    }
    out
}

/// Assemble the concrete LLM build-context for a slice — the What subgraph
/// reachable from its anchors, as a markdown bundle. `None` if no anchor
/// resolves.
pub fn slice_context(slice: &Slice, graph: &DomainGraph, depth: usize, product: &str) -> Option<String> {
    bundle_many(graph, &slice.anchors, depth, product)
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
#[path = "slice_tests.rs"]
mod tests;
