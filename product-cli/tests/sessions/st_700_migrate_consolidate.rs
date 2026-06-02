//! TC-700 — `product migrate consolidate` moves legacy layout to canonical
//! `.product/` directory (FT-057, ADR-048).

use super::harness::Session;
use super::repo_scaffold::{git, init_git, git_commit_all};

/// Build a legacy-layout repo and run consolidation end to end.
#[test]
fn tc_700_product_migrate_consolidate_moves_legacy_layout() {
    let mut s = Session::new();

    // Session::new() already wrote a legacy product.toml at root and the
    // docs/ skeleton. Apply a request that creates one feature, one ADR, and
    // one TC so the move has real artifacts to relocate.
    let r = s.apply(
        r#"type: create
schema-version: 1
reason: "FT-057 fixture"
artifacts:
  - type: feature
    ref: ft-x
    title: Smoke
    phase: 1
    domains: [api]
  - type: adr
    ref: adr-x
    title: Smoke ADR
    domains: [api]
    scope: domain
    status: proposed
  - type: tc
    ref: tc-x
    title: Smoke TC
    test-type: scenario
    validates:
      features: [ref:ft-x]
      adrs: [ref:adr-x]
"#,
    );
    r.assert_applied();
    let ft_id = r.id_for("ft-x");

    // Add some legacy files outside docs/: gaps.json + benchmarks/prompts/.
    s.write("gaps.json", "{}\n");
    s.write("benchmarks/prompts/implement-v1.md", "# legacy implement prompt\n");

    // git init + commit so the dirty-tree guard has a clean baseline.
    init_git(&s);
    git_commit_all(&s, "fixture");

    // ----- Dry-run -----
    let dry = s.run(&["migrate", "consolidate"]);
    dry.assert_exit(0);
    assert!(
        dry.stdout.contains("Planned consolidation"),
        "dry-run output: {}",
        dry.stdout
    );
    // Filesystem unchanged
    s.assert_file_exists("product.toml")
        .assert_file_exists(&format!("docs/features/{}-smoke.md", ft_id))
        .assert_file_exists("benchmarks/prompts/implement-v1.md")
        .assert_file_exists("gaps.json")
        .assert_file_missing(".product/config.toml");

    // ----- Apply -----
    let apply = s.run(&["migrate", "consolidate", "--apply"]);
    apply.assert_exit(0);

    s.assert_file_missing("product.toml")
        .assert_file_exists(".product/config.toml")
        .assert_file_exists(&format!(".product/features/{}-smoke.md", ft_id))
        .assert_file_missing(&format!("docs/features/{}-smoke.md", ft_id))
        .assert_file_exists(".product/prompts/implement-v1.md")
        .assert_file_missing("benchmarks/prompts/implement-v1.md")
        .assert_file_exists(".product/gaps.json")
        .assert_file_missing("gaps.json");

    // The rewritten config carries canonical [paths].
    let cfg = s.read(".product/config.toml");
    assert!(cfg.contains(".product/features"), "config:\n{}", cfg);
    assert!(cfg.contains(".product/prompts"), "config:\n{}", cfg);
    assert!(cfg.contains(".product/gaps.json"), "config:\n{}", cfg);

    // .gitignore picks up the canonical generated paths.
    let gi = s.read(".gitignore");
    assert!(gi.contains(".product/graph/"), ".gitignore:\n{}", gi);
    assert!(gi.contains(".product/sessions/"), ".gitignore:\n{}", gi);

    // The new request log carries one (or more) `migrate` entries — the last
    // one is the consolidate-paths entry.
    let log = s.read(".product/requests.jsonl");
    assert!(
        log.contains("consolidate-paths"),
        "request log missing consolidate sentinel:\n{}",
        log
    );

    // graph check should still be clean.
    s.assert_graph_clean();

    // feature list still finds the same feature ID.
    let list = s.run(&["feature", "list"]);
    list.assert_exit(0);
    assert!(list.stdout.contains(&ft_id), "feature list:\n{}", list.stdout);

    // ----- Idempotency: re-running --apply is a no-op -----
    let pre_log = s.read(".product/requests.jsonl");
    let again = s.run(&["migrate", "consolidate", "--apply"]);
    again.assert_exit(0);
    assert!(
        again.stdout.contains("Already canonical"),
        "expected no-op message, got:\n{}",
        again.stdout
    );
    let post_log = s.read(".product/requests.jsonl");
    assert_eq!(pre_log, post_log, "idempotent re-run must not append a log entry");
}

/// Dirty-tree guard: refuses to migrate when uncommitted changes touch a
/// path the migration would move; `--force-uncommitted` overrides.
#[test]
fn tc_700b_consolidate_dirty_tree_guard() {
    let mut s = Session::new();

    s.apply(
        r#"type: create
schema-version: 1
reason: "FT-057 dirty fixture"
artifacts:
  - type: feature
    ref: ft-d
    title: Dirty
    phase: 1
    domains: [api]
"#,
    )
    .assert_applied();

    init_git(&s);
    git_commit_all(&s, "fixture");

    // Modify a feature file but don't commit.
    let path = format!(
        "docs/features/{}-dirty.md",
        s.run(&["feature", "list"])
            .stdout
            .lines()
            .filter_map(|l| l.split_whitespace().find(|w| w.starts_with("FT-")))
            .map(|s| s.to_string())
            .next()
            .unwrap_or_else(|| "FT-001".into())
    );
    let original = s.read(&path);
    s.write(&path, &(original.clone() + "\nedit\n"));

    // Sanity — git sees the modification.
    let porcelain = git(&s, &["status", "--porcelain"]);
    let porcelain_str = String::from_utf8_lossy(&porcelain.stdout);
    assert!(
        !porcelain_str.is_empty(),
        "expected dirty git tree, got: {}",
        porcelain_str
    );

    // Without --force-uncommitted, consolidate refuses.
    let blocked = s.run(&["migrate", "consolidate", "--apply"]);
    assert_ne!(blocked.exit_code, 0, "expected non-zero exit, got 0:\n{}", blocked.stdout);
    let combined = format!("{}{}", blocked.stdout, blocked.stderr);
    assert!(
        combined.contains("uncommitted"),
        "expected 'uncommitted' in output, got:\nstdout: {}\nstderr: {}",
        blocked.stdout,
        blocked.stderr
    );

    // With --force-uncommitted, it goes through.
    let forced = s.run(&["migrate", "consolidate", "--apply", "--force-uncommitted"]);
    forced.assert_exit(0);
    s.assert_file_exists(".product/config.toml");
}
