//! Session tests for FT-062 — MCP parity for feature `depends-on` and
//! strict request shape validation.
//!
//! Each `#[test]` here corresponds to a TC under FT-062. The harness drives
//! the compiled `product` binary against a fresh tempdir repo for every
//! scenario, mirroring the existing FT-046 / FT-059 session pattern.

use super::harness::Session;

// ---------------------------------------------------------------------------
// TC-732 — product_schema and request validator share field allowlist
// ---------------------------------------------------------------------------

/// TC-732 — fitness invariant: the public field-allowlist constants and the
/// validator's `is_known_field` query share a single source of truth.
#[test]
fn tc_732_product_schema_and_request_validator_share_field_allowlist() {
    use product_core::field_schema;
    use product_core::request::types::ArtifactType;

    // Every constant entry must round-trip through the lookup function.
    for &f in field_schema::FEATURE_FIELDS {
        assert!(
            field_schema::is_known_field(ArtifactType::Feature, f),
            "feature field {} must be known",
            f
        );
    }
    for &f in field_schema::ADR_FIELDS {
        assert!(
            field_schema::is_known_field(ArtifactType::Adr, f),
            "adr field {} must be known",
            f
        );
    }
    for &f in field_schema::TC_FIELDS {
        assert!(
            field_schema::is_known_field(ArtifactType::Tc, f),
            "tc field {} must be known",
            f
        );
    }
    for &f in field_schema::DEP_FIELDS {
        assert!(
            field_schema::is_known_field(ArtifactType::Dep, f),
            "dep field {} must be known",
            f
        );
    }

    // The pseudo-field `body` is universally accepted (ADR-038 dec. 9).
    for at in [
        ArtifactType::Feature,
        ArtifactType::Adr,
        ArtifactType::Tc,
        ArtifactType::Dep,
    ] {
        assert!(field_schema::is_known_field(at, "body"));
    }

    // depends-on must be in the feature allowlist — the heart of FT-062.
    assert!(
        field_schema::FEATURE_FIELDS.contains(&"depends-on"),
        "depends-on must be in FEATURE_FIELDS"
    );

    // The `ArtifactType` lookup and the constant slice carry the same
    // names. (Rust does not guarantee identical pointers for separate uses
    // of the same `&'static`, so compare contents.)
    assert_eq!(
        field_schema::known_fields_for(ArtifactType::Feature),
        field_schema::FEATURE_FIELDS
    );
    assert_eq!(
        field_schema::known_fields_for(ArtifactType::Adr),
        field_schema::ADR_FIELDS
    );
    assert_eq!(
        field_schema::known_fields_for(ArtifactType::Tc),
        field_schema::TC_FIELDS
    );
    assert_eq!(
        field_schema::known_fields_for(ArtifactType::Dep),
        field_schema::DEP_FIELDS
    );

    // The label-based lookup also must agree.
    assert_eq!(
        field_schema::known_fields_for_label("feature"),
        field_schema::FEATURE_FIELDS
    );
}

// ---------------------------------------------------------------------------
// TC-733 — `product feature depends-on FT-X --add FT-Y` writes the edge
// ---------------------------------------------------------------------------

#[test]
fn tc_733_mcp_feature_depends_on_add_writes_edge() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-733 — seed two features"
artifacts:
  - type: feature
    ref: ft-a
    title: Feature A
    phase: 1
  - type: feature
    ref: ft-b
    title: Feature B
    phase: 1
