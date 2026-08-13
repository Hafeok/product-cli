//! Decision entries — volitional resolutions `basedOn` claims, by a principal.
//!
//! One flat YAML file per entry under `.ddd/decisions/`. Two kinds share the
//! directory: `decision` (requires ≥1 typed basis — ontology rule 3) plus
//! `risk-acceptance` (the record a manifest suppression must cite — rule 4).
//! Both require a named principal. Format 2 (PRD §6 rule 6) pins each
//! `based_on` edge with the claim's status and `changed` value at decision
//! time, so basis loss is detectable by comparing pin to current. Format 5
//! types each basis (`claim | constraint | mandate | preference | experiment
//! | risk-acceptance`) — a claim edge stays the load-bearing form where a
//! claim is the basis; the other types are honest first-class bases, so no
//! decision has to manufacture a weak claim to pass validation.

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
    /// Format 7: claims whose *death reopens* this decision. Never ground —
    /// a separate field, a separate type, a separate scan, so nothing can
    /// read one as basis (`crate::revisit`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisit_if: Vec<crate::revisit::RevisitEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// One basis entry: a plain claim id (format 1), a pinned claim edge
/// carrying the claim's status plus `changed` at decision time (format 2),
/// or a typed non-claim basis (format 5). A format-5 claim basis is the
/// pinned form with `type: claim` declared.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BasedOn {
    /// Format 1: the claim id alone — basis loss is not detectable.
    Plain(String),
    /// Format 2: the id plus the pin taken when the decision was made.
    /// Format 5 adds `type: claim` on the same shape.
    Pinned(BasisPin),
    /// Format 5: a non-claim basis, typed from the ruled vocabulary.
    Typed(TypedBasis),
}

/// The pin on a format-2 `based_on` edge (PRD §6 rule 6).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BasisPin {
    /// Format 5: the declared basis type; when present it must be `claim`
    /// (any other type carries no pin and parses as `TypedBasis`).
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub basis_type: Option<BasisType>,
    /// The claim id the edge lands on.
    pub claim: String,
    /// The claim's status at decision time.
    pub status: ClaimStatus,
    /// The claim's `changed` date at decision time.
    pub changed: String,
    /// Format 6 (M8): the hash of the claim's canonical content at
    /// decision time (`crate::basis_pin`). Basis loss becomes
    /// pinned-hash ≠ current-hash — mechanically exact (invariant 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// A format-5 non-claim basis. What it rests on is stated, not statused:
/// `statement` says what the constraint/mandate/preference/experiment is;
/// `ref` points at a record where one exists (required for
/// `risk-acceptance`, whose record lives in `decisions/`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedBasis {
    /// The basis type; never `claim` after validation (a claim basis pins).
    #[serde(rename = "type")]
    pub basis_type: BasisType,
    /// What this basis is, in one honest sentence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statement: Option<String>,
    /// A record the basis resolves to (`risk/...` for risk-acceptance;
    /// optional pointer — a doc anchor, an id — for the other types).
    #[serde(rename = "ref", default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

/// The ruled basis-type vocabulary (format 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BasisType {
    Claim,
    Constraint,
    Mandate,
    Preference,
    Experiment,
    RiskAcceptance,
}

impl BasisType {
    /// The kebab-case label, as filed.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Claim => "claim",
            Self::Constraint => "constraint",
            Self::Mandate => "mandate",
            Self::Preference => "preference",
            Self::Experiment => "experiment",
            Self::RiskAcceptance => "risk-acceptance",
        }
    }
}

impl BasedOn {
    /// The claim id the edge points at; `None` for a non-claim basis.
    pub fn claim_id(&self) -> Option<&str> {
        match self {
            Self::Plain(id) => Some(id),
            Self::Pinned(pin) => Some(&pin.claim),
            Self::Typed(_) => None,
        }
    }

