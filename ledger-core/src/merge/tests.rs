//! Reconciliation over in-memory stores: the plan's classification, and
//! the arbitration acts over a real tempdir store.

use crate::allocation::AllocationKind;
use crate::store::Store;
use crate::testkit;

use super::conflicts;
use super::plan::plan;
use super::ConflictClass;

fn store_of(versions: Vec<crate::version::VersionRaw>) -> Store {
    testkit::store(testkit::changeset(versions, Vec::new()))
}

fn root() -> crate::version::VersionRaw {
    testkit::sealed(testkit::version())
}

fn child_of(parent: &crate::version::VersionRaw, statement: &str) -> crate::version::VersionRaw {
    let mut child = testkit::version();
    child.parent = Some(parent.hash.clone());
    child.statement = statement.into();
    testkit::sealed(child)
}

#[test]
fn the_plan_calls_disjoint_and_fast_forward_work_mechanical() {
    let base = store_of(vec![root()]);
    let ours = store_of(vec![root()]);
    let theirs = store_of(vec![root(), child_of(&root(), "revised on theirs")]);
    let p = plan(["ours", "theirs", "base"], &ours, &theirs, &base);
    assert!(p.conflicts.is_empty(), "{p:?}");
    assert!(
        p.mechanical.iter().any(|l| l.contains("theirs fast-forwards")),
        "{:?}",
        p.mechanical
    );
}

#[test]
fn the_plan_names_a_divergent_revision_with_the_common_parent() {
    let base = store_of(vec![root()]);
    let ours = store_of(vec![root(), child_of(&root(), "ours' reading")]);
    let theirs = store_of(vec![root(), child_of(&root(), "theirs' reading")]);
    let p = plan(["ours", "theirs", "base"], &ours, &theirs, &base);
    assert_eq!(p.conflicts.len(), 1, "{p:?}");
    assert_eq!(p.conflicts[0].class, ConflictClass::DivergentRevision);
    assert!(p.conflicts[0].detail.contains("common parent"), "{p:?}");
    assert!(super::plan::render(&p).contains("CONFLICT [divergent-revision]"));
}

#[test]
fn the_plan_distinguishes_a_divergent_allocation() {
    let base = store_of(vec![root()]);
    let mut ours_child = testkit::version();
    ours_child.parent = Some(root().hash.clone());
    ours_child.allocation = Some(AllocationKind::Judgment);
    ours_child.discharge.clear();
    ours_child.actor = Some(testkit::identity("fixture-human@example"));
    let ours = store_of(vec![root(), testkit::sealed(ours_child)]);
    let theirs = store_of(vec![root(), child_of(&root(), "still a constraint, restated")]);
    let p = plan(["ours", "theirs", "base"], &ours, &theirs, &base);
    assert_eq!(p.conflicts[0].class, ConflictClass::DivergentAllocation, "{p:?}");
    assert!(p.conflicts[0].detail.contains("judgment"), "{p:?}");
}

#[test]
fn the_plan_surfaces_an_overtaken_acceptance_as_a_notice() {
    let accepted = root();
    let acceptance = testkit::acceptance(&accepted);
    let ours = testkit::store(testkit::changeset(vec![accepted.clone()], vec![acceptance]));
    let theirs = store_of(vec![root(), child_of(&root(), "revised past the signature")]);
    let base = store_of(vec![root()]);
    let p = plan(["ours", "theirs", "base"], &ours, &theirs, &base);
    assert!(p.conflicts.is_empty(), "nothing to arbitrate: {p:?}");
    assert_eq!(p.notices.len(), 1, "{p:?}");
    assert!(p.notices[0].contains("overtaken"), "{p:?}");
    assert!(p.notices[0].contains("awaits fresh acceptance"), "{p:?}");
}

#[test]
fn the_plan_names_a_divergent_floor() {
    let base = store_of(vec![root()]);
    let mut ours = store_of(vec![root()]);
    ours.sets[0].tolerance_floor = crate::tier::Tier::T0;
    let mut theirs = store_of(vec![root()]);
    theirs.sets[0].tolerance_floor = crate::tier::Tier::T2;
    let p = plan(["ours", "theirs", "base"], &ours, &theirs, &base);
    assert_eq!(p.conflicts.len(), 1, "{p:?}");
    assert_eq!(p.conflicts[0].class, ConflictClass::DivergentFloor);
}

