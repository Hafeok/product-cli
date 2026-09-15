//! The candidate report — the set, its overlaps, the proxies with incidence, the limits, the recall (Gate 1b, Gate A).
//!
//! Assembles seeds, paths and overlaps into one report; measures the reader's
//! recall against a hand enumeration of the solution's integration points
//! (CG-R-71's form); prints every proxy with its incidence (CG-R-77) and every
//! instrument limit that bears on the set, beside the Razor blind spot (CG-R-78).
//! Every figure is transport-derived (CG-R-105).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::csharp_candidates::{namespace_tail, Candidate, Kind};
use super::csharp_candidates_path::{fill_path, hosts, overlaps, Host, Overlaps, FACTS_PROXY};
use super::csharp_candidates_seed::{seeds, Seeds};
use super::csharp_di::Resolver;
use super::csharp_inventory::{Index, Inventory};
use super::csharp_reach::BlindSpot;

pub const VOCABULARY: &str = "measurement vocabulary, transport-derived (CG-R-105): good for computing the delta, not for accruing determinations; every figure here is transport-derived; no candidate is an act until someone names it (CG-R-106)";

pub const CRITERION: &str = "meta/sessions/2026-09-15-gate1b-candidates/gateA-criterion.md — E-1 controller actions (base chain reaches ControllerBase; public instance methods without [NonAction]); E-2 Razor page handlers (chain reaches PageModel; On{Verb}[{Handler}][Async], P-EP-1; route from the code-behind path, P-EP-2); E-3 ViewComponents (Invoke/InvokeAsync, P-EP-3); E-4 FastEndpoints (chain reaches FastEndpoints.BaseEndpoint; verb from the configuring call by member id; route not read, L-EP-2); E-5 hosted services (IHostedService/BackgroundService). Test projects excluded. A candidate walks from its handler(s) plus constructors through its host's registration sites, O-17 off";

