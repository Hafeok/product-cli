//! `show` over hand-built stores: the fields, the two edge kinds kept
//! apart, and the acceptance situation.

use super::*;
use crate::testkit;

fn today() -> NaiveDate {
    testkit::date("2026-08-13")
}

fn pointers(edges: &[Edge]) -> Vec<String> {
    edges.iter().map(|e| e.pointer.clone()).collect()
}

/// A resolver that knows one pointer, standing in for the store a real
/// pointer lands in.
struct OneText(&'static str, &'static str);

impl BasisText for OneText {
    fn text(&self, pointer: &str) -> Option<String> {
        (pointer == self.0).then(|| self.1.to_string())
    }
}

#[test]
fn an_unknown_decision_has_no_screen() {
    let store = testkit::store(testkit::changeset(vec![], vec![]));
    assert!(screen(&store, &testkit::decision_id(), today(), &NoText).is_none());
}

#[test]
fn the_screen_carries_the_hashed_content_and_the_filing_act() {
    let v = testkit::sealed(testkit::version());
    let store = testkit::store(testkit::changeset(vec![v.clone()], vec![]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    assert_eq!(s.statement, v.statement);
    assert_eq!(s.set, "ledger-design");
    assert_eq!(s.set_floor.as_deref(), Some("T1"));
    assert_eq!(s.hash, v.hash.to_string());
    assert_eq!(s.versions_in_chain, 1);
    assert_eq!(s.allocation.as_deref(), Some("constraint"));
    assert_eq!(s.filed.by, "fixture-human@example");
    assert_eq!(s.state, DispositionState::AwaitingAcceptance);
}

#[test]
fn ground_and_reopen_edges_are_separate_fields_and_separate_blocks() {
    let mut raw = testkit::version();
    raw.revisit_if = vec!["claim:DDD-adapter-02@sha256:b333063d".parse().expect("token")];
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], vec![]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    assert_eq!(pointers(&s.based_on), vec!["prd:decision-ledger-prd#4.2.1".to_string()]);
    assert_eq!(pointers(&s.revisit_if), vec!["claim:DDD-adapter-02@sha256:b333063d".to_string()]);
    let text = render(&s);
    let ground = text.find("based on").expect("ground block");
    let reopen = text.find("revisit if").expect("reopen block");
    assert!(ground < reopen, "the two blocks are distinct and ordered:\n{text}");
    // The reopen pointer must not appear inside the ground block.
    let between = &text[ground..reopen];
    assert!(!between.contains("DDD-adapter-02"), "a reopen edge rendered as ground:\n{text}");
}

#[test]
fn an_unaccepted_decision_names_the_command_that_would_sign_it() {
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(testkit::version())], vec![]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    let text = render(&s);
    assert!(text.contains("none on record"), "{text}");
    assert!(text.contains("ledger accept dec:hafeok.ledger/"), "{text}");
}

#[test]
fn a_live_acceptance_shows_as_live_and_stops_prompting() {
    let v = testkit::sealed(testkit::version());
    let acc = testkit::acceptance(&v);
    let store = testkit::store(testkit::changeset(vec![v], vec![acc]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    assert_eq!(s.state, DispositionState::Decided);
    assert_eq!(s.acceptances.len(), 1);
    assert_eq!(s.acceptances[0].standing, "live");
    let text = render(&s);
    assert!(!text.contains("to accept"), "a signed decision still prompts:\n{text}");
}

#[test]
fn the_json_form_carries_the_same_facts_as_the_text() {
    let mut raw = testkit::version();
    raw.revisit_if = vec!["claim:DDD-gates-01@sha256:6f500ee8".parse().expect("token")];
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], vec![]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    let json = serde_json::to_value(&s).expect("serialize");
    assert_eq!(json["revisit_if"][0]["pointer"], "claim:DDD-gates-01@sha256:6f500ee8");
    assert_eq!(json["state"], "awaiting-acceptance");
    assert_eq!(json["hash"], s.hash);
    // An absent edge list is omitted rather than rendered as an empty
    // array: absent and empty are the same fact, and one spelling is enough.
    assert!(json.get("supersedes").is_none());
}

/// The argument, not just the type. A reader ruling on `mandate:…` is
/// ruling on what the mandate says, so the screen has to carry it.
#[test]
fn a_resolved_edge_renders_its_argument_under_the_pointer() {
    let mut raw = testkit::version();
    raw.based_on = vec!["mandate:dec/ddd/internal-not-surface".parse().expect("basis")];
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(raw)], vec![]));
    let argument = "Context& repositories are application repositories.";
    let texts = OneText("mandate:dec/ddd/internal-not-surface", argument);
    let s = screen(&store, &testkit::decision_id(), today(), &texts).expect("screen");
    assert_eq!(s.based_on[0].argument.as_deref(), Some(argument));
    assert!(render(&s).contains(argument), "{}", render(&s));
}

#[test]
fn an_unresolvable_edge_renders_bare_rather_than_blank() {
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(testkit::version())], vec![]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    assert!(s.based_on[0].argument.is_none());
    assert!(render(&s).contains("prd:decision-ledger-prd#4.2.1"));
}

/// The grouping is derived from content, so it cannot go stale: a
/// criterion discharged solely by the contract check is a transcription;
/// a judgment is an individual read.
#[test]
fn the_weight_class_is_derived_from_the_discharge() {
    let mut transcription = testkit::version();
    transcription.allocation = Some(crate::allocation::AllocationKind::Criterion);
    transcription.discharge = vec!["contract:seam/ledger/canonical-form".parse().expect("ref")];
    transcription.discharge_stage = Some(crate::discharge::Stage::Pr);
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(transcription)], vec![]));
    let s = screen(&store, &testkit::decision_id(), today(), &NoText).expect("screen");
    assert_eq!(s.group, "mechanical");

    // The fixture's default is an analyzer-discharged constraint.
    let plain = testkit::store(testkit::changeset(vec![testkit::sealed(testkit::version())], vec![]));
    let s = screen(&plain, &testkit::decision_id(), today(), &NoText).expect("screen");
    assert_eq!(s.group, "individual");
}

#[test]
fn a_selector_covers_a_whole_group_in_one_pass() {
    let store = testkit::store(testkit::changeset(vec![testkit::sealed(testkit::version())], vec![]));
    let all = screens(&store, &Selector::All, today(), &NoText);
    assert_eq!(all.len(), 1);
    let by_set = screens(&store, &Selector::Set("ledger-design".into()), today(), &NoText);
    assert_eq!(by_set.len(), 1);
    assert!(screens(&store, &Selector::Set("absent".into()), today(), &NoText).is_empty());
    let mechanical = screens(&store, &Selector::Group(Group::Mechanical), today(), &NoText);
    assert!(mechanical.is_empty(), "the fixture is an individual read");
    let individual = screens(&store, &Selector::Group(Group::Individual), today(), &NoText);
    assert_eq!(individual.len(), 1);
    assert!(render_all(&individual).starts_with("1 decision(s)"));
    assert_eq!(render_all(&[]), "no decisions match\n");
}

#[test]
fn an_unknown_group_names_the_ones_that_exist() {
    let err = Group::parse("mechanicals").expect_err("unknown group");
    assert!(err.contains("mechanical | individual"), "{err}");
}
