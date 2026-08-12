//! Declaration signing — a seam binding names the exact change it
//! discharges (M8, spec invariant 3, closing `DDD-arch-09`).
//!
//! The signed subject is the *transition*, not the resulting state:
//! subject symbol, before/after content hashes, and the base revision the
//! parent state was committed at. Signing post-change content alone would
//! let one declaration cover a different edit that happens to land on
//! identical content — the session-scoping hole in a new costume. Hashing
//! rides the ledger's canonical-JSON law (`ledger_core::canon`) under this
//! payload's own domain prefix — one law, never a second scheme.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Domain-separation prefix for the binding hash. A change to the hashed
/// field set or its normalisation bumps this to `.v2` with a migration
/// note — the ledger's `CANONICAL_FORM` discipline, applied here.
pub const BINDING_FORM: &str = "ddd.seam-binding.v1";

/// One signed transition on a declared seam.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeamBinding {
    /// The subject symbol the declaration discharges a change to.
    pub symbol: String,
    /// Repo-relative file the transition happens in.
    pub file: String,
    /// Content hash of the parent state (`sha256:…`), or `absent`.
    pub before: String,
    /// Content hash of the proposed state (`sha256:…`), or `absent`.
    pub after: String,
    /// The commit the parent state was verified committed at. A dirty
    /// file refuses to bind — a declaration never pins uncommitted
    /// parent state (M8 ruling 2; `L009`'s construction-limit precedent).
    pub base_revision: String,
    /// The binding hash: [`BINDING_FORM`]-prefixed digest of the
    /// canonical form of the five fields above.
    pub hash: String,
}

impl SeamBinding {
    /// Assemble a binding, computing its hash from the transition fields.
    pub fn seal(
        symbol: &str,
        file: &str,
        before: &str,
        after: &str,
        base_revision: &str,
    ) -> Self {
        let mut binding = Self {
            symbol: symbol.to_string(),
            file: file.to_string(),
            before: before.to_string(),
            after: after.to_string(),
            base_revision: base_revision.to_string(),
            hash: String::new(),
        };
        binding.hash = binding.computed_hash();
        binding
    }

    /// The canonical JSON of the hashed field set — exhaustive
    /// destructure, no rest pattern, so a new field cannot join the wire
    /// form without someone deciding whether it is hashed.
    pub fn canonical_json(&self) -> String {
        let Self { symbol, file, before, after, base_revision, hash: _ } = self;
        let mut m = Map::new();
        for (key, value) in [
            ("symbol", symbol),
            ("file", file),
            ("before", before),
            ("after", after),
            ("base_revision", base_revision),
        ] {
            ledger_core::canon::put(&mut m, key, Some(value.clone()));
        }
        Value::Object(m).to_string()
    }

    /// The binding hash recomputed from content.
    pub fn computed_hash(&self) -> String {
        ledger_core::hash::domain_hash(BINDING_FORM, self.canonical_json().as_bytes())
    }

    /// Does this binding sign exactly the given transition?
    pub fn signs(&self, symbol: &str, file: &str, before: &str, after: &str) -> bool {
        self.symbol == symbol && self.file == file && self.before == before && self.after == after
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_produces_a_stable_domain_separated_hash() {
        let b = SeamBinding::seal("f", "src/a.rs", "sha256:aa", "sha256:bb", "deadbeef");
        assert!(b.hash.starts_with("sha256:"));
        assert_eq!(b.hash, b.computed_hash());
        let again = SeamBinding::seal("f", "src/a.rs", "sha256:aa", "sha256:bb", "deadbeef");
        assert_eq!(b.hash, again.hash);
    }

    #[test]
    fn a_different_transition_hashes_differently() {
        let b = SeamBinding::seal("f", "src/a.rs", "sha256:aa", "sha256:bb", "deadbeef");
        let other_before = SeamBinding::seal("f", "src/a.rs", "sha256:cc", "sha256:bb", "deadbeef");
        let other_base = SeamBinding::seal("f", "src/a.rs", "sha256:aa", "sha256:bb", "cafe");
        assert_ne!(b.hash, other_before.hash);
        assert_ne!(b.hash, other_base.hash);
    }

    #[test]
    fn signs_matches_on_the_content_pair_not_the_hash() {
        let b = SeamBinding::seal("f", "src/a.rs", "sha256:aa", "sha256:bb", "deadbeef");
        assert!(b.signs("f", "src/a.rs", "sha256:aa", "sha256:bb"));
        assert!(!b.signs("f", "src/a.rs", "sha256:bb", "sha256:bb"));
        assert!(!b.signs("g", "src/a.rs", "sha256:aa", "sha256:bb"));
    }
}
