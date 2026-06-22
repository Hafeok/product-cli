//! Unit tests for the §11.3 design-system manifest profile.

use super::*;
use crate::pf::model::{ContextOfUse, DomainGraph};

const WHOLE: &str = r#"
[design_system]
id = "acme"
version = "1.0"
wcag_target = "AA"
contexts_supported = ["phone"]
tokens = ["color.accent"]
[[components]]
id = "segmented-control"
tokens = ["color.accent"]
satisfies = [{ criterion = "1.3.1", level = "A", verification = "machine" }]

[[reification]]
aio = "single-select"
when = "phone"
cio = "segmented-control"
rationale = "thumb-reachable"
"#;

#[test]
fn whole_manifest_validates() {
    let m = parse_ds(WHOLE).expect("parses");
    assert!(validate_ds(&m).is_empty(), "should be whole: {:?}", validate_ds(&m));
}

#[test]
fn dangling_reification_cio_fails() {
    let src = WHOLE.replace("cio = \"segmented-control\"", "cio = \"ghost-cio\"");
    let m = parse_ds(&src).expect("parses");
    let f = validate_ds(&m);
    assert!(f.iter().any(|s| s.contains("ghost-cio") && s.contains("absent")), "{f:?}");
}

#[test]
fn undeclared_token_and_fake_criterion_fail() {
    let src = WHOLE
        .replace("tokens = [\"color.accent\"]\nsatisfies", "tokens = [\"color.ghost\"]\nsatisfies")
        .replace("criterion = \"1.3.1\"", "criterion = \"9.9.9\"");
    let m = parse_ds(&src).expect("parses");
    let f = validate_ds(&m);
    assert!(f.iter().any(|s| s.contains("color.ghost")), "token: {f:?}");
    assert!(f.iter().any(|s| s.contains("9.9.9")), "criterion: {f:?}");
}

#[test]
fn coupling_covers_core_aios_over_declared_contexts() {
    let m = parse_ds(WHOLE).expect("parses");
    let mut g = DomainGraph::default();
    g.contexts_of_use.push(ContextOfUse { id: "phone".into(), label: "P".into(), ..Default::default() });
    let f = couple_ds(&m, &g);
    // WHOLE only reifies single-select; the other 9 core AIOs on phone are gaps.
    assert!(f.iter().any(|s| s.contains("trigger-action") && s.contains("phone")), "{f:?}");
    assert!(!f.iter().any(|s| s.contains("single-select")), "single-select IS covered: {f:?}");
}
