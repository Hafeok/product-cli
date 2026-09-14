//! Text rendering of the reachability report — the convention first, coverage with its population next, every figure beside its unresolved count.

use super::csharp_reach::{Disposition, ReachReport};
use super::csharp_resolution::{BoundaryRow, Resolution};

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
        "reached: {} of {} production types ({:.1}%) — unresolved: {} — partial: {} — unreached: {}\n",
        t.reached, t.types, report.percent_reached, t.unresolved, t.partial, t.unreached
    ));
    render_resolution(report, &mut s);
    if let Some(g) = &report.ground_truth {
        s.push_str(&format!(
            "ground truth ({}, {} edges{}): reader recall {}/{} ({:.1}%) — walk recall {}/{} ({:.1}%) — walk precision {}/{} ({:.1}%)\n",
            g.project, g.edges, if g.source.is_empty() { String::new() } else { format!(", {}", g.source) },
            g.reader_present, g.edges, g.reader_recall_percent, g.walk_hits, g.edges, g.walk_recall_percent, g.walk_hits, g.walk_edges, g.walk_precision_percent
        ));
        for e in &g.reader_missing {
            s.push_str(&format!("  reader missed:  {} -> {}\n", e.from, e.target));
        }
        for e in &g.walk_missing {
            s.push_str(&format!("  walk missed:    {} -> {}\n", e.from, e.target));
        }
        for e in &g.walk_extra {
            s.push_str(&format!("  walk extra:     {} -> {}\n", e.from, e.target));
        }
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
    if !report.test_projects.is_empty() {
        s.push_str(&format!("\ntest projects, excluded from the primary convention (CG-R-75): {}\n  {}", report.test_projects.join(", "), row(&report.test_project_types)));
    }
    render_edges(&report.resolution, &mut s);
    s
}

/// Coverage with its population, the disposition of every other state, roles, reasons, calls, proxies.
fn render_resolution(report: &ReachReport, s: &mut String) {
    let r = &report.resolution;
    s.push_str(&format!(
        "resolution coverage: {}/{} of the denominator ({:.1}%), {}/{} of composition edges ({:.1}%)\n  unresolved: {} — partial: {} (held: composition chooses the factory, the factory chooses later; never divided) — boundary: {} (the library supplies it) — registration-not-read: {} (a framework call the resolver does not parse supplies it) — excluded by role: {}\n  registrations read: {} — calls ignored: {} — registration sites in test projects skipped: {}\n",
        r.resolved, r.scored, r.coverage_percent, r.scored, r.composition_edges, r.population_percent,
        r.unresolved, r.partial, r.boundary, r.registration_not_read, r.excluded,
        report.registrations_read, report.calls_ignored, report.test_sites_skipped
    ));
    for (role, n) in &r.by_role {
        s.push_str(&format!("  edges by role:        {role:<26} {n}\n"));
    }
    for (reason, n) in &r.by_reason {
        s.push_str(&format!("  unresolved by reason: {reason:<26} {n}\n"));
    }
    for (call, n) in &r.by_provider {
        s.push_str(&format!("  not read, registered by: {call:<23} {n}\n"));
    }
    let mut ext: Vec<(&String, &usize)> = report.ignored_external.iter().collect();
    ext.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (method, n) in ext.iter().take(15) {
        s.push_str(&format!("  ignored external call: {method:<23} {n}\n"));
    }
    s.push_str(&format!("  ignored calls to the solution's own extension methods (bodies walked): {}\n", report.ignored_in_solution.values().sum::<usize>()));
    if !report.unlearned_calls.is_empty() {
        s.push_str(&format!("  reached external calls the resolver neither parses nor knows ({} — what it has to learn):\n", report.unlearned_calls.len()));
        for (m, n) in &report.unlearned_calls {
            s.push_str(&format!("    {m} ×{n}\n"));
        }
    }
    s.push_str("\nrole proxies (each a proxy for the criterion, with its divergence):\n");
    for p in r.proxies {
        s.push_str(&format!("  {:<18} proxy: {}\n  {:<18} divergence: {}\n", p.role.label(), p.proxy, "", p.known_divergence));
    }
}

fn surface(title: &str, rows: &[BoundaryRow], s: &mut String) {
    if rows.is_empty() {
        return;
    }
    s.push_str(&format!("\n{title} (first {} of {} external types):\n", rows.len().min(40), rows.len()));
    for b in rows.iter().take(40) {
        s.push_str(&format!("  {:<70} {:<44} {:<18} edges={}{}\n", b.target, b.assembly, b.role, b.edges, b.call.as_ref().map(|c| format!(" via {c}")).unwrap_or_default()));
    }
}

/// The two surfaces and the unresolved edges, by name.
fn render_edges(r: &Resolution, s: &mut String) {
    surface("boundary — the used library surface; each needs declaring at member level", &r.boundary_surface, s);
    surface("registration-not-read — supplied by a framework call the resolver does not parse", &r.registration_not_read_surface, s);
    if !r.unresolved_edges.is_empty() {
        s.push_str(&format!("\nunresolved edges (first {} of {}):\n", r.unresolved_edges.len().min(25), r.unresolved_edges.len()));
        for u in r.unresolved_edges.iter().take(25) {
            s.push_str(&format!("  {:<18} {:?} {} -> {}\n", u.role.label(), u.state, u.from, u.target));
        }
    }
}
