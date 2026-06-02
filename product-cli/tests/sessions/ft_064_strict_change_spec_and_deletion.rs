//! Session tests for FT-064 — Strict change-spec validation and artifact
//! deletion surface.
//!
//! Each `#[test]` here corresponds to a TC under FT-064. The harness drives
//! the compiled `product` binary against a fresh tempdir repo for every
//! scenario, mirroring the existing FT-062 session pattern.

use super::harness::Session;

// ---------------------------------------------------------------------------
// TC-770 — request rejects unknown keys on a change block (E025)
// ---------------------------------------------------------------------------

#[test]
fn tc_770_request_rejects_unknown_keys_on_a_change_block() {
    let mut s = Session::new();

    // Seed FT-001 carrying a stub `tests:` list so the user's reported
    // shape (op/field/value at the change level instead of inside
    // mutations:) targets a real list-valued field.
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-770 — seed feature"
artifacts:
  - type: feature
    ref: ft-target
    title: Target Feature
    phase: 1
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-target");
    let pre = s.docs_digest();

    // The pathological YAML: `op:` / `field:` / `value:` declared at the
    // change level instead of nested inside `mutations:`. Before FT-064
    // this validated clean and applied with `mutations: 0` — the headline
    // bug. After FT-064 it must surface E025 (one per misplaced key) and
    // refuse to write anything.
    let yaml = format!(
        r#"type: change
schema-version: 1
reason: "TC-770 — misplaced mutation fields"
changes:
  - target: {id}
    op: remove
    field: tests
    value: TC-002
"#
    );

    let validate = s.validate(&yaml);
    validate.assert_failed();
    validate.assert_finding("E025");
    // The validator must report every offender in one pass — the user
    // shouldn't fix one key, re-run, fix the next.
    let e025_count = validate
        .findings
        .iter()
        .filter(|f| f.code == "E025")
        .count();
    assert!(
        e025_count >= 3,
        "expected E025 for each of op/field/value (>=3), got {}: {:?}",
        e025_count,
        validate.findings.iter().map(|f| (f.code.as_str(), f.location.as_str())).collect::<Vec<_>>()
    );

    // Locations are JSONPath-correct.
    let locations: Vec<String> =
        validate.findings.iter().map(|f| f.location.clone()).collect();
    assert!(
        locations.iter().any(|l| l == "$.changes[0].op"),
        "expected $.changes[0].op in {:?}",
        locations
    );
    assert!(
        locations.iter().any(|l| l == "$.changes[0].field"),
        "expected $.changes[0].field in {:?}",
        locations
    );
    assert!(
        locations.iter().any(|l| l == "$.changes[0].value"),
        "expected $.changes[0].value in {:?}",
        locations
    );

    // The apply path must reject it too, with zero files touched.
    let apply = s.apply(&yaml);
    apply.assert_failed();
    apply.assert_finding("E025");
    let post = s.docs_digest();
    assert_eq!(pre, post, "rejected request must leave docs/ unchanged");
}

// ---------------------------------------------------------------------------
// TC-771 — request rejects change with empty mutations list (E006)
// ---------------------------------------------------------------------------

#[test]
fn tc_771_request_rejects_change_with_empty_mutations_list() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-771 — seed feature"
artifacts:
  - type: feature
    ref: ft-target
    title: Target Feature
    phase: 1
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-target");
    let pre = s.docs_digest();

    // Empty mutations list.
    let yaml_empty = format!(
        r#"type: change
schema-version: 1
reason: "TC-771 — empty mutations"
changes:
  - target: {id}
    mutations: []
"#
    );

    let apply = s.apply(&yaml_empty);
    apply.assert_failed();
    apply.assert_finding("E006");
    let post = s.docs_digest();
    assert_eq!(pre, post, "rejected request must leave docs/ unchanged");

    // Missing mutations: key (defaults to empty) — same rejection.
    let yaml_missing = format!(
        r#"type: change
schema-version: 1
reason: "TC-771 — missing mutations key"
changes:
  - target: {id}
"#
    );
    let apply2 = s.apply(&yaml_missing);
    apply2.assert_failed();
    apply2.assert_finding("E006");
    let post2 = s.docs_digest();
    assert_eq!(pre, post2, "rejected request must leave docs/ unchanged");
}

// ---------------------------------------------------------------------------
// TC-772 — request rejects unknown keys on a mutation block (E025)
// ---------------------------------------------------------------------------

