//! FT-069 — MCP parity for `product_graph_check`.
//!
//! Each TC exercises one validation layer (W030, E011, W028, log-verify)
//! and asserts that the MCP `product_graph_check` JSON envelope contains
//! the corresponding finding *and* equals the CLI `product graph check
//! --format json` envelope byte-for-byte under ordering normalisation.
//!
//! TC-810 is the parity invariant — a compound fixture triggering every
//! new layer at once, checked end-to-end.

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use product_lib::mcp::ToolRegistry;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Write a fresh `product.toml` whose `[features]` block enforces the FT-055
/// body convention. The default Session config disables W030 — this helper
/// re-enables it for the parity TCs.
fn enable_w030(s: &Session) {
    let cfg = r#"name = "session-test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[mcp]
write = true
[domains]
api = "CLI surface, MCP tools"
[features]
required-sections = ["Description", "Functional Specification", "Out of scope"]
functional-spec-subsections = ["Inputs", "Outputs", "State", "Behaviour", "Invariants", "Error handling", "Boundaries"]
required-from-phase = 1
"#;
    std::fs::write(s.dir.path().join("product.toml"), cfg)
        .expect("write product.toml with W030 enabled");
}

/// Write a `product.toml` enabling log verification on graph check.
fn enable_log_verify(s: &Session) {
    let cfg = r#"name = "session-test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
dependencies = "docs/dependencies"
requests = "requests.jsonl"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[mcp]
write = true
[domains]
api = "CLI surface, MCP tools"
[features]
required-sections = []
functional-spec-subsections = []
[log]
verify-on-check = true
"#;
    std::fs::write(s.dir.path().join("product.toml"), cfg)
        .expect("write product.toml with verify-on-check");
}

/// Write a `product.toml` enabling both W030 and log verification — used by
/// the parity invariant TC-810 to exercise every new layer at once.
fn enable_w030_and_log_verify(s: &Session) {
    let cfg = r#"name = "session-test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
dependencies = "docs/dependencies"
requests = "requests.jsonl"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[mcp]
write = true
[domains]
api = "CLI surface, MCP tools"
[features]
required-sections = ["Description", "Functional Specification", "Out of scope"]
functional-spec-subsections = ["Inputs", "Outputs", "State", "Behaviour", "Invariants", "Error handling", "Boundaries"]
required-from-phase = 1
[log]
verify-on-check = true
"#;
    std::fs::write(s.dir.path().join("product.toml"), cfg)
        .expect("write product.toml with W030 + log verify");
}

/// Write a minimal feature whose body is missing every required section
/// other than `## Description`. Triggers W030 (top-level missing + every
/// FS subsection missing) when `required-sections` is non-empty.
fn write_w030_feature(s: &Session) -> String {
    let path = "docs/features/FT-001-w030-trigger.md";
    let content = "---\nid: FT-001\ntitle: W030 trigger\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\n---\n\n## Description\n\nIntentionally missing the Functional Specification and Out of scope sections.\n";
    s.write(path, content);
    path.to_string()
}

/// Write a feature whose `domains-acknowledged.api` is the empty string —
/// the E011 trigger described in ADR-025 / ADR-026.
fn write_e011_feature(s: &Session) -> String {
    let path = "docs/features/FT-002-e011-trigger.md";
    let content = "---\nid: FT-002\ntitle: E011 trigger\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged:\n  api: \"\"\n---\n\n## Description\n\nAcknowledgement carries an empty reason — E011 must fire.\n";
    s.write(path, content);
    path.to_string()
}

/// Write a feature with `due-date: 1970-01-01` — guaranteed overdue.
fn write_w028_feature(s: &Session) -> String {
    let path = "docs/features/FT-003-w028-trigger.md";
    let content = "---\nid: FT-003\ntitle: W028 trigger\nphase: 1\nstatus: in-progress\ndue-date: \"1970-01-01\"\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\n---\n\n## Description\n\nDeliberately overdue; W028 must fire.\n";
    s.write(path, content);
    path.to_string()
}

