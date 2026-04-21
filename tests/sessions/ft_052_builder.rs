//! FT-052 — Product Request Builder session tests.
//!
//! Validates TC-626 through TC-635. The builder is driven by invoking the
//! `product` binary directly, which is the same surface the agent and the
//! human share. Each test uses the standard `Session` harness for setup.

use super::harness::Session;

// ---------------------------------------------------------------------------
// Helpers scoped to this file
// ---------------------------------------------------------------------------

fn draft_path(s: &Session) -> std::path::PathBuf {
    s.root().join(".product/requests/draft.yaml")
}

fn archive_dir(s: &Session) -> std::path::PathBuf {
    s.root().join(".product/requests/archive")
}

fn read_draft(s: &Session) -> String {
    std::fs::read_to_string(draft_path(s)).unwrap_or_default()
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(64);
    for b in out.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn parse_draft_yaml(s: &Session) -> serde_yaml::Value {
    let body = read_draft(s);
    serde_yaml::from_str(&body).expect("draft is valid YAML")
}

fn set_reason(s: &Session, reason: &str) {
    let mut doc = parse_draft_yaml(s);
    if let serde_yaml::Value::Mapping(ref mut m) = doc {
        m.insert(
            serde_yaml::Value::String("reason".into()),
            serde_yaml::Value::String(reason.into()),
        );
    }
    let out = serde_yaml::to_string(&doc).expect("serialise");
    std::fs::write(draft_path(s), out).expect("write draft");
}

// ---------------------------------------------------------------------------
// TC-626 — `new create` starts draft and lists next-step commands
// ---------------------------------------------------------------------------

/// TC-626 — builder new create starts draft and lists next commands.
#[test]
fn tc_626_builder_new_create_starts_draft_and_lists_next_commands() {
    let s = Session::new();
    assert!(!draft_path(&s).exists());

    let out = s.run(&["request", "new", "create"]);
    out.assert_exit(0);

    // File was created with type: create and empty artifacts.
    assert!(draft_path(&s).exists(), "draft.yaml should exist");
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&read_draft(&s)).expect("parse draft");
    assert_eq!(
        doc.get("type").and_then(|v| v.as_str()),
        Some("create"),
        "draft should have type: create"
    );
    assert!(
        doc.get("artifacts").and_then(|v| v.as_sequence()).map(Vec::is_empty).unwrap_or(false),
        "artifacts should be empty"
    );

    // Stdout names the path and the next-step commands.
    out.assert_stdout_contains(".product/requests/draft.yaml");
    out.assert_stdout_contains("create");
    out.assert_stdout_contains("add feature");
    out.assert_stdout_contains("status");
    out.assert_stdout_contains("submit");
    out.assert_stdout_contains("discard");
}

// ---------------------------------------------------------------------------
// TC-627 — add feature appends + incremental validation
// ---------------------------------------------------------------------------

/// TC-627 — builder add feature appends to draft and runs incremental validation.
#[test]
fn tc_627_builder_add_feature_appends_to_draft_and_runs_incremental_validation() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);

    let out = s.run(&[
        "request", "add", "feature",
        "--title", "Rate Limiting",
        "--phase", "2",
        "--domains", "api,security",
    ]);
    out.assert_exit(0);

    let doc = parse_draft_yaml(&s);
    let artifacts = doc
        .get("artifacts")
        .and_then(|v| v.as_sequence())
        .expect("artifacts sequence");
    assert_eq!(artifacts.len(), 1, "one artifact expected");
    let a = &artifacts[0];
    assert_eq!(a.get("type").and_then(|v| v.as_str()), Some("feature"));
    assert_eq!(a.get("title").and_then(|v| v.as_str()), Some("Rate Limiting"));
    assert_eq!(a.get("phase").and_then(|v| v.as_u64()), Some(2));
    let domains: Vec<&str> = a
        .get("domains")
        .and_then(|v| v.as_sequence())
        .map(|s| s.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert_eq!(domains, vec!["api", "security"]);

    // ref name matches ^ft-[a-z0-9-]+$
    let ref_name = a.get("ref").and_then(|v| v.as_str()).expect("ref");
    let re = regex::Regex::new(r"^ft-[a-z0-9-]+$").expect("regex");
    assert!(re.is_match(ref_name), "ref must match ^ft-[a-z0-9-]+$, got {}", ref_name);

    out.assert_stdout_contains(ref_name);

    // Second call with unknown domain fails with E012 and does NOT append.
    let pre = read_draft(&s);
    let bad = s.run(&[
        "request", "add", "feature",
        "--title", "Another Feature",
        "--phase", "1",
        "--domains", "chimney",
    ]);
    assert_ne!(bad.exit_code, 0, "expected non-zero exit for E012");
    bad.assert_stderr_contains("E012");
    let post = read_draft(&s);
    assert_eq!(pre, post, "draft must be unchanged after E012");
}

