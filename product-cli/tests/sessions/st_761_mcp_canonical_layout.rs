//! FT-061 — MCP server honors `.product/config.toml` discovery.
//!
//! Drives MCP tools directly against a canonical-layout fixture
//! (`.product/config.toml` plus `.product/features/`, `.product/adrs/`,
//! `.product/tests/`, `.product/dependencies/`) and asserts that read
//! and write tools succeed. Before FT-061 every one of these calls would
//! return a `ConfigError` because the handlers were hardcoded to
//! `repo_root.join("product.toml")`.

#![allow(clippy::unwrap_used)]

use product_lib::mcp::ToolRegistry;
use std::path::Path;

/// Build a canonical-layout repo at `root`. Mirrors the fixture
/// `product init` produces by default after FT-057.
fn write_canonical_repo(root: &Path) {
    let config = r#"name = "ft-061-canonical"
schema-version = "1"
[paths]
features = ".product/features"
adrs = ".product/adrs"
tests = ".product/tests"
graph = ".product/graph"
checklist = ".product/checklist.md"
dependencies = ".product/dependencies"
prompts = ".product/prompts"
gaps = ".product/gaps.json"
requests = ".product/requests.jsonl"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[mcp]
write = true
[domains]
api = "CLI surface, MCP tools"
[product]
name = "Canonical Test Product"
responsibility = "A test product for FT-061 canonical layout"
"#;

    let cfg_path = root.join(".product/config.toml");
    std::fs::create_dir_all(cfg_path.parent().unwrap()).expect("mkdir .product");
    std::fs::write(&cfg_path, config).expect("write .product/config.toml");

    for sub in [
        ".product/features",
        ".product/adrs",
        ".product/tests",
        ".product/dependencies",
        ".product/graph",
    ] {
        std::fs::create_dir_all(root.join(sub)).expect("mkdir canonical sub");
    }
}

// ---------------------------------------------------------------------------
// product_responsibility against canonical layout
// ---------------------------------------------------------------------------
#[test]
fn ft_061_mcp_responsibility_reads_canonical_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_canonical_repo(dir.path());

    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let result = registry
        .call_tool("product_responsibility", &serde_json::json!({}))
        .expect("product_responsibility must succeed against .product/config.toml");

    assert_eq!(
        result.get("name").and_then(|v| v.as_str()),
        Some("Canonical Test Product")
    );
    assert_eq!(
        result.get("responsibility").and_then(|v| v.as_str()),
        Some("A test product for FT-061 canonical layout")
    );
}

// ---------------------------------------------------------------------------
// product_graph_check against canonical layout — exercises both load_graph
// and the W019 responsibility re-load on the same repo_root.
// ---------------------------------------------------------------------------
#[test]
fn ft_061_mcp_graph_check_loads_canonical_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_canonical_repo(dir.path());

    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let result = registry
        .call_tool("product_graph_check", &serde_json::json!({}))
        .expect("product_graph_check must succeed against .product/config.toml");

    // The check returns a JSON object; the exact shape depends on findings.
    assert!(
        result.is_object(),
        "graph check result should be JSON object, got: {}",
        result
    );
}

// ---------------------------------------------------------------------------
// product_feature_new against canonical layout — exercises a write handler
// and confirms the new feature is dropped under `.product/features/`,
// not `docs/features/`.
// ---------------------------------------------------------------------------
#[test]
fn ft_061_mcp_feature_new_writes_into_canonical_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    write_canonical_repo(dir.path());

    let registry = ToolRegistry::new(dir.path().to_path_buf(), true);
    let result = registry
        .call_tool(
            "product_feature_new",
            &serde_json::json!({
                "title": "A canonical layout feature",
                "phase": 1,
            }),
        )
        .expect("product_feature_new must succeed against .product/config.toml");

    let written_path = result
        .get("path")
        .and_then(|v| v.as_str())
        .expect("response should include path");

    // Path must live under `.product/features/`, not `docs/features/`.
    assert!(
        written_path.contains(".product/features/"),
        "expected feature to land under .product/features/, got: {}",
        written_path
    );
    assert!(
        !written_path.contains("docs/features/"),
        "feature must not land under docs/features/ in canonical layout: {}",
        written_path
    );

    let on_disk = Path::new(written_path);
    assert!(
        on_disk.exists(),
        "feature file should exist on disk: {}",
        written_path
    );
}

// ---------------------------------------------------------------------------
// Legacy layout still works — guard against accidental regression.
// ---------------------------------------------------------------------------
#[test]
fn ft_061_mcp_responsibility_still_reads_legacy_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let config = r#"name = "ft-061-legacy"
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
[domains]
api = "CLI surface"
[product]
responsibility = "Legacy layout still honoured"
"#;
    std::fs::write(dir.path().join("product.toml"), config).expect("write");
    for sub in ["docs/features", "docs/adrs", "docs/tests", "docs/dependencies"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
    }

    let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
    let result = registry
        .call_tool("product_responsibility", &serde_json::json!({}))
        .expect("legacy product.toml must continue to work");

    assert_eq!(
        result.get("responsibility").and_then(|v| v.as_str()),
        Some("Legacy layout still honoured")
    );
}
