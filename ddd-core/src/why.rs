//! `why` resolution — from a governed id to its decision chain.
//!
//! Resolves a decision, claim, diagnostic (manifest rule), pattern, or seam
//! id to decision → rationale → principal → basedOn claims, rendered as
//! readable text (PRD §7). Diagnostic ids resolve both fully qualified
//! (`analyzers/CA2007`) and bare (`CA2007`).
//!
//! Resolution is three-way (`dec/ddd/why-resolves-three-ways`): a miss on
//! the store is not necessarily an unknown id. When detection knows the
//! rule but the manifest does not, that is an ungoverned rule, and saying
//! "not found" would read as a missing decision rather than a missing
//! mapping.

use crate::claim::Claim;
use crate::decision::{Decision, DecisionKind};
use crate::detect::DetectedState;
use crate::manifest::{ManifestRule, ManifestSet, UNGOVERNED};
use crate::store::DddStore;

/// The three outcomes of resolving an id.
#[derive(Debug)]
pub enum WhyResolution {
    /// The id resolved; the chain is rendered.
    Resolved(String),
    /// Detection knows the rule but no manifest entry maps it — an escape,
    /// not a lookup failure.
    DetectedUnfiled { namespace: String, rule_id: String, sources: String },
    /// Nothing in the graph or in detection knows this id.
    NotFound,
}

/// Resolve `id` against the store, falling back to `detected` so an
/// ungoverned rule reports as such rather than as an unknown id.
pub fn resolve_why(store: &DddStore, detected: Option<&DetectedState>, id: &str) -> WhyResolution {
    if let Some(text) = render_why(store, id) {
        return WhyResolution::Resolved(text);
    }
    if let Some(d) = detected {
        if let Some((ns, rule_id, sources)) = crate::diff::find_detected(d, id) {
            return WhyResolution::DetectedUnfiled { namespace: ns, rule_id, sources };
        }
    }
    WhyResolution::NotFound
}

/// Render the governance chain behind `id`, or `None` when nothing matches.
/// Both spellings of a migrated id resolve: a `dec:` ledger identity maps
/// back through the permanent concordance (M8 ruling 6), and a historical
/// id that migrated renders with its ledger identity appended.
pub fn render_why(store: &DddStore, id: &str) -> Option<String> {
    let concordance = crate::concordance::load(&store.dir);
    let (id, via) = match &concordance {
        Some(c) if id.starts_with("dec:") => match c.historical_of(id) {
            Some(historical) => (historical, Some(id)),
            None => (id, None),
        },
        _ => (id, None),
    };
    let text = render_by_family(store, id)?;
    let mut out = text;
    if let Some(row) = concordance.as_ref().and_then(|c| c.row(id)) {
        if let Some(ledger) = &row.ledger {
            out.push_str(&format!("\nledger identity: {ledger} (concordance, permanent)"));
        }
        if let Some(d) = &row.disposition {
            out.push_str(&format!("\nconcordance: {d}"));
        }
    }
    if let Some(via) = via {
        out.push_str(&format!("\nresolved from {via} via the concordance"));
    }
    Some(out)
}

fn render_by_family(store: &DddStore, id: &str) -> Option<String> {
    if let Some(d) = find_decision(store, id) {
        return Some(render_decision(store, d, 0));
    }
    if let Some(c) = store.claims.iter().find(|c| c.id == id) {
        return Some(render_claim(c, 0));
    }
    if let Some((set, rule)) = find_manifest_rule(store, id) {
        return Some(render_manifest(store, set, rule));
    }
    if let Some(p) = store.patterns.iter().find(|p| p.id == id) {
        return Some(render_pattern(store, p));
    }
    if let Some(s) = store.seams.iter().find(|s| s.id == id) {
        return Some(render_seam(s));
    }
    None
}

fn find_decision<'a>(store: &'a DddStore, id: &str) -> Option<&'a Decision> {
    store.decisions.iter().find(|d| d.id == id)
}

fn find_manifest_rule<'a>(store: &'a DddStore, id: &str) -> Option<(&'a ManifestSet, &'a ManifestRule)> {
    for set in &store.manifests {
        for rule in &set.file.rules {
            if set.entry_id(rule) == id || rule.rule_id == id {
                return Some((set, rule));
            }
        }
    }
    None
}

fn pad(depth: usize) -> String {
    "  ".repeat(depth)
}

