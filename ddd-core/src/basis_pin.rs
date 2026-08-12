//! Content-hash basis pins — basis loss as an exact comparison (M8,
//! decision format 6, spec invariant 2).
//!
//! A format-6 decision pins every claim edge with the hash of the
//! claim's canonical content at decision time; basis loss becomes
//! pinned-hash ≠ current-hash, mechanically exact, retiring the
//! status+date heuristic. The canonical form rides the ledger's
//! canonicalisation law (`ledger_core::canon`) under this payload's own
//! domain prefix — one law, never a second scheme.

use serde_json::{Map, Value};

use crate::claim::Claim;

/// Domain-separation prefix for claim-content hashes. A change to the
/// hashed field set or its normalisation bumps this to `.v2` with a
/// migration note — the ledger's `CANONICAL_FORM` discipline.
pub const CLAIM_CONTENT_FORM: &str = "ddd.claim-content.v1";

/// The hash a format-6 basis pin stores: the claim's canonical content,
/// domain-separated. Status and `changed` are part of the content, so an
/// equal hash implies the status+date pin holds too — one comparison
/// subsumes the heuristic it retires.
pub fn claim_content_hash(claim: &Claim) -> String {
    ledger_core::hash::domain_hash(CLAIM_CONTENT_FORM, canonical_json(claim).as_bytes())
}

/// The canonical JSON of a claim's content — exhaustive destructure, no
/// rest pattern, so a new claim field cannot join (or silently miss) the
/// hashed form without someone deciding.
pub fn canonical_json(claim: &Claim) -> String {
    // `format` is bound and discarded: it says how the *file* is read,
    // not what the claim states — re-declaring a file's format is not a
    // content change and must not read as basis loss.
    let Claim {
        format: _,
        id,
        statement,
        status,
        evidence,
        falsifier,
        owner,
        changed,
        revalidate_by,
        version_index,
        depends_on,
        refines,
        predicate,
    } = claim;
    let mut m = Map::new();
    for (key, value) in [
        ("id", Some(id.clone())),
        ("statement", Some(statement.clone())),
        ("status", Some(status.as_str().to_string())),
        ("evidence", evidence.clone()),
        ("falsifier", falsifier.clone()),
        ("owner", Some(owner.clone())),
        ("changed", Some(changed.clone())),
        ("revalidate_by", revalidate_by.clone()),
        ("predicate", predicate.clone()),
        ("version_index.language", version_index.as_ref().and_then(|v| v.language.clone())),
        ("version_index.sdk", version_index.as_ref().and_then(|v| v.sdk.clone())),
        ("version_index.target", version_index.as_ref().and_then(|v| v.target.clone())),
        ("version_index.tool", version_index.as_ref().and_then(|v| v.tool.clone())),
    ] {
        ledger_core::canon::put(&mut m, key, value);
    }
    ledger_core::canon::put_set(&mut m, "depends_on", depends_on.iter().cloned());
    ledger_core::canon::put_set(&mut m, "refines", refines.iter().cloned());
    Value::Object(m).to_string()
}

/// Domain-separation prefix for decision-content hashes: what a migrated
/// ledger entry pins of its historical `.ddd` decision file.
pub const DECISION_CONTENT_FORM: &str = "ddd.decision-content.v1";

/// Domain-separation prefix for seam-content hashes. A seam's `bindings`
/// are deliberately outside the hash: they are the append-only discharge
/// log, each sealed by its own binding hash — routine discharge must not
/// read as content drift on the declaration.
pub const SEAM_CONTENT_FORM: &str = "ddd.seam-content.v1";

/// The hash a migrated ledger entry pins of its decision's content.
pub fn decision_content_hash(d: &crate::decision::Decision) -> String {
    let crate::decision::Decision {
        format: _, id, kind, title, rationale, principal, based_on, date, notes,
    } = d;
    let mut m = Map::new();
    for (key, value) in [
        ("id", Some(id.clone())),
        ("kind", serde_json::to_value(kind).ok().and_then(|v| v.as_str().map(str::to_string))),
        ("title", Some(title.clone())),
        ("rationale", Some(rationale.clone())),
        ("principal", Some(principal.clone())),
        ("date", date.clone()),
        ("notes", notes.clone()),
    ] {
        ledger_core::canon::put(&mut m, key, value);
    }
    // Bases keep their authored order — primary vs. secondary is meaning.
    let bases: Vec<Value> = based_on.iter().filter_map(|b| serde_json::to_value(b).ok()).collect();
    if !bases.is_empty() {
        m.insert("based_on".to_string(), Value::Array(bases));
    }
    ledger_core::hash::domain_hash(DECISION_CONTENT_FORM, Value::Object(m).to_string().as_bytes())
}

/// The hash a migrated ledger entry pins of its seam declaration.
pub fn seam_content_hash(s: &crate::seam::SeamDeclaration) -> String {
    let crate::seam::SeamDeclaration {
        format: _,
        id,
        boundary,
        verdict_knowledge,
        contract_location,
        obligations,
        metadata,
        bindings: _,
        notes,
    } = s;
    let mut m = Map::new();
    for (key, value) in [
        ("id", Some(id.clone())),
        ("boundary", Some(boundary.clone())),
        ("verdict_knowledge", Some(verdict_knowledge.clone())),
        ("contract_location", Some(contract_location.clone())),
        ("notes", notes.clone()),
    ] {
        ledger_core::canon::put(&mut m, key, value);
    }
    let obligations: Vec<Value> =
        obligations.iter().map(|o| Value::String(o.clone())).collect();
    if !obligations.is_empty() {
        m.insert("obligations".to_string(), Value::Array(obligations));
    }
    let meta: Map<String, Value> = metadata
        .iter()
        .filter_map(|(k, v)| serde_json::to_value(v).ok().map(|j| (k.clone(), j)))
        .collect();
    if !meta.is_empty() {
        m.insert("metadata".to_string(), Value::Object(meta));
    }
    ledger_core::hash::domain_hash(SEAM_CONTENT_FORM, Value::Object(m).to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::ClaimStatus;

    fn claim() -> Claim {
        Claim {
            format: 2,
            id: "DDD-x-01".into(),
            statement: "X closes Y.".into(),
            status: ClaimStatus::Reported,
            evidence: Some("exercised".into()),
            falsifier: Some("Y escapes X".into()),
            owner: "none".into(),
            changed: "2026-08-01".into(),
            revalidate_by: None,
            version_index: None,
            depends_on: vec!["DDD-y-01".into()],
            refines: Vec::new(),
            predicate: None,
        }
    }

    #[test]
    fn the_hash_is_stable_and_domain_separated() {
        let h = claim_content_hash(&claim());
        assert!(h.starts_with("sha256:"));
        assert_eq!(h, claim_content_hash(&claim()));
    }

    #[test]
    fn a_status_move_moves_the_hash() {
        let mut moved = claim();
        moved.status = ClaimStatus::Established;
        assert_ne!(claim_content_hash(&claim()), claim_content_hash(&moved));
    }

    #[test]
    fn a_format_redeclaration_is_not_a_content_change() {
        let mut redeclared = claim();
        redeclared.format = 4;
        assert_eq!(claim_content_hash(&claim()), claim_content_hash(&redeclared));
    }
}
