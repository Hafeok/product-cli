//! Finding the conflicts a merged store carries, with enough context to
//! present each one clearly: both chains, the common parent, who filed
//! what, and which acceptances ride on each side. `--resolve` refuses to
//! proceed on anything this module cannot present.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::hash::VersionHash;
use crate::store::Store;
use crate::verify::view::View;
use crate::version::VersionRaw;

use super::ConflictClass;

/// One tip of a forked chain, presented.
#[derive(Debug, Clone, Serialize)]
pub struct TipView {
    pub hash: VersionHash,
    pub statement: String,
    pub allocation: String,
    /// Who performed the act that filed this tip, and when.
    pub filed_by: String,
    /// Actors holding an unrevoked acceptance of this exact tip.
    pub accepted_by: Vec<String>,
}

/// A forked decision, ready to present.
#[derive(Debug, Clone, Serialize)]
pub struct Fork {
    pub decision: String,
    pub class: ConflictClass,
    /// The fork point, when the chains share one: `hash — statement`.
    pub common_parent: Option<String>,
    pub tips: Vec<TipView>,
}

/// One decision superseded by two live claimants (`G005`), to present.
#[derive(Debug, Clone, Serialize)]
pub struct SupersessionConflict {
    pub target: String,
    pub claimants: Vec<ClaimView>,
}

/// One claimant in a competing supersession.
#[derive(Debug, Clone, Serialize)]
pub struct ClaimView {
    pub decision: String,
    pub tip: VersionHash,
    pub statement: String,
    pub filed_by: String,
}

/// Everything in the store that awaits arbitration.
#[derive(Debug, Default, Serialize)]
pub struct Conflicts {
    pub forks: Vec<Fork>,
    pub supersessions: Vec<SupersessionConflict>,
}

impl Conflicts {
    pub fn is_empty(&self) -> bool {
        self.forks.is_empty() && self.supersessions.is_empty()
    }
}

/// Read every conflict out of the store as it stands.
pub fn find(store: &Store) -> Conflicts {
    let view = View::build(store);
    Conflicts { forks: forks(store, &view), supersessions: supersessions(store, &view) }
}

fn forks(store: &Store, view: &View) -> Vec<Fork> {
    view.forked
        .iter()
        .map(|(decision, tip_indices)| {
            let tips: Vec<TipView> =
                tip_indices.iter().filter_map(|&i| tip_view(store, view, i)).collect();
            let class = if allocations_differ(&tips) {
                ConflictClass::DivergentAllocation
            } else {
                ConflictClass::DivergentRevision
            };
            Fork {
                decision: decision.clone(),
                class,
                common_parent: common_parent(view, decision, tip_indices),
                tips,
            }
        })
        .collect()
}

fn allocations_differ(tips: &[TipView]) -> bool {
    tips.windows(2).any(|w| w[0].allocation != w[1].allocation)
}

fn tip_view(store: &Store, view: &View, index: usize) -> Option<TipView> {
    let viewed = view.versions.get(index)?;
    let accepted_by = view
        .acceptances
        .iter()
        .map(|a| a.acceptance)
        .filter(|a| !view.is_revoked(a) && a.version == viewed.raw.hash)
        .map(|a| a.actor.to_string())
        .collect();
    Some(TipView {
        hash: viewed.raw.hash.clone(),
        statement: viewed.raw.statement.clone(),
        allocation: allocation_name(viewed.raw),
        filed_by: filed_by(store, &viewed.raw.hash),
        accepted_by,
    })
}

pub(crate) fn allocation_name(raw: &VersionRaw) -> String {
    raw.allocation.map(|k| k.as_str()).unwrap_or("unallocated").to_string()
}

/// Who filed the change-set carrying this version, and when.
fn filed_by(store: &Store, hash: &VersionHash) -> String {
    store
        .log
        .iter()
        .find(|c| c.file.versions.iter().any(|v| v.hash == *hash))
        .map(|c| format!("{} at {}", c.file.created_by, c.file.created_at.date_naive()))
        .unwrap_or_else(|| "unknown".to_string())
}

/// The deepest version every tip's chain passes through, presented as
/// `short-hash — statement`. `None` when the chains share nothing.
fn common_parent(view: &View, decision: &str, tips: &[usize]) -> Option<String> {
    let by_hash: BTreeMap<&VersionHash, &VersionRaw> = view
        .versions
        .iter()
        .filter(|v| v.raw.decision.to_string() == decision)
        .map(|v| (&v.raw.hash, v.raw))
        .collect();
    let ancestor_chain = |start: &VersionHash| -> Vec<VersionHash> {
        let mut chain = Vec::new();
        let mut cursor = Some(start.clone());
        for _ in 0..=by_hash.len() {
            let Some(hash) = cursor else { break };
            chain.push(hash.clone());
            cursor = by_hash.get(&hash).and_then(|raw| raw.parent.clone());
        }
        chain
    };
    let (first, rest) = tips.split_first()?;
    let first_chain = ancestor_chain(&view.versions.get(*first)?.raw.hash);
    let shared = first_chain.into_iter().find(|candidate| {
        rest.iter().all(|&i| {
            view.versions
                .get(i)
                .is_some_and(|v| ancestor_chain(&v.raw.hash).contains(candidate))
        })
    })?;
    let statement =
        by_hash.get(&shared).map(|raw| raw.statement.clone()).unwrap_or_default();
    Some(format!("{} — {statement}", shared.short()))
}

/// Decisions superseded by more than one live claimant: the same reading
/// as graph shape `G005`, derived from tips so a withdrawn claim is
/// history.
fn supersessions(store: &Store, view: &View) -> Vec<SupersessionConflict> {
    let mut by_target: BTreeMap<String, Vec<ClaimView>> = BTreeMap::new();
    for viewed in view.latest_versions() {
        let Some(target) = &viewed.raw.supersedes else { continue };
        by_target.entry(target.to_string()).or_default().push(ClaimView {
            decision: viewed.raw.decision.to_string(),
            tip: viewed.raw.hash.clone(),
            statement: viewed.raw.statement.clone(),
            filed_by: filed_by(store, &viewed.raw.hash),
        });
    }
    by_target
        .into_iter()
        .filter(|(_, claimants)| claimants.len() > 1)
        .map(|(target, claimants)| SupersessionConflict { target, claimants })
        .collect()
}
