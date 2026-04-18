//! Gap analysis between ADRs, features, test criteria.

use clap::Subcommand;
use product_lib::gap;
use std::process;

use super::{load_graph, BoxResult};

#[derive(Subcommand)]
pub enum GapCommands {
    /// Check for gaps (optionally for a single ADR or feature, or only changed ADRs)
    Check {
        /// ADR or Feature ID to check (omit for all)
        adr_id: Option<String>,
        /// Only check ADRs changed in the last commit
        #[arg(long)]
        changed: bool,
        /// Output format: text or json
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Produce an LLM-ready gap-analysis bundle on stdout (ADR-040)
    Bundle {
        /// ADR ID (mutually exclusive with --all / --changed)
        adr_id: Option<String>,
        /// Emit a bundle for every ADR in the graph
        #[arg(long)]
        all: bool,
        /// Emit bundles only for ADRs changed since the last commit
        #[arg(long)]
        changed: bool,
        /// Output format: markdown (default) or json
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    /// Print a human-readable gap report for all ADRs
    Report,
    /// Suppress a gap finding
    Suppress {
        /// Gap finding ID to suppress
        gap_id: String,
        /// Reason for suppression
        #[arg(long)]
        reason: String,
    },
    /// Remove suppression for a gap finding
    Unsuppress {
        /// Gap finding ID to unsuppress
        gap_id: String,
    },
    /// Print gap analysis statistics
    Stats,
}

pub(crate) fn handle_gap(cmd: GapCommands, _global_fmt: &str) -> BoxResult {
    let (_, root, graph) = load_graph()?;
    let baseline_path = root.join("gaps.json");
    let mut baseline = gap::GapBaseline::load(&baseline_path);

    match cmd {
        GapCommands::Check { adr_id, changed, format } => {
            gap_check(adr_id, changed, &format, &graph, &mut baseline, &baseline_path, &root)
        }
        GapCommands::Bundle { adr_id, all, changed, format } => {
            gap_bundle(adr_id, all, changed, &format, &graph, &root)
        }
        GapCommands::Report => gap_report(&graph, &baseline),
        GapCommands::Suppress { gap_id, reason } => {
            gap_suppress(&mut baseline, &gap_id, &reason, &baseline_path)
        }
        GapCommands::Unsuppress { gap_id } => {
            gap_unsuppress(&mut baseline, &gap_id, &baseline_path)
        }
        GapCommands::Stats => gap_stats(&graph, &baseline),
    }
}

fn gap_bundle(
    adr_id: Option<String>,
    all: bool,
    changed: bool,
    format: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    root: &std::path::Path,
) -> BoxResult {
    let markdown = if all {
        gap::bundle_all(graph, root)
    } else if changed {
        gap::bundle_changed(graph, root)
    } else if let Some(ref id) = adr_id {
        match gap::bundle_for_adr(id, graph, root) {
            Some(b) => b,
            None => {
                eprintln!("error: ADR {} not found", id);
                process::exit(1);
            }
        }
    } else {
        eprintln!("error: specify an ADR ID, or use --all or --changed");
        process::exit(1);
    };

    if format == "json" {
        let value = serde_json::json!({ "bundle": markdown });
        println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default());
    } else {
        print!("{}", markdown);
    }
    Ok(())
}

fn gap_check(
    adr_id: Option<String>,
    changed: bool,
    format: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    baseline: &mut gap::GapBaseline,
    baseline_path: &std::path::Path,
    root: &std::path::Path,
) -> BoxResult {
    // If the ID is a feature, run feature-specific dep gap check (G008)
    if let Some(ref id) = adr_id {
        if id.starts_with("FT-") {
            let findings = gap::check_feature_dep_gaps(graph, id, baseline);
            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&findings).unwrap_or_default());
            } else {
                for finding in &findings {
                    let suppressed_tag = if finding.suppressed { " [suppressed]" } else { "" };
                    println!(
                        "  [{:>6}] {} \u{2014} {}{}",
                        finding.severity, finding.code, finding.description, suppressed_tag
                    );
                }
            }
            let has_findings = !findings.is_empty();
            if has_findings {
                process::exit(1);
            }
            return Ok(());
        }
    }

    let adr_ids_to_check: Vec<String> = if let Some(ref id) = adr_id {
        vec![id.clone()]
    } else if changed {
        return gap_check_changed(format, graph, baseline, baseline_path, root);
    } else {
        graph.adrs.keys().cloned().collect()
    };

    let reports = build_gap_reports(&adr_ids_to_check, graph, baseline);
    save_and_print_reports(&reports, format, baseline, baseline_path)
}

fn gap_check_changed(
    format: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    baseline: &mut gap::GapBaseline,
    baseline_path: &std::path::Path,
    root: &std::path::Path,
) -> BoxResult {
    let reports = gap::check_changed(graph, baseline, root);
    let all_finding_ids: Vec<String> = reports
        .iter()
        .flat_map(|r| r.findings.iter().map(|f| f.id.clone()))
        .collect();
    baseline.update_resolved(&all_finding_ids);
    baseline.save(baseline_path)?;

    print_gap_reports(&reports, format);

    let has_new_high = reports.iter().any(|r| {
        r.findings.iter().any(|f| f.severity == gap::GapSeverity::High && !f.suppressed)
    });
    if has_new_high {
        process::exit(1);
    }
    Ok(())
}

