//! Integration tests — tags.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_451_tags_list_all() {
    let h = Harness::new();
    git_init_with_commit(&h);

    // Create two annotated tags
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete"])
        .current_dir(h.dir.path()).output().expect("tag 1");
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-002/complete", "-m", "FT-002 complete"])
        .current_dir(h.dir.path()).output().expect("tag 2");

    let out = h.run(&["tags", "list"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("complete");
    out.assert_stdout_contains("FT-002");

    // JSON variant
    let json_out = h.run(&["tags", "list", "--format", "json"]);
    json_out.assert_exit(0);
    let parsed: serde_json::Value = serde_json::from_str(&json_out.stdout)
        .unwrap_or_else(|e| panic!("Invalid JSON: {} stdout: {}", e, json_out.stdout));
    assert!(parsed.as_array().map(|a| a.len() >= 2).unwrap_or(false), "Should have >=2 tags");
}

#[test]
fn tc_452_tags_list_filter_feature() {
    let h = Harness::new();
    git_init_with_commit(&h);

    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete"])
        .current_dir(h.dir.path()).output().expect("tag 1");
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete-v2", "-m", "FT-001 v2"])
        .current_dir(h.dir.path()).output().expect("tag 2");
    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-002/complete", "-m", "FT-002 complete"])
        .current_dir(h.dir.path()).output().expect("tag 3");

    let out = h.run(&["tags", "list", "--feature", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001/complete");
    assert!(!out.stdout.contains("FT-002"), "Should not contain FT-002.\nStdout: {}", out.stdout);
}

#[test]
fn tc_453_tags_list_filter_type() {
    let h = Harness::new();
    git_init_with_commit(&h);

    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete"])
        .current_dir(h.dir.path()).output().expect("tag 1");
    std::process::Command::new("git")
        .args(["tag", "-a", "product/ADR-002/accepted", "-m", "ADR-002 accepted"])
        .current_dir(h.dir.path()).output().expect("tag 2");

    let out = h.run(&["tags", "list", "--type", "complete"]);
    out.assert_exit(0);
    out.assert_stdout_contains("FT-001");
    out.assert_stdout_contains("complete");
    assert!(!out.stdout.contains("ADR-002"), "Should not contain ADR-002.\nStdout: {}", out.stdout);
    assert!(!out.stdout.contains("accepted"), "Should not contain 'accepted'.\nStdout: {}", out.stdout);
}

#[test]
fn tc_454_tags_show_feature() {
    let h = Harness::new();
    git_init_with_commit(&h);

    std::process::Command::new("git")
        .args(["tag", "-a", "product/FT-001/complete", "-m", "FT-001 complete: 2/2 TCs passing (TC-001, TC-002)"])
        .current_dir(h.dir.path()).output().expect("tag");

    let out = h.run(&["tags", "show", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("product/FT-001/complete");
    // Tag message should appear
    assert!(
        out.stdout.contains("TC-001") || out.stdout.contains("FT-001 complete"),
        "Should show tag message.\nStdout: {}", out.stdout
    );

    // Not-found case
    let out2 = h.run(&["tags", "show", "FT-999"]);
    assert!(out2.exit_code != 0 || out2.stderr.contains("No tags found"),
        "Should indicate no tags found for FT-999");
}

#[test]
fn tc_458_tags_config_defaults() {
    // No [tags] section — should use defaults
    let h = Harness::new();
    git_init_with_commit(&h);

    // Tags list should work without [tags] section in product.toml
    let out = h.run(&["tags", "list"]);
    out.assert_exit(0);

    // Verify with explicit config
    h.write(
        "product.toml",
        "name = \"test\"\nschema-version = \"1\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\ndependencies = \"docs/dependencies\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"\ndependency = \"DEP\"\n[tags]\nauto-push-tags = false\nimplementation-depth = 30\n",
    );
    git_add_commit(&h, "add tags config");

    let out2 = h.run(&["tags", "list"]);
    out2.assert_exit(0); // Parses correctly, no crash
}

#[test]
fn tc_459_tag_namespace_format() {
    let h = fixture_verify_with_git();
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);

    // Get the tag and verify format
    let tag_out = std::process::Command::new("git")
        .args(["tag", "-l", "product/*"])
        .current_dir(h.dir.path())
        .output()
        .expect("git tag -l");
    let tags = String::from_utf8_lossy(&tag_out.stdout);
    let re = regex::Regex::new(r"^product/[A-Z]+-\d{3,}/[a-z][a-z0-9-]*$").expect("regex");
    for line in tags.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        assert!(re.is_match(line), "Tag '{}' should match product/{{ID}}/{{EVENT}} format", line);
    }
}

#[test]
fn tc_460_tag_based_drift_detection_exit() {
    // 1. Verify creates a completion tag
    let h = fixture_verify_with_git();
    let out = h.run(&["verify", "FT-001"]);
    out.assert_exit(0);
    out.assert_stdout_contains("Tagged: product/FT-001/complete");

    // 2. Tags list works
    let list_out = h.run(&["tags", "list"]);
    list_out.assert_exit(0);
    list_out.assert_stdout_contains("FT-001");

    // 3. Tags show works
    let show_out = h.run(&["tags", "show", "FT-001"]);
    show_out.assert_exit(0);
    show_out.assert_stdout_contains("product/FT-001/complete");

    // 4. Drift check with tag works
    let drift_out = h.run(&["drift", "check", "FT-001"]);
    drift_out.assert_exit(0);

    // 5. All-complete flag works
    let all_out = h.run(&["drift", "check", "--all-complete"]);
    all_out.assert_exit(0);
}

