//! TC-702 — FT-057 exit-criteria smoke test.
//!
//! This aggregator test does not duplicate every TC's assertion. It asserts
//! the minimum signal that FT-057's deliverables are wired up: the new
//! discovery contract, the new path-config keys, and the migration command
//! reach an idempotent steady state on a fresh canonical layout.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn product_bin() -> PathBuf {
    if let Some(b) = option_env!("CARGO_BIN_EXE_product") {
        let p = PathBuf::from(b);
        if p.exists() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut p = exe.clone();
        p.pop();
        p.pop();
        p.push("product");
        if p.exists() {
            return p;
        }
    }
    PathBuf::from("target/debug/product")
}

#[test]
fn tc_702_ft_057_exit_criteria() {
    // Build a canonical-layout repo and verify the new keys round-trip.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join(".product")).expect("mkdir .product");
    let canonical_toml = r#"name = "exit-criteria"
schema-version = "1"

[paths]
features = ".product/features"
adrs = ".product/adrs"
tests = ".product/tests"
dependencies = ".product/dependencies"
graph = ".product/graph"
checklist = ".product/checklist.md"
requests = ".product/requests.jsonl"
prompts = ".product/prompts"
gaps = ".product/gaps.json"

[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"

[domains]
api = "CLI surface"
"#;
    std::fs::write(dir.path().join(".product/config.toml"), canonical_toml).expect("write toml");
    for sub in [
        ".product/features",
        ".product/adrs",
        ".product/tests",
        ".product/dependencies",
        ".product/graph",
        ".product/prompts",
    ] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
    }

    // Library-level: discover finds the canonical config and the new keys
    // resolve correctly.
    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(dir.path()).expect("chdir");
    let result = product_lib::config::ProductConfig::discover();
    let _ = std::env::set_current_dir(cwd);
    let (config, _root) = result.expect("discover canonical");
    assert_eq!(config.paths.features, ".product/features");
    assert_eq!(config.paths.prompts.as_deref(), Some(".product/prompts"));
    assert_eq!(config.paths.gaps.as_deref(), Some(".product/gaps.json"));
    assert_eq!(config.paths.prompts_resolved(), ".product/prompts");
    assert_eq!(config.paths.gaps_resolved(), ".product/gaps.json");

    // CLI-level: `feature list` runs in the canonical layout from a deep cwd.
    let bin = product_bin();
    let sub = dir.path().join(".product/features");
    let out = Command::new(&bin)
        .args(["feature", "list"])
        .current_dir(&sub)
        .stdin(Stdio::null())
        .output()
        .expect("spawn product");
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "feature list failed in canonical-layout repo: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // Migration command exits cleanly on an already-canonical layout.
    let mig = Command::new(&bin)
        .args(["migrate", "consolidate"])
        .current_dir(dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("spawn product migrate");
    assert_eq!(
        mig.status.code().unwrap_or(-1),
        0,
        "migrate consolidate on canonical layout: stderr={}",
        String::from_utf8_lossy(&mig.stderr)
    );

    // Legacy fallback still resolves — minimum signal that legacy support
    // remains live (exit-criteria item 4).
    let legacy_dir = tempfile::tempdir().expect("tempdir2");
    let legacy_toml = r#"name = "legacy"
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
"#;
    std::fs::write(legacy_dir.path().join("product.toml"), legacy_toml).expect("write legacy");
    for sub in [
        "docs/features",
        "docs/adrs",
        "docs/tests",
        "docs/dependencies",
        "docs/graph",
    ] {
        std::fs::create_dir_all(legacy_dir.path().join(sub)).expect("mkdir docs");
    }
    let out = Command::new(&bin)
        .args(["feature", "list"])
        .current_dir(legacy_dir.path())
        .stdin(Stdio::null())
        .output()
        .expect("spawn product");
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "feature list failed in legacy-layout repo: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}
