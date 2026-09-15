//! The reachable / unresolved / isolated split over undeclared production types (Gate 1b, Gate B).
//!
//! One walk from the union of the accepted entry points' roots, through the
//! union of the hosts' registration sites, O-17 off — the same walk the
//! candidate paths use — with its unscored fraction as the error bound on
//! the split (CG-R-89). Unresolved is never folded into isolated (CG-R-62).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::csharp_candidates::Candidate;
use super::csharp_candidates_path::hosts;
use super::csharp_di::Resolver;
use super::csharp_inventory::{Index, Inventory};
use super::csharp_reach::sets;
use super::csharp_regions::RatifiedRow;
use super::csharp_walk::{closure_from, EdgeState};

#[derive(Debug, Clone, Default, Serialize)]
pub struct Ratios {
    pub accepted_entry_points: usize,
    pub roots: usize,
    pub undeclared_types: usize,
    pub reachable_undeclared: usize,
    pub unresolved_undeclared: usize,
    pub isolated_undeclared: usize,
    pub reachable_percent: f64,
    pub isolated_percent: f64,
    pub composition_edges: usize,
    pub scored: usize,
    pub unscored: usize,
    pub error_bound_percent: f64,
    /// (namespace, reachable, unresolved, isolated)
    pub by_namespace: Vec<(String, usize, usize, usize)>,
    /// (project, reachable, unresolved, isolated)
    pub by_project: Vec<(String, usize, usize, usize)>,
}

/// (reachable, unresolved, isolated) per group.
type Tally<'a> = BTreeMap<&'a str, (usize, usize, usize)>;

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 { 0.0 } else { 100.0 * n as f64 / d as f64 }
}

/// The split, from the accepted rows' candidates.
pub fn ratios<'a>(inv: &'a Inventory, ix: &Index<'a>, cands: &[Candidate], rows: &BTreeMap<&str, &RatifiedRow>) -> Ratios {
    let resolver = Resolver::build(inv);
    let accepted: Vec<&Candidate> = cands.iter().filter(|c| rows.get(c.id.as_str()).is_some_and(|r| r.decision.as_deref() == Some("accept"))).collect();
    let roots: BTreeSet<&'a str> = accepted
        .iter()
        .flat_map(|c| c.roots.iter())
        .filter_map(|r| ix.members.get_key_value(r.as_str()).map(|(k, _)| *k).or_else(|| ix.types.get_key_value(r.as_str()).map(|(k, _)| *k)))
        .collect();
    let h = hosts(inv, ix, &resolver);
    let base: BTreeSet<&'a str> = h.sites.values().flatten().copied().collect();
    let cl = closure_from(ix, &resolver, &roots, &base, false);
    let s = sets(ix, &cl);
    let own: BTreeSet<&str> = accepted.iter().map(|c| c.type_id.as_str()).collect();
    let mut r = Ratios { accepted_entry_points: accepted.len(), roots: roots.len(), ..Default::default() };
    let (mut by_ns, mut by_pr): (Tally<'_>, Tally<'_>) = (BTreeMap::new(), BTreeMap::new());
    for t in inv.types.iter().filter(|t| !ix.is_test(&t.id) && !own.contains(t.id.as_str())) {
        r.undeclared_types += 1;
        let (ns, pr) = (by_ns.entry(t.namespace.as_str()).or_default(), by_pr.entry(t.project.as_str()).or_default());
        if cl.types.contains(t.id.as_str()) {
            r.reachable_undeclared += 1;
            ns.0 += 1;
            pr.0 += 1;
        } else if s.unresolved.contains(t.id.as_str()) || s.partial.contains(t.id.as_str()) {
            r.unresolved_undeclared += 1;
            ns.1 += 1;
            pr.1 += 1;
        } else {
            r.isolated_undeclared += 1;
            ns.2 += 1;
            pr.2 += 1;
        }
    }
    r.composition_edges = cl.edges.len();
    r.scored = cl.count(|e| matches!(e, EdgeState::Resolved | EdgeState::Unresolved(_) | EdgeState::RegistrationNotRead(_)));
    r.unscored = r.composition_edges - r.scored;
    r.reachable_percent = pct(r.reachable_undeclared, r.undeclared_types);
    r.isolated_percent = pct(r.isolated_undeclared, r.undeclared_types);
    r.error_bound_percent = pct(r.unscored, r.composition_edges);
    r.by_namespace = by_ns.into_iter().map(|(k, (a, b, c))| (if k.is_empty() { "(global)".to_string() } else { k.to_string() }, a, b, c)).collect();
    r.by_project = by_pr.into_iter().map(|(k, (a, b, c))| (k.to_string(), a, b, c)).collect();
    r
}
