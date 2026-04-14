//! Transitive TC→Feature link inference (ADR-027)
//!
//! Algorithm: for each non-cross-cutting ADR A, find features F_set that link to A
//! and TCs T_set that validate A. For each (T, F) pair where F ∉ T.validates.features,
//! infer a new TC→Feature link. Also maintains bidirectional consistency: when a TC
//! gains a feature, the feature gains the TC in its tests list.

use crate::error::ProductError;
use crate::parser;
use crate::types::AdrScope;
use std::collections::HashMap;
use std::path::PathBuf;

use super::KnowledgeGraph;

/// Options controlling inference scope
pub struct InferenceOptions {
    /// Skip cross-cutting ADRs (required by ADR-027 for link-tests/infer)
    pub skip_cross_cutting: bool,
    /// Restrict inference to a single ADR
    pub adr_filter: Option<String>,
    /// Restrict inference to a single feature
    pub feature_filter: Option<String>,
}

/// A single inferred link: TC → Feature, via a specific ADR
#[derive(Debug, Clone)]
pub struct InferredLink {
    pub tc_id: String,
    pub feature_id: String,
    pub via_adr: String,
}

/// Result of inference computation
pub struct InferenceResult {
    pub links: Vec<InferredLink>,
    pub skipped_cross_cutting: Vec<String>,
    pub already_existed: usize,
}

/// A file write: path + new content
pub struct FileWrite {
    pub path: PathBuf,
    pub content: String,
}

/// Summary report for display
pub struct InferenceReport {
    pub new_link_count: usize,
    pub tc_count: usize,
    pub adr_count: usize,
    pub skipped_cross_cutting_count: usize,
    pub already_existed: usize,
    pub writes: Vec<FileWrite>,
    pub grouped: Vec<AdrGroup>,
}

/// Dry-run display group
pub struct AdrGroup {
    pub adr_id: String,
    pub adr_title: String,
    pub scope: String,
    pub skipped: bool,
    pub tc_links: Vec<TcLinkGroup>,
}

pub struct TcLinkGroup {
    pub tc_id: String,
    pub tc_title: String,
    pub new_features: Vec<String>,
}

/// Compute all inferred links without writing anything
pub fn compute_inference(graph: &KnowledgeGraph, opts: &InferenceOptions) -> InferenceResult {
    let adr_to_features = build_adr_feature_map(graph);
    let mut links = Vec::new();
    let mut skipped_cross_cutting = Vec::new();
    let mut already_existed: usize = 0;

    for adr in graph.adrs.values() {
        if let Some(ref filter) = opts.adr_filter {
            if adr.front.id != *filter { continue; }
        }
        if opts.skip_cross_cutting && adr.front.scope == AdrScope::CrossCutting {
            if !skipped_cross_cutting.contains(&adr.front.id) {
                skipped_cross_cutting.push(adr.front.id.clone());
            }
            continue;
        }
        let Some(feature_ids) = adr_to_features.get(&adr.front.id) else { continue };

        for tc in graph.tests.values() {
            if !tc.front.validates.adrs.contains(&adr.front.id) { continue; }
            for fid in feature_ids {
                if let Some(ref filter) = opts.feature_filter {
                    if fid != filter { continue; }
                }
                if tc.front.validates.features.contains(fid) {
                    already_existed += 1;
                } else if !links.iter().any(|l: &InferredLink| l.tc_id == tc.front.id && l.feature_id == *fid) {
                    links.push(InferredLink {
                        tc_id: tc.front.id.clone(),
                        feature_id: fid.clone(),
                        via_adr: adr.front.id.clone(),
                    });
                }
            }
        }
    }
    skipped_cross_cutting.sort();
    InferenceResult { links, skipped_cross_cutting, already_existed }
}

fn build_adr_feature_map(graph: &KnowledgeGraph) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for f in graph.features.values() {
        for adr_id in &f.front.adrs {
            map.entry(adr_id.clone()).or_default().push(f.front.id.clone());
        }
    }
    map
}

/// Prepare file writes for an inference result (without writing to disk).
/// Handles bidirectional updates: TC.validates.features and Feature.tests.
pub fn prepare_writes(graph: &KnowledgeGraph, result: &InferenceResult) -> Vec<FileWrite> {
    let mut tc_adds: HashMap<String, Vec<String>> = HashMap::new();
    let mut ft_adds: HashMap<String, Vec<String>> = HashMap::new();

    for link in &result.links {
        tc_adds.entry(link.tc_id.clone()).or_default().push(link.feature_id.clone());
        ft_adds.entry(link.feature_id.clone()).or_default().push(link.tc_id.clone());
    }

    let mut writes = Vec::new();
    for (tc_id, new_features) in &tc_adds {
        if let Some(tc) = graph.tests.get(tc_id.as_str()) {
            let mut front = tc.front.clone();
            for fid in new_features {
                if !front.validates.features.contains(fid) {
                    front.validates.features.push(fid.clone());
                }
            }
            front.validates.features.sort();
            writes.push(FileWrite { path: tc.path.clone(), content: parser::render_test(&front, &tc.body) });
        }
    }
    for (fid, new_tcs) in &ft_adds {
        if let Some(f) = graph.features.get(fid.as_str()) {
            let mut front = f.front.clone();
            for tc_id in new_tcs {
                if !front.tests.contains(tc_id) { front.tests.push(tc_id.clone()); }
            }
            front.tests.sort();
            writes.push(FileWrite { path: f.path.clone(), content: parser::render_feature(&front, &f.body) });
        }
    }
    writes
}