    /// The pin, when the edge carries one.
    pub fn pin(&self) -> Option<&BasisPin> {
        match self {
            Self::Pinned(pin) => Some(pin),
            Self::Plain(_) | Self::Typed(_) => None,
        }
    }

    /// The basis type: declared on format-5 entries, `claim` implicitly on
    /// format 1–2 edges (the migration note states this reading).
    pub fn basis_type(&self) -> BasisType {
        match self {
            Self::Plain(_) => BasisType::Claim,
            Self::Pinned(pin) => pin.basis_type.unwrap_or(BasisType::Claim),
            Self::Typed(t) => t.basis_type,
        }
    }

    /// Whether the entry declares a type (the format-5 feature).
    pub fn is_typed(&self) -> bool {
        match self {
            Self::Plain(_) => false,
            Self::Pinned(pin) => pin.basis_type.is_some(),
            Self::Typed(_) => true,
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
        assert_eq!(d.based_on[0].claim_id(), Some("DDD-a-1"));
        assert!(d.based_on[0].pin().is_none());
        assert_eq!(d.based_on[0].basis_type(), BasisType::Claim);
        assert!(!d.based_on[0].is_typed());
    }

    #[test]
    fn a_format_two_pinned_edge_parses_and_round_trips() {
        let text = "format: 2\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on:\n  - claim: DDD-a-1\n    status: reported\n    changed: 2026-08-02\n";
        let d: Decision = serde_yaml::from_str(text).expect("parse");
        let pin = d.based_on[0].pin().expect("pinned");
        assert_eq!(d.based_on[0].claim_id(), Some("DDD-a-1"));
        assert_eq!(pin.status, ClaimStatus::Reported);
        assert_eq!(pin.changed, "2026-08-02");
        let out = serde_yaml::to_string(&d).expect("serialize");
        let back: Decision = serde_yaml::from_str(&out).expect("reparse");
        assert!(back.based_on[0].pin().is_some(), "pin lost on round-trip:\n{out}");
    }

    #[test]
    fn a_format_five_typed_claim_basis_parses_as_a_pinned_edge() {
        let text = "format: 5\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on:\n  - type: claim\n    claim: DDD-a-1\n    status: reported\n    changed: 2026-08-02\n";
        let d: Decision = serde_yaml::from_str(text).expect("parse");
        assert_eq!(d.based_on[0].claim_id(), Some("DDD-a-1"));
        assert!(d.based_on[0].pin().is_some());
        assert_eq!(d.based_on[0].basis_type(), BasisType::Claim);
        assert!(d.based_on[0].is_typed());
        let out = serde_yaml::to_string(&d).expect("serialize");
        let back: Decision = serde_yaml::from_str(&out).expect("reparse");
        assert!(back.based_on[0].is_typed(), "type lost on round-trip:\n{out}");
    }

    #[test]
    fn a_format_five_non_claim_basis_round_trips_with_its_type() {
        let text = "format: 5\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on:\n  - type: preference\n    statement: Four documents beat one\n  - type: risk-acceptance\n    ref: risk/rule/ca2007\n";
        let d: Decision = serde_yaml::from_str(text).expect("parse");
        assert_eq!(d.based_on[0].basis_type(), BasisType::Preference);
        assert_eq!(d.based_on[0].claim_id(), None);
        assert!(d.based_on[0].pin().is_none());
        assert_eq!(d.based_on[1].basis_type(), BasisType::RiskAcceptance);
        let out = serde_yaml::to_string(&d).expect("serialize");
        let back: Decision = serde_yaml::from_str(&out).expect("reparse");
        assert_eq!(back.based_on[0].basis_type(), BasisType::Preference);
        assert_eq!(back.based_on[1].basis_type(), BasisType::RiskAcceptance);
        match &back.based_on[1] {
            BasedOn::Typed(t) => assert_eq!(t.reference.as_deref(), Some("risk/rule/ca2007")),
            other => panic!("expected typed basis, got {other:?}"),
        }
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