#[derive(Debug, Clone, Serialize)]
pub struct ProxyIncidence {
    pub id: &'static str,
    pub proxy: &'static str,
    pub framework_rule: &'static str,
    pub known_divergence: &'static str,
    pub incidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Limit {
    pub id: &'static str,
    pub limit: &'static str,
    pub incidence: String,
}

/// A hand enumeration of integration points: `entry_points` the reader
/// should find, `not_visible` those a stated limit hides (counted in the
/// denominator). Keyed by kind, type (`Short` or `Short@NamespaceTail`),
/// member, method; for `fastendpoints` the member is not compared.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EntryPointTruth {
    #[serde(default)]
    pub solution: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub entry_points: Vec<TruthEntry>,
    #[serde(default)]
    pub not_visible: Vec<TruthEntry>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TruthEntry {
    pub kind: String,
    #[serde(rename = "type")]
    pub type_name: String,
    #[serde(default)]
    pub member: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub why: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RecallReport {
    /// Hand-enumerated entries, visible plus not visible — the denominator.
    pub hand: usize,
    pub not_visible: usize,
    pub derived: usize,
    pub matched: usize,
    pub recall_percent: f64,
    pub precision_percent: f64,
    pub misses: Vec<String>,
    pub extras: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CandidateReport {
    pub vocabulary: &'static str,
    pub criterion: &'static str,
    pub hosts: Vec<Host>,
    pub count: usize,
    pub by_kind: BTreeMap<String, usize>,
    pub candidates: Vec<Candidate>,
    pub overlaps: Overlaps,
    pub proxies: Vec<ProxyIncidence>,
    pub limits: Vec<Limit>,
    pub blind_spot: BlindSpot,
    pub excluded: Vec<String>,
    pub recall: Option<RecallReport>,
}

/// Derive the candidate set.
pub fn candidates(inv: &Inventory, truth: Option<&EntryPointTruth>) -> CandidateReport {
    let ix = inv.index();
    let resolver = Resolver::build(inv);
    let h = hosts(inv, &ix, &resolver);
    let mut s = seeds(inv, &ix);
    for c in &mut s.candidates {
        fill_path(&ix, &resolver, &h, c);
    }
    s.candidates.sort_by(|a, b| a.kind.cmp(&b.kind).then(a.id.cmp(&b.id)));
    let mut by_kind = BTreeMap::new();
    for c in &s.candidates {
        *by_kind.entry(c.kind.label().to_string()).or_insert(0) += 1;
    }
    let blind_spot = inv.projects.iter().filter(|p| !p.is_test()).fold(BlindSpot::default(), |b, p| BlindSpot { razor_files: b.razor_files + p.razor_files, razor_inject_directives: b.razor_inject_directives + p.razor_inject_directives });
    let recall = truth.map(|t| recall(&ix, t, &s.candidates));
    let limits = limits(inv, &s, &blind_spot);
    CandidateReport {
        vocabulary: VOCABULARY,
        criterion: CRITERION,
        hosts: h.rows,
        count: s.candidates.len(),
        by_kind,
        overlaps: overlaps(&s.candidates),
        proxies: proxies(&s),
        limits,
        blind_spot,
        excluded: s.excluded,
        recall,
        candidates: s.candidates,
    }
}

fn proxies(s: &Seeds) -> Vec<ProxyIncidence> {
    vec![
        ProxyIncidence {
            id: "P-EP-1",
            proxy: "a public method on a PageModel named On{Verb}[{Handler}][Async] is a handler",
            framework_rule: "ASP.NET Core Razor Pages handler discovery is this name rule",
            known_divergence: "a public helper named OnXxx reads as a handler; a handler with a non-standard name is invisible",
            incidence: format!("{} public PageModel methods match the rule, {} do not", s.handler_rule.0, s.handler_rule.1),
        },
        ProxyIncidence {
            id: "P-EP-2",
            proxy: "the page route is the code-behind's path under Pages/ (or Areas/<A>/Pages/) without .cshtml.cs",
            framework_rule: "ASP.NET Core Razor Pages file-path routing",
            known_divergence: "a @page \"template\" directive appends to or replaces the route; it lives in the .cshtml the reader does not read (CG-R-78)",
            incidence: "not measurable by the instrument; measured by hand per solution in the gate report".to_string(),
        },
        ProxyIncidence {
            id: "P-EP-3",
            proxy: "a public Invoke / InvokeAsync on a ViewComponent is its entry",
            framework_rule: "ASP.NET Core ViewComponent invocation",
            known_divergence: "none known",
            incidence: format!("{} ViewComponents with Invoke/InvokeAsync, {} without", s.invoke_rule.0, s.invoke_rule.1),
        },
        ProxyIncidence { id: "P-EP-4", proxy: FACTS_PROXY, framework_rule: "none — a reading of persistence abstractions", known_divergence: "stated in the proxy", incidence: "unmeasured — first use; no ground truth over facts exists".to_string() },
    ]
}

fn limits(inv: &Inventory, s: &Seeds, blind: &BlindSpot) -> Vec<Limit> {
    let builder_calls = inv.references.iter().filter(|r| r.to.starts_with("M:Microsoft.AspNetCore.Builder.")).count();
    let fe = s.candidates.iter().filter(|c| c.kind == Kind::FastEndpoints).count();
    vec![
        Limit {
            id: "L-EP-1",
            limit: "calls to Microsoft.AspNetCore.Builder.* extension members are absent from the reference graph: MapControllerRoute, MapRazorPages, MapHealthChecks, MapFallbackToFile, UseFastEndpoints — endpoints mapped there are invisible, and so are the routing conventions they carry",
            incidence: format!("{builder_calls} references to M:Microsoft.AspNetCore.Builder.* in {}", inv.references.len()),
        },
        Limit {
            id: "L-EP-2",
            limit: "call arguments are not emitted (emission rule 5): FastEndpoints routes, FastEndpoints role/scheme strings, Razor Pages authorisation conventions",
            incidence: format!("{fe} of {fe} FastEndpoints routes unread"),
        },
        Limit {
            id: "L-EP-3",
            limit: "Razor is not read (CG-R-78): custom @page templates, view-side component invocation, @inject",
            incidence: format!("{} razor files, {} @inject directives in production projects", blind.razor_files, blind.razor_inject_directives),
        },
    ]
}

fn matches(ix: &Index<'_>, e: &TruthEntry, c: &Candidate) -> bool {
    let (short, tail) = e.type_name.split_once('@').map(|(a, b)| (a, Some(b))).unwrap_or((e.type_name.as_str(), None));
    let Some(t) = ix.types.get(c.type_id.as_str()) else { return false };
    e.kind == c.kind.label() && short == t.name && tail.is_none_or(|s| s == namespace_tail(t)) && e.method == c.method && (c.kind == Kind::FastEndpoints || e.member == c.member)
}

/// Reader recall over the hand enumeration; precision over the derived set.
pub fn recall(ix: &Index<'_>, truth: &EntryPointTruth, cands: &[Candidate]) -> RecallReport {
    let mut matched = 0;
    let mut misses = Vec::new();
    for e in &truth.entry_points {
        if cands.iter().any(|c| matches(ix, e, c)) {
            matched += 1;
        } else {
            misses.push(format!("{}:{}.{}#{} ({})", e.kind, e.type_name, e.member, e.method, e.path));
        }
    }
    misses.extend(truth.not_visible.iter().map(|e| format!("{}:{}.{}#{} ({}) — not visible: {}", e.kind, e.type_name, e.member, e.method, e.path, e.why)));
    let extras: Vec<String> = cands.iter().filter(|c| !truth.entry_points.iter().any(|e| matches(ix, e, c))).map(|c| c.id.clone()).collect();
    let hand = truth.entry_points.len() + truth.not_visible.len();
    let pct = |n: usize, d: usize| if d == 0 { 100.0 } else { 100.0 * n as f64 / d as f64 };
    RecallReport {
        hand,
        not_visible: truth.not_visible.len(),
        derived: cands.len(),
        matched,
        recall_percent: pct(matched, hand),
        precision_percent: pct(cands.len() - extras.len(), cands.len()),
        misses,
        extras,
    }
}
