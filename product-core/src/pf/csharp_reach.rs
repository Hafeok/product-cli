//! Reachability over the C# inventory — the measure, from a stated root convention.
//!
//! Roots are a convention chosen here, never by the reader, and every report
//! prints the convention it used. The walk itself is `csharp_walk`: one
//! graph, composition edges resolved through the container's registrations,
//! every abstraction's role read through the declared proxies of
//! `csharp_roles`. A type is *reached*, *unresolved* (it implements an
//! abstraction an unresolved composition edge landed on), *partial* (likewise
//! for a partial edge) or *unreached*; no reachability figure prints without
//! those counts beside it (CG-R-62), and no coverage figure without its
//! population (CG-R-73, `csharp_resolution`). Test projects — classified by
//! the test framework they reference — are outside the primary convention
//! and reported on their own row (CG-R-75). A ground-truth file adds reader
//! recall and walk recall/precision (CG-R-71, `csharp_ground_truth`).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::csharp_di::{Resolver, TableCoverage};
use super::csharp_ground_truth::{measure, GroundTruth, GroundTruthReport};
use super::csharp_inventory::{Index, Inventory};
use super::csharp_resolution::{resolution_of, Resolution};
use super::csharp_walk::{closure, implementors_of, Closure, EdgeState};

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
    /// Types implementing or inheriting this type id, through any chain of
    /// in-solution bases (P-4).
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct ReachOptions {
    pub roots: Vec<Root>,
    /// Symbol sets whose disposition is reported on their own (e.g. every
    /// handler), selected with the root syntax.
    pub track: Vec<Root>,
    /// A hand-enumerated ground truth for one project (CG-R-71).
    pub ground_truth: Option<GroundTruth>,
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

/// The extent of what the instrument cannot see (CG-R-78): Razor views are
/// not in the inventory, and every `@inject` in them is a composition edge.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BlindSpot {
    pub razor_files: u64,
    pub razor_inject_directives: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReachReport {
    pub roots: Vec<String>,
    pub blind_spot: BlindSpot,
    pub table_coverage: TableCoverage,
    /// Over production types: test projects are excluded and reported below.
    pub total: Disposition,
    pub percent_reached: f64,
    pub test_projects: Vec<String>,
    pub test_project_types: Disposition,
    pub resolution: Resolution,
    pub ground_truth: Option<GroundTruthReport>,
    pub registrations_read: usize,
    pub calls_ignored: usize,
    pub test_sites_skipped: usize,
    /// Ignored calls to the solution's own extension methods — their bodies are walked.
    pub ignored_in_solution: BTreeMap<String, usize>,
    /// Ignored calls to external methods — bodies unavailable.
    pub ignored_external: BTreeMap<String, usize>,
    /// Reached external calls the resolver neither parses nor knows (CG-R-75).
    pub unlearned_calls: BTreeMap<String, usize>,
    pub by_root: Vec<RootRow>,
    pub by_namespace: Vec<Disposition>,
    pub tracked: Vec<Disposition>,
    pub unreached: Vec<String>,
}

/// Types implementing or inheriting `id`, through any chain of in-solution bases.
fn transitive_implementors<'a>(ix: &Index<'a>, id: &str) -> BTreeSet<&'a str> {
    let mut out = BTreeSet::new();
    let mut stack: Vec<&str> = ix.implementors.get(id).into_iter().flatten().copied().collect();
    while let Some(t) = stack.pop() {
        if out.insert(t) {
            stack.extend(ix.implementors.get(t).into_iter().flatten().copied());
        }
    }
    out
}

/// The symbols a root convention selects, as member ids plus type ids —
/// never from a test project.
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
        Root::Implements(id) => out.extend(transitive_implementors(ix, id)),
        Root::Declared(id) => out.extend(inv.types_with_attribute(id).iter().map(|(t, _)| t.id.as_str())),
        Root::Member(id) => {
            out.extend(ix.members.get(id.as_str()).map(|m| m.id.as_str()));
            out.extend(ix.types.get(id.as_str()).map(|t| t.id.as_str()));
        }
    }
    out.retain(|s| !ix.is_test(ix.members.get(s).map(|m| m.declaring_type.as_str()).unwrap_or(s)));
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
    let (production, test): (BTreeSet<&str>, BTreeSet<&str>) = inv.type_ids().into_iter().partition(|t| !ix.is_test(t));
    let total = disposition("total (production)", &production, &c, &s);
    let reached_sites = |x: &str| c.members.contains(x);
    let (ignored_in_solution, ignored_external) = resolver.ignored_split();
    let production_projects = inv.projects.iter().filter(|p| !p.is_test());
    ReachReport {
        roots: opts.roots.iter().map(Root::label).collect(),
        blind_spot: production_projects.fold(BlindSpot::default(), |b, p| BlindSpot { razor_files: b.razor_files + p.razor_files, razor_inject_directives: b.razor_inject_directives + p.razor_inject_directives }),
        table_coverage: resolver.table_coverage(&reached_sites),
        percent_reached: if production.is_empty() { 0.0 } else { 100.0 * total.reached as f64 / production.len() as f64 },
        total,
        test_projects: ix.test_projects.iter().map(|p| p.to_string()).collect(),
        test_project_types: disposition("test projects (excluded from the primary convention)", &test, &c, &s),
        resolution: resolution_of(&ix, &c),
        ground_truth: opts.ground_truth.as_ref().map(|gt| measure(inv, &ix, &c, gt)),
        registrations_read: resolver.registrations_read,
        calls_ignored: resolver.calls_ignored,
        test_sites_skipped: resolver.test_sites_skipped,
        ignored_in_solution,
        ignored_external,
        unlearned_calls: resolver.unlearned(&reached_sites),
        by_root,
        by_namespace: namespace_rows(inv, &ix, &c, &s),
        tracked: tracked_rows(inv, &ix, &opts.track, &c, &s),
        unreached: production.iter().filter(|t| !c.types.contains(*t) && !s.unresolved.contains(*t) && !s.partial.contains(*t)).map(|t| t.to_string()).collect(),
    }
}

fn namespace_rows<'a>(inv: &'a Inventory, ix: &Index<'a>, c: &Closure<'a>, s: &Sets<'a>) -> Vec<Disposition> {
    let mut groups: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for t in inv.types.iter().filter(|t| !ix.is_test(&t.id)) {
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
pub use super::csharp_resolution::BoundaryRow;

#[cfg(test)]
#[path = "csharp_reach_tests.rs"]
mod tests;