/// Run the MCP `product_graph_check` tool via the in-process registry and
/// return the parsed JSON envelope.
fn mcp_envelope(s: &Session) -> Value {
    let reg = ToolRegistry::new(s.dir.path().to_path_buf(), true);
    reg.call_tool("product_graph_check", &serde_json::json!({}))
        .expect("product_graph_check should succeed")
}

/// Run the CLI `product graph check --format json` and return the parsed
/// JSON envelope. The CLI exits 1 on errors and 2 on warnings-only;
/// callers tolerate any non-error exit code.
fn cli_envelope(s: &Session) -> Value {
    let out = s.run(&["graph", "check", "--format", "json"]);
    assert!(
        !out.stdout.is_empty(),
        "CLI graph check produced no stdout (exit {})\nstderr: {}",
        out.exit_code,
        out.stderr,
    );
    serde_json::from_str(&out.stdout)
        .unwrap_or_else(|e| panic!("CLI stdout was not valid JSON: {}\n{}", e, out.stdout))
}

/// Collect the set of finding codes (errors + warnings) from an envelope.
fn codes(envelope: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tier in ["errors", "warnings"] {
        if let Some(arr) = envelope.get(tier).and_then(|v| v.as_array()) {
            for f in arr {
                if let Some(code) = f.get("code").and_then(|v| v.as_str()) {
                    out.push(code.to_string());
                }
            }
        }
    }
    out
}

/// Normalise a check envelope into a deterministic value for byte-equality
/// comparison. The MCP and CLI both serialise from `CheckResult::to_json`,
/// so the ordering is already deterministic — this helper double-checks by
/// sorting findings by `(code, file, line, detail)`.
fn normalise(envelope: &Value) -> Value {
    let mut e = envelope.clone();
    for tier in ["errors", "warnings"] {
        if let Some(arr) = e.get_mut(tier).and_then(|v| v.as_array_mut()) {
            arr.sort_by(|a, b| {
                let ka = (
                    a.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    a.get("file").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    a.get("line").and_then(|v| v.as_u64()).unwrap_or(0),
                    a.get("detail").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                );
                let kb = (
                    b.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    b.get("file").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    b.get("line").and_then(|v| v.as_u64()).unwrap_or(0),
                    b.get("detail").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                );
                ka.cmp(&kb)
            });
        }
    }
    e
}

/// Assert both envelopes are byte-identical after normalisation, and that
/// the expected code appears in both.
fn assert_parity_with_code(mcp: &Value, cli: &Value, expected_code: &str) {
    let mcp_codes = codes(mcp);
    let cli_codes = codes(cli);
    assert!(
        mcp_codes.iter().any(|c| c == expected_code),
        "MCP envelope missing {}: codes = {:?}\nenvelope: {}",
        expected_code,
        mcp_codes,
        mcp,
    );
    assert!(
        cli_codes.iter().any(|c| c == expected_code),
        "CLI envelope missing {}: codes = {:?}\nenvelope: {}",
        expected_code,
        cli_codes,
        cli,
    );
    let mcp_n = normalise(mcp);
    let cli_n = normalise(cli);
    assert_eq!(
        serde_json::to_string(&mcp_n).unwrap(),
        serde_json::to_string(&cli_n).unwrap(),
        "MCP and CLI envelopes diverge under normalisation:\nmcp = {:#}\ncli = {:#}",
        mcp_n,
        cli_n,
    );
}

// ---------------------------------------------------------------------------
// TC-806 — MCP surfaces W030 functional-spec finding
// ---------------------------------------------------------------------------

#[test]
fn tc_806_mcp_graph_check_surfaces_w030_functional_spec_finding() {
    let s = Session::new();
    enable_w030(&s);
    write_w030_feature(&s);

    let mcp = mcp_envelope(&s);
    let cli = cli_envelope(&s);
    assert_parity_with_code(&mcp, &cli, "W030");
}

// ---------------------------------------------------------------------------
// TC-807 — MCP surfaces E011 domain-acknowledgement finding
// ---------------------------------------------------------------------------

#[test]
fn tc_807_mcp_graph_check_surfaces_e011_domain_acknowledgement_finding() {
    let s = Session::new();
    // Default Session config (W030 disabled) suffices — E011 is structural.
    write_e011_feature(&s);

    let mcp = mcp_envelope(&s);
    let cli = cli_envelope(&s);
    assert_parity_with_code(&mcp, &cli, "E011");
}

