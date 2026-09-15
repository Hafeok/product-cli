//! The three-region delta over a ratified candidate set (Gate 1b, Gate B).
//!
//! Loads a ratification worksheet — Emil's decisions over the candidate set
//! — builds the act vocabulary from its accepted rows (positions read through
//! proxy P-EP-4 on each entry point's path), then classifies every undeclared
//! entry point as *declarable*, *unstructured* or *no facts under the proxy*
//! by CG-R-52's separator applied per type, and reports the reachable /
//! unresolved / isolated split over undeclared production types with its
//! error bound (CG-R-89). Every figure is transport-derived (CG-R-105) and
//! carries the worksheet's grade (CG-R-115). The criterion in full:
//! `meta/sessions/2026-09-15-gate1b-candidates/gateB-criterion.md`.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::csharp_candidates::Candidate;
use super::csharp_candidates_path::facts_of;
use super::csharp_candidates_report::candidates;
use super::csharp_inventory::{Index, Inventory};
use super::csharp_regions_ratios::{ratios, Ratios};
use super::csharp_walk::Closure;

/// The worksheet as filled; fields the delta does not read are ignored.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Ratification {
    #[serde(default)]
    pub vocabulary_grade: String,
    #[serde(default)]
    pub ratifier: String,
    #[serde(default)]
    pub candidates: Vec<RatifiedRow>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RatifiedRow {
    pub id: String,
    #[serde(default)]
    pub decision: Option<String>,
    #[serde(default)]
    pub act: Option<String>,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub reject_reason: Option<String>,
}

/// One act of the ratified vocabulary, with positions by proxy P-EP-4.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Act {
    pub name: String,
    pub entry_points: Vec<String>,
    pub channels: Vec<String>,
    pub reads: Vec<String>,
    pub writes: Vec<String>,
}

