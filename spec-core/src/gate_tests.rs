use super::*;
use crate::act::tests::act;
use crate::inventory::tests::sample;
use crate::record::tests::opening;

const SETTLE: &str = "Shop.Api.BasketController.Settle#HttpPost";
const SETTLE_CANDIDATE: &str = "cand/shop-api-basketcontroller-settle-httppost";

/// An unsigned refusal, for the signing tests next door.
pub(crate) fn unsigned_rejection() -> Rejection {
    rejection(SETTLE_CANDIDATE)
}

fn rejection(candidate: &str) -> Rejection {
    let mut refused = Rejection {
        form: crate::act::REJECTION_FORM.into(),
        candidate: candidate.into(),
        reason: "A health probe is not an act.".into(),
        principal: "emil@example.com".parse().expect("identity parses"),
        at: chrono::Utc::now(),
        binds: String::new(),
        signature: None,
    };
    refused.binds = refused.computed_binds();
    refused
}

fn store_with(acts: Vec<Act>, rejections: Vec<Rejection>) -> SpecStore {
    SpecStore {
        records: Vec::new(),
        acts,
        rejections,
        inventory: Some(sample()),
        ..SpecStore::default()
    }
}

fn classes(findings: &[Finding]) -> Vec<Class> {
    findings.iter().map(|f| f.class).collect()
}

#[test]
fn s005_an_entry_point_no_act_covers_is_drift() {
    let store = store_with(Vec::new(), Vec::new());
    assert_eq!(classes(&judge_drift(&store)), vec![Class::S005]);
}

#[test]
fn a_ratified_act_covers_its_entry_point() {
    let store = store_with(vec![act("act/settle-basket", &[SETTLE])], Vec::new());
    assert!(judge_drift(&store).is_empty());
}

#[test]
fn a_refused_candidate_covers_its_entry_point_too() {
    let store = store_with(Vec::new(), vec![rejection(SETTLE_CANDIDATE)]);
    assert!(
        judge_drift(&store).is_empty(),
        "a principal looked at it and said it is not an act — that is a decision"
    );
}

#[test]
fn s002_a_machine_cannot_ratify() {
    let mut machine = act("act/a", &[SETTLE]);
    machine.ratified_by = "ci@example.com".parse().expect("identity parses");
    machine.binds = machine.computed_binds();
    assert!(classes(&judge_act(&machine)).contains(&Class::S002));
}

#[test]
fn s002_a_machine_cannot_refuse_either() {
    let mut machine = rejection(SETTLE_CANDIDATE);
    machine.principal = "github-actions@example.com".parse().expect("identity parses");
    machine.binds = machine.computed_binds();
    assert!(classes(&judge_rejection(&machine)).contains(&Class::S002));
}

#[test]
fn s007_an_edited_act_breaks_its_binding() {
    let mut edited = act("act/a", &[SETTLE]);
    edited.settles = "Quietly rewritten after ratification.".into();
    assert!(classes(&judge_act(&edited)).contains(&Class::S007));
}

#[test]
fn s006_a_record_naming_an_unratified_act_is_a_dangling_reference() {
    let mut store = store_with(vec![act("act/settle-basket", &[SETTLE])], Vec::new());
    let mut record_opening = opening("checkout-totals");
    record_opening.act_ref = "act/does-not-exist".into();
    store.records = vec![ActRecord::open(record_opening)];
    assert_eq!(classes(&judge_references(&store)), vec![Class::S006]);
}

#[test]
fn a_record_naming_a_ratified_act_resolves() {
    let mut store = store_with(vec![act("act/settle-basket", &[SETTLE])], Vec::new());
    let mut record_opening = opening("checkout-totals");
    record_opening.act_ref = "act/settle-basket".into();
    store.records = vec![ActRecord::open(record_opening)];
    assert!(judge_references(&store).is_empty());
}

