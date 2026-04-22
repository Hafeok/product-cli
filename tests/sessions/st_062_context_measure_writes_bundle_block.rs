//! ST-062 — `product context <FT> --measure` writes a `bundle:` block to the
//! feature's front-matter with `depth-1-adrs`, `tcs`, `tokens-approx`, and
//! `measured-at` fields (FT-040).
//!
//! Validates TC-680.

use super::harness::Session;

/// TC-680 — session ST-062 context-measure-writes-bundle-block.
#[test]
fn tc_680_session_st_062_context_measure_writes_bundle_block() {
    let mut s = Session::new();

    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-062 — feature + ADR + TC to measure"
artifacts:
  - type: adr
    ref: adr-core
    title: Core Decision
    domains: [api]
  - type: feature
    ref: ft-main
    title: Measured Feature
    phase: 1
    domains: [api]
    adrs: [ref:adr-core]
    tests: [ref:tc-one]
  - type: tc
    ref: tc-one
    title: One Test
    tc-type: scenario
    validates:
      features: [ref:ft-main]
"#,
    );
    r.assert_applied();
    let ft = r.id_for("ft-main");
    let ft_file = format!("docs/features/{}-measured-feature.md", ft);

    // No bundle block before --measure runs.
    let before = s.read(&ft_file);
    assert!(
        !before.contains("bundle:"),
        "feature should not have a bundle block before --measure; got:\n{before}"
    );

    let out = s.run(&["context", &ft, "--measure"]);
    out.assert_exit(0);

    let after = s.read(&ft_file);
    assert!(
        after.contains("bundle:"),
        "expected bundle block after --measure; got:\n{after}"
    );
    for field in ["depth-1-adrs:", "tcs:", "tokens-approx:", "measured-at:"] {
        assert!(
            after.contains(field),
            "expected '{field}' inside the bundle block; got:\n{after}"
        );
    }
}
