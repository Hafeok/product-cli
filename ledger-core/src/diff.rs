//! Semantic diff between two readings of the store (PRD §6, L3).
//!
//! Decision-level change, never textual: which sets were declared, which
//! floors moved, which decisions were added, which tips moved and how
//! (fast-forward along the parent DAG, or divergence), what was superseded,
//! which acceptances were gained, lost or revoked. Textual diff on YAML is
//! noise (PRD §3.4); this is the reading a reviewer actually needs.
//!
//! Both sides are whole [`Store`]s — usually one loaded at a git revision
//! ([`crate::revision::load_at`]) and one from disk — so the same
//! comparison serves `ledger diff <ref>..<ref>` and the merge machinery's
//! per-decision base.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::store::Store;
use crate::verify::view::View;

/// How one decision's tip moved between the two readings.
#[derive(Debug, Clone, Serialize)]
pub struct TipMovement {
    pub decision: String,
    pub from: String,
    pub to: String,
    /// Whether `to` descends from `from` along the parent chain. A tip that
    /// does not is a divergence — two histories, not a revision.
    pub fast_forward: bool,
    /// Allocation transition, when the move changed it (e.g. an escape).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation: Option<String>,
}

/// One acceptance's presence changing between the readings.
#[derive(Debug, Clone, Serialize)]
pub struct AcceptanceDelta {
    pub acceptance: String,
    pub decision: String,
    pub actor: String,
    /// Short form of the hash the acceptance signs.
    pub signs: String,
}

/// The decision-level change between two readings of the store.
#[derive(Debug, Default, Serialize)]
pub struct DiffReport {
    pub from: String,
    pub to: String,
    /// Sets declared in `to` that `from` does not know.
    pub sets_declared: Vec<String>,
    /// `set: T-from -> T-to` floor movements.
    pub floors_changed: Vec<String>,
    /// Decisions in `to` that `from` does not know, with their set.
    pub decisions_added: Vec<String>,
    /// Decisions in `from` that `to` does not know — representable across
    /// branches, and worth stating rather than hiding.
    pub decisions_removed: Vec<String>,
    pub tips_moved: Vec<TipMovement>,
    /// `old <- by` supersessions present in `to` only.
    pub superseded: Vec<String>,
    /// Decisions whose chain is forked in `to`.
    pub forked: Vec<String>,
    pub acceptances_gained: Vec<AcceptanceDelta>,
    pub acceptances_lost: Vec<AcceptanceDelta>,
    /// Acceptances revoked in `to` but live in `from`.
    pub revoked: Vec<String>,
}

impl DiffReport {
    pub fn is_empty(&self) -> bool {
        self.sets_declared.is_empty()
            && self.floors_changed.is_empty()
            && self.decisions_added.is_empty()
            && self.decisions_removed.is_empty()
            && self.tips_moved.is_empty()
            && self.superseded.is_empty()
            && self.forked.is_empty()
            && self.acceptances_gained.is_empty()
            && self.acceptances_lost.is_empty()
            && self.revoked.is_empty()
    }
}

/// Compare two readings, labelled for the report.
pub fn diff(from_label: &str, from: &Store, to_label: &str, to: &Store) -> DiffReport {
    let from_view = View::build(from);
    let to_view = View::build(to);
    let mut report = DiffReport {
        from: from_label.to_string(),
        to: to_label.to_string(),
        ..DiffReport::default()
    };
    diff_sets(&mut report, from, to);
    diff_decisions(&mut report, &from_view, &to_view);
    diff_acceptances(&mut report, &from_view, &to_view);
    report
}

fn diff_sets(report: &mut DiffReport, from: &Store, to: &Store) {
    for set in &to.sets {
        match from.sets.iter().find(|s| s.id == set.id) {
            None => report.sets_declared.push(format!("{} (floor {})", set.id, set.tolerance_floor)),
            Some(old) if old.tolerance_floor != set.tolerance_floor => {
                report.floors_changed.push(format!(
                    "{}: {} -> {}",
                    set.id, old.tolerance_floor, set.tolerance_floor
                ));
            }
            Some(_) => {}
        }
    }
}

fn diff_decisions(report: &mut DiffReport, from: &View, to: &View) {
    for (decision, &i) in &to.latest {
        let tip = &to.versions[i];
        let Some(&j) = from.latest.get(decision) else {
            let set = tip.parsed.as_ref().map(|p| p.set.as_str()).unwrap_or("?");
            report.decisions_added.push(format!("{decision} to `{set}`"));
            continue;
        };
        let old_tip = &from.versions[j];
        if old_tip.raw.hash != tip.raw.hash {
            report.tips_moved.push(TipMovement {
                decision: decision.clone(),
                from: old_tip.raw.hash.short().to_string(),
                to: tip.raw.hash.short().to_string(),
                fast_forward: descends(to, decision, &tip.raw.hash, &old_tip.raw.hash),
                allocation: allocation_transition(old_tip, tip),
            });
        }
    }
    for decision in from.latest.keys() {
        if !to.latest.contains_key(decision) {
            report.decisions_removed.push(decision.clone());
        }
    }
    let from_super = crate::verify::state::supersession(from);
    for (old, by) in crate::verify::state::supersession(to) {
        if !from_super.contains_key(&old) {
            report.superseded.push(format!("{old} by {by}"));
        }
    }
    report.forked = to.forked.keys().cloned().collect();
}

