//! Golden canonicalisation cases — determinism, invariance, sensitivity.

use crate::allocation::AllocationKind;
use crate::discharge::Stage;
use crate::hash::version_hash;
use crate::testkit;
use crate::tier::Tier;
use crate::version::VersionRaw;

use super::*;

/// The pinned conformance vector. This constant is published in
/// `docs/ledger-format-v1.md`: an outside implementation of the format hashes
/// the same fixture and compares. If this test fails, either the canonical
/// form changed — which is a `CANONICAL_FORM` bump and a migration note, not
/// a fix — or a platform is disagreeing, which is the whole reason the vector
/// is pinned.
const VECTOR: &str = "sha256:ac2a68023018391550b542b1f093104f1f32115d3603e2fda9c37805875437cc";

/// The canonical text the vector is taken over, published alongside it so an
/// outside implementation can tell *which* step it disagrees on rather than
/// only that it does.
const VECTOR_JSON: &str = concat!(
    r#"{"allocation":"constraint","#,
    r#""based_on":["prd:decision-ledger-prd#4.2.1"],"#,
    r#""decision":"dec:hafeok.ledger/01K2C4YQJ3F8M0PT5W7NZ9RDXV","#,
    r#""discharge":["analyzer:DEC001-no-float-money"],"#,
    r#""set":"ledger-design","#,
    r#""statement":"Monetary amounts use decimal, never double.","#,
    r#""tolerance_floor_at_creation":"T1"}"#,
);

