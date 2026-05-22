//! TC runner — command execution, result interpretation, front-matter updates (ADR-021)

use crate::error::Result;
use crate::fileops;
use std::path::Path;
use std::process::Command;

pub(crate) enum TcResult {
    Pass(f64),
    Fail(f64, String),
}

pub(crate) fn run_tc(runner: &str, args: &str, root: &Path) -> TcResult {
    let start = std::time::Instant::now();
    let result = build_runner_command(runner, args, root).output();
    let duration = start.elapsed().as_secs_f64();
    interpret_runner_output(result, runner, args, duration)
}

fn build_runner_command(runner: &str, args: &str, root: &Path) -> Command {
    match runner {
        "cargo-test" => {
            let mut cmd = Command::new("cargo");
            cmd.arg("test");
            add_cleaned_args(&mut cmd, args);
            cmd.current_dir(root);
            cmd
        }
        "bash" => {
            let script = strip_balanced_quotes(args);
            let mut cmd = Command::new("bash");
            cmd.arg("-c").arg(script).current_dir(root);
            cmd
        }
        "pytest" => {
            let mut cmd = Command::new("pytest");
            add_cleaned_args(&mut cmd, args);
            cmd.current_dir(root);
            cmd
        }
        _ => {
            let mut cmd = Command::new(runner);
            let parts: Vec<&str> = args.split_whitespace().collect();
            if !parts.is_empty() { cmd.args(&parts); }
            cmd.current_dir(root);
            cmd
        }
    }
}

fn add_cleaned_args(cmd: &mut Command, args: &str) {
    if !args.is_empty() {
        for arg in args.split_whitespace() {
            cmd.arg(arg.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']' || c == ','));
        }
    }
}

