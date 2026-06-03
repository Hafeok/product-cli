//! FT-066 — MCP parity for feature/TC status writes and reciprocal linking.
//!
//! Drives `product_feature_status`, `product_test_status`, and
//! `product_feature_link` MCP tools directly against a fresh temp repo and
//! asserts on the post-write front-matter. Mirrors the FT-046 session
//! style in `st_140_mcp_adr_lifecycle.rs`.

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use product_mcp::ToolRegistry;

// ---------------------------------------------------------------------------
// Helpers — seed a session repo with raw feature / TC YAML.
// ---------------------------------------------------------------------------

fn write_feature(s: &Session, id: &str, title: &str, status: &str, tests: &[&str]) -> String {
    let tests_yaml: String = if tests.is_empty() {
        "[]".to_string()
    } else {
        let inner = tests
            .iter()
            .map(|t| format!("- {}", t))
            .collect::<Vec<_>>()
            .join("\n  ");
        format!("\n  {}", inner)
    };
    let slug = title.to_lowercase().replace(' ', "-");
    let path = format!("docs/features/{}-{}.md", id, slug);
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nphase: 1\nstatus: {status}\ndepends-on: []\nadrs: []\ntests: {tests_yaml}\ndomains: []\n---\n\n## Description\n\nseed for {id}.\n",
        id = id, title = title, status = status, tests_yaml = tests_yaml,
    );
    s.write(&path, &content);
    path
}

fn write_tc(
    s: &Session,
    id: &str,
    title: &str,
    status: &str,
    validates_features: &[&str],
    with_runner: bool,
) -> String {
    let features_yaml: String = if validates_features.is_empty() {
        "[]".to_string()
    } else {
        let inner = validates_features
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n    ");
        format!("\n    {}", inner)
    };
    let runner_block = if with_runner {
        format!(
            "runner: cargo-test\nrunner-args: \"{}\"\n",
            title.to_lowercase()
        )
    } else {
        String::new()
    };
    let slug = title.to_lowercase().replace(' ', "-");
    let path = format!("docs/tests/{}-{}.md", id, slug);
    let content = format!(
        "---\nid: {id}\ntitle: {title}\ntype: scenario\nstatus: {status}\nvalidates:\n  features: {features_yaml}\n  adrs: []\nphase: 1\n{runner_block}---\n\n## Description\n\nseed for {id}.\n",
        id = id,
        title = title,
        status = status,
        features_yaml = features_yaml,
        runner_block = runner_block,
    );
    s.write(&path, &content);
    path
}

fn write_adr(s: &Session, id: &str, title: &str, features: &[&str]) -> String {
    let features_yaml: String = if features.is_empty() {
        "[]".to_string()
    } else {
        let inner = features
            .iter()
            .map(|f| format!("- {}", f))
            .collect::<Vec<_>>()
            .join("\n  ");
        format!("\n  {}", inner)
    };
    let slug = title.to_lowercase().replace(' ', "-");
    let path = format!("docs/adrs/{}-{}.md", id, slug);
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: proposed\nfeatures: {features_yaml}\nsupersedes: []\nsuperseded-by: []\ndomains: [api]\nscope: domain\n---\n\n**Status:** Proposed\n\n**Context:** seed for {id}.\n\n**Decision:** decided.\n\n**Rationale:** because.\n\n**Rejected alternatives:** none.\n",
        id = id, title = title, features_yaml = features_yaml,
    );
    s.write(&path, &content);
    path
}

fn registry(s: &Session) -> ToolRegistry {
    ToolRegistry::new(s.dir.path().to_path_buf(), true)
}

/// Assert that a TC's `validates.features` array on disk contains `value`.
/// `assert_array_contains(..., "features", ...)` only inspects the top
/// level — TC files nest under `validates:`, so this helper digs in.
fn assert_tc_validates_feature(s: &Session, path: &str, value: &str) {
    let body = s.read(path);
    let between = body
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(fm, _)| fm)
        .unwrap_or_else(|| panic!("no front-matter in {}", path));
    let doc: serde_yaml::Value =
        serde_yaml::from_str(between).unwrap_or_else(|e| panic!("parse {}: {}", path, e));
    let arr = doc
        .get("validates")
        .and_then(|v| v.get("features"))
        .and_then(|v| v.as_sequence())
        .unwrap_or_else(|| panic!("{}: validates.features missing or not a list", path));
    let found = arr.iter().any(|v| v.as_str() == Some(value));
    assert!(
        found,
        "{}: validates.features must contain {}; got: {:?}",
        path, value, arr
    );
}