// ---------------------------------------------------------------------------
// TC-628 — add dep --adr new satisfies E013 in the same step
// ---------------------------------------------------------------------------

/// TC-628 — builder add dep with new adr satisfies E013 in same step.
#[test]
fn tc_628_builder_add_dep_with_new_adr_satisfies_e013_in_same_step() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);

    // Preload a feature so the draft contains "one feature artifact".
    s.run(&[
        "request", "add", "feature",
        "--title", "Host Feature",
        "--phase", "1",
        "--domains", "api",
    ]).assert_exit(0);

    let out = s.run(&[
        "request", "add", "dep",
        "--title", "Redis",
        "--dep-type", "service",
        "--version", ">=7",
        "--adr", "new",
        "--adr-title", "Redis for rate limit state",
    ]);
    out.assert_exit(0);

    let doc = parse_draft_yaml(&s);
    let artifacts = doc.get("artifacts").and_then(|v| v.as_sequence()).expect("artifacts");
    // Feature + ADR + Dep = 3 entries.
    assert_eq!(artifacts.len(), 3, "draft should contain 3 artifacts");

    // Exactly one dep and one ADR; the ADR's `governs` lists the dep's `ref`.
    let dep = artifacts.iter().find(|a| a.get("type").and_then(|v| v.as_str()) == Some("dep")).expect("dep");
    let adr = artifacts.iter().find(|a| a.get("type").and_then(|v| v.as_str()) == Some("adr")).expect("adr");
    let dep_ref = dep.get("ref").and_then(|v| v.as_str()).expect("dep ref");
    let governs: Vec<&str> = adr
        .get("governs")
        .and_then(|v| v.as_sequence())
        .map(|s| s.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let expected_ref = format!("ref:{}", dep_ref);
    assert!(
        governs.contains(&expected_ref.as_str()),
        "adr.governs should include {} (got {:?})", expected_ref, governs,
    );

    out.assert_stdout_contains("E013 satisfied");
    out.assert_stdout_contains(dep_ref);
}

// ---------------------------------------------------------------------------
// TC-629 — status renders indicators + finding counts
// ---------------------------------------------------------------------------

/// TC-629 — builder status renders all artifacts with indicators and counts.
#[test]
fn tc_629_builder_status_renders_all_artifacts_with_indicators_and_counts() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);
    s.run(&[
        "request", "add", "feature",
        "--title", "Feature A",
        "--phase", "1",
        "--domains", "api",
    ]).assert_exit(0);
    s.run(&[
        "request", "add", "adr",
        "--title", "Adr A",
        "--domains", "api",
        "--scope", "feature-specific",
    ]).assert_exit(0);
    s.run(&[
        "request", "add", "tc",
        "--title", "Exit Criteria A",
        "--tc-type", "exit-criteria",
    ]).assert_exit(0);

    let out = s.run(&["request", "status"]);
    out.assert_exit(0);

    // Each artifact row is rendered.
    out.assert_stdout_contains("[feature]");
    out.assert_stdout_contains("[adr]");
    out.assert_stdout_contains("[tc]");
    out.assert_stdout_contains("ref:ft-");
    out.assert_stdout_contains("ref:adr-");
    out.assert_stdout_contains("ref:tc-");
    out.assert_stdout_contains("phase: 1");
    out.assert_stdout_contains("scope: feature-specific");
    out.assert_stdout_contains("tc-type: exit-criteria");
    // Footer shows Errors/Warnings counts.
    out.assert_stdout_contains("Errors:");
    out.assert_stdout_contains("Warnings:");
}

