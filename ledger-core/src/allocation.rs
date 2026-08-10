//! The four stores specification demand is allocated across (§2.1).
//!
//! `constraint` before the act, `criterion` after it, `judgment` during it,
//! `escaped` nowhere. Total demand never shrinks; `escaped` is the only
//! forbidden state, and even that is legitimate when *priced* — exposure
//! stated, acceptor named, review date set. Silent escape is not.
//!
//! Modelled as a sum type rather than a tag plus loose sibling fields, so
//! §4.4's per-store obligations are unrepresentable when violated: a
//! `Criterion` without a stage, a `Judgment` without an actor, or an
//! `Escaped` without its price cannot be built. [`Allocation::assemble`] is
//! the single gate from the flat wire form into that shape.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::discharge::{DischargeRef, Stage};
use crate::finding::{Reason, VerifyClass};
use crate::identity::Identity;

/// The store tag as it appears on the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AllocationKind {
    Constraint,
    Criterion,
    Judgment,
    Escaped,
}

impl AllocationKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Constraint => "constraint",
            Self::Criterion => "criterion",
            Self::Judgment => "judgment",
            Self::Escaped => "escaped",
        }
    }
}

/// A decision-version's allocation, with the fields its store requires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Allocation {
    /// Encoded before the act; an extra-actor refuses the artifact.
    Constraint { discharge: Vec<DischargeRef> },
    /// Checked after the act, at a named stage of the ground table (§5.1).
    Criterion { discharge: Vec<DischargeRef>, stage: Stage, expectation: Option<String> },
    /// Exercised per run by a named accountable actor.
    Judgment { actor: Identity },
    /// Allocated to nobody — legitimate only when priced.
    Escaped { exposure: String, accepted_by: Identity, review_by: NaiveDate },
}

/// The loose fields an allocation may draw on, as read off the wire.
#[derive(Debug, Clone, Default)]
pub struct AllocationFields {
    pub discharge: Vec<DischargeRef>,
    pub discharge_stage: Option<Stage>,
    pub actor: Option<Identity>,
    pub expectation: Option<String>,
    pub exposure: Option<String>,
    pub accepted_by: Option<Identity>,
    pub review_by: Option<NaiveDate>,
}

impl AllocationFields {
    /// Names every field set here that the given store has no use for.
    /// A field an allocation cannot read is not harmless: it reads as
    /// governance that is not there.
    fn strays(&self, kind: AllocationKind) -> Vec<&'static str> {
        let used: &[&str] = match kind {
            AllocationKind::Constraint => &["discharge"],
            AllocationKind::Criterion => &["discharge", "discharge_stage", "expectation"],
            AllocationKind::Judgment => &["actor"],
            AllocationKind::Escaped => &["exposure", "accepted_by", "review_by"],
        };
        let present: [(&'static str, bool); 7] = [
            ("discharge", !self.discharge.is_empty()),
            ("discharge_stage", self.discharge_stage.is_some()),
            ("actor", self.actor.is_some()),
            ("expectation", self.expectation.is_some()),
            ("exposure", self.exposure.is_some()),
            ("accepted_by", self.accepted_by.is_some()),
            ("review_by", self.review_by.is_some()),
        ];
        present
            .into_iter()
            .filter(|(name, set)| *set && !used.contains(name))
            .map(|(name, _)| name)
            .collect()
    }
}

impl Allocation {
    /// Assemble the sum type from the flat wire fields, enforcing §4.4.
    pub fn assemble(kind: AllocationKind, f: AllocationFields) -> Result<Self, Reason> {
        let strays = f.strays(kind);
        if !strays.is_empty() {
            return Err(Reason::schema(format!(
                "allocation `{}` does not read {} — a field its store cannot use reads as governance that is not there",
                kind.as_str(),
                strays.join(", ")
            )));
        }
        match kind {
            AllocationKind::Constraint => Ok(Self::Constraint { discharge: f.discharge }),
            AllocationKind::Criterion => Self::criterion(f),
            AllocationKind::Judgment => f
                .actor
                .map(|actor| Self::Judgment { actor })
                .ok_or_else(|| missing(AllocationKind::Judgment, "actor")),
            AllocationKind::Escaped => Self::escaped(f),
        }
    }

    fn criterion(f: AllocationFields) -> Result<Self, Reason> {
        if f.discharge.is_empty() {
            return Err(missing(AllocationKind::Criterion, "discharge"));
        }
        let stage = f.discharge_stage.ok_or_else(|| missing(AllocationKind::Criterion, "discharge_stage"))?;
        if f.discharge.iter().any(DischargeRef::is_otel) && f.expectation.is_none() {
            return Err(Reason::schema(
                "an `otel:` discharge requires a stated `expectation` — a dashboard read afterward is unfalsifiable and always confirms (§9.1)",
            ));
        }
        Ok(Self::Criterion { discharge: f.discharge, stage, expectation: f.expectation })
    }

    fn escaped(f: AllocationFields) -> Result<Self, Reason> {
        let exposure = f.exposure.ok_or_else(|| missing(AllocationKind::Escaped, "exposure"))?;
        let accepted_by = f.accepted_by.ok_or_else(|| missing(AllocationKind::Escaped, "accepted_by"))?;
        let review_by = f.review_by.ok_or_else(|| missing(AllocationKind::Escaped, "review_by"))?;
        Ok(Self::Escaped { exposure, accepted_by, review_by })
    }

    /// The store tag, for the wire form and for messages.
    pub fn kind(&self) -> AllocationKind {
        match self {
            Self::Constraint { .. } => AllocationKind::Constraint,
            Self::Criterion { .. } => AllocationKind::Criterion,
            Self::Judgment { .. } => AllocationKind::Judgment,
            Self::Escaped { .. } => AllocationKind::Escaped,
        }
    }

    /// The identity pricing this escape, when this is one. Verify class
    /// `L006` runs over exactly this and over an acceptance's actor.
    pub fn escape_acceptor(&self) -> Option<&Identity> {
        match self {
            Self::Escaped { accepted_by, .. } => Some(accepted_by),
            _ => None,
        }
    }

    /// The pointers this allocation discharges through.
    pub fn discharge(&self) -> &[DischargeRef] {
        match self {
            Self::Constraint { discharge } | Self::Criterion { discharge, .. } => discharge,
            Self::Judgment { .. } | Self::Escaped { .. } => &[],
        }
    }
}

/// A store obligation that is not met.
///
/// An incomplete escape is class `L002`, one of the nine — silent escape is
/// the failure the whole tool exists to catch, so it is named rather than
/// folded into the parse gate. The other stores' obligations are schema
/// faults: the file simply does not describe an allocation.
fn missing(kind: AllocationKind, field: &str) -> Reason {
    let class = match kind {
        AllocationKind::Escaped => VerifyClass::L002,
        _ => VerifyClass::Schema,
    };
    Reason::new(class, format!("allocation `{}` requires `{field}` (§4.4)", kind.as_str()))
}

#[path = "allocation_tests.rs"]
#[cfg(test)]
mod tests;