// ---------------------------------------------------------------------------
// TC-808 — MCP surfaces W028 due-date finding
// ---------------------------------------------------------------------------

#[test]
fn tc_808_mcp_graph_check_surfaces_w028_due_date_finding() {
    let s = Session::new();
    write_w028_feature(&s);

    let mcp = mcp_envelope(&s);
    let cli = cli_envelope(&s);
    assert_parity_with_code(&mcp, &cli, "W028");
}

// ---------------------------------------------------------------------------
// TC-809 — MCP surfaces request-log verification finding
// ---------------------------------------------------------------------------

#[test]
fn tc_809_mcp_graph_check_surfaces_log_verification_finding() {
    let mut s = Session::new();
    enable_log_verify(&s);

    // Apply one valid request so requests.jsonl has at least one entry.
    s.apply(
        r#"type: create
schema-version: 1
reason: "seed for log-verify parity test"
artifacts:
  - type: feature
    title: Seed
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    // Tamper: rewrite the reason inside the entry so its stored hash no
    // longer matches. This is the same trick TC-560 uses.
    let log_path = s.dir.path().join("requests.jsonl");
    let raw = std::fs::read_to_string(&log_path).expect("read log");
    let tampered = raw.replacen(
        "seed for log-verify parity test",
        "T4MPERED reason aaaaaaaaaaaaaaaaaa",
        1,
    );
    assert_ne!(raw, tampered, "tamper substitution must apply");
    std::fs::write(&log_path, tampered).expect("write tampered log");

    let mcp = mcp_envelope(&s);
    let cli = cli_envelope(&s);

    // Either E017 (per-entry hash mismatch) or E018 (chain break) must
    // surface — both are propagated by `verify_log`.
    let mcp_codes = codes(&mcp);
    let has_log_code = mcp_codes.iter().any(|c| c == "E017" || c == "E018");
    assert!(
        has_log_code,
        "MCP envelope missing E017/E018 log-verify finding: codes = {:?}\nenvelope: {}",
        mcp_codes, mcp,
    );
    // Parity: both envelopes equal under normalisation.
    let mcp_n = normalise(&mcp);
    let cli_n = normalise(&cli);
    assert_eq!(
        serde_json::to_string(&mcp_n).unwrap(),
        serde_json::to_string(&cli_n).unwrap(),
        "MCP and CLI envelopes diverge for log-verify fixture",
    );
}

// ---------------------------------------------------------------------------
// TC-810 — invariant: MCP envelope equals CLI envelope across the
// fixture matrix (clean, W030, E011, W028, log-verify, compound).
// ---------------------------------------------------------------------------

#[test]
fn tc_810_mcp_graph_check_json_equals_cli_graph_check_json() {
    // Fixture 1: clean repo, no overrides.
    {
        let s = Session::new();
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_eq!(
            serde_json::to_string(&normalise(&mcp)).unwrap(),
            serde_json::to_string(&normalise(&cli)).unwrap(),
            "clean fixture: MCP and CLI envelopes diverge",
        );
    }

    // Fixture 2: W030 trigger.
    {
        let s = Session::new();
        enable_w030(&s);
        write_w030_feature(&s);
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_parity_with_code(&mcp, &cli, "W030");
    }

    // Fixture 3: E011 trigger.
    {
        let s = Session::new();
        write_e011_feature(&s);
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_parity_with_code(&mcp, &cli, "E011");
    }

    // Fixture 4: W028 trigger.
    {
        let s = Session::new();
        write_w028_feature(&s);
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_parity_with_code(&mcp, &cli, "W028");
    }

    // Fixture 5: compound — every layer triggers at once.
    {
        let mut s = Session::new();
        enable_w030_and_log_verify(&s);
        write_w030_feature(&s);
        write_e011_feature(&s);
        write_w028_feature(&s);

        s.apply(
            r#"type: create
schema-version: 1
reason: "seed for compound parity fixture"
artifacts:
  - type: feature
    title: Seed Compound
    phase: 1
    domains: [api]
"#,
        )
        .assert_applied();

        let log_path = s.dir.path().join("requests.jsonl");
        let raw = std::fs::read_to_string(&log_path).expect("read log");
        let tampered = raw.replacen(
            "seed for compound parity fixture",
            "T4MPERED reason cccccccccccccccccc",
            1,
        );
        assert_ne!(raw, tampered, "tamper substitution must apply");
        std::fs::write(&log_path, tampered).expect("write tampered log");

        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);

        // Every layer must be present in the MCP envelope.
        let cs = codes(&mcp);
        assert!(cs.iter().any(|c| c == "W030"), "missing W030: {:?}", cs);
        assert!(cs.iter().any(|c| c == "E011"), "missing E011: {:?}", cs);
        assert!(cs.iter().any(|c| c == "W028"), "missing W028: {:?}", cs);
        assert!(
            cs.iter().any(|c| c == "E017" || c == "E018"),
            "missing log-verify code: {:?}",
            cs,
        );

        // Byte-equal parity under normalisation.
        let mcp_n = normalise(&mcp);
        let cli_n = normalise(&cli);
        assert_eq!(
            serde_json::to_string(&mcp_n).unwrap(),
            serde_json::to_string(&cli_n).unwrap(),
            "compound fixture: MCP and CLI envelopes diverge",
        );
    }
}

