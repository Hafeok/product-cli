//! Integration tests — init.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_002_binary_compiles_x86() {
    // Skip if the musl target is not installed
    let check = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output();
    if let Ok(out) = check {
        let installed = String::from_utf8_lossy(&out.stdout);
        if !installed.contains("x86_64-unknown-linux-musl") {
            eprintln!("Skipping tc_002: x86_64-unknown-linux-musl target not installed");
            return;
        }
    }

    let output = Command::new("cargo")
        .args(["build", "--release", "--target", "x86_64-unknown-linux-musl"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo build");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cargo build --release --target x86_64-unknown-linux-musl failed:\n{}",
        stderr
    );
}

#[test]
fn tc_004_cargo_build_release() {
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo build");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cargo build --release failed:\n{}",
        stderr
    );
}

#[test]
fn tc_001_binary_compiles_arm64() {
    // Skip if the ARM64 target is not installed
    let check = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output();
    if let Ok(out) = check {
        let installed = String::from_utf8_lossy(&out.stdout);
        if !installed.contains("aarch64-unknown-linux-gnu") {
            eprintln!("Skipping tc_001: aarch64-unknown-linux-gnu target not installed");
            return;
        }
    }

    let output = Command::new("cargo")
        .args(["build", "--release", "--target", "aarch64-unknown-linux-gnu"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo build");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cargo build --release --target aarch64-unknown-linux-gnu failed:\n{}",
        stderr
    );
    // Check for zero warnings (allow "Compiling" and "Finished" lines)
    let has_warnings = stderr.lines().any(|l| l.starts_with("warning"));
    assert!(
        !has_warnings,
        "Expected zero warnings, got:\n{}",
        stderr
    );
}

#[test]
fn tc_003_binary_no_deps() {
    // Build check: verify the debug binary has minimal deps
    // On a musl-static build this would show "not a dynamic executable"
    // On a glibc build, only libc/libm/ld-linux are expected
    let h = Harness::new();
    let out = Command::new("ldd")
        .arg(&h.bin)
        .output();
    match out {
        Ok(output) => {
            let ldd_output = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            // Either statically linked (not a dynamic executable) or only libc deps
            let is_static = ldd_output.contains("not a dynamic executable")
                || ldd_output.contains("statically linked")
                || stderr.contains("not a dynamic executable");

            if !is_static {
                // Check that all deps are libc-related
                for line in ldd_output.lines() {
                    let line = line.trim();
                    if line.is_empty() { continue; }
                    // Allowed: libc, libm, libdl, libpthread, librt, libgcc_s, ld-linux, linux-vdso
                    let allowed = ["libc.", "libm.", "libdl.", "libpthread.", "librt.",
                                   "libgcc_s.", "ld-linux", "linux-vdso", "linux-gate",
                                   "/lib64/ld-", "/lib/ld-"];
                    let is_allowed = allowed.iter().any(|a| line.contains(a));
                    assert!(
                        is_allowed,
                        "Unexpected dynamic dependency: {}",
                        line
                    );
                }
            }
            // If static, test passes automatically
        }
        Err(_) => {
            // ldd not available (e.g., macOS) — skip
            eprintln!("ldd not available, skipping TC-003");
        }
    }
}

#[test]
fn tc_070_tmp_cleanup_on_startup() {
    let h = Harness::new();

    // Create leftover .product-tmp.* files in artifact directories
    h.write("docs/features/.test.product-tmp.99999", "leftover");
    h.write("docs/adrs/.adr.product-tmp.88888", "leftover");
    h.write("docs/tests/.tc.product-tmp.77777", "leftover");

    // Run a read-only command
    let out = h.run(&["feature", "list"]);
    assert_eq!(out.exit_code, 0, "feature list should succeed: {}", out.stderr);

    // All tmp files should have been cleaned up
    assert!(
        !h.exists("docs/features/.test.product-tmp.99999"),
        "features tmp should be cleaned"
    );
    assert!(
        !h.exists("docs/adrs/.adr.product-tmp.88888"),
        "adrs tmp should be cleaned"
    );
    assert!(
        !h.exists("docs/tests/.tc.product-tmp.77777"),
        "tests tmp should be cleaned"
    );
}

#[test]
fn tc_431_init_creates_product_toml_and_directory_skeleton() {
    let h = Harness::new_bare();
    let out = h.run(&["init", "--yes"]);
    out.assert_exit(0);

    // 1. product.toml exists and contains all required sections
    assert!(h.exists("product.toml"), "product.toml should exist");
    let toml_content = h.read("product.toml");
    assert!(toml_content.contains("name = "), "should contain name");
    assert!(
        toml_content.contains("schema-version = "),
        "should contain schema-version"
    );
    assert!(toml_content.contains("[paths]"), "should contain [paths]");
    assert!(
        toml_content.contains("[prefixes]"),
        "should contain [prefixes]"
    );
    assert!(toml_content.contains("[phases]"), "should contain [phases]");
    assert!(
        toml_content.contains("[domains]"),
        "should contain [domains]"
    );
    assert!(toml_content.contains("[mcp]"), "should contain [mcp]");

    // 2. name defaults to directory name
    let dir_name = h
        .dir
        .path()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    assert!(
        toml_content.contains(&format!("name = \"{}\"", dir_name)),
        "name should default to directory name '{}', got:\n{}",
        dir_name,
        toml_content
    );

    // 3. schema-version equals CURRENT_SCHEMA_VERSION (1)
    assert!(
        toml_content.contains("schema-version = \"1\""),
        "schema-version should be 1"
    );

    // 4. Directories exist
    assert!(h.exists("docs/features"), "docs/features/ should exist");
    assert!(h.exists("docs/adrs"), "docs/adrs/ should exist");
    assert!(h.exists("docs/tests"), "docs/tests/ should exist");
    assert!(h.exists("docs/graph"), "docs/graph/ should exist");

    // 5. Exit code 0 — already asserted

    // 6. Stdout contains summary of created files
    out.assert_stdout_contains("product.toml");
    out.assert_stdout_contains("docs/features/");
    out.assert_stdout_contains("docs/adrs/");
    out.assert_stdout_contains("docs/tests/");
    out.assert_stdout_contains("docs/graph/");
}

#[test]
fn tc_432_init_interactive_mode_prompts_for_name_and_domains() {
    let h = Harness::new_bare();

    // Stdin input:
    //   Line 1: project name "my-interactive-proj"
    //   Line 2: product description (blank to skip)
    //   Line 3: select domain 1 (security)
    //   Line 4: blank (no custom domain)
    //   Line 5: blank (no MCP write tools — default N)
    //   Line 6: blank (default port)
    let stdin_input = "my-interactive-proj\n\n1\n\n\n\n";
    let out = h.run_with_stdin(&["init"], stdin_input);

    // 4. Exit code is 0
    out.assert_exit(0);

    // 1. product.toml contains the provided project name
    let toml_content = h.read("product.toml");
    assert!(
        toml_content.contains("name = \"my-interactive-proj\""),
        "should contain provided project name, got:\n{}",
        toml_content
    );

    // 2. The selected domain (security) appears in [domains]
    assert!(
        toml_content.contains("security"),
        "should contain selected domain 'security', got:\n{}",
        toml_content
    );

    // 3. Default prefixes are preserved
    assert!(
        toml_content.contains("feature = \"FT\""),
        "feature prefix should be FT"
    );
    assert!(
        toml_content.contains("adr = \"ADR\""),
        "adr prefix should be ADR"
    );
    assert!(
        toml_content.contains("test = \"TC\""),
        "test prefix should be TC"
    );
}

#[test]
fn tc_433_init_yes_uses_defaults_without_prompts() {
    let h = Harness::new_bare();

    // Run with --yes and --name, stdin closed (no tty)
    let out = h.run(&["init", "--yes", "--name", "test-project"]);

    // 1. Command completes without blocking
    // (if it blocked, the test would timeout)

    // 5. Exit code is 0
    out.assert_exit(0);

    // 2. product.toml exists with name = "test-project"
    let toml_content = h.read("product.toml");
    assert!(
        toml_content.contains("name = \"test-project\""),
        "should contain name = \"test-project\", got:\n{}",
        toml_content
    );

    // 3. [domains] section present but empty
    assert!(
        toml_content.contains("[domains]"),
        "should contain [domains] section"
    );
    // No domain entries — check there's nothing between [domains] and [mcp]
    let domains_idx = toml_content.find("[domains]").unwrap_or(0);
    let mcp_idx = toml_content.find("[mcp]").unwrap_or(toml_content.len());
    let between = &toml_content[domains_idx + "[domains]".len()..mcp_idx];
    let domain_lines: Vec<&str> = between
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect();
    assert!(
        domain_lines.is_empty(),
        "domains section should be empty, got lines: {:?}",
        domain_lines
    );

    // 4. [mcp] section with write = false and port = 7777
    assert!(
        toml_content.contains("write = false"),
        "mcp write should be false"
    );
    assert!(
        toml_content.contains("port = 7777"),
        "mcp port should be 7777"
    );
}

#[test]
fn tc_434_init_errors_on_existing_product_toml_without_force() {
    let h = Harness::new_bare();
    let original_content = "name = \"original\"\nschema-version = \"1\"\n";
    h.write("product.toml", original_content);

    let out = h.run(&["init", "--yes"]);

    // 1. Exit code is 1
    out.assert_exit(1);

    // 2. Stderr contains "product.toml already exists"
    out.assert_stderr_contains("product.toml already exists");

    // 3. Stderr contains a hint mentioning --force
    assert!(
        out.stderr.contains("--force"),
        "stderr should mention --force, got:\n{}",
        out.stderr
    );

    // 4. Original content is unchanged
    let content = h.read("product.toml");
    assert_eq!(
        content, original_content,
        "original product.toml should be unchanged"
    );
}

#[test]
fn tc_435_init_force_overwrites_existing_product_toml() {
    let h = Harness::new_bare();
    h.write("product.toml", "name = \"old\"\nschema-version = \"1\"\n");

    // Create an existing artifact directory to verify it's not deleted
    std::fs::create_dir_all(h.dir.path().join("docs/features")).expect("mkdir");
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\n---\n");

    let out = h.run(&["init", "--yes", "--force", "--name", "new-project"]);

    // 1. Exit code is 0
    out.assert_exit(0);

    // 2. product.toml now contains name = "new-project"
    let toml_content = h.read("product.toml");
    assert!(
        toml_content.contains("name = \"new-project\""),
        "should contain new name, got:\n{}",
        toml_content
    );

    // 3. Old content is fully replaced
    assert!(
        !toml_content.contains("name = \"old\""),
        "old name should be gone"
    );

    // 4. Existing artifact directories and files are not deleted
    assert!(
        h.exists("docs/features/FT-001-test.md"),
        "existing feature file should be preserved"
    );
}

#[test]
fn tc_436_init_appends_to_existing_gitignore() {
    let h = Harness::new_bare();
    h.write(".gitignore", "target/\n");

    let out = h.run(&["init", "--yes"]);
    out.assert_exit(0);

    // 1. .gitignore still contains target/ (original content preserved)
    let gitignore = h.read(".gitignore");
    assert!(
        gitignore.contains("target/"),
        "original target/ should be preserved, got:\n{}",
        gitignore
    );

    // 2. .gitignore now also contains docs/graph/
    assert!(
        gitignore.contains("docs/graph/"),
        "should contain docs/graph/, got:\n{}",
        gitignore
    );

    // 3. Running init --force --yes again does not duplicate docs/graph/
    let out2 = h.run(&["init", "--force", "--yes"]);
    out2.assert_exit(0);
    let gitignore2 = h.read(".gitignore");
    let count = gitignore2.matches("docs/graph/").count();
    assert_eq!(
        count, 1,
        "docs/graph/ should appear exactly once after second init, found {} times in:\n{}",
        count, gitignore2
    );
}

#[test]
fn tc_437_init_creates_gitignore_when_absent() {
    let h = Harness::new_bare();
    assert!(!h.exists(".gitignore"), ".gitignore should not exist initially");

    let out = h.run(&["init", "--yes"]);
    out.assert_exit(0);

    // 1. .gitignore is created
    assert!(h.exists(".gitignore"), ".gitignore should be created");

    // 2. .gitignore contains docs/graph/
    let gitignore = h.read(".gitignore");
    assert!(
        gitignore.contains("docs/graph/"),
        "should contain docs/graph/, got:\n{}",
        gitignore
    );

    // 3. .gitignore contains a comment header with "Product CLI"
    assert!(
        gitignore.contains("# Product CLI"),
        "should contain Product CLI comment header, got:\n{}",
        gitignore
    );
}

#[test]
fn tc_367_platform_verify_cross_cutting() {
    let h = Harness::new();
    // Cross-cutting ADR with a TC that has a runner
    h.write("docs/adrs/ADR-001-cross.md", "\
---
id: ADR-001
title: Cross Cutting ADR
status: accepted
scope: cross-cutting
---

Cross-cutting.
");
    // Feature-specific ADR with its own TC
    h.write("docs/adrs/ADR-002-domain.md", "\
---
id: ADR-002
title: Domain ADR
status: accepted
scope: domain
---

Domain.
");
    h.write("docs/features/FT-001-test.md", "\
---
id: FT-001
title: Test Feature
phase: 1
status: planned
adrs:
- ADR-001
- ADR-002
tests:
- TC-002
---

Feature.
");
    // TC linked to cross-cutting ADR (should be run by --platform)
    h.write("docs/tests/TC-001-cross.md", "\
---
id: TC-001
title: Cross Cutting TC
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-001
phase: 1
runner: cargo-test
runner-args: tc_001_binary_compiles_arm64
---

Cross-cutting TC.
");
    // Feature-specific TC (should NOT be run by --platform)
    h.write("docs/tests/TC-002-feature.md", "\
---
id: TC-002
title: Feature Specific TC
type: scenario
status: unimplemented
validates:
  features:
  - FT-001
  adrs:
  - ADR-002
phase: 1
runner: cargo-test
runner-args: tc_002_binary_compiles_x86
---

Feature-specific TC.
");
    let out = h.run(&["verify", "--platform"]);
    // Should run and process cross-cutting TCs
    // The exit code may vary depending on test outcome, but it should execute
    assert!(out.exit_code == 0 || out.exit_code == 1, "verify --platform should run. Got exit {}.\nstdout: {}\nstderr: {}",
        out.exit_code, out.stdout, out.stderr);

    // Should mention running platform TCs
    assert!(out.stdout.contains("platform TC") || out.stdout.contains("TC-001"),
        "Should run cross-cutting TCs. Got:\n{}", out.stdout);
}

#[test]
fn tc_467_test_runner_validates_runner_enum() {
    let h = fixture_with_domains();
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: []\n  adrs: []\nphase: 1\n---\n\nDesc.\n");

    // Invalid runner → exit 1 with E001
    let out = h.run(&["test", "runner", "TC-001", "--runner", "invalid-runner", "--args", "test_name"]);
    out.assert_exit(1);
    out.assert_stderr_contains("E001");

    // Valid runners → exit 0
    for runner in &["cargo-test", "bash", "pytest", "custom"] {
        let out = h.run(&["test", "runner", "TC-001", "--runner", runner]);
        out.assert_exit(0);
        let content = h.read("docs/tests/TC-001-test.md");
        assert!(content.contains(&format!("runner: {}", runner)),
            "runner should be set to {} in front-matter, got:\n{}", runner, content);
    }
}

