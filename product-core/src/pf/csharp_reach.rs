//! Reachability over the C# inventory — one graph, resolved through the container (CG-R-60).
//!
//! Roots are a convention chosen here, never by the reader, and every report
//! prints the convention it used. Edges `call`, `construct`, `access` and
//! `type-reference` are followed directly; an edge that lands on an interface,
//! an abstract type, or a registered service is **interface-mediated** and is
//! followed through [`Resolver`] to the implementations the container would
//! supply. An edge the resolver cannot follow is recorded with its reason and
//! is never folded into "unreached" (CG-R-62): a type is *unresolved* when it
//! is not reached and implements an interface some unresolved edge landed on.
//! No reachability figure is rendered without its unresolved count beside it.
//! Change coupling is not a cluster dimension here — it needs the solution's
//! git history, which the inventory does not carry (not built, 2026-09-13).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

use super::csharp_di::{Reason, Resolver};
use super::csharp_inventory::{Index, Inventory};

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
    /// MediatR handler), selected with the root syntax.
    pub track: Vec<Root>,
}

/// An interface-mediated edge the resolver could not follow, from the
/// referencing type.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct UnresolvedEdge {
    pub from: String,
    pub interface: String,
    pub reason: Reason,
}

/// Resolution coverage — the figure CG-R-61 wants before any percentage.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Resolution {
    pub interface_edges: usize,
    pub resolved: usize,
    pub unresolved: usize,
    pub coverage_percent: f64,
    pub by_reason: BTreeMap<String, usize>,
    pub unresolved_edges: Vec<UnresolvedEdge>,
}

/// Three-way disposition of a type set: reached, unresolved, unreached.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Disposition {
    pub label: String,
    pub types: usize,
    pub reached: usize,
    pub unresolved: usize,
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

/// What one walk reached, and what it could not follow. Interface-mediated
/// edges are counted once per (referencing type, interface) pair.
#[derive(Debug, Default)]
pub struct Closure<'a> {
    pub members: BTreeSet<&'a str>,
    pub types: BTreeSet<&'a str>,
    pub unresolved: BTreeSet<UnresolvedEdge>,
    pub resolved_edges: usize,
    seen_edges: BTreeSet<(&'a str, &'a str)>,
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

/// Walk to a fixpoint: a registration counts once the member it sits in has
/// been reached, so rounds repeat until nothing new is reached.
pub fn closure<'a>(ix: &Index<'a>, resolver: &Resolver, start: &BTreeSet<&'a str>) -> Closure<'a> {
    let mut previous: BTreeSet<&'a str> = BTreeSet::new();
    for _ in 0..8 {
        let c = walk(ix, resolver, start, &previous);
        if c.members == previous {
            return c;
        }
        previous = c.members;
    }
    walk(ix, resolver, start, &previous)
}

fn walk<'a>(ix: &Index<'a>, resolver: &Resolver, start: &BTreeSet<&'a str>, sites: &BTreeSet<&'a str>) -> Closure<'a> {
    let mut c = Closure::default();
    let mut queue: VecDeque<&'a str> = start.iter().copied().collect();
    for id in start {
        queue.extend(ix.members_of.get(id).into_iter().flatten().copied());
    }
    let site_reached = |s: &str| sites.contains(s);
    while let Some(id) = queue.pop_front() {
        let Some(m) = ix.members.get(id) else {
            if ix.types.contains_key(id) {
                c.types.insert(id);
            }
            continue;
        };
        if !c.members.insert(id) {
            continue;
        }
        c.types.insert(m.declaring_type.as_str());
        for r in ix.out.get(id).into_iter().flatten() {
            if !matches!(r.kind.as_str(), "call" | "construct" | "access" | "type-reference") {
                continue;
            }
            let to = r.to.as_str();
            let target_type = ix.members.get(to).map(|t| t.declaring_type.as_str()).unwrap_or(to);
            if !ix.types.contains_key(target_type) || is_registration_itself(resolver, id, to, target_type, &r.kind) {
                continue; // outside the solution, or the registration call itself
            }
            if ix.abstract_types.contains(target_type) || resolver.is_registered(target_type) {
                c.types.insert(target_type);
                if !c.seen_edges.insert((m.declaring_type.as_str(), target_type)) {
                    continue;
                }
                match resolver.resolve(target_type, &site_reached) {
                    Ok(impls) => {
                        c.resolved_edges += 1;
                        for i in impls {
                            let impl_id = ix.types.get(i.implementation.as_str()).map(|t| t.id.as_str());
                            if let Some(impl_id) = impl_id {
                                c.types.insert(impl_id);
                                queue.extend(ix.members_of.get(impl_id).into_iter().flatten().copied());
                            }
                        }
                    }
                    Err(reason) => {
                        c.unresolved.insert(UnresolvedEdge { from: m.declaring_type.to_string(), interface: target_type.to_string(), reason });
                    }
                }
            } else {
                queue.push_back(to);
            }
        }
    }
    c
}

