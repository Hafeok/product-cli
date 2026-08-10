//! Repo-independent identifiers for the ledger's entities (PRD §4.1).
//!
//! A decision's identity does not belong to a repository: seam decisions
//! cross repos, and the highest-consequence decisions often have no natural
//! repo home. So an id is `dec:<namespace>/<ulid>` — a ULID for offline
//! generation with lexicographic creation order, under an owning *scope*
//! rather than a path. The id is stable forever; supersession mints a new
//! decision with a `supersedes` edge, never a mutation or a reuse.
//!
//! L0 validates ids; it does not mint them. Generation arrives with L1's
//! `ledger add`, which is the first command that creates a decision.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Crockford base32 — the ULID alphabet, with I, L, O, U excluded so a
/// transcribed id cannot be read two ways.
const CROCKFORD: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A ULID rendered in the canonical 26-character Crockford base32 form.
pub fn validate_ulid(s: &str) -> Result<(), String> {
    if s.len() != 26 {
        return Err(format!("`{s}` is {} characters; a ULID is 26", s.len()));
    }
    if let Some(bad) = s.bytes().find(|b| !CROCKFORD.contains(b)) {
        return Err(format!(
            "`{s}` carries `{}`, which is outside Crockford base32 (I, L, O, U excluded)",
            bad as char
        ));
    }
    // The first character encodes the top 3 bits of a 48-bit millisecond
    // timestamp; anything above '7' overflows the ULID time field.
    if s.as_bytes().first().is_some_and(|b| *b > b'7') {
        return Err(format!("`{s}` overflows the 48-bit ULID timestamp field"));
    }
    Ok(())
}

/// A namespace: an owning scope, not a repo path. Dot-separated lowercase
/// segments, so `hafeok.ledger` reads as scope-within-scope.
fn validate_namespace(ns: &str) -> Result<(), String> {
    if ns.is_empty() {
        return Err("namespace is empty".to_string());
    }
    for segment in ns.split('.') {
        if segment.is_empty() {
            return Err(format!("`{ns}` has an empty namespace segment"));
        }
        let ok = segment
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !ok {
            return Err(format!(
                "`{ns}` segment `{segment}` is not lowercase alphanumeric or dashes"
            ));
        }
    }
    Ok(())
}

/// A decision's permanent identity: `dec:<namespace>/<ulid>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionId {
    namespace: String,
    ulid: String,
}

impl DecisionId {
    /// The owning scope this decision was minted under.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The ULID half, without the scheme or the namespace.
    pub fn ulid(&self) -> &str {
        &self.ulid
    }
}

impl FromStr for DecisionId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let body = s
            .strip_prefix("dec:")
            .ok_or_else(|| format!("`{s}` is not a decision id — expected `dec:<namespace>/<ulid>`"))?;
        let (namespace, ulid) = body
            .split_once('/')
            .ok_or_else(|| format!("`{s}` has no `/` between namespace and ULID"))?;
        validate_namespace(namespace)?;
        validate_ulid(ulid)?;
        Ok(Self { namespace: namespace.to_string(), ulid: ulid.to_string() })
    }
}

impl fmt::Display for DecisionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dec:{}/{}", self.namespace, self.ulid)
    }
}

/// A change-set id, `cs:<ulid>`; the log file it lands in is `<ulid>.yml`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChangeSetId(String);

/// An acceptance id, `acc:<ulid>`; a revocation names one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AcceptanceId(String);

macro_rules! prefixed_ulid_id {
    ($ty:ident, $prefix:literal, $label:literal) => {
        impl $ty {
            /// The ULID half, without the scheme prefix.
            pub fn ulid(&self) -> &str {
                &self.0
            }
        }

        impl FromStr for $ty {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let ulid = s.strip_prefix(concat!($prefix, ":")).ok_or_else(|| {
                    format!(concat!("`{}` is not a ", $label, " id — expected `", $prefix, ":<ulid>`"), s)
                })?;
                validate_ulid(ulid)?;
                Ok(Self(ulid.to_string()))
            }
        }

        impl fmt::Display for $ty {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!($prefix, ":{}"), self.0)
            }
        }
    };
}

prefixed_ulid_id!(ChangeSetId, "cs", "change-set");
prefixed_ulid_id!(AcceptanceId, "acc", "acceptance");

/// Serde for every id type: the wire form is the rendered string.
macro_rules! string_serde {
    ($ty:ty) => {
        impl Serialize for $ty {
            fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                s.serialize_str(&self.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                let raw = String::deserialize(d)?;
                raw.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

string_serde!(DecisionId);
string_serde!(ChangeSetId);
string_serde!(AcceptanceId);

#[path = "id_tests.rs"]
#[cfg(test)]
mod tests;
