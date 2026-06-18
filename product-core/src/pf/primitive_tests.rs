//! Tests for named-algorithm primitives (§3.5) — reference + oracle.

use super::*;

fn centrality() -> Primitive {
    Primitive {
        id: "betweenness".into(),
        reference: "Brandes' betweenness centrality".into(),
        input: "a graph (nodes + edges)".into(),
        output: "a centrality score per node".into(),
        oracle: vec![
            OraclePair { input: "path a-b-c".into(), output: "b=1.0".into() },
            OraclePair { input: "star a-{b,c,d}".into(), output: "a=3.0".into() },
        ],
    }
}

#[test]
fn a_well_declared_primitive_is_valid() {
    assert!(validate_primitive(&centrality()).is_empty());
}

#[test]
fn a_primitive_without_an_oracle_is_rejected() {
    let mut p = centrality();
    p.oracle.clear();
    assert!(validate_primitive(&p).iter().any(|x| x.message.contains("oracle pair")));
}

#[test]
fn a_primitive_without_a_reference_is_rejected() {
    let mut p = centrality();
    p.reference.clear();
    assert!(validate_primitive(&p).iter().any(|x| x.path == "reference"));
}

#[test]
fn matching_outputs_are_oracle_conformant() {
    assert!(check_oracle(&centrality(), &["b=1.0".into(), "a=3.0".into()]).is_empty());
}

#[test]
fn a_mismatched_output_fails_oracle_conformance() {
    let vs = check_oracle(&centrality(), &["b=1.0".into(), "a=2.0".into()]);
    assert!(vs.iter().any(|x| x.message.contains("pair 1") && x.message.contains("expected 'a=3.0'")));
}

#[test]
fn the_wrong_output_count_fails() {
    assert!(check_oracle(&centrality(), &["b=1.0".into()]).iter().any(|x| x.message.contains("output(s) for")));
}
