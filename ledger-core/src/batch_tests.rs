//! The plan over hand-built stores: what it pins, what moves it, what it
//! refuses to call signable.

use super::*;
use crate::acceptance::Acceptance;
use crate::changeset::{ChangeSet, DecisionRecord};
use crate::id::DecisionId;
use crate::show::{screens, Group, NoText};
use crate::store::{LoggedChangeSet, Store};
use crate::testkit;
use crate::version::VersionRaw;

fn today() -> NaiveDate {
    testkit::date("2026-08-13")
}

fn me() -> Identity {
    testkit::identity("fixture-human@example")
}

/// A decision id differing only in its last ULID character, so a test store
/// can hold several decisions without hand-writing ids.
fn dec(tail: char) -> DecisionId {
    let base = testkit::DEC_ULID;
    let ulid = format!("{}{tail}", &base[..base.len() - 1]);
    format!("dec:hafeok.ledger/{ulid}").parse().expect("decision id")
}

/// One sealed version of `dec(tail)`, with `statement` distinguishing it.
fn version(tail: char, statement: &str) -> VersionRaw {
    let mut raw = testkit::version();
    raw.decision = dec(tail);
    raw.statement = statement.to_string();
    testkit::sealed(raw)
}

/// A change-set whose id ends in `tail`, so the log orders deterministically.
fn changeset(tail: char, versions: Vec<VersionRaw>, acceptances: Vec<Acceptance>) -> ChangeSet {
    let base = testkit::CS_ULID;
    let mut cs = testkit::changeset(versions, acceptances);
    cs.id = format!("cs:{}{tail}", &base[..base.len() - 1]).parse().expect("change-set id");
    cs.decisions = cs
        .versions
        .iter()
        .map(|v| DecisionRecord {
            id: v.decision.clone(),
            created_at: testkit::stamp("2026-08-10T09:14:22Z"),
            created_by: me(),
        })
        .collect();
    cs
}

fn store(changesets: Vec<ChangeSet>) -> Store {
    Store {
        root: std::path::PathBuf::from("/fixture"),
        dir: std::path::PathBuf::from("/fixture/.decisions"),
        sets: vec![testkit::set()],
        log: changesets
            .into_iter()
            .map(|file| LoggedChangeSet {
                path: std::path::PathBuf::from(format!("/fixture/.decisions/log/{}.yml", file.id.ulid())),
                file,
            })
            .collect(),
        schema_findings: Vec::new(),
    }
}

fn planned(store: &Store, selector: &Selector) -> Plan {
    plan(&screens(store, selector, today(), &NoText), selector, &me(), today())
}

#[test]
fn the_selection_is_enumerated_as_pinned_pairs_in_id_order() {
    let s = store(vec![changeset(
        'A',
        vec![version('A', "first"), version('B', "second")],
        vec![],
    )]);
    let p = planned(&s, &Selector::All);
    assert_eq!(p.members.len(), 2);
    assert!(p.members[0].decision < p.members[1].decision);
    for m in &p.members {
        assert!(m.version.starts_with("sha256:"), "{m:?}");
        assert_eq!(m.standing, Standing::Signable.as_str());
        assert_eq!(m.state, "awaiting-acceptance");
    }
    assert!(p.manifest.starts_with("sha256:"), "{}", p.manifest);
}

/// The whole point of the manifest: what `show` resolved is what `accept`
/// resolves, so a principal signs the set they read.
#[test]
fn show_and_the_plan_resolve_to_the_identical_set() {
    let s = store(vec![changeset(
        'A',
        vec![version('A', "first"), version('B', "second")],
        vec![],
    )]);
    let selector = Selector::Group(Group::Individual);
    let shown: Vec<String> =
        screens(&s, &selector, today(), &NoText).iter().map(|x| x.decision.clone()).collect();
    let planned: Vec<String> =
        planned(&s, &selector).members.iter().map(|m| m.decision.clone()).collect();
    assert_eq!(shown, planned);
}

#[test]
fn a_selector_that_matches_nothing_plans_nothing() {
    let s = store(vec![changeset('A', vec![version('A', "first")], vec![])]);
    assert!(planned(&s, &Selector::Set("absent".into())).members.is_empty());
}

#[test]
fn the_manifest_moves_when_a_member_is_revised() {
    let s = store(vec![changeset('A', vec![version('A', "first")], vec![])]);
    let before = planned(&s, &Selector::All).manifest;
    let mut revised = version('A', "restated");
    revised.parent = Some(version('A', "first").hash);
    let revised = testkit::sealed(revised);
    let after = store(vec![changeset('A', vec![version('A', "first"), revised], vec![])]);
    assert_ne!(before, planned(&after, &Selector::All).manifest);
}