#[test]
fn conflicts_present_both_tips_with_provenance_and_the_fork_point() {
    let r = root();
    let store = store_of(vec![r.clone(), child_of(&r, "left"), child_of(&r, "right")]);
    let found = conflicts::find(&store);
    assert_eq!(found.forks.len(), 1, "{found:?}");
    let fork = &found.forks[0];
    assert_eq!(fork.tips.len(), 2);
    assert!(fork.common_parent.as_deref().is_some_and(|c| c.contains(r.hash.short())), "{fork:?}");
    assert!(fork.tips.iter().all(|t| t.filed_by.contains("fixture-human@example")), "{fork:?}");
}

// ---------------------------------------------------------------------------
// Arbitration acts, over a real store on disk.
// ---------------------------------------------------------------------------

mod acts {
    use std::path::Path;

    use chrono::{DateTime, Utc};

    use crate::author::{AddArgs, AllocationArgs, Author, AuthorError, ReviseArgs};
    use crate::id::DecisionId;
    use crate::merge::conflicts;
    use crate::merge::resolve::ForkChoice;
    use crate::mint::UlidMint;
    use crate::set::Ground;
    use crate::testkit;
    use crate::tier::Tier;
    use crate::verify::{self, Options};

    fn now() -> DateTime<Utc> {
        testkit::stamp("2026-08-11T12:00:00Z")
    }

    fn author(dir: &Path, mint_millis: u64) -> Author {
        Author::new(
            dir,
            testkit::identity("fixture-human@example"),
            now(),
            UlidMint::fixed(mint_millis),
        )
    }

    fn verify_report(dir: &Path) -> verify::Report {
        let store = crate::store::load(dir);
        verify::verify(
            &store,
            &Options { gate: None, today: testkit::date("2026-08-11"), blame: false },
        )
    }

    /// Init the store and open the fixture author on a declared set.
    fn scaffold(dir: &Path) -> Author {
        crate::init::apply_init(dir, &crate::init::plan_init(dir)).expect("init");
        let mut a = author(dir, 1_755_000_000_000);
        a.declare(crate::author::DeclareArgs {
            id: "merge-acts".into(),
            title: "Arbitration acts".into(),
            tolerance_floor: Tier::T1,
            ground: Ground::Characterised,
            owner: None,
            notes: None,
        })
        .expect("declare");
        a
    }

    fn add_decision(a: &mut Author, statement: &str) -> DecisionId {
        a.add(AddArgs {
            set: "merge-acts".into(),
            statement: statement.into(),
            namespace: Some("fixture.merge".into()),
            allocation: AllocationArgs {
                store: Some(crate::allocation::AllocationKind::Constraint),
                discharge: vec!["analyzer:DEC001".parse().expect("ref")],
                ..AllocationArgs::default()
            },
            tolerance_override: None,
            based_on: Vec::new(),
            note: None,
        })
        .expect("add")
        .lines
        .first()
        .and_then(|l| l.split_whitespace().nth(1))
        .expect("id")
        .parse()
        .expect("decision id")
    }

    /// File the other branch's change-set as a log file of its own — what
    /// a git merge of two branches' logs leaves behind.
    fn file_branch_changeset(dir: &Path, versions: Vec<crate::version::VersionRaw>, ulid: &str) {
        let mut cs = testkit::changeset(versions, Vec::new());
        cs.decisions.clear();
        cs.id = format!("cs:{ulid}").parse().expect("id");
        let path = dir.join(".decisions/log").join(cs.file_name());
        std::fs::write(&path, serde_yaml::to_string(&cs).expect("yaml")).expect("write");
    }

    /// A store whose one decision is forked: the left tip via the verb, the
    /// right tip as the other branch's change-set file.
    fn forked_store(dir: &Path) -> (Author, DecisionId) {
        let mut a = scaffold(dir);
        let id = add_decision(&mut a, "The statement both writers started from.");
        a.revise(&id, ReviseArgs {
            statement: "The left writer's revision.".into(),
            based_on: Vec::new(),
            expected_parent: None,
        })
        .expect("left revision");
        let store = crate::store::load(dir);
        let left = store
            .log
            .iter()
            .flat_map(|c| &c.file.versions)
            .find(|v| v.parent.is_some())
            .expect("left")
            .clone();
        let mut right = left.clone();
        right.statement = "The right writer's revision.".into();
        right.hash = crate::hash::version_hash(&right);
        file_branch_changeset(dir, vec![right], "01K2C4YQJ3F8M0PT5W7NZ9RD99");
        (a, id)
    }

    #[test]
    fn a_fork_arbitration_heals_the_chain_and_leaves_the_tip_unaccepted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (mut a, id) = forked_store(dir.path());
        let before = verify_report(dir.path());
        assert!(!before.is_conformant(), "the fork fails the gate first");
        assert!(before.graph.iter().any(|g| g.class == crate::graph::GraphClass::G004));