#[test]
fn references_are_not_judged_before_the_first_act_is_ratified() {
    let mut store = store_with(Vec::new(), Vec::new());
    store.records = vec![ActRecord::open(opening("checkout-totals"))];
    assert!(
        judge_references(&store).is_empty(),
        "a repo mid-adoption is not a broken one"
    );
}

#[test]
fn a_candidate_stays_unreviewed_until_someone_reviews_it() {
    let mut store = store_with(Vec::new(), Vec::new());
    assert!(store.is_unreviewed(SETTLE_CANDIDATE));

    store.rejections = vec![rejection(SETTLE_CANDIDATE)];
    assert!(!store.is_unreviewed(SETTLE_CANDIDATE));
}

#[test]
fn ratifying_a_candidate_marks_it_reviewed() {
    let mut ratified = act("act/settle-basket", &[SETTLE]);
    ratified.from_candidate = Some(SETTLE_CANDIDATE.into());
    ratified.binds = ratified.computed_binds();
    let store = store_with(vec![ratified], Vec::new());
    assert!(!store.is_unreviewed(SETTLE_CANDIDATE));
}

fn with_closure(slice: &str, determinations: &[&str]) -> ActRecord {
    let mut record_opening = opening(slice);
    record_opening.act_ref = "act/settle-basket".into();
    let mut record = ActRecord::open(record_opening);
    let mut closure = crate::Closure {
        kind: if determinations.is_empty() {
            crate::ClosureKind::NothingArose
        } else {
            crate::ClosureKind::Determinations
        },
        principal: "emil@example.com".parse().expect("identity parses"),
        at: chrono::Utc::now(),
        determinations: determinations.iter().map(|d| (*d).to_string()).collect(),
        binds: String::new(),
        signature: None,
    };
    closure.binds = crate::digest::closure_digest(&record.computed_binds(), &closure);
    record.closure = Some(closure);
    record
}

#[test]
fn s012_a_slice_attribute_naming_nothing_declared_is_an_orphan() {
    let store = store_with(Vec::new(), Vec::new());
    let classes = classes(&judge_claims(&store));
    assert!(classes.contains(&Class::S012), "{classes:?}");
}

#[test]
fn a_slice_attribute_resolves_once_a_record_declares_that_slice() {
    let mut store = store_with(Vec::new(), Vec::new());
    store.records = vec![ActRecord::open(opening("checkout-totals"))];
    assert!(!classes(&judge_claims(&store)).contains(&Class::S012));
}

#[test]
fn s013_a_realises_fact_attribute_no_closure_filed_is_an_orphan() {
    let mut store = store_with(Vec::new(), Vec::new());
    store.records = vec![ActRecord::open(opening("checkout-totals"))];
    let classes = classes(&judge_claims(&store));
    assert!(classes.contains(&Class::S013), "{classes:?}");
}

#[test]
fn a_realises_fact_attribute_resolves_once_a_closure_files_it() {
    let mut store = store_with(Vec::new(), Vec::new());
    store.records = vec![with_closure("checkout-totals", &["det/basket-rounding-is-half-even"])];
    let found = classes(&judge_claims(&store));
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn a_determination_a_closure_did_not_file_stays_an_orphan() {
    let mut store = store_with(Vec::new(), Vec::new());
    store.records = vec![with_closure("checkout-totals", &["det/something-else"])];
    assert!(classes(&judge_claims(&store)).contains(&Class::S013));
}

#[test]
fn claims_are_not_judged_without_an_inventory() {
    let store = SpecStore { records: vec![ActRecord::open(opening("x"))], ..SpecStore::default() };
    assert!(judge_claims(&store).is_empty());
}

#[test]
fn an_empty_store_has_nothing_to_judge() {
    assert!(judge_store(&SpecStore::default()).is_empty());
}

#[test]
fn judge_store_composes_every_class() {
    let mut store = store_with(Vec::new(), Vec::new());
    store.records = vec![ActRecord::open(opening("checkout-totals"))];
    let found = classes(&judge_store(&store));
    assert!(found.contains(&Class::S001), "the open record");
    assert!(found.contains(&Class::S005), "the uncovered entry point");
}
