//! Unit tests for the cycle-times pure slice.

use super::compute::*;
use super::model::*;
use crate::graph::KnowledgeGraph;
use crate::types::{self, Feature, FeatureFrontMatter, FeatureStatus};
use chrono::{DateTime, NaiveDate};
use std::collections::HashMap;
use std::path::PathBuf;

fn make_feature(id: &str, phase: u32, status: FeatureStatus) -> Feature {
    Feature {
        path: PathBuf::from(format!("docs/features/{}.md", id)),
        body: String::new(),
        front: FeatureFrontMatter {
            id: id.to_string(),
            title: id.to_string(),
            phase,
            status,
            depends_on: vec![],
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: std::collections::HashMap::new(),
            due_date: None,
            bundle: None,
        },
    }
}

fn tag_ts(entries: &[(&str, Option<&str>, Option<&str>)]) -> HashMap<String, (Option<String>, Option<String>)> {
    let mut m = HashMap::new();
    for (id, s, c) in entries {
        m.insert(
            id.to_string(),
            (s.map(|x| x.to_string()), c.map(|x| x.to_string())),
        );
    }
    m
}

fn graph_with(features: Vec<Feature>) -> KnowledgeGraph {
    KnowledgeGraph::build(features, vec![], vec![])
}

#[test]
fn build_report_lists_complete_features() {
    let g = graph_with(vec![
        make_feature("FT-101", 1, FeatureStatus::Complete),
        make_feature("FT-102", 1, FeatureStatus::Complete),
        make_feature("FT-103", 1, FeatureStatus::Complete),
    ]);
    let ts = tag_ts(&[
        ("FT-101", Some("2026-04-08T13:00:00+00:00"), Some("2026-04-11T09:14:00+00:00")),
        ("FT-102", Some("2026-04-12T10:30:00+00:00"), Some("2026-04-17T15:42:00+00:00")),
        ("FT-103", Some("2026-04-15T08:00:00+00:00"), Some("2026-04-18T18:00:00+00:00")),
    ]);
    let report = build_report(&g, &ts, 5, 0.25, None);
    assert_eq!(report.summary.count, 3);
    assert_eq!(report.features.len(), 3);
    assert_eq!(report.features[0].id, "FT-101");
    assert!(report.summary.trend.is_none(), "count<6 must omit trend");
}

#[test]
fn build_report_excludes_features_without_started() {
    let g = graph_with(vec![
        make_feature("FT-201", 1, FeatureStatus::Complete),
        make_feature("FT-202", 1, FeatureStatus::Complete),
    ]);
    let ts = tag_ts(&[
        ("FT-201", Some("2026-04-08T13:00:00+00:00"), Some("2026-04-11T09:00:00+00:00")),
        ("FT-202", None, Some("2026-04-15T00:00:00+00:00")),
    ]);
    let r = build_report(&g, &ts, 5, 0.25, None);
    assert_eq!(r.summary.count, 1);
    assert_eq!(r.features.len(), 1);
    assert_eq!(r.features[0].id, "FT-201");
}

#[test]
fn build_report_excludes_features_without_complete() {
    let g = graph_with(vec![
        make_feature("FT-301", 1, FeatureStatus::Complete),
        make_feature("FT-302", 1, FeatureStatus::InProgress),
    ]);
    let ts = tag_ts(&[
        ("FT-301", Some("2026-04-08T13:00:00+00:00"), Some("2026-04-11T09:00:00+00:00")),
        ("FT-302", Some("2026-04-15T00:00:00+00:00"), None),
    ]);
    let r = build_report(&g, &ts, 5, 0.25, None);
    assert_eq!(r.summary.count, 1);
    assert_eq!(r.features[0].id, "FT-301");
}

