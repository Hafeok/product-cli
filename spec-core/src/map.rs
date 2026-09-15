//! The join between acts and entry points, where the disagreements are output.
//!
//! One act to one entry point is the easy case and says nothing. What is worth
//! reading is where the two shapes fail to line up, because that is where the
//! transport boundary and the act boundary disagree — and the act wins.

use serde::Serialize;

use crate::gate::SpecStore;

/// What the join found.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "finding")]
pub enum MapFinding {
    /// Several entry points, one act: transport boundaries finer than act
    /// boundaries. The restructuring is justified by the act.
    Merge { act: String, entry_points: Vec<String> },
    /// One entry point, several acts: an entry point spanning an act boundary.
    Split { entry_point: String, acts: Vec<String> },
    /// Drift, or an act nobody has named.
    UnmappedEntryPoint { entry_point: String, transport: String },
    /// Specified and unrealised.
    UnmappedAct { act: String },
}

impl MapFinding {
    /// A one-line rendering, with the restructuring the act justifies.
    pub fn line(&self) -> String {
        match self {
            Self::Merge { act, entry_points } => format!(
                "merge    {act} ← {} entry points: {}",
                entry_points.len(),
                entry_points.join(", ")
            ),
            Self::Split { entry_point, acts } => {
                format!("split    {entry_point} → {}", acts.join(", "))
            }
            Self::UnmappedEntryPoint { entry_point, transport } => {
                format!("unmapped {entry_point}  ({transport})")
            }
            Self::UnmappedAct { act } => format!("unrealised {act}"),
        }
    }
}

/// Run the join.
///
/// Ordered merge, split, unmapped entry point, unmapped act — the order a
/// reviewer wants, not the order the data arrives in.
pub fn join(store: &SpecStore) -> Vec<MapFinding> {
    let mut findings = Vec::new();
    findings.extend(merges(store));
    findings.extend(splits(store));
    findings.extend(unmapped_entry_points(store));
    findings.extend(unmapped_acts(store));
    findings
}

fn merges(store: &SpecStore) -> Vec<MapFinding> {
    store
        .acts
        .iter()
        .filter(|a| a.realised_at.len() > 1)
        .map(|a| MapFinding::Merge {
            act: a.id.clone(),
            entry_points: sorted(&a.realised_at),
        })
        .collect()
}

fn splits(store: &SpecStore) -> Vec<MapFinding> {
    let Some(inventory) = &store.inventory else {
        return Vec::new();
    };
    inventory
        .entry_points
        .iter()
        .filter_map(|entry| {
            let acts = store.acts_at(&entry.id);
            (acts.len() > 1).then(|| MapFinding::Split {
                entry_point: entry.id.clone(),
                acts: sorted(&acts.iter().map(|a| a.id.clone()).collect::<Vec<_>>()),
            })
        })
        .collect()
}

fn unmapped_entry_points(store: &SpecStore) -> Vec<MapFinding> {
    let Some(inventory) = &store.inventory else {
        return Vec::new();
    };
    inventory
        .entry_points
        .iter()
        .filter(|entry| store.acts_at(&entry.id).is_empty())
        .map(|entry| MapFinding::UnmappedEntryPoint {
            entry_point: entry.id.clone(),
            transport: entry.transport.clone(),
        })
        .collect()
}

/// An act claiming no entry point, or claiming one the scan no longer sees.
///
/// Both are "specified and unrealised", and both are reports rather than
/// verdicts: a codebase with unspecified regions is unspecified, not
/// non-conformant, and the same courtesy is owed the other direction.
fn unmapped_acts(store: &SpecStore) -> Vec<MapFinding> {
    store
        .acts
        .iter()
        .filter(|act| match &store.inventory {
            None => act.realised_at.is_empty(),
            Some(inventory) => act
                .realised_at
                .iter()
                .all(|e| inventory.entry_point(e).is_none()),
        })
        .map(|act| MapFinding::UnmappedAct { act: act.id.clone() })
        .collect()
}

fn sorted(items: &[String]) -> Vec<String> {
    let mut out = items.to_vec();
    out.sort();
    out
}

#[path = "map_tests.rs"]
#[cfg(test)]
mod tests;
