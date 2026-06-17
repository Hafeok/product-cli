//! Tests for the worker capability catalog + role resolution.

use super::*;

fn catalog() -> Catalog {
    let caps = Catalog::capabilities_from_yaml(
        "capabilities:\n- id: code-writer\n  endpoint: litellm\n  model_identifier: qwen3-coder\n  tier: 1\n- id: deep-reasoning\n  endpoint: litellm\n  model_identifier: anthropic/claude-opus\n  tier: 3\n",
    ).expect("caps");
    let bindings = Catalog::role_bindings_from_yaml(
        "role_bindings:\n- role_id: implementer\n  default_capability: code-writer\n  escalation_steps:\n  - capability: deep-reasoning\n    triggers:\n    - prior_attempts_ge_5\n    - stakes_foundational\n  active: true\n",
    ).expect("bindings");
    Catalog { capabilities: caps, role_bindings: bindings }
}

#[test]
fn resolves_default_with_no_triggers() {
    let c = catalog();
    assert_eq!(c.resolve("implementer", &[]).expect("resolve").id, "code-writer");
}

#[test]
fn escalates_when_a_trigger_fires() {
    let c = catalog();
    let cap = c.resolve("implementer", &["stakes_foundational".to_string()]).expect("resolve");
    assert_eq!(cap.id, "deep-reasoning");
    assert_eq!(cap.tier, 3);
}

#[test]
fn unknown_role_resolves_to_nothing() {
    assert!(catalog().resolve("ghost", &[]).is_none());
}

#[test]
fn validate_flags_a_dangling_capability() {
    let mut c = catalog();
    c.role_bindings[0].default_capability = "missing".into();
    assert!(validate_catalog(&c).iter().any(|v| v.path == "default_capability"));
}

#[test]
fn validate_flags_an_unknown_trigger() {
    let mut c = catalog();
    c.role_bindings[0].escalation_steps[0].triggers = vec!["nonsense".into()];
    assert!(validate_catalog(&c).iter().any(|v| v.path == "triggers"));
}

#[test]
fn a_conformant_catalog_validates_clean() {
    assert!(validate_catalog(&catalog()).is_empty());
}

#[test]
fn validate_flags_an_unknown_endpoint() {
    let mut c = catalog();
    c.capabilities[0].endpoint = "bogus".into();
    assert!(validate_catalog(&c).iter().any(|v| v.path == "endpoint" && v.message.contains("bogus")));
}
