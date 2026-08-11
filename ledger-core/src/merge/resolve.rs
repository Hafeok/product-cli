//! Recording an arbitration as an ordinary ledger act.
//!
//! A fork resolution files a *reconciled version*: content the human chose
//! (a tip as it stands, or a composed statement), `parent` the chosen tip,
//! `merged_from` the tip it closes — the DAG heals because the act names
//! both chains. A competing-supersession resolution files the losing
//! claimants' next versions with the `supersedes` edge withdrawn. Both are
//! attributed to the arbitrating identity from git config, both run the
//! same delta-gate as every verb — file classes *and* graph classes — and
//! both leave the reconciled content **unaccepted**: acceptance signs a
//! hash, the hash is fresh, and no prior signature follows content it
//! never named.

use crate::author::{Applied, Author, AuthorError};
use crate::changeset::ChangeSet;
use crate::hash::version_hash;
use crate::store::{LoggedChangeSet, Store};
use crate::verify::view::View;
use crate::version::VersionRaw;

use super::conflicts::{Fork, SupersessionConflict};

/// What the human chose for one fork.
#[derive(Debug, Clone)]
pub enum ForkChoice {
    /// The numbered tip's content stands as the reconciled version.
    TipStands(usize),
    /// A composed statement stands, on the numbered tip's other fields.
    Compose { base_tip: usize, statement: String },
}

impl Author {
    /// Record a fork arbitration. One change-set, one act: the reconciled
    /// version extends the chosen tip and closes every other tip via
    /// `merged_from`, so a single tip remains. The reconciled version is
    /// unaccepted by construction.
    pub fn resolve_fork(&mut self, fork: &Fork, choice: &ForkChoice) -> Result<Applied, AuthorError> {
        let store = self.load();
        let view = View::build(&store);
        let tips = current_tips(&view, &fork.decision, fork)?;
        let (base_tip, statement) = match choice {
            ForkChoice::TipStands(i) => (*i, None),
            ForkChoice::Compose { base_tip, statement } => (*base_tip, Some(statement.clone())),
        };
        let chosen: &VersionRaw = tips.get(base_tip).copied().ok_or_else(|| {
            AuthorError::Usage(format!("tip {} does not exist on {}", base_tip + 1, fork.decision))
        })?;
        let mut candidate = self.shell(Some(note(fork, chosen, &tips, base_tip)))?;
        candidate.format = crate::format::MERGE_FORMAT;
        let mut parent = chosen.clone();
        if let Some(text) = statement {
            parent.statement = text;
        }
        for other in tips.iter().enumerate().filter(|(i, _)| *i != base_tip).map(|(_, t)| t) {
            let mut reconciled = parent.clone();
            reconciled.parent = Some(parent.hash.clone());
            reconciled.merged_from = Some(other.hash.clone());
            repin_floor(&store, &mut reconciled);
            reconciled.hash = version_hash(&reconciled);
            candidate.versions.push(reconciled.clone());
            parent = reconciled;
        }
        self.gate_and_append(&store, candidate)
    }

    /// Record a competing-supersession arbitration: every losing claimant
    /// files its next version with the `supersedes` edge withdrawn. The
    /// winner's claim is untouched — it already says what the human chose.
    pub fn resolve_supersession(
        &mut self,
        conflict: &SupersessionConflict,
        winner: usize,
    ) -> Result<Applied, AuthorError> {
        if winner >= conflict.claimants.len() {
            return Err(AuthorError::Usage(format!(
                "claimant {} does not exist for {}",
                winner + 1,
                conflict.target
            )));
        }
        let store = self.load();
        let view = View::build(&store);
        let mut candidate = self.shell(Some(format!(
            "merge arbitration: competing supersession of {} — {} stands; the other claim is withdrawn",
            conflict.target,
            conflict.claimants[winner].decision
        )))?;
        for (i, claimant) in conflict.claimants.iter().enumerate() {
            if i == winner {
                continue;
            }
            let tip = view
                .latest
                .get(&claimant.decision)
                .and_then(|&idx| view.versions.get(idx))
                .filter(|v| v.raw.hash == claimant.tip)
                .ok_or_else(|| {
                    AuthorError::Conflict(format!(
                        "{} moved since the conflict was read — re-run `ledger merge --resolve`",
                        claimant.decision
                    ))
                })?;
            let mut withdrawn = tip.raw.clone();
            withdrawn.parent = Some(tip.raw.hash.clone());
            withdrawn.supersedes = None;
            repin_floor(&store, &mut withdrawn);
            withdrawn.hash = version_hash(&withdrawn);
            candidate.versions.push(withdrawn);
        }
        self.gate_and_append(&store, candidate)
    }

