//! Implementation pipeline — product implement FT-XXX (ADR-021)

use crate::config::ProductConfig;
use crate::context;
use crate::error::{ProductError, Result};
use crate::gap;
use crate::graph::KnowledgeGraph;
use crate::parser;
use std::path::Path;
use std::process::Command;

use super::runner_autofill::{
    apply_autofill, plan_autofill, AutofillPlan, AUTOFILL_RUNNER, AUTOFILL_TIMEOUT_SECS,
};
use super::verify::run_verify;

/// Run the 5-step implementation pipeline
#[allow(clippy::too_many_arguments)]
pub fn run_implement(
    feature_id: &str,
    config: &ProductConfig,
    root: &Path,
    graph: &KnowledgeGraph,
    dry_run: bool,
    no_verify: bool,
    headless: bool,
    no_auto_runners: bool,
) -> Result<()> {
    // Validate the feature exists before any output. We re-fetch it from
    // the working graph after Step 0a so the rest of the pipeline sees
    // any updated state.
    let _ = graph.features.get(feature_id).ok_or_else(|| {
        ProductError::NotFound(format!("feature {}", feature_id))
    })?;

    println!("product implement {}", feature_id);
    println!();

    // Step 0a — Auto-fill TC runner config (FT-068, opt-out via
    // --no-auto-runners). Runs immediately before Step 0 so the
    // preflight gate sees the updated state.
    let reloaded_graph: Option<KnowledgeGraph> =
        run_step_0a(graph, config, root, feature_id, dry_run, no_auto_runners)?;
    let working_graph: &KnowledgeGraph = reloaded_graph.as_ref().unwrap_or(graph);

    let feature = working_graph
        .features
        .get(feature_id)
        .ok_or_else(|| ProductError::NotFound(format!("feature {}", feature_id)))?;

    // Step 0 — Preflight (domain + cross-cutting coverage)
    print!("  Step 0: Preflight... ");
    let preflight_result =
        crate::domains::preflight(working_graph, feature_id, &config.domains)?;
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
    let baseline = gap::GapBaseline::load(&root.join(config.paths.gaps_resolved()));
    let mut all_findings = Vec::new();
    for adr_id in &feature.front.adrs {
        let findings = gap::check_adr(working_graph, adr_id, &baseline);
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
    let bundle = context::bundle_feature(working_graph, feature_id, 2, true)
        .unwrap_or_default();

    // Build TC status table
    let mut tc_table = String::new();
    tc_table.push_str("| TC | Title | Type | Status |\n|---|---|---|---|\n");
    for tc_id in &feature.front.tests {
        if let Some(tc) = working_graph.tests.get(tc_id.as_str()) {
            tc_table.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                tc.front.id, tc.front.title, tc.front.test_type, tc.front.status
            ));
        }
    }

    // Per ADR-022, load the base prompt from the per-repo override file
    // when present; fall back to the embedded default otherwise.
    // `prompts::get` honours `[paths].prompts` (FT-057, ADR-048) with a
    // legacy `benchmarks/prompts/` fallback.
    let base_prompt = crate::author::prompts::get(root, config.paths.prompts_resolved(), "implement").unwrap_or_default();

    let dynamic_suffix = format!(
        "# Implementation Task: {} — {}\n\n## Current test status\n{}\n\n## Hard constraints\n- Run the test suite before reporting complete\n- Test functions must match the configured `runner-args` names, or run `product test runner TC-XXX --args ...` to rename them.\n- When done: `product verify {}`\n\n## Context Bundle\n{}\n",
        feature.front.id, feature.front.title,
        tc_table,
        feature.front.id,
        bundle,
    );

    let impl_prompt = format!("{}\n\n{}", base_prompt, dynamic_suffix);

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
        run_verify(feature_id, config, root, working_graph, false)?;
    }

    Ok(())
}

/// Step 0a — auto-fill missing TC runner config (FT-068).
///
/// Returns `Some(reloaded_graph)` when the auto-fill performed writes;
/// the caller should use the reloaded graph for the remainder of the
/// pipeline so Step 0's preflight evaluates the updated TC state.
/// Returns `None` when no writes occurred (opt-out, dry-run, or no
/// offenders) — the caller continues with the original graph.
fn run_step_0a(
    graph: &KnowledgeGraph,
    config: &ProductConfig,
    root: &Path,
    feature_id: &str,
    dry_run: bool,
    no_auto_runners: bool,
) -> Result<Option<KnowledgeGraph>> {
    if no_auto_runners {
        println!("  Step 0a: Auto-fill runner config... SKIPPED (--no-auto-runners)");
        // Fire E022 explicitly so the implement pipeline refuses to invoke
        // the agent against an unverifiable spec. This mirrors the original
        // pre-FT-068 strict behaviour: with --no-auto-runners, the user has
        // opted in to the strict gate.
        let plans = plan_autofill(graph, feature_id);
        if !plans.is_empty() {
            let tc_ids: Vec<String> = plans.iter().map(|p| p.tc_id.clone()).collect();
            let tc_paths: Vec<std::path::PathBuf> =
                plans.iter().map(|p| p.tc_path.clone()).collect();
            let err = ProductError::TcRunnerMissing {
                feature_id: feature_id.to_string(),
                tc_ids,
                tc_paths,
            };
            eprintln!("{}", err);
            return Err(err);
        }
        return Ok(None);
    }
    let plans = plan_autofill(graph, feature_id);
    if plans.is_empty() {
        println!("  Step 0a: Auto-fill runner config... OK (all TCs already configured)");
        return Ok(None);
    }
    if dry_run {
        println!("  Step 0a: Auto-fill runner config... DRY-RUN");
        print_autofill_plan(&plans);
        println!(
            "  --dry-run: auto-fill plan shown above; no writes performed.\n  auto-fill plan: {} TC(s).",
            plans.len()
        );
        return Ok(None);
    }
    println!("  Step 0a: Auto-fill runner config...");
    print_autofill_plan(&plans);
    apply_autofill(&plans, config, graph)?;
    println!("  auto-filled runner config on {} TC(s).", plans.len());
    Ok(Some(reload_graph(config, root)?))
}

/// Print one line per Step 0a auto-fill plan in the canonical harness
/// format. The summary line is emitted by the caller.
fn print_autofill_plan(plans: &[AutofillPlan]) {
    for plan in plans {
        println!(
            "  pre-flight: {} missing runner config — auto-setting runner={} args={} timeout={}s",
            plan.tc_id, AUTOFILL_RUNNER, plan.derived_args, AUTOFILL_TIMEOUT_SECS,
        );
    }
}

/// Reload the knowledge graph from disk so downstream steps see the
/// updated TC front-matter Step 0a just wrote.
fn reload_graph(config: &ProductConfig, root: &Path) -> Result<KnowledgeGraph> {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let deps_dir = config.resolve_path(root, &config.paths.dependencies);
    let loaded =
        parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;
    let graph = KnowledgeGraph::build_with_deps(
        loaded.features,
        loaded.adrs,
        loaded.tests,
        loaded.dependencies,
    )
    .with_parse_errors(loaded.parse_errors);
    Ok(graph)
}
