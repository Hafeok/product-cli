//! The decision-version — the content a hash names, an acceptance signs.
//!
//! Two shapes for one thing. [`VersionRaw`] is the flat wire form: exactly
//! the keys that appear in a log file, which is also exactly what
//! [`crate::canon`] canonicalises. [`DecisionVersion`] is the validated
//! domain form, where §4.4's obligations are already discharged by the type.
//! [`DecisionVersion::assemble`] is the one gate between them, so every rule
//! that must hold "at write" and "at parse" holds in a single place.
//!
//! `set` is a version field, deliberately. §4.2.1 names filing a consequential
//! decision into a low-floor set as the residual gaming move; putting `set`
//! inside the hash makes a set move a new version requiring re-acceptance —
//! bound by acceptance rather than merely visible to a reviewer.

use std::fmt;
use std::str::FromStr;

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::allocation::{Allocation, AllocationFields, AllocationKind};
use crate::discharge::{DischargeRef, Stage};
use crate::finding::{Finding, VerifyClass};
use crate::hash::VersionHash;
use crate::id::DecisionId;
use crate::identity::Identity;
use crate::tier::{Tier, Tolerance};

/// A pointer to the basis a version rests on: a PRD section, an upstream
/// decision, a claim id.
///
/// Schema-reserved at L0 — hashed, never resolved. §9.4 walks `based_on`
/// edges through pinned upstreams at L4; the field exists in v1 so that
/// milestone is additive. The vocabulary is deliberately open until then:
/// closing it now would reject adopters' existing reference schemes for no
/// gain, since nothing dereferences a basis pointer yet.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BasisRef(String);

impl BasisRef {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for BasisRef {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let t = s.trim();
        if t.is_empty() {
            return Err("a basis pointer is empty".to_string());
        }
        if t.chars().any(char::is_whitespace) {
            return Err(format!("`{t}` carries whitespace; a basis pointer is a single token"));
        }
        Ok(Self(t.to_string()))
    }
}

impl fmt::Display for BasisRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for BasisRef {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for BasisRef {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        String::deserialize(d)?.parse().map_err(serde::de::Error::custom)
    }
}

/// The wire form of a version, key for key as it appears in a log file.
///
/// Every field here except `hash` is hashed content; `hash` is the stored
/// digest verify recomputes against (class `L007`). Adding a field to this
/// struct breaks [`crate::canon::canonical_bytes`] at compile time, which is
/// the guard: no field can join the wire form without someone deciding
/// whether it is inside the hash.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionRaw {
    pub decision: DecisionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<VersionHash>,
    pub hash: VersionHash,
    pub set: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allocation: Option<AllocationKind>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub discharge: Vec<DischargeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub discharge_stage: Option<Stage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expectation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exposure: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_by: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review_by: Option<NaiveDate>,
    pub tolerance_floor_at_creation: Tier,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance_override: Option<Tier>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub based_on: Vec<BasisRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<DecisionId>,
}

/// A version whose §4.4 obligations already hold.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionVersion {
    pub decision: DecisionId,
    pub parent: Option<VersionHash>,
    pub stored_hash: VersionHash,
    pub set: String,
    pub statement: String,
    /// `None` is the unallocated state verify class `L001` fails on. It is
    /// representable because an enumerated-but-not-yet-allocated decision is
    /// a real intermediate state; it is simply not a shippable one.
    pub allocation: Option<Allocation>,
    pub tolerance: Tolerance,
    pub based_on: Vec<BasisRef>,
    pub supersedes: Option<DecisionId>,
}

impl DecisionVersion {
    /// Validate a wire version into its domain form.
    ///
    /// A below-floor override surfaces as [`VerifyClass::L004`] rather than a
    /// generic schema fault: §4.2.1 requires it rejected at write, and the
    /// same code path must name it when it is met in a hand-edited file.
    pub fn assemble(raw: &VersionRaw) -> Result<Self, Finding> {
        let subject = raw.decision.to_string();
        let tolerance =
            Tolerance::new(raw.tolerance_floor_at_creation, raw.tolerance_override)
                .map_err(|e| Finding::new(VerifyClass::L004, &subject, e.to_string()))?;
        if raw.statement.trim().is_empty() {
            return Err(Finding::schema(&subject, "`statement` is empty"));
        }
        if raw.set.trim().is_empty() {
            return Err(Finding::schema(&subject, "`set` is empty"));
        }
        let allocation = raw
            .allocation
            .map(|kind| Allocation::assemble(kind, raw.allocation_fields()))
            .transpose()
            .map_err(|e| Finding::about(&subject, e))?;
        Ok(Self {
            decision: raw.decision.clone(),
            parent: raw.parent.clone(),
            stored_hash: raw.hash.clone(),
            set: raw.set.trim().to_string(),
            statement: raw.statement.trim().to_string(),
            allocation,
            tolerance,
            based_on: raw.based_on.clone(),
            supersedes: raw.supersedes.clone(),
        })
    }
}

impl VersionRaw {
    fn allocation_fields(&self) -> AllocationFields {
        AllocationFields {
            discharge: self.discharge.clone(),
            discharge_stage: self.discharge_stage,
            actor: self.actor.clone(),
            expectation: self.expectation.clone(),
            exposure: self.exposure.clone(),
            accepted_by: self.accepted_by.clone(),
            review_by: self.review_by,
        }
    }
}

#[path = "version_tests.rs"]
#[cfg(test)]
mod tests;
