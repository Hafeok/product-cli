//! `product cycle-times` — thin adapter over the cycle-times slice.

use super::{load_graph_typed, CmdResult, Output};
use chrono::{DateTime, FixedOffset};
use product_lib::cycle_times::{
    self, build_in_progress_report, build_report, render_csv, render_in_progress_text, render_text,
    TagTimestamps,
};
use product_lib::error::ProductError;
use product_lib::graph::KnowledgeGraph;
#[allow(unused_imports)]
use product_lib::types;
use product_lib::tags;
use std::path::Path;
use std::process::Command;

/// Collect started/complete tag timestamps for every feature in the graph.
/// Uses `git for-each-ref` to avoid per-feature shell-outs.
pub(crate) fn read_tag_timestamps(root: &Path, graph: &KnowledgeGraph) -> TagTimestamps {
    if !tags::is_git_repo(root) {
        return std::collections::HashMap::new();
    }
    let stdout = run_for_each_ref(root);
    let (started_map, complete_map) = parse_tag_lines(&stdout, graph);
    collect_per_feature(graph, &started_map, &complete_map)
}

fn run_for_each_ref(root: &Path) -> String {
    let output = Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname:short)\t%(creatordate:iso-strict)",
            "refs/tags/product/",
        ])
        .current_dir(root)
        .output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => String::new(),
    }
}

type TagsById = std::collections::HashMap<String, String>;
type CompleteById = std::collections::HashMap<String, (String, String)>;

fn parse_tag_lines(stdout: &str, graph: &KnowledgeGraph) -> (TagsById, CompleteById) {
    let mut started_map: TagsById = std::collections::HashMap::new();
    let mut complete_map: CompleteById = std::collections::HashMap::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        let Some(name) = parts.first() else { continue };
        let ts = parts.get(1).copied().unwrap_or("").trim().to_string();
        let Some((id, event)) = tags::parse_tag_name(name) else { continue };
        if !graph.features.contains_key(&id) {
            continue;
        }
        if event == "started" {
            started_map.insert(id, ts);
        } else if event == "complete" || event.starts_with("complete-v") {
            let better = match complete_map.get(&id) {
                Some((ev, _)) => event < *ev,
                None => true,
            };
            if better {
                complete_map.insert(id, (event, ts));
            }
        }
    }
    (started_map, complete_map)
}

fn collect_per_feature(
    graph: &KnowledgeGraph,
    started: &TagsById,
    complete: &CompleteById,
) -> TagTimestamps {
    let mut out: TagTimestamps = std::collections::HashMap::new();
    for id in graph.features.keys() {
        let s = started.get(id).cloned();
        let c = complete.get(id).map(|(_, ts)| ts.clone());
        if s.is_some() || c.is_some() {
            out.insert(id.clone(), (s, c));
        }
    }
    out
}

