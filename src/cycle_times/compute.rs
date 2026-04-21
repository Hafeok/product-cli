//! Pure builders for cycle-time reports — no I/O, no printing.
//!
//! Callers supply `(graph, tag_timestamps, now, config)` and receive a typed
//! `CycleTimeReport` or `InProgressReport`. Tag reads live in the adapter.

use super::model::{
    CycleTimeReport, CycleTimeRow, InProgressReport, InProgressRow, NaiveForecast, Stats, Summary,
    Trend,
};
use crate::graph::KnowledgeGraph;
use crate::types::FeatureStatus;
use chrono::{DateTime, FixedOffset, NaiveDate};
use std::collections::HashMap;

/// Round a float to one decimal place.
pub fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// Median of a sample (sorted or unsorted).
pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut v = values.to_vec();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    Some(if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    })
}

/// Compute min / median / max for a sample.
pub fn stats_of(values: &[f64]) -> Option<Stats> {
    let m = median(values)?;
    let mn = values
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let mx = values
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    Some(Stats {
        median: round1(m),
        min: round1(mn),
        max: round1(mx),
    })
}

/// Parse ISO 8601 instant (e.g. `2026-04-08 13:00:00 +0000` or RFC 3339).
pub fn parse_instant(s: &str) -> Option<DateTime<FixedOffset>> {
    let trimmed = s.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S %z") {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%z") {
        return Some(dt);
    }
    if let Ok(dt) = DateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S%z") {
        return Some(dt);
    }
    None
}

/// Seconds between two instants as days.
pub fn elapsed_days(start: &DateTime<FixedOffset>, end: &DateTime<FixedOffset>) -> f64 {
    let secs = (*end - *start).num_seconds();
    secs as f64 / 86400.0
}

/// Classify trend from recent-N median vs all-time median (ADR-046 §4).
/// Returns `None` when either sample is empty or denominator is 0.
pub fn classify_trend(
    recent_median: f64,
    all_median: f64,
    threshold: f64,
) -> Option<Trend> {
    if all_median <= 0.0 {
        return None;
    }
    let ratio = (recent_median - all_median) / all_median;
    if ratio.abs() <= threshold {
        Some(Trend::Stable)
    } else if ratio < -threshold {
        Some(Trend::Accelerating)
    } else {
        Some(Trend::Slowing)
    }
}

/// Per-feature tag timestamps. `None` in either slot means the tag is missing.
pub type TagTimestamps = HashMap<String, (Option<String>, Option<String>)>;

/// Build the `CycleTimeReport` for every feature with both tags.
///
/// `phase_filter` restricts the output (but recent/all stats are computed
/// over every feature with both tags — consumers relying on phase-specific
/// stats should filter themselves before calling).
pub fn build_report(
    graph: &KnowledgeGraph,
    tag_ts: &TagTimestamps,
    recent_window: usize,
    trend_threshold: f64,
    phase_filter: Option<u32>,
) -> CycleTimeReport {
    let mut rows: Vec<CycleTimeRow> = Vec::new();
    for feat in graph.features.values() {
        let id = &feat.front.id;
        let Some((started, completed)) = tag_ts.get(id) else {
            continue;
        };
        let (Some(st), Some(cp)) = (started, completed) else {
            continue;
        };
        let (Some(st_dt), Some(cp_dt)) = (parse_instant(st), parse_instant(cp)) else {
            continue;
        };
        if cp_dt < st_dt {
            continue;
        }
        let days = round1(elapsed_days(&st_dt, &cp_dt));
        rows.push(CycleTimeRow {
            id: id.clone(),
            phase: feat.front.phase,
            started: format_date(&st_dt),
            completed: format_date(&cp_dt),
            cycle_time_days: days,
        });
    }
    // Sort by completed timestamp ascending so "recent" = last N in the list.
    rows.sort_by(|a, b| a.completed.cmp(&b.completed));

    let all_days: Vec<f64> = rows.iter().map(|r| r.cycle_time_days).collect();
    let recent_days: Vec<f64> = if all_days.len() > recent_window {
        all_days[all_days.len() - recent_window..].to_vec()
    } else {
        all_days.clone()
    };

    let all_stats = stats_of(&all_days);
    let recent_stats = stats_of(&recent_days);

    // Trend requires ≥ 6 complete features (ADR-046 §4).
    let trend = if all_days.len() >= 6 {
        match (recent_stats.as_ref(), all_stats.as_ref()) {
            (Some(r), Some(a)) => classify_trend(r.median, a.median, trend_threshold),
            _ => None,
        }
    } else {
        None
    };

    // Apply phase filter to displayed rows (stats computed across all).
    let displayed: Vec<CycleTimeRow> = match phase_filter {
        Some(p) => rows.into_iter().filter(|r| r.phase == p).collect(),
        None => rows,
    };

    CycleTimeReport {
        summary: Summary {
            count: all_days.len(),
            recent_5: recent_stats,
            all: all_stats,
            trend,
        },
        features: displayed,
    }
}

