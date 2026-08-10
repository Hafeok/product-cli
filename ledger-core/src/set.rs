//! The declared scope coverage is measured against (§4.2, §8).
//!
//! A set fixes the tolerance floor, the ground state, and the owner. It does
//! **not** list its members: a decision names its set, and membership is
//! derived by query. Restating membership here would make every `add` a
//! rewrite of a shared file — fighting append-only, and conflicting on every
//! branch for no gain, since the denominator is the same either way.
//!
//! The honest limit, which §8 requires every coverage report to state:
//! coverage is measured against the *enumerated* set, and nothing verifies
//! the set itself. L0 ships no coverage query, but the limit is already true
//! of the floor rule — a set drawn too narrowly hides decisions rather than
//! allocating them.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::identity::Identity;
use crate::tier::Tier;

/// What is known about the ground a set's decisions are taken against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ground {
    /// The facts an actor needs are available for inspection.
    Characterised,
    /// They are not; the deliverable is discovery records, not the artifact.
    Uncharacterised,
}

/// A declared set, one file under `.decisions/sets/`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionSet {
    pub format: u32,
    pub id: String,
    pub title: String,
    /// The floor, not a default. A member may override up, never down.
    pub tolerance_floor: Tier,
    pub ground: Ground,
    pub owner: Identity,
    pub created_at: NaiveDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl DecisionSet {
    /// Whether a set id is usable as a filename stem and as a stable key.
    pub fn validate_id(id: &str) -> Result<(), String> {
        if id.is_empty() {
            return Err("set id is empty".to_string());
        }
        let ok = id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'.');
        if !ok {
            return Err(format!(
                "`{id}` is not a set id — lowercase alphanumerics, dashes and dots only"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const YAML: &str = "format: 1\nid: ledger-design\ntitle: L0 settled design\ntolerance_floor: T1\nground: characterised\nowner: emk@delegate.dk\ncreated_at: 2026-08-10\n";

    #[test]
    fn a_set_file_parses_with_its_floor_and_ground() {
        let s: DecisionSet = serde_yaml::from_str(YAML).expect("parse");
        assert_eq!(s.tolerance_floor, Tier::T1);
        assert_eq!(s.ground, Ground::Characterised);
        assert_eq!(s.owner.as_str(), "emk@delegate.dk");
        assert!(s.notes.is_none());
    }

    #[test]
    fn a_set_file_carries_no_membership_list() {
        let with_members = format!("{YAML}members: [dec:x/Y]\n");
        let err = serde_yaml::from_str::<DecisionSet>(&with_members).expect_err("no such field");
        assert!(err.to_string().contains("unknown field"), "{err}");
    }

    #[test]
    fn set_ids_are_filename_safe() {
        assert!(DecisionSet::validate_id("ledger-design").is_ok());
        assert!(DecisionSet::validate_id("ft.104").is_ok());
        assert!(DecisionSet::validate_id("Ledger Design").is_err());
        assert!(DecisionSet::validate_id("../escape").is_err());
        assert!(DecisionSet::validate_id("").is_err());
    }

    #[test]
    fn a_set_round_trips_through_yaml() {
        let s: DecisionSet = serde_yaml::from_str(YAML).expect("parse");
        let text = serde_yaml::to_string(&s).expect("serialize");
        assert_eq!(serde_yaml::from_str::<DecisionSet>(&text).expect("reparse"), s);
    }
}
