//! `product drift diff` handler — LLM-ready drift bundle (FT-045, ADR-040).

use product_lib::drift;
use product_lib::types::FeatureStatus;
use std::process;

use super::BoxResult;

pub fn drift_diff(
    feature_id: Option<String>,
    all_complete: bool,
    changed: bool,
    format: &str,
    graph: &product_lib::graph::KnowledgeGraph,
    root: &std::path::Path,
    config: &product_lib::config::ProductConfig,
) -> BoxResult {
    let depth = config.tags.implementation_depth;
    let mut had_w020 = false;

    let mut rendered = String::new();
    let feature_ids: Vec<String> = if all_complete {
        let mut ids: Vec<String> = graph
            .features
            .values()
            .filter(|f| f.front.status == FeatureStatus::Complete)
            .map(|f| f.front.id.clone())
            .collect();
        ids.sort();
        ids
    } else if changed {
        changed_feature_ids(root, graph)
    } else if let Some(id) = feature_id {
        vec![id]
    } else {
        eprintln!("error: specify a feature ID, or use --all-complete or --changed");
        process::exit(1);
    };

    for fid in &feature_ids {
        match drift::diff_for_feature(fid, graph, root, depth) {
            Some(result) => {
                if result.warn_w020 {
                    had_w020 = true;
                    eprintln!(
                        "warning[W020]: no completion tag for {} — drift diff emitted without implementation anchor",
                        fid
                    );
                }
                rendered.push_str(&result.markdown);
                rendered.push_str("\n---\n\n");
            }
            None => {
                eprintln!("error: feature {} not found", fid);
                process::exit(1);
            }
        }
    }

    if format == "json" {
        let value = serde_json::json!({ "bundle": rendered });
        println!("{}", serde_json::to_string_pretty(&value).unwrap_or_default());
    } else {
        print!("{}", rendered);
    }

    if had_w020 {
        process::exit(2);
    }
    Ok(())
}

fn changed_feature_ids(
    root: &std::path::Path,
    graph: &product_lib::graph::KnowledgeGraph,
) -> Vec<String> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .current_dir(root)
        .output();
    let changed = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => return Vec::new(),
    };
    let mut ids: Vec<String> = Vec::new();
    for line in changed.lines() {
        if line.contains("features/") {
            if let Some(id) = line.rsplit('/').next().and_then(|fname| {
                let parts: Vec<&str> = fname.splitn(3, '-').collect();
                if parts.len() >= 2 {
                    Some(format!("{}-{}", parts[0], parts[1]))
                } else {
                    None
                }
            }) {
                if graph.features.contains_key(&id) && !ids.contains(&id) {
                    ids.push(id);
                }
            }
        }
    }
    ids.sort();
    ids
}