#[test]
fn tc_772_request_rejects_unknown_keys_on_a_mutation_block() {
    let mut s = Session::new();

    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-772 — seed feature"
artifacts:
  - type: feature
    ref: ft-target
    title: Target Feature
    phase: 1
"#,
    );
    created.assert_applied();
    let id = created.id_for("ft-target");
    let pre = s.docs_digest();

    // `path:` / `to:` are not in the closed mutation key set
    // {op, field, value}.
    let yaml = format!(
        r#"type: change
schema-version: 1
reason: "TC-772 — unknown mutation keys"
changes:
  - target: {id}
    mutations:
      - op: append
        field: domains
        value: api
        path: "/whatever"
        to: "somewhere"
"#
    );

    let apply = s.apply(&yaml);
    apply.assert_failed();
    apply.assert_finding("E025");

    let bad_locations: Vec<String> = apply
        .findings
        .iter()
        .filter(|f| f.code == "E025")
        .map(|f| f.location.clone())
        .collect();
    assert!(
        bad_locations.iter().any(|l| l.contains("path")),
        "expected E025 for 'path', got: {:?}",
        bad_locations
    );
    assert!(
        bad_locations.iter().any(|l| l.contains("to")),
        "expected E025 for 'to', got: {:?}",
        bad_locations
    );

    let post = s.docs_digest();
    assert_eq!(pre, post, "rejected request must leave docs/ unchanged");
}

// ---------------------------------------------------------------------------
// TC-773 — op:remove on list-valued feature field removes the entry
// ---------------------------------------------------------------------------

#[test]
fn tc_773_request_op_remove_on_list_valued_field_removes_the_entry() {
    let mut s = Session::new();

    // Seed a feature whose tests: list will be mutated. Create a couple
    // of TCs and link them at create time so we have a deterministic
    // starting state.
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-773 — seed feature with two TCs"
artifacts:
  - type: feature
    ref: ft-main
    title: Main Feature
    phase: 1
    tests: [ref:tc-a, ref:tc-b]
  - type: tc
    ref: tc-a
    title: Test A
    validates:
      features: [ref:ft-main]
    phase: 1
    runner: cargo-test
    runner-args: "tc_a"
  - type: tc
    ref: tc-b
    title: Test B
    validates:
      features: [ref:ft-main]
    phase: 1
    runner: cargo-test
    runner-args: "tc_b"
"#,
    );
    created.assert_applied();
    let ft = created.id_for("ft-main");
    let tc_a = created.id_for("tc-a");
    let tc_b = created.id_for("tc-b");
    let ft_file = format!("docs/features/{}-main-feature.md", ft);

    s.assert_array_contains(&ft_file, "tests", &tc_a);
    s.assert_array_contains(&ft_file, "tests", &tc_b);

    // Apply the (well-formed) remove mutation.
    let yaml = format!(
        r#"type: change
schema-version: 1
reason: "TC-773 — remove TC-B from feature.tests"
changes:
  - target: {ft}
    mutations:
      - op: remove
        field: tests
        value: {tc_b}
"#
    );

    let changed = s.apply(&yaml);
    changed.assert_applied();

    // The apply summary reports >=1 mutation. The earlier silent bug
    // surfaced as `mutations: 0` despite "successful" apply.
    let mutations = changed
        .changed
        .iter()
        .find(|c| c.id == ft)
        .map(|c| c.mutations)
        .unwrap_or(0);
    assert!(
        mutations >= 1,
        "apply summary must report >=1 mutation for the feature, got {}",
        mutations
    );

    s.assert_array_contains(&ft_file, "tests", &tc_a);
    s.assert_array_missing(&ft_file, "tests", &tc_b);
}

// ---------------------------------------------------------------------------
// TC-774 — MCP and CLI expose an artifact-deletion operation
// ---------------------------------------------------------------------------

#[test]
fn tc_774_mcp_and_cli_expose_an_artifact_deletion_operation() {
    let mut s = Session::new();

    // Create a leaf artifact — an orphan TC with no inbound links so
    // deletion validation passes without cascade.
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-774 — seed throwaway TC"
artifacts:
  - type: tc
    ref: tc-doomed
    title: Doomed Test
    phase: 1
    runner: cargo-test
    runner-args: "tc_doomed"
"#,
    );
    created.assert_applied();
    let tc = created.id_for("tc-doomed");
    let tc_file = format!("docs/tests/{}-doomed-test.md", tc);
    s.assert_file_exists(&tc_file);

    // Round 1 — exercise the CLI form: `product request delete <ID>
    // --reason "..."`. This is the convenience wrapper that builds the
    // YAML in memory and runs it through the same apply pipeline.
    let out = s.run(&[
        "--format",
        "json",
        "request",
        "delete",
        &tc,
        "--reason",
        "TC-774 — drop obsolete TC",
    ]);
    out.assert_exit(0);
    s.assert_file_missing(&tc_file);

    // Round 2 — apply a hand-written `type: delete` request YAML to prove
    // the request shape works end-to-end through `product request apply`.
    let created2 = s.apply(
        r#"type: create
schema-version: 1
reason: "TC-774 — seed second throwaway TC"
artifacts:
  - type: tc
    ref: tc-doomed-2
    title: Doomed Test 2
    phase: 1
    runner: cargo-test
    runner-args: "tc_doomed_2"
"#,
    );
    created2.assert_applied();
    let tc2 = created2.id_for("tc-doomed-2");
    let tc2_file = format!("docs/tests/{}-doomed-test-2.md", tc2);
    s.assert_file_exists(&tc2_file);

    let delete_yaml = format!(
        r#"type: delete
schema-version: 1
reason: "TC-774 — drop second TC via YAML"
deletions:
  - target: {tc2}
"#
    );
    let apply = s.apply(&delete_yaml);
    apply.assert_applied();
    s.assert_file_missing(&tc2_file);

    // Graph is clean after deletion.
    s.assert_graph_clean();

    // The deletion is recorded in requests.jsonl (FT-042 / hash-chained
    // log). `product request log show` lists at least one `delete` entry.
    let log = s.run(&["request", "log", "show", "--type-filter", "delete"]);
    log.assert_exit(0);
    let combined = format!("{}{}", log.stdout, log.stderr);
    assert!(
        combined.contains("delete"),
        "expected `request log show --type delete` to list at least one entry, got:\nstdout: {}\nstderr: {}",
        log.stdout, log.stderr
    );

    // Verifying the log chain succeeds — the delete entry hashes link.
    let verify = s.run(&["request", "log", "verify"]);
    assert!(
        verify.exit_code == 0,
        "log verify must succeed after delete, got exit={} stdout:\n{}\nstderr:\n{}",
        verify.exit_code, verify.stdout, verify.stderr
    );
}

