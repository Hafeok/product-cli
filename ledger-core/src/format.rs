//! Entry format versions this tool validates against.
//!
//! Every `.decisions/` file declares `format: N`; validation is always
//! against the declared version, never against the newest one. A schema
//! change is a version bump with a migration note in
//! `docs/ledger-format-migrations.md`, so existing entries never break
//! silently. Inherited from the `ddd` store's discipline (`ddd` PRD §6).
//!
//! Note the second, independent version: [`crate::canon::CANONICAL_FORM`]
//! versions the *hash* form. A `format` bump that changes what a hashed
//! field means must bump that one too, because acceptances signed under the
//! old reading would otherwise silently re-point.

/// The format version written by this tool for ordinary acts.
pub const CURRENT_FORMAT: u32 = 1;

/// The format a merge-arbitration change-set declares: format 2 adds the
/// optional `merged_from` field on a version (the other tip a reconciliation
/// closed). A file declares the format it actually needs, so a store that
/// never merged stays readable to a format-1 implementation.
pub const MERGE_FORMAT: u32 = 2;

/// The format a change-set carrying a `contract:` discharge pointer
/// declares (spec v1.4, ddd M8): the repository-diff contract check as a
/// discharge kind. Same declare-what-you-need rule as format 2; hashing
/// is untouched, so `CANONICAL_FORM` stays `v1`.
pub const CONTRACT_FORMAT: u32 = 3;

/// The format a change-set carrying a `revisit_if` edge declares (spec
/// v1.5): the reopen edge, ruled a distinct edge type rather than a basis.
/// Same declare-what-you-need rule as formats 2 and 3; the field is hashed
/// when present and omitted when absent, so `CANONICAL_FORM` stays `v1`.
pub const REVISIT_FORMAT: u32 = 4;

/// Every format version this tool can validate an entry against.
pub const SUPPORTED_FORMATS: &[u32] = &[1, 2, 3, 4];

/// Whether an entry declaring `format: n` can be validated here.
pub fn is_supported(n: u32) -> bool {
    SUPPORTED_FORMATS.contains(&n)
}

/// The format a change-set actually needs — a file declares what it
/// uses, so stores that never merge or never cite the contract check
/// stay readable to older implementations.
pub fn needed_for(cs: &crate::changeset::ChangeSet) -> u32 {
    let mut needed = CURRENT_FORMAT;
    if cs.versions.iter().any(|v| v.merged_from.is_some()) {
        needed = needed.max(MERGE_FORMAT);
    }
    if cs.versions.iter().any(|v| v.discharge.iter().any(|d| d.is_contract())) {
        needed = needed.max(CONTRACT_FORMAT);
    }
    if cs.versions.iter().any(|v| !v.revisit_if.is_empty()) {
        needed = needed.max(REVISIT_FORMAT);
    }
    needed
}

/// The message a file declaring an unknown format fails with.
pub fn unsupported_message(n: u32) -> String {
    let known: Vec<String> = SUPPORTED_FORMATS.iter().map(u32::to_string).collect();
    format!("declares format {n}; this tool validates format(s) {}", known.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_format_is_supported() {
        assert!(is_supported(CURRENT_FORMAT));
    }

    #[test]
    fn an_unknown_format_names_what_is_known() {
        assert!(!is_supported(9));
        assert_eq!(
            unsupported_message(9),
            "declares format 9; this tool validates format(s) 1, 2, 3, 4"
        );
    }

    #[test]
    fn the_merge_format_is_supported() {
        assert!(is_supported(MERGE_FORMAT));
    }

    #[test]
    fn the_contract_format_is_supported() {
        assert!(is_supported(CONTRACT_FORMAT));
    }

    #[test]
    fn the_revisit_format_is_supported() {
        assert!(is_supported(REVISIT_FORMAT));
    }

    #[test]
    fn a_change_set_needs_the_revisit_format_only_when_it_carries_a_reopen_edge() {
        let plain = crate::testkit::version();
        assert_eq!(needed_for(&crate::testkit::changeset(vec![plain.clone()], vec![])), CURRENT_FORMAT);
        let mut reopening = plain;
        reopening.revisit_if = vec!["claim:DDD-x-01@sha256:abc".parse().expect("token")];
        assert_eq!(
            needed_for(&crate::testkit::changeset(vec![reopening], vec![])),
            REVISIT_FORMAT
        );
    }
}
