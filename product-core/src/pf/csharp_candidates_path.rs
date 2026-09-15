//! The path from a candidate — its walk through the host's registration sites (Gate 1b §3).
//!
//! A candidate walks from its handler(s) plus constructors (or its type)
//! with the hosting project's own closure as registration sites and O-17
//! off, so the path is what the candidate's edges reach. Facts along the
//! path are read through proxy P-EP-4 (repository type arguments, `DbSet`
//! accesses); client-identity checks are the identity members touched by
//! symbol id. Overlaps between paths are the merge signal (CG-R-106) —
//! reported, never decided.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::csharp_candidates::{chain_reaches, constructors, Candidate};
use super::csharp_candidates_seed::short;
use super::csharp_di::Resolver;
use super::csharp_inventory::{Index, Inventory};
use super::csharp_walk::{closure, closure_from, Closure, EdgeState};

pub const READ_REPOSITORY_BASE: &str = "T:Ardalis.Specification.IReadRepositoryBase`1";
pub const REPOSITORY_BASE: &str = "T:Ardalis.Specification.IRepositoryBase`1";
pub const DBSET: &str = "T:Microsoft.EntityFrameworkCore.DbSet`1";
/// Write members of `IRepositoryBase<T>`, by member-id prefix.
pub const WRITE_MEMBERS: &[&str] = &[
    "M:Ardalis.Specification.IRepositoryBase`1.AddAsync(",
    "M:Ardalis.Specification.IRepositoryBase`1.AddRangeAsync(",
    "M:Ardalis.Specification.IRepositoryBase`1.UpdateAsync(",
    "M:Ardalis.Specification.IRepositoryBase`1.UpdateRangeAsync(",
    "M:Ardalis.Specification.IRepositoryBase`1.DeleteAsync(",
    "M:Ardalis.Specification.IRepositoryBase`1.DeleteRangeAsync(",
];
/// Client-identity members, by symbol id.
pub const IDENTITY_MEMBERS: &[&str] = &[
    "P:Microsoft.AspNetCore.Mvc.ControllerBase.User",
    "P:Microsoft.AspNetCore.Mvc.RazorPages.PageModel.User",
    "P:Microsoft.AspNetCore.Mvc.ViewComponent.User",
    "P:Microsoft.AspNetCore.Http.HttpContext.User",
    "P:System.Security.Principal.IIdentity.Name",
    "P:System.Security.Principal.IIdentity.IsAuthenticated",
];

pub const FACTS_PROXY: &str = "P-EP-4: read = type arguments of repository-typed constructor parameters (chain reaches IReadRepositoryBase`1 or IRepositoryBase`1) on path types, plus DbSet<T> properties accessed on the path; written = the type argument of a write-capable repository parameter on a path type that calls a write member of IRepositoryBase`1 by member id; more than one write-capable parameter → all listed as possibly written (the call site's type argument is not in the inventory). Original predicate: the facts an act reads and writes. Divergence: an entity type is where state lives, not a state change; DbSet accesses do not distinguish read from write. Incidence: unmeasured — first use";

/// Facts along the path, through proxy P-EP-4.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Facts {
    pub read: Vec<String>,
    pub written: Vec<String>,
    pub possibly_written: Vec<String>,
    pub dbset_touched: Vec<String>,
    pub proxy: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct Host {
    pub project: String,
    pub entry_point: String,
    /// Members in the host's own closure — the registration sites candidates resolve through.
    pub sites: usize,
}

/// One host per production project with an entry point.
#[derive(Default)]
pub struct Hosts<'a> {
    pub rows: Vec<Host>,
    pub sites: BTreeMap<String, BTreeSet<&'a str>>,
}

