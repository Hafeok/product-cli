//! Agent orchestration — `product implement` and `product verify` (ADR-021)

use crate::config::ProductConfig;
use crate::context;
use crate::error::{ProductError, Result};
use crate::gap;
use crate::graph::KnowledgeGraph;
use crate::parser;
use crate::types::*;
use crate::fileops;
use std::path::Path;
use std::process::Command;

// ---------------------------------------------------------------------------
// product implement FT-XXX
// ---------------------------------------------------------------------------

/// Run the 5-step implementation pipeline
pub fn run_implement(
    feature_id: &str,
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
    dry_run: bool,
    no_verify: bool,
    headless: bool,
) -> Result<()> {
    let feature = graph.features.get(feature_id).ok_or_else(|| {
        ProductError::NotFound(format!("feature {}", feature_id))
    })?;

    println!("product implement {}", feature_id);
    println!();

    // Step 0 — Preflight (domain + cross-cutting coverage)
    print!("  Step 0: Preflight... ");
    let preflight_result = crate::domains::preflight(graph, feature_id, &config.domains)?;
    if !preflight_result.is_clean {
        println!("BLOCKED");
        eprintln!();
        eprintln!("{}", crate::domains::render_preflight(&preflight_result));
        eprintln!("  resolve domain/cross-cutting gaps or acknowledge them before implementing.");
        return Err(ProductError::ConfigError("preflight not clean".to_string()));
    }
    println!("OK (all domains and cross-cutting ADRs covered)");

    // Step 1 — Gap gate
    print!("  Step 1: Gap gate... ");
    let baseline = gap::GapBaseline::load(&root.join("gaps.json"));
    let mut all_findings = Vec::new();
    for adr_id in &feature.front.adrs {
        let findings = gap::check_adr(graph, adr_id, &baseline);
        all_findings.extend(findings);
    }
    let unsuppressed_high: Vec<_> = all_findings
        .iter()
        .filter(|f| f.severity == gap::GapSeverity::High && !f.suppressed)
        .collect();

    if !unsuppressed_high.is_empty() {
        println!("BLOCKED");
        eprintln!();
        eprintln!("error[E009]: implementation blocked by specification gaps");
        eprintln!("  feature: {} — {}", feature.front.id, feature.front.title);
        for g in &unsuppressed_high {
            eprintln!("  gap[{}]: {}", g.code, g.description);
        }
        eprintln!();
        eprintln!("  suppress gaps or add TCs before implementing.");
        return Err(ProductError::ConfigError("gap gate failed".to_string()));
    }
    println!("OK (no high-severity gaps)");

    // Step 2 — Drift check (advisory only)
    println!("  Step 2: Drift check... (advisory, skipped — no drift config)");

    // Step 3 — Context assembly
    print!("  Step 3: Context assembly... ");
    let bundle = context::bundle_feature(graph, feature_id, 2, true)
        .unwrap_or_default();

    // Build TC status table
    let mut tc_table = String::new();
    tc_table.push_str("| TC | Title | Type | Status |\n|---|---|---|---|\n");
    for tc_id in &feature.front.tests {
        if let Some(tc) = graph.tests.get(tc_id.as_str()) {
            tc_table.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                tc.front.id, tc.front.title, tc.front.test_type, tc.front.status
            ));
        }
    }

    let impl_prompt = format!(
        "# Implementation Task: {} — {}\n\n## Your role\nImplement this feature according to the architectural decisions in the context bundle. The test criteria define done — your implementation is complete when all linked TCs pass.\n\n## Current test status\n{}\n\n## Hard constraints\n- Run the test suite before reporting complete\n- When done: `product verify {}`\n\n## Context Bundle\n{}\n",
        feature.front.id, feature.front.title,
        tc_table,
        feature.front.id,
        bundle,
    );

    // Write to temp file
    let tmp_dir = std::env::temp_dir();
    let tmp_name = format!("product-impl-{}-{}.md", feature_id, chrono::Utc::now().timestamp());
    let tmp_path = tmp_dir.join(&tmp_name);
    std::fs::write(&tmp_path, &impl_prompt).map_err(|e| {
        ProductError::WriteError {
            path: tmp_path.clone(),
            message: e.to_string(),
        }
    })?;
    println!("OK");
    println!("  Context file: {}", tmp_path.display());

    if dry_run {
        println!();
        println!("  --dry-run: stopping before agent invocation.");
        println!("  Inspect the context file above, then run without --dry-run.");
        return Ok(());
    }

    // Step 4 — Agent invocation
    if headless {
        println!("  Step 4: Invoking agent (headless)...");
        let agent_result = Command::new("claude")
            .args([
                "-p",
                "--dangerously-skip-permissions",
                "--system-prompt-file",
                &tmp_path.display().to_string(),
                "Implement the feature described in the system prompt. Follow all constraints and run product verify when done.",
            ])
            .current_dir(root)
            .status();

        match agent_result {
            Ok(status) => {
                if status.success() {
                    println!("  Agent completed successfully.");
                } else {
                    println!("  Agent exited with status: {}", status);
                }
            }
            Err(e) => {
                eprintln!("  Warning: could not invoke agent: {}", e);
                eprintln!("  (Is 'claude' in PATH? Or configure a custom agent in product.toml)");
            }
        }
    } else {
        println!("  Step 4: Invoking agent (interactive)...");
        let agent_result = Command::new("claude")
            .args(["--dangerously-skip-permissions", "--system-prompt-file", &tmp_path.display().to_string()])
            .current_dir(root)
            .status();

        match agent_result {
            Ok(status) => {
                if status.success() {
                    println!("  Agent completed successfully.");
                } else {
                    println!("  Agent exited with status: {}", status);
                }
            }
            Err(e) => {
                eprintln!("  Warning: could not invoke agent: {}", e);
                eprintln!("  (Is 'claude' in PATH? Or configure a custom agent in product.toml)");
            }
        }
    }

    // Step 5 — Auto-verify
    if !no_verify {
        println!("  Step 5: Running verify...");
        run_verify(feature_id, config, root, graph)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// product verify FT-XXX
// ---------------------------------------------------------------------------

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

    let mut all_pass = true;
    let mut any_runnable = false;
    let now = chrono::Utc::now().to_rfc3339();

    for tc_id in &feature.front.tests {
        if let Some(tc) = graph.tests.get(tc_id.as_str()) {
            // Check for runner in front-matter
            // Runner info is in extra YAML fields that serde_yaml ignores
            // We re-read the file to check for runner/runner-args
            let content = std::fs::read_to_string(&tc.path).unwrap_or_default();
            let runner = extract_yaml_field(&content, "runner");
            let runner_args = extract_yaml_field(&content, "runner-args");

            if runner.is_empty() {
                println!("  {} {:<30} UNRUNNABLE (no runner configured)", tc.front.id, tc.front.title);
                continue;
            }

            any_runnable = true;
            let status = run_tc(&runner, &runner_args, root);

            match status {
                TcResult::Pass(duration) => {
                    println!("  {} {:<30} PASS ({:.1}s)", tc.front.id, tc.front.title, duration);
                    update_tc_status(&tc.path, "passing", &now, None)?;
                }
                TcResult::Fail(duration, message) => {
                    println!("  {} {:<30} FAIL ({:.1}s)", tc.front.id, tc.front.title, duration);
                    update_tc_status(&tc.path, "failing", &now, Some(&message))?;
                    all_pass = false;
                }
            }
        }
    }

    // Update feature status
    if any_runnable {
        let features_dir = config.resolve_path(root, &config.paths.features);
        let adrs_dir = config.resolve_path(root, &config.paths.adrs);
        let tests_dir = config.resolve_path(root, &config.paths.tests);
        // Reload to get latest statuses
        let (features, adrs, tests) = parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;
        let new_graph = KnowledgeGraph::build(features, adrs, tests);

        if let Some(f) = new_graph.features.get(feature_id) {
            let new_status = if all_pass {
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

        // Regenerate checklist
        let checklist_content = crate::checklist::generate(&new_graph);
        let checklist_path = config.resolve_path(root, &config.paths.checklist);
        if let Some(parent) = checklist_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        fileops::write_file_atomic(&checklist_path, &checklist_content)?;
    } else {
        eprintln!("warning[W001]: no runnable TCs found for {}", feature_id);
    }

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

    let result = match runner {
        "cargo-test" => {
            let mut cmd = Command::new("cargo");
            cmd.arg("test");
            if !args.is_empty() {
                for arg in args.split_whitespace() {
                    cmd.arg(arg.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']' || c == ','));
                }
            }
            cmd.current_dir(root).output()
        }
        "bash" => {
            let script = args.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']');
            Command::new("bash").arg(script).current_dir(root).output()
        }
        "pytest" => {
            let mut cmd = Command::new("pytest");
            if !args.is_empty() {
                for arg in args.split_whitespace() {
                    cmd.arg(arg.trim_matches(|c| c == '"' || c == '\'' || c == '[' || c == ']' || c == ','));
                }
            }
            cmd.current_dir(root).output()
        }
        _ => {
            // Custom runner
            let parts: Vec<&str> = args.split_whitespace().collect();
            if parts.is_empty() {
                Command::new(runner).current_dir(root).output()
            } else {
                Command::new(runner).args(&parts).current_dir(root).output()
            }
        }
    };

    let duration = start.elapsed().as_secs_f64();

    match result {
        Ok(output) if output.status.success() => TcResult::Pass(duration),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let msg = if stderr.len() > 500 { &stderr[..500] } else { &stderr };
            TcResult::Fail(duration, msg.to_string())
        }
        Err(e) => TcResult::Fail(duration, format!("Failed to run {}: {}", runner, e)),
    }
}