/// Whether `descendant` reaches `ancestor` by walking `parent` links among
/// the decision's versions in `view`. Bounded by the version count, so a
/// corrupted cycle terminates.
pub fn descends(
    view: &View,
    decision: &str,
    descendant: &crate::hash::VersionHash,
    ancestor: &crate::hash::VersionHash,
) -> bool {
    let by_hash: BTreeMap<&crate::hash::VersionHash, &crate::version::VersionRaw> = view
        .versions
        .iter()
        .filter(|v| v.raw.decision.to_string() == decision)
        .map(|v| (&v.raw.hash, v.raw))
        .collect();
    let mut cursor = Some(descendant);
    for _ in 0..=by_hash.len() {
        let Some(hash) = cursor else { return false };
        if hash == ancestor {
            return true;
        }
        cursor = by_hash.get(hash).and_then(|raw| raw.parent.as_ref());
    }
    false
}

fn allocation_transition(
    old: &crate::verify::view::ViewedVersion<'_>,
    new: &crate::verify::view::ViewedVersion<'_>,
) -> Option<String> {
    let name = |v: &crate::verify::view::ViewedVersion<'_>| {
        v.raw.allocation.map(|k| k.as_str()).unwrap_or("unallocated").to_string()
    };
    let (before, after) = (name(old), name(new));
    (before != after).then(|| format!("{before} -> {after}"))
}

fn diff_acceptances(report: &mut DiffReport, from: &View, to: &View) {
    let ids = |view: &View| -> BTreeSet<String> {
        view.acceptances.iter().map(|a| a.acceptance.id.to_string()).collect()
    };
    let (from_ids, to_ids) = (ids(from), ids(to));
    for viewed in &to.acceptances {
        if !from_ids.contains(&viewed.acceptance.id.to_string()) {
            report.acceptances_gained.push(delta(viewed));
        }
    }
    for viewed in &from.acceptances {
        if !to_ids.contains(&viewed.acceptance.id.to_string()) {
            report.acceptances_lost.push(delta(viewed));
        }
    }
    for id in &to.revoked {
        if !from.revoked.contains(id) && from_ids.contains(id) {
            report.revoked.push(id.clone());
        }
    }
    report.acceptances_gained.sort_by(|a, b| a.acceptance.cmp(&b.acceptance));
    report.acceptances_lost.sort_by(|a, b| a.acceptance.cmp(&b.acceptance));
}

fn delta(viewed: &crate::verify::view::ViewedAcceptance<'_>) -> AcceptanceDelta {
    AcceptanceDelta {
        acceptance: viewed.acceptance.id.to_string(),
        decision: viewed.acceptance.decision.to_string(),
        actor: viewed.acceptance.actor.to_string(),
        signs: viewed.acceptance.version.short().to_string(),
    }
}

/// The terminal rendering: one line per fact, nothing textual.
pub fn render(report: &DiffReport) -> String {
    let mut lines = vec![format!("ledger diff {}..{}", report.from, report.to)];
    if report.is_empty() {
        lines.push("no decision-level change".to_string());
        return lines.join("\n");
    }
    section(&mut lines, "sets declared", &report.sets_declared);
    section(&mut lines, "floors changed", &report.floors_changed);
    section(&mut lines, "decisions added", &report.decisions_added);
    section(&mut lines, "decisions removed", &report.decisions_removed);
    for m in &report.tips_moved {
        let kind = if m.fast_forward { "fast-forward" } else { "DIVERGED" };
        let alloc = m.allocation.as_deref().map(|a| format!(", {a}")).unwrap_or_default();
        lines.push(format!("revised {}: {} -> {} ({kind}{alloc})", m.decision, m.from, m.to));
    }
    section(&mut lines, "superseded", &report.superseded);
    section(&mut lines, "forked in the target — awaiting arbitration", &report.forked);
    for a in &report.acceptances_gained {
        lines.push(format!("acceptance gained: {} signed {} of {}", a.actor, a.signs, a.decision));
    }
    for a in &report.acceptances_lost {
        lines.push(format!("acceptance lost: {} on {} (signed {})", a.acceptance, a.decision, a.signs));
    }
    section(&mut lines, "revoked", &report.revoked);
    lines.join("\n")
}

fn section(lines: &mut Vec<String>, title: &str, items: &[String]) {
    for item in items {
        lines.push(format!("{title}: {item}"));
    }
}

#[path = "diff_tests.rs"]
#[cfg(test)]
mod tests;
