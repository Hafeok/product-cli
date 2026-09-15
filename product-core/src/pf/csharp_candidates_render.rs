//! Text rendering of the candidate report (Gate 1b, Gate A).
//!
//! Each candidate prints its observed fields, its path figures, the facts
//! proxy, the unfilled ground slots as *not stated* (CG-R-108) and the
//! invited determination empty (CG-R-109). The vocabulary label leads.

use super::csharp_candidates::Candidate;
use super::csharp_candidates_report::{CandidateReport, RecallReport};
use super::csharp_candidates_seed::short;

pub fn render_candidates(r: &CandidateReport) -> String {
    let mut s = format!("vocabulary: {}\ncriterion:  {}\n", r.vocabulary, r.criterion);
    for h in &r.hosts {
        s.push_str(&format!("host: {} — entry point {} ({} registration sites in its own closure)\n", h.project, short(&h.entry_point), h.sites));
    }
    s.push_str(&format!("\nblind spot (CG-R-78): {} razor files, {} @inject directives in production projects\n", r.blind_spot.razor_files, r.blind_spot.razor_inject_directives));
    s.push_str(&format!("\ncandidates: {} (transport-derived)\n", r.count));
    for (k, n) in &r.by_kind {
        s.push_str(&format!("  {k:<20} {n}\n"));
    }
    s.push('\n');
    for c in &r.candidates {
        s.push_str(&render_candidate(c));
    }
    s.push_str(&render_overlaps(r));
    s.push_str(&render_instrument(r));
    if let Some(rc) = &r.recall {
        s.push_str(&render_recall(rc));
    }
    s
}

fn opt(o: &Option<String>) -> &str {
    o.as_deref().unwrap_or("—")
}

fn render_candidate(c: &Candidate) -> String {
    let mut s = format!("[{}] {}.{}  {}  {}\n    path source:  {}\n", c.kind.label(), short(&c.type_id), c.member, c.method, c.path, c.path_source);
    let auth = if c.observed.authorisation.is_empty() { "(none found)".to_string() } else { c.observed.authorisation.join("; ") };
    s.push_str(&format!("    observed:     {auth}; anti-forgery: {}; identity checks: {}", if c.observed.anti_forgery { "yes" } else { "no" }, c.observed.identity_checks.len()));
    if !c.observed.identity_checks.is_empty() {
        s.push_str(&format!(" ({})", c.observed.identity_checks.iter().take(3).cloned().collect::<Vec<_>>().join(", ")));
    }
    if !c.observed.argument_symbols.is_empty() {
        s.push_str(&format!("\n    arg symbols:  {}", c.observed.argument_symbols.iter().map(|x| short(x)).collect::<Vec<_>>().join(", ")));
    }
    let edge = |k: &str| c.path_edges.get(k).copied().unwrap_or(0);
    let total: usize = c.path_edges.values().sum();
    s.push_str(&format!(
        "\n    path:         {} production types; edges resolved {}, unresolved {}, partial {}, boundary {}, registration-not-read {}, excluded {} — unscored {} of {}\n",
        c.path_types.len(),
        edge("resolved"),
        edge("unresolved"),
        edge("partial"),
        edge("boundary"),
        edge("registration-not-read"),
        edge("excluded"),
        c.unscored,
        total
    ));
    let names = |v: &[String]| if v.is_empty() { "—".to_string() } else { v.iter().map(|x| short(x)).collect::<Vec<_>>().join(", ") };
    s.push_str(&format!("    facts P-EP-4: read [{}] written [{}] possibly written [{}] DbSet [{}]\n", names(&c.facts.read), names(&c.facts.written), names(&c.facts.possibly_written), names(&c.facts.dbset_touched)));
    let g = &c.ground;
    s.push_str(&format!(
        "    ground:       expected actor kinds: {} · population: {} · rate: {}   (unfilled — not stated, never no actors, CG-R-108)\n",
        g.expected_actor_kinds.as_ref().map(|v| v.join(", ")).unwrap_or_else(|| "not stated".to_string()),
        g.population_order_of_magnitude.as_deref().unwrap_or("not stated"),
        g.rate_order_of_magnitude.as_deref().unwrap_or("not stated")
    ));
    let t = &c.supported_throughput;
    s.push_str(&format!("    invited:      supported throughput — sustained {}, peak {}, window {}, above the limit {}   (a determination, not authored here, CG-R-109)\n\n", opt(&t.sustained), opt(&t.peak), opt(&t.window), opt(&t.behaviour_above_limit)));
    s
}

fn render_overlaps(r: &CandidateReport) -> String {
    let o = &r.overlaps;
    let mut s = format!("overlaps — the merge signal (CG-R-106), reported not decided:\n  identical path sets: {} group(s)\n", o.identical.len());
    for g in &o.identical {
        s.push_str(&format!("    {}\n", g.join(" | ")));
    }
    s.push_str(&format!("  pairs with Jaccard ≥ 0.5: {} — of which across types: {} (within a type the overlap is structural: constructor injection is per type)\n", o.pairs.len(), o.cross_type_pairs.len()));
    for p in o.cross_type_pairs.iter().take(40) {
        s.push_str(&format!("    {:.2}  {} ~ {}  ({} shared)\n", p.jaccard, p.a, p.b, p.shared));
    }
    if o.cross_type_pairs.len() > 40 {
        s.push_str(&format!("    … {} more across types, and every pair, in JSON\n", o.cross_type_pairs.len() - 40));
    }
    s.push_str("  types reached by two or more candidates (top 25):\n");
    for (t, n) in o.shared_types.iter().take(25) {
        s.push_str(&format!("    {n:>4}  {}\n", short(t)));
    }
    s
}

fn render_instrument(r: &CandidateReport) -> String {
    let mut s = String::from("\nproxies (CG-R-77):\n");
    for p in &r.proxies {
        s.push_str(&format!("  {}: {}\n      framework rule: {}\n      divergence:     {}\n      incidence:      {}\n", p.id, p.proxy, p.framework_rule, p.known_divergence, p.incidence));
    }
    s.push_str("\ninstrument limits bearing on the set (reported, not repaired — CG-R-103):\n");
    for l in &r.limits {
        s.push_str(&format!("  {}: {}\n      incidence: {}\n", l.id, l.limit, l.incidence));
    }
    if !r.excluded.is_empty() {
        s.push_str("\nadmitted by base, no candidate:\n");
        for e in &r.excluded {
            s.push_str(&format!("  {e}\n"));
        }
    }
    s
}

fn render_recall(rc: &RecallReport) -> String {
    let mut s = format!(
        "\nrecall against the hand enumeration (CG-R-71's form): {} of {} found ({:.1}%), {} not visible by stated limit; precision {} of {} derived ({:.1}%)\n",
        rc.matched,
        rc.hand,
        rc.recall_percent,
        rc.not_visible,
        rc.derived - rc.extras.len(),
        rc.derived,
        rc.precision_percent
    );
    for m in &rc.misses {
        s.push_str(&format!("  miss:  {m}\n"));
    }
    for e in &rc.extras {
        s.push_str(&format!("  extra: {e}\n"));
    }
    s
}
