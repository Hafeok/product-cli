//! Resolution over the composition edges — every figure beside its population (CG-R-73).
//!
//! The coverage fraction is resolved / (resolved + unresolved). Its
//! denominator is a *subset* of the composition edges, so it is never printed
//! alone: every report carries `scored / composition_edges` beside it. The
//! disposition of the other states is stated, not implied by a column:
//! **partial** (composition chooses a factory or provider; the factory
//! chooses later, outside composition) is neither resolved nor unresolved
//! and is never divided; **boundary** (the library supplies it, CG-R-68) and
//! **registration-not-read** (a framework call the resolver cannot parse
//! supplies it, CG-R-75) are outside the denominator and listed by type;
//! **excluded** edges failed the criterion's role test.

use std::collections::BTreeMap;

use serde::Serialize;

use super::csharp_inventory::Index;
use super::csharp_roles::{RoleProxy, PROXIES, RETIRED_PROXIES};
use super::csharp_walk::{by_role, Closure, CompositionEdge, EdgeState, Injection};

/// One external type on the boundary, or supplied by an unparsed call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BoundaryRow {
    pub target: String,
    pub assembly: String,
    pub role: String,
    /// Composition edges landing on it.
    pub edges: usize,
    /// For `registration-not-read`: the call that registers it.
    pub call: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Resolution {
    /// Every composition edge, whatever its state.
    pub composition_edges: usize,
    /// resolved + unresolved: the denominator of the coverage fraction.
    pub scored: usize,
    pub resolved: usize,
    pub unresolved: usize,
    /// Held, never divided (see the module doc).
    pub partial: usize,
    pub boundary: usize,
    pub registration_not_read: usize,
    pub excluded: usize,
    /// Edges that ask for every registration of a type argument (P-6) — a
    /// distinct classification, counted inside the states above.
    pub collection_edges: usize,
    /// resolved / scored.
    pub coverage_percent: f64,
    /// scored / composition_edges — the first reading of the scored fraction (CG-R-90).
    pub population_percent: f64,
    /// resolved + unresolved + registration-not-read: every edge with a verdict.
    pub classified: usize,
    /// classified / composition_edges — the second reading (CG-R-90).
    pub classified_percent: f64,
    /// composition_edges − classified: partial + boundary + excluded — the
    /// edges that could silently reclassify a type between reachable and
    /// isolated: the error bound on that split (CG-R-89).
    pub unscored: usize,
    pub error_bound_percent: f64,
    /// resolved / (resolved + unresolved + registration-not-read): the rule
    /// in force from run 7 (CG-R-83) — an unread registration is the
    /// instrument's ignorance, not a boundary, and counts against coverage.
    pub coverage_with_not_read_percent: f64,
    pub by_role: BTreeMap<String, usize>,
    pub excluded_by_role: BTreeMap<String, usize>,
    pub by_reason: BTreeMap<String, usize>,
    /// registration-not-read edges by the call that registers the target.
    pub by_provider: BTreeMap<String, usize>,
    pub unresolved_edges: Vec<CompositionEdge>,
    /// Every composition edge with its role and state (JSON).
    pub edges: Vec<CompositionEdge>,
    /// The used library surface, by external type, most edges first.
    pub boundary_surface: Vec<BoundaryRow>,
    /// Types a reached, unparsed framework call registers, with the call.
    pub registration_not_read_surface: Vec<BoundaryRow>,
    pub proxies: &'static [RoleProxy],
    pub retired_proxies: &'static [RoleProxy],
}

fn pct(num: usize, den: usize) -> f64 {
    if den == 0 { 100.0 } else { 100.0 * num as f64 / den as f64 }
}

pub fn resolution_of(ix: &Index<'_>, c: &Closure<'_>) -> Resolution {
    let resolved = c.count(|s| *s == EdgeState::Resolved);
    let partial = c.count(|s| *s == EdgeState::Partial);
    let unresolved = c.count(|s| matches!(s, EdgeState::Unresolved(_)));
    let boundary = c.count(|s| *s == EdgeState::Boundary);
    let registration_not_read = c.count(|s| matches!(s, EdgeState::RegistrationNotRead(_)));
    let (by_reason, excluded, by_provider) = tallies(c);
    Resolution {
        composition_edges: c.edges.len(),
        scored: resolved + unresolved,
        resolved,
        unresolved,
        partial,
        boundary,
        registration_not_read,
        excluded: excluded.values().sum(),
        collection_edges: c.edges.iter().filter(|e| e.injection == Injection::Collection).count(),
        coverage_percent: pct(resolved, resolved + unresolved),
        population_percent: pct(resolved + unresolved, c.edges.len()),
        classified: resolved + unresolved + registration_not_read,
        classified_percent: pct(resolved + unresolved + registration_not_read, c.edges.len()),
        unscored: c.edges.len() - (resolved + unresolved + registration_not_read),
        error_bound_percent: if c.edges.is_empty() { 0.0 } else { 100.0 * (c.edges.len() - (resolved + unresolved + registration_not_read)) as f64 / c.edges.len() as f64 },
        coverage_with_not_read_percent: pct(resolved, resolved + unresolved + registration_not_read),
        by_role: by_role(c),
        excluded_by_role: excluded,
        by_reason,
        by_provider,
        unresolved_edges: c.edges.iter().filter(|e| matches!(e.state, EdgeState::Unresolved(_))).cloned().collect(),
        edges: c.edges.iter().cloned().collect(),
        boundary_surface: surface(ix, c, |s| *s == EdgeState::Boundary),
        registration_not_read_surface: surface(ix, c, |s| matches!(s, EdgeState::RegistrationNotRead(_))),
        proxies: PROXIES,
        retired_proxies: RETIRED_PROXIES,
    }
}

type Tally = BTreeMap<String, usize>;

/// Unresolved edges by reason, excluded edges by role, not-read edges by call.
fn tallies(c: &Closure<'_>) -> (Tally, Tally, Tally) {
    let (mut by_reason, mut excluded, mut by_provider) = (Tally::new(), Tally::new(), Tally::new());
    for e in &c.edges {
        match e.state {
            EdgeState::Unresolved(r) => *by_reason.entry(r.label().to_string()).or_insert(0) += 1,
            EdgeState::Excluded(role) => *excluded.entry(role.label().to_string()).or_insert(0) += 1,
            EdgeState::RegistrationNotRead(call) => *by_provider.entry(call.to_string()).or_insert(0) += 1,
            _ => {}
        }
    }
    (by_reason, excluded, by_provider)
}

fn surface(ix: &Index<'_>, c: &Closure<'_>, pred: impl Fn(&EdgeState) -> bool) -> Vec<BoundaryRow> {
    let mut rows: BTreeMap<&str, BoundaryRow> = BTreeMap::new();
    for e in c.edges.iter().filter(|e| pred(&e.state)) {
        let assembly = ix.external.get(e.target.as_str()).map(|x| x.assembly.clone()).unwrap_or_default();
        let call = match e.state {
            EdgeState::RegistrationNotRead(call) => Some(call.to_string()),
            _ => None,
        };
        rows.entry(e.target.as_str()).or_insert_with(|| BoundaryRow { target: e.target.clone(), assembly, role: e.role.label().to_string(), edges: 0, call }).edges += 1;
    }
    let mut out: Vec<BoundaryRow> = rows.into_values().collect();
    out.sort_by(|a, b| b.edges.cmp(&a.edges).then(a.target.cmp(&b.target)));
    out
}
