//! Seam-declaration entries — boundaries with their declared demand.
//!
//! One flat YAML file per declaration under `.ddd/seams/`. A boundary is only
//! justified when it encodes something about the verdict; `verdict_knowledge`
//! states what. M4's interceptor harvests these with LSP-derived structural
//! metadata; M1 files them by hand, so `metadata` stays free-form.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A declared seam: the boundary, the demand it absorbs, its obligations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeamDeclaration {
    /// Entry format version (v1 is current).
    pub format: u32,
    /// Stable id `seam/<area>/<name>`.
    pub id: String,
    /// The boundary this declares (e.g. a public type or a module contract).
    pub boundary: String,
    /// What the boundary encodes about the verdict; empty means seam cost
    /// with no demand absorbed (PRD §8 warns on it at declaration time).
    pub verdict_knowledge: String,
    /// Where the contract lives (file, symbol, or module reference).
    pub contract_location: String,
    #[serde(default)]
    pub obligations: Vec<String>,
    /// Structural metadata (symbol, kind, reference count) — the
    /// correspondence-dataset row; free-form until M4 fills it from LSP.
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_free_form_metadata() {
        let s: SeamDeclaration = serde_yaml::from_str(
            "format: 1\nid: seam/core/store-api\nboundary: ddd-core public store surface\nverdict_knowledge: which entries parsed cleanly\ncontract_location: ddd-core/src/store.rs\nmetadata:\n  reference_count: 3\n",
        )
        .expect("parse");
        assert_eq!(s.id, "seam/core/store-api");
        assert!(s.metadata.contains_key("reference_count"));
    }
}
