//! The §5.1 Build seam — the work-unit emission contract.
//!
//! The boundary where specification ends and execution begins. The two message
//! shapes are owned by the **contracts tier**
//! ([ai-development-contracts](https://github.com/Hafeok/ai-development-contracts)
//! v0.1.0), which both pillars depend on; this framework authors that shape
//! (producer-owns) and emits it on the wire. The **canonical encoding is
//! kebab-case JSON** — these Rust structs keep snake_case field names but
//! serialize/deserialize as the contract's kebab-case shape, so a WorkUnit this
//! module emits validates against `work-unit.schema.json` and a VerdictEvent it
//! reads validates against `verdict-event.schema.json`.
//!
//! A WorkUnit is a complete, executable SPMC package: a fully-pinned model
//! binding (M, unit-level), a content-addressed context pool (C), and a sealed
//! cell-graph whose cells each carry their own shape (S) and prompt (P) inline.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ProductError, Result};

// The conversion entry points live in `build_seam_convert` (to keep files within
// the length budget) but are part of this module's public surface.
pub use super::build_seam_convert::{to_seam_envelope, SeamParams};

/// Tells a consumer whether an `accepted` verdict may auto-commit or must be
/// surfaced. Passed through unread by the executor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcceptanceClass {
    AutoCommitIfGreen,
    NeedsVerdict,
}

/// The §6.2 verdict vocabulary, pinned: a declared outcome, not an exit code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    Accepted,
    Rejected,
    Escalate,
}

/// The Transition-Contract consequence a consumer acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NextConsequence {
    Advance,
    Halt,
    Retry,
    Escalate,
}

/// A per-cell verdict inside a VerdictEvent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellVerdict {
    Accepted,
    Rejected,
    Skipped,
}

/// The fully-pinned Model binding (SPMC M, RFC 0002 axis precision): a served
/// binding, not a bare name. Pinned by the producer before emit, so the verdict
/// is attributable to an immutable input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelBinding {
    pub provider: String,
    pub model_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<String>,
    pub quantization: String,
    /// Invocation parameters (temperature, top-p, …) — at least one.
    pub invocation: Value,
}

/// The unit-level Model section: an optional capability tag over the binding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModelSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_tag: Option<String>,
    pub binding: ModelBinding,
}

/// One inline, content-addressed context fragment (SPMC C). Content travels by
/// value; `id` is the handle a cell selects it by.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContextFragment {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    pub content: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Value>,
}

/// The unit-level content pool (SPMC C) — every cell `context-refs` id resolves here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ContextPool {
    /// Named canonical serialization the `bundle-hash` is computed over.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundle_form_profile: Option<String>,
    pub fragments: Vec<ContextFragment>,
}

/// The frozen SPMC bundle (§5): unit-level M (one binding) and C (the pool).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SpmcBundle {
    pub model: ModelSection,
    pub context_pool: ContextPool,
}

/// A cell's Schema axis (S): shape language pinned + the shape document inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CellSchema {
    pub shape_language: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_version: Option<String>,
    pub document: Value,
}

/// A cell's Prompt axis (P): the instruction content inline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CellPrompt {
    pub content: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_version: Option<String>,
}

/// The artifact a cell produces — the executor's "where to do it".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CellOutput {
    pub artifact_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// One cell — one discrete model call in the sealed cell-graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SeamCell {
    pub id: String,
    #[serde(default)]
    pub requires: Vec<String>,
    pub schema: CellSchema,
    pub prompt: CellPrompt,
    #[serde(default)]
    pub context_refs: Vec<String>,
    pub output: CellOutput,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<Value>,
}

/// The sealed interior DAG — a ≥1-cell graph with no cross-unit edges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellGraph {
    pub cells: Vec<SeamCell>,
}

/// The declared transport for produced artifacts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum ArtifactDelivery {
    Inline,
    Workspace {
        kind: String,
        location: String,
        #[serde(default, skip_serializing_if = "Option::is_none", rename = "ref")]
        reference: Option<String>,
    },
}

/// The emitted work unit: the complete, executable SPMC package. Travels by
/// value; its `bundle_hash` (on the wire, `bundle-hash`) *is* its identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct SeamWorkUnit {
    pub unit_ref: String,
    pub parent_deliverable: String,
    pub bundle_hash: String,
    pub tier: String,
    pub acceptance_class: AcceptanceClass,
    pub ladder_position: u32,
    pub artifact_delivery: ArtifactDelivery,
    pub spmc_bundle: SpmcBundle,
    pub cell_graph: CellGraph,
}

/// One cell's result inside a VerdictEvent. Open by design (a consumer may
/// ignore executor-specific extras such as `passed`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CellResult {
    pub cell_id: String,
    pub verdict: CellVerdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Value>,
}

/// The self-describing verdict event: enough on its own for a consumer that
/// never saw the emission to reconcile it. The top-level envelope is closed;
/// per-cell results are open.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct VerdictEvent {
    pub event_id: String,
    pub emitted_at: String,
    pub unit_ref: String,
    pub parent_deliverable: String,
    pub bundle_hash: String,
    pub verdict: Verdict,
    pub tier_ran: String,
    pub cell_results: Vec<CellResult>,
    pub next_consequence: NextConsequence,
}

/// Validate an inbound verdict event against the seam contract: required fields
/// and the pinned verdict vocabulary. The top-level envelope is closed; per-cell
/// results tolerate executor-specific extras. Returns the parsed event.
pub fn validate_verdict(value: &Value) -> Result<VerdictEvent> {
    serde_json::from_value(value.clone()).map_err(|e| {
        ProductError::ConfigError(format!("invalid verdict event (build seam §5.1): {e}"))
    })
}

/// Reconcile a verdict against an emitted unit: the verdict is attributable iff
/// it echoes the unit's identity and the bundle hash it ran against.
pub fn reconciles(unit: &SeamWorkUnit, verdict: &VerdictEvent) -> bool {
    unit.unit_ref == verdict.unit_ref && unit.bundle_hash == verdict.bundle_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_unknown_verdict_value_is_rejected() {
        let ev = json!({
            "event-id": "ev-1", "emitted-at": "t", "unit-ref": "wu-1",
            "parent-deliverable": "d", "bundle-hash": "h", "verdict": "maybe",
            "tier-ran": "x", "cell-results": [], "next-consequence": "advance"
        });
        assert!(validate_verdict(&ev).is_err(), "the verdict vocabulary is pinned (§6.2)");
    }

    #[test]
    fn a_verdict_missing_a_required_field_is_rejected() {
        let ev = json!({ "event-id": "ev-1", "unit-ref": "wu-1", "verdict": "accepted" });
        assert!(validate_verdict(&ev).is_err(), "emitted-at, tier-ran, cell-results, next-consequence are required");
    }
}