pub(crate) fn handle_cycle_times(
    recent: Option<usize>,
    phase: Option<u32>,
    in_progress: bool,
    fmt: &str,
) -> CmdResult {
    let (config, root, graph) = load_graph_typed()?;
    let tag_ts = read_tag_timestamps(&root, &graph);

    let window = recent.unwrap_or(config.cycle_times.recent_window);
    let threshold = config.cycle_times.trend_threshold;
    let min_features = config.cycle_times.min_features;

    if in_progress {
        let now = chrono_now();
        let report = build_in_progress_report(&graph, &tag_ts, &now, window);
        if fmt == "json" {
            let json =
                serde_json::to_value(&report).unwrap_or(serde_json::Value::Null);
            return Ok(Output::json(json));
        }
        if fmt == "csv" {
            return Ok(Output::text(
                "feature_id,started,status,elapsed_days,phase\n".to_string()
                    + &report
                        .features
                        .iter()
                        .map(|r| {
                            format!(
                                "{},{},{},{:.1},{}",
                                r.id, r.started, r.status, r.elapsed_days, r.phase
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
            ));
        }
        let text = render_in_progress_text(&report);
        let json = serde_json::to_value(&report).unwrap_or(serde_json::Value::Null);
        return Ok(Output::both(text, json));
    }

    let report = build_report(&graph, &tag_ts, window, threshold, phase);

    // Below min-features: empty result, exit 0 (ADR-046 §8).
    if report.summary.count < min_features {
        if fmt == "json" {
            let empty = product_lib::cycle_times::CycleTimeReport {
                features: vec![],
                summary: product_lib::cycle_times::Summary {
                    count: report.summary.count,
                    recent_5: None,
                    all: None,
                    trend: None,
                },
            };
            let json = serde_json::to_value(&empty).unwrap_or(serde_json::Value::Null);
            return Ok(Output::json(json));
        }
        if fmt == "csv" {
            // Header only.
            return Ok(Output::text(
                "feature_id,started,completed,cycle_time_days,phase".to_string(),
            ));
        }
        let msg = format!(
            "No completed features with cycle-time data.\n\nAt least {} complete features (both started + complete tags) are required for summary statistics.",
            min_features
        );
        return Ok(Output::text(msg));
    }

    if fmt == "csv" {
        return Ok(Output::text(render_csv(&report).trim_end().to_string()));
    }
    let text = render_text(&report);
    let json = serde_json::to_value(&report).unwrap_or(serde_json::Value::Null);
    if fmt == "json" {
        return Ok(Output::json(json));
    }
    Ok(Output::both(text, json))
}

/// Forecast handler — uses BoxResult to express exit-code-2 semantics
/// for insufficient-data (ADR-046 §8).
pub(crate) fn handle_forecast(
    id: Option<&str>,
    phase: Option<u32>,
    naive: bool,
    sample_size: Option<usize>,
    fmt: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !naive {
        eprintln!(
            "error: --naive flag is required. No unlabelled forecast surface exists (ADR-046)."
        );
        std::process::exit(1);
    }

    let (config, root, graph) = match load_graph_typed() {
        Ok(v) => v,
        Err(e) => return Err(Box::new(e)),
    };
    let tag_ts = read_tag_timestamps(&root, &graph);
    let window = sample_size.unwrap_or(config.cycle_times.recent_window);
    let threshold = config.cycle_times.trend_threshold;
    let min_features = config.cycle_times.min_features;

    let report = build_report(&graph, &tag_ts, window, threshold, None);
    if report.summary.count < min_features {
        eprintln!("Insufficient data for naive projection.");
        eprintln!(
            "Only {} features with cycle-time data; requires at least {}.",
            report.summary.count, min_features
        );
        eprintln!("View current cycle times:  product cycle-times");
        std::process::exit(2);
    }
    let Some(recent) = report.summary.recent_5.clone() else {
        eprintln!("Insufficient data for naive projection.");
        std::process::exit(2);
    };
    let sample_count = report.summary.count.min(window);

    let result: CmdResult = if let Some(fid) = id {
        forecast_single(&graph, &tag_ts, fid, &recent, sample_count)
    } else if let Some(p) = phase {
        forecast_phase(&graph, p, &recent, sample_count)
    } else {
        return Err("error: specify a feature ID or --phase N".into());
    };

    super::output::render_result(result, fmt)
}

fn forecast_single(
    graph: &KnowledgeGraph,
    tag_ts: &TagTimestamps,
    feature_id: &str,
    recent: &cycle_times::Stats,
    sample_count: usize,
) -> CmdResult {
    let Some(feat) = graph.features.get(feature_id) else {
        return Err(ProductError::NotFound(format!("feature {}", feature_id)));
    };
    let started = tag_ts
        .get(feature_id)
        .and_then(|(s, _)| s.clone())
        .and_then(|s| cycle_times::parse_instant(&s));
    let now = chrono_now();
    let elapsed = started
        .map(|st| cycle_times::elapsed_days(&st, &now).max(0.0))
        .unwrap_or(0.0);
    let today = now.date_naive();
    let fc = cycle_times::project_naive_single(today, elapsed, recent);

    let title = format!(
        "{} \u{2014} {}  [{}, started {}]",
        feature_id,
        feat.front.title,
        feat.front.status,
        started
            .map(|st| st.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "—".to_string()),
    );
    let header = "".to_string();
    let text = cycle_times::render_forecast_single(
        &title,
        &header,
        cycle_times::round1(elapsed),
        recent,
        sample_count,
        &fc,
    );
    let json = serde_json::json!({
        "feature": feature_id,
        "elapsed_days": cycle_times::round1(elapsed),
        "recent": recent,
        "forecast": fc,
        "sample_size": sample_count,
    });
    Ok(Output::both(text, json))
}

fn forecast_phase(
    graph: &KnowledgeGraph,
    phase: u32,
    recent: &cycle_times::Stats,
    sample_count: usize,
) -> CmdResult {
    let mut remaining: Vec<String> = graph
        .features
        .values()
        .filter(|f| f.front.phase == phase && f.front.status != product_lib::types::FeatureStatus::Complete)
        .map(|f| f.front.id.clone())
        .collect();
    remaining.sort();
    let k = remaining.len();
    if k == 0 {
        return Ok(Output::text(format!(
            "Phase {} has no remaining features.",
            phase
        )));
    }
    let now = chrono_now();
    let today = now.date_naive();
    let fc = cycle_times::project_naive_phase(today, k, recent);
    let text = cycle_times::render_forecast_phase(phase, &remaining, recent, sample_count, &fc);
    let json = serde_json::json!({
        "phase": phase,
        "remaining": remaining,
        "recent": recent,
        "forecast": fc,
        "sample_size": sample_count,
    });
    Ok(Output::both(text, json))
}

fn chrono_now() -> DateTime<FixedOffset> {
    chrono::Local::now().fixed_offset()
}
