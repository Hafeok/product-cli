//! The three-region delta over a C# inventory — measurement against an act vocabulary (Gate 1b).
//!
//! Indexed by act, never by symbol (R-A, R-D): for every act in the event model
//! the report says *declared*, *declarable*, *unstructured* or *unrealised*.
//! Undeclared symbols appear only as counts by cluster in the two ratios; no
//! output lists symbols as candidate slices.
//!
//! The declarable/unstructured separator is a declared proxy (CG-R-52), carried
//! on every report as [`PROXY`]. Operationally: a non-fact type `T` has the
//! referenced-fact set `F(T)` (facts realised by types `T`'s members reach);
//! the acts covering `T` are those whose read ∪ write set ⊇ `F(T)`. One or
//! more covering acts → declarable for them; none, with `F(T)` non-empty →
//! the act boundary runs through `T` (unstructured); `F(T)` empty → neither.
//! A spanning type is attributed to the acts that *write* a fact it constructs
//! (a `construct` edge to the realising type), or, when it constructs none, to
//! the acts that read a fact it reaches.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::csharp_inventory::{Index, Inventory};
use super::csharp_reach::{closure, root_symbols, Root};
use super::eventmodel::EventModel;

/// CG-R-52: the separator is a proxy and is reported as one, every time.
pub const PROXY: ProxyDeclaration = ProxyDeclaration {
    proxy: "a type references realised facts of two or more acts (no single act's positions cover them)",
    original_predicate: "the type contains decision logic belonging to more than one act",
    known_divergence: "a shared value object, DTO or mapping type referenced across many acts reads as unstructured while being neither; shared types are shared, not unstructured",
};