fn render_decision(store: &DddStore, d: &Decision, depth: usize) -> String {
    let p = pad(depth);
    let kind = match d.kind {
        DecisionKind::Decision => "decision",
        DecisionKind::RiskAcceptance => "risk acceptance",
    };
    let mut out = format!("{p}{kind} {} — {}\n", d.id, d.title);
    out.push_str(&format!("{p}  principal: {}\n", d.principal));
    if let Some(date) = &d.date {
        out.push_str(&format!("{p}  date: {date}\n"));
    }
    out.push_str(&format!("{p}  rationale: {}\n", d.rationale.trim()));
    if !d.based_on.is_empty() {
        out.push_str(&format!("{p}  basedOn:\n"));
        for basis in &d.based_on {
            let Some(id) = basis.claim_id() else {
                out.push_str(&render_typed_basis(basis, depth + 2));
                continue;
            };
            match store.claims.iter().find(|c| c.id == id) {
                Some(c) => {
                    out.push_str(&render_claim(c, depth + 2));
                    if let Some(pin) = basis.pin() {
                        if pin.status != c.status || pin.changed != c.changed {
                            out.push_str(&format!(
                                "{p}      pinned at {}@{} — basis has moved (see `ddd report escapes`)\n",
                                pin.status.as_str(),
                                pin.changed
                            ));
                        }
                    }
                }
                None => out.push_str(&format!("{p}    {id} (claim not found)\n")),
            }
        }
    }
    out
}

/// A format-5 non-claim basis: its type, statement, and ref.
fn render_typed_basis(basis: &crate::decision::BasedOn, depth: usize) -> String {
    let p = pad(depth);
    let crate::decision::BasedOn::Typed(t) = basis else {
        return String::new();
    };
    let mut out = format!("{p}basis ({})", t.basis_type.as_str());
    if let Some(s) = t.statement.as_deref().filter(|s| !s.trim().is_empty()) {
        out.push_str(&format!(" — {}", s.trim()));
    }
    out.push('\n');
    if let Some(r) = t.reference.as_deref().filter(|r| !r.trim().is_empty()) {
        out.push_str(&format!("{p}  ref: {r}\n"));
    }
    out
}

fn render_claim(c: &Claim, depth: usize) -> String {
    let p = pad(depth);
    let mut out = format!("{p}claim {} [{}] — {}\n", c.id, c.status.as_str(), c.statement.trim());
    if let Some(f) = &c.falsifier {
        out.push_str(&format!("{p}  falsifier: {}\n", f.trim()));
    }
    if let Some(e) = &c.evidence {
        out.push_str(&format!("{p}  evidence: {}\n", e.trim()));
    }
    out
}

fn render_manifest(store: &DddStore, set: &ManifestSet, rule: &ManifestRule) -> String {
    let mut out =
        format!("diagnostic {} — severity {}\n", set.entry_id(rule), rule.severity);
    if let Some(sup) = &rule.suppression {
        out.push_str("  suppressed");
        match sup.risk_acceptance.as_deref().and_then(|id| find_decision(store, id)) {
            Some(ra) => {
                out.push_str(" — cites:\n");
                out.push_str(&render_decision(store, ra, 1));
            }
            None => out.push_str(" — cites no resolvable risk-acceptance record\n"),
        }
    }
    match rule.decision.as_deref() {
        Some(id) => match find_decision(store, id) {
            Some(d) => {
                out.push_str("  governed by:\n");
                out.push_str(&render_decision(store, d, 1));
            }
            None => out.push_str(&format!("  governed by {id} (decision not found)\n")),
        },
        None if rule.governance.as_deref() == Some(UNGOVERNED) => {
            out.push_str("  UNGOVERNED — no decision filed yet\n");
        }
        None => out.push_str("  maps to no decision (ontology rule 4 violation)\n"),
    }
    out
}

fn render_pattern(store: &DddStore, p: &crate::pattern::PatternInstance) -> String {
    let mut out = format!("pattern {} — instantiates {}\n", p.id, p.pattern_predicate);
    out.push_str(&format!("  instance: {}\n", p.instance));
    for (k, v) in &p.obligations {
        out.push_str(&format!("  obligation {k}: {v}\n"));
    }
    if let Some(id) = &p.decision {
        match find_decision(store, id) {
            Some(d) => {
                out.push_str("  introduced by:\n");
                out.push_str(&render_decision(store, d, 1));
            }
            None => out.push_str(&format!("  introduced by {id} (decision not found)\n")),
        }
    }
    out
}

fn render_seam(s: &crate::seam::SeamDeclaration) -> String {
    let mut out = format!("seam {} — {}\n", s.id, s.boundary);
    out.push_str(&format!("  verdict_knowledge: {}\n", s.verdict_knowledge));
    out.push_str(&format!("  contract_location: {}\n", s.contract_location));
    for o in &s.obligations {
        out.push_str(&format!("  obligation: {o}\n"));
    }
    out
}

#[cfg(test)]
#[path = "why_tests.rs"]
mod tests;
