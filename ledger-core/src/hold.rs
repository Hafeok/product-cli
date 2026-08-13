//! Held — the entries a batched acceptance must refuse to sign.
//!
//! A hold is *derived from the record*, never hand-listed. A worksheet can
//! carry a "do not sign this one yet" note, but a note is not a mechanism:
//! it goes stale the moment an entry is added, and nothing checks that a
//! reader saw it. The ledger already states the fact in the entry itself —
//! a basis pointer in the `indeterminate:` scheme says, in hashed content,
//! that the ground under this decision is not settled.
//!
//! Signing such a version would sign the indeterminacy into the record as
//! though it were settled. So a held member does not merely drop out of a
//! batch: it stops the whole batch by name, because a run that silently
//! skipped it would have made a judgment nobody recorded. The live case is
//! `dec/ddd/interceptor-not-extension`, whose basis is
//! `indeterminate:DDD-arch-03` awaiting a ruling.

use serde::Serialize;

/// The basis-pointer scheme that marks ground as unsettled.
pub const INDETERMINATE: &str = "indeterminate:";

/// Why a decision must not be signed yet, with the pointer that says so.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hold {
    /// The basis pointer carrying the scheme.
    pub pointer: String,
    /// The sentence a refusal prints.
    pub reason: String,
}

/// The hold a version's ground implies, if any.
///
/// Reads `based_on` only. A reopen edge (`revisit_if`) is deliberately not
/// consulted: a tripwire is not ground, and a decision resting on settled
/// ground is signable however many reopen conditions it watches.
pub fn hold<'a>(based_on: impl Iterator<Item = &'a str>) -> Option<Hold> {
    based_on.filter(|p| p.starts_with(INDETERMINATE)).map(of_pointer).next()
}

fn of_pointer(pointer: &str) -> Hold {
    Hold {
        pointer: pointer.to_string(),
        reason: format!(
            "ground is filed as indeterminate ({pointer}) — accepting would sign that indeterminacy in as settled; rule it first"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_indeterminate_basis_holds_the_entry() {
        let held = hold(["indeterminate:DDD-arch-03@sha256:aa".into()].iter().copied())
            .expect("held");
        assert_eq!(held.pointer, "indeterminate:DDD-arch-03@sha256:aa");
        assert!(held.reason.contains("sign that indeterminacy in as settled"), "{held:?}");
    }

    #[test]
    fn settled_ground_is_not_held() {
        let pointers = ["mandate:emil-2026-08-11#x", "ddd-content:seam/web/y"];
        assert!(hold(pointers.iter().copied()).is_none());
    }

    #[test]
    fn a_pointer_merely_mentioning_the_word_is_not_the_scheme() {
        let pointers = ["claim:indeterminate-results-are-common"];
        assert!(hold(pointers.iter().copied()).is_none());
    }

    #[test]
    fn the_first_indeterminate_pointer_is_the_one_named() {
        let pointers = ["prd:x", "indeterminate:A", "indeterminate:B"];
        let held = hold(pointers.iter().copied()).expect("held");
        assert_eq!(held.pointer, "indeterminate:A");
    }
}