pub fn hosts<'a>(inv: &'a Inventory, ix: &Index<'a>, resolver: &Resolver) -> Hosts<'a> {
    let mut h = Hosts::default();
    for m in inv.members.iter().filter(|m| m.is_entry_point) {
        let Some(t) = ix.types.get(m.declaring_type.as_str()) else { continue };
        if ix.is_test(&t.id) || h.sites.contains_key(t.project.as_str()) {
            continue;
        }
        let start = BTreeSet::from([m.id.as_str()]);
        let c = closure(ix, resolver, &start);
        h.rows.push(Host { project: t.project.clone(), entry_point: m.id.clone(), sites: c.members.len() });
        h.sites.insert(t.project.clone(), c.members);
    }
    h
}

fn state_label(s: &EdgeState) -> &'static str {
    match s {
        EdgeState::Resolved => "resolved",
        EdgeState::Partial => "partial",
        EdgeState::Boundary => "boundary",
        EdgeState::RegistrationNotRead(_) => "registration-not-read",
        EdgeState::Unresolved(_) => "unresolved",
        EdgeState::Excluded(_) => "excluded",
    }
}

/// Walk the candidate; fill its path, edge states, facts and identity checks.
pub fn fill_path<'a>(ix: &Index<'a>, resolver: &Resolver, hosts: &Hosts<'a>, c: &mut Candidate) {
    let start: BTreeSet<&'a str> = c
        .roots
        .iter()
        .filter_map(|r| ix.members.get_key_value(r.as_str()).map(|(k, _)| *k).or_else(|| ix.types.get_key_value(r.as_str()).map(|(k, _)| *k)))
        .collect();
    let empty = BTreeSet::new();
    let base = hosts.sites.get(&c.project).unwrap_or(&empty);
    let cl = closure_from(ix, resolver, &start, base, false);
    c.path_types = cl.types.iter().filter(|t| ix.types.contains_key(*t) && !ix.is_test(t)).map(|t| t.to_string()).collect();
    for e in &cl.edges {
        *c.path_edges.entry(state_label(&e.state).to_string()).or_insert(0) += 1;
    }
    let scored: usize = ["resolved", "unresolved", "registration-not-read"].iter().map(|k| c.path_edges.get(*k).copied().unwrap_or(0)).sum();
    c.unscored = cl.edges.len() - scored;
    c.facts = facts_of(ix, &cl);
    c.observed.identity_checks = identity_checks(ix, &cl);
}

/// Some(write-capable) when the parameter type is a repository abstraction.
fn repository_kind(ix: &Index<'_>, parameter_type: &str) -> Option<bool> {
    if parameter_type == REPOSITORY_BASE || chain_reaches(ix, parameter_type, &[REPOSITORY_BASE]) {
        Some(true)
    } else if parameter_type == READ_REPOSITORY_BASE || chain_reaches(ix, parameter_type, &[READ_REPOSITORY_BASE]) {
        Some(false)
    } else {
        None
    }
}

fn calls_write_member(ix: &Index<'_>, type_id: &str) -> bool {
    ix.members_of.get(type_id).into_iter().flatten().any(|m| ix.out.get(*m).into_iter().flatten().any(|r| r.kind == "call" && WRITE_MEMBERS.iter().any(|w| r.to.starts_with(w))))
}

/// Proxy P-EP-4 over the closure.
pub fn facts_of(ix: &Index<'_>, cl: &Closure<'_>) -> Facts {
    let (mut read, mut written, mut possibly, mut dbset) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
    for t in cl.types.iter().filter(|t| ix.types.contains_key(*t)) {
        let mut write_capable: Vec<String> = Vec::new();
        for ctor in constructors(ix, t) {
            for p in ix.members.get(ctor).into_iter().flat_map(|m| &m.parameters) {
                let (Some(write), Some(arg)) = (repository_kind(ix, &p.parameter_type), p.type_arguments.first()) else { continue };
                read.insert(arg.clone());
                if write {
                    write_capable.push(arg.clone());
                }
            }
        }
        if calls_write_member(ix, t) {
            match write_capable.as_slice() {
                [one] => {
                    written.insert(one.clone());
                }
                [] => {
                    possibly.insert(format!("? via {} (write call; the repository is not a constructor parameter)", short(t)));
                }
                many => possibly.extend(many.iter().cloned()),
            }
        }
    }
    for m in &cl.members {
        for r in ix.out.get(m).into_iter().flatten().filter(|r| r.kind == "access") {
            if let Some(p) = ix.members.get(r.to.as_str()).filter(|p| p.return_type.as_deref() == Some(DBSET)) {
                dbset.extend(p.return_type_arguments.first().cloned());
            }
        }
    }
    Facts { read: read.into_iter().collect(), written: written.into_iter().collect(), possibly_written: possibly.into_iter().collect(), dbset_touched: dbset.into_iter().collect(), proxy: FACTS_PROXY }
}