fn build_14_feature_fixture() -> (Vec<Feature>, HashMap<String, (Option<String>, Option<String>)>) {
    let fts: Vec<Feature> = (1..=14)
        .map(|i| make_feature(&format!("FT-{:03}", 100 + i), 1, FeatureStatus::Complete))
        .collect();
    let days = [2.84, 5.12, 3.21, 8.44, 2.10, 4.88, 1.95, 11.32, 3.67, 2.44, 6.78, 4.01, 3.55, 7.22];
    let start = NaiveDate::from_ymd_opt(2026, 1, 1).expect("date");
    let mut ts = HashMap::new();
    for (i, d) in days.iter().enumerate() {
        let id = format!("FT-{:03}", 101 + i);
        let started = start + chrono::Duration::days((i as i64) * 20);
        let comp_dt = started.and_hms_opt(0, 0, 0).expect("hms")
            + chrono::Duration::seconds((*d * 86400.0) as i64);
        ts.insert(
            id,
            (
                Some(format!("{}T00:00:00+00:00", started.format("%Y-%m-%d"))),
                Some(format!("{}+00:00", comp_dt.format("%Y-%m-%dT%H:%M:%S"))),
            ),
        );
    }
    (fts, ts)
}

#[test]
fn recent_5_computed_correctly() {
    let (fts, ts) = build_14_feature_fixture();
    let g = graph_with(fts);
    let r = build_report(&g, &ts, 5, 0.25, None);
    assert_eq!(r.summary.count, 14);
    let recent = r.summary.recent_5.expect("recent stats");
    assert!((recent.median - 4.0).abs() <= 0.1, "median got {}", recent.median);
    assert!((recent.min - 2.4).abs() <= 0.1, "min got {}", recent.min);
    assert!((recent.max - 7.2).abs() <= 0.1, "max got {}", recent.max);
    assert!(r.summary.trend.is_some(), "trend should be populated with ≥6 samples");
}

#[test]
fn first_complete_tag_wins_when_v2_exists() {
    // Simulated with explicit earliest-complete timestamp in the map.
    let g = graph_with(vec![make_feature("FT-401", 1, FeatureStatus::Complete)]);
    let ts = tag_ts(&[(
        "FT-401",
        Some("2026-04-08T13:00:00+00:00"),
        // first complete — caller must pass this earliest value even if complete-v2 exists.
        Some("2026-04-11T09:14:00+00:00"),
    )]);
    let r = build_report(&g, &ts, 5, 0.25, None);
    assert_eq!(r.features.len(), 1);
    // Cycle time ≈ 2.8 days
    assert!(
        (r.features[0].cycle_time_days - 2.8).abs() <= 0.2,
        "expected ≈ 2.8, got {}",
        r.features[0].cycle_time_days
    );
}

#[test]
fn elapsed_exceeds_sample_clamps_to_today() {
    let recent = Stats { median: 4.01, min: 2.44, max: 7.22 };
    let today = NaiveDate::from_ymd_opt(2026, 6, 10).expect("date");
    let fc = project_naive_single(today, 10.0, &recent);
    assert_eq!(fc.likely, "2026-06-10");
    assert_eq!(fc.optimistic, "2026-06-10");
    assert_eq!(fc.pessimistic, "2026-06-10");
}

#[test]
fn in_progress_report_elapsed() {
    let mut f = make_feature("FT-015", 1, FeatureStatus::InProgress);
    f.front.status = FeatureStatus::InProgress;
    let g = graph_with(vec![f]);
    let ts = tag_ts(&[("FT-015", Some("2026-05-20T07:00:00+00:00"), None)]);
    let now = DateTime::parse_from_rfc3339("2026-05-22T12:00:00+00:00").expect("now");
    let report = build_in_progress_report(&g, &ts, &now, 5);
    assert_eq!(report.features.len(), 1);
    assert!(
        (report.features[0].elapsed_days - 2.2).abs() <= 0.1,
        "expected ≈2.2, got {}",
        report.features[0].elapsed_days
    );
}

#[test]
fn types_import() {
    // Ensure the types module is reachable via the super crate path.
    let _s: types::FeatureStatus = types::FeatureStatus::Planned;
}