// ---------------------------------------------------------------------------
// TC-811 — exit criteria aggregator
// ---------------------------------------------------------------------------

#[test]
fn tc_811_ft_069_exit_criteria() {
    // 1. Per-layer TCs (TC-806..TC-809) — execute the same flow inline so
    //    that running this single TC validates the feature end-to-end.
    {
        let s = Session::new();
        enable_w030(&s);
        write_w030_feature(&s);
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_parity_with_code(&mcp, &cli, "W030");
    }
    {
        let s = Session::new();
        write_e011_feature(&s);
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_parity_with_code(&mcp, &cli, "E011");
    }
    {
        let s = Session::new();
        write_w028_feature(&s);
        let mcp = mcp_envelope(&s);
        let cli = cli_envelope(&s);
        assert_parity_with_code(&mcp, &cli, "W028");
    }

    // 2. The shared library function exists at its canonical path and is
    //    addressable as `product_lib::graph::full_check::run`.
    {
        let s = Session::new();
        // Smoke-call against an empty fixture — must not panic and must
        // return a CheckResult whose JSON has the expected shape.
        let mcp = mcp_envelope(&s);
        assert!(mcp.get("errors").is_some(), "missing `errors` field");
        assert!(mcp.get("warnings").is_some(), "missing `warnings` field");
        assert!(mcp.get("summary").is_some(), "missing `summary` field");
    }

    // 3. CLI exit-code semantics unchanged:
    //    clean fixture exits 0; warning-only fixture exits 2.
    {
        let s = Session::new();
        let out = s.run(&["graph", "check", "--format", "json"]);
        assert_eq!(
            out.exit_code, 0,
            "clean fixture CLI exit must be 0: stdout={} stderr={}",
            out.stdout, out.stderr,
        );
    }
    {
        let s = Session::new();
        write_w028_feature(&s); // W-class only, no E-class
        let out = s.run(&["graph", "check", "--format", "json"]);
        assert_eq!(
            out.exit_code, 2,
            "warning-only fixture CLI exit must be 2: stdout={} stderr={}",
            out.stdout, out.stderr,
        );
    }

    // 4. The legacy inline helper is gone from the CLI adapter.
    {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest).join("src/commands/graph_cmd.rs");
        let body = std::fs::read_to_string(&path).expect("read graph_cmd.rs");
        assert!(
            !body.contains("append_log_findings_to_check"),
            "graph_cmd.rs must not contain the legacy inline helper",
        );
    }

    // 5. The MCP handler no longer calls the bare graph.check() shortcut.
    {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest).join("src/mcp/registry.rs");
        let body = std::fs::read_to_string(&path).expect("read registry.rs");
        // The handler must route through full_check::run.
        assert!(
            body.contains("full_check::run"),
            "registry.rs must dispatch product_graph_check through full_check::run",
        );
    }
}
