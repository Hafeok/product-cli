//! Cycle-time visibility slice — ADR-046 historical descriptive statistics.
//!
//! Pure builders live in `compute`; rendering in `render`; shared types in
//! `model`. No I/O; git tag reads happen in the command adapter.

pub mod compute;
pub mod model;
pub mod render;

pub use compute::{
    build_in_progress_report, build_report, classify_trend, elapsed_days, median, parse_instant,
    project_naive_phase, project_naive_single, round1, stats_of, TagTimestamps,
};
pub use model::{
    CycleTimeReport, CycleTimeRow, InProgressReport, InProgressRow, NaiveForecast, Stats, Summary,
    Trend,
};
pub use render::{
    cycle_time_cell, in_progress_cell, render_csv, render_forecast_phase,
    render_forecast_single, render_in_progress_text, render_text,
};

#[cfg(test)]
mod tests;
