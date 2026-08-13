//! The signing verbs: accept, revoke, supersede, plus their refusals.
//!
//! Split out of `tests.rs` at the workspace's 400-line ceiling. The seam is
//! the natural one: filing a decision and signing one are different acts,
//! and only the second is the principal's.

use crate::finding::VerifyClass;
use crate::testkit;

// The fixtures live in the sibling that files the decisions these tests
// sign; re-stating them here would be a second copy that can disagree.
use super::super::*;
use super::*;

#[test]
fn accept_signs_the_tip_and_a_revision_returns_the_decision_to_awaiting() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    a.accept(AcceptArgs { decision: id.clone(), expires_at: None }).expect("accept");
    let err = a.accept(AcceptArgs { decision: id.clone(), expires_at: None }).expect_err("twice");
    assert!(matches!(err, AuthorError::Conflict(_)), "{err}");

    a.revise(&id, ReviseArgs {
        statement: "Restated after acceptance.".into(),
        based_on: Vec::new(),
        revisit_if: None,
        expected_parent: None,
    })
    .expect("revise");
    let text = report::status(&crate::store::load(dir.path()), testkit::date("2026-08-10"));
    assert!(text.contains("awaiting-acceptance"), "the old signature is history: {text}");
}

#[test]
fn accept_refuses_an_expired_on_arrival_signature_with_l003() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    let err = a
        .accept(AcceptArgs { decision: id, expires_at: Some(testkit::date("2026-08-01")) })
        .expect_err("expired on arrival");
    let AuthorError::Refused(findings) = err else { panic!("expected refusal, got {err}") };
    assert!(findings.iter().any(|f| f.class == VerifyClass::L003), "{findings:?}");
}

#[test]
fn accept_refuses_a_model_identity_with_l006() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    let mut model = Author::new(
        dir.path(),
        testkit::identity("claude@example.com"),
        now(),
        UlidMint::fixed(1_755_000_200_000),
    );
    let err = model.accept(AcceptArgs { decision: id, expires_at: None }).expect_err("model acceptor");
    let AuthorError::Refused(findings) = err else { panic!("expected refusal, got {err}") };
    assert!(findings.iter().any(|f| f.class == VerifyClass::L006), "{findings:?}");
}

#[test]
fn revoke_unsays_and_refuses_the_unknown_and_the_double() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let id = add_constraint(&mut a);
    a.accept(AcceptArgs { decision: id.clone(), expires_at: None }).expect("accept");
    let store = crate::store::load(dir.path());
    let view = crate::verify::view::View::build(&store);
    let acc = view.acceptances.first().expect("acceptance").acceptance.id.clone();

    // Unknown: refused with the same schema class verify reports.
    let ghost: crate::id::AcceptanceId = "acc:01K2C4YQJ3F8M0PT5W7NZ9RDXZ".parse().expect("id");
    let err = a
        .revoke(RevokeArgs { acceptance: ghost, reason: "never filed".into() })
        .expect_err("unknown acceptance");
    let AuthorError::Refused(findings) = err else { panic!("expected refusal, got {err}") };
    assert!(findings.iter().any(|f| f.class == VerifyClass::Schema), "{findings:?}");

    a.revoke(RevokeArgs { acceptance: acc.clone(), reason: "signed the wrong tier".into() })
        .expect("revoke");
    let err = a
        .revoke(RevokeArgs { acceptance: acc, reason: "again".into() })
        .expect_err("double revocation");
    assert!(matches!(err, AuthorError::Conflict(_)), "{err}");
    assert_eq!(gate(dir.path()), Vec::<VerifyClass>::new());
}

#[test]
fn supersede_builds_a_walkable_chain_and_refuses_forking_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let first = add_constraint(&mut a);
    let second = add_constraint(&mut a);
    let third = add_constraint(&mut a);
    a.supersede(SupersedeArgs { superseded: first.clone(), by: second.clone(), reason: Some("narrowed".into()) })
        .expect("supersede");
    a.supersede(SupersedeArgs { superseded: second.clone(), by: third.clone(), reason: None })
        .expect("chain");
    let err = a
        .supersede(SupersedeArgs { superseded: first.clone(), by: third, reason: None })
        .expect_err("forking the chain");
    assert!(matches!(err, AuthorError::Conflict(_)), "{err}");

    let store = crate::store::load(dir.path());
    let view = crate::verify::view::View::build(&store);
    let chain = crate::verify::state::chain_from(
        &first.to_string(),
        &crate::verify::state::supersession(&view),
    );
    assert_eq!(chain.len(), 3, "{chain:?}");
    let rows = crate::verify::state::states(&view, testkit::date("2026-08-10"));
    assert_eq!(rows.get(&first.to_string()).expect("row").state.as_str(), "superseded");
}

/// A store whose one decision two writers forked: one revised through the
/// verb; the other branch's change-set (same parent, different content)
/// lands as a file of its own — what a git merge of two logs leaves.
fn forked_store(dir: &Path) -> (Author, DecisionId) {
    let mut a = author(dir);
    declare(&mut a);
    let id = add_constraint(&mut a);
    let args = ReviseArgs {
        statement: "The left writer's revision.".into(),
        based_on: Vec::new(),
        revisit_if: None,
        expected_parent: None,
    };
    a.revise(&id, args).expect("left revision");
    let store = crate::store::load(dir);
    let versions = store.log.iter().flat_map(|c| &c.file.versions);
    let left = versions.filter(|v| v.parent.is_some()).next_back().expect("the left revision");
    let mut right = left.clone();
    right.statement = "The right writer's revision.".into();
    right.hash = crate::hash::version_hash(&right);
    let mut cs = testkit::changeset(vec![right], Vec::new());
    cs.decisions.clear();
    cs.id = "cs:01K2C4YQJ3F8M0PT5W7NZ9RD99".parse().expect("id");
    let path = dir.join(".decisions/log").join(cs.file_name());
    std::fs::write(&path, serde_yaml::to_string(&cs).expect("yaml")).expect("write");
    (a, id)
}

#[test]
fn a_forked_chain_refuses_revision_rather_than_burying_the_conflict() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (mut a, id) = forked_store(dir.path());
    let err = a
        .revise(&id, ReviseArgs {
            statement: "Extending either tip buries the conflict.".into(),
            based_on: Vec::new(),
        revisit_if: None,
            expected_parent: None,
        })
        .expect_err("revise on a fork");
    assert!(matches!(err, AuthorError::Conflict(_)), "{err}");
    assert!(err.to_string().contains("merge --resolve"), "{err}");
}

#[test]
fn a_forked_chain_has_no_signable_tip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (mut a, id) = forked_store(dir.path());
    let err = a
        .accept(AcceptArgs { decision: id, expires_at: None })
        .expect_err("accept on a fork");
    assert!(matches!(err, AuthorError::Conflict(_)), "{err}");
    assert!(err.to_string().contains("no one latest version"), "{err}");
}

#[test]
fn declare_refuses_a_second_declaration_of_the_same_set() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut a = author(dir.path());
    declare(&mut a);
    let err = a
        .declare(DeclareArgs {
            id: "verbs".into(),
            title: "Again".into(),
            tolerance_floor: Tier::T2,
            ground: Ground::Characterised,
            owner: None,
            notes: None,
        })
        .expect_err("duplicate set");
    assert!(matches!(err, AuthorError::Usage(_)), "{err}");
}
