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
// FT-060 / TC-727: cli_subcommands_are_sorted
//
// For every `Subcommand`-deriving enum declared under `src/commands/`,
// assert that the sequence of variant names (translated to clap's
// kebab-case rendering) is sorted under `str::cmp`.
//
// String-based parsing — consistent with the file-length and SRP checks
// (no `syn` dependency).
// =============================================================================

/// One variant of a parsed `Subcommand` enum.
#[derive(Debug)]
struct ParsedVariant {
    /// The variant identifier as written in source (e.g. `AgentInit`).
    /// Retained for diagnostic detail in panic messages, even when not
    /// directly used.
    #[allow(dead_code)]
    ident: String,
    /// The clap-rendered command name (e.g. `agent-init`, or an explicit
    /// `#[command(name = "X")]` override).
    rendered: String,
}

/// One `Subcommand`-deriving enum extracted from a source file.
#[derive(Debug)]
struct ParsedEnum {
    name: String,
    variants: Vec<ParsedVariant>,
}

/// Convert a PascalCase identifier to clap's default kebab-case rendering.
/// Inserts `-` before every uppercase letter except at position 0, then
/// lowercases. Example: `AgentInit` -> `agent-init`.
fn ident_to_kebab(ident: &str) -> String {
    let mut out = String::with_capacity(ident.len() + 4);
    for (i, c) in ident.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('-');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Find every `pub enum <Name> { ... }` whose preceding lines contain a
/// `#[derive(... Subcommand ...)]` attribute, and parse out the variant
/// identifiers between `{` and `}`. Returns one entry per Subcommand enum.
fn parse_subcommand_enums(source: &str) -> Vec<ParsedEnum> {
    let lines: Vec<&str> = source.lines().collect();
    let mut enums = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        // Look for an enum declaration. A Subcommand-deriving enum has a
        // `#[derive(... Subcommand ...)]` attribute on one of the lines
        // immediately preceding the `pub enum` line.
        if let Some(name) = parse_enum_header(line) {
            // Walk back up to 5 lines looking for the derive attribute.
            let start = i.saturating_sub(5);
            let mut is_subcommand = false;
            for prev in &lines[start..i] {
                if prev.contains("#[derive(") && prev.contains("Subcommand") {
                    is_subcommand = true;
                    break;
                }
            }
            if !is_subcommand {
                i += 1;
                continue;
            }
            // Collect the body until the matching closing brace at column 0.
            let (variants, end) = parse_enum_body(&lines, i + 1);
            enums.push(ParsedEnum { name, variants });
            i = end + 1;
            continue;
        }
        i += 1;
    }
    enums
}

/// If `line` starts with `pub enum <Name> {`, return `<Name>`.
fn parse_enum_header(line: &str) -> Option<String> {
    let s = line.trim_start_matches("pub(crate) ").trim_start_matches("pub ");
    let rest = s.strip_prefix("enum ")?;
    let name = rest.split_whitespace().next()?;
    // Require an opening brace on the same line (the codebase style).
    if !line.contains('{') {
        return None;
    }
    Some(name.trim_end_matches('{').to_string())
}

/// Parse the body of an enum starting at `start` (the line after the
/// opening `{`). Returns the list of variants and the index of the
/// closing `}`.
fn parse_enum_body(lines: &[&str], start: usize) -> (Vec<ParsedVariant>, usize) {
    let mut variants = Vec::new();
    let mut pending_name_override: Option<String> = None;
    let mut i = start;
    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();

        // End of enum body — closing brace at column 0.
        if raw.starts_with('}') {
            return (variants, i);
        }

        // Skip blank lines and full-line comments.
        if trimmed.is_empty() || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        // Capture `#[command(name = "...")]` overrides for the next
        // variant. Aliases are deliberately ignored — the test asserts
        // primary rendered names only.
        if trimmed.starts_with("#[command(") {
            if let Some(name) = parse_command_name_override(trimmed) {
                pending_name_override = Some(name);
            }
            i += 1;
            continue;
        }

        // Skip any other attribute lines.
        if trimmed.starts_with('#') {
            i += 1;
            continue;
        }

        // A variant line starts with a PascalCase identifier. Anything
        // else (field declarations inside a struct-like variant body,
        // continuation lines, `}` markers) is skipped.
        if let Some(ident) = first_pascal_word(trimmed) {
            let rendered = pending_name_override
                .take()
                .unwrap_or_else(|| ident_to_kebab(&ident));
            variants.push(ParsedVariant { ident, rendered });
        }

        i += 1;
    }
    (variants, lines.len() - 1)
}

