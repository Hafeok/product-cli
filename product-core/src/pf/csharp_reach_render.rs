//! Text rendering of the reachability report — the convention first, coverage next, every figure beside its unresolved count.

use super::csharp_reach::{Disposition, ReachReport, Resolution};

fn row(d: &Disposition) -> String {
    format!(
        "{:<52} types={:<5} reached={:<5} unresolved={:<4} partial={:<4} unreached={}\n",
        d.label, d.types, d.reached, d.unresolved, d.partial, d.unreached
    )
}

pub fn render_reach(report: &ReachReport) -> String {
    let mut s = format!("roots: {}\n", report.roots.join(", "));
    let t = &report.total;
    s.push_str(&format!(
        "reached: {} of {} types ({:.1}%) — unresolved: {} — partial: {} — unreached: {}\n",
        t.reached, t.types, report.percent_reached, t.unresolved, t.partial, t.unreached
    ));
    render_resolution(report, &mut s);
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
    render_edges(&report.resolution, &mut s);
    s
}

/// Coverage with every count beside it, then roles, reasons, ignored calls, proxies.
fn render_resolution(report: &ReachReport, s: &mut String) {
    let r = &report.resolution;
    s.push_str(&format!(
        "resolution coverage: {} resolved of {} scored composition edges ({:.1}%) — {} unresolved — {} partial (factory/provider, not divided) — {} boundary (external abstractions the library satisfies, outside the denominator)\n  composition edges: {} — in denominator: {} — boundary: {} — excluded by role: {}\n  registrations read: {} — IServiceCollection calls ignored: {}\n",
        r.resolved, r.resolved + r.unresolved, r.coverage_percent, r.unresolved, r.partial, r.boundary,
        r.composition_edges, r.in_denominator, r.boundary, r.composition_edges - r.in_denominator - r.boundary,
        report.registrations_read, report.calls_ignored
    ));
    for (role, n) in &r.by_role {
        s.push_str(&format!("  edges by role:        {role:<26} {n}\n"));
    }
    for (reason, n) in &r.by_reason {
        s.push_str(&format!("  unresolved by reason: {reason:<26} {n}\n"));
    }
    let mut ignored: Vec<(&String, &usize)> = report.ignored_by_method.iter().collect();
    ignored.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (method, n) in ignored.iter().take(12) {
        s.push_str(&format!("  ignored call:         {method:<26} {n}\n"));
    }
    s.push_str("\nrole proxies (each a proxy for the criterion, with its divergence):\n");
    for p in r.proxies {
        s.push_str(&format!("  {:<18} proxy: {}\n  {:<18} divergence: {}\n", p.role.label(), p.proxy, "", p.known_divergence));
    }
}

/// The boundary surface and the unresolved edges, by name.
fn render_edges(r: &Resolution, s: &mut String) {
    if !r.boundary_surface.is_empty() {
        s.push_str(&format!("\nboundary — the used library surface (first {} of {} external types; each needs declaring at member level):\n", r.boundary_surface.len().min(40), r.boundary_surface.len()));
        for b in r.boundary_surface.iter().take(40) {
            s.push_str(&format!("  {:<70} {:<44} {:<18} edges={}\n", b.target, b.assembly, b.role, b.edges));
        }
    }
    if !r.unresolved_edges.is_empty() {
        s.push_str(&format!("\nunresolved edges (first {} of {}):\n", r.unresolved_edges.len().min(25), r.unresolved_edges.len()));
        for u in r.unresolved_edges.iter().take(25) {
            s.push_str(&format!("  {:<18} {:?} {} -> {}\n", u.role.label(), u.state, u.from, u.target));
        }
    }
}