/// A type argument of a registration, or the construction inside its
/// instance/factory argument, is the registration — not a use of the type.
fn is_registration_itself(resolver: &Resolver, site: &str, to: &str, target_type: &str, kind: &str) -> bool {
    match kind {
        "type-reference" => resolver.mentions(site, target_type),
        "construct" => resolver.constructs(site, target_type),
        "call" => to.contains(".#ctor(") && resolver.constructs(site, target_type),
        _ => false,
    }
}

/// Types that are not reached and implement an interface an unresolved edge landed on.
pub fn unresolved_types<'a>(ix: &Index<'a>, c: &Closure<'a>) -> BTreeSet<&'a str> {
    let ifaces: BTreeSet<&str> = c.unresolved.iter().map(|u| u.interface.as_str()).collect();
    ifaces
        .iter()
        .flat_map(|i| ix.implementors.get(i).into_iter().flatten().copied())
        .filter(|t| !c.types.contains(t))
        .collect()
}

fn disposition(label: &str, ids: &BTreeSet<&str>, reached: &BTreeSet<&str>, unresolved: &BTreeSet<&str>) -> Disposition {
    let r = ids.iter().filter(|t| reached.contains(*t)).count();
    let u = ids.iter().filter(|t| !reached.contains(*t) && unresolved.contains(*t)).count();
    Disposition { label: label.to_string(), types: ids.len(), reached: r, unresolved: u, unreached: ids.len() - r - u }
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
        by_root.push(RootRow { root: root.label(), root_symbols: symbols.len(), reached_types: c.types.len(), unresolved_edges: c.unresolved.len() });
        all_roots.extend(symbols);
    }
    let c = closure(&ix, &resolver, &all_roots);
    let unresolved = unresolved_types(&ix, &c);
    let every: BTreeSet<&str> = inv.type_ids();
    let total = disposition("total", &every, &c.types, &unresolved);
    ReachReport {
        roots: opts.roots.iter().map(Root::label).collect(),
        percent_reached: if every.is_empty() { 0.0 } else { 100.0 * total.reached as f64 / every.len() as f64 },
        total,
        resolution: resolution_of(&c),
        registrations_read: resolver.registrations_read,
        calls_ignored: resolver.calls_ignored,
        ignored_by_method: resolver.ignored_by_method.clone(),
        by_root,
        by_namespace: namespace_rows(inv, &c.types, &unresolved),
        tracked: tracked_rows(inv, &ix, &opts.track, &c, &unresolved),
        unreached: inv.types.iter().filter(|t| !c.types.contains(t.id.as_str()) && !unresolved.contains(t.id.as_str())).map(|t| t.id.clone()).collect(),
    }
}

