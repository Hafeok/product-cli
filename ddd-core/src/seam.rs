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
    /// Signed transitions this declaration discharges (M8, seam format 2):
    /// each binding names the exact change — subject symbol, before/after
    /// content hashes, base revision. Append-only in practice: a binding,
    /// once used, is the durable record of what was discharged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bindings: Vec<crate::binding::SeamBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Whether filing created a new entry or revised one already filed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filed {
    Created,
    Amended,
}

impl Filed {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "filed",
            Self::Amended => "amended",
        }
    }
}

/// The path a declaration id files to, under a `.ddd/seams/` directory.
pub fn path_for(seams_dir: &std::path::Path, id: &str) -> std::path::PathBuf {
    let slug: String = id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    seams_dir.join(format!("{slug}.yaml"))
}

/// Read the declaration filed at `path`, when one is.
pub fn load_at(path: &std::path::Path) -> Option<SeamDeclaration> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}

/// Resolve the create-vs-amend intent, the way the declare tools do
/// (`dec/ddd/amend-explicit-evidence-frozen`): `amend` requires an entry to
/// exist, and its absence requires that none does.
pub fn resolve_intent(
    path: &std::path::Path,
    id: &str,
    amend: bool,
) -> Result<Option<SeamDeclaration>, String> {
    match (amend, path.exists()) {
        (false, true) => {
            Err(format!("{id} is already filed — pass `amend: true` to revise it"))
        }
        (true, false) => Err(format!("nothing is filed at {id} — drop `amend` to file it")),
        (true, true) => load_at(path)
            .map(Some)
            .ok_or_else(|| format!("{id} is filed but unreadable")),
        (false, false) => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slug_is_derived_from_the_id() {
        let p = path_for(std::path::Path::new("/x/seams"), "seam/what/payments-api");
        assert!(p.ends_with("seam-what-payments-api.yaml"), "{p:?}");
    }

    #[test]
    fn intent_refuses_a_create_over_a_file_and_an_amend_over_nothing() {
        let dir = std::path::Path::new("/definitely/not/here");
        let p = path_for(dir, "seam/x/y");
        assert!(resolve_intent(&p, "seam/x/y", false).expect("create").is_none());
        assert!(resolve_intent(&p, "seam/x/y", true).is_err());
    }

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
