//! Integration tests for code structure and quality check scripts (ADR-029).
//! Tests TC-369 through TC-380 (scenario) and TC-402 (exit-criteria).

use std::path::PathBuf;
use std::process::Command;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn script_path(name: &str) -> PathBuf {
    project_root().join("scripts/checks").join(name)
}

/// Helper: generate N lines of `// filler` comments
fn filler_lines(n: usize) -> String {
    (0..n).map(|i| format!("// line {i}\n")).collect()
}

/// Helper: generate a Rust function with exactly `stmt_count` statement lines.
/// The fn signature counts as 1, then (stmt_count - 1) let bindings inside.
fn rust_fn_with_stmts(name: &str, stmt_count: usize) -> String {
    let mut s = format!("fn {name}() {{\n");
    for i in 0..(stmt_count.saturating_sub(1)) {
        s.push_str(&format!("    let _x{i} = {i};\n"));
    }
    s.push_str("}\n");
    s
}

// =============================================================================
// TC-369: file_length_passes
// =============================================================================
#[test]
fn tc_369_file_length_passes() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Create files well under 300 lines
    let content = format!("//! Test module.\n{}", filler_lines(100));
    std::fs::write(dir.path().join("src/lib.rs"), &content).expect("write");
    std::fs::write(dir.path().join("src/utils.rs"), &content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("file-length.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "Expected exit 0, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(stdout.contains("OK"), "Expected OK message, got: {stdout}");
}

// =============================================================================
// TC-370: file_length_warn
// =============================================================================
#[test]
fn tc_370_file_length_warn() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Create a 350-line file (between 300 warn and 400 hard)
    let content = format!("//! Test module.\n{}", filler_lines(349));
    std::fs::write(dir.path().join("src/big_file.rs"), &content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("file-length.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        2,
        "Expected exit 2 (warning), got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(
        stdout.contains("big_file.rs"),
        "Expected file name in output, got: {stdout}"
    );
}

// =============================================================================
// TC-371: file_length_fail
// =============================================================================
#[test]
fn tc_371_file_length_fail() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Create a 450-line file (over 400 hard limit)
    let content = format!("//! Test module.\n{}", filler_lines(449));
    std::fs::write(dir.path().join("src/huge_file.rs"), &content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("file-length.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        1,
        "Expected exit 1 (hard fail), got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(
        stdout.contains("huge_file.rs"),
        "Expected file name in output, got: {stdout}"
    );
    assert!(
        stdout.contains("450"),
        "Expected line count in output, got: {stdout}"
    );
}

// =============================================================================
// TC-372: function_length_passes
// =============================================================================
#[test]
fn tc_372_function_length_passes() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Functions with under 30 statement lines each
    let content = format!(
        "//! Test module.\n{}\n{}",
        rust_fn_with_stmts("short_fn", 10),
        rust_fn_with_stmts("medium_fn", 25),
    );
    std::fs::write(dir.path().join("src/lib.rs"), &content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("function-length.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "Expected exit 0, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
}

// =============================================================================
// TC-373: function_length_warn
// =============================================================================
#[test]
fn tc_373_function_length_warn() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Function with 35 statement lines (between 30 warn and 40 hard)
    let content = format!("//! Test module.\n{}", rust_fn_with_stmts("warn_fn", 35));
    std::fs::write(dir.path().join("src/lib.rs"), &content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("function-length.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        2,
        "Expected exit 2 (warning), got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
}

// =============================================================================
// TC-374: function_length_fail
// =============================================================================
#[test]
fn tc_374_function_length_fail() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Function with 45 statement lines (over 40 hard limit)
    let content = format!("//! Test module.\n{}", rust_fn_with_stmts("long_fn", 45));
    std::fs::write(dir.path().join("src/lib.rs"), &content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("function-length.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        1,
        "Expected exit 1 (hard fail), got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    // Should contain file path and line number
    assert!(
        stdout.contains("src/lib.rs"),
        "Expected file path in output, got: {stdout}"
    );
    assert!(
        stdout.contains("45"),
        "Expected statement count in output, got: {stdout}"
    );
}

// =============================================================================
// TC-375: module_structure_passes
// =============================================================================
#[test]
fn tc_375_module_structure_passes() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Create all required module directories
    for module in &["graph", "parse", "context", "commands", "verify", "mcp", "io"] {
        std::fs::create_dir_all(dir.path().join(format!("src/{module}"))).expect("mkdir");
    }

    // Create main.rs under 80 lines
    let main_content = "fn main() {}\n".repeat(10);
    std::fs::write(dir.path().join("src/main.rs"), &main_content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("module-structure.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "Expected exit 0, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(stdout.contains("OK"), "Expected OK message, got: {stdout}");
}

// =============================================================================
// TC-376: module_structure_missing
// =============================================================================
#[test]
fn tc_376_module_structure_missing() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Create most modules but NOT graph/
    for module in &["parse", "context", "commands", "verify", "mcp", "io"] {
        std::fs::create_dir_all(dir.path().join(format!("src/{module}"))).expect("mkdir");
    }

    let main_content = "fn main() {}\n";
    std::fs::write(dir.path().join("src/main.rs"), main_content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("module-structure.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        1,
        "Expected exit 1, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(
        stdout.contains("src/graph/"),
        "Expected missing module name in output, got: {stdout}"
    );
}

// =============================================================================
// TC-377: module_structure_main_too_long
// =============================================================================
#[test]
fn tc_377_module_structure_main_too_long() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Create all required modules
    for module in &["graph", "parse", "context", "commands", "verify", "mcp", "io"] {
        std::fs::create_dir_all(dir.path().join(format!("src/{module}"))).expect("mkdir");
    }

    // Create main.rs with 100 lines (over 80 limit)
    let main_content = "// line\n".repeat(100);
    std::fs::write(dir.path().join("src/main.rs"), &main_content).expect("write");

    let output = Command::new("bash")
        .arg(script_path("module-structure.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        1,
        "Expected exit 1, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(
        stdout.contains("100"),
        "Expected line count in output, got: {stdout}"
    );
}

// =============================================================================
// TC-378: single_responsibility_passes
// =============================================================================
#[test]
fn tc_378_single_responsibility_passes() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // Files with valid //! doc comments (no "and")
    std::fs::write(
        dir.path().join("src/parser.rs"),
        "//! YAML front-matter parser for artifact types.\nfn parse() {}\n",
    )
    .expect("write");
    std::fs::write(
        dir.path().join("src/graph.rs"),
        "//! Knowledge graph construction from parsed artifacts.\nfn build() {}\n",
    )
    .expect("write");

    // mod.rs and main.rs are excluded from the check
    std::fs::write(dir.path().join("src/mod.rs"), "pub mod parser;\n").expect("write");
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").expect("write");

    let output = Command::new("bash")
        .arg(script_path("single-responsibility.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "Expected exit 0, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(stdout.contains("OK"), "Expected OK message, got: {stdout}");
}

// =============================================================================
// TC-379: single_responsibility_missing
// =============================================================================
#[test]
fn tc_379_single_responsibility_missing() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // File without //! first line
    std::fs::write(
        dir.path().join("src/bad_file.rs"),
        "use std::io;\nfn main() {}\n",
    )
    .expect("write");

    let output = Command::new("bash")
        .arg(script_path("single-responsibility.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        1,
        "Expected exit 1, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(
        stdout.contains("bad_file.rs"),
        "Expected file name in output, got: {stdout}"
    );
}

// =============================================================================
// TC-380: single_responsibility_and
// =============================================================================
#[test]
fn tc_380_single_responsibility_and() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");

    // File with "and" in the doc comment
    std::fs::write(
        dir.path().join("src/multi.rs"),
        "//! Graph construction and traversal.\nfn build() {}\n",
    )
    .expect("write");

    let output = Command::new("bash")
        .arg(script_path("single-responsibility.sh"))
        .current_dir(dir.path())
        .output()
        .expect("run script");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code().unwrap(),
        1,
        "Expected exit 1, got {}. Output: {}",
        output.status.code().unwrap(),
        stdout
    );
    assert!(
        stdout.contains("Graph construction and traversal"),
        "Expected violating comment in output, got: {stdout}"
    );
}

// =============================================================================
// TC-402: All source files under 400 lines and all quality checks pass
// =============================================================================
#[test]
fn tc_402_all_source_files_under_400_lines_and_all_quality_checks_pass() {
    let root = project_root();

    // Run file-length check (exit 1 = hard fail, exit 0 or 2 = ok)
    let output = Command::new("bash")
        .arg(root.join("scripts/checks/file-length.sh"))
        .current_dir(&root)
        .output()
        .expect("run file-length.sh");
    let code = output.status.code().unwrap();
    assert_ne!(
        code, 1,
        "file-length.sh failed (exit 1): {}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Run function-length check (exit 1 = hard fail)
    let output = Command::new("bash")
        .arg(root.join("scripts/checks/function-length.sh"))
        .current_dir(&root)
        .output()
        .expect("run function-length.sh");
    let code = output.status.code().unwrap();
    assert_ne!(
        code, 1,
        "function-length.sh failed (exit 1): {}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Run module-structure check (must be exit 0)
    let output = Command::new("bash")
        .arg(root.join("scripts/checks/module-structure.sh"))
        .current_dir(&root)
        .output()
        .expect("run module-structure.sh");
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "module-structure.sh failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    // Run single-responsibility check (must be exit 0)
    let output = Command::new("bash")
        .arg(root.join("scripts/checks/single-responsibility.sh"))
        .current_dir(&root)
        .output()
        .expect("run single-responsibility.sh");
    assert_eq!(
        output.status.code().unwrap(),
        0,
        "single-responsibility.sh failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
