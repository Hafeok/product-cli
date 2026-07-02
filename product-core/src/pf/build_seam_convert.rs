//! Build-seam conversion — mapping an internal [`WorkUnit`] (§5) into the
//! canonical by-value seam envelope. Split from the type definitions in
//! [`super::build_seam`] to keep each file within the repo's length budget; the
//! entry points are re-exported from `build_seam`, so the public path stays
//! `product_core::pf::build_seam::{to_seam_envelope, SeamParams}`.

use serde_json::{json, Value};

use crate::error::{ProductError, Result};
use crate::pf::work_unit::WorkUnit;

use super::build_seam::{
    AcceptanceClass, ArtifactDelivery, CellGraph, CellOutput, CellPrompt, CellSchema,
    ContextFragment, ContextPool, ModelBinding, ModelSection, SeamCell, SeamWorkUnit, SpmcBundle,
};

/// The parameters the producer pins at dispatch that the internal [`WorkUnit`]
/// (§5) does not itself carry: the served model binding, the tier it runs at,
/// the escalation rung, and the artifact transport.
#[derive(Debug, Clone)]
pub struct SeamParams<'a> {
    pub acceptance_class: AcceptanceClass,
    pub parent_deliverable: &'a str,
    pub tier: String,
    pub binding: ModelBinding,
    pub capability_tag: Option<String>,
    pub ladder_position: u32,
    pub artifact_delivery: ArtifactDelivery,
}

/// Normalize a graph id into a fragment handle (`Order#decide` → `frag-order-decide`).
fn frag_id(raw: &str) -> String {
    let slug: String = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    format!("frag-{}", slug.trim_matches('-'))
}

/// The frozen `bundle-hash` (contract `algo:digest`), or an error if the unit is
/// not frozen (no content hash) and so must not be emitted (§5.1).
fn frozen_bundle_hash(wu: &WorkUnit) -> Result<String> {
    let hash = wu.context.hash.clone().ok_or_else(|| {
        ProductError::ConfigError(format!(
            "work unit '{}' is not frozen (no content hash) — it must not be emitted across the seam",
            wu.id
        ))
    })?;
    Ok(if hash.contains(':') { hash } else { format!("sha256:{hash}") })
}

/// Inline the unit's declared context (`derived_from` / `applies` / `trace`) as
/// the content-addressed fragment pool, returning the fragments and the cell's
/// `context-refs` (deduped, in declaration order).
fn inline_context(wu: &WorkUnit) -> (Vec<ContextFragment>, Vec<String>) {
    let mut fragments = Vec::new();
    let mut refs = Vec::new();
    let mut push = |raw: &str, role: &str, content: Value| {
        let id = frag_id(raw);
        if refs.iter().any(|r| r == &id) {
            return;
        }
        refs.push(id.clone());
        fragments.push(ContextFragment { id, role: Some(role.into()), media_type: None, content, provenance: None });
    };
    for concept in &wu.context.derived_from {
        push(concept, "what", json!(concept));
    }
    for decision in &wu.applies {
        push(decision, "decision", json!(decision));
    }
    if let Some(trace) = &wu.trace {
        push("trace", "trace", serde_json::to_value(trace).unwrap_or(Value::Null));
    }
    (fragments, refs)
}

/// Build the single cell a product work unit (one bounded transformation) maps
/// to: its Schema (S) and Prompt (P) inline, its declared context refs (C), and
/// the artifact it produces.
fn single_cell(wu: &WorkUnit, context_refs: Vec<String>) -> SeamCell {
    let artifact_id = if wu.produces.artifact.trim().is_empty() {
        format!("art-{}", frag_id(&wu.id).trim_start_matches("frag-"))
    } else {
        wu.produces.artifact.clone()
    };
    SeamCell {
        id: wu.id.clone(),
        requires: Vec::new(),
        schema: CellSchema {
            shape_language: "prose".into(),
            shape_version: None,
            document: json!({ "description": wu.schema }),
        },
        prompt: CellPrompt { content: json!(wu.prompt), prompt_version: None },
        context_refs,
        output: CellOutput {
            artifact_id,
            media_type: None,
            path: Some(wu.produces.path.clone()),
            description: None,
        },
        gate: None,
    }
}

