//! ST-013 — change replaces the markdown body below front-matter.
//!
//! Validates TC-668. The `body` field is a virtual field the request
//! pipeline maps to the prose region of the file, preserving the
//! front-matter block unchanged.

use super::harness::Session;

/// TC-668 — session ST-013 change-body-mutation.
#[test]
fn tc_668_session_st_013_change_body_mutation() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-013 — seed feature with placeholder body"
artifacts:
  - type: feature
    ref: ft-main
    title: Main Feature
    phase: 1
    domains: [api]
    body: "Original placeholder."
"#,
    );
    created.assert_applied();
    let ft = created.id_for("ft-main");
    let ft_file = format!("docs/features/{}-main-feature.md", ft);

    let before = s.read(&ft_file);
    assert!(before.contains("Original placeholder."));

    let new_body = "Rewritten body describing the feature's intent.";
    let changed = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-013 — rewrite body"
changes:
  - target: {ft}
    mutations:
      - op: set
        field: body
        value: "{new_body}"
"#
    ));
    changed.assert_applied();

    let after = s.read(&ft_file);
    assert!(
        after.contains(new_body),
        "expected new body text in {ft_file}; got:\n{after}"
    );
    assert!(
        !after.contains("Original placeholder."),
        "expected old body to be replaced; got:\n{after}"
    );
    // Front-matter preserved.
    s.assert_frontmatter(&ft_file, "id", &ft);
    s.assert_frontmatter(&ft_file, "title", "Main Feature");
}