/// An acceptance filed between the read and the run changes what the run
/// would *do* without changing which decisions it covers. Standing is in
/// the digest precisely so that cannot pass unnoticed.
#[test]
fn the_manifest_moves_when_a_member_is_signed_in_the_meantime() {
    let v = version('A', "first");
    let before = planned(&store(vec![changeset('A', vec![v.clone()], vec![])]), &Selector::All);
    let mut acceptance = testkit::acceptance(&v);
    acceptance.decision = v.decision.clone();
    let after = planned(
        &store(vec![changeset('A', vec![v], vec![acceptance])]),
        &Selector::All,
    );
    assert_ne!(before.manifest, after.manifest);
    assert_eq!(after.members[0].standing, Standing::AlreadySigned.as_str());
}

#[test]
fn the_manifest_is_selector_and_actor_specific() {
    let s = store(vec![changeset('A', vec![version('A', "first")], vec![])]);
    let all = planned(&s, &Selector::All);
    let by_set = planned(&s, &Selector::Set("ledger-design".into()));
    assert_eq!(
        all.members.iter().map(|m| &m.decision).collect::<Vec<_>>(),
        by_set.members.iter().map(|m| &m.decision).collect::<Vec<_>>()
    );
    assert_ne!(all.manifest, by_set.manifest, "the same members under a different selector");
    let other = manifest("all", "someone-else@example", &all.members);
    assert_ne!(all.manifest, other, "the same selection read by a different principal");
}

#[test]
fn an_indeterminate_basis_makes_a_member_held_and_blocks_the_plan() {
    let mut raw = testkit::version();
    raw.decision = dec('A');
    raw.based_on = vec!["indeterminate:DDD-arch-03@sha256:8e7e".parse().expect("basis")];
    let s = store(vec![changeset('A', vec![testkit::sealed(raw)], vec![])]);
    let p = planned(&s, &Selector::All);
    assert_eq!(p.members[0].standing, Standing::Held.as_str());
    assert_eq!(p.blockers().len(), 1);
    assert!(p.signable().next().is_none());
    assert!(
        p.members[0].because.as_deref().is_some_and(|b| b.contains("indeterminate")),
        "{:?}",
        p.members[0]
    );
}

#[test]
fn an_expired_signature_leaves_a_member_signable_again() {
    let v = version('A', "first");
    let mut acceptance = testkit::acceptance(&v);
    acceptance.decision = v.decision.clone();
    acceptance.expires_at = Some(testkit::date("2026-08-12"));
    let s = store(vec![changeset('A', vec![v], vec![acceptance])]);
    let p = planned(&s, &Selector::All);
    assert_eq!(p.members[0].standing, Standing::Signable.as_str());
}

#[test]
fn a_manifest_is_recognised_by_shape_before_it_is_compared() {
    assert!(is_manifest_shaped(&format!("sha256:{}", "a".repeat(64))));
    assert!(!is_manifest_shaped(&format!("sha256:{}", "A".repeat(64))));
    assert!(!is_manifest_shaped("sha256:abc"));
    assert!(!is_manifest_shaped("mechanical"));
}

/// The log is append-only, so every state it has passed through is a prefix
/// of its change-sets — which is what lets a refusal name what moved without
/// anything having been stored between the two runs.
#[test]
fn the_state_a_manifest_was_read_against_is_recovered_from_the_log() {
    let first = changeset('A', vec![version('A', "first")], vec![]);
    let read_against = store(vec![first.clone()]);
    let manifest = planned(&read_against, &Selector::All).manifest;
    let moved = store(vec![first, changeset('B', vec![version('B', "second")], vec![])]);
    let now = planned(&moved, &Selector::All);
    assert_ne!(now.manifest, manifest);
    let before =
        locate(&moved, &Selector::All, &me(), today(), &manifest).expect("prefix located");
    let moved_rows = changes(&before, &now);
    assert_eq!(moved_rows.len(), 1);
    assert_eq!(moved_rows[0].decision, dec('B').to_string());
    assert!(moved_rows[0].what.contains("entered the selection, unread"), "{moved_rows:?}");
}

#[test]
fn a_manifest_from_no_state_this_log_has_held_is_not_located() {
    let s = store(vec![changeset('A', vec![version('A', "first")], vec![])]);
    let stranger = format!("sha256:{}", "b".repeat(64));
    assert!(locate(&s, &Selector::All, &me(), today(), &stranger).is_none());
}

#[test]
fn the_render_names_the_confirm_command_without_performing_it() {
    let s = store(vec![changeset('A', vec![version('A', "first")], vec![])]);
    let p = planned(&s, &Selector::Group(Group::Individual));
    let manifest = p.manifest.clone();
    let outcome =
        Outcome { plan: p, dry_run: true, signed: Vec::new(), filed: None, refusal: None };
    let text = render(&outcome, "0.01s");
    assert!(text.contains("dry run — nothing written"), "{text}");
    assert!(text.contains(&format!("ledger accept --group individual --confirm {manifest}")), "{text}");
    assert!(text.contains("1 signable · 0 already signed · 0 held · 0 forked"), "{text}");
}