/// An edit to a version, applied by the table-driven cases below.
type Edit = Box<dyn Fn(&mut VersionRaw)>;
/// One field with an edit that changes it.
type Mutation = (&'static str, Edit);
/// Two edits that must produce the same hash.
type Equivalence = (&'static str, Edit, Edit);

fn base() -> VersionRaw {
    testkit::version()
}

fn edit(f: impl Fn(&mut VersionRaw) + 'static) -> Edit {
    Box::new(f)
}

#[test]
fn the_pinned_conformance_vector_holds() {
    assert_eq!(version_hash(&base()).to_string(), VECTOR);
}

#[test]
fn the_canonical_form_is_compact_key_sorted_json() {
    let json = canonical_json(&base());
    assert_eq!(json, VECTOR_JSON);
    assert!(!json.contains('\n') && !json.contains('\t'), "no structural whitespace");
    assert!(!json.contains("\": ") && !json.contains(", \""), "no separator padding");
    assert!(!json.contains("null"), "absent optionals are omitted: {json}");
}

#[test]
fn hashing_is_deterministic_across_repeated_runs() {
    let a = version_hash(&base());
    for _ in 0..8 {
        assert_eq!(version_hash(&base()), a);
    }
}

#[test]
fn the_stored_hash_is_outside_its_own_content() {
    // Otherwise sealing a version would change what it hashes to.
    let mut v = base();
    let before = version_hash(&v);
    v.hash = before.clone();
    assert_eq!(version_hash(&v), before);
}

#[test]
fn the_domain_prefix_separates_this_form_from_any_other() {
    use sha2::{Digest, Sha256};
    let bare = format!("sha256:{:x}", Sha256::digest(canonical_bytes(&base())));
    assert_ne!(bare, version_hash(&base()).to_string(), "the prefix is load-bearing");
    assert_eq!(CANONICAL_FORM, "ledger.decision-version.v1");
}

// ---------------------------------------------------------------------------
// Invariance: formatting-only differences leave the hash alone.
// ---------------------------------------------------------------------------

/// How the same text can be written differently.
fn text_equivalences() -> Vec<Equivalence> {
    vec![
        (
            "CRLF against LF",
            edit(|v| v.statement = "one\r\ntwo".into()),
            edit(|v| v.statement = "one\ntwo".into()),
        ),
        (
            "a bare CR against LF",
            edit(|v| v.statement = "one\rtwo".into()),
            edit(|v| v.statement = "one\ntwo".into()),
        ),
        (
            "leading and trailing whitespace",
            edit(|v| v.statement = "  \tdecimal, never double.\n ".into()),
            edit(|v| v.statement = "decimal, never double.".into()),
        ),
        (
            "a padded set name",
            edit(|v| v.set = " ledger-design ".into()),
            edit(|v| v.set = "ledger-design".into()),
        ),
        (
            "NFD against NFC",
            edit(|v| v.statement = "Bele\u{0301}b bruger decimal".into()),
            edit(|v| v.statement = "Bel\u{00E9}b bruger decimal".into()),
        ),
    ]
}

/// How the same structure can be written differently.
fn structure_equivalences() -> Vec<Equivalence> {
    fn refs(items: &[&str]) -> Vec<crate::discharge::DischargeRef> {
        items.iter().map(|s| s.parse().expect("ref")).collect()
    }
    fn basis(items: &[&str]) -> Vec<crate::version::BasisRef> {
        items.iter().map(|s| s.parse().expect("basis")).collect()
    }
    vec![
        (
            "a reordered discharge list",
            edit(|v| v.discharge = refs(&["test:Z", "analyzer:A"])),
            edit(|v| v.discharge = refs(&["analyzer:A", "test:Z"])),
        ),
        (
            "a duplicated discharge entry",
            edit(|v| v.discharge = refs(&["analyzer:A", "analyzer:A"])),
            edit(|v| v.discharge = refs(&["analyzer:A"])),
        ),
        (
            "a reordered basis list",
            edit(|v| v.based_on = basis(&["prd:b", "prd:a"])),
            edit(|v| v.based_on = basis(&["prd:a", "prd:b"])),
        ),
        (
            "a whitespace-only optional against absent",
            edit(|v| v.expectation = Some("   ".into())),
            edit(|v| v.expectation = None),
        ),
    ]
}

/// The `null`-versus-absent and `[]`-versus-absent distinctions vanish in
/// the parse, before anything is hashed. Tested at the YAML level because
/// that is the only place the two spellings exist.
#[test]
fn absent_null_and_empty_spellings_hash_alike() {
    let sparse = serde_yaml::to_string(&base()).expect("serialize");
    let padded = format!("{sparse}supersedes: null\nreview_by: null\n")
        .replace("based_on:", "discharge_stage: null\nbased_on:");
    let a: VersionRaw = serde_yaml::from_str(&sparse).expect("parse");
    let b: VersionRaw = serde_yaml::from_str(&padded).expect("parse");
    assert_eq!(version_hash(&a), version_hash(&b), "explicit nulls are absent");
}

#[test]
fn formatting_only_changes_leave_the_hash_unchanged() {
    let cases = text_equivalences().into_iter().chain(structure_equivalences());
    for (name, left, right) in cases {
        let (mut a, mut b) = (base(), base());
        left(&mut a);
        right(&mut b);
        assert_eq!(version_hash(&a), version_hash(&b), "{name} must not move the hash");
    }
}

// ---------------------------------------------------------------------------
// Sensitivity: every hashed field, one at a time.
// ---------------------------------------------------------------------------

/// What a version *is* — identity, basis, position in the chain.
fn identity_mutations() -> Vec<Mutation> {
    vec![
        ("decision", edit(|v| {
            v.decision = "dec:hafeok.other/01K2C4YQJ3F8M0PT5W7NZ9RDXV".parse().expect("id");
        })),
        ("parent", edit(|v| v.parent = Some(testkit::zero_hash()))),
        ("set", edit(|v| v.set = "other-set".into())),
        ("statement", edit(|v| v.statement = "Something else.".into())),
        ("based_on", edit(|v| {
            v.based_on = vec!["prd:decision-ledger-prd#9.4".parse().expect("basis")];
        })),
        ("revisit_if", edit(|v| {
            v.revisit_if = vec!["claim:DDD-adapter-02@sha256:b333063d".parse().expect("token")];
        })),
        ("supersedes", edit(|v| v.supersedes = Some(testkit::decision_id()))),
        ("tolerance_floor_at_creation", edit(|v| v.tolerance_floor_at_creation = Tier::T0)),
        ("tolerance_override", edit(|v| v.tolerance_override = Some(Tier::T2))),
    ]
}

/// What a version is *allocated to* — the store and its obligations.
fn disposition_mutations() -> Vec<Mutation> {
    vec![
        ("allocation", edit(|v| v.allocation = Some(AllocationKind::Judgment))),
        ("discharge", edit(|v| {
            v.discharge = vec!["analyzer:OTHER".parse().expect("ref")];
        })),
        ("discharge_stage", edit(|v| v.discharge_stage = Some(Stage::Prod))),
        ("actor", edit(|v| v.actor = Some(testkit::identity("fixture-human@example")))),
        ("expectation", edit(|v| v.expectation = Some("under 1%".into()))),
        ("exposure", edit(|v| v.exposure = Some("may break".into()))),
        ("accepted_by", edit(|v| {
            v.accepted_by = Some(testkit::identity("fixture-human@example"));
        })),
        ("review_by", edit(|v| v.review_by = Some(testkit::date("2026-09-01")))),
    ]
}

fn all_mutations() -> Vec<Mutation> {
    identity_mutations().into_iter().chain(disposition_mutations()).collect()
}

#[test]
fn every_hashed_field_moves_the_hash() {
    let base_hash = version_hash(&base());
    for (field, mutate) in all_mutations() {
        let mut v = base();
        mutate(&mut v);
        assert_ne!(version_hash(&v), base_hash, "changing `{field}` must move the hash");
    }
}

/// The mutation table must cover the canonical form exactly. A field that
/// quietly stopped being canonicalised, or one added to the wire form and
/// never exercised, fails here rather than passing in silence.
#[test]
fn the_mutation_table_covers_the_whole_canonical_form() {
    let mutations = all_mutations();
    let mut every = base();
    for (_, mutate) in &mutations {
        mutate(&mut every);
    }
    let json = canonical_json(&every);
    for (field, _) in &mutations {
        assert!(json.contains(&format!("\"{field}\":")), "`{field}` is not canonicalised");
    }
    assert_eq!(json.matches("\":").count(), mutations.len(), "no field is emitted unexercised");
}

/// The same token as ground and as a reopen edge are different content.
/// This is the ruling made mechanical: `revisit_if` is not a basis, so the
/// two lists canonicalise to separate keys and an acceptance always names
/// which of them it signed.
#[test]
fn one_token_hashes_differently_as_ground_and_as_a_reopen_edge() {
    let token = "claim:DDD-adapter-02@sha256:b333063d";
    let mut ground = base();
    ground.based_on = vec![token.parse().expect("basis")];
    ground.revisit_if = Vec::new();
    let mut reopen = base();
    reopen.based_on = Vec::new();
    reopen.revisit_if = vec![token.parse().expect("token")];
    assert_ne!(version_hash(&ground), version_hash(&reopen));
    assert!(canonical_json(&reopen).contains("\"revisit_if\":"));
    assert!(!canonical_json(&reopen).contains("\"based_on\":"));
}

/// A version that states no reopen edge canonicalises exactly as it did
/// before the field existed — which is why format 4 moves no digest and
/// invalidates no acceptance.
#[test]
fn an_absent_reopen_list_is_omitted_from_the_canonical_form() {
    let mut v = base();
    v.revisit_if = Vec::new();
    assert!(!canonical_json(&v).contains("revisit_if"));
    assert_eq!(version_hash(&v), version_hash(&base()));
}

#[test]
fn the_tolerance_inputs_are_hashed_rather_than_the_resolved_tier() {
    // `T0 floor + T2 override` and `T2 floor` both resolve to an effective
    // T2, but they are different provenance and must not collide — that is
    // what keeps override-rate-per-set (§10) computable from hashed content.
    let mut overridden = base();
    overridden.tolerance_floor_at_creation = Tier::T0;
    overridden.tolerance_override = Some(Tier::T2);
    let mut native = base();
    native.tolerance_floor_at_creation = Tier::T2;
    assert_ne!(version_hash(&overridden), version_hash(&native));
}
