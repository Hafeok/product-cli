//! The version hash — what an acceptance signs (§4.3).
//!
//! Acceptance signs the hash, never the id. That is the property that makes
//! acceptance mean something: it names an exact state, not "whatever this
//! decision currently says". Any edit to an accepted version moves the hash
//! and invalidates the acceptance, which is then a re-acceptance by a named
//! actor rather than a silent carry-over.
//!
//! The digest is taken over [`crate::canon`]'s bytes with a domain-separation
//! prefix carrying the canonical-form version, so a hash computed under one
//! reading of the fields can never collide with a hash computed under
//! another. See `docs/ledger-format-v1.md` for the specification an outside
//! implementation works from, including its pinned conformance vector.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

use crate::canon::{canonical_bytes, CANONICAL_FORM};
use crate::version::VersionRaw;

/// A `sha256:`-prefixed lowercase-hex digest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VersionHash(String);

impl VersionHash {
    /// The 64 hex characters, without the algorithm prefix.
    pub fn hex(&self) -> &str {
        self.0.strip_prefix("sha256:").unwrap_or(&self.0)
    }

    /// The first 12 hex characters, for display only. Never compared.
    pub fn short(&self) -> &str {
        self.hex().get(..12).unwrap_or(self.hex())
    }
}

impl FromStr for VersionHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s
            .strip_prefix("sha256:")
            .ok_or_else(|| format!("`{s}` is not a version hash — expected `sha256:<64 hex>`"))?;
        if hex.len() != 64 {
            return Err(format!("`{s}` has {} hex characters; sha-256 renders 64", hex.len()));
        }
        if let Some(bad) = hex.chars().find(|c| !c.is_ascii_hexdigit() || c.is_ascii_uppercase()) {
            return Err(format!("`{s}` carries `{bad}`; the digest is lowercase hex"));
        }
        Ok(Self(s.to_string()))
    }
}

impl fmt::Display for VersionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for VersionHash {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for VersionHash {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// Hash a version's canonical content.
///
/// `sha256( CANONICAL_FORM || 0x0A || canonical_bytes )`. The prefix is
/// domain separation *and* a version pin: a `format` bump that changes what a
/// hashed field means must bump `CANONICAL_FORM` too, so acceptances signed
/// under the old reading cannot silently re-point at the new one.
pub fn version_hash(raw: &VersionRaw) -> VersionHash {
    let mut hasher = Sha256::new();
    hasher.update(CANONICAL_FORM.as_bytes());
    hasher.update(b"\n");
    hasher.update(canonical_bytes(raw));
    VersionHash(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hash_round_trips_and_exposes_a_short_form() {
        let h: VersionHash = format!("sha256:{}", "a".repeat(64)).parse().expect("parse");
        assert_eq!(h.hex().len(), 64);
        assert_eq!(h.short(), "aaaaaaaaaaaa");
        assert_eq!(h.to_string(), format!("sha256:{}", "a".repeat(64)));
    }

    #[test]
    fn the_algorithm_prefix_is_required() {
        let err = "a".repeat(64).parse::<VersionHash>().expect_err("no prefix");
        assert!(err.contains("expected `sha256:<64 hex>`"), "{err}");
    }

    #[test]
    fn a_digest_of_the_wrong_length_names_the_length() {
        let err = "sha256:abc".parse::<VersionHash>().expect_err("short");
        assert!(err.contains("has 3 hex characters"), "{err}");
    }

    #[test]
    fn uppercase_hex_is_rejected_so_one_digest_has_one_spelling() {
        assert!(format!("sha256:{}", "A".repeat(64)).parse::<VersionHash>().is_err());
    }
}
