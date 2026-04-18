//! Structural ADR conflict check + conflict-bundle handlers (FT-045, ADR-040).

use product_lib::graph::KnowledgeGraph;
use product_lib::types::{Adr, AdrScope};

use super::{load_graph, BoxResult};

type Finding = (String, String, String); // (code, adr, message)

/// Structural-only `adr check-conflicts`. No LLM call. Reports cycle
/// detection on supersedes, symmetry on `superseded-by`, domain-overlap
/// against cross-cutting ADRs, and scope-consistency.
pub fn adr_check_conflicts(id: Option<String>, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;

    let targets: Vec<String> = match id {
        Some(ref x) => vec![x.clone()],
        None => {
            let mut ids: Vec<String> = graph.adrs.keys().cloned().collect();
            ids.sort();
            ids
        }
    };

    let mut findings: Vec<Finding> = Vec::new();
    for adr_id in &targets {
        let adr = match graph.adrs.get(adr_id) {
            Some(a) => a,
            None => {
                eprintln!("error: ADR {} not found", adr_id);
                std::process::exit(1);
            }
        };
        check_one_adr(adr_id, adr, &graph, &mut findings);
    }

    render_findings(&findings, fmt);

    let has_error = findings.iter().any(|(code, _, _)| code.starts_with('E'));
    if has_error {
        std::process::exit(1);
    }
    Ok(())
}

fn check_one_adr(adr_id: &str, adr: &Adr, graph: &KnowledgeGraph, findings: &mut Vec<Finding>) {
    if has_supersession_cycle(adr_id, graph) {
        findings.push((
            "E004".into(),
            adr_id.into(),
            "supersession cycle detected".into(),
        ));
    }
    check_supersession_symmetry(adr_id, adr, graph, findings);
    check_domain_overlap(adr_id, adr, graph, findings);
    check_scope_consistency(adr_id, adr, findings);
}

fn check_supersession_symmetry(
    adr_id: &str,
    adr: &Adr,
    graph: &KnowledgeGraph,
    findings: &mut Vec<Finding>,
) {
    for by in &adr.front.superseded_by {
        if let Some(succ) = graph.adrs.get(by) {
            if !succ.front.supersedes.contains(&adr_id.to_string()) {
                findings.push((
                    "W022".into(),
                    adr_id.into(),
                    format!(
                        "supersession asymmetry: {} does not list {} in supersedes",
                        by, adr_id
                    ),
                ));
            }
        }
    }
    for sup in &adr.front.supersedes {
        if let Some(other) = graph.adrs.get(sup) {
            if !other.front.superseded_by.contains(&adr_id.to_string()) {
                findings.push((
                    "W022".into(),
                    adr_id.into(),
                    format!(
                        "supersession asymmetry: {} does not list {} in superseded-by",
                        sup, adr_id
                    ),
                ));
            }
        }
    }
}

fn check_domain_overlap(
    adr_id: &str,
    adr: &Adr,
    graph: &KnowledgeGraph,
    findings: &mut Vec<Finding>,
) {
    if adr.front.scope == AdrScope::CrossCutting {
        return;
    }
    for other in graph.adrs.values() {
        if other.front.id == adr_id {
            continue;
        }
        if other.front.scope == AdrScope::CrossCutting
            && other.front.domains.iter().any(|d| adr.front.domains.contains(d))
        {
            findings.push((
                "W023".into(),
                adr_id.into(),
                format!(
                    "shares domain with cross-cutting {}; verify intent is aligned",
                    other.front.id
                ),
            ));
        }
    }
}

fn check_scope_consistency(adr_id: &str, adr: &Adr, findings: &mut Vec<Finding>) {
    if adr.front.scope == AdrScope::FeatureSpecific && adr.front.features.len() > 3 {
        findings.push((
            "W024".into(),
            adr_id.into(),
            format!(
                "scope is feature-specific but is linked to {} features — consider scope: domain or cross-cutting",
                adr.front.features.len()
            ),
        ));
    }
}

fn render_findings(findings: &[Finding], fmt: &str) {
    if fmt == "json" {
        let arr: Vec<serde_json::Value> = findings
            .iter()
            .map(|(code, adr, msg)| {
                serde_json::json!({ "code": code, "adr": adr, "message": msg })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else if findings.is_empty() {
        println!("No structural ADR conflicts.");
    } else {
        for (code, adr, msg) in findings {
            println!("[{}] {} — {}", code, adr, msg);
        }
    }
}

/// Emit an LLM-ready conflict-bundle to stdout (no LLM call inside Product).
pub fn adr_conflict_bundle(id: &str, format: &str) -> BoxResult {
    let (_, root, graph) = load_graph()?;
    let markdown = match product_lib::gap::conflict::bundle_for_adr(id, &graph, &root) {
        Some(b) => b,
        None => {
            eprintln!("error: ADR {} not found", id);
            std::process::exit(1);
        }
    };
    if format == "json" {
        let v = serde_json::json!({ "bundle": markdown });
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    } else {
        print!("{}", markdown);
    }
    Ok(())
}

fn has_supersession_cycle(
    start: &str,
    graph: &product_lib::graph::KnowledgeGraph,
) -> bool {
    let mut visited = std::collections::HashSet::new();
    let mut stack: Vec<String> = vec![start.to_string()];
    while let Some(cur) = stack.pop() {
        if !visited.insert(cur.clone()) {
            if cur == start {
                return true;
            }
            continue;
        }
        if let Some(adr) = graph.adrs.get(&cur) {
            for s in &adr.front.supersedes {
                if s == start {
                    return true;
                }
                stack.push(s.clone());
            }
        }
    }
    false
}