#[derive(Debug, Clone, Serialize)]
pub struct ProxyDeclaration {
    pub proxy: &'static str,
    pub original_predicate: &'static str,
    pub known_divergence: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeltaOptions {
    /// Type id of the slice attribute (default `T:Product.Binding.SliceAttribute`).
    pub slice_attribute: String,
    /// Type id of the fact attribute (default `T:Product.Binding.RealisesFactAttribute`).
    pub realises_fact_attribute: String,
    pub through_implementations: bool,
}

impl Default for DeltaOptions {
    fn default() -> Self {
        DeltaOptions {
            slice_attribute: "T:Product.Binding.SliceAttribute".into(),
            realises_fact_attribute: "T:Product.Binding.RealisesFactAttribute".into(),
            through_implementations: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Region {
    Declared,
    Declarable,
    Unstructured,
    Unrealised,
}

#[derive(Debug, Clone, Serialize)]
pub struct ActRow {
    pub address: String,
    pub region: Region,
    /// Declared: the declaring symbols. Declarable: the attachable types.
    /// Unstructured: the spanning types. Unrealised: empty.
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpanningType {
    pub type_id: String,
    pub facts: Vec<String>,
    pub acts_touched: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Ratios {
    pub through_implementations: bool,
    pub undeclared_types: usize,
    pub reachable_undeclared: usize,
    pub isolated_undeclared: usize,
    pub by_namespace: Vec<(String, usize, usize)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeltaReport {
    pub proxy: ProxyDeclaration,
    pub context: String,
    pub acts: Vec<ActRow>,
    pub spanning_types: Vec<SpanningType>,
    /// `[Slice]` naming an act the vocabulary does not have: (symbol, act).
    pub declared_unresolved: Vec<(String, String)>,
    /// `[RealisesFact]` naming a fact the vocabulary does not have: (type, fact).
    pub fact_orphans: Vec<(String, String)>,
    /// Facts in the vocabulary no type realises.
    pub facts_unrealised: Vec<String>,
    pub ratios: Ratios,
}

/// Fact realisation as declared: fact id → realising type ids.
fn fact_types<'a>(inv: &'a Inventory, opts: &DeltaOptions) -> BTreeMap<&'a str, Vec<&'a str>> {
    let mut out: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for (t, a) in inv.types_with_attribute(&opts.realises_fact_attribute) {
        if let Some(fact) = a.arg(0) {
            out.entry(fact).or_default().push(t.id.as_str());
        }
    }
    out
}

/// `F(T)`, the facts whose realising types `T`'s members reach directly, and
/// the subset `T` *produces* (a `construct` edge to the realising type).
fn referenced_facts<'a>(
    ix: &Index<'a>,
    type_id: &str,
    type_of_fact: &BTreeMap<&'a str, &'a str>,
) -> (BTreeSet<&'a str>, BTreeSet<&'a str>) {
    let mut all = BTreeSet::new();
    let mut produced = BTreeSet::new();
    for m in ix.members_of.get(type_id).into_iter().flatten() {
        for r in ix.out.get(*m).into_iter().flatten() {
            if !matches!(r.kind.as_str(), "call" | "construct" | "access" | "type-reference") {
                continue;
            }
            let to = r.to.as_str();
            let owner = ix.members.get(to).map(|m| m.declaring_type.as_str()).unwrap_or(to);
            let Some(f) = type_of_fact.get(owner).filter(|_| owner != type_id) else { continue };
            all.insert(*f);
            if r.kind == "construct" {
                produced.insert(*f);
            }
        }
    }
    (all, produced)
}

/// The acts a spanning type is attributed to: those that write a fact it
/// constructs — or, when it constructs none, those that read a fact it reaches.
fn acts_spanned(model: &EventModel, all: &BTreeSet<&str>, produced: &BTreeSet<&str>) -> BTreeSet<String> {
    let by_write: BTreeSet<String> = model
        .slices
        .iter()
        .filter(|s| s.writes.iter().any(|w| produced.contains(w.as_str())))
        .map(|s| s.address())
        .collect();
    if !by_write.is_empty() {
        return by_write;
    }
    model.slices.iter().filter(|s| s.reads.iter().any(|r| all.contains(r.as_str()))).map(|s| s.address()).collect()
}

/// Run the delta.
pub fn delta(inv: &Inventory, model: &EventModel, opts: &DeltaOptions) -> DeltaReport {
    let ix = inv.index();
    let facts = fact_types(inv, opts);
    let type_of_fact: BTreeMap<&str, &str> = facts.iter().flat_map(|(f, ts)| ts.iter().map(move |t| (*t, *f))).collect();
    let (declared_by_act, declared_unresolved) = declared(inv, model, opts);
    let classified = classify_types(inv, &ix, model, &type_of_fact, &declared_by_act);
    let acts = model
        .slices
        .iter()
        .map(|s| act_row(&s.address(), &declared_by_act, &classified.declarable, &classified.spanning_by_act))
        .collect();
    DeltaReport {
        proxy: PROXY,
        context: model.context.clone(),
        acts,
        spanning_types: classified.spanning_types,
        declared_unresolved,
        fact_orphans: facts
            .iter()
            .filter(|(f, _)| !model.has_fact(f))
            .flat_map(|(f, ts)| ts.iter().map(move |t| (t.to_string(), f.to_string())))
            .collect(),
        facts_unrealised: model.facts.iter().filter(|f| !facts.contains_key(f.id.as_str())).map(|f| f.id.clone()).collect(),
        ratios: ratios(inv, &ix, opts),
    }
}

#[derive(Default)]
struct Classified {
    declarable: BTreeMap<String, Vec<String>>,
    spanning_by_act: BTreeMap<String, Vec<String>>,
    spanning_types: Vec<SpanningType>,
}

/// Apply the proxy to every non-fact, undeclared type.
fn classify_types(
    inv: &Inventory,
    ix: &Index<'_>,
    model: &EventModel,
    type_of_fact: &BTreeMap<&str, &str>,
    declared_by_act: &BTreeMap<String, Vec<String>>,
) -> Classified {
    let mut out = Classified::default();
    for t in &inv.types {
        if type_of_fact.contains_key(t.id.as_str()) || declared_by_act.values().any(|v| v.contains(&t.id)) {
            continue;
        }
        let (f, produced) = referenced_facts(ix, &t.id, type_of_fact);
        if f.is_empty() {
            continue;
        }
        let covering: Vec<String> = model.slices.iter().filter(|s| f.iter().all(|x| s.facts().contains(x))).map(|s| s.address()).collect();
        if covering.is_empty() {
            let acts_touched = acts_spanned(model, &f, &produced);
            for a in &acts_touched {
                out.spanning_by_act.entry(a.clone()).or_default().push(t.id.clone());
            }
            out.spanning_types.push(SpanningType {
                type_id: t.id.clone(),
                facts: f.iter().map(|s| s.to_string()).collect(),
                acts_touched: acts_touched.into_iter().collect(),
            });
        } else {
            for a in covering {
                out.declarable.entry(a).or_default().push(t.id.clone());
            }
        }
    }
    out
}

/// Act address → declaring symbols, plus the `[Slice]` uses naming no act.
type Declared = (BTreeMap<String, Vec<String>>, Vec<(String, String)>);

/// `[Slice]` declarations: act address → declaring symbols, plus the unresolved ones.
fn declared(inv: &Inventory, model: &EventModel, opts: &DeltaOptions) -> Declared {
    let mut by_act: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut unresolved = Vec::new();
    let uses = inv.types_with_attribute(&opts.slice_attribute).into_iter().map(|(t, a)| (t.id.as_str(), a))
        .chain(inv.members_with_attribute(&opts.slice_attribute).into_iter().flat_map(|m| m.attributes.iter().filter(|a| a.attribute_type == opts.slice_attribute).map(move |a| (m.id.as_str(), a))));
    for (symbol, a) in uses {
        let act = a.arg(0).unwrap_or("");
        let named = model.slices_named(act);
        if named.is_empty() {
            unresolved.push((symbol.to_string(), act.to_string()));
        }
        for s in named {
            by_act.entry(s.address()).or_default().push(symbol.to_string());
        }
    }
    (by_act, unresolved)
}

fn act_row(address: &str, declared: &BTreeMap<String, Vec<String>>, declarable: &BTreeMap<String, Vec<String>>, spanning: &BTreeMap<String, Vec<String>>) -> ActRow {
    let (region, symbols) = if let Some(s) = declared.get(address) {
        (Region::Declared, s.clone())
    } else if let Some(s) = declarable.get(address) {
        (Region::Declarable, s.clone())
    } else if let Some(s) = spanning.get(address) {
        (Region::Unstructured, s.clone())
    } else {
        (Region::Unrealised, Vec::new())
    };
    ActRow { address: address.to_string(), region, symbols }
}

/// The two ratios: undeclared types reachable from declared slices vs not.
fn ratios(inv: &Inventory, ix: &Index<'_>, opts: &DeltaOptions) -> Ratios {
    let declared_root = Root::Declared(opts.slice_attribute.clone());
    let start = root_symbols(inv, ix, &declared_root);
    let reached = closure(ix, &start, opts.through_implementations);
    let is_declared = |id: &str| -> bool {
        ix.types.get(id).is_some_and(|t| t.attributes.iter().any(|a| a.attribute_type == opts.slice_attribute || a.attribute_type == opts.realises_fact_attribute))
    };
    let mut by_ns: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    let (mut total, mut reachable) = (0, 0);
    for t in &inv.types {
        if is_declared(&t.id) {
            continue;
        }
        total += 1;
        let e = by_ns.entry(t.namespace.as_str()).or_insert((0, 0));
        if reached.contains(t.id.as_str()) {
            reachable += 1;
            e.0 += 1;
        } else {
            e.1 += 1;
        }
    }
    Ratios {
        through_implementations: opts.through_implementations,
        undeclared_types: total,
        reachable_undeclared: reachable,
        isolated_undeclared: total - reachable,
        by_namespace: by_ns.into_iter().map(|(ns, (r, i))| (ns.to_string(), r, i)).collect(),
    }
}

/// Text rendering: the proxy first, then acts by region, then the ratios.
pub fn render_delta(report: &DeltaReport) -> String {
    let mut s = format!("context: {}\n", report.context);
    s.push_str(&format!(
        "separator (a declared proxy, CG-R-52):\n  proxy:              {}\n  original predicate: {}\n  known divergence:   {}\n\n",
        report.proxy.proxy, report.proxy.original_predicate, report.proxy.known_divergence
    ));
    for region in [Region::Declared, Region::Declarable, Region::Unstructured, Region::Unrealised] {
        let rows: Vec<&ActRow> = report.acts.iter().filter(|a| a.region == region).collect();
        s.push_str(&format!("{:?}: {} act(s)\n", region, rows.len()));
        for a in rows {
            s.push_str(&format!("  {:<28} {}\n", a.address, a.symbols.join(", ")));
        }
    }
    for st in &report.spanning_types {
        s.push_str(&format!("\nboundary runs through {} — facts {} — acts {}", st.type_id, st.facts.join(", "), st.acts_touched.join(", ")));
    }
    for (sym, act) in &report.declared_unresolved {
        s.push_str(&format!("\nunresolved: {sym} declares act '{act}', which the vocabulary does not name"));
    }
    for (t, f) in &report.fact_orphans {
        s.push_str(&format!("\norphan: {t} realises '{f}', which the vocabulary does not name"));
    }
    if !report.facts_unrealised.is_empty() {
        s.push_str(&format!("\nfacts no type realises: {}", report.facts_unrealised.join(", ")));
    }
    let r = &report.ratios;
    s.push_str(&format!(
        "\n\nratios (through implementations: {}):\n  reachable-undeclared: {} of {} undeclared types\n  isolated-undeclared:  {} of {}\n",
        r.through_implementations, r.reachable_undeclared, r.undeclared_types, r.isolated_undeclared, r.undeclared_types
    ));
    for (ns, reach, iso) in &r.by_namespace {
        s.push_str(&format!("  {:<50} reachable={reach} isolated={iso}\n", if ns.is_empty() { "(global)" } else { ns }));
    }
    s
}

#[cfg(test)]
#[path = "csharp_delta_tests.rs"]
mod tests;