        let found = conflicts::find(&crate::store::load(dir.path()));
        let fork = found.forks.first().expect("a fork");
        let applied = a.resolve_fork(fork, &ForkChoice::TipStands(0)).expect("arbitrate");
        assert!(applied.lines.iter().any(|l| l.contains("unaccepted")), "{:?}", applied.lines);

        let after = verify_report(dir.path());
        assert!(after.is_conformant(), "{:?} {:?}", after.findings, after.graph);
        assert_eq!(after.awaiting_acceptance, vec![id.to_string()],
            "the reconciled version awaits fresh acceptance — no signature survived");
        // The arbitration is on the record as an ordinary act.
        let store = crate::store::load(dir.path());
        assert!(
            store.log.iter().any(|c| c
                .file
                .note
                .as_deref()
                .is_some_and(|n| n.contains("merge arbitration: divergent-revision"))),
            "the arbitration act names itself"
        );
    }

    #[test]
    fn a_composed_arbitration_files_the_composed_statement() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (mut a, id) = forked_store(dir.path());
        let found = conflicts::find(&crate::store::load(dir.path()));
        let fork = found.forks.first().expect("a fork");
        a.resolve_fork(
            fork,
            &ForkChoice::Compose {
                base_tip: 0,
                statement: "Both writers' concerns, restated together.".into(),
            },
        )
        .expect("arbitrate");
        let store = crate::store::load(dir.path());
        let view = crate::verify::view::View::build(&store);
        let tip = view
            .latest
            .get(&id.to_string())
            .and_then(|&i| view.versions.get(i))
            .expect("tip");
        assert_eq!(tip.raw.statement, "Both writers' concerns, restated together.");
        assert!(tip.raw.merged_from.is_some(), "the act names the chain it closed");
        assert!(verify_report(dir.path()).is_conformant());
    }

    /// A store where two claimants supersede one decision: ours' claim via
    /// the verb, theirs' as the other branch's change-set file.
    fn competing_store(dir: &Path) -> (Author, DecisionId, DecisionId) {
        let mut a = scaffold(dir);
        let d = add_decision(&mut a, "The decision both claim to replace.");
        let x = add_decision(&mut a, "The first claimant.");
        let y = add_decision(&mut a, "The second claimant.");
        a.supersede(crate::author::SupersedeArgs {
            superseded: d.clone(),
            by: x.clone(),
            reason: None,
        })
        .expect("ours' claim");
        let store = crate::store::load(dir);
        let view = crate::verify::view::View::build(&store);
        let y_tip = view
            .latest
            .get(&y.to_string())
            .and_then(|&i| view.versions.get(i))
            .expect("y tip");
        let mut claim = y_tip.raw.clone();
        claim.parent = Some(y_tip.raw.hash.clone());
        claim.supersedes = Some(d.clone());
        claim.hash = crate::hash::version_hash(&claim);
        file_branch_changeset(dir, vec![claim], "01K2C4YQJ3F8M0PT5W7NZ9RD98");
        (a, d, x)
    }

    #[test]
    fn a_competing_supersession_settles_by_withdrawing_the_losing_claim() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (mut a, d, x) = competing_store(dir.path());
        let before = verify_report(dir.path());
        assert!(
            before.graph.iter().any(|g| g.class == crate::graph::GraphClass::G005),
            "{:?}",
            before.graph
        );
        let found = conflicts::find(&crate::store::load(dir.path()));
        assert_eq!(found.supersessions.len(), 1, "{found:?}");
        let winner = found.supersessions[0]
            .claimants
            .iter()
            .position(|c| c.decision == x.to_string())
            .expect("x among claimants");
        a.resolve_supersession(&found.supersessions[0], winner).expect("arbitrate");

        let after = verify_report(dir.path());
        assert!(after.is_conformant(), "{:?} {:?}", after.findings, after.graph);
        let store = crate::store::load(dir.path());
        let view = crate::verify::view::View::build(&store);
        let map = crate::verify::state::supersession(&view);
        assert_eq!(map.get(&d.to_string()), Some(&x.to_string()), "the chosen claim stands");
    }

    #[test]
    fn an_arbitration_against_a_moved_store_refuses() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (mut a, _) = forked_store(dir.path());
        let found = conflicts::find(&crate::store::load(dir.path()));
        let fork = found.forks.first().expect("a fork");
        a.resolve_fork(fork, &ForkChoice::TipStands(0)).expect("first arbitration");
        // The same presentation applied again: the store has moved on.
        let err = a.resolve_fork(fork, &ForkChoice::TipStands(0)).expect_err("stale");
        assert!(matches!(err, AuthorError::Conflict(_)), "{err}");
        assert!(err.to_string().contains("no longer forked"), "{err}");
    }
}
