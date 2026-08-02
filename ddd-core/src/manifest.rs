//! Analyzer/linter manifests — one governed entry per enabled rule.
//!
//! `.ddd/manifest/analyzers.yaml` covers Roslyn rules, `linter.yaml` the
//! `bicepconfig.json` rules (PRD §6). Every entry maps to a decision — or is
//! explicitly `UNGOVERNED` pending one (a warning, not a violation, so `init`
//! skeletons can pass CI). A suppression must cite a risk-acceptance record.

use serde::{Deserialize, Serialize};

/// The marker value an entry carries while no decision governs it yet.
pub const UNGOVERNED: &str = "UNGOVERNED";

/// One manifest file: a format-versioned list of rule entries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestFile {
    /// Entry format version (v1 is current).
    pub format: u32,
    #[serde(default)]
    pub rules: Vec<ManifestRule>,
}

/// One enabled analyzer/linter rule with its governance link.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestRule {
    /// The diagnostic id as the toolchain reports it (e.g. `CA2007`).
    pub rule_id: String,
    /// The configured severity — the assurance level the decision bought.
    pub severity: String,
    /// The governing decision id; every entry maps to one (rule 4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    /// `UNGOVERNED` while a decision entry is pending (from `ddd init`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance: Option<String>,
    /// Present when the rule is suppressed; must cite a risk acceptance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suppression: Option<Suppression>,
}

/// A suppression record; `risk_acceptance` names the record it cites.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Suppression {
    /// The risk-acceptance record id the suppression cites (rule 4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk_acceptance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A named manifest file loaded from `.ddd/manifest/<name>.yaml`.
#[derive(Debug, Clone)]
pub struct ManifestSet {
    /// The file stem, e.g. `analyzers` or `linter`.
    pub name: String,
    pub file: ManifestFile,
}

impl ManifestSet {
    /// The graph id of one entry: `<file-stem>/<rule_id>`.
    pub fn entry_id(&self, rule: &ManifestRule) -> String {
        format!("{}/{}", self.name, rule.rule_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_governed_and_an_ungoverned_entry() {
        let m: ManifestFile = serde_yaml::from_str(
            "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/cs/async-config\n  - rule_id: BCP037\n    severity: error\n    governance: UNGOVERNED\n",
        )
        .expect("parse");
        assert_eq!(m.rules.len(), 2);
        assert_eq!(m.rules[0].decision.as_deref(), Some("dec/cs/async-config"));
        assert_eq!(m.rules[1].governance.as_deref(), Some(UNGOVERNED));
    }

    #[test]
    fn entry_id_joins_file_stem_with_rule_id() {
        let set = ManifestSet {
            name: "analyzers".into(),
            file: serde_yaml::from_str("format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n").expect("parse"),
        };
        assert_eq!(set.entry_id(&set.file.rules[0]), "analyzers/CA2007");
    }
}