fn resolution_of(c: &Closure<'_>) -> Resolution {
    let mut by_reason: BTreeMap<String, usize> = BTreeMap::new();
    for u in &c.unresolved {
        *by_reason.entry(u.reason.label().to_string()).or_insert(0) += 1;
    }
    let interface_edges = c.resolved_edges + c.unresolved.len();
    Resolution {
        interface_edges,
        resolved: c.resolved_edges,
        unresolved: c.unresolved.len(),
        coverage_percent: if interface_edges == 0 { 100.0 } else { 100.0 * c.resolved_edges as f64 / interface_edges as f64 },
        by_reason,
        unresolved_edges: c.unresolved.iter().cloned().collect(),
    }
}

fn tracked_rows<'a>(inv: &'a Inventory, ix: &Index<'a>, track: &[Root], c: &Closure<'a>, unresolved: &BTreeSet<&'a str>) -> Vec<Disposition> {
    track
        .iter()
        .map(|t| {
            let ids: BTreeSet<&str> = root_symbols(inv, ix, t)
                .into_iter()
                .map(|s| ix.members.get(s).map(|m| m.declaring_type.as_str()).unwrap_or(s))
                .collect();
            disposition(&t.label(), &ids, &c.types, unresolved)
        })
        .collect()
}

fn namespace_rows<'a>(inv: &'a Inventory, reached: &BTreeSet<&'a str>, unresolved: &BTreeSet<&'a str>) -> Vec<Disposition> {
    let mut groups: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for t in &inv.types {
        groups.entry(t.namespace.as_str()).or_default().insert(t.id.as_str());
    }
    groups
        .into_iter()
        .map(|(ns, ids)| disposition(if ns.is_empty() { "(global)" } else { ns }, &ids, reached, unresolved))
        .collect()
}

fn row(d: &Disposition) -> String {
    format!("{:<52} types={:<5} reached={:<5} unresolved={:<5} unreached={}\n", d.label, d.types, d.reached, d.unresolved, d.unreached)
}

/// Text rendering: the convention first, then coverage, then every figure
/// with its unresolved count beside it.
pub fn render_reach(report: &ReachReport) -> String {
    let mut s = format!("roots: {}\n", report.roots.join(", "));
    let t = &report.total;
    s.push_str(&format!(
        "reached: {} of {} types ({:.1}%) — unresolved: {} — unreached: {}\n",
        t.reached, t.types, report.percent_reached, t.unresolved, t.unreached
    ));
    let r = &report.resolution;
    s.push_str(&format!(
        "resolution coverage: {} of {} interface-mediated edges resolved ({:.1}%) — {} unresolved\n  registrations read: {} — IServiceCollection calls ignored: {}\n",
        r.resolved, r.interface_edges, r.coverage_percent, r.unresolved, report.registrations_read, report.calls_ignored
    ));
    for (reason, n) in &r.by_reason {
        s.push_str(&format!("  unresolved by reason: {reason:<26} {n}\n"));
    }
    let mut ignored: Vec<(&String, &usize)> = report.ignored_by_method.iter().collect();
    ignored.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (method, n) in ignored.iter().take(12) {
        s.push_str(&format!("  ignored call:         {method:<26} {n}\n"));
    }
    s.push_str("\nby root convention:\n");
    for b in &report.by_root {
        s.push_str(&format!("  {:<52} roots={:<5} reached types={:<5} unresolved edges={}\n", b.root, b.root_symbols, b.reached_types, b.unresolved_edges));
    }
    s.push_str("\nby namespace:\n");
    for d in &report.by_namespace {
        s.push_str(&format!("  {}", row(d)));
    }
    if !report.tracked.is_empty() {
        s.push_str("\ntracked:\n");
        for d in &report.tracked {
            s.push_str(&format!("  {}", row(d)));
        }
    }
    if !r.unresolved_edges.is_empty() {
        s.push_str(&format!("\nunresolved edges (first {} of {}):\n", r.unresolved_edges.len().min(25), r.unresolved_edges.len()));
        for u in r.unresolved_edges.iter().take(25) {
            s.push_str(&format!("  {:<26} {} -> {}\n", u.reason.label(), u.from, u.interface));
        }
    }
    s
}

#[cfg(test)]
#[path = "csharp_reach_tests.rs"]
mod tests;