/// Assert that a TC's `validates.features` array on disk does NOT contain `value`.
fn assert_tc_validates_feature_missing(s: &Session, path: &str, value: &str) {
    let body = s.read(path);
    let between = body
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---"))
        .map(|(fm, _)| fm)
        .unwrap_or_else(|| panic!("no front-matter in {}", path));
    let doc: serde_yaml::Value =
        serde_yaml::from_str(between).unwrap_or_else(|e| panic!("parse {}: {}", path, e));
    if let Some(arr) = doc
        .get("validates")
        .and_then(|v| v.get("features"))
        .and_then(|v| v.as_sequence())
    {
        assert!(
            !arr.iter().any(|v| v.as_str() == Some(value)),
            "{}: validates.features must not contain {}; got: {:?}",
            path,
            value,
            arr
        );
    }
}

// ---------------------------------------------------------------------------
// TC-778 — feature_status writes status to disk
// ---------------------------------------------------------------------------

#[test]
fn tc_778_mcp_feature_status_writes_to_disk() {
    let s = Session::new();
    let path = write_feature(&s, "FT-001", "alpha-feature", "planned", &[]);

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-001", "status": "complete"}),
        )
        .expect("status write should succeed");

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("FT-001"));
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("complete")
    );
    assert!(
        result.get("note").is_none(),
        "response must not advise CLI fallback; got: {}",
        result
    );

    s.assert_frontmatter(&path, "status", "complete");
}

// ---------------------------------------------------------------------------
// TC-779 — test_status writes status to disk
// ---------------------------------------------------------------------------

#[test]
fn tc_779_mcp_test_status_writes_to_disk() {
    let s = Session::new();
    let path = write_tc(&s, "TC-001", "alpha-tc", "unimplemented", &[], true);

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_test_status",
            &serde_json::json!({"id": "TC-001", "status": "passing"}),
        )
        .expect("status write should succeed");

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("TC-001"));
    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("passing")
    );
    assert!(
        result.get("note").is_none(),
        "response must not advise CLI fallback; got: {}",
        result
    );

    s.assert_frontmatter(&path, "status", "passing");
}

// ---------------------------------------------------------------------------
// TC-780 — abandonment runs orphan-test cascade over MCP
// ---------------------------------------------------------------------------

#[test]
fn tc_780_mcp_feature_status_abandonment_runs_orphan_cascade() {
    let s = Session::new();
    let f_path = write_feature(&s, "FT-001", "to-abandon", "planned", &["TC-001"]);
    let tc_path = write_tc(&s, "TC-001", "only-validates-ft001", "unimplemented", &["FT-001"], true);

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-001", "status": "abandoned"}),
        )
        .expect("abandonment should succeed");

    assert_eq!(
        result.get("status").and_then(|v| v.as_str()),
        Some("abandoned")
    );

    // orphaned-tests array names the affected TC.
    let orphaned = result
        .get("orphaned-tests")
        .and_then(|v| v.as_array())
        .expect("orphaned-tests array present");
    assert_eq!(orphaned.len(), 1, "exactly one TC orphaned");
    assert_eq!(
        orphaned[0].get("test_id").and_then(|v| v.as_str()),
        Some("TC-001")
    );

    // FT-001 file is abandoned.
    s.assert_frontmatter(&f_path, "status", "abandoned");

    // TC-001's validates.features no longer contains FT-001.
    assert_tc_validates_feature_missing(&s, &tc_path, "FT-001");
}

// ---------------------------------------------------------------------------
// TC-781 — in-progress refused when linked TC lacks runner config (E022)
// ---------------------------------------------------------------------------

#[test]
fn tc_781_mcp_feature_status_in_progress_blocked_by_tc_runner_gate() {
    let s = Session::new();
    let f_path = write_feature(&s, "FT-001", "needs-runners", "planned", &["TC-001"]);
    let tc_path = write_tc(&s, "TC-001", "no-runner-config", "unimplemented", &["FT-001"], false);
    let pre_digest = s.docs_digest();

    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-001", "status": "in-progress"}),
        )
        .expect_err("in-progress without runner config must be rejected");

    assert!(
        err.contains("E022") || err.to_lowercase().contains("tc runner"),
        "expected TcRunnerMissing error; got: {}",
        err
    );
    assert!(
        err.contains("TC-001"),
        "error must name the offending TC; got: {}",
        err
    );

    // No file on disk was modified.
    let post_digest = s.docs_digest();
    assert_eq!(
        pre_digest, post_digest,
        "rejected status change must not touch disk"
    );

    // Front-matter sanity.
    s.assert_frontmatter(&f_path, "status", "planned");
    assert!(std::fs::read_to_string(s.dir.path().join(&tc_path)).is_ok());
}

