//! Planning annotations — due-date advisory warnings (W028, W029).
//!
//! FT-053 / ADR-045. Both warnings are advisory (W-class, exit 2). They
//! never block verification or phase-gate evaluation.

use super::model::KnowledgeGraph;
use crate::config::PlanningConfig;
use crate::error::{CheckResult, Diagnostic};
use crate::types::FeatureStatus;
use chrono::NaiveDate;

/// Evaluate due-date warnings against a reference `today`. Pure — the caller
/// supplies the clock so tests can pin time.
pub fn check_due_dates(
    graph: &KnowledgeGraph,
    planning: &PlanningConfig,
    today: NaiveDate,
    result: &mut CheckResult,
) {
    let mut features: Vec<_> = graph.features.values().collect();
    features.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    for f in features {
        // Only features with a due_date and not complete emit warnings.
        let due = match f.front.due_date {
            Some(d) => d,
            None => continue,
        };
        if f.front.status == FeatureStatus::Complete {
            continue;
        }

        if due < today {
            let days_late = (today - due).num_days();
            let detail = format!(
                "{} due {} \u{2014} {} day(s) overdue (status: {})",
                f.front.id, due, days_late, f.front.status
            );
            result.warnings.push(
                Diagnostic::warning("W028", "due-date has passed")
                    .with_file(f.path.clone())
                    .with_detail(&detail)
                    .with_hint("review the commitment — advisory only, no verify failure"),
            );
        } else if planning.due_date_warning_days > 0 {
            let window = planning.due_date_warning_days as i64;
            let days_out = (due - today).num_days();
            if days_out <= window {
                let detail = format!(
                    "{} due {} in {} day(s) (status: {})",
                    f.front.id, due, days_out, f.front.status
                );
                result.warnings.push(
                    Diagnostic::warning("W029", "due-date approaching")
                        .with_file(f.path.clone())
                        .with_detail(&detail)
                        .with_hint(
                            "advisory only — disable with [planning].due-date-warning-days = 0",
                        ),
                );
            }
        }
    }
}
