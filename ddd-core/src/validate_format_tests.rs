//! Format-gate cases: each version-scoped field rejected below its format.
//!
//! Split from `validate_tests.rs` to keep both files inside the 400-line
//! fitness limit. Helpers come from the parent test module.

use super::*;
use crate::claim::{ClaimStatus, VersionIndex};
use crate::decision::{BasedOn, BasisPin};

#[test]
fn declared_formats_beyond_the_known_set_are_named() {
    let mut s = DddStore::default();
    let mut p = pred("pred/a/x");
    p.format = 5;
    s.predicates.push(p);
    assert_names(&validate_store(&s), "pred/a/x", "declares format 5");
}

#[test]
fn revalidate_by_under_format_one_is_a_violation() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1");
    c.revalidate_by = Some("2027-01-01".into());
    s.claims.push(c);
    assert_names(&validate_store(&s), "DDD-a-1", "format-2 field");

    let mut ok = DddStore::default();
    let mut c2 = claim("DDD-a-2");
    c2.format = 2;
    c2.revalidate_by = Some("2027-01-01".into());
    ok.claims.push(c2);
    assert!(validate_store(&ok).is_empty(), "{}", msgs(&validate_store(&ok)));
}

#[test]
fn version_index_under_format_three_is_a_violation() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1");
    c.format = 3;
    c.version_index = Some(VersionIndex {
        sdk: Some("8.0.413".into()),
        ..Default::default()
    });
    s.claims.push(c);
    assert_names(&validate_store(&s), "DDD-a-1", "format-4 field");

    let mut ok = DddStore::default();
    let mut c2 = claim("DDD-a-2");
    c2.format = 4;
    c2.version_index = Some(VersionIndex {
        sdk: Some("8.0.413".into()),
        ..Default::default()
    });
    ok.claims.push(c2);
    assert!(validate_store(&ok).is_empty(), "{}", msgs(&validate_store(&ok)));
}

#[test]
fn a_version_index_anchoring_nothing_is_a_violation() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1");
    c.format = 4;
    c.version_index = Some(VersionIndex::default());
    s.claims.push(c);
    assert_names(&validate_store(&s), "DDD-a-1", "anchor at least one");
}

#[test]
fn format_four_is_a_known_format() {
    let mut s = DddStore::default();
    let mut c = claim("DDD-a-1");
    c.format = 4;
    s.claims.push(c);
    assert!(validate_store(&s).is_empty(), "{}", msgs(&validate_store(&s)));

    let mut ahead = DddStore::default();
    let mut c2 = claim("DDD-a-2");
    c2.format = 5;
    ahead.claims.push(c2);
    assert_names(&validate_store(&ahead), "DDD-a-2", "formats 1-4");
}

#[test]
fn pinned_edges_under_format_one_are_a_violation() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1"));
    let mut d = decision("dec/a/adopt", &[]);
    d.based_on = vec![BasedOn::Pinned(BasisPin {
        claim: "DDD-a-1".into(),
        status: ClaimStatus::Projected,
        changed: "2026-08-01".into(),
    })];
    s.decisions.push(d);
    assert_names(&validate_store(&s), "dec/a/adopt", "format 2");
}

#[test]
fn a_format_two_decision_must_pin_every_edge() {
    let mut s = DddStore::default();
    s.claims.push(claim("DDD-a-1"));
    let mut d = decision("dec/a/adopt", &["DDD-a-1"]);
    d.format = 2;
    s.decisions.push(d);
    assert_names(&validate_store(&s), "dec/a/adopt", "pins every basedOn edge");

    let mut ok = DddStore::default();
    ok.claims.push(claim("DDD-a-1"));
    let mut d2 = decision("dec/a/adopt", &[]);
    d2.format = 2;
    d2.based_on = vec![BasedOn::Pinned(BasisPin {
        claim: "DDD-a-1".into(),
        status: ClaimStatus::Projected,
        changed: "2026-08-01".into(),
    })];
    ok.decisions.push(d2);
    assert!(validate_store(&ok).is_empty(), "{}", msgs(&validate_store(&ok)));
}

#[test]
fn diff_or_detect_sections_under_format_one_are_a_violation() {
    let s = DddStore {
        config: Some(crate::config::DddConfig {
            diff: Some(Default::default()),
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_names(&validate_store(&s), "config.yaml", "format 2");
}
