//! Reachability over the C# inventory — the §12.1 measure (Gate 1a).
//!
//! Roots are a convention chosen here, never by the reader, and every report
//! prints the convention it used. Traversal follows `call`, `construct`,
//! `access` and `type-reference` edges; `implement`/`inherit` edges (a DI
//! container's resolution) are followed only when asked, and the report says
//! which. A type counts as reached when it is referenced or any of its members
//! is reached. Change coupling is not a cluster dimension here — it needs the
//! solution's git history, which the inventory does not carry (not built,
//! 2026-09-13).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::Serialize;

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
}

impl Root {
    /// Parse `entry-point`, `public`, `attribute:<T:…>`, `implements:<T:…>`,
    /// `declared:<T:…>`.
    pub fn parse(spec: &str) -> Option<Root> {
        match spec.split_once(':') {
            None if spec == "entry-point" => Some(Root::EntryPoint),
            None if spec == "public" => Some(Root::Public),
            Some(("attribute", id)) => Some(Root::Attribute(id.to_string())),
            Some(("implements", id)) => Some(Root::Implements(id.to_string())),
            Some(("declared", id)) => Some(Root::Declared(id.to_string())),
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
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ReachOptions {
    pub roots: Vec<Root>,
    /// Follow `implement`/`inherit` edges from a reached type to its
    /// implementors — the DI container's resolution, approximated.
    pub through_implementations: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NamespaceRow {
    pub namespace: String,
    pub types: usize,
    pub reached: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RootRow {
    pub root: String,
    pub root_symbols: usize,
    pub reached_types: usize,
}

/// The reachability report: totals, per root convention, per namespace.
#[derive(Debug, Clone, Serialize)]
pub struct ReachReport {
    pub through_implementations: bool,
    pub types_total: usize,
    pub types_reached: usize,
    pub percent_reached: f64,
    pub by_root: Vec<RootRow>,
    pub by_namespace: Vec<NamespaceRow>,
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
    }
    out
}

/// The set of type ids reached from `start` (member or type ids).
pub fn closure<'a>(ix: &Index<'a>, start: &BTreeSet<&'a str>, through_implementations: bool) -> BTreeSet<&'a str> {
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut types: BTreeSet<&str> = BTreeSet::new();
    let mut queue: VecDeque<&str> = start.iter().copied().collect();
    // A type chosen as a root stands for its members.
    for id in start {
        queue.extend(ix.members_of.get(id).into_iter().flatten().copied());
    }
    while let Some(id) = queue.pop_front() {
        if !seen.insert(id) {
            continue;
        }
        if let Some(m) = ix.members.get(id) {
            queue.push_back(m.declaring_type.as_str());
        } else if ix.types.contains_key(id) {
            types.insert(id);
            if through_implementations {
                for impl_id in ix.implementors.get(id).into_iter().flatten() {
                    queue.push_back(impl_id);
                    queue.extend(ix.members_of.get(impl_id).into_iter().flatten().copied());
                }
            }
        } else {
            continue; // outside the solution
        }
        if id.starts_with("T:") && !start.contains(id) && !through_implementations {
            continue; // a referenced type's members are not executed by the reference
        }
        for r in ix.out.get(id).into_iter().flatten() {
            if matches!(r.kind.as_str(), "call" | "construct" | "access" | "type-reference") {
                queue.push_back(r.to.as_str());
            }
        }
    }
    types
}

/// Run the measure.
pub fn reach(inv: &Inventory, opts: &ReachOptions) -> ReachReport {
    let ix = inv.index();
    let mut all_roots: BTreeSet<&str> = BTreeSet::new();
    let mut by_root = Vec::new();
    for root in &opts.roots {
        let symbols = root_symbols(inv, &ix, root);
        let reached = closure(&ix, &symbols, opts.through_implementations);
        by_root.push(RootRow { root: root.label(), root_symbols: symbols.len(), reached_types: reached.len() });
        all_roots.extend(symbols);
    }
    let reached = closure(&ix, &all_roots, opts.through_implementations);
    let by_namespace = namespace_rows(inv, &reached);
    let types_total = inv.types.len();
    let unreached = inv.types.iter().filter(|t| !reached.contains(t.id.as_str())).map(|t| t.id.clone()).collect();
    ReachReport {
        through_implementations: opts.through_implementations,
        types_total,
        types_reached: reached.len(),
        percent_reached: if types_total == 0 { 0.0 } else { 100.0 * reached.len() as f64 / types_total as f64 },
        by_root,
        by_namespace,
        unreached,
    }
}

fn namespace_rows(inv: &Inventory, reached: &BTreeSet<&str>) -> Vec<NamespaceRow> {
    let mut rows: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for t in &inv.types {
        let e = rows.entry(t.namespace.as_str()).or_insert((0, 0));
        e.0 += 1;
        if reached.contains(t.id.as_str()) {
            e.1 += 1;
        }
    }
    rows.into_iter().map(|(ns, (types, r))| NamespaceRow { namespace: ns.to_string(), types, reached: r }).collect()
}

/// Text rendering, with the convention stated first.
pub fn render_reach(report: &ReachReport, opts: &ReachOptions) -> String {
    let mut s = String::new();
    let roots: Vec<String> = opts.roots.iter().map(Root::label).collect();
    s.push_str(&format!("roots: {}\nthrough implementations (DI): {}\n", roots.join(", "), report.through_implementations));
    s.push_str(&format!(
        "reached: {} of {} types ({:.1}%)\n",
        report.types_reached, report.types_total, report.percent_reached
    ));
    s.push_str("\nby root convention:\n");
    for r in &report.by_root {
        s.push_str(&format!("  {:<50} roots={:<5} reached types={}\n", r.root, r.root_symbols, r.reached_types));
    }
    s.push_str("\nby namespace:\n");
    for n in &report.by_namespace {
        let ns = if n.namespace.is_empty() { "(global)" } else { n.namespace.as_str() };
        s.push_str(&format!("  {:<50} {}/{}\n", ns, n.reached, n.types));
    }
    if !report.unreached.is_empty() {
        s.push_str(&format!("\nunreached ({}):\n", report.unreached.len()));
        for t in &report.unreached {
            s.push_str(&format!("  {t}\n"));
        }
    }
    s
}

#[cfg(test)]
#[path = "csharp_reach_tests.rs"]
mod tests;
