//! Signing, at the gate and at the door.

use super::*;
use crate::act::tests::act;
use crate::record::tests::opening;
use crate::signing::{self, tests::key_pair};

const SETTLE: &str = "Shop.Api.BasketController.Settle#HttpPost";

fn classes(findings: &[Finding]) -> Vec<Class> {
    findings.iter().map(|f| f.class).collect()
}

fn principal() -> ledger_core::identity::Identity {
    "emil@example.com".parse().expect("identity parses")
}

#[test]
fn signing_is_off_until_a_key_is_trusted() {
    let found = judge_signature("subject", "sha256:aa", &principal(), None, &[]);
    assert!(found.is_empty(), "a repo with no trust root runs the whole flow unsigned");
}

#[test]
fn s014_unsigned_fails_once_a_key_is_trusted() {
    let (_, trust) = key_pair();
    let found = judge_signature("subject", "sha256:aa", &principal(), None, &[trust]);
    assert_eq!(classes(&found), vec![Class::S014]);
}

#[test]
fn a_valid_signature_passes() {
    let (signing_key, trust) = key_pair();
    let signature = signing::sign("sha256:aa", &signing_key);
    let found = judge_signature("subject", "sha256:aa", &principal(), Some(&signature), &[trust]);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn s015_a_signature_over_a_different_digest_fails() {
    let (signing_key, trust) = key_pair();
    let signature = signing::sign("sha256:bb", &signing_key);
    let found = judge_signature("subject", "sha256:aa", &principal(), Some(&signature), &[trust]);
    assert_eq!(classes(&found), vec![Class::S015]);
}

#[test]
fn s015_a_signature_by_an_untrusted_key_fails() {
    let (signing_key, _) = key_pair();
    let (_, someone_elses) = key_pair();
    let signature = signing::sign("sha256:aa", &signing_key);
    let found =
        judge_signature("subject", "sha256:aa", &principal(), Some(&signature), &[someone_elses]);
    assert_eq!(classes(&found), vec![Class::S015]);
}

#[test]
fn s015_gibberish_is_refused_rather_than_panicking() {
    let (_, trust) = key_pair();
    let found = judge_signature("subject", "sha256:aa", &principal(), Some("not-a-signature"), &[trust]);
    assert_eq!(classes(&found), vec![Class::S015]);
}

#[test]
fn the_store_judge_reaches_acts_closures_and_refusals() {
    let (_, trust) = key_pair();
    let mut store = SpecStore { trust: vec![trust], ..SpecStore::default() };
    store.acts = vec![act("act/settle-basket", &[SETTLE])];
    store.records = vec![crate::store::tests::closed_unsigned()];
    store.rejections = vec![crate::gate::tests::unsigned_rejection()];

    let found = judge_signatures(&store);
    assert_eq!(found.len(), 3, "every signed kind is reached: {found:?}");
    assert!(found.iter().all(|f| f.class == Class::S014));
}

#[test]
fn a_signed_act_survives_the_store_judge() {
    let (signing_key, trust) = key_pair();
    let mut signed = act("act/settle-basket", &[SETTLE]);
    signed.signature = Some(signing::sign(&signed.binds, &signing_key));
    let store = SpecStore { trust: vec![trust], acts: vec![signed], ..SpecStore::default() };

    assert!(judge_signatures(&store).is_empty());
}

#[test]
fn editing_a_signed_act_breaks_both_the_binding_and_the_signature() {
    let (signing_key, trust) = key_pair();
    let mut signed = act("act/settle-basket", &[SETTLE]);
    signed.signature = Some(signing::sign(&signed.binds, &signing_key));
    signed.settles = "Quietly rewritten after signing.".into();

    let store = SpecStore { trust: vec![trust], acts: vec![signed], ..SpecStore::default() };
    let found = classes(&judge_store(&store));
    assert!(found.contains(&Class::S007), "the binding: {found:?}");
    assert!(found.contains(&Class::S015), "and the signature: {found:?}");
}

#[test]
fn an_unsigned_open_record_has_no_signature_to_judge() {
    let (_, trust) = key_pair();
    let store = SpecStore {
        trust: vec![trust],
        records: vec![ActRecord::open(opening("checkout-totals"))],
        ..SpecStore::default()
    };
    // S001 fires because it is open; S014 does not, because there is no
    // closure yet and an absent act is not an unsigned one.
    let found = classes(&judge_signatures(&store));
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn an_act_performed_before_adoption_is_not_judged_unsigned() {
    let (_, mut trust) = key_pair();
    trust.added_at = chrono::DateTime::parse_from_rfc3339("2026-09-15T12:00:00Z")
        .map(|t| t.with_timezone(&chrono::Utc))
        .expect("fixed timestamp");
    let before = chrono::DateTime::parse_from_rfc3339("2026-09-15T09:00:00Z")
        .map(|t| t.with_timezone(&chrono::Utc))
        .expect("fixed timestamp");

    let found =
        judge_signature_at("subject", "sha256:aa", &principal(), None, &[trust], Some(before));
    assert!(found.is_empty(), "nobody could have signed it: {found:?}");
}

#[test]
fn an_act_performed_after_adoption_still_needs_a_signature() {
    let (_, mut trust) = key_pair();
    trust.added_at = chrono::DateTime::parse_from_rfc3339("2026-09-15T12:00:00Z")
        .map(|t| t.with_timezone(&chrono::Utc))
        .expect("fixed timestamp");
    let after = chrono::DateTime::parse_from_rfc3339("2026-09-15T13:00:00Z")
        .map(|t| t.with_timezone(&chrono::Utc))
        .expect("fixed timestamp");

    let found =
        judge_signature_at("subject", "sha256:aa", &principal(), None, &[trust], Some(after));
    assert_eq!(classes(&found), vec![Class::S014]);
}

#[test]
fn a_signature_that_does_not_verify_fails_whenever_it_was_written() {
    let (_, mut trust) = key_pair();
    trust.added_at = chrono::DateTime::parse_from_rfc3339("2026-09-15T12:00:00Z")
        .map(|t| t.with_timezone(&chrono::Utc))
        .expect("fixed timestamp");
    let before = chrono::DateTime::parse_from_rfc3339("2026-09-15T09:00:00Z")
        .map(|t| t.with_timezone(&chrono::Utc))
        .expect("fixed timestamp");

    // The grace is for absence, never for a signature that does not verify.
    let found = judge_signature_at(
        "subject", "sha256:aa", &principal(), Some("deadbeef"), &[trust], Some(before));
    assert_eq!(classes(&found), vec![Class::S015]);
}