// ---------------------------------------------------------------------------
// TC-782 — unknown ID returns NotFound on both status tools
// ---------------------------------------------------------------------------

#[test]
fn tc_782_mcp_status_update_unknown_id_returns_error() {
    let s = Session::new();
    let reg = registry(&s);

    let err_f = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-999", "status": "complete"}),
        )
        .expect_err("unknown feature ID must error");
    let lower = err_f.to_lowercase();
    assert!(
        lower.contains("not found") || lower.contains("notfound"),
        "expected NotFound for FT-999; got: {}",
        err_f
    );
    assert!(
        err_f.contains("FT-999"),
        "error must name the unknown ID; got: {}",
        err_f
    );

    let err_t = reg
        .call_tool(
            "product_test_status",
            &serde_json::json!({"id": "TC-999", "status": "passing"}),
        )
        .expect_err("unknown TC ID must error");
    let lower = err_t.to_lowercase();
    assert!(
        lower.contains("not found") || lower.contains("notfound"),
        "expected NotFound for TC-999; got: {}",
        err_t
    );
    assert!(
        err_t.contains("TC-999"),
        "error must name the unknown ID; got: {}",
        err_t
    );
}

// ---------------------------------------------------------------------------
// TC-783 — invalid status string returns parse error, no write
// ---------------------------------------------------------------------------

#[test]
fn tc_783_mcp_status_update_invalid_status_string_returns_error() {
    let s = Session::new();
    let f_path = write_feature(&s, "FT-001", "feature-a", "planned", &[]);
    let tc_path = write_tc(&s, "TC-001", "tc-a", "unimplemented", &[], true);
    let pre_digest = s.docs_digest();

    let reg = registry(&s);
    let err_f = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-001", "status": "bogus"}),
        )
        .expect_err("invalid feature status must error");
    assert!(
        err_f.contains("bogus") || err_f.to_lowercase().contains("unknown"),
        "expected parse error mentioning the bogus value; got: {}",
        err_f
    );

    let err_t = reg
        .call_tool(
            "product_test_status",
            &serde_json::json!({"id": "TC-001", "status": "bogus"}),
        )
        .expect_err("invalid TC status must error");
    assert!(
        err_t.contains("bogus") || err_t.to_lowercase().contains("unknown"),
        "expected parse error mentioning the bogus value; got: {}",
        err_t
    );

    // No file was modified.
    let post_digest = s.docs_digest();
    assert_eq!(
        pre_digest, post_digest,
        "rejected status change must not touch disk"
    );

    s.assert_frontmatter(&f_path, "status", "planned");
    s.assert_frontmatter(&tc_path, "status", "unimplemented");
}

// ---------------------------------------------------------------------------
// TC-784 — invariant: legacy CLI-fallback note string absent from src/
// ---------------------------------------------------------------------------

#[test]
fn tc_784_mcp_status_update_legacy_note_string_absent_from_codebase() {
    use std::path::Path;
    let forbidden = "Use CLI for status updates with full side-effects";

    fn walk(dir: &Path, forbidden: &str, hits: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk(&p, forbidden, hits);
                } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                    if let Ok(s) = std::fs::read_to_string(&p) {
                        if s.contains(forbidden) {
                            hits.push(p.display().to_string());
                        }
                    }
                }
            }
        }
    }

    // Compute the repo's src/ relative to this test binary. We're running
    // under `cargo test --test sessions`, so CARGO_MANIFEST_DIR points at
    // the crate root.
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src = Path::new(manifest).join("src");
    assert!(src.exists(), "src/ must exist at {}", src.display());

    let mut hits = Vec::new();
    walk(&src, forbidden, &mut hits);
    assert!(
        hits.is_empty(),
        "legacy note string must not appear in src/. Found in: {:?}",
        hits
    );
}

// ---------------------------------------------------------------------------
// TC-785 — feature_link with --test reciprocates validates.features
// ---------------------------------------------------------------------------

