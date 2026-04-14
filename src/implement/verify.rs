//! Verification pipeline — product verify FT-XXX (ADR-021)

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::parser;
use crate::types::*;
use crate::fileops;
use std::path::Path;
use std::process::Command;

/// Verify all TCs linked to a feature by running their configured runners
pub fn run_verify(
    feature_id: &str,
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
) -> Result<()> {
    let feature = graph.features.get(feature_id).ok_or_else(|| {
        ProductError::NotFound(format!("feature {}", feature_id))
    })?;

    let now = chrono::Utc::now().to_rfc3339();
    let (all_pass, any_runnable, has_unimplemented, unrunnable_count) =
        run_all_tcs(feature, graph, root, &now)?;

    if unrunnable_count > 0 {
        eprintln!(
            "warning[W016]: {} TC(s) acknowledged as unrunnable for {}",
            unrunnable_count, feature_id
        );
    }

    if any_runnable || has_unimplemented {
        update_feature_and_checklist(feature_id, config, root, all_pass, has_unimplemented)?;
    } else {
        eprintln!("warning[W001]: no runnable TCs found for {}", feature_id);
    }

    Ok(())
}

/// Run all TCs for a feature, returning (all_pass, any_runnable, has_unimplemented, unrunnable_count).
fn run_all_tcs(
    feature: &Feature,
    graph: &KnowledgeGraph,
    root: &Path,
    now: &str,
) -> Result<(bool, bool, bool, usize)> {
    let mut all_pass = true;
    let mut any_runnable = false;
    let mut has_unimplemented = false;
    let mut unrunnable_count: usize = 0;

    for tc_id in &feature.front.tests {
        if let Some(tc) = graph.tests.get(tc_id.as_str()) {
            let content = std::fs::read_to_string(&tc.path).unwrap_or_default();
            let runner = extract_yaml_field(&content, "runner");
            let runner_args = extract_yaml_field(&content, "runner-args");

            if tc.front.status == TestStatus::Unrunnable {
                println!("  {} {:<30} UNRUNNABLE (acknowledged)", tc.front.id, tc.front.title);
                unrunnable_count += 1;
                continue;
            }
            if runner.is_empty() {
                println!("  {} {:<30} UNIMPLEMENTED (no runner configured)", tc.front.id, tc.front.title);
                has_unimplemented = true;
                continue;
            }
            any_runnable = true;
            let status = run_tc(&runner, &runner_args, root);
            match status {
                TcResult::Pass(duration) => {
                    println!("  {} {:<30} PASS ({:.1}s)", tc.front.id, tc.front.title, duration);
                    update_tc_status(&tc.path, "passing", now, None)?;
                }
                TcResult::Fail(duration, message) => {
                    println!("  {} {:<30} FAIL ({:.1}s)", tc.front.id, tc.front.title, duration);
                    update_tc_status(&tc.path, "failing", now, Some(&message))?;
                    all_pass = false;
                }
            }
        }
    }
    Ok((all_pass, any_runnable, has_unimplemented, unrunnable_count))
}

/// Reload the graph, update feature status, and regenerate the checklist.
fn update_feature_and_checklist(
    feature_id: &str,
    config: &ProductConfig,
    root: &Path,
    all_pass: bool,
    has_unimplemented: bool,
) -> Result<()> {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let loaded = parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;
    let new_graph = KnowledgeGraph::build(loaded.features, loaded.adrs, loaded.tests);

    if let Some(f) = new_graph.features.get(feature_id) {
        let new_status = if all_pass && !has_unimplemented {
            FeatureStatus::Complete
        } else {
            FeatureStatus::InProgress
        };
        if f.front.status != new_status {
            let mut front = f.front.clone();
            front.status = new_status;
            let content = parser::render_feature(&front, &f.body);
            fileops::write_file_atomic(&f.path, &content)?;
            println!();
            println!("  Feature {} status -> {}", feature_id, new_status);
        }
    }

    let checklist_content = crate::checklist::generate(&new_graph);
    let checklist_path = config.resolve_path(root, &config.paths.checklist);
    if let Some(parent) = checklist_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    fileops::write_file_atomic(&checklist_path, &checklist_content)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// TC runner
// ---------------------------------------------------------------------------

enum TcResult {
    Pass(f64),
    Fail(f64, String),
}

fn run_tc(runner: &str, args: &str, root: &Path) -> TcResult {
    let start = std::time::Instant::now();
    let result = build_runner_command(runner, args, root).output();
    let duration = start.elapsed().as_secs_f64();
    interpret_runner_output(result, runner, duration)
}

/// Build the appropriate Command for the given runner type.
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
            if !parts.is_empty() {
                cmd.args(&parts);
            }
            cmd.current_dir(root);
            cmd
        }
    }
}

