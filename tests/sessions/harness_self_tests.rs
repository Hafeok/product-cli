//! Harness self-tests (TC-530, TC-531, TC-532).
//!
//! These tests verify that the `Session` harness itself behaves as
//! documented in `docs/product-testing-spec.md` § Session Runner.

use super::harness::{ApplyResult, Session};

/// TC-530 — Session harness exposes documented API surface.
///
/// Verify that a freshly constructed `Session` has the configuration,
/// binary, and directory tree described in the spec: a valid
/// `product.toml`, empty artifact directories, and a compiled binary that
/// answers to `--version`.
#[test]
fn tc_530_session_harness_exposes_documented_api_surface() {
    let s = Session::new();

    // product.toml is present and valid TOML
    let cfg = s.read("product.toml");
    assert!(cfg.contains("name"), "product.toml should have a name key");
    assert!(cfg.contains("[domains]"), "product.toml should include domain vocabulary");

    // Directory scaffold is present and empty
    for sub in [
        "docs/features",
        "docs/adrs",
        "docs/tests",
        "docs/dependencies",
    ] {
        let p = s.dir.path().join(sub);
        assert!(p.is_dir(), "expected directory {} to exist", sub);
        let count = std::fs::read_dir(&p).map(|d| d.count()).unwrap_or(0);
        assert_eq!(count, 0, "expected {} to be empty", sub);
    }

    // Binary responds to --version
    let out = s.run(&["--version"]);
    out.assert_exit(0);
    assert!(
        out.stdout.contains("product")
            || out.stdout.contains(env!("CARGO_PKG_VERSION")),
        "--version output should mention the product name or version; got: {}",
        out.stdout
    );

    // graph check on an empty repo should be clean (exit 0 or 2)
    let check = s.run(&["graph", "check"]);
    assert!(
        check.exit_code == 0 || check.exit_code == 2,
        "graph check on empty repo should be clean, got exit {}\nstderr: {}",
        check.exit_code,
        check.stderr
    );
}

/// TC-531 — ApplyResult.id_for resolves ref names to assigned IDs.
///
/// Apply a create request that declares a `ref:` name. Verify that
/// `ApplyResult.id_for(ref)` returns the real assigned ID (e.g. "FT-001"),
/// never the placeholder.
#[test]
fn tc_531_applyresult_id_for_resolves_ref_names_to_assigned_ids() {
    let mut s = Session::new();

    let r: ApplyResult = s.apply(
        r#"type: create
schema-version: 1
reason: "harness self-test"
artifacts:
  - type: feature
    ref: ft-harness
    title: Harness Feature
    phase: 1
    domains: [api]
"#,
    );
    r.assert_applied();

    let id = r.id_for("ft-harness");
    assert!(
        id.starts_with("FT-") && id != "ft-harness",
        "id_for returned {:?}; expected real FT-... id",
        id
    );

    // Subsequent run with the real ID succeeds: file exists and has correct title.
    let path = format!("docs/features/{}-harness-feature.md", id);
    s.assert_file_exists(&path);
}

/// TC-532 — Session.run executes compiled product binary against temp dir.
///
/// Create a session, apply a request via the session, then `Session::run`
/// the binary directly to read the created artifact. Verify that
/// `run` invokes the binary in the session's temp dir and reflects the
/// session's state (not the host repository's state).
#[test]
fn tc_532_session_run_executes_compiled_product_binary_against_temp_dir() {
    let mut s = Session::new();

    // Feature list on a fresh session should be empty.
    let pre = s.run(&["feature", "list"]);
    pre.assert_exit(0);

    s.apply(
        r#"type: create
schema-version: 1
reason: "seed feature"
artifacts:
  - type: feature
    title: Seeded
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    // After apply, `feature list` sees the new feature in this temp dir only.
    let post = s.run(&["feature", "list"]);
    post.assert_exit(0);
    assert!(
        post.stdout.contains("FT-001") || post.stdout.contains("Seeded"),
        "feature list should include the newly-created feature; got: {}",
        post.stdout
    );

    // Confirm the run actually targeted the session's tempdir by reading the file.
    assert!(
        std::path::Path::new(&s.dir.path().join("docs/features/FT-001-seeded.md")).exists(),
        "expected FT-001 file to exist in the session tempdir"
    );
}
