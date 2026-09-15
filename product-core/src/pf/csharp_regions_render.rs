//! Text rendering of the regions report (Gate 1b, Gate B).
//!
//! Grade and labels first; the acts with their proxy positions; the regions
//! by kind and by project, never a flat list of symbols; then each entry point
//! with its region and reason; the spanning types; the split with its error
//! bound; §12.1's statement.

use super::csharp_candidates_seed::short;
use super::csharp_regions::{Region, RegionsReport};

pub fn render_regions(r: &RegionsReport) -> String {
    let mut s = format!("grade: {}\n", r.grade);
    for l in &r.labels {
        s.push_str(&format!("  · {l}\n"));
    }
    s.push_str(&format!("\nacts (ratified): {}\n", r.acts.len()));
    for a in &r.acts {
        let names = |v: &[String]| if v.is_empty() { "—".to_string() } else { v.iter().map(|x| short(x)).collect::<Vec<_>>().join(", ") };
        s.push_str(&format!("  {:<26} entry points {} · channel [{}] · reads [{}] · writes [{}]\n", a.name, a.entry_points.len(), a.channels.join(", "), names(&a.reads), names(&a.writes)));
    }
    s.push_str(&render_tables(r));
    s.push_str("\nentry points:\n");
    for region in [Region::Declared, Region::Declarable, Region::Unstructured, Region::NoFactsUnderProxy] {
        for e in r.entries.iter().filter(|e| e.region == region) {
            s.push_str(&format!("  [{:<20}] {:<75} {:<20} acts [{}] — {}\n", e.region.label(), e.id, e.disposition, e.acts.join(", "), e.why));
        }
    }
    for st in &r.spanning_types {
        s.push_str(&format!("\nboundary runs through {} — facts [{}] — acts touching [{}]", short(&st.type_id), st.facts.iter().map(|x| short(x)).collect::<Vec<_>>().join(", "), st.acts_touching.join(", ")));
    }
    s.push_str(&format!("\nseparator incidence (CG-R-52 on P-EP-4): {}\n", r.separator_incidence));
    if !r.rows_without_path.is_empty() {
        s.push_str(&format!("\nworksheet rows with no derived path (hand-supplied; outside the regions): {}\n", r.rows_without_path.join("; ")));
    }
    s.push_str(&render_ratios(r));
    s
}

fn render_tables(r: &RegionsReport) -> String {
    let mut s = String::from("\nregions (entry points):\n");
    for (k, n) in &r.by_region {
        s.push_str(&format!("  {k:<22} {n}\n"));
    }
    s.push_str("\nby kind:\n");
    for (k, m) in &r.by_kind_region {
        s.push_str(&format!("  {k:<22} {}\n", m.iter().map(|(a, b)| format!("{a} {b}")).collect::<Vec<_>>().join(" · ")));
    }
    s.push_str("\nby project:\n");
    for (k, m) in &r.by_project_region {
        s.push_str(&format!("  {k:<22} {}\n", m.iter().map(|(a, b)| format!("{a} {b}")).collect::<Vec<_>>().join(" · ")));
    }
    s
}

fn render_ratios(r: &RegionsReport) -> String {
    let x = &r.ratios;
    let mut s = format!(
        "\nsplit over undeclared production types (walk from {} roots of {} accepted entry points, hosts' sites, O-17 off):\n  reachable-undeclared:  {} of {} ({:.1}%)\n  unresolved-undeclared: {} of {}\n  isolated-undeclared:   {} of {} ({:.1}%)\n  error bound (CG-R-89): {} unscored of {} composition edges = {:.1}%\n",
        x.roots, x.accepted_entry_points, x.reachable_undeclared, x.undeclared_types, x.reachable_percent, x.unresolved_undeclared, x.undeclared_types, x.isolated_undeclared, x.undeclared_types, x.isolated_percent, x.unscored, x.composition_edges, x.error_bound_percent
    );
    s.push_str("  by project:\n");
    for (p, a, b, c) in &x.by_project {
        s.push_str(&format!("    {p:<40} reachable={a} unresolved={b} isolated={c}\n"));
    }
    s.push_str("  by namespace:\n");
    for (p, a, b, c) in &x.by_namespace {
        s.push_str(&format!("    {p:<60} reachable={a} unresolved={b} isolated={c}\n"));
    }
    s.push_str(&format!("\n§12.1: {}\n", r.twelve_one.statement));
    s
}