/// Map an internal [`WorkUnit`] (§5) into the canonical by-value seam envelope.
/// The unit must be frozen (§5.1). A product WorkUnit is one bounded
/// transformation, so it maps to a single-cell cell-graph; its declared
/// `derived_from` / `applies` / `trace` become the inlined context fragments.
pub fn to_seam_envelope(wu: &WorkUnit, params: &SeamParams) -> Result<SeamWorkUnit> {
    let bundle_hash = frozen_bundle_hash(wu)?;
    let (fragments, context_refs) = inline_context(wu);
    let cell = single_cell(wu, context_refs);
    Ok(SeamWorkUnit {
        unit_ref: wu.id.clone(),
        parent_deliverable: params.parent_deliverable.to_string(),
        bundle_hash,
        tier: params.tier.clone(),
        acceptance_class: params.acceptance_class,
        ladder_position: params.ladder_position,
        artifact_delivery: params.artifact_delivery.clone(),
        spmc_bundle: SpmcBundle {
            model: ModelSection {
                capability_tag: params.capability_tag.clone(),
                binding: params.binding.clone(),
            },
            context_pool: ContextPool {
                bundle_form_profile: Some("json-canonical".into()),
                fragments,
            },
        },
        cell_graph: CellGraph { cells: vec![cell] },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::build_seam::{reconciles, validate_verdict, Verdict};
    use crate::pf::work_unit::{Context, Produces, WorkUnit};

    fn frozen_unit() -> WorkUnit {
        WorkUnit {
            id: "wu-1".into(),
            schema: "a pure decide() fn".into(),
            prompt: "Realise the Order refund Decider.".into(),
            model: Some("code-implementation".into()),
            context: Context { derived_from: vec!["Order".into()], frozen: true, hash: Some("sha256:abc".into()) },
            produces: Produces { artifact: "module".into(), path: "src/order.rs".into() },
            applies: vec!["DEC-1".into()],
            trace: None,
        }
    }

    fn params<'a>() -> SeamParams<'a> {
        SeamParams {
            acceptance_class: AcceptanceClass::NeedsVerdict,
            parent_deliverable: "del-refunds",
            tier: "constrained-implementer".into(),
            binding: ModelBinding {
                provider: "local-vllm".into(),
                model_id: "coder-m".into(),
                revision: None,
                architecture: None,
                quantization: "fp8".into(),
                invocation: json!({ "temperature": 0 }),
            },
            capability_tag: Some("constrained-implementer".into()),
            ladder_position: 0,
            artifact_delivery: ArtifactDelivery::Inline,
        }
    }

    #[test]
    fn an_unfrozen_unit_cannot_be_emitted() {
        let mut wu = frozen_unit();
        wu.context.hash = None;
        assert!(to_seam_envelope(&wu, &params()).is_err());
    }

    #[test]
    fn envelope_is_canonical_and_round_trips() {
        let env = to_seam_envelope(&frozen_unit(), &params()).unwrap();
        assert_eq!(env.unit_ref, "wu-1");
        assert_eq!(env.parent_deliverable, "del-refunds");
        assert_eq!(env.bundle_hash, "sha256:abc");
        assert_eq!(env.cell_graph.cells.len(), 1);

        // Serialises to the CANONICAL kebab-case wire shape.
        let v = serde_json::to_value(&env).unwrap();
        for k in ["unit-ref", "parent-deliverable", "bundle-hash", "tier", "acceptance-class", "ladder-position", "artifact-delivery", "spmc-bundle", "cell-graph"] {
            assert!(v.get(k).is_some(), "missing kebab-case key {k}: {v}");
        }
        assert!(v["spmc-bundle"]["model"]["binding"]["model-id"].is_string());
        assert!(v["spmc-bundle"]["context-pool"]["fragments"].is_array());
        assert_eq!(v["cell-graph"]["cells"][0]["id"], "wu-1");
        assert!(v["cell-graph"]["cells"][0]["schema"]["shape-language"].is_string());
        assert!(v["cell-graph"]["cells"][0]["prompt"]["content"].is_string());

        // …and validates back to the same envelope.
        let back = serde_json::from_value::<SeamWorkUnit>(v).unwrap();
        assert_eq!(env, back);
    }

    #[test]
    fn a_canonical_verdict_parses_and_reconciles() {
        let env = to_seam_envelope(&frozen_unit(), &params()).unwrap();
        let ev = json!({
            "event-id": "ev-1", "emitted-at": "2026-06-26T02:14:08Z",
            "unit-ref": "wu-1", "parent-deliverable": "del-refunds", "bundle-hash": "sha256:abc",
            "verdict": "accepted", "tier-ran": "constrained-implementer",
            "cell-results": [ { "cell-id": "wu-1", "verdict": "accepted", "passed": true } ],
            "next-consequence": "advance"
        });
        let parsed = validate_verdict(&ev).unwrap();
        assert_eq!(parsed.verdict, Verdict::Accepted);
        assert_eq!(parsed.cell_results.len(), 1);
        assert!(reconciles(&env, &parsed), "matching identity + hash reconciles");
    }
}
