//! The pre-merge reading: what a merge with another revision would do,
//! decision by decision, before anything is written. Mechanical cases are
//! listed as such; every conflict is surfaced by class; the overtaken
//! acceptances are named so nobody discovers after the fact that their
//! signature became history.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::diff::descends;
use crate::store::Store;
use crate::verify::state;
use crate::verify::view::View;

use super::ConflictClass;

/// One conflict the plan surfaces. Nothing here is resolved — the plan is
/// a reading, and the arbitration belongs to `ledger merge --resolve`.
#[derive(Debug, Clone, Serialize)]
pub struct PlannedConflict {
    pub class: ConflictClass,
    pub subject: String,
    pub detail: String,
}

/// What a merge with the other side would do.
#[derive(Debug, Default, Serialize)]
pub struct MergePlan {
    pub ours: String,
    pub theirs: String,
    pub base: String,
    /// Cases that merge without judgment, one line each.
    pub mechanical: Vec<String>,
    pub conflicts: Vec<PlannedConflict>,
    /// Facts worth knowing that require no arbitration — an acceptance the
    /// other side revised past awaits a fresh signature, per the law that
    /// acceptance never follows content.
    pub notices: Vec<String>,
}

/// Read the three stores into a plan. `base` is the merge base's store —
/// empty when the histories share nothing.
pub fn plan(labels: [&str; 3], ours: &Store, theirs: &Store, base: &Store) -> MergePlan {
    let (ours_view, theirs_view, base_view) =
        (View::build(ours), View::build(theirs), View::build(base));
    let mut out = MergePlan {
        ours: labels[0].to_string(),
        theirs: labels[1].to_string(),
        base: labels[2].to_string(),
        ..MergePlan::default()
    };
    plan_decisions(&mut out, &ours_view, &theirs_view, &base_view);
    plan_supersessions(&mut out, &ours_view, &theirs_view);
    plan_sets(&mut out, ours, theirs, base);
    plan_acceptances(&mut out, &ours_view, &theirs_view);
    out
}

fn plan_decisions(out: &mut MergePlan, ours: &View, theirs: &View, base: &View) {
    let all: BTreeSet<&String> = ours.latest.keys().chain(theirs.latest.keys()).collect();
    for decision in all {
        let o = ours.latest.get(decision).and_then(|&i| ours.versions.get(i));
        let t = theirs.latest.get(decision).and_then(|&i| theirs.versions.get(i));
        match (o, t) {
            (Some(o), None) => out.mechanical.push(format!("{decision}: ours only ({})", o.raw.hash.short())),
            (None, Some(t)) => out
                .mechanical
                .push(format!("{decision}: theirs only ({}) — merges in as filed", t.raw.hash.short())),
            (Some(o), Some(t)) if o.raw.hash == t.raw.hash => {
                out.mechanical.push(format!("{decision}: identical on both sides"));
            }
            (Some(o), Some(t)) => plan_moved(out, decision, ours, theirs, base, o, t),
            (None, None) => {}
        }
    }
}

fn plan_moved(
    out: &mut MergePlan,
    decision: &str,
    ours: &View,
    theirs: &View,
    base: &View,
    o: &crate::verify::view::ViewedVersion<'_>,
    t: &crate::verify::view::ViewedVersion<'_>,
) {
    if descends(theirs, decision, &t.raw.hash, &o.raw.hash) {
        out.mechanical.push(format!(
            "{decision}: theirs fast-forwards ({} -> {})",
            o.raw.hash.short(),
            t.raw.hash.short()
        ));
        return;
    }
    if descends(ours, decision, &o.raw.hash, &t.raw.hash) {
        out.mechanical.push(format!(
            "{decision}: ours is ahead ({} -> {})",
            t.raw.hash.short(),
            o.raw.hash.short()
        ));
        return;
    }
    let class = if super::conflicts::allocation_name(o.raw) != super::conflicts::allocation_name(t.raw)
    {
        ConflictClass::DivergentAllocation
    } else {
        ConflictClass::DivergentRevision
    };
    let parent = base
        .latest
        .get(decision)
        .and_then(|&i| base.versions.get(i))
        .map(|b| b.raw.hash.short().to_string())
        .unwrap_or_else(|| "no common version".to_string());
    out.conflicts.push(PlannedConflict {
        class,
        subject: decision.to_string(),
        detail: format!(
            "ours {} (\"{}\", {}) vs theirs {} (\"{}\", {}); common parent {parent}",
            o.raw.hash.short(),
            o.raw.statement,
            super::conflicts::allocation_name(o.raw),
            t.raw.hash.short(),
            t.raw.statement,
            super::conflicts::allocation_name(t.raw),
        ),
    });
}

