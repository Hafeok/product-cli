//! The `revisit_if` edge — a claim whose death reopens a decision.
//!
//! Ruled by the principal (2026-08-13), settling the watched-edge question
//! the basis-quality re-typing session left open and the ddd M8 migration
//! carried as a provisional `watched:` marker inside `based_on`.
//!
//! A `revisit_if` pointer is **not** a basis. Its meaning is the converse of
//! ground: the decision does not rest on this claim, but the claim's death
//! reopens the decision. Filing it as its own field with its own type is
//! what keeps the two readings apart mechanically rather than by convention
//! — the basis-loss scan reads `based_on` and never this field, and a
//! status movement here produces a *reopen* finding, not a basis-loss one.
//! The two mean different things and must report differently.
//!
//! The vocabulary is open at this version, exactly as `based_on`'s is: L0
//! resolves no pointer, and closing a vocabulary nothing dereferences yet
//! would reject adopters' schemes for no gain. Open is not the same as
//! shared — a `RevisitRef` is a distinct type, so no code path can read one
//! as ground by passing it where a basis is expected.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A pointer to the claim whose falsification reopens a decision.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevisitRef(String);

impl RevisitRef {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RevisitRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t.is_empty() {
            return Err("a revisit-if pointer is empty".to_string());
        }
        if t.chars().any(char::is_whitespace) {
            return Err(format!("`{t}` carries whitespace; a revisit-if pointer is a single token"));
        }
        Ok(Self(t.to_string()))
    }
}

impl fmt::Display for RevisitRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for RevisitRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for RevisitRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_token_parses_and_whitespace_is_refused() {
        assert_eq!(
            "claim:DDD-adapter-02@sha256:abc".parse::<RevisitRef>().expect("parse").as_str(),
            "claim:DDD-adapter-02@sha256:abc"
        );
        assert!("two tokens".parse::<RevisitRef>().is_err());
        assert!("   ".parse::<RevisitRef>().is_err());
    }

    #[test]
    fn surrounding_whitespace_is_trimmed_like_a_basis_pointer() {
        assert_eq!("  claim:X  ".parse::<RevisitRef>().expect("parse").as_str(), "claim:X");
    }

    #[test]
    fn the_type_is_distinct_from_a_basis_pointer() {
        // The guard is structural: a `RevisitRef` cannot be passed where a
        // `BasisRef` is expected, so no scan can read one as ground by
        // accident. This test exists to fail loudly if the two are ever
        // merged into one type — see the module docs for why they are not.
        let revisit: RevisitRef = "claim:X@sha256:abc".parse().expect("parse");
        let basis: crate::version::BasisRef = "claim:X@sha256:abc".parse().expect("parse");
        assert_eq!(revisit.to_string(), basis.to_string());
        assert_ne!(
            std::any::TypeId::of::<RevisitRef>(),
            std::any::TypeId::of::<crate::version::BasisRef>()
        );
    }
}