/// Add whitespace-split, quote-cleaned arguments to a command.
fn add_cleaned_args(cmd: &mut Command, args: &str) {
    if !args.is_empty() {
        for arg in args.split_whitespace() {
            cmd.arg(arg.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']' || c == ','));
        }
    }
}

/// Interpret the output of a runner command into a TcResult.
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

/// Detect if cargo-test ran 0 tests (filter matched nothing).
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

fn update_tc_status(path: &Path, status: &str, timestamp: &str, failure_msg: Option<&str>) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let new_lines = rewrite_tc_frontmatter(&content, status, timestamp, failure_msg);
    let new_content = new_lines.join("\n");
    fileops::write_file_atomic(path, &new_content)
}

/// Rewrite front-matter lines to update status, last-run, and failure-message.
fn rewrite_tc_frontmatter(
    content: &str,
    status: &str,
    timestamp: &str,
    failure_msg: Option<&str>,
) -> Vec<String> {
    let mut new_lines = Vec::new();
    let mut in_frontmatter = false;
    let mut last_run_written = false;
    let mut failure_msg_written = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                new_lines.push(line.to_string());
                continue;
            }
            inject_missing_fields(&mut new_lines, timestamp, failure_msg, &mut last_run_written, &mut failure_msg_written);
            in_frontmatter = false;
            new_lines.push(line.to_string());
            continue;
        }
        if in_frontmatter {
            rewrite_frontmatter_line(&mut new_lines, line, status, timestamp, failure_msg, &mut last_run_written, &mut failure_msg_written);
        } else {
            new_lines.push(line.to_string());
        }
    }
    new_lines
}

/// Inject last-run and failure-message if not already written, before closing ---.
fn inject_missing_fields(
    lines: &mut Vec<String>,
    timestamp: &str,
    failure_msg: Option<&str>,
    last_run_written: &mut bool,
    failure_msg_written: &mut bool,
) {
    if !*last_run_written {
        lines.push(format!("last-run: {}", timestamp));
        *last_run_written = true;
    }
    if !*failure_msg_written {
        if let Some(msg) = failure_msg {
            let escaped = msg.replace('"', "\\\"");
            lines.push(format!("failure-message: \"{}\"", escaped));
        }
        *failure_msg_written = true;
    }
}

/// Rewrite a single front-matter line, replacing status/last-run/failure-message as needed.
fn rewrite_frontmatter_line(
    lines: &mut Vec<String>,
    line: &str,
    status: &str,
    timestamp: &str,
    failure_msg: Option<&str>,
    last_run_written: &mut bool,
    failure_msg_written: &mut bool,
) {
    if line.trim().starts_with("status:") {
        lines.push(format!("status: {}", status));
    } else if line.trim().starts_with("last-run:") {
        lines.push(format!("last-run: {}", timestamp));
        *last_run_written = true;
    } else if line.trim().starts_with("failure-message:") {
        if let Some(msg) = failure_msg {
            let escaped = msg.replace('"', "\\\"");
            lines.push(format!("failure-message: \"{}\"", escaped));
        }
        *failure_msg_written = true;
    } else {
        lines.push(line.to_string());
    }
}