"#,
    );
    created.assert_applied();
    let a = created.id_for("ft-a");
    let b = created.id_for("ft-b");
    let a_file = format!("docs/features/{}-feature-a.md", a);

    // First add — should change.
    let out = s.run(&[
        "--format",
        "json",
        "feature",
        "depends-on",
        &a,
        "--add",
        &b,
    ]);
    out.assert_exit(0);
    let json: serde_json::Value =
        serde_json::from_str(out.stdout.trim()).expect("valid JSON");
    assert_eq!(json["changed"], serde_json::Value::Bool(true));
    assert_eq!(json["added"][0], serde_json::Value::String(b.clone()));

    s.assert_array_contains(&a_file, "depends-on", &b);

    // Second add — idempotent, should be no-op.
    let out2 = s.run(&[
        "--format",
        "json",
        "feature",
        "depends-on",
        &a,
        "--add",
        &b,
    ]);
    out2.assert_exit(0);
    let json2: serde_json::Value =
        serde_json::from_str(out2.stdout.trim()).expect("valid JSON");
    assert_eq!(json2["changed"], serde_json::Value::Bool(false));

    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-734 — depends-on rejects cycle-creating add
// ---------------------------------------------------------------------------

#[test]
fn tc_734_mcp_feature_depends_on_rejects_cycle() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-734 — seed three-feature chain"
artifacts:
  - type: feature
    ref: ft-a
    title: Feature A
    phase: 1
  - type: feature
    ref: ft-b
    title: Feature B
    phase: 1
  - type: feature
    ref: ft-c
    title: Feature C
    phase: 1
  - type: feature
    ref: ft-x
    title: Feature X
    phase: 1
"#,
    );
    created.assert_applied();
    let a = created.id_for("ft-a");
    let b = created.id_for("ft-b");
    let c = created.id_for("ft-c");
    let x = created.id_for("ft-x");

    // Build the chain a -> b -> c.
    s.run(&["feature", "depends-on", &a, "--add", &b]).assert_exit(0);
    s.run(&["feature", "depends-on", &b, "--add", &c]).assert_exit(0);
    let pre_digest = s.docs_digest();

    // Now adding c -> a closes the cycle.
    let out = s.run(&["feature", "depends-on", &c, "--add", &a]);
    assert_ne!(out.exit_code, 0, "cycle add must fail");
    let combined = format!("{}{}", out.stderr, out.stdout);
    assert!(
        combined.contains("dependency cycle") || combined.contains("E003"),
        "expected cycle error, got: {}",
        combined
    );

    // Self-loop variant.
    let out2 = s.run(&["feature", "depends-on", &x, "--add", &x]);
    assert_ne!(out2.exit_code, 0, "self-loop add must fail");

    // Files unchanged.
    let post_digest = s.docs_digest();
    assert_eq!(
        pre_digest, post_digest,
        "files must be byte-identical after cycle rejection"
    );
}

// ---------------------------------------------------------------------------
// TC-735 — depends-on rejects unknown target
// ---------------------------------------------------------------------------

#[test]
fn tc_735_mcp_feature_depends_on_rejects_broken_link() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-735 — seed lone feature"
artifacts:
  - type: feature
    ref: ft-only
    title: Only Feature
    phase: 1
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-only");
    let pre_digest = s.docs_digest();

    let out = s.run(&[
        "feature",
        "depends-on",
        &id,
        "--add",
        "FT-DOES-NOT-EXIST",
    ]);
    assert_ne!(out.exit_code, 0, "broken-link add must fail");

    let post_digest = s.docs_digest();
    assert_eq!(pre_digest, post_digest, "files must be unchanged");
}

// ---------------------------------------------------------------------------
// TC-736 — `feature link --dep` (and the equivalent MCP `feature` arg)
// adds the depends-on edge through the same plan helper.
// ---------------------------------------------------------------------------

#[test]
fn tc_736_mcp_feature_link_feature_arg_adds_edge() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-736 — seed two features"
artifacts:
  - type: feature
    ref: ft-a
    title: Feature A
    phase: 1
  - type: feature
    ref: ft-b
    title: Feature B
    phase: 1
