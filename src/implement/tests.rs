//! Unit tests for agent orchestration (ADR-021)

use super::verify::extract_yaml_field;

#[test]
fn extract_runner_field() {
    let content = "---\nid: TC-001\nrunner: cargo-test\nrunner-args: [\"--test\", \"foo\"]\n---\n";
    assert_eq!(extract_yaml_field(content, "runner"), "cargo-test");
}

#[test]
fn extract_missing_field() {
    let content = "---\nid: TC-001\n---\n";
    assert_eq!(extract_yaml_field(content, "runner"), "");
}