/// `member → identity member` for every identity member touched on the path.
pub fn identity_checks(ix: &Index<'_>, cl: &Closure<'_>) -> Vec<String> {
    let mut out = BTreeSet::new();
    for m in &cl.members {
        for r in ix.out.get(m).into_iter().flatten().filter(|r| (r.kind == "access" || r.kind == "call") && IDENTITY_MEMBERS.contains(&r.to.as_str())) {
            out.insert(format!("{} → {}", short(m), short(&r.to)));
        }
    }
    out.into_iter().collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct Pair {
    pub a: String,
    pub b: String,
    pub shared: usize,
    pub jaccard: f64,
}

/// Where paths coincide — the merge candidates, reported not decided.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Overlaps {
    /// Candidates whose path sets (own type excluded) are identical.
    pub identical: Vec<Vec<String>>,
    /// Pairs with Jaccard overlap ≥ 0.5, highest first.
    pub pairs: Vec<Pair>,
    /// The subset of `pairs` whose two candidates live on different types —
    /// overlap within a type is structural (constructor injection is per
    /// type); overlap across types is the signal.
    pub cross_type_pairs: Vec<Pair>,
    /// Production types reached by two or more candidates, most first.
    pub shared_types: Vec<(String, usize)>,
}

pub fn overlaps(cands: &[Candidate]) -> Overlaps {
    let sets: Vec<BTreeSet<&str>> = cands.iter().map(|c| c.path_types.iter().map(String::as_str).filter(|t| *t != c.type_id).collect()).collect();
    let mut groups: BTreeMap<Vec<&str>, Vec<String>> = BTreeMap::new();
    for (c, s) in cands.iter().zip(&sets).filter(|(_, s)| !s.is_empty()) {
        groups.entry(s.iter().copied().collect()).or_default().push(c.id.clone());
    }
    let mut pairs = Vec::new();
    for i in 0..sets.len() {
        for j in i + 1..sets.len() {
            let (shared, union) = (sets[i].intersection(&sets[j]).count(), sets[i].union(&sets[j]).count());
            let jaccard = if union == 0 { 0.0 } else { shared as f64 / union as f64 };
            if jaccard >= 0.5 {
                pairs.push(Pair { a: cands[i].id.clone(), b: cands[j].id.clone(), shared, jaccard });
            }
        }
    }
    pairs.sort_by(|x, y| y.jaccard.total_cmp(&x.jaccard).then(x.a.cmp(&y.a)).then(x.b.cmp(&y.b)));
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for t in sets.iter().flatten() {
        *count.entry(t).or_insert(0) += 1;
    }
    let mut shared_types: Vec<(String, usize)> = count.into_iter().filter(|(_, n)| *n >= 2).map(|(t, n)| (t.to_string(), n)).collect();
    shared_types.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let by_id: BTreeMap<&str, &str> = cands.iter().map(|c| (c.id.as_str(), c.type_id.as_str())).collect();
    let cross_type_pairs = pairs.iter().filter(|p| by_id.get(p.a.as_str()) != by_id.get(p.b.as_str())).cloned().collect();
    Overlaps { identical: groups.into_values().filter(|g| g.len() >= 2).collect(), pairs, cross_type_pairs, shared_types }
}