"#,
    );
    created.assert_applied();
    let a = created.id_for("ft-a");
    let b = created.id_for("ft-b");
    let a_file = format!("docs/features/{}-feature-a.md", a);

    // The CLI `feature link --dep` shares the cycle-checked depends-on path
    // with the MCP `feature` argument. Both are wired to
    // `plan_depends_on_edit` after FT-062.
    s.run(&["feature", "link", &a, "--dep", &b]).assert_exit(0);
    s.assert_array_contains(&a_file, "depends-on", &b);

    // Idempotent — second invocation does not duplicate.
    s.run(&["feature", "link", &a, "--dep", &b]).assert_exit(0);
    let body = s.read(&a_file);
    let count = body.matches(&format!("- {}", b)).count();
    assert!(count <= 1, "expected at most one occurrence of {}, got {}", b, count);

    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-737 — CLI feature depends-on mirrors the MCP tool
// ---------------------------------------------------------------------------

#[test]
fn tc_737_cli_feature_depends_on_mirrors_mcp_tool() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-737 — seed three features"
artifacts:
  - type: feature
    ref: ft-a
    title: Feature A
    phase: 1
  - type: feature
    ref: ft-b
    title: Feature B
    phase: 1
  - type: feature
    ref: ft-c
    title: Feature C
    phase: 1
"#,
    );
    created.assert_applied();
    let a = created.id_for("ft-a");
    let b = created.id_for("ft-b");
    let c = created.id_for("ft-c");
    let a_file = format!("docs/features/{}-feature-a.md", a);

    // Multi-add.
    s.run(&[
        "feature",
        "depends-on",
        &a,
        "--add",
        &b,
        "--add",
        &c,
    ])
    .assert_exit(0);
    s.assert_array_contains(&a_file, "depends-on", &b);
    s.assert_array_contains(&a_file, "depends-on", &c);

    // Remove one, leave the other.
    s.run(&["feature", "depends-on", &a, "--remove", &b])
        .assert_exit(0);
    s.assert_array_missing(&a_file, "depends-on", &b);
    s.assert_array_contains(&a_file, "depends-on", &c);

    // Unknown target fails.
    let pre = s.docs_digest();
    let out = s.run(&[
        "feature",
        "depends-on",
        &a,
        "--add",
        "FT-DOES-NOT-EXIST",
    ]);
    assert_ne!(out.exit_code, 0);
    assert_eq!(pre, s.docs_digest(), "unknown target left files unchanged");

    // Self-loop fails.
    let out = s.run(&["feature", "depends-on", &a, "--add", &a]);
    assert_ne!(out.exit_code, 0);

    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-738 — request rejects unknown top-level key with E025
// ---------------------------------------------------------------------------

#[test]
fn tc_738_request_rejects_unknown_top_level_key_with_e025() {
    let mut s = Session::new();
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-738 — seed feature"
artifacts:
  - type: feature
    ref: ft-only
    title: Only Feature
    phase: 1
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-only");
    let pre = s.docs_digest();

    // A request with a `patch:` wrapper (unknown top-level key) plus a
    // valid `changes:` array. The unknown key alone must be enough to
    // reject the entire request — silent acceptance was the bug.
    let yaml = format!(
        r#"type: change
schema-version: 1
reason: "TC-738 patch wrapper"
patch:
  target: {id}
  mutations:
    - op: append
      field: domains
      value: api
changes:
  - target: {id}
    mutations:
      - op: append
        field: domains
        value: api
"#
    );

    let validate = s.validate(&yaml);
    validate.assert_failed();
    validate.assert_finding("E025");
    assert!(
        validate.findings.iter().any(|f| f.location == "$.patch"),
        "E025 finding must point to $.patch, got: {:?}",
        validate.findings
    );

    let apply = s.apply(&yaml);
    apply.assert_failed();
    apply.assert_finding("E025");

    let post = s.docs_digest();
    assert_eq!(pre, post, "rejected request must leave docs/ unchanged");
}

// ---------------------------------------------------------------------------
// TC-739 — request rejects unknown mutation field with E026
// ---------------------------------------------------------------------------