// ---------------------------------------------------------------------------
// TC-775 — FT-064 exit criteria
// ---------------------------------------------------------------------------

/// Consolidated exit-criteria check for FT-064. The individual capability
/// TCs (TC-770..TC-774) carry the detailed assertions; this TC threads a
/// minimal end-to-end flow through every capability in one session.
#[test]
fn tc_775_ft064_exit_criteria_strict_shape_and_deletion_work_end_to_end() {
    let mut s = Session::new();

    // Capability 1 — strict change-shape rejects mis-shaped mutation
    // (TC-770).
    let mis_shaped = r#"type: change
schema-version: 1
reason: "FT-064 EC — mis-shaped"
changes:
  - target: FT-001
    op: remove
    field: tests
    value: TC-002
"#;
    let a = s.apply(mis_shaped);
    a.assert_failed();
    a.assert_finding("E025");

    // Capability 2 — empty mutations rejected (TC-771).
    let empty = r#"type: change
schema-version: 1
reason: "FT-064 EC — empty mutations"
changes:
  - target: FT-001
    mutations: []
"#;
    let b = s.apply(empty);
    b.assert_failed();
    b.assert_finding("E006");

    // Capability 3 — unknown mutation key rejected (TC-772).
    let bad_mutation = r#"type: change
schema-version: 1
reason: "FT-064 EC — unknown mutation key"
changes:
  - target: FT-001
    mutations:
      - op: append
        field: domains
        value: api
        path: "/oops"
"#;
    let c = s.apply(bad_mutation);
    c.assert_failed();
    c.assert_finding("E025");

    // Capability 4 — op:remove actually removes the entry (TC-773).
    let created = s.apply(
        r#"type: create
schema-version: 1
reason: "FT-064 EC — seed feature with domain"
artifacts:
  - type: feature
    ref: ft-ec
    title: EC Feature
    phase: 1
    domains: [api, security]
"#,
    );
    created.assert_applied();
    let ft = created.id_for("ft-ec");
    let ft_file = format!("docs/features/{}-ec-feature.md", ft);

    let removed = s.apply(&format!(
        r#"type: change
schema-version: 1
reason: "FT-064 EC — remove security domain"
changes:
  - target: {ft}
    mutations:
      - op: remove
        field: domains
        value: security
"#
    ));
    removed.assert_applied();
    let mutations = removed
        .changed
        .iter()
        .find(|c| c.id == ft)
        .map(|c| c.mutations)
        .unwrap_or(0);
    assert!(mutations >= 1, "expected mutations >= 1, got {}", mutations);
    s.assert_array_missing(&ft_file, "domains", "security");

    // Capability 5 — artifact deletion round-trips through the request
    // interface and lands in the hash-chained log (TC-774).
    let created_tc = s.apply(
        r#"type: create
schema-version: 1
reason: "FT-064 EC — seed throwaway TC"
artifacts:
  - type: tc
    ref: tc-throwaway
    title: Throwaway Test
    phase: 1
    runner: cargo-test
    runner-args: "tc_throwaway"
"#,
    );
    created_tc.assert_applied();
    let tc = created_tc.id_for("tc-throwaway");
    let tc_file = format!("docs/tests/{}-throwaway-test.md", tc);
    s.assert_file_exists(&tc_file);

    let delete_yaml = format!(
        r#"type: delete
schema-version: 1
reason: "FT-064 EC — delete throwaway"
deletions:
  - target: {tc}
"#
    );
    let deleted = s.apply(&delete_yaml);
    deleted.assert_applied();
    s.assert_file_missing(&tc_file);

    s.assert_graph_clean();

    // The log verify still passes — delete entries link correctly.
    let verify = s.run(&["request", "log", "verify"]);
    assert_eq!(
        verify.exit_code, 0,
        "log verify must succeed at the end of FT-064 EC: stdout={} stderr={}",
        verify.stdout, verify.stderr
    );
}
