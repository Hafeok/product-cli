//! Grouping a proposal into reviewable batches — §7.4's volume answer.
//!
//! 561 files was already at the edge of per-file review; the full solution is
//! roughly fifteen times that corpus. Wholesale merge is manufactured ground
//! by §5.6, so the answer is neither "review everything" nor "review nothing"
//! but **review by kind**.
//!
//! A batch is the set of assertions sharing a *derivation signature*: the two
//! marks, the row, the rule, the member kind, the container kind. Everything
//! in one batch was read the same way about the same kind of thing, so a
//! reviewer's verdict on a sample carries to the batch — which is what makes
//! Group A and Group B reviewable as classes rather than as 49 separate files.
//!
//! Ordering is total, so two runs over the same facts emit the same batches in
//! the same order, which is what lets a batch listing diff across refs.

use serde::Serialize;

use super::axes::{DerivationConfidence, DomainRelevance, RelevanceBasis};
use super::facts::{DeclKind, MemberKind};
use super::triple::ProposedAssertion;

/// How many assertions a batch names in full before it starts counting.
const SAMPLE: usize = 5;

/// One reviewable group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Batch {
    /// Axis 1, and the first sort key.
    pub confidence: DerivationConfidence,
    /// Axis 2.
    pub relevance: DomainRelevance,
    /// Why axis 2 says what it says.
    pub relevance_basis: RelevanceBasis,
    /// The §5.2 row.
    pub row: &'static str,
    /// The named rule inside the row.
    pub rule: &'static str,
    /// The member kind the batch derives from, where it derives from one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_kind: Option<MemberKind>,
    /// The kind of declaration the batch is about.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_kind: Option<DeclKind>,
    /// The operations every assertion in the batch stood on.
    pub stands_on: Vec<String>,
    pub count: usize,
    /// A bounded sample, each read as a reviewer reads a triple.
    pub sample: Vec<String>,
    /// Every member's id, so the batch is actionable rather than descriptive.
    pub ids: Vec<String>,
}

impl Batch {
    /// The batch's key as one line — what a reviewer scans down a listing.
    pub fn label(&self) -> String {
        let member = self.member_kind.map(|k| format!(" · {}", k.as_str())).unwrap_or_default();
        let container =
            self.container_kind.map(|k| format!(" · {}", k.as_str())).unwrap_or_default();
        format!(
            "{}/{} · {} · {}{}{}",
            self.confidence.as_str(),
            self.relevance.as_str(),
            self.row,
            self.rule,
            member,
            container
        )
    }
}

/// Batch `assertions`, deterministically.
///
/// Sorted by confidence (best first — the enum is ordinal), then relevance
/// rank, then row, rule, member kind, container kind. Members keep their id
/// order inside a batch, which is the order the file names sort in.
pub fn batches(assertions: &[ProposedAssertion]) -> Vec<Batch> {
    let mut out: Vec<Batch> = Vec::new();
    for a in assertions {
        let (rule, member_kind, container_kind) = a.derivation.signature();
        match out.iter_mut().find(|b| {
            b.confidence == a.confidence
                && b.relevance == a.relevance
                && b.row == a.row
                && b.rule == rule
                && b.member_kind == member_kind
                && b.container_kind == container_kind
        }) {
            Some(batch) => push(batch, a),
            None => out.push(open(a, rule, member_kind, container_kind)),
        }
    }
    for batch in &mut out {
        batch.ids.sort();
        batch.sample.sort();
        batch.sample.truncate(SAMPLE);
    }
    out.sort_by(|a, b| sort_key(a).cmp(&sort_key(b)));
    out
}

/// The total order two runs must agree on.
fn sort_key(
    b: &Batch,
) -> (DerivationConfidence, u8, &'static str, &'static str, String, String) {
    (
        b.confidence,
        b.relevance.rank(),
        b.row,
        b.rule,
        b.member_kind.map(|k| k.as_str().to_string()).unwrap_or_default(),
        b.container_kind.map(|k| k.as_str().to_string()).unwrap_or_default(),
    )
}

fn open(
    a: &ProposedAssertion,
    rule: &'static str,
    member_kind: Option<MemberKind>,
    container_kind: Option<DeclKind>,
) -> Batch {
    Batch {
        confidence: a.confidence,
        relevance: a.relevance,
        relevance_basis: a.relevance_basis,
        row: a.row,
        rule,
        member_kind,
        container_kind,
        stands_on: a.derivation.stands_on.iter().map(|o| (*o).to_string()).collect(),
        count: 1,
        sample: vec![a.triple.to_value()],
        ids: vec![a.id.clone()],
    }
}

fn push(batch: &mut Batch, a: &ProposedAssertion) {
    batch.count += 1;
    batch.ids.push(a.id.clone());
    if batch.sample.len() < SAMPLE * 4 {
        batch.sample.push(a.triple.to_value());
    }
}

#[cfg(test)]
#[path = "batch_tests.rs"]
mod tests;
