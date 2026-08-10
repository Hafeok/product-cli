//! Rules over whether the record is what it claims to be.
//!
//! Three classes: `L007` a stored hash that does not match its content,
//! `L008` an acceptance signing a hash nobody filed, `L009` an acceptance
//! whose actor is not the author of the commit that introduced it. Together
//! they are what stands in for a signature at L0 — the record is
//! append-only, content-addressed, attributed by git, and each of those is
//! checkable without any key material.

use crate::blame::{introducing_author, Attribution};
use crate::finding::{Finding, VerifyClass};
use crate::hash::version_hash;
use crate::store::Store;

use super::view::View;

/// `L007` — the append-only tamper check.
///
/// Recomputes each version's canonical hash and compares it to the digest
/// stored beside it. An edit to an accepted version moves the hash; a
/// hand-edit that leaves the stored digest behind is exactly what this
/// catches, and it is why acceptance signs a hash rather than an id.
pub fn hash_mismatch(view: &View) -> Vec<Finding> {
    view.versions
        .iter()
        .filter_map(|v| {
            let computed = version_hash(v.raw);
            (computed != v.raw.hash).then(|| {
                Finding::new(
                    VerifyClass::L007,
                    &v.raw.decision.to_string(),
                    // The recomputed digest is given in full, not shortened.
                    // L0 mints no ids and seals no versions — `ledger add`
                    // is L1 — so hand-authoring a log file is the only way
                    // in, and this line is where the author gets the hash to
                    // write. Shortening it would make the format
                    // unusable by the audience the spec is written for.
                    format!(
                        "stored hash {} does not match content, which hashes to {computed}",
                        v.raw.hash.short()
                    ),
                )
            })
        })
        .collect()
}

/// `L008` — an acceptance signing a hash that matches no stored version.
///
/// The pair is checked, not the hash alone: an acceptance that names one
/// decision while signing another's hash is signing nothing about the
/// decision it claims to accept.
pub fn dangling_acceptance(view: &View) -> Vec<Finding> {
    view.acceptances
        .iter()
        .map(|a| a.acceptance)
        .filter(|a| !view.stored.contains(&(a.decision.to_string(), a.version.to_string())))
        .map(|a| {
            Finding::new(
                VerifyClass::L008,
                &a.id.to_string(),
                format!("signs {} of {}, which is not a filed version", a.version.short(), a.decision),
            )
        })
        .collect()
}

/// The outcome of the blame pass, so the caller can report what it skipped
/// rather than let a silently-unrun rule read as a passing one.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct BlameOutcome {
    pub findings: Vec<Finding>,
    /// Acceptances with no introducing commit yet.
    pub uncommitted: usize,
    /// Whether git could be consulted at all.
    pub no_repository: bool,
}

/// `L009` — blame consistency (OD-3's second mechanical corroboration).
///
/// An acceptance not yet committed has no introducing commit and is skipped,
/// not failed: it is the state every acceptance passes through on its way
/// into history. The check therefore lands on the next run over committed
/// history, which in practice is CI.
pub fn blame_consistency(view: &View, store: &Store) -> BlameOutcome {
    let mut out = BlameOutcome::default();
    for viewed in &view.acceptances {
        let a = viewed.acceptance;
        match introducing_author(&store.root, viewed.path, &a.id.to_string()) {
            Attribution::Author(email) if email != a.actor.as_str() => {
                out.findings.push(Finding::new(
                    VerifyClass::L009,
                    &a.id.to_string(),
                    format!(
                        "names {} but was introduced by commit author {email} — an acceptance is signed by the actor who deposits it",
                        a.actor
                    ),
                ));
            }
            Attribution::Author(_) => {}
            Attribution::NotCommitted => out.uncommitted += 1,
            Attribution::NoRepository => {
                out.no_repository = true;
                out.uncommitted += 1;
            }
        }
    }
    out
}
