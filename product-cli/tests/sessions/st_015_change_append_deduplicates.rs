//! ST-015 — appending an existing value is idempotent.
//!
//! Validates TC-670. Appending the same value twice must not produce a
//! duplicate entry; the request pipeline deduplicates at apply time so
//! retried requests converge on the same state.

use super::harness::Session;

/// TC-670 — session ST-015 change-append-deduplicates.
#[test]
fn tc_670_session_st_015_change_append_deduplicates() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "ST-015 — seed feature with api domain"
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

    // Append 'api' (already present) — second apply should be idempotent.
    let first = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-015 — append api again"
changes:
  - target: {ft}
    mutations:
      - op: append
        field: domains
        value: api
"#
    ));
    first.assert_applied();

    let second = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "ST-015 — append api a third time"
changes:
  - target: {ft}
    mutations:
      - op: append
        field: domains
        value: api
"#
    ));
    second.assert_applied();

    // domains should still contain exactly one 'api'.
    let body = s.read(&ft_file);
    let api_count = body
        .lines()
        .filter(|l| l.trim() == "- api" || l.trim() == "api")
        .count();
    assert_eq!(
        api_count, 1,
        "expected exactly one 'api' entry in domains after duplicate appends; got {api_count} in:\n{body}"
    );
}