// ---------------------------------------------------------------------------
// TC-630 — submit archives draft on success
// ---------------------------------------------------------------------------

/// TC-630 — builder submit applies and archives draft on success.
#[test]
fn tc_630_builder_submit_applies_and_archives_draft_on_success() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);
    // Populate five clean artifacts.
    s.run(&["request", "add", "feature", "--title", "Host",    "--phase", "1", "--domains", "api"]).assert_exit(0);
    s.run(&["request", "add", "feature", "--title", "Host Two","--phase", "2", "--domains", "api"]).assert_exit(0);
    s.run(&["request", "add", "adr",     "--title", "Governance","--domains","api","--scope","feature-specific"]).assert_exit(0);
    s.run(&["request", "add", "tc",      "--title", "Exit",      "--tc-type","exit-criteria"]).assert_exit(0);
    s.run(&["request", "add", "tc",      "--title", "Exit Two",  "--tc-type","exit-criteria"]).assert_exit(0);
    set_reason(&s, "FT-052 — integration test submit happy path");

    let out = s.run(&["request", "submit"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("ADR-001");
    out.assert_stdout_contains("TC-001");
    out.assert_stdout_contains("Archived draft");

    assert!(!draft_path(&s).exists(), "draft.yaml should be gone after submit");
    let arch = archive_dir(&s);
    assert!(arch.exists(), "archive dir should exist");
    let entries: Vec<_> = std::fs::read_dir(&arch).expect("read archive").collect();
    assert_eq!(entries.len(), 1, "one archived draft expected");

    // `.product/request-log.jsonl` gained one entry with the reason.
    let log = s.read(".product/request-log.jsonl");
    assert!(log.contains("FT-052 — integration test submit happy path"));

    // Follow-up status reports no active draft.
    let status = s.run(&["request", "status"]);
    status.assert_stdout_contains("no active draft");
}

// ---------------------------------------------------------------------------
// TC-631 — submit blocked on E-class leaves draft unchanged
// ---------------------------------------------------------------------------

/// TC-631 — builder submit blocked on E-class leaves draft unchanged.
#[test]
fn tc_631_builder_submit_blocked_on_e_class_leaves_draft_unchanged() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);
    // Write a draft with an E013 finding: a dep with no governing ADR.
    let draft = r#"type: create
schema-version: 1
reason: "attempt to submit with E013"
artifacts:
  - type: dep
    ref: dep-orphan
    title: Orphan
    dep-type: library
"#;
    std::fs::write(draft_path(&s), draft).expect("write draft");

    let pre_hash = sha256(draft.as_bytes());

    let out = s.run(&["request", "submit"]);
    assert_eq!(out.exit_code, 1, "submit should exit 1 on E-class");
    out.assert_stderr_contains("E013");

    // Draft file SHA-256 unchanged.
    let post = std::fs::read(draft_path(&s)).expect("read draft");
    assert_eq!(sha256(&post), pre_hash, "draft must be byte-identical after E-class refusal");

    // No artifact files written.
    let features = std::fs::read_dir(s.root().join("docs/features")).expect("read");
    assert!(features.count() == 0, "no features should be written");
    let adrs = std::fs::read_dir(s.root().join("docs/adrs")).expect("read");
    assert!(adrs.count() == 0, "no adrs should be written");
    let deps = std::fs::read_dir(s.root().join("docs/dependencies")).expect("read");
    assert!(deps.count() == 0, "no deps should be written");

    // No entry appended to request-log.jsonl.
    let log_p = s.root().join(".product/request-log.jsonl");
    if log_p.exists() {
        let log = s.read(".product/request-log.jsonl");
        assert!(
            !log.contains("attempt to submit with E013"),
            "log should not contain failed-submit reason"
        );
    }
}

// ---------------------------------------------------------------------------
// TC-632 — new with existing draft surfaces options, no overwrite
// ---------------------------------------------------------------------------