fn build_gap_reports(
    adr_ids: &[String],
    graph: &product_lib::graph::KnowledgeGraph,
    baseline: &gap::GapBaseline,
) -> Vec<gap::GapReport> {
    let mut reports = Vec::new();
    for id in adr_ids {
        let findings = gap::check_adr(graph, id, baseline);

        let summary = gap::GapSummary {
            high: findings.iter().filter(|f| f.severity == gap::GapSeverity::High && !f.suppressed).count(),
            medium: findings.iter().filter(|f| f.severity == gap::GapSeverity::Medium && !f.suppressed).count(),
            low: findings.iter().filter(|f| f.severity == gap::GapSeverity::Low && !f.suppressed).count(),
            suppressed: findings.iter().filter(|f| f.suppressed).count(),
        };
        reports.push(gap::GapReport {
            adr: id.clone(),
            run_date: chrono::Utc::now().to_rfc3339(),
            product_version: env!("CARGO_PKG_VERSION").to_string(),
            findings,
            summary,
        });
    }
    reports
}

fn save_and_print_reports(
    reports: &[gap::GapReport],
    format: &str,
    baseline: &mut gap::GapBaseline,
    baseline_path: &std::path::Path,
) -> BoxResult {
    let all_finding_ids: Vec<String> = reports
        .iter()
        .flat_map(|r| r.findings.iter().map(|f| f.id.clone()))
        .collect();
    baseline.update_resolved(&all_finding_ids);
    baseline.save(baseline_path)?;

    print_gap_reports(reports, format);

    let has_new_high = reports.iter().any(|r| {
        r.findings.iter().any(|f| f.severity == gap::GapSeverity::High && !f.suppressed)
    });
    if has_new_high {
        process::exit(1);
    }
    Ok(())
}

fn print_gap_reports(reports: &[gap::GapReport], format: &str) {
    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&reports).unwrap_or_default());
    } else {
        for report in reports {
            if report.findings.is_empty() {
                continue;
            }
            println!("--- {} ---", report.adr);
            for finding in &report.findings {
                let suppressed_tag = if finding.suppressed { " [suppressed]" } else { "" };
                println!(
                    "  [{:>6}] {} — {}{}",
                    finding.severity, finding.code, finding.description, suppressed_tag
                );
            }
        }
    }
}

fn gap_report(
    graph: &product_lib::graph::KnowledgeGraph,
    baseline: &gap::GapBaseline,
) -> BoxResult {
    let reports = gap::check_all(graph, baseline);
    let total_findings: usize = reports.iter().map(|r| r.findings.len()).sum();
    let total_high: usize = reports.iter().flat_map(|r| &r.findings)
        .filter(|f| f.severity == gap::GapSeverity::High && !f.suppressed).count();
    let total_medium: usize = reports.iter().flat_map(|r| &r.findings)
        .filter(|f| f.severity == gap::GapSeverity::Medium && !f.suppressed).count();
    let total_low: usize = reports.iter().flat_map(|r| &r.findings)
        .filter(|f| f.severity == gap::GapSeverity::Low && !f.suppressed).count();
    let total_suppressed: usize = reports.iter().flat_map(|r| &r.findings)
        .filter(|f| f.suppressed).count();

    println!("Gap Analysis Report");
    println!("====================");
    println!("ADRs analysed: {}", reports.len());
    println!("Total findings: {} (high: {}, medium: {}, low: {}, suppressed: {})",
        total_findings, total_high, total_medium, total_low, total_suppressed);
    println!();

    for report in &reports {
        if report.findings.is_empty() {
            continue;
        }
        println!("--- {} ({} findings) ---", report.adr, report.findings.len());
        for finding in &report.findings {
            let suppressed_tag = if finding.suppressed { " [suppressed]" } else { "" };
            println!(
                "  [{:>6}] {} — {}{}",
                finding.severity, finding.code, finding.description, suppressed_tag
            );
            println!("           Action: {}", finding.suggested_action);
            if !finding.affected_artifacts.is_empty() {
                println!("           Affects: {}", finding.affected_artifacts.join(", "));
            }
        }
        println!();
    }
    Ok(())
}

fn gap_suppress(
    baseline: &mut gap::GapBaseline,
    gap_id: &str,
    reason: &str,
    baseline_path: &std::path::Path,
) -> BoxResult {
    baseline.suppress(gap_id, reason);
    baseline.save(baseline_path)?;
    println!("Suppressed: {}", gap_id);
    Ok(())
}

fn gap_unsuppress(
    baseline: &mut gap::GapBaseline,
    gap_id: &str,
    baseline_path: &std::path::Path,
) -> BoxResult {
    baseline.unsuppress(gap_id);
    baseline.save(baseline_path)?;
    println!("Unsuppressed: {}", gap_id);
    Ok(())
}

fn gap_stats(
    graph: &product_lib::graph::KnowledgeGraph,
    baseline: &gap::GapBaseline,
) -> BoxResult {
    let reports = gap::check_all(graph, baseline);
    let stats = gap::gap_stats(&reports, baseline);
    println!("{}", serde_json::to_string_pretty(&stats).unwrap_or_default());
    Ok(())
}