#[test]
fn tc_785_mcp_feature_link_reciprocates_tc_validates_features() {
    let s = Session::new();
    let f_path = write_feature(&s, "FT-001", "linker", "planned", &[]);
    let tc_path = write_tc(&s, "TC-001", "linked-tc", "unimplemented", &[], true);

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_feature_link",
            &serde_json::json!({"id": "FT-001", "test": "TC-001"}),
        )
        .expect("link should succeed");

    // FT-001.tests contains TC-001.
    s.assert_array_contains(&f_path, "tests", "TC-001");

    // TC-001's validates.features contains FT-001.
    assert_tc_validates_feature(&s, &tc_path, "FT-001");

    // Response includes the reciprocation entry.
    let reciprocated = result
        .get("reciprocated")
        .and_then(|v| v.as_array())
        .expect("reciprocated array present");
    let found = reciprocated.iter().any(|r| {
        r.get("id").and_then(|v| v.as_str()) == Some("TC-001")
            && r.get("field").and_then(|v| v.as_str()) == Some("validates.features")
    });
    assert!(found, "reciprocated must name TC-001/validates.features; got: {}", result);
}

// ---------------------------------------------------------------------------
// TC-786 — feature_link with --adr reciprocates ADR.features
// ---------------------------------------------------------------------------

#[test]
fn tc_786_mcp_feature_link_reciprocates_adr_features() {
    let s = Session::new();
    let f_path = write_feature(&s, "FT-001", "linker", "planned", &[]);
    let a_path = write_adr(&s, "ADR-001", "linked-adr", &[]);

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_feature_link",
            &serde_json::json!({"id": "FT-001", "adr": "ADR-001"}),
        )
        .expect("link should succeed");

    // FT-001.adrs contains ADR-001.
    s.assert_array_contains(&f_path, "adrs", "ADR-001");

    // ADR-001.features contains FT-001.
    s.assert_array_contains(&a_path, "features", "FT-001");

    let reciprocated = result
        .get("reciprocated")
        .and_then(|v| v.as_array())
        .expect("reciprocated array present");
    let found = reciprocated.iter().any(|r| {
        r.get("id").and_then(|v| v.as_str()) == Some("ADR-001")
            && r.get("field").and_then(|v| v.as_str()) == Some("features")
    });
    assert!(found, "reciprocated must name ADR-001/features; got: {}", result);
}

// ---------------------------------------------------------------------------
// TC-787 — feature_link returns a structured writes report
// ---------------------------------------------------------------------------

#[test]
fn tc_787_mcp_feature_link_returns_structured_writes_report() {
    let s = Session::new();
    let _f_path = write_feature(&s, "FT-001", "linker", "planned", &[]);
    let _t_path = write_tc(&s, "TC-001", "linked-tc", "unimplemented", &[], true);
    let _a_path = write_adr(&s, "ADR-001", "linked-adr", &[]);

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_feature_link",
            &serde_json::json!({"id": "FT-001", "test": "TC-001", "adr": "ADR-001"}),
        )
        .expect("link should succeed");

    // No legacy `linked` bool field.
    assert!(
        result.get("linked").is_none(),
        "response must not carry the legacy `linked` field; got: {}",
        result
    );

    // Top-level id field still echoes FT-001.
    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("FT-001"));

    // Writes array has 3 entries: feature, ADR, TC. Each carries path+kind.
    let writes = result
        .get("writes")
        .and_then(|v| v.as_array())
        .expect("writes array present");
    assert_eq!(writes.len(), 3, "expected 3 writes; got: {}", result);
    for w in writes {
        assert!(
            w.get("path").and_then(|v| v.as_str()).is_some(),
            "write entry must carry a `path`: {}", w
        );
        let kind = w.get("kind").and_then(|v| v.as_str()).expect("kind");
        assert!(
            matches!(kind, "feature" | "adr" | "tc"),
            "kind must be feature|adr|tc; got: {}",
            kind
        );
    }

    // Reciprocated array has 2 entries.
    let reciprocated = result
        .get("reciprocated")
        .and_then(|v| v.as_array())
        .expect("reciprocated array present");
    assert_eq!(reciprocated.len(), 2, "expected 2 reciprocations; got: {}", result);

    // Second call (no-op idempotent): writes and reciprocated are empty.
    let reg = registry(&s);
    let result2 = reg
        .call_tool(
            "product_feature_link",
            &serde_json::json!({"id": "FT-001", "test": "TC-001", "adr": "ADR-001"}),
        )
        .expect("idempotent link should succeed");
    let writes2 = result2
        .get("writes")
        .and_then(|v| v.as_array())
        .expect("writes array present on idempotent call");
    assert!(writes2.is_empty(), "no-op writes must be empty; got: {}", result2);
    let reciprocated2 = result2
        .get("reciprocated")
        .and_then(|v| v.as_array())
        .expect("reciprocated array present on idempotent call");
    assert!(
        reciprocated2.is_empty(),
        "no-op reciprocation must be empty; got: {}",
        result2
    );
}