/// Extract the leading PascalCase identifier from a line, if any. Lines
/// that begin with a non-uppercase character (lower-case field names,
/// `}`, `)`, etc.) return `None`.
fn first_pascal_word(line: &str) -> Option<String> {
    let mut chars = line.chars();
    let first = chars.next()?;
    if !first.is_ascii_uppercase() {
        return None;
    }
    let mut out = String::new();
    out.push(first);
    for c in chars {
        if c.is_ascii_alphanumeric() || c == '_' {
            out.push(c);
        } else {
            break;
        }
    }
    // Variants must be followed by `,`, `{`, `(`, or end of line — that's
    // already true for lines that start with a PascalCase token in the
    // codebase style (no enclosing parens, no leading expressions).
    Some(out)
}

/// Parse `#[command(name = "X")]` and return `X`. Aliases (`alias = ...`)
/// are not honoured — the spec says to use the primary rendered name.
fn parse_command_name_override(attr: &str) -> Option<String> {
    let key = "name = \"";
    let start = attr.find(key)? + key.len();
    let rest = &attr[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

#[test]
fn cli_subcommands_are_sorted() {
    let cmd_dir = project_root().join("src/commands");
    let entries = std::fs::read_dir(&cmd_dir).expect("read src/commands");

    let mut files: Vec<std::path::PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("rs"))
        .collect();
    files.sort();

    let mut total_enums = 0;
    for file in &files {
        let source = std::fs::read_to_string(file).expect("read source");
        for parsed in parse_subcommand_enums(&source) {
            total_enums += 1;
            assert_sorted_variants(file, &parsed);
        }
    }

    // Sanity check: we found a non-trivial number of Subcommand enums.
    // If parsing breaks, this guards against silently asserting nothing.
    assert!(
        total_enums >= 15,
        "expected at least 15 Subcommand enums under src/commands/, found {}",
        total_enums,
    );
}

fn assert_sorted_variants(file: &std::path::Path, parsed: &ParsedEnum) {
    let names: Vec<&str> = parsed.variants.iter().map(|v| v.rendered.as_str()).collect();
    for window in names.windows(2) {
        if window[0] > window[1] {
            panic!(
                "{}: enum {}: variants out of order — expected `{}` before `{}` but got `{}` before `{}`",
                file.display(),
                parsed.name,
                window[1],
                window[0],
                window[0],
                window[1],
            );
        }
    }
}

// =============================================================================
// FT-061: MCP and CLI handlers must use ProductConfig::load_from_root,
// never `repo_root.join("product.toml")`. The hardcoded path defeated
// `.product/config.toml` discovery against canonical-layout repos.
// =============================================================================
#[test]
fn ft_061_mcp_handlers_do_not_hardcode_product_toml_path() {
    let root = project_root();
    let mcp_dir = root.join("src/mcp");
    let mut offenders: Vec<String> = Vec::new();

    fn walk(dir: &std::path::Path, offenders: &mut Vec<String>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, offenders);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            for (lineno, line) in source.lines().enumerate() {
                if line.contains("repo_root.join(\"product.toml\")") {
                    offenders.push(format!("{}:{}", path.display(), lineno + 1));
                }
            }
        }
    }

    walk(&mcp_dir, &mut offenders);

    // Also gate src/commands/mcp_cmd.rs, where the same regression bit us.
    let mcp_cmd = root.join("src/commands/mcp_cmd.rs");
    if let Ok(source) = std::fs::read_to_string(&mcp_cmd) {
        for (lineno, line) in source.lines().enumerate() {
            if line.contains("repo_root.join(\"product.toml\")") {
                offenders.push(format!("{}:{}", mcp_cmd.display(), lineno + 1));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "FT-061: MCP handlers must use ProductConfig::load_from_root; \
         offending sites:\n  {}",
        offenders.join("\n  "),
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
