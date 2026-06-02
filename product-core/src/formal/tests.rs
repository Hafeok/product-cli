//! Unit tests for formal block parser (ADR-011, ADR-016)

use super::*;

#[test]
fn parse_evidence_block() {
    let body = "Some text\n\n\u{27E6}\u{0395}\u{27E7}\u{27E8}\u{03B4}\u{225C}0.95;\u{03C6}\u{225C}100;\u{03C4}\u{225C}\u{25CA}\u{207A}\u{27E9}\n";
    let blocks = parse_formal_blocks(body);
    let evidence = blocks.iter().find_map(|b| match b {
        FormalBlock::Evidence(e) => Some(e),
        _ => None,
    });
    assert!(evidence.is_some());
    let e = evidence.expect("evidence present");
    assert!((e.delta - 0.95).abs() < 0.001);
    assert_eq!(e.phi, 100);
    assert_eq!(e.tau, Stability::Stable);
}

#[test]
fn parse_types_block() {
    let body = "\u{27E6}\u{03A3}:Types\u{27E7}{\n  Node\u{225C}IRI\n  Role\u{225C}Leader|Follower|Learner\n}\n";
    let blocks = parse_formal_blocks(body);
    let types = blocks.iter().find_map(|b| match b {
        FormalBlock::Types(t) => Some(t),
        _ => None,
    });
    assert!(types.is_some());
    let t = types.expect("types present");
    assert_eq!(t.len(), 2);
    assert_eq!(t[0].name, "Node");
    assert_eq!(t[0].expr, "IRI");
    assert_eq!(t[1].name, "Role");
}

#[test]
fn parse_scenario_block() {
    let body = "\u{27E6}\u{039B}:Scenario\u{27E7}{\n  given\u{225C}cluster_init(nodes:2)\n  when\u{225C}elapsed(10s)\n  then\u{225C}\u{2203}n\u{2208}nodes: roles(n)=Leader\n}\n";
    let blocks = parse_formal_blocks(body);
    let scenario = blocks.iter().find_map(|b| match b {
        FormalBlock::Scenario(s) => Some(s),
        _ => None,
    });
    assert!(scenario.is_some());
    let s = scenario.expect("scenario present");
    assert!(s.given.is_some());
    assert!(s.when.is_some());
    assert!(s.then.is_some());
    assert!(s.given.as_ref().expect("given present").contains("cluster_init"));
}

#[test]
fn parse_invariants_block() {
    let body = "\u{27E6}\u{0393}:Invariants\u{27E7}{\n  \u{2200}s:ClusterState: |{n\u{2208}s.nodes | s.roles(n)=Leader}| = 1\n}\n";
    let blocks = parse_formal_blocks(body);
    let invs = blocks.iter().find_map(|b| match b {
        FormalBlock::Invariants(i) => Some(i),
        _ => None,
    });
    assert!(invs.is_some());
    assert_eq!(invs.expect("invariants present").len(), 1);
}

#[test]
fn evidence_delta_out_of_range() {
    let body = "\u{27E6}\u{0395}\u{27E7}\u{27E8}\u{03B4}\u{225C}1.5;\u{03C6}\u{225C}100;\u{03C4}\u{225C}\u{25CA}\u{207A}\u{27E9}\n";
    let result = parse_formal_blocks_with_diagnostics(body);
    assert!(!result.errors.is_empty(), "should report error for delta > 1.0");
    assert!(result.errors[0].contains("E001"));
    assert!(result.errors[0].contains("out of range"));
}

#[test]
fn empty_block_warning() {
    let body = "\u{27E6}\u{0393}:Invariants\u{27E7}{}\n";
    let result = parse_formal_blocks_with_diagnostics(body);
    assert!(!result.warnings.is_empty(), "should warn on empty block");
    assert!(result.warnings[0].contains("W004"));
}

#[test]
fn unrecognised_block_type_error() {
    let body = "\u{27E6}X:Unknown\u{27E7}{ stuff }\n";
    let result = parse_formal_blocks_with_diagnostics(body);
    assert!(!result.errors.is_empty(), "should error on unknown block type");
    assert!(result.errors[0].contains("unrecognised"));
}

#[test]
fn unclosed_delimiter_error() {
    // Unclosed \u{27E6} without \u{27E7}
    let body2 = "\u{27E6}\u{0393}:Invariants some text\n";
    let result = parse_formal_blocks_with_diagnostics(body2);
    assert!(!result.errors.is_empty(), "should detect unclosed \u{27E6}");
}

#[test]
fn valid_evidence_passes() {
    let body = "\u{27E6}\u{0395}\u{27E7}\u{27E8}\u{03B4}\u{225C}0.0;\u{03C6}\u{225C}0;\u{03C4}\u{225C}\u{25CA}?\u{27E9}\n";
    let result = parse_formal_blocks_with_diagnostics(body);
    assert!(result.errors.is_empty());
    assert_eq!(result.blocks.len(), 1);
}
