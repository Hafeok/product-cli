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
    interpret_runner_output(result, runner, duration)
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
            let script = args.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']');
            let mut cmd = Command::new("bash");
            cmd.arg(script).current_dir(root);
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

fn interpret_runner_output(
    result: std::io::Result<std::process::Output>,
    runner: &str,
    duration: f64,
) -> TcResult {
    match result {
        Ok(output) if output.status.success() => {
            if runner == "cargo-test" {
                if let Some(fail) = detect_zero_tests(&output.stdout, duration) {
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

fn detect_zero_tests(stdout_bytes: &[u8], duration: f64) -> Option<TcResult> {
    let stdout = String::from_utf8_lossy(stdout_bytes);
    if stdout.contains("0 passed") || stdout.contains("running 0 tests") {
        let ran_any = stdout.lines().any(|line| {
            line.contains("test result: ok.") && !line.contains("0 passed")
        });
        if !ran_any {
            return Some(TcResult::Fail(
                duration,
                "No matching test function found (0 tests ran)".to_string(),
            ));
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

struct RwState { last_run: bool, duration: bool, failure: bool }
impl RwState { fn new() -> Self { Self { last_run: false, duration: false, failure: false } } }

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
    if !st.failure { if let Some(msg) = u.failure_msg { lines.push(format!("failure-message: \"{}\"", msg.replace('"', "\\\""))); } st.failure = true; }
}

fn rewrite_line(lines: &mut Vec<String>, line: &str, u: &TcUpdate<'_>, st: &mut RwState) {
    let t = line.trim();
    if t.starts_with("status:") {
        lines.push(format!("status: {}", u.status));
    } else if t.starts_with("last-run-duration:") {
        if let Some(d) = u.duration { lines.push(format!("last-run-duration: {:.1}s", d)); }
        st.duration = true;
    } else if t.starts_with("last-run:") {
        lines.push(format!("last-run: {}", u.timestamp));
        st.last_run = true;
    } else if t.starts_with("failure-message:") {
        if let Some(msg) = u.failure_msg { lines.push(format!("failure-message: \"{}\"", msg.replace('"', "\\\""))); }
        st.failure = true;
    } else {
        lines.push(line.to_string());
    }
}