/// Build grouped display data for dry-run output
pub fn build_report(graph: &KnowledgeGraph, result: &InferenceResult, writes: &[FileWrite]) -> InferenceReport {
    let mut adr_ids: Vec<String> = result.links.iter().map(|l| l.via_adr.clone())
        .collect::<std::collections::HashSet<_>>().into_iter().collect();
    adr_ids.sort();
    let tc_ids: std::collections::HashSet<&str> = result.links.iter().map(|l| l.tc_id.as_str()).collect();
    let mut grouped = Vec::new();

    for adr_id in &adr_ids {
        if let Some(adr) = graph.adrs.get(adr_id.as_str()) {
            let mut tc_map: HashMap<String, Vec<String>> = HashMap::new();
            for link in &result.links {
                if link.via_adr == *adr_id {
                    tc_map.entry(link.tc_id.clone()).or_default().push(link.feature_id.clone());
                }
            }
            let mut tc_links: Vec<TcLinkGroup> = tc_map.into_iter().map(|(tc_id, mut features)| {
                features.sort();
                let title = graph.tests.get(tc_id.as_str()).map(|t| t.front.title.clone()).unwrap_or_default();
                TcLinkGroup { tc_id, tc_title: title, new_features: features }
            }).collect();
            tc_links.sort_by(|a, b| a.tc_id.cmp(&b.tc_id));
            grouped.push(AdrGroup {
                adr_id: adr.front.id.clone(), adr_title: adr.front.title.clone(),
                scope: adr.front.scope.to_string(), skipped: false, tc_links,
            });
        }
    }
    for adr_id in &result.skipped_cross_cutting {
        if let Some(adr) = graph.adrs.get(adr_id.as_str()) {
            grouped.push(AdrGroup {
                adr_id: adr.front.id.clone(), adr_title: adr.front.title.clone(),
                scope: "cross-cutting".to_string(), skipped: true, tc_links: Vec::new(),
            });
        }
    }

    InferenceReport {
        new_link_count: result.links.len(), tc_count: tc_ids.len(), adr_count: adr_ids.len(),
        skipped_cross_cutting_count: result.skipped_cross_cutting.len(),
        already_existed: result.already_existed,
        writes: writes.iter().map(|w| FileWrite { path: w.path.clone(), content: w.content.clone() }).collect(),
        grouped,
    }
}

/// Print dry-run output in ADR-027 format
pub fn print_dry_run(report: &InferenceReport) {
    println!("Transitive TC link inference (dry run)");
    println!("{}", "\u{2500}".repeat(60));
    for group in &report.grouped {
        println!("\n{} \u{2014} {}  [scope: {}]", group.adr_id, group.adr_title, group.scope);
        if group.skipped {
            println!("  \u{2192} skipped (cross-cutting, would link to all features)");
        } else {
            for tc in &group.tc_links {
                let fstr = tc.new_features.iter().map(|f| format!("+{}", f)).collect::<Vec<_>>().join(", ");
                println!("  {} {:<30} \u{2192} {}   ({} new)", tc.tc_id, tc.tc_title, fstr, tc.new_features.len());
            }
        }
    }
    println!("{}", "\u{2500}".repeat(60));
    println!("{} new TC\u{2192}Feature links across {} TCs and {} ADRs",
        report.new_link_count, report.tc_count, report.adr_count);
    if report.skipped_cross_cutting_count > 0 {
        println!("{} ADRs skipped (cross-cutting)", report.skipped_cross_cutting_count);
    }
    println!("{} links already existed (idempotent)", report.already_existed);
    println!();
    println!("Run without --dry-run to apply.");
}

/// Print summary after applying inference
pub fn print_summary(report: &InferenceReport) {
    if report.new_link_count == 0 {
        println!("0 new links. Graph TC\u{2192}Feature links are fully inferred.");
        return;
    }
    println!("{} new TC\u{2192}Feature links across {} TCs and {} ADRs",
        report.new_link_count, report.tc_count, report.adr_count);
    if report.skipped_cross_cutting_count > 0 {
        println!("{} ADRs skipped (cross-cutting)", report.skipped_cross_cutting_count);
    }
    println!("{} links already existed (idempotent)", report.already_existed);
}

/// High-level: run inference, print output, and optionally apply writes.
pub fn run_inference(graph: &KnowledgeGraph, opts: &InferenceOptions, dry_run: bool) -> Result<usize, ProductError> {
    let result = compute_inference(graph, opts);
    let writes = prepare_writes(graph, &result);
    let report = build_report(graph, &result, &writes);

    if dry_run {
        print_dry_run(&report);
        return Ok(0);
    }
    if report.new_link_count == 0 {
        print_summary(&report);
        return Ok(0);
    }
    let write_pairs: Vec<(&std::path::Path, &str)> =
        report.writes.iter().map(|w| (w.path.as_path(), w.content.as_str())).collect();
    crate::fileops::write_batch_atomic(&write_pairs)?;
    print_summary(&report);
    Ok(report.new_link_count)
}