fn plan_supersessions(out: &mut MergePlan, ours: &View, theirs: &View) {
    let ours_map = state::supersession(ours);
    for (target, their_claim) in state::supersession(theirs) {
        let Some(our_claim) = ours_map.get(&target) else { continue };
        if *our_claim != their_claim {
            out.conflicts.push(PlannedConflict {
                class: ConflictClass::CompetingSupersession,
                subject: target.clone(),
                detail: format!("ours says {our_claim} supersedes it; theirs says {their_claim}"),
            });
        }
    }
}

fn plan_sets(out: &mut MergePlan, ours: &Store, theirs: &Store, base: &Store) {
    for set in &theirs.sets {
        let ours_set = ours.sets.iter().find(|s| s.id == set.id);
        let base_set = base.sets.iter().find(|s| s.id == set.id);
        match (ours_set, base_set) {
            (None, _) => out.mechanical.push(format!("set `{}`: theirs only — declared by merge", set.id)),
            (Some(o), _) if o == set => {}
            (Some(o), Some(b)) if b == set => {
                out.mechanical.push(format!("set `{}`: ours' change stands", o.id));
            }
            (Some(o), Some(b)) if b == o => {
                out.mechanical.push(format!("set `{}`: theirs' change stands", o.id));
            }
            (Some(o), _) => out.conflicts.push(PlannedConflict {
                class: ConflictClass::DivergentFloor,
                subject: set.id.clone(),
                detail: format!(
                    "both sides changed the set (ours floor {}, theirs floor {})",
                    o.tolerance_floor, set.tolerance_floor
                ),
            }),
        }
    }
}

/// Live acceptances one side holds that the other side's chain moved past:
/// after the merge they sign history, and the tip awaits a fresh signature.
fn plan_acceptances(out: &mut MergePlan, ours: &View, theirs: &View) {
    overtaken(out, ours, theirs, "ours", "theirs");
    overtaken(out, theirs, ours, "theirs", "ours");
}

fn overtaken(out: &mut MergePlan, side: &View, other: &View, side_name: &str, other_name: &str) {
    for viewed in &side.acceptances {
        let a = viewed.acceptance;
        if side.is_revoked(a) || !side.signs_latest(a) {
            continue;
        }
        let decision = a.decision.to_string();
        let Some(their_tip) = other.latest.get(&decision).and_then(|&i| other.versions.get(i))
        else {
            continue;
        };
        if their_tip.raw.hash != a.version
            && descends(other, &decision, &their_tip.raw.hash, &a.version)
        {
            out.notices.push(format!(
                "{}'s acceptance by {} of {} on {decision} is overtaken by {}'s revision {} — the reconciled tip awaits fresh acceptance",
                side_name,
                a.actor,
                a.version.short(),
                other_name,
                their_tip.raw.hash.short()
            ));
        }
    }
}

/// The terminal rendering of a plan.
pub fn render(plan: &MergePlan) -> String {
    let mut lines = vec![format!(
        "merge plan: {} <- {} (base {})",
        plan.ours, plan.theirs, plan.base
    )];
    for line in &plan.mechanical {
        lines.push(format!("  mechanical: {line}"));
    }
    for c in &plan.conflicts {
        lines.push(format!("  CONFLICT [{}] {}: {}", c.class, c.subject, c.detail));
    }
    for n in &plan.notices {
        lines.push(format!("  notice: {n}"));
    }
    match plan.conflicts.len() {
        0 => lines.push("no judgment required — the merge is mechanical".to_string()),
        n => lines.push(format!(
            "{n} conflict(s) require arbitration — merge, then run `ledger merge --resolve`"
        )),
    }
    lines.join("\n")
}
