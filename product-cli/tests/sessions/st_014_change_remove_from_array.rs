//! ST-014 — change removes a value from an array field.
//!
//! Validates TC-669. The `remove` op is the inverse of `append`;
//! together they let agents reshape array fields without hand-editing
//! front-matter.

use super::harness::Session;

/// TC-669 — session ST-014 change-remove-from-array.
#[test]
fn tc_669_session_st_014_change_remove_from_array() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-014 — seed feature with two domains"
artifacts:
  - type: feature
    ref: ft-main
    title: Main Feature
    phase: 1
    domains: [api, security]
"#,
    );
    created.assert_applied();
    let ft = created.id_for("ft-main");
    let ft_file = format!("docs/features/{}-main-feature.md", ft);

    s.assert_array_contains(&ft_file, "domains", "api");
    s.assert_array_contains(&ft_file, "domains", "security");

    let changed = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-014 — remove security domain"
changes:
  - target: {ft}
    mutations:
      - op: remove
        field: domains
        value: security
"#
    ));
    changed.assert_applied();

    s.assert_array_contains(&ft_file, "domains", "api");
    s.assert_array_missing(&ft_file, "domains", "security");
}
