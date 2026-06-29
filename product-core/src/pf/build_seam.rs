//! The §5.1 Build seam — the work-unit emission contract.
//!
//! The boundary where specification ends and execution begins. The framework
//! fixes only the contract that crosses it: an emitted work unit travelling by
//! value with a content-hash identity ([`SeamWorkUnit`]), and a self-describing
//! [`VerdictEvent`] returning. Everything past the seam — scheduling, batching,
//! how realisation is produced — is the executor's concern, carried opaquely in
//! the one `executor_extension` slot. Mirrors `schema/json/build-seam/*`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{ProductError, Result};

use super::work_unit::WorkUnit;

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

/// The Transition-Contract consequence a consumer may act on (optional).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NextConsequence {
    Advance,
    Halt,
    Retry,
    Escalate,
}

/// The frozen SPMC bundle (§5). Each axis is opaque to the framework — its
/// internal shape is the executor's to read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpmcBundle {
    pub schema: Value,
    pub prompt: Value,
    pub model: Value,
    pub context: Value,
}

/// The emitted work unit: the universal envelope plus one opaque extension slot.
/// Travels by value; its `bundle_hash` *is* its identity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeamWorkUnit {
    pub unit_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_lineage: Option<String>,
    pub bundle_hash: String,
    pub bundle: SpmcBundle,
    pub acceptance_class: AcceptanceClass,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_extension: Option<Value>,
}

/// The self-describing verdict event: enough on its own for a consumer that
/// never saw the emission to reconcile it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictEvent {
    pub event_id: String,
    pub emitted_at: String,
    pub unit_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_lineage: Option<String>,
    pub bundle_hash: String,
    pub verdict: Verdict,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_consequence: Option<NextConsequence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executor_extension: Option<Value>,
}

/// Map an internal [`WorkUnit`] (§5) into the by-value seam envelope. The unit
/// must be frozen — its `context.hash` is the identity that crosses the seam; a
/// unit with no hash is not frozen and must not be emitted (the freeze-before-emit
/// obligation, §5.1).
pub fn to_seam_envelope(
    wu: &WorkUnit,
    acceptance_class: AcceptanceClass,
    parent_lineage: Option<&str>,
) -> Result<SeamWorkUnit> {
    let bundle_hash = wu.context.hash.clone().ok_or_else(|| {
        ProductError::ConfigError(format!(
            "work unit '{}' is not frozen (no content hash) — it must not be emitted across the seam",
            wu.id
        ))
    })?;
    let bundle = SpmcBundle {
        schema: json!({
            "artifact_type": wu.produces.artifact,
            "path": wu.produces.path,
            "acceptance": wu.schema,
        }),
        prompt: json!({ "transformation": wu.prompt }),
        model: json!({ "capability": wu.model.clone().unwrap_or_else(|| "code-implementation".to_string()) }),
        context: json!({
            "derived_from": wu.context.derived_from,
            "frozen": wu.context.frozen,
            "applies": wu.applies,
            "trace": wu.trace,
        }),
    };
    Ok(SeamWorkUnit {
        unit_ref: wu.id.clone(),
        parent_lineage: parent_lineage.map(String::from),
        bundle_hash,
        bundle,
        acceptance_class,
        executor_extension: None,
    })
}

/// Validate an inbound verdict event against the seam contract: required fields,
/// the pinned verdict vocabulary, and no fields outside the envelope (the lone
/// `executor_extension` slot apart). Returns the parsed event.
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

    #[test]
    fn an_unfrozen_unit_cannot_be_emitted() {
        let mut wu = frozen_unit();
        wu.context.hash = None;
        assert!(to_seam_envelope(&wu, AcceptanceClass::NeedsVerdict, None).is_err());
    }

    #[test]
    fn envelope_carries_identity_and_round_trips() {
        let env = to_seam_envelope(&frozen_unit(), AcceptanceClass::NeedsVerdict, Some("del-refunds")).unwrap();
        assert_eq!(env.unit_ref, "wu-1");
        assert_eq!(env.bundle_hash, "sha256:abc");
        // Serialises and the JSON validates back to the same envelope.
        let v = serde_json::to_value(&env).unwrap();
        let back = serde_json::from_value::<SeamWorkUnit>(v).unwrap();
        assert_eq!(env, back);
    }

    #[test]
    fn a_valid_verdict_parses_and_reconciles() {
        let env = to_seam_envelope(&frozen_unit(), AcceptanceClass::NeedsVerdict, None).unwrap();
        let ev = json!({
            "event_id": "ev-1", "emitted_at": "2026-06-26T02:14:08Z",
            "unit_ref": "wu-1", "bundle_hash": "sha256:abc",
            "verdict": "accepted", "next_consequence": "advance",
            "executor_extension": { "kind": "x", "tier_ran": "medium" }
        });
        let parsed = validate_verdict(&ev).unwrap();
        assert_eq!(parsed.verdict, Verdict::Accepted);
        assert!(reconciles(&env, &parsed), "matching identity + hash reconciles");
    }

    #[test]
    fn an_unknown_verdict_value_is_rejected() {
        let ev = json!({
            "event_id": "ev-1", "emitted_at": "t", "unit_ref": "wu-1",
            "bundle_hash": "h", "verdict": "maybe"
        });
        assert!(validate_verdict(&ev).is_err(), "the verdict vocabulary is pinned (§6.2)");
    }

    #[test]
    fn a_verdict_missing_a_required_field_is_rejected() {
        let ev = json!({ "event_id": "ev-1", "unit_ref": "wu-1", "verdict": "accepted" });
        assert!(validate_verdict(&ev).is_err(), "emitted_at + bundle_hash are required");
    }
}
