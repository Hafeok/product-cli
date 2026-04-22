//! ST-060 — `product context <FT>` renders a `## Dependencies` section
//! when the feature has a dependency linked through a governing ADR.
//!
//! Validates TC-678.

use super::harness::Session;

/// TC-678 — session ST-060 context-includes-dependency-section.
#[test]
fn tc_678_session_st_060_context_includes_dependency_section() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-060 — feature + governing ADR + dependency"
artifacts:
  - type: adr
    ref: adr-storage
    title: Storage Backend
    domains: [storage]
  - type: feature
    ref: ft-main
    title: Storage Feature
    phase: 1
    domains: [storage]
    adrs: [ref:adr-storage]
  - type: dep
    ref: dep-db
    title: ExampleDB
    dep-type: library
    version: "1.0"
    adrs: [ref:adr-storage]
"#,
    );
    r.assert_applied();
    let ft = r.id_for("ft-main");
    let dep = r.id_for("dep-db");

    // Accept the ADR so the feature is in a healthy state — not required for
    // rendering but avoids noisy warnings in the output.
    s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-060 — accept ADR"
changes:
  - target: {}
    mutations:
      - op: set
        field: status
        value: accepted
"#,
        r.id_for("adr-storage")
    ))
    .assert_applied();

    let out = s.run(&["context", &ft, "--depth", "2"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("## Dependencies"),
        "expected '## Dependencies' section in context output; got:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains(&dep),
        "expected the dependency id {dep} to appear in context output; got:\n{}",
        out.stdout
    );
    assert!(
        out.stdout.contains("ExampleDB"),
        "expected the dependency title 'ExampleDB' in context output; got:\n{}",
        out.stdout
    );
}
