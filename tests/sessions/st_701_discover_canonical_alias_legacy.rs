//! TC-701 — `ProductConfig::discover` walks `.product/config.toml`, then
//! `.product/product.toml`, then `product.toml` (FT-057, ADR-048).

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

fn minimal_toml(name: &str) -> String {
    format!(
        r#"name = "{name}"
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
"#
    )
}

/// Make a repo with the given config layout, then run `product feature list`
/// in it (or in `subdir` if provided). Returns the command output.
fn run_in_layout(
    layout: &[(&str, String)],
    subdir: Option<&str>,
) -> (i32, String, String, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    for (rel, content) in layout {
        let p = dir.path().join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&p, content).expect("write");
    }
    for sub in ["docs/features", "docs/adrs", "docs/tests", "docs/dependencies"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir docs");
    }
    let cwd = match subdir {
        Some(s) => {
            std::fs::create_dir_all(dir.path().join(s)).expect("mkdir subdir");
            dir.path().join(s)
        }
        None => dir.path().to_path_buf(),
    };
    let bin = product_bin();
    let out = Command::new(&bin)
        .args(["feature", "list"])
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .output()
        .expect("spawn product");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        dir,
    )
}

#[test]
fn tc_701a_canonical_layout_loads() {
    let (code, _stdout, stderr, _dir) = run_in_layout(
        &[(".product/config.toml", minimal_toml("repo-a"))],
        None,
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn tc_701b_alias_layout_loads() {
    let (code, _stdout, stderr, _dir) = run_in_layout(
        &[(".product/product.toml", minimal_toml("repo-b"))],
        None,
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn tc_701c_legacy_layout_loads() {
    let (code, _stdout, stderr, _dir) = run_in_layout(
        &[("product.toml", minimal_toml("repo-c"))],
        None,
    );
    assert_eq!(code, 0, "stderr: {}", stderr);
}

#[test]
fn tc_701d_canonical_wins_over_legacy() {
    // Repo D: both .product/config.toml AND product.toml exist with different
    // names. We use `product schema --type feature` which doesn't depend on
    // the name field — instead we use `feature list` and inspect the lock /
    // load by loading the config via the library directly. The simpler
    // assertion: discover() should succeed and the config returned is the
    // canonical one.
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join(".product")).expect("mkdir");
    std::fs::write(
        dir.path().join(".product/config.toml"),
        minimal_toml("canonical-wins"),
    )
    .expect("write canonical");
    std::fs::write(dir.path().join("product.toml"), minimal_toml("legacy-loses"))
        .expect("write legacy");
    for sub in ["docs/features", "docs/adrs", "docs/tests", "docs/dependencies"] {
        std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir docs");
    }

    // Library-level assertion — discover() walks the canonical path first.
    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(dir.path()).expect("chdir");
    let result = product_lib::config::ProductConfig::discover();
    let restored = std::env::set_current_dir(cwd);
    let _ = restored;
    let (config, root) = result.expect("discover should succeed");
    assert_eq!(config.name, "canonical-wins", "canonical must win over legacy");
    assert_eq!(root, dir.path().canonicalize().expect("canon root"));
}

#[test]
fn tc_701e_walks_up_from_subdirectory() {
    let (code, _stdout, stderr, _dir) = run_in_layout(
        &[(".product/config.toml", minimal_toml("walk-up"))],
        Some("src/commands"),
    );
    assert_eq!(code, 0, "running from subdirectory failed: {}", stderr);
}