/// Build the in-progress report — elapsed-so-far for every feature with a
/// `started` tag but no `complete` tag (ADR-046 §12).
pub fn build_in_progress_report(
    graph: &KnowledgeGraph,
    tag_ts: &TagTimestamps,
    now: &DateTime<FixedOffset>,
    recent_window: usize,
) -> InProgressReport {
    let mut rows: Vec<InProgressRow> = Vec::new();
    for feat in graph.features.values() {
        // Only surface features whose status is not complete/abandoned.
        if feat.front.status == FeatureStatus::Complete
            || feat.front.status == FeatureStatus::Abandoned
        {
            continue;
        }
        let id = &feat.front.id;
        let Some((started, completed)) = tag_ts.get(id) else {
            continue;
        };
        if completed.is_some() {
            continue;
        }
        let Some(st) = started else { continue };
        let Some(st_dt) = parse_instant(st) else {
            continue;
        };
        let days = round1(elapsed_days(&st_dt, now).max(0.0));
        rows.push(InProgressRow {
            id: id.clone(),
            phase: feat.front.phase,
            started: format_date(&st_dt),
            status: feat.front.status.to_string(),
            elapsed_days: days,
        });
    }
    rows.sort_by(|a, b| a.started.cmp(&b.started));

    // Compute recent-N median from complete features as reference.
    let mut complete_days: Vec<f64> = Vec::new();
    for feat in graph.features.values() {
        let id = &feat.front.id;
        let Some((started, completed)) = tag_ts.get(id) else {
            continue;
        };
        let (Some(st), Some(cp)) = (started, completed) else {
            continue;
        };
        let (Some(st_dt), Some(cp_dt)) = (parse_instant(st), parse_instant(cp)) else {
            continue;
        };
        if cp_dt < st_dt {
            continue;
        }
        complete_days.push(round1(elapsed_days(&st_dt, &cp_dt)));
    }
    // Sort by value; but recent means last-by-completion. Since we lost the
    // completion order, just sort by value and take a conservative median.
    // Use the same sample-window semantic: last N after date-sort.
    let mut rows_for_median: Vec<(String, f64)> = Vec::new();
    for feat in graph.features.values() {
        let id = &feat.front.id;
        let Some((started, completed)) = tag_ts.get(id) else {
            continue;
        };
        let (Some(st), Some(cp)) = (started, completed) else {
            continue;
        };
        let (Some(st_dt), Some(cp_dt)) = (parse_instant(st), parse_instant(cp)) else {
            continue;
        };
        if cp_dt < st_dt {
            continue;
        }
        rows_for_median.push((format_date(&cp_dt), round1(elapsed_days(&st_dt, &cp_dt))));
    }
    rows_for_median.sort_by(|a, b| a.0.cmp(&b.0));
    let recent: Vec<f64> = if rows_for_median.len() > recent_window {
        rows_for_median[rows_for_median.len() - recent_window..]
            .iter()
            .map(|(_, v)| *v)
            .collect()
    } else {
        rows_for_median.iter().map(|(_, v)| *v).collect()
    };
    let reference_median = median(&recent).map(round1);

    InProgressReport {
        features: rows,
        reference_median,
    }
}

