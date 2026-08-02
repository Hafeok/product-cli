//! Decision entries — volitional resolutions `basedOn` claims, by a principal.
//!
//! One flat YAML file per entry under `.ddd/decisions/`. Two kinds share the
//! directory: `decision` (requires ≥1 `based_on` claim — ontology rule 3) plus
//! `risk-acceptance` (the record a manifest suppression must cite — rule 4).
//! Both require a named principal.

use serde::{Deserialize, Serialize};

/// A decision or risk-acceptance record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    /// Entry format version (v1 is current).
    pub format: u32,
    /// Stable id `dec/<area>/<name>` (or `risk/<area>/<name>`).
    pub id: String,
    /// Which kind of volitional entry this is.
    #[serde(default)]
    pub kind: DecisionKind,
    /// Short human title.
    pub title: String,
    /// Why the resolution went this way.
    pub rationale: String,
    /// The named principal who made it; never empty.
    pub principal: String,
    /// Claim ids this decision rests on; ≥1 for `kind: decision`.
    #[serde(default)]
    pub based_on: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// The two volitional entry kinds sharing `.ddd/decisions/`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionKind {
    #[default]
    Decision,
    RiskAcceptance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_defaults_to_decision() {
        let d: Decision = serde_yaml::from_str(
            "format: 1\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on: [DDD-a-1]\n",
        )
        .expect("parse");
        assert_eq!(d.kind, DecisionKind::Decision);
        assert_eq!(d.based_on, vec!["DDD-a-1"]);
    }

    #[test]
    fn risk_acceptance_kind_parses() {
        let d: Decision = serde_yaml::from_str(
            "format: 1\nid: risk/x/y\nkind: risk-acceptance\ntitle: T\nrationale: R\nprincipal: Emil\n",
        )
        .expect("parse");
        assert_eq!(d.kind, DecisionKind::RiskAcceptance);
        assert!(d.based_on.is_empty());
    }
}
