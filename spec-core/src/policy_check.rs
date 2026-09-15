//! Judging a policy, then running the verdicts it declares.
//!
//! Two kinds of verdict live here and they never mix. The **structural** ones
//! (`S008`–`S011`) say the policy itself is malformed and are not a matter of
//! project policy — a project that could switch them off would have a tool
//! that reports what it was told to report. The **policy** verdicts are the
//! project's own, declared with a threshold and a basis, and they are reported
//! as a separate class so a reader can always tell which is which.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::check::{Class, Finding};
use crate::metrics::Metric;
use crate::policy::{self, Policy, PolicyVerdict};

/// A project's own verdict, fired.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PolicyFinding {
    pub metric: String,
    pub fires_when: String,
    pub observed: String,
    pub basis: String,
    pub principal: String,
}

impl std::fmt::Display for PolicyFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "policy: {} {} (observed {}) — {}",
            self.metric, self.fires_when, self.observed, self.principal
        )
    }
}

/// `S008`–`S011`: is the policy itself well formed?
pub fn judge_policy(policy: &Policy, metrics: &BTreeMap<String, Metric>) -> Vec<Finding> {
    let mut findings = Vec::new();
    for verdict in &policy.policy_verdicts {
        findings.extend(judge_verdict(verdict, metrics));
    }
    findings.extend(duplicate_bases(policy));
    findings.extend(uncovered_set(policy));
    findings
}

/// `S008` and `S009` over one verdict.
fn judge_verdict(verdict: &PolicyVerdict, metrics: &BTreeMap<String, Metric>) -> Vec<Finding> {
    let mut findings = Vec::new();
    if verdict.basis.trim().is_empty() {
        findings.push(Finding::new(Class::S008, &verdict.metric, "no basis stated"));
    }
    if verdict.principal.identifier.trim().is_empty() {
        findings.push(Finding::new(Class::S008, &verdict.metric, "no principal named"));
    }
    match metrics.get(&verdict.metric) {
        None => findings.push(Finding::new(
            Class::S008,
            &verdict.metric,
            "names no metric this flow computes",
        )),
        Some(_) if Condition::parse(&verdict.fires_when).is_none() => findings.push(Finding::new(
            Class::S008,
            &verdict.metric,
            &format!("`{}` is not a condition this flow can evaluate", verdict.fires_when),
        )),
        Some(_) => {}
    }
    let expected = policy::basis_digest(&verdict.fires_when);
    if verdict.basis_binds != expected {
        // The digest is reported rather than filled in on the author's behalf,
        // the way the ledger reports a version hash: pasting it back is the
        // edit at the basis that revisiting the argument requires.
        findings.push(Finding::new(
            Class::S009,
            &verdict.metric,
            &format!(
                "the basis does not bind `{}` — the threshold moved, the argument did not. \
                 Revisit the basis, then set `basis_binds: {expected}`",
                verdict.fires_when
            ),
        ));
    }
    findings
}

/// `S010`: copy-paste is the likeliest degeneration after silence, and it is
/// trivially detectable within one policy.
fn duplicate_bases(policy: &Policy) -> Vec<Finding> {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    let mut findings = Vec::new();
    for verdict in &policy.policy_verdicts {
        let key = verdict.basis.trim();
        if key.is_empty() {
            continue;
        }
        match seen.get(key) {
            Some(first) => findings.push(Finding::new(
                Class::S010,
                &verdict.metric,
                &format!("carries the same basis as `{first}`"),
            )),
            None => {
                seen.insert(key, &verdict.metric);
            }
        }
    }
    findings
}

/// `S011`: a policy listing what fires without listing what it deliberately
/// does not is a coverage claim with no uncovered set.
fn uncovered_set(policy: &Policy) -> Vec<Finding> {
    if !policy.not_gated.is_empty() || policy.not_gated_asserted_none {
        return Vec::new();
    }
    vec![Finding::new(
        Class::S011,
        &policy.id,
        "no `not_gated` list — say what is reported and deliberately not gated, \
         or set `not_gated_asserted_none` to claim everything reported is gated",
    )]
}

/// Run the project's own verdicts against the observed metrics.
///
/// Only well-formed verdicts fire. A malformed one has already produced a
/// structural finding, and letting it also fire would report the same defect
/// twice under two different meanings.
pub fn run(policy: &Policy, metrics: &BTreeMap<String, Metric>) -> Vec<PolicyFinding> {
    policy
        .policy_verdicts
        .iter()
        .filter(|v| v.basis_binds == policy::basis_digest(&v.fires_when))
        .filter_map(|verdict| {
            let metric = metrics.get(&verdict.metric)?;
            let condition = Condition::parse(&verdict.fires_when)?;
            condition.holds(metric).then(|| PolicyFinding {
                metric: verdict.metric.clone(),
                fires_when: verdict.fires_when.clone(),
                observed: metric.render(),
                basis: verdict.basis.clone(),
                principal: verdict.principal.render(),
            })
        })
        .collect()
}

/// A `fires_when` expression: `count > 0`, `percent <= 80`, and nothing else.
///
/// Deliberately tiny. A policy language rich enough to be interesting is a
/// policy language rich enough to hide a threshold in, and the point of the
/// condition is that a reader can check it against the basis at a glance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Condition {
    subject: Subject,
    op: Op,
    value: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Subject {
    Count,
    Percent,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
}

impl Condition {
    /// Parse, or `None` when the expression is not one this flow evaluates.
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = text.split_whitespace();
        let subject = match parts.next()? {
            "count" => Subject::Count,
            "percent" => Subject::Percent,
            _ => return None,
        };
        let op = match parts.next()? {
            ">" => Op::Gt,
            ">=" => Op::Ge,
            "<" => Op::Lt,
            "<=" => Op::Le,
            "==" => Op::Eq,
            "!=" => Op::Ne,
            _ => return None,
        };
        let value: f64 = parts.next()?.trim_end_matches('%').parse().ok()?;
        parts.next().is_none().then_some(Self { subject, op, value })
    }

    /// Whether the condition holds for an observed metric.
    ///
    /// A percentage condition over a metric with no population never fires:
    /// there is nothing to be a proportion of, and firing on an undefined
    /// figure would be a verdict about the absence of data.
    pub fn holds(&self, metric: &Metric) -> bool {
        let observed = match self.subject {
            #[allow(clippy::cast_precision_loss)]
            Subject::Count => metric.count as f64,
            Subject::Percent => match metric.percent() {
                Some(pct) => pct,
                None => return false,
            },
        };
        match self.op {
            Op::Gt => observed > self.value,
            Op::Ge => observed >= self.value,
            Op::Lt => observed < self.value,
            Op::Le => observed <= self.value,
            Op::Eq => (observed - self.value).abs() < f64::EPSILON,
            Op::Ne => (observed - self.value).abs() >= f64::EPSILON,
        }
    }
}

#[path = "policy_check_tests.rs"]
#[cfg(test)]
mod tests;
