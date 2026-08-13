//! Appending the next version of an existing decision.
//!
//! `allocate`, `escape`, `revise` and `supersede` are one pipeline: take the
//! latest version, link it as `parent`, re-pin the set's *current* floor
//! (that re-pin is how a stranded member gets back above a raised floor),
//! apply the change, seal the hash, and run the delta-gate. The hash moving
//! is what invalidates prior acceptances — no verb here touches an existing
//! file, ever.
//!
//! The single-writer stance L3 inherits: a revision against an explicitly
//! stated parent that is no longer the latest is a **refusal**, not a merge.
//! Whatever merge semantics arrive at L3, they arrive there; L1 refuses.

use crate::hash::{version_hash, VersionHash};
use crate::id::DecisionId;
use crate::revisit::RevisitRef;
use crate::verify::view::View;
use crate::version::{BasisRef, VersionRaw};

use super::decision::AllocationArgs;
use super::{Applied, Author, AuthorError};

/// What an escape prices.
pub struct EscapeArgs {
    pub exposure: String,
    pub review_by: chrono::NaiveDate,
}

/// What a revision restates.
pub struct ReviseArgs {
    pub statement: String,
    pub based_on: Vec<BasisRef>,
    /// The reopen edges this revision states. Stated separately from
    /// `based_on` all the way down, so re-typing an edge from ground to
    /// watched is one act the reader can see, not a silent reclassification.
    /// `None` leaves the inherited edges alone; `Some(vec![])` clears them —
    /// a decision that has been reopened and re-decided drops the edge that
    /// reopened it, and that has to be sayable.
    pub revisit_if: Option<Vec<RevisitRef>>,
    /// What this act was, on the change-set. A revision that re-types an
    /// edge or restates a decision has a reason, and the reason belongs
    /// where the filing is read — `supersede` already carries one.
    pub note: Option<String>,
    /// The version the author believes is the tip. When stated and stale,
    /// the verb refuses — merge is L3's problem, not a quiet overwrite.
    pub expected_parent: Option<VersionHash>,
}

/// What a supersession records.
pub struct SupersedeArgs {
    pub superseded: DecisionId,
    pub by: DecisionId,
    pub reason: Option<String>,
}

impl Author {
    /// Allocate (or re-allocate) a decision by filing its next version.
    pub fn allocate(&mut self, id: &DecisionId, alloc: AllocationArgs) -> Result<Applied, AuthorError> {
        if alloc.store.is_none() {
            return Err(AuthorError::Usage(
                "allocate needs a store — one of constraint | criterion | judgment (escapes go through `ledger escape`)"
                    .to_string(),
            ));
        }
        let store_name = alloc.store.map(|k| k.as_str()).unwrap_or_default();
        self.next_version(id, None, |raw| alloc.apply(raw))
            .map(|a| a.saying(format!("allocated {id} to `{store_name}`")))
    }

    /// Price an escape: exposure stated, acceptor named, review date set.
    /// The acceptor is the author — identity from git config, per OD-3 —
    /// and a model or CI identity is refused by the same rule `verify`
    /// applies (`L006`).
    pub fn escape(&mut self, id: &DecisionId, args: EscapeArgs) -> Result<Applied, AuthorError> {
        let who = self.who.clone();
        self.next_version(id, None, |raw| {
            let alloc = AllocationArgs {
                store: Some(crate::allocation::AllocationKind::Escaped),
                exposure: Some(args.exposure.clone()),
                accepted_by: Some(who.clone()),
                review_by: Some(args.review_by),
                ..AllocationArgs::default()
            };
            alloc.apply(raw);
        })
        .map(|a| a.saying(format!("priced the escape of {id} — review by {}", args.review_by)))
    }

    /// Restate a decision. The allocation carries over unchanged; the hash
    /// moves, so every acceptance of the prior version becomes history.
    pub fn revise(&mut self, id: &DecisionId, args: ReviseArgs) -> Result<Applied, AuthorError> {
        self.next_version_noted(id, args.expected_parent.as_ref(), args.note.clone(), |raw| {
            raw.statement = args.statement.clone();
            if !args.based_on.is_empty() {
                raw.based_on = args.based_on.clone();
            }
            if let Some(edges) = &args.revisit_if {
                raw.revisit_if = edges.clone();
            }
        })
        .map(|a| a.saying(format!("revised {id} — prior acceptances now sign history")))
    }

