//! ST-010 — change appends a value to the domains array.
//!
//! Validates TC-665. The canonical change-op session: create a feature,
//! then issue a second request that appends a domain to it.

use super::harness::Session;

/// TC-665 — session ST-010 change-append-domain.
#[test]
fn tc_665_session_st_010_change_append_domain() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-010 — seed feature"
artifacts:
  - type: feature
    ref: ft-main
    title: Main Feature
    phase: 1
    domains: [api]
"#,
    );
    created.assert_applied();
    let ft = created.id_for("ft-main");
    let ft_file = format!("docs/features/{}-main-feature.md", ft);

    let changed = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-010 — append security domain"
changes:
  - target: {ft}
    mutations:
      - op: append
        field: domains
        value: security
"#
    ));
    changed.assert_applied();
    assert_eq!(changed.changed.len(), 1);

    s.assert_array_contains(&ft_file, "domains", "api");
    s.assert_array_contains(&ft_file, "domains", "security");
}