    /// The delta-gate, both stages: refuse any fresh file finding (the same
    /// classes and messages `verify` reports) and any fresh graph finding.
    /// An arbitration may only *remove* findings, never trade one for
    /// another.
    fn gate_and_append(
        &mut self,
        store: &Store,
        candidate: ChangeSet,
    ) -> Result<Applied, AuthorError> {
        self.refusal_check(store, &candidate, |_| false)?;
        graph_delta_check(store, &candidate)?;
        let path = self.append(&candidate)?;
        let lines = candidate
            .versions
            .iter()
            .map(|v| {
                format!(
                    "filed {} of {} — unaccepted; a reconciled version awaits fresh acceptance",
                    v.hash.short(),
                    v.decision
                )
            })
            .collect();
        Ok(Applied { path, lines })
    }
}

/// The fork's tips as they stand right now, index-aligned with the
/// presentation. Refuses when the store moved under the arbitration.
fn current_tips<'a>(
    view: &'a View<'a>,
    decision: &str,
    fork: &Fork,
) -> Result<Vec<&'a VersionRaw>, AuthorError> {
    let indices = view.forked.get(decision).ok_or_else(|| {
        AuthorError::Conflict(format!(
            "{decision} is no longer forked — the store moved since the conflict was read"
        ))
    })?;
    let live: Vec<&VersionRaw> = fork
        .tips
        .iter()
        .filter_map(|tip| {
            indices
                .iter()
                .filter_map(|&idx| view.versions.get(idx))
                .find(|v| v.raw.hash == tip.hash)
                .map(|v| v.raw)
        })
        .collect();
    if live.len() != fork.tips.len() || live.len() != indices.len() {
        return Err(AuthorError::Conflict(format!(
            "the tips of {decision} moved since the conflict was read — re-run `ledger merge --resolve`"
        )));
    }
    Ok(live)
}

/// Re-pin the set's current floor on a version being filed now, exactly as
/// the ordinary next-version pipeline does.
fn repin_floor(store: &Store, raw: &mut VersionRaw) {
    if let Some(set) = store.set(&raw.set) {
        raw.tolerance_floor_at_creation = set.tolerance_floor;
        if raw.tolerance_override.is_some_and(|o| o <= set.tolerance_floor) {
            raw.tolerance_override = None;
        }
    }
}

fn note(fork: &Fork, chosen: &VersionRaw, tips: &[&VersionRaw], base_tip: usize) -> String {
    let closed: Vec<String> = tips
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != base_tip)
        .map(|(_, raw)| raw.hash.short().to_string())
        .collect();
    format!(
        "merge arbitration: {} on {} — {} stands, {} closed",
        fork.class,
        fork.decision,
        chosen.hash.short(),
        closed.join(", ")
    )
}

/// No fresh graph finding may ride in on an arbitration.
fn graph_delta_check(current: &Store, candidate: &ChangeSet) -> Result<(), AuthorError> {
    let baseline = crate::graph::shapes::graph_findings(current);
    let mut with = Store {
        root: current.root.clone(),
        dir: current.dir.clone(),
        sets: current.sets.clone(),
        log: current.log.to_vec(),
        schema_findings: current.schema_findings.clone(),
    };
    with.log.push(LoggedChangeSet {
        path: current.dir.join("log").join(candidate.file_name()),
        file: candidate.clone(),
    });
    let fresh: Vec<String> = crate::graph::shapes::graph_findings(&with)
        .into_iter()
        .filter(|f| !baseline.contains(f))
        .map(|f| f.to_string())
        .collect();
    if fresh.is_empty() {
        Ok(())
    } else {
        Err(AuthorError::Conflict(format!(
            "refused — the arbitration would introduce graph findings:\n  - {}",
            fresh.join("\n  - ")
        )))
    }
}
