//! Reachability over the C# inventory — the measure, from a stated root convention.
//!
//! Roots are a convention chosen here, never by the reader, and every report
//! prints the convention it used. The walk itself is `csharp_walk`: one
//! graph, composition edges resolved through the container's registrations,
//! every abstraction's role read through the declared proxies of
//! `csharp_roles`. A type is *reached*, *unresolved* (it implements an
//! abstraction an unresolved composition edge landed on), *partial* (likewise
//! for a partial edge) or *unreached*; no reachability figure prints without
//! those counts beside it (CG-R-62). Change coupling is not a cluster
//! dimension here — it needs the solution's git history, which the inventory
//! does not carry (not built, 2026-09-13).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::csharp_di::Resolver;
use super::csharp_inventory::{Index, Inventory};
use super::csharp_roles::{RoleProxy, PROXIES};
use super::csharp_walk::{by_role, closure, implementors_of, Closure, CompositionEdge, EdgeState};

/// One root convention. Which symbols it selects is a fact about the
/// inventory; that it is a root is this module's choice, printed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "kebab-case")]
pub enum Root {
    /// `Compilation.GetEntryPoint` members.
    EntryPoint,
    /// Public members of public types (effective accessibility).
    Public,
    /// Members or types carrying the attribute with this type id.
    Attribute(String),
    /// Types implementing or inheriting this type id (e.g. a handler interface).
    Implements(String),
    /// Types carrying the given attribute — the declared-slice root Gate 1b uses.
    Declared(String),
    /// One named member or type, e.g. a specific `Main` when a solution has several.
    Member(String),
}