// ---------------------------------------------------------------------------
// TC-788 — FT-066 exit criteria — every TC-778..TC-787 path succeeds
// ---------------------------------------------------------------------------

#[test]
fn tc_788_ft_066_exit_criteria_mcp_status_and_link_parity() {
    // 1) feature_status writes to disk
    let s = Session::new();
    let f1 = write_feature(&s, "FT-100", "smoke-feature-status", "planned", &[]);
    let reg = registry(&s);
    let r = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-100", "status": "complete"}),
        )
        .expect("feature_status complete");
    assert_eq!(r.get("status").and_then(|v| v.as_str()), Some("complete"));
    assert!(r.get("note").is_none());
    s.assert_frontmatter(&f1, "status", "complete");

    // 2) test_status writes to disk
    let s = Session::new();
    let t1 = write_tc(&s, "TC-100", "smoke-test-status", "unimplemented", &[], true);
    let reg = registry(&s);
    let r = reg
        .call_tool(
            "product_test_status",
            &serde_json::json!({"id": "TC-100", "status": "passing"}),
        )
        .expect("test_status passing");
    assert_eq!(r.get("status").and_then(|v| v.as_str()), Some("passing"));
    s.assert_frontmatter(&t1, "status", "passing");

    // 3) abandonment orphan cascade
    let s = Session::new();
    let f_a = write_feature(&s, "FT-100", "smoke-abandon", "planned", &["TC-100"]);
    let t_a = write_tc(&s, "TC-100", "smoke-orphan", "unimplemented", &["FT-100"], true);
    let reg = registry(&s);
    let r = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-100", "status": "abandoned"}),
        )
        .expect("abandon");
    let orphaned = r.get("orphaned-tests").and_then(|v| v.as_array()).expect("orphan array");
    assert_eq!(orphaned.len(), 1);
    s.assert_frontmatter(&f_a, "status", "abandoned");
    assert_tc_validates_feature_missing(&s, &t_a, "FT-100");

    // 4) feature_link reciprocates TC + ADR with structured response
    let s = Session::new();
    let f = write_feature(&s, "FT-100", "smoke-link", "planned", &[]);
    let t = write_tc(&s, "TC-100", "smoke-link-tc", "unimplemented", &[], true);
    let a = write_adr(&s, "ADR-100", "smoke-link-adr", &[]);
    let reg = registry(&s);
    let r = reg
        .call_tool(
            "product_feature_link",
            &serde_json::json!({"id": "FT-100", "test": "TC-100", "adr": "ADR-100"}),
        )
        .expect("link");
    assert!(r.get("linked").is_none(), "legacy `linked` flag must not be present");
    let writes = r.get("writes").and_then(|v| v.as_array()).expect("writes");
    assert_eq!(writes.len(), 3, "feature + adr + tc");
    let reciprocated = r.get("reciprocated").and_then(|v| v.as_array()).expect("reciprocated");
    assert_eq!(reciprocated.len(), 2);
    s.assert_array_contains(&f, "tests", "TC-100");
    s.assert_array_contains(&f, "adrs", "ADR-100");
    assert_tc_validates_feature(&s, &t, "FT-100");
    s.assert_array_contains(&a, "features", "FT-100");

    // 5) unknown ID returns NotFound on status tools
    let s = Session::new();
    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-999", "status": "complete"}),
        )
        .expect_err("unknown");
    assert!(err.to_lowercase().contains("not found"));

    // 6) invalid status string returns parse error
    let s = Session::new();
    write_feature(&s, "FT-100", "parse-fail", "planned", &[]);
    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_feature_status",
            &serde_json::json!({"id": "FT-100", "status": "garbage"}),
        )
        .expect_err("bad status");
    assert!(err.contains("garbage") || err.to_lowercase().contains("unknown"));
}
