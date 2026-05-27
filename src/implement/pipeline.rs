//! Implementation pipeline — product implement FT-XXX (ADR-021)

use crate::config::ProductConfig;
use crate::context;
use crate::error::{ProductError, Result};
use crate::gap;
use crate::graph::KnowledgeGraph;
use crate::parser;
use std::path::Path;
use std::process::Command;

use super::observes_table::{build_observes_table, inject_observes_inline};
use super::runner_autofill::{
    apply_autofill, plan_autofill, AutofillPlan, AUTOFILL_RUNNER, AUTOFILL_TIMEOUT_SECS,
};
use super::verify::run_verify;

/// ADR-051 hard-constraint line — included verbatim in the implement
/// prompt so the executor agent sees the rule before writing a TC.
/// FT-074 codifies this so the constraint cannot drift between the prompt
/// template and the pipeline.
pub const ADR_051_HARD_CONSTRAINT: &str =
    "Every TC under test must declare `observes:` (ADR-051) and its \
assertions must target the named surface(s). When writing a TC, \
assert against the underlying causation (file on disk, graph node, \
exit code, git tag, stdout/stderr, MCP envelope, etc.) — never on a \
response envelope alone. The structural gate \
(`product graph check`) enforces presence; the body-reference gate \
flags TCs whose body never mentions any declared surface.";

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
    target: Option<&str>,
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
    let raw_bundle = context::bundle_feature(working_graph, feature_id, 2, true)
        .unwrap_or_default();

    // Legacy template (FT-074): caller opted into the pre-FT-074 bundle
    // shape — strip the Patterns section produced by `bundle_feature`,
    // skip observes-inline injection, and omit the ADR-051 reminder.
    let is_legacy = matches!(target, Some(t) if t == "legacy-template");

    // FT-074: surface each linked TC's `observes:` adjacent to its body
    // so the executor agent sees the assertion shape at glance. Skipped
    // in legacy mode for backwards compatibility with templates that
    // never read the new variable.
    let bundle = if is_legacy {
        strip_patterns_section(&raw_bundle)
    } else {
        let observes_rows = build_observes_table(working_graph, feature_id);
        inject_observes_inline(&raw_bundle, &observes_rows)
    };

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

    // FT-074: extend the hard-constraint block with the ADR-051 reminder
    // line. Legacy mode preserves the pre-FT-074 hard-constraint block.
    let adr_051_line = if is_legacy {
        String::new()
    } else {
        format!("- {}\n", ADR_051_HARD_CONSTRAINT)
    };

    let dynamic_suffix = format!(
        "# Implementation Task: {} — {}\n\n## Current test status\n{}\n\n## Hard constraints\n- Run the test suite before reporting complete\n- Test functions must match the configured `runner-args` names, or run `product test runner TC-XXX --args ...` to rename them.\n- When done: `product verify {}`\n{}\n## Context Bundle\n{}\n",
        feature.front.id, feature.front.title,
        tc_table,
        feature.front.id,
        adr_051_line,
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
    //
    // The depth-2 context bundle for non-trivial features can exceed Linux's
    // MAX_ARG_STRLEN (128 KB per argv entry), so we can't pass it via
    // `--system-prompt <content>` or as the positional `[prompt]` argument.
    // `--system-prompt-file` only works under `--bare`, which disables
    // OAuth/keychain auth (requires ANTHROPIC_API_KEY) and is not viable.
    //
    // Solution: the bundle is already on disk at `tmp_path`. Spawn claude
    // interactively with a short positional prompt (~200 chars) that tells
    // the agent to Read the bundle file as its first turn. The bundle then
    // arrives as a Read tool-result in the context window, identical effect
    // to delivering it as system prompt, but with no argv pressure.
    //
    // Interactive TUI is preserved end-to-end — drivers (tmux-orchestrator
    // etc.) can attach, capture-pane, send-keys, etc. The agent exits
    // naturally after `product verify` reports passing TCs.
    let kickoff = format!(
        "Read {} — that file is your complete implementation specification \
         (system framing, hard constraints, depth-2 context bundle of the \
         feature plus its linked ADRs and TCs). Follow it to implement {}. \
         When the test suite passes, run `product verify {}`, then exit.",
        tmp_path.display(),
        feature_id,
        feature_id,
    );

    println!(
        "  Step 4: Invoking agent ({})...",
        if headless { "headless" } else { "interactive" }
    );

    let mut args: Vec<&str> = Vec::new();
    if headless {
        args.push("-p");
    }
    args.push("--dangerously-skip-permissions");
    args.push(&kickoff);

    let agent_result = Command::new("claude")
        .args(&args)
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
    let _ = impl_prompt; // bundle stays on disk at tmp_path for the agent to Read

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

/// Strip the `## Patterns` section from a rendered bundle (FT-074 legacy
/// mode). Returns the bundle unchanged when no such section exists.
fn strip_patterns_section(bundle: &str) -> String {
    let Some(start) = bundle.find("## Patterns") else {
        return bundle.to_string();
    };
    // Find the next top-level heading after the patterns section.
    let after = &bundle[start..];
    let search_offset = 2; // skip past the leading "##"
    let cut = after[search_offset..]
        .find("\n## ")
        .map(|next| search_offset + next + 1);
    match cut {
        Some(c) => {
            let mut out = String::with_capacity(bundle.len());
            out.push_str(&bundle[..start]);
            out.push_str(&after[c..]);
            out
        }
        None => {
            // No following section — drop everything from the heading.
            bundle[..start].to_string()
        }
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
