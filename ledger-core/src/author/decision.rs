//! `ledger add` — minting a decision with its first version.
//!
//! The one verb allowed to leave a decision unallocated: enumerated-but-
//! unallocated is a real intermediate state the format represents on
//! purpose, so an `add` without allocation flags writes, and the `L001` the
//! gate then reports is the gate doing its job on pendency's harder sibling.
//! Every *other* fresh finding refuses the write, through the same
//! delta-gate every verb runs.

use std::collections::BTreeSet;

use crate::allocation::AllocationKind;
use crate::changeset::DecisionRecord;
use crate::discharge::{DischargeRef, Stage};
use crate::finding::VerifyClass;
use crate::hash::version_hash;
use crate::id::DecisionId;
use crate::identity::Identity;
use crate::store::Store;
use crate::tier::Tier;
use crate::version::{BasisRef, VersionRaw};

use super::{Applied, Author, AuthorError};

/// The allocation portion of a version, as the flags spell it.
#[derive(Default)]
pub struct AllocationArgs {
    pub store: Option<AllocationKind>,
    pub discharge: Vec<DischargeRef>,
    pub discharge_stage: Option<Stage>,
    pub expectation: Option<String>,
    pub actor: Option<Identity>,
    pub exposure: Option<String>,
    pub accepted_by: Option<Identity>,
    pub review_by: Option<chrono::NaiveDate>,
}

impl AllocationArgs {
    /// Copy the allocation fields onto a wire version. The hash is not
    /// stamped here; the caller seals after every field is in place.
    pub(crate) fn apply(&self, raw: &mut VersionRaw) {
        raw.allocation = self.store;
        raw.discharge = self.discharge.clone();
        raw.discharge_stage = self.discharge_stage;
        raw.expectation = self.expectation.clone();
        raw.actor = self.actor.clone();
        raw.exposure = self.exposure.clone();
        raw.accepted_by = self.accepted_by.clone();
        raw.review_by = self.review_by;
    }
}

/// What an `add` states.
pub struct AddArgs {
    pub set: String,
    pub statement: String,
    /// The owning scope of the new id. Inferred when the store already
    /// speaks one namespace; required when it speaks several or none.
    pub namespace: Option<String>,
    pub allocation: AllocationArgs,
    pub tolerance_override: Option<Tier>,
    pub based_on: Vec<BasisRef>,
}

impl Author {
    /// Mint a decision and file its first version.
    pub fn add(&mut self, args: AddArgs) -> Result<Applied, AuthorError> {
        let store = self.load();
        let floor = store
            .set(&args.set)
            .ok_or_else(|| {
                AuthorError::Usage(format!(
                    "set `{}` is not declared under sets/ — `ledger declare` it first",
                    args.set
                ))
            })?
            .tolerance_floor;
        let namespace = resolve_namespace(args.namespace.as_deref(), &store)?;
        let decision: DecisionId = format!("dec:{namespace}/{}", self.mint.mint())
            .parse()
            .map_err(AuthorError::Usage)?;
        let raw = first_version(&decision, &args, floor);
        let mut candidate = self.shell(None)?;
        candidate.decisions.push(DecisionRecord {
            id: decision.clone(),
            created_at: self.now,
            created_by: self.who.clone(),
        });
        candidate.versions.push(raw);

        // The sanctioned intermediate state: an add with no allocation may
        // carry exactly its own L001, and nothing else.
        let unallocated = args.allocation.store.is_none();
        let subject = decision.to_string();
        self.refusal_check(&store, &candidate, |f| {
            unallocated && f.class == VerifyClass::L001 && f.subject == subject
        })?;
        let path = self.append(&candidate)?;
        let mut lines = vec![format!("added {decision} to set `{}`", args.set)];
        if unallocated {
            lines.push("unallocated — the readiness gate fails until `ledger allocate` runs".into());
        }
        Ok(Applied { path, lines })
    }
}

/// The first version of a fresh decision, sealed with its own hash.
fn first_version(decision: &DecisionId, args: &AddArgs, floor: Tier) -> VersionRaw {
    let mut raw = VersionRaw {
        decision: decision.clone(),
        parent: None,
        hash: crate::hash::VersionHash::zero(),
        set: args.set.clone(),
        statement: args.statement.clone(),
        allocation: None,
        discharge: Vec::new(),
        discharge_stage: None,
        actor: None,
        expectation: None,
        exposure: None,
        accepted_by: None,
        review_by: None,
        tolerance_floor_at_creation: floor,
        tolerance_override: args.tolerance_override,
        based_on: args.based_on.clone(),
        supersedes: None,
    };
    args.allocation.apply(&mut raw);
    raw.hash = version_hash(&raw);
    raw
}

/// The namespace an `add` mints under: explicit, else the one namespace the
/// store already speaks, else a usage error naming the ambiguity.
fn resolve_namespace(explicit: Option<&str>, store: &Store) -> Result<String, AuthorError> {
    if let Some(ns) = explicit {
        return Ok(ns.to_string());
    }
    let spoken: BTreeSet<&str> = store
        .log
        .iter()
        .flat_map(|c| c.file.decisions.iter())
        .map(|d| d.id.namespace())
        .collect();
    match spoken.len() {
        1 => Ok(spoken.into_iter().collect::<Vec<_>>()[0].to_string()),
        0 => Err(AuthorError::Usage(
            "the store has no decisions yet, so there is no namespace to infer — pass --namespace"
                .to_string(),
        )),
        _ => Err(AuthorError::Usage(format!(
            "the store speaks several namespaces ({}) — pass --namespace",
            spoken.into_iter().collect::<Vec<_>>().join(", ")
        ))),
    }
}
