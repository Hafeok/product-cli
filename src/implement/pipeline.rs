//! Implementation pipeline — product implement FT-XXX (ADR-021)

use crate::config::ProductConfig;
use crate::context;
use crate::error::{ProductError, Result};
use crate::gap;
use crate::graph::KnowledgeGraph;
use std::path::Path;
use std::process::Command;

use super::verify::run_verify;

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
        run_verify(feature_id, config, root, graph, false)?;
    }

    Ok(())
}