fn format_date(dt: &DateTime<FixedOffset>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

/// Naive projection for a single feature (ADR-046 §6).
///
/// All three outcomes clamp to today when elapsed exceeds the respective
/// recent stat (§9). Returns `YYYY-MM-DD` strings in UTC-equivalent.
pub fn project_naive_single(
    today: NaiveDate,
    elapsed_days: f64,
    recent: &Stats,
) -> NaiveForecast {
    let off_like = (recent.median - elapsed_days).max(0.0);
    let off_opt = (recent.min - elapsed_days).max(0.0);
    let off_pess = (recent.max - elapsed_days).max(0.0);
    NaiveForecast {
        likely: add_days(today, off_like),
        optimistic: add_days(today, off_opt),
        pessimistic: add_days(today, off_pess),
    }
}

/// Naive projection for a phase (ADR-046 §7).
pub fn project_naive_phase(today: NaiveDate, k: usize, recent: &Stats) -> NaiveForecast {
    let k_f = k as f64;
    NaiveForecast {
        likely: add_days(today, k_f * recent.median),
        optimistic: add_days(today, k_f * recent.min),
        pessimistic: add_days(today, k_f * recent.max),
    }
}

fn add_days(today: NaiveDate, days: f64) -> String {
    let whole = days.round() as i64;
    let shifted = today + chrono::Duration::days(whole);
    shifted.format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod compute_tests {
    use super::*;

    #[test]
    fn median_odd_and_even() {
        assert_eq!(median(&[1.0, 2.0, 3.0]), Some(2.0));
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
        assert_eq!(median(&[]), None);
    }

    #[test]
    fn stats_of_sample() {
        let s = stats_of(&[2.44, 6.78, 4.01, 3.55, 7.22]).expect("stats");
        assert_eq!(s.median, 4.0);
        assert_eq!(s.min, 2.4);
        assert_eq!(s.max, 7.2);
    }

    #[test]
    fn trend_stable_within_threshold() {
        let t = classify_trend(4.01, 4.02, 0.25).expect("trend");
        assert_eq!(t, Trend::Stable);
    }

    #[test]
    fn trend_accelerating_below_threshold() {
        let t = classify_trend(3.0, 6.0, 0.25).expect("trend");
        assert_eq!(t, Trend::Accelerating);
    }

    #[test]
    fn trend_slowing_above_threshold() {
        let t = classify_trend(8.0, 4.0, 0.25).expect("trend");
        assert_eq!(t, Trend::Slowing);
    }

    #[test]
    fn project_clamps_when_elapsed_exceeds() {
        let recent = Stats { median: 4.01, min: 2.44, max: 7.22 };
        let today = NaiveDate::from_ymd_opt(2026, 6, 10).expect("date");
        let fc = project_naive_single(today, 10.0, &recent);
        assert_eq!(fc.likely, "2026-06-10");
        assert_eq!(fc.optimistic, "2026-06-10");
        assert_eq!(fc.pessimistic, "2026-06-10");
    }

    #[test]
    fn project_normal_case() {
        let recent = Stats { median: 4.01, min: 2.44, max: 7.22 };
        let today = NaiveDate::from_ymd_opt(2026, 5, 22).expect("date");
        let fc = project_naive_single(today, 2.2, &recent);
        // 2.2d elapsed, min 2.44 → +0.24d ≈ today
        assert_eq!(fc.optimistic, "2026-05-22");
        // median 4.01 - 2.2 = 1.81 → +2d
        assert_eq!(fc.likely, "2026-05-24");
    }
}
