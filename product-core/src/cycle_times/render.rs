//! Text + CSV renderers for cycle-time reports.

use super::model::{
    CycleTimeReport, CycleTimeRow, InProgressReport, InProgressRow, NaiveForecast, Stats,
};
use std::fmt::Write;

/// Render `CycleTimeReport` as a human-readable text table with summary.
pub fn render_text(report: &CycleTimeReport) -> String {
    let mut out = String::new();
    if report.features.is_empty() && report.summary.count == 0 {
        let _ = writeln!(&mut out, "No completed features with cycle-time data.");
        let _ = writeln!(&mut out);
        let _ = writeln!(
            &mut out,
            "At least 3 complete features (both started + complete tags) are required for summary statistics."
        );
        return out;
    }

    let _ = writeln!(
        &mut out,
        "{:<10} {:<12} {:<12} {:>10}",
        "Feature", "Started", "Completed", "Cycle time"
    );
    for row in &report.features {
        let _ = writeln!(
            &mut out,
            "{:<10} {:<12} {:<12} {:>8.1}d",
            row.id, row.started, row.completed, row.cycle_time_days
        );
    }
    let _ = writeln!(&mut out);

    let _ = writeln!(&mut out, "Summary:");
    let _ = writeln!(&mut out, "  count: {}", report.summary.count);
    if let Some(s) = &report.summary.recent_5 {
        let _ = writeln!(&mut out, "  Recent:  {}", render_stats(s));
    }
    if let Some(s) = &report.summary.all {
        let _ = writeln!(&mut out, "  All:     {}", render_stats(s));
    }
    if let Some(trend) = report.summary.trend {
        let sub = match trend {
            super::model::Trend::Accelerating => "(recent < historical)",
            super::model::Trend::Stable => "(recent \u{2248} historical)",
            super::model::Trend::Slowing => "(recent > historical)",
        };
        let _ = writeln!(&mut out, "  Trend:   {} {}", trend.as_str(), sub);
    }

    out
}

fn render_stats(s: &Stats) -> String {
    format!(
        "median {:.1}d  \u{00b7}  min {:.1}d  \u{00b7}  max {:.1}d",
        s.median, s.min, s.max
    )
}

/// Render `CycleTimeReport` as CSV with a fixed header (ADR-046 §10).
pub fn render_csv(report: &CycleTimeReport) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "feature_id,started,completed,cycle_time_days,phase");
    for row in &report.features {
        let _ = writeln!(
            &mut out,
            "{},{},{},{:.1},{}",
            row.id, row.started, row.completed, row.cycle_time_days, row.phase
        );
    }
    out
}

/// Render in-progress report as text.
pub fn render_in_progress_text(report: &InProgressReport) -> String {
    let mut out = String::new();
    if report.features.is_empty() {
        let _ = writeln!(&mut out, "No in-progress features with started tags.");
        return out;
    }
    let _ = writeln!(
        &mut out,
        "{:<10} {:<12} {:<14} {:>10}",
        "Feature", "Started", "Status", "Elapsed"
    );
    for row in &report.features {
        let _ = writeln!(
            &mut out,
            "{:<10} {:<12} {:<14} {:>8.1}d",
            row.id, row.started, row.status, row.elapsed_days
        );
    }
    if let Some(m) = report.reference_median {
        let _ = writeln!(&mut out);
        let _ = writeln!(
            &mut out,
            "Reference: median cycle time (recent 5) is {:.1}d",
            m
        );
    }
    out
}

/// Render a forecast block with rough-estimate disclaimer.
pub fn render_forecast_single(
    title: &str,
    header_line: &str,
    elapsed_days: f64,
    recent: &Stats,
    sample_size: usize,
    fc: &NaiveForecast,
) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "{}", title);
    let _ = writeln!(&mut out, "{}", header_line);
    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "Elapsed: {:.1}d", elapsed_days);
    let _ = writeln!(
        &mut out,
        "Recent {} complete features:  median {:.2}d  \u{00b7}  range {:.2} \u{2013} {:.2}d",
        sample_size, recent.median, recent.min, recent.max
    );
    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "Naive projection:");
    let _ = writeln!(&mut out, "  Likely completion:   {}", fc.likely);
    let _ = writeln!(&mut out, "  Optimistic:          {}", fc.optimistic);
    let _ = writeln!(&mut out, "  Pessimistic:         {}", fc.pessimistic);
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "This is a rough estimate based on {} recent features. It is not a probability forecast.",
        sample_size
    );
    out
}

/// Render a phase forecast block.
pub fn render_forecast_phase(
    phase: u32,
    remaining: &[String],
    recent: &Stats,
    sample_size: usize,
    fc: &NaiveForecast,
) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "Phase {} \u{2014} naive projection", phase);
    let _ = writeln!(
        &mut out,
        "Features remaining: {} ({})",
        remaining.len(),
        remaining.join(", ")
    );
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "Recent {} complete features:  median {:.2}d  \u{00b7}  range {:.2} \u{2013} {:.2}d",
        sample_size, recent.median, recent.min, recent.max
    );
    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "Naive projection:");
    let _ = writeln!(&mut out, "  Likely completion:   {}", fc.likely);
    let _ = writeln!(&mut out, "  Optimistic:          {}", fc.optimistic);
    let _ = writeln!(&mut out, "  Pessimistic:         {}", fc.pessimistic);
    let _ = writeln!(&mut out);
    let _ = writeln!(&mut out, "Assumes no parallelism and no dependency blocking.");
    let _ = writeln!(&mut out, "This is a rough estimate. It is not a probability forecast.");
    let _ = writeln!(&mut out);
    let _ = writeln!(
        &mut out,
        "For a more precise forecast, export cycle times: product cycle-times --format csv > cycle-times.csv"
    );
    out
}

/// Helper used by other renderers.
pub fn cycle_time_cell(row: &CycleTimeRow) -> String {
    format!("{:.1}d", row.cycle_time_days)
}

pub fn in_progress_cell(row: &InProgressRow, reference_median: Option<f64>) -> String {
    match reference_median {
        Some(m) => format!("elapsed {:.1}d  (recent median: {:.1}d)", row.elapsed_days, m),
        None => format!("elapsed {:.1}d", row.elapsed_days),
    }
}
