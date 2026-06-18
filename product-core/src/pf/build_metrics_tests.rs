//! Tests for build session metrics aggregation.

use super::*;

fn session() -> BuildSession {
    let mut s = BuildSession::new("casing-c");
    s.calls.push(CallRecord { capability: "code-writer".into(), gate: "dispatch".into(), prompt_tokens: 100, completion_tokens: 40 });
    s.calls.push(CallRecord { capability: "code-writer-heavy".into(), gate: "verify".into(), prompt_tokens: 200, completion_tokens: 60 });
    s.calls.push(CallRecord { capability: "code-writer-heavy".into(), gate: "verify".into(), prompt_tokens: 150, completion_tokens: 30 });
    s
}

#[test]
fn totals_sum_across_calls() {
    let s = session();
    assert_eq!(s.prompt_tokens(), 450);
    assert_eq!(s.completion_tokens(), 130);
    assert_eq!(s.total_tokens(), 580);
}

#[test]
fn rounds_count_calls_per_gate() {
    let r = session().rounds();
    assert_eq!(r.get("dispatch"), Some(&1));
    assert_eq!(r.get("verify"), Some(&2));
}

#[test]
fn tokens_attributed_by_capability() {
    let by = session().tokens_by_capability();
    assert_eq!(by.get("code-writer"), Some(&140));
    assert_eq!(by.get("code-writer-heavy"), Some(&440));
}

#[test]
fn round_trips_through_json() {
    let s = session();
    let back: BuildSession = serde_json::from_str(&s.to_json().expect("json")).expect("parse");
    assert_eq!(s, back);
}

#[test]
fn tc_spec_depth_recorded_on_session() {
    let mut s = BuildSession::new("demo");
    s.spec_depth = SpecDepth {
        nodes: 10,
        depth: 2,
        acceptance: 5,
        deciders: 3,
        context_tokens: 1024,
    };
    assert_eq!(s.spec_depth.nodes, 10);
    assert_eq!(s.spec_depth.depth, 2);
    assert_eq!(s.spec_depth.acceptance, 5);
    assert_eq!(s.spec_depth.deciders, 3);
    assert_eq!(s.spec_depth.context_tokens, 1024);

    let json = s.to_json().expect("json");
    let back: BuildSession = serde_json::from_str(&json).expect("parse");
    assert_eq!(back.spec_depth.nodes, 10);
    assert_eq!(back.spec_depth.depth, 2);
    assert_eq!(back.spec_depth.acceptance, 5);
    assert_eq!(back.spec_depth.deciders, 3);
    assert_eq!(back.spec_depth.context_tokens, 1024);
}