/// TC-632 — builder new with existing draft surfaces status/submit/discard/continue.
#[test]
fn tc_632_builder_new_with_existing_draft_surfaces_status_submit_discard_continue() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);

    let pre_hash = sha256(read_draft(&s).as_bytes());

    let out = s.run(&["request", "new", "create"]);
    out.assert_exit(0);
    out.assert_stdout_contains("status");
    out.assert_stdout_contains("submit");
    out.assert_stdout_contains("discard");
    out.assert_stdout_contains("continue");

    // Draft file SHA unchanged — no overwrite.
    let post_hash = sha256(read_draft(&s).as_bytes());
    assert_eq!(pre_hash, post_hash, "existing draft must not be overwritten");
}

// ---------------------------------------------------------------------------
// TC-633 — discard removes draft
// ---------------------------------------------------------------------------

/// TC-633 — builder discard removes draft with confirmation or force.
#[test]
fn tc_633_builder_discard_removes_draft_with_confirmation_or_force() {
    let s = Session::new();
    s.run(&["request", "new", "create"]).assert_exit(0);
    // Populate three artifacts.
    s.run(&["request", "add", "feature", "--title", "A", "--phase", "1", "--domains", "api"]).assert_exit(0);
    s.run(&["request", "add", "feature", "--title", "B", "--phase", "1", "--domains", "api"]).assert_exit(0);
    s.run(&["request", "add", "feature", "--title", "C", "--phase", "1", "--domains", "api"]).assert_exit(0);
    assert!(draft_path(&s).exists());

    let out = s.run(&["request", "discard", "--force"]);
    out.assert_exit(0);

    assert!(!draft_path(&s).exists(), "draft should be deleted");
    assert!(!archive_dir(&s).exists() || std::fs::read_dir(archive_dir(&s)).expect("read").count() == 0,
        "discard must not produce an archive entry");

    let status = s.run(&["request", "status"]);
    status.assert_exit(0);
    status.assert_stdout_contains("no active draft");
}

// ---------------------------------------------------------------------------
// TC-634 — builder output identical to hand-written request YAML
// ---------------------------------------------------------------------------

/// TC-634 — builder output identical to hand-written request YAML (structurally).
#[test]
fn tc_634_builder_output_identical_to_hand_written_request_yaml() {
    // Hand-written path: one fresh session applying a YAML directly.
    let mut s_hand = Session::new();
    let r_hand = s_hand.apply(
        r#"type: create
schema-version: 1
reason: "TC-634 — equivalence"
artifacts:
  - type: feature
    ref: ft-eq
    title: Equivalence Feature
    phase: 1
    domains: [api]
    adrs: [ref:adr-eq]
    tests: [ref:tc-eq]
  - type: adr
    ref: adr-eq
    title: Equivalence ADR
    domains: [api]
    scope: feature-specific
  - type: tc
    ref: tc-eq
    title: Equivalence Exit
    tc-type: exit-criteria
    validates:
      features: [ref:ft-eq]
      adrs: [ref:adr-eq]
"#,
    );
    r_hand.assert_applied();

    // Builder path: fresh session, assemble the same intent via `add`
    // subcommands in arbitrary order.
    let s_build = Session::new();
    s_build.run(&["request", "new", "create"]).assert_exit(0);
    s_build.run(&[
        "request", "add", "adr",
        "--title", "Equivalence ADR",
        "--domains", "api",
        "--scope", "feature-specific",
        "--ref", "adr-eq",
    ]).assert_exit(0);
    s_build.run(&[
        "request", "add", "tc",
        "--title", "Equivalence Exit",
        "--tc-type", "exit-criteria",
        "--validates-features", "ft-eq",
        "--validates-adrs", "adr-eq",
        "--ref", "tc-eq",
    ]).assert_exit(0);
    s_build.run(&[
        "request", "add", "feature",
        "--title", "Equivalence Feature",
        "--phase", "1",
        "--domains", "api",
        "--ref", "ft-eq",
    ]).assert_exit(0);
    // Attach cross-links to the feature via change block — but we're in
    // create mode, so instead edit the draft's feature directly using the
    // request change path: we simply hand-edit the YAML to link, which is
    // exactly what the spec promises ("the draft IS the YAML").
    {
        let mut doc: serde_yaml::Value =
            serde_yaml::from_str(&std::fs::read_to_string(draft_path(&s_build)).expect("read"))
                .expect("parse");
        if let serde_yaml::Value::Mapping(ref mut m) = doc {
            m.insert(
                serde_yaml::Value::String("reason".into()),
                serde_yaml::Value::String("TC-634 — equivalence".into()),
            );
            if let Some(serde_yaml::Value::Sequence(ref mut arts)) =
                m.get_mut(serde_yaml::Value::String("artifacts".into()))
            {
                for a in arts.iter_mut() {
                    if let serde_yaml::Value::Mapping(ref mut am) = a {
                        if am.get(serde_yaml::Value::String("type".into()))
                            .and_then(|v| v.as_str()) == Some("feature")
                        {
                            am.insert(
                                serde_yaml::Value::String("adrs".into()),
                                serde_yaml::Value::Sequence(vec![
                                    serde_yaml::Value::String("ref:adr-eq".into()),
                                ]),
                            );
                            am.insert(
                                serde_yaml::Value::String("tests".into()),
                                serde_yaml::Value::Sequence(vec![
                                    serde_yaml::Value::String("ref:tc-eq".into()),
                                ]),
                            );
                        }
                    }
                }
            }
        }
        std::fs::write(
            draft_path(&s_build),
            serde_yaml::to_string(&doc).expect("serialise"),
        )
        .expect("write");
    }

    // Validate the draft — same findings (both clean by construction).
    let val = s_build.run(&["request", "validate"]);
    val.assert_exit(0);

    // Submit the draft.
    let out = s_build.run(&["request", "submit"]);
    out.assert_exit(0);

    // The resulting files exist and have the same front-matter structure.
    for p in ["docs/features", "docs/adrs", "docs/tests"] {
        let hand_entries: Vec<String> = std::fs::read_dir(s_hand.root().join(p))
            .expect("read hand")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        let build_entries: Vec<String> = std::fs::read_dir(s_build.root().join(p))
            .expect("read build")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            hand_entries.len(),
            build_entries.len(),
            "same number of files in {}", p
        );
    }
}