impl Root {
    /// Parse `entry-point`, `public`, `attribute:<T:…>`, `implements:<T:…>`,
    /// `declared:<T:…>`, `member:<M:…|T:…>`.
    pub fn parse(spec: &str) -> Option<Root> {
        match spec.split_once(':') {
            None if spec == "entry-point" => Some(Root::EntryPoint),
            None if spec == "public" => Some(Root::Public),
            Some(("attribute", id)) => Some(Root::Attribute(id.to_string())),
            Some(("implements", id)) => Some(Root::Implements(id.to_string())),
            Some(("declared", id)) => Some(Root::Declared(id.to_string())),
            Some(("member", id)) => Some(Root::Member(id.to_string())),
            _ => None,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Root::EntryPoint => "entry-point".into(),
            Root::Public => "public".into(),
            Root::Attribute(id) => format!("attribute:{id}"),
            Root::Implements(id) => format!("implements:{id}"),
            Root::Declared(id) => format!("declared:{id}"),
            Root::Member(id) => format!("member:{id}"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReachOptions {
    pub roots: Vec<Root>,
    /// Symbol sets whose disposition is reported on their own (e.g. every
    /// handler), selected with the root syntax.
    pub track: Vec<Root>,
}

/// Resolution over the composition edges — the figure CG-R-61 wants first.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Resolution {
    /// Every composition edge, whatever its role.
    pub composition_edges: usize,
    /// Edges whose role enters the denominator (service, generic dispatch, factory).
    pub in_denominator: usize,
    pub resolved: usize,
    pub partial: usize,
    pub unresolved: usize,
    /// resolved / (resolved + unresolved); partial is reported, not divided.
    pub coverage_percent: f64,
    pub by_role: BTreeMap<String, usize>,
    pub excluded_by_role: BTreeMap<String, usize>,
    pub by_reason: BTreeMap<String, usize>,
    pub unresolved_edges: Vec<CompositionEdge>,
    pub proxies: &'static [RoleProxy],
}

/// Four-way disposition of a type set.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Disposition {
    pub label: String,
    pub types: usize,
    pub reached: usize,
    pub unresolved: usize,
    pub partial: usize,
    pub unreached: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootRow {
    pub root: String,
    pub root_symbols: usize,
    pub reached_types: usize,
    pub unresolved_edges: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReachReport {
    pub roots: Vec<String>,
    pub total: Disposition,
    pub percent_reached: f64,
    pub resolution: Resolution,
    pub registrations_read: usize,
    pub calls_ignored: usize,
    pub ignored_by_method: BTreeMap<String, usize>,
    pub by_root: Vec<RootRow>,
    pub by_namespace: Vec<Disposition>,
    pub tracked: Vec<Disposition>,
    pub unreached: Vec<String>,
}

/// The symbols a root convention selects, as member ids plus type ids.
pub fn root_symbols<'a>(inv: &'a Inventory, ix: &Index<'a>, root: &Root) -> BTreeSet<&'a str> {
    let mut out = BTreeSet::new();
    match root {
        Root::EntryPoint => out.extend(inv.members.iter().filter(|m| m.is_entry_point).map(|m| m.id.as_str())),
        Root::Public => {
            for m in &inv.members {
                let public_type = ix.types.get(m.declaring_type.as_str()).is_some_and(|t| t.accessibility == "public");
                if public_type && m.accessibility == "public" {
                    out.insert(m.id.as_str());
                }
            }
        }
        Root::Attribute(id) => {
            out.extend(inv.members_with_attribute(id).iter().map(|m| m.id.as_str()));
            out.extend(inv.types_with_attribute(id).iter().map(|(t, _)| t.id.as_str()));
        }
        Root::Implements(id) => out.extend(ix.implementors.get(id.as_str()).into_iter().flatten().copied()),
        Root::Declared(id) => out.extend(inv.types_with_attribute(id).iter().map(|(t, _)| t.id.as_str())),
        Root::Member(id) => {
            out.extend(ix.members.get(id.as_str()).map(|m| m.id.as_str()));
            out.extend(ix.types.get(id.as_str()).map(|t| t.id.as_str()));
        }
    }
    out
}

/// The three non-reached sets a closure implies.
pub struct Sets<'a> {
    pub unresolved: BTreeSet<&'a str>,
    pub partial: BTreeSet<&'a str>,
}

pub fn sets<'a>(ix: &Index<'a>, c: &Closure<'a>) -> Sets<'a> {
    let unresolved = implementors_of(ix, c, &c.targets_where(|s| matches!(s, EdgeState::Unresolved(_))));
    let partial = implementors_of(ix, c, &c.targets_where(|s| *s == EdgeState::Partial))
        .into_iter()
        .filter(|t| !unresolved.contains(t))
        .collect();
    Sets { unresolved, partial }
}

pub fn disposition(label: &str, ids: &BTreeSet<&str>, c: &Closure<'_>, s: &Sets<'_>) -> Disposition {
    let reached = ids.iter().filter(|t| c.types.contains(*t)).count();
    let unresolved = ids.iter().filter(|t| !c.types.contains(*t) && s.unresolved.contains(*t)).count();
    let partial = ids.iter().filter(|t| !c.types.contains(*t) && s.partial.contains(*t)).count();
    Disposition { label: label.to_string(), types: ids.len(), reached, unresolved, partial, unreached: ids.len() - reached - unresolved - partial }
}

/// Run the measure.
pub fn reach(inv: &Inventory, opts: &ReachOptions) -> ReachReport {
    let ix = inv.index();
    let resolver = Resolver::build(inv);
    let mut all_roots: BTreeSet<&str> = BTreeSet::new();
    let mut by_root = Vec::new();
    for root in &opts.roots {
        let symbols = root_symbols(inv, &ix, root);
        let c = closure(&ix, &resolver, &symbols);
        by_root.push(RootRow { root: root.label(), root_symbols: symbols.len(), reached_types: c.types.len(), unresolved_edges: c.count(|s| matches!(s, EdgeState::Unresolved(_))) });
        all_roots.extend(symbols);
    }
    let c = closure(&ix, &resolver, &all_roots);
    let s = sets(&ix, &c);
    let every: BTreeSet<&str> = inv.type_ids();
    let total = disposition("total", &every, &c, &s);
    ReachReport {
        roots: opts.roots.iter().map(Root::label).collect(),
        percent_reached: if every.is_empty() { 0.0 } else { 100.0 * total.reached as f64 / every.len() as f64 },
        total,
        resolution: resolution_of(&c),
        registrations_read: resolver.registrations_read,
        calls_ignored: resolver.calls_ignored,
        ignored_by_method: resolver.ignored_by_method.clone(),
        by_root,
        by_namespace: namespace_rows(inv, &c, &s),
        tracked: tracked_rows(inv, &ix, &opts.track, &c, &s),
        unreached: inv.types.iter().map(|t| t.id.as_str()).filter(|t| !c.types.contains(t) && !s.unresolved.contains(t) && !s.partial.contains(t)).map(String::from).collect(),
    }
}

pub fn resolution_of(c: &Closure<'_>) -> Resolution {
    let resolved = c.count(|s| *s == EdgeState::Resolved);
    let partial = c.count(|s| *s == EdgeState::Partial);
    let unresolved = c.count(|s| matches!(s, EdgeState::Unresolved(_)));
    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    let mut excluded: BTreeMap<String, usize> = BTreeMap::new();
    for e in &c.edges {
        match e.state {
            EdgeState::Unresolved(r) => *by_reason.entry(r.label().to_string()).or_insert(0) += 1,
            EdgeState::Excluded(role) => *excluded.entry(role.label().to_string()).or_insert(0) += 1,
            _ => {}
        }
    }
    let scored = resolved + unresolved;
    Resolution {
        composition_edges: c.edges.len(),
        in_denominator: resolved + partial + unresolved,
        resolved,
        partial,
        unresolved,
        coverage_percent: if scored == 0 { 100.0 } else { 100.0 * resolved as f64 / scored as f64 },
        by_role: by_role(c),
        excluded_by_role: excluded,
        by_reason,
        unresolved_edges: c.edges.iter().filter(|e| matches!(e.state, EdgeState::Unresolved(_))).cloned().collect(),
        proxies: PROXIES,
    }
}

fn namespace_rows<'a>(inv: &'a Inventory, c: &Closure<'a>, s: &Sets<'a>) -> Vec<Disposition> {
    let mut groups: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for t in &inv.types {
        groups.entry(t.namespace.as_str()).or_default().insert(t.id.as_str());
    }
    groups.into_iter().map(|(ns, ids)| disposition(if ns.is_empty() { "(global)" } else { ns }, &ids, c, s)).collect()
}

fn tracked_rows<'a>(inv: &'a Inventory, ix: &Index<'a>, track: &[Root], c: &Closure<'a>, s: &Sets<'a>) -> Vec<Disposition> {
    track
        .iter()
        .map(|t| {
            let ids: BTreeSet<&str> = root_symbols(inv, ix, t).into_iter().map(|x| ix.members.get(x).map(|m| m.declaring_type.as_str()).unwrap_or(x)).collect();
            disposition(&t.label(), &ids, c, s)
        })
        .collect()
}

pub use super::csharp_reach_render::render_reach;

#[cfg(test)]
#[path = "csharp_reach_tests.rs"]
mod tests;