fn extract_yaml_field(content: &str, field: &str) -> String {
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
    // Replace/add status, last-run, and failure-message fields in front-matter
    let mut new_lines = Vec::new();
    let mut in_frontmatter = false;
    let mut status_written = false;
    let mut last_run_written = false;
    let mut failure_msg_written = false;
    let mut frontmatter_close_pending = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                new_lines.push(line.to_string());
                continue;
            }
            // Closing --- of front-matter: inject missing fields before closing
            frontmatter_close_pending = true;
            if !last_run_written {
                new_lines.push(format!("last-run: {}", timestamp));
                last_run_written = true;
            }
            if !failure_msg_written {
                if let Some(msg) = failure_msg {
                    let escaped = msg.replace('"', "\\\"");
                    new_lines.push(format!("failure-message: \"{}\"", escaped));
                }
                failure_msg_written = true;
            }
            in_frontmatter = false;
            new_lines.push(line.to_string());
            continue;
        }
        if in_frontmatter && line.trim().starts_with("status:") {
            new_lines.push(format!("status: {}", status));
            status_written = true;
        } else if in_frontmatter && line.trim().starts_with("last-run:") {
            new_lines.push(format!("last-run: {}", timestamp));
            last_run_written = true;
        } else if in_frontmatter && line.trim().starts_with("failure-message:") {
            // Replace or remove failure-message
            if let Some(msg) = failure_msg {
                let escaped = msg.replace('"', "\\\"");
                new_lines.push(format!("failure-message: \"{}\"", escaped));
            }
            // If no failure_msg, omit line (test passed, remove old failure)
            failure_msg_written = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    let _ = (status_written, frontmatter_close_pending);
    let new_content = new_lines.join("\n");
    fileops::write_file_atomic(path, &new_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_runner_field() {
        let content = "---\nid: TC-001\nrunner: cargo-test\nrunner-args: [\"--test\", \"foo\"]\n---\n";
        assert_eq!(extract_yaml_field(content, "runner"), "cargo-test");
    }

    #[test]
    fn extract_missing_field() {
        let content = "---\nid: TC-001\n---\n";
        assert_eq!(extract_yaml_field(content, "runner"), "");
    }
}