#[test]
fn tc_739_request_rejects_unknown_mutation_field_with_e026() {
    let mut s = Session::new();
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-739 — seed feature"
artifacts:
  - type: feature
    ref: ft-only
    title: Only Feature
    phase: 1
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-only");
    let pre = s.docs_digest();

    // dependsOn (camelCase) is the canonical typo this feature exists to
    // catch. The validator must surface E026 with a Levenshtein-2 hint
    // suggesting "depends-on".
    let yaml = format!(
        r#"type: change
schema-version: 1
reason: "TC-739 camelCase typo"
changes:
  - target: {id}
    mutations:
      - op: append
        field: dependsOn
        value: FT-002
"#
    );

    let apply = s.apply(&yaml);
    apply.assert_failed();
    apply.assert_finding("E026");
    let f = apply
        .findings
        .iter()
        .find(|f| f.code == "E026")
        .expect("E026 present");
    assert!(
        f.message.contains("depends-on"),
        "E026 must suggest depends-on, got: {}",
        f.message
    );
    assert!(
        f.location.starts_with("$.changes[0].mutations[0]"),
        "E026 location must point at the mutation, got: {}",
        f.location
    );

    let post = s.docs_digest();
    assert_eq!(pre, post, "rejected request must leave docs/ unchanged");
}

// ---------------------------------------------------------------------------
// TC-740 — request accepts dot-notation on a known head field
// ---------------------------------------------------------------------------

#[test]
fn tc_740_request_accepts_dot_notation_on_known_head() {
    let mut s = Session::new();
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-740 — seed feature with api domain"
artifacts:
  - type: feature
    ref: ft-only
    title: Only Feature
    phase: 1
    domains: [api]
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-only");
    let ft_file = format!("docs/features/{}-only-feature.md", id);

    let yaml = format!(
        r#"type: change
schema-version: 1
reason: "TC-740 acknowledge security"
changes:
  - target: {id}
    mutations:
      - op: set
        field: domains-acknowledged.security
        value: "No new trust boundaries introduced."
"#
    );

    let apply = s.apply(&yaml);
    apply.assert_applied();
    apply.assert_no_finding("E026");

    let body = s.read(&ft_file);
    assert!(
        body.contains("security: No new trust boundaries introduced.")
            || body.contains("security: \"No new trust boundaries introduced.\""),
        "front-matter must carry the acknowledgement; got:\n{}",
        body
    );

    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-741 — FT-062 exit criteria
// ---------------------------------------------------------------------------

/// TC-741 — exit-criteria roll-up for FT-062. Anchors the feature on a
/// single test that exercises the happy path end-to-end: create features,
/// add a depends-on edge via the new CLI command, send a request with a
/// dot-notation acknowledgement, ensure graph health is clean.
#[test]
fn tc_741_ft_062_exit_criteria() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-741 — exit criteria seed"
artifacts:
  - type: feature
    ref: ft-a
    title: Feature A
    phase: 1
  - type: feature
    ref: ft-b
    title: Feature B
    phase: 1
"#,
    );
    created.assert_applied();
    let a = created.id_for("ft-a");
    let b = created.id_for("ft-b");
    let a_file = format!("docs/features/{}-feature-a.md", a);

    // Granular setter writes the depends-on edge.
    s.run(&["feature", "depends-on", &a, "--add", &b])
        .assert_exit(0);
    s.assert_array_contains(&a_file, "depends-on", &b);

    // Unknown top-level key is rejected (E025).
    let bad_top = format!(
        r#"type: change
schema-version: 1
reason: "TC-741 unknown top key"
totally-bogus:
  no: thing
changes:
  - target: {a}
    mutations:
      - op: append
        field: domains
        value: api
"#
    );
    s.apply(&bad_top).assert_finding("E025");

    // Unknown mutation field is rejected (E026).
    let bad_field = format!(
        r#"type: change
schema-version: 1
reason: "TC-741 unknown mutation field"
changes:
  - target: {a}
    mutations:
      - op: append
        field: dependsOn
        value: {b}
"#
    );
    s.apply(&bad_field).assert_finding("E026");

    // Sanity: graph still clean after both rejections.
    s.assert_graph_clean();
}