    /// Record that `by` supersedes `superseded`, as the next version of the
    /// successor. The superseded id is never touched: supersession is an
    /// edge on the successor, and the old decision's state is derived.
    pub fn supersede(&mut self, args: SupersedeArgs) -> Result<Applied, AuthorError> {
        if args.superseded == args.by {
            return Err(AuthorError::Usage("a decision cannot supersede itself".to_string()));
        }
        let store = self.load();
        let view = View::build(&store);
        if !view.latest.contains_key(&args.superseded.to_string()) {
            return Err(AuthorError::Usage(format!(
                "{} has no filed version — a supersession target must exist",
                args.superseded
            )));
        }
        if let Some((_, successor)) = crate::verify::state::supersession(&view)
            .into_iter()
            .find(|(old, _)| *old == args.superseded.to_string())
        {
            return Err(AuthorError::Conflict(format!(
                "{} is already superseded by {successor} — the chain continues from its tip",
                args.superseded
            )));
        }
        let old = args.superseded.clone();
        self.next_version_noted(&args.by, None, args.reason.clone(), |raw| {
            raw.supersedes = Some(old.clone());
        })
        .map(|a| a.saying(format!("{} now supersedes {}", args.by, args.superseded)))
    }

    fn next_version(
        &mut self,
        id: &DecisionId,
        expected_parent: Option<&VersionHash>,
        mutate: impl FnOnce(&mut VersionRaw),
    ) -> Result<Applied, AuthorError> {
        self.next_version_noted(id, expected_parent, None, mutate)
    }

    /// The shared pipeline: latest → parent link → floor re-pin → change →
    /// seal → delta-gate → append.
    fn next_version_noted(
        &mut self,
        id: &DecisionId,
        expected_parent: Option<&VersionHash>,
        note: Option<String>,
        mutate: impl FnOnce(&mut VersionRaw),
    ) -> Result<Applied, AuthorError> {
        let store = self.load();
        let view = View::build(&store);
        if view.is_forked(&id.to_string()) {
            return Err(AuthorError::Conflict(format!(
                "the version chain of {id} is forked — two writers diverged, and extending either tip would bury the conflict; `ledger merge --resolve` arbitrates first"
            )));
        }
        let latest = view
            .latest
            .get(&id.to_string())
            .and_then(|i| view.versions.get(*i))
            .ok_or_else(|| {
                AuthorError::Usage(format!("{id} has no filed version — `ledger add` creates one"))
            })?;
        if let Some(expected) = expected_parent {
            if *expected != latest.raw.hash {
                return Err(AuthorError::Conflict(format!(
                    "{} is not the latest version of {id} (the tip is {}) — L1 refuses to fork a decision; merge is L3",
                    expected.short(),
                    latest.raw.hash.short()
                )));
            }
        }
        let mut raw = latest.raw.clone();
        raw.parent = Some(latest.raw.hash.clone());
        // Re-pin the floor as it stands now; drop an override the new floor
        // has caught up with, so the version stays representable (§4.2.1).
        if let Some(set) = store.set(&raw.set) {
            raw.tolerance_floor_at_creation = set.tolerance_floor;
            if raw.tolerance_override.is_some_and(|o| o <= set.tolerance_floor) {
                raw.tolerance_override = None;
            }
        }
        mutate(&mut raw);
        raw.hash = version_hash(&raw);
        let mut candidate = self.shell(note)?;
        candidate.versions.push(raw);
        self.refusal_check(&store, &candidate, |_| false)?;
        let path = self.append(&candidate)?;
        Ok(Applied { path, lines: Vec::new() })
    }
}

impl Applied {
    fn saying(mut self, line: String) -> Self {
        self.lines.insert(0, line);
        self
    }
}
