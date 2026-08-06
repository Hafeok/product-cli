//! Decision entries — volitional resolutions `basedOn` claims, by a principal.
//!
//! One flat YAML file per entry under `.ddd/decisions/`. Two kinds share the
//! directory: `decision` (requires ≥1 `based_on` claim — ontology rule 3) plus
//! `risk-acceptance` (the record a manifest suppression must cite — rule 4).
//! Both require a named principal. Format 2 (PRD §6 rule 6) pins each
//! `based_on` edge with the claim's status and `changed` value at decision
//! time, so basis loss is detectable by comparing pin to current.

use serde::{Deserialize, Serialize};

use crate::claim::ClaimStatus;

/// A decision or risk-acceptance record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    /// Entry format version: 1 (plain basedOn ids) or 2 (pinned edges).
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
    /// Claims this decision rests on; ≥1 for `kind: decision`. Format 1
    /// files plain claim ids; format 2 pins each edge (rule 6).
    #[serde(default)]
    pub based_on: Vec<BasedOn>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// One `based_on` edge: a plain claim id (format 1) or a pinned edge
/// carrying the claim's status plus `changed` at decision time (format 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BasedOn {
    /// Format 1: the claim id alone — basis loss is not detectable.
    Plain(String),
    /// Format 2: the id plus the pin taken when the decision was made.
    Pinned(BasisPin),
}

/// The pin on a format-2 `based_on` edge (PRD §6 rule 6).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasisPin {
    /// The claim id the edge lands on.
    pub claim: String,
    /// The claim's status at decision time.
    pub status: ClaimStatus,
    /// The claim's `changed` date at decision time.
    pub changed: String,
}

impl BasedOn {
    /// The claim id the edge points at, pinned or not.
    pub fn claim_id(&self) -> &str {
        match self {
            Self::Plain(id) => id,
            Self::Pinned(pin) => &pin.claim,
        }
    }

    /// The pin, when the edge carries one.
    pub fn pin(&self) -> Option<&BasisPin> {
        match self {
            Self::Plain(_) => None,
            Self::Pinned(pin) => Some(pin),
        }
    }
}

impl From<&str> for BasedOn {
    fn from(id: &str) -> Self {
        Self::Plain(id.to_string())
    }
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
        assert_eq!(d.based_on.len(), 1);
        assert_eq!(d.based_on[0].claim_id(), "DDD-a-1");
        assert!(d.based_on[0].pin().is_none());
    }

    #[test]
    fn a_format_two_pinned_edge_parses_and_round_trips() {
        let text = "format: 2\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on:\n  - claim: DDD-a-1\n    status: reported\n    changed: 2026-08-02\n";
        let d: Decision = serde_yaml::from_str(text).expect("parse");
        let pin = d.based_on[0].pin().expect("pinned");
        assert_eq!(d.based_on[0].claim_id(), "DDD-a-1");
        assert_eq!(pin.status, ClaimStatus::Reported);
        assert_eq!(pin.changed, "2026-08-02");
        let out = serde_yaml::to_string(&d).expect("serialize");
        let back: Decision = serde_yaml::from_str(&out).expect("reparse");
        assert!(back.based_on[0].pin().is_some(), "pin lost on round-trip:\n{out}");
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