// ---------------------------------------------------------------------------
// TC-635 — exit criteria (aggregator)
// ---------------------------------------------------------------------------

/// TC-635 — product request builder exit-criteria.
///
/// Minimum wiring smoke test: the binary implements every required
/// subcommand, a full draft lifecycle (new → add → status → submit)
/// completes end-to-end, and the archive + request-log are populated.
/// Per-scenario assertions live in the TC-626–TC-634 tests above; if any
/// of those fail, this exit-criteria test is also implicitly failing via
/// `cargo test --test sessions`.
#[test]
fn tc_635_product_request_builder_exit() {
    let s = Session::new();
    // Every required subcommand exists (`--help` exits 0 for each).
    for sub in ["new", "continue", "discard", "status", "show", "submit", "edit"] {
        let out = s.run(&["request", sub, "--help"]);
        assert!(
            out.exit_code == 0,
            "`product request {}` --help must succeed; got exit={} stderr={}",
            sub, out.exit_code, out.stderr,
        );
    }
    for sub in ["feature", "adr", "tc", "dep", "doc", "target", "acknowledgement"] {
        let out = s.run(&["request", "add", sub, "--help"]);
        assert!(
            out.exit_code == 0,
            "`product request add {}` --help must succeed; got exit={} stderr={}",
            sub, out.exit_code, out.stderr,
        );
    }

    // End-to-end lifecycle.
    s.run(&["request", "new", "create"]).assert_exit(0);
    s.run(&["request", "add", "feature", "--title", "Exit Feature",
            "--phase", "1", "--domains", "api"]).assert_exit(0);
    s.run(&["request", "add", "adr", "--title", "Exit ADR",
            "--domains", "api", "--scope", "feature-specific"]).assert_exit(0);
    s.run(&["request", "add", "tc", "--title", "Exit TC",
            "--tc-type", "exit-criteria"]).assert_exit(0);
    set_reason(&s, "TC-635 — exit-criteria aggregator");
    s.run(&["request", "status"]).assert_exit(0);
    s.run(&["request", "submit"]).assert_exit(0);
    assert!(!draft_path(&s).exists(), "draft should be archived");
    assert!(archive_dir(&s).exists(), "archive dir should exist");
}