fn strip_balanced_quotes(s: &str) -> &str {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.len() < 2 { return s; }
    let first = bytes[0];
    let last = bytes[bytes.len() - 1];
    if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Escape a string for use as a YAML double-quoted scalar value.
/// Order matters: escape backslashes first, then quotes, then control chars.
fn escape_yaml_double_quoted(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

fn interpret_runner_output(
    result: std::io::Result<std::process::Output>,
    runner: &str,
    args: &str,
    duration: f64,
) -> TcResult {
    match result {
        Ok(output) if output.status.success() => {
            if runner == "cargo-test" {
                if let Some(fail) = detect_zero_tests(&output.stdout, args, duration) {
                    return fail;
                }
            }
            TcResult::Pass(duration)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = if stderr.len() > 500 { &stderr[..500] } else { &stderr };
            TcResult::Fail(duration, msg.to_string())
        }
        Err(e) => TcResult::Fail(duration, format!("Failed to run {}: {}", runner, e)),
    }
}

/// FT-058: when cargo reports "0 tests ran", the runner-args almost
/// certainly point at a function that does not exist. Name the missing
/// function so the developer can find or write it without re-reading
/// cargo's output.
fn detect_zero_tests(stdout_bytes: &[u8], args: &str, duration: f64) -> Option<TcResult> {
    let stdout = String::from_utf8_lossy(stdout_bytes);
    if stdout.contains("0 passed") || stdout.contains("running 0 tests") {
        let ran_any = stdout.lines().any(|line| {
            line.contains("test result: ok.") && !line.contains("0 passed")
        });
        if !ran_any {
            let cleaned = args
                .trim()
                .trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']');
            let msg = if cleaned.is_empty() {
                "No #[test] fn matching '' found in tests/*.rs — did you forget to add the integration test?".to_string()
            } else {
                format!(
                    "No #[test] fn matching '{}' found in tests/*.rs — did you forget to add the integration test?",
                    cleaned
                )
            };
            return Some(TcResult::Fail(duration, msg));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// YAML field extraction
// ---------------------------------------------------------------------------

pub(crate) fn extract_yaml_field(content: &str, field: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&format!("{}:", field)) {
            return rest.trim().to_string();
        }
        if let Some(rest) = trimmed.strip_prefix(field).and_then(|s| s.strip_prefix(':')) {
            return rest.trim().to_string();
        }
    }
    String::new()
}

pub(crate) fn extract_yaml_list(content: &str, field: &str) -> Vec<String> {
    let raw = extract_yaml_field(content, field);
    if raw.is_empty() { return Vec::new(); }
    let trimmed = raw.trim_matches(|c| c == '[' || c == ']');
    if trimmed.is_empty() { return Vec::new(); }
    trimmed.split(',')
        .map(|s| s.trim().trim_matches(|c| c == '"' || c == '\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// TC front-matter rewriter
// ---------------------------------------------------------------------------

pub(crate) fn update_tc_status(
    path: &Path, status: &str, timestamp: &str,
    failure_msg: Option<&str>, duration: Option<f64>,
) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let new_lines = rewrite_tc_frontmatter(&content, status, timestamp, failure_msg, duration);
    fileops::write_file_atomic(path, &new_lines.join("\n"))
}

struct TcUpdate<'a> {
    status: &'a str,
    timestamp: &'a str,
    failure_msg: Option<&'a str>,
    duration: Option<f64>,
}

struct RwState {
    last_run: bool,
    duration: bool,
    failure: bool,
    /// When `Some(base)`, the previous mutable key declared a YAML block
    /// scalar (`|`, `>`, etc.). Skip subsequent blank lines and lines
    /// indented strictly more than `base` — they were continuation lines
    /// of that block scalar's value and would otherwise be orphaned after
    /// the key is dropped or replaced with a single-line scalar.
    skip_indent: Option<usize>,
}
impl RwState {
    fn new() -> Self {
        Self { last_run: false, duration: false, failure: false, skip_indent: None }
    }
}

fn rewrite_tc_frontmatter(
    content: &str, status: &str, timestamp: &str,
    failure_msg: Option<&str>, duration: Option<f64>,
) -> Vec<String> {
    let u = TcUpdate { status, timestamp, failure_msg, duration };
    let mut st = RwState::new();
    let mut out = Vec::new();
    let mut in_fm = false;
    for line in content.lines() {
        if line.trim() == "---" {
            if !in_fm { in_fm = true; out.push(line.to_string()); continue; }
            inject_missing(&mut out, &u, &mut st);
            in_fm = false;
            out.push(line.to_string());
            continue;
        }
        if in_fm { rewrite_line(&mut out, line, &u, &mut st); }
        else { out.push(line.to_string()); }
    }
    out
}

fn inject_missing(lines: &mut Vec<String>, u: &TcUpdate<'_>, st: &mut RwState) {
    if !st.last_run { lines.push(format!("last-run: {}", u.timestamp)); st.last_run = true; }
    if !st.duration { if let Some(d) = u.duration { lines.push(format!("last-run-duration: {:.1}s", d)); } st.duration = true; }
    if !st.failure { if let Some(msg) = u.failure_msg { lines.push(format!("failure-message: \"{}\"", escape_yaml_double_quoted(msg))); } st.failure = true; }
}

fn rewrite_line(lines: &mut Vec<String>, line: &str, u: &TcUpdate<'_>, st: &mut RwState) {
    if let Some(base) = st.skip_indent {
        if line.trim().is_empty() || indent_of(line) > base {
            return;
        }
        st.skip_indent = None;
    }

    let t = line.trim();
    if let Some(after) = t.strip_prefix("status:") {
        lines.push(format!("status: {}", u.status));
        enter_skip_if_block_scalar(line, after, st);
    } else if let Some(after) = t.strip_prefix("last-run-duration:") {
        if let Some(d) = u.duration { lines.push(format!("last-run-duration: {:.1}s", d)); }
        st.duration = true;
        enter_skip_if_block_scalar(line, after, st);
    } else if let Some(after) = t.strip_prefix("last-run:") {
        lines.push(format!("last-run: {}", u.timestamp));
        st.last_run = true;
        enter_skip_if_block_scalar(line, after, st);
    } else if let Some(after) = t.strip_prefix("failure-message:") {
        if let Some(msg) = u.failure_msg { lines.push(format!("failure-message: \"{}\"", escape_yaml_double_quoted(msg))); }
        st.failure = true;
        enter_skip_if_block_scalar(line, after, st);
    } else {
        lines.push(line.to_string());
    }
}

fn indent_of(line: &str) -> usize {
    line.bytes().take_while(|b| *b == b' ' || *b == b'\t').count()
}

/// If the value after a known key declares a YAML block scalar (`|`, `>`
/// with optional chomping/keep indicators, and an optional trailing
/// comment), enter skip mode so the next call drops its continuation lines.
fn enter_skip_if_block_scalar(line: &str, value: &str, st: &mut RwState) {
    let bare = value.split('#').next().unwrap_or("").trim();
    if bare.starts_with('|') || bare.starts_with('>') {
        st.skip_indent = Some(indent_of(line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rewrite(content: &str, status: &str, msg: Option<&str>, dur: Option<f64>) -> String {
        rewrite_tc_frontmatter(content, status, "2026-05-22T12:00:00Z", msg, dur).join("\n")
    }

    /// Regression: when a previously failing TC becomes passing,
    /// the `failure-message: |` block scalar must be removed in
    /// full — header line *and* every indented continuation line.
    /// Otherwise the orphaned continuation lines parse as a
    /// malformed mapping value attached to `last-run:` and the
    /// front-matter fails to load (E001).
    #[test]
    fn passing_run_strips_block_scalar_continuation_lines() {
        let before = "---\nid: TC-001\nstatus: failing\nlast-run: 2026-05-21T00:00:00Z\nlast-run-duration: 0.5s\nfailure-message: |\n  ERROR: first line\n  ERROR: second line\n---\nbody\n";
        let after = rewrite(before, "passing", None, Some(0.7));
        assert!(after.contains("status: passing"), "after: {after}");
        assert!(!after.contains("failure-message"), "should drop failure-message entirely; after: {after}");
        assert!(!after.contains("ERROR: first line"), "orphan continuation line leaked; after: {after}");
        assert!(!after.contains("ERROR: second line"), "orphan continuation line leaked; after: {after}");
        // The result must round-trip through a real YAML parser.
        let fm = extract_frontmatter(&after);
        serde_yaml::from_str::<serde_yaml::Value>(&fm)
            .unwrap_or_else(|e| panic!("rewritten front-matter is invalid YAML: {e}\n---\n{fm}"));
    }

    /// Replacing an existing `failure-message: |` block scalar with a
    /// new single-line value must also drop the old continuation lines.
    #[test]
    fn failing_run_replaces_block_scalar_with_single_line() {
        let before = "---\nid: TC-001\nstatus: failing\nlast-run: 2026-05-21T00:00:00Z\nlast-run-duration: 0.5s\nfailure-message: |\n  old error line 1\n  old error line 2\n---\nbody\n";
        let after = rewrite(before, "failing", Some("new error"), Some(0.7));
        assert!(after.contains("failure-message: \"new error\""), "after: {after}");
        assert!(!after.contains("old error line 1"), "old continuation leaked; after: {after}");
        assert!(!after.contains("old error line 2"), "old continuation leaked; after: {after}");
        let fm = extract_frontmatter(&after);
        serde_yaml::from_str::<serde_yaml::Value>(&fm)
            .unwrap_or_else(|e| panic!("rewritten front-matter is invalid YAML: {e}\n---\n{fm}"));
    }

    /// Block scalar variants with chomping (`|-`) and folded (`>`)
    /// indicators must also have their continuation lines stripped.
    #[test]
    fn passing_run_handles_block_scalar_variants() {
        for header in &["|-", "|+", ">", ">-", ">+", "|  # leftover comment"] {
            let before = format!(
                "---\nid: TC-001\nstatus: failing\nlast-run: 2026-05-21T00:00:00Z\nfailure-message: {header}\n  continuation a\n  continuation b\n---\n"
            );
            let after = rewrite(&before, "passing", None, Some(0.1));
            assert!(
                !after.contains("continuation a") && !after.contains("continuation b"),
                "variant `{header}` failed to strip continuations; after: {after}"
            );
        }
    }

    /// A blank line between top-level keys must be preserved when the
    /// preceding key was a single-line scalar (not a block scalar).
    /// This guards against an over-eager skip mode that would eat
    /// blank lines after every mutable key replacement.
    #[test]
    fn single_line_scalars_preserve_following_blank_lines() {
        let before = "---\nid: TC-001\nstatus: failing\nlast-run: 2026-05-21T00:00:00Z\n\nfailure-message: \"old\"\n---\n";
        let after = rewrite(before, "passing", None, None);
        // The blank line between `last-run:` and what follows survives.
        assert!(after.contains("\n\n"), "blank line was eaten; after: {after:?}");
    }

    fn extract_frontmatter(s: &str) -> String {
        let mut lines = s.lines();
        assert_eq!(lines.next(), Some("---"));
        let mut out = String::new();
        for l in lines {
            if l == "---" { break; }
            out.push_str(l);
            out.push('\n');
        }
        out
    }
}
