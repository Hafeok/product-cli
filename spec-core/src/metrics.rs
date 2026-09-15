//! The one computed set every report surface selects from.
//!
//! Computed once, in one place, so `check`, `check --ci`, `check --assessment`
//! and a policy evaluation cannot disagree about what a figure is. Three
//! report modes reading three computations is how a gate stops being
//! trustworthy rather than merely wrong.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::gate::SpecStore;

/// A figure, with the denominator that makes it readable.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Metric {
    /// The count the policy compares against.
    pub count: u64,
    /// The population, where the figure is a proportion of something.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub of: Option<u64>,
}

impl Metric {
    /// A bare count.
    pub fn count(n: usize) -> Self {
        Self { count: n as u64, of: None }
    }

    /// A count out of a population.
    pub fn out_of(n: usize, total: usize) -> Self {
        Self { count: n as u64, of: Some(total as u64) }
    }

    /// The proportion as a percentage, or `None` where there is nothing to
    /// cover.
    ///
    /// `None` rather than 100%: an empty codebase is not fully specified, and
    /// a figure that says otherwise is the kind of flattering default that
    /// makes a metric useless.
    pub fn percent(&self) -> Option<f64> {
        match self.of {
            Some(total) if total > 0 => {
                #[allow(clippy::cast_precision_loss)]
                Some((self.count as f64 / total as f64) * 100.0)
            }
            _ => None,
        }
    }

    /// How the figure reads on one line.
    pub fn render(&self) -> String {
        match (self.of, self.percent()) {
            (Some(total), Some(pct)) => format!("{} / {total} ({pct:.0}%)", self.count),
            (Some(total), None) => format!("{} / {total}", self.count),
            _ => self.count.to_string(),
        }
    }
}

/// Every metric the flow reports, by name.
///
/// The set is open by design — which metrics exist will grow. Each addition is
/// a place a verdict could be attached without a basis, and the basis
/// requirement is the only brake on that.
pub fn compute(store: &SpecStore) -> BTreeMap<String, Metric> {
    let entry_points = store.inventory.as_ref().map_or(0, |i| i.entry_points.len());
    let candidates: Vec<_> = store
        .inventory
        .as_ref()
        .map(|i| i.candidates.iter().collect())
        .unwrap_or_default();
    let unreviewed = candidates.iter().filter(|c| store.is_unreviewed(&c.id)).count();
    let mapped = store
        .inventory
        .as_ref()
        .map_or(0, |i| i.entry_points.iter().filter(|e| !store.acts_at(&e.id).is_empty()).count());
    let unmapped = entry_points.saturating_sub(mapped);
    let unrealised = store
        .acts
        .iter()
        .filter(|act| match &store.inventory {
            None => act.realised_at.is_empty(),
            Some(inventory) => {
                act.realised_at.iter().all(|e| inventory.entry_point(e).is_none())
            }
        })
        .count();

    BTreeMap::from([
        ("entry_points".into(), Metric::count(entry_points)),
        ("candidates".into(), Metric::count(candidates.len())),
        ("candidates_unreviewed".into(), Metric::out_of(unreviewed, candidates.len())),
        ("candidates_refused".into(), Metric::count(store.rejections.len())),
        ("acts_ratified".into(), Metric::count(store.acts.len())),
        ("acts_unrealised".into(), Metric::out_of(unrealised, store.acts.len())),
        ("entry_points_mapped".into(), Metric::out_of(mapped, entry_points)),
        ("unmapped_entry_points".into(), Metric::out_of(unmapped, entry_points)),
        ("mapping_coverage".into(), Metric::out_of(mapped, entry_points)),
        ("records_open".into(), Metric::count(store.records.iter().filter(|r| r.is_open()).count())),
        ("records".into(), Metric::count(store.records.len())),
    ])
}

#[path = "metrics_tests.rs"]
#[cfg(test)]
mod tests;
