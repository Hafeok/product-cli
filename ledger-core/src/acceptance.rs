//! Acceptance — a named human signing an exact version hash (§4.3, §9.3).
//!
//! Acceptance is the entry that compounds, and the one act that cannot be
//! delegated. Models may propose enumerations, allocations and discharge
//! procedures; routing *acceptance* to a model makes the composite pinnable
//! only distributionally, at which point outcome-accountability is
//! unavailable rather than merely absent. So an acceptance names an identity
//! and signs a hash, and verify polices both (classes `L006`, `L008`).
//!
//! Precommitment at class level ([`AcceptanceScope::Class`]) is how human
//! load falls without accountability leaving — an analyzer assignment is
//! exactly that. Delegation is not.

use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::discharge::DischargeRef;
use crate::finding::Finding;
use crate::hash::VersionHash;
use crate::id::{AcceptanceId, DecisionId};
use crate::identity::Identity;

/// What an acceptance covers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AcceptanceScope {
    /// This one version hash.
    #[default]
    Version,
    /// A precommitment: every decision discharged through this pointer.
    Class(DischargeRef),
}

impl FromStr for AcceptanceScope {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "version" => Ok(Self::Version),
            other => other
                .strip_prefix("class:")
                .ok_or_else(|| {
                    format!("`{other}` is not a scope — expected `version` or `class:<discharge-ref>`")
                })
                .and_then(|r| r.parse::<DischargeRef>())
                .map(Self::Class),
        }
    }
}

impl fmt::Display for AcceptanceScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Version => f.write_str("version"),
            Self::Class(r) => write!(f, "class:{r}"),
        }
    }
}

impl Serialize for AcceptanceScope {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for AcceptanceScope {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// One acceptance, as it appears in a log file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acceptance {
    pub id: AcceptanceId,
    pub decision: DecisionId,
    /// The hash signed. Never the decision id — that is the whole point.
    pub version: VersionHash,
    pub actor: Identity,
    pub at: DateTime<Utc>,
    #[serde(default)]
    pub scope: AcceptanceScope,
    /// Optional pending OD-6 (expiry default). When absent, class `L003`
    /// has nothing to judge and the acceptance never goes stale — which is
    /// exactly the risk OD-6 has to settle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<NaiveDate>,
    /// Reserved for the cryptographic upgrade; format unspecified and empty
    /// in L0. Reserving it now keeps the upgrade additive rather than a
    /// migration (OD-3).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub signature: String,
}

impl Acceptance {
    /// Whether this acceptance has gone stale as of `today`. §4.4 is
    /// explicit that an expired acceptance is a failure, not a warning.
    pub fn is_expired(&self, today: NaiveDate) -> bool {
        self.expires_at.is_some_and(|e| e < today)
    }

    /// Schema obligations this format version places on an acceptance.
    pub fn schema_faults(&self) -> Vec<Finding> {
        let subject = self.id.to_string();
        let mut out = Vec::new();
        if !self.signature.is_empty() {
            out.push(Finding::schema(
                &subject,
                "`signature` is reserved and empty in format 1 — a signed acceptance is a later format",
            ));
        }
        out
    }
}

/// Explicit reversal of a prior acceptance. History is append-only: a
/// mistaken acceptance is revoked, never deleted (§3.4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Revocation {
    pub acceptance: AcceptanceId,
    pub at: DateTime<Utc>,
    pub by: Identity,
    pub reason: String,
}

#[path = "acceptance_tests.rs"]
#[cfg(test)]
mod tests;
