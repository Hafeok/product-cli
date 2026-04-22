//! ST-011 — change sets a nested `domains-acknowledged.<ADR>` entry.
//!
//! Validates TC-666. Nested field set is how an agent records that a
//! cross-cutting ADR has been considered but deliberately not linked.

use super::harness::Session;

/// TC-666 — session ST-011 change-set-acknowledgement.
#[test]
fn tc_666_session_st_011_change_set_acknowledgement() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-011 — seed feature and ADR"
artifacts:
  - type: adr
    ref: adr-core
    title: Core Decision
    domains: [api]
    scope: cross-cutting
  - type: feature
    ref: ft-main
    title: Main Feature
    phase: 1
    domains: [api]
"#,
    );
    created.assert_applied();
    let ft = created.id_for("ft-main");
    let adr = created.id_for("adr-core");
    let ft_file = format!("docs/features/{}-main-feature.md", ft);

    let reason = "Predates the cross-cutting classification — no retroactive rework.";
    let changed = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-011 — acknowledge ADR"
changes:
  - target: {ft}
    mutations:
      - op: set
        field: domains-acknowledged.{adr}
        value: "{reason}"
"#
    ));
    changed.assert_applied();

    let body = s.read(&ft_file);
    assert!(
        body.contains(&format!("{adr}: {reason}"))
            || body.contains(&format!("{adr}: \"{reason}\"")),
        "expected front-matter to carry {adr}: {reason}; got:\n{body}"
    );
}