impl Act {
    fn facts(&self) -> BTreeSet<&str> {
        self.reads.iter().chain(self.writes.iter()).map(String::as_str).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    Declared,
    Declarable,
    Unstructured,
    /// P-EP-4 reads nothing on the path: its own row, never folded (CG-R-62).
    NoFactsUnderProxy,
}

impl Region {
    pub fn label(self) -> &'static str {
        match self {
            Region::Declared => "declared",
            Region::Declarable => "declarable",
            Region::Unstructured => "unstructured",
            Region::NoFactsUnderProxy => "no-facts-under-proxy",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntryRow {
    pub id: String,
    pub kind: String,
    pub project: String,
    /// accept | reject | defer | default (CG-R-115) | not in ratification
    pub disposition: String,
    pub region: Region,
    /// Declared: the act. Declarable: the covering acts.
    pub acts: Vec<String>,
    pub facts: Vec<String>,
    pub spanning_types: Vec<String>,
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanningType {
    pub type_id: String,
    pub facts: Vec<String>,
    pub acts_touching: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TwelveOne {
    pub reachable_percent: f64,
    pub error_bound_percent: f64,
    pub threshold_percent: f64,
    pub distance_percent: f64,
    pub bound_smaller_than_distance: bool,
    pub statement: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegionsReport {
    pub grade: String,
    pub labels: Vec<&'static str>,
    pub acts: Vec<Act>,
    pub entries: Vec<EntryRow>,
    pub by_region: BTreeMap<String, usize>,
    /// kind → region → count; never a flat list of symbols.
    pub by_kind_region: BTreeMap<String, BTreeMap<String, usize>>,
    pub by_project_region: BTreeMap<String, BTreeMap<String, usize>>,
    pub spanning_types: Vec<SpanningType>,
    /// spanning types / types with facts — the separator's incidence on this solution.
    pub separator_incidence: String,
    pub ratios: Ratios,
    pub twelve_one: TwelveOne,
    /// Worksheet rows with no derived candidate (the hand-supplied ones).
    pub rows_without_path: Vec<String>,
}

pub const LABELS: &[&str] = &[
    "transport-derived (CG-R-105): the vocabulary is entry points ratified, not domain modelling",
    "stand-in (CG-R-115): the ratifier holds no intent for this codebase; every figure carries the grade",
    "positions by proxy P-EP-4 (incidence unmeasured): writes are a superset — a possible write counts as a write",
    "separator: CG-R-52's proxy per type — a type whose P-EP-4 facts no single act covers is spanning",
    "the registration table is unvalidated (CG-R-101 outstanding): registration-not-read verdicts on the paths rest on it",
    "L-EP-1 … L-EP-4 as the candidate derivation prints them; F-EP-1 does not touch composition edges",
];

/// Facts P-EP-4 reads on one type alone.
fn type_facts<'a>(ix: &Index<'a>, type_id: &'a str) -> BTreeSet<String> {
    let cl = Closure { types: BTreeSet::from([type_id]), members: ix.members_of.get(type_id).into_iter().flatten().copied().collect(), ..Default::default() };
    let f = facts_of(ix, &cl);
    f.read.into_iter().chain(f.written).chain(f.possibly_written).chain(f.dbset_touched).filter(|x| x.starts_with("T:")).collect()
}

/// The acts, from the accepted rows.
fn acts_of(rows: &BTreeMap<&str, &RatifiedRow>, cands: &[Candidate]) -> Vec<Act> {
    let mut acts: BTreeMap<String, Act> = BTreeMap::new();
    for c in cands {
        let Some(row) = rows.get(c.id.as_str()).filter(|r| r.decision.as_deref() == Some("accept")) else { continue };
        let Some(name) = row.act.clone() else { continue };
        let a = acts.entry(name.clone()).or_insert_with(|| Act { name, ..Default::default() });
        a.entry_points.push(c.id.clone());
        a.channels.extend(row.channel.clone());
        a.reads.extend(c.facts.read.iter().chain(&c.facts.dbset_touched).cloned());
        a.writes.extend(c.facts.written.iter().chain(&c.facts.possibly_written).filter(|f| f.starts_with("T:")).cloned());
    }
    for a in acts.values_mut() {
        for v in [&mut a.channels, &mut a.reads, &mut a.writes] {
            v.sort();
            v.dedup();
        }
    }
    acts.into_values().collect()
}

fn disposition(row: Option<&&RatifiedRow>) -> String {
    match row {
        None => "not in ratification".to_string(),
        Some(r) => r.decision.clone().unwrap_or_else(|| "default (CG-R-115)".to_string()),
    }
}

/// Classify one undeclared entry point.
fn classify(c: &Candidate, acts: &[Act], tf: &BTreeMap<&str, BTreeSet<String>>, spanning: &BTreeSet<&str>) -> (Region, Vec<String>, Vec<String>, Vec<String>, String) {
    let facts: BTreeSet<String> = c.path_types.iter().flat_map(|t| tf.get(t.as_str()).into_iter().flatten().cloned()).collect();
    if facts.is_empty() {
        return (Region::NoFactsUnderProxy, vec![], vec![], vec![], "P-EP-4 reads no fact on the path".to_string());
    }
    let on_path: Vec<String> = c.path_types.iter().filter(|t| spanning.contains(t.as_str())).cloned().collect();
    let covering: Vec<String> = acts.iter().filter(|a| facts.iter().all(|f| a.facts().contains(f.as_str()))).map(|a| a.name.clone()).collect();
    let facts_v: Vec<String> = facts.into_iter().collect();
    if on_path.is_empty() && !covering.is_empty() {
        return (Region::Declarable, covering, facts_v, on_path, "a clean path: one act's positions cover every fact on it".to_string());
    }
    let why = match (on_path.is_empty(), covering.is_empty()) {
        (false, false) => "a spanning type lies on the path",
        (true, true) => "no single act's positions cover the path's facts",
        _ => "a spanning type lies on the path, and no single act covers the path's facts",
    };
    (Region::Unstructured, covering, facts_v, on_path, why.to_string())
}

fn entry_row(c: &Candidate, row: Option<&&RatifiedRow>, acts: &[Act], tf: &BTreeMap<&str, BTreeSet<String>>, spanning: &BTreeSet<&str>) -> EntryRow {
    let disposition = disposition(row);
    let declared = row.and_then(|r| r.act.clone()).filter(|_| disposition == "accept");
    let (region, acts, facts, spanning_types, why) = match declared {
        Some(a) => (Region::Declared, vec![a], c.facts.read.iter().chain(&c.facts.written).cloned().collect(), vec![], "accepted at ratification".to_string()),
        None => classify(c, acts, tf, spanning),
    };
    EntryRow { id: c.id.clone(), kind: c.kind.label().to_string(), project: c.project.clone(), disposition, region, acts, facts, spanning_types, why }
}

/// Run the delta over the ratified set.
pub fn regions(inv: &Inventory, rat: &Ratification) -> RegionsReport {
    let ix = inv.index();
    let cr = candidates(inv, None);
    let rows: BTreeMap<&str, &RatifiedRow> = rat.candidates.iter().map(|r| (r.id.as_str(), r)).collect();
    let acts = acts_of(&rows, &cr.candidates);
    let path_types: BTreeSet<&str> = cr.candidates.iter().flat_map(|c| c.path_types.iter().map(String::as_str)).collect();
    let tf: BTreeMap<&str, BTreeSet<String>> = path_types.iter().map(|t| (*t, type_facts(&ix, t))).collect();
    let with_facts = tf.values().filter(|f| !f.is_empty()).count();
    let spanning_types: Vec<SpanningType> = tf
        .iter()
        .filter(|(_, f)| !f.is_empty() && !acts.iter().any(|a| f.iter().all(|x| a.facts().contains(x.as_str()))))
        .map(|(t, f)| SpanningType { type_id: t.to_string(), facts: f.iter().cloned().collect(), acts_touching: acts.iter().filter(|a| f.iter().any(|x| a.facts().contains(x.as_str()))).map(|a| a.name.clone()).collect() })
        .collect();
    let spanning: BTreeSet<&str> = spanning_types.iter().map(|s| s.type_id.as_str()).collect();
    let entries: Vec<EntryRow> = cr.candidates.iter().map(|c| entry_row(c, rows.get(c.id.as_str()), &acts, &tf, &spanning)).collect();
    let (mut by_region, mut by_kind_region, mut by_project_region) = (BTreeMap::new(), BTreeMap::<String, BTreeMap<String, usize>>::new(), BTreeMap::<String, BTreeMap<String, usize>>::new());
    for e in &entries {
        *by_region.entry(e.region.label().to_string()).or_insert(0) += 1;
        *by_kind_region.entry(e.kind.clone()).or_default().entry(e.region.label().to_string()).or_insert(0) += 1;
        *by_project_region.entry(e.project.clone()).or_default().entry(e.region.label().to_string()).or_insert(0) += 1;
    }
    let ratios = ratios(inv, &ix, &cr.candidates, &rows);
    let twelve_one = twelve_one(&ratios);
    RegionsReport {
        grade: if rat.vocabulary_grade.is_empty() { "ungraded — the worksheet carries no grade".to_string() } else { rat.vocabulary_grade.clone() },
        labels: LABELS.to_vec(),
        acts,
        entries,
        by_region,
        by_kind_region,
        by_project_region,
        separator_incidence: format!("{} spanning of {} path types with facts under P-EP-4 ({} path types in all)", spanning_types.len(), with_facts, tf.len()),
        spanning_types,
        ratios,
        twelve_one,
        rows_without_path: rat.candidates.iter().filter(|r| !cr.candidates.iter().any(|c| c.id == r.id)).map(|r| r.id.clone()).collect(),
    }
}

fn twelve_one(r: &Ratios) -> TwelveOne {
    let distance = (90.0 - r.reachable_percent).abs();
    let smaller = r.error_bound_percent < distance;
    let statement = format!(
        "{:.1}% of undeclared production types are reachable from the accepted entry points, with an error bound of {:.1} points; the distance to §12.1's ~90% is {:.1} points, so the split {} (CG-R-89: no fire/clear form; the bound is reported beside the split)",
        r.reachable_percent,
        r.error_bound_percent,
        distance,
        if smaller { "discriminates" } else { "does not discriminate — the bound exceeds the difference it would reveal" }
    );
    TwelveOne { reachable_percent: r.reachable_percent, error_bound_percent: r.error_bound_percent, threshold_percent: 90.0, distance_percent: distance, bound_smaller_than_distance: smaller, statement }
}

#[cfg(test)]
#[path = "csharp_regions_tests.rs"]
mod tests;
