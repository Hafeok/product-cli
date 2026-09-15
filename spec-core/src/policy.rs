//! The check policy: which metrics may carry a verdict, and on whose word.
//!
//! A determination, not a config file. Versions are append-only and a change
//! is a supersession, so *who lowered this, when, and on what basis* has an
//! answer and a policy loosened twice is evidence about the original claim.
//! Editing one in place would destroy exactly that.

use chrono::{DateTime, Utc};
use ledger_core::canon::put;
use ledger_core::hash::domain_hash;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Domain-separation prefix for a policy version's digest.
pub const POLICY_FORM: &str = "spec.policy.v1";
/// Domain-separation prefix for the digest a basis binds to.
pub const BASIS_FORM: &str = "spec.policy-basis.v1";

/// Who answers for a verdict.
///
/// There is no `machine` member, and there is not going to be one. The
/// restriction is inherited from the type rather than from a check, which is
/// what makes it structural instead of instructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalKind {
    Human,
    Team,
}

/// A named principal for a policy verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Principal {
    pub kind: PrincipalKind,
    pub identifier: String,
}

impl Principal {
    /// How the principal reads in a report.
    pub fn render(&self) -> String {
        let kind = match self.kind {
            PrincipalKind::Human => "human",
            PrincipalKind::Team => "team",
        };
        format!("{kind}:{}", self.identifier)
    }
}

/// One metric a project chose to gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyVerdict {
    /// The metric this fires on.
    pub metric: String,
    /// The condition, mechanically evaluable.
    pub fires_when: String,
    /// Why this threshold and not another. Prose, read-enforced — no
    /// instrument can check it. It exists so a reader can see whether a
    /// threshold was reasoned or reached for, and so lowering one later is
    /// visibly an argument rather than an edit.
    pub basis: String,
    /// The digest of the `fires_when` this basis was written against.
    ///
    /// Not computed on the author's behalf. Moving a threshold must cost an
    /// edit at the basis, which is exactly the degeneration path this catches:
    /// quietly lower a number, leave the argument that justified the old one.
    pub basis_binds: String,
    pub principal: Principal,
}

/// A metric a project reports and deliberately does not gate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotGated {
    pub metric: String,
    /// Why it is not gated.
    pub reason: String,
}

/// One version of a project's check policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub form: String,
    pub id: String,
    /// The version this replaces. Absent on the first.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    pub filed_by: ledger_core::identity::Identity,
    pub filed_at: DateTime<Utc>,
    #[serde(default)]
    pub policy_verdicts: Vec<PolicyVerdict>,
    /// What the policy reports and deliberately does not gate.
    ///
    /// Never empty. A policy listing what fires without listing what it
    /// deliberately does not is a coverage claim with no uncovered set; a
    /// project that genuinely gates everything it reports says so explicitly.
    #[serde(default)]
    pub not_gated: Vec<NotGated>,
    /// Set when the project gates every metric it reports, which is the only
    /// way an empty `not_gated` is admitted.
    #[serde(default)]
    pub not_gated_asserted_none: bool,
    pub binds: String,
}

impl Policy {
    /// The digest this policy's content computes to.
    pub fn computed_binds(&self) -> String {
        let Self {
            form: _,
            id,
            supersedes,
            filed_by,
            filed_at,
            policy_verdicts,
            not_gated,
            not_gated_asserted_none,
            binds: _,
        } = self;

        let mut m = Map::new();
        put(&mut m, "filed_at", Some(filed_at.to_rfc3339()));
        put(&mut m, "filed_by", Some(filed_by.as_str().to_string()));
        put(&mut m, "id", Some(id.clone()));
        put(&mut m, "not_gated", Some(render_not_gated(not_gated)));
        put(
            &mut m,
            "not_gated_asserted_none",
            Some(not_gated_asserted_none.to_string()),
        );
        put(&mut m, "policy_verdicts", Some(render_verdicts(policy_verdicts)));
        put(&mut m, "supersedes", supersedes.clone());
        domain_hash(POLICY_FORM, &canonical_bytes(m))
    }
}

/// The digest a basis must carry to bind its threshold.
///
/// Over the `fires_when` text alone, so that changing the threshold and
/// leaving the argument is the case that fails.
pub fn basis_digest(fires_when: &str) -> String {
    let mut m = Map::new();
    put(&mut m, "fires_when", Some(fires_when.to_string()));
    domain_hash(BASIS_FORM, &canonical_bytes(m))
}

/// The tip of the supersession chain, which is the policy in force.
///
/// Derived from the chain, never from id order. A chain with two tips is
/// forked: no ordering heuristic may pick a side, so the caller is told rather
/// than quietly given one of them.
pub fn in_force(versions: &[Policy]) -> std::result::Result<Option<&Policy>, Vec<&Policy>> {
    let superseded: Vec<&str> = versions
        .iter()
        .filter_map(|p| p.supersedes.as_deref())
        .collect();
    let tips: Vec<&Policy> = versions
        .iter()
        .filter(|p| !superseded.contains(&p.id.as_str()))
        .collect();
    match tips.len() {
        0 => Ok(None),
        1 => Ok(tips.first().copied()),
        _ => Err(tips),
    }
}

fn render_verdicts(verdicts: &[PolicyVerdict]) -> String {
    verdicts
        .iter()
        .map(|v| {
            format!(
                "{}|{}|{}|{}|{}",
                v.metric,
                v.fires_when,
                v.basis,
                v.basis_binds,
                v.principal.render()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_not_gated(entries: &[NotGated]) -> String {
    entries
        .iter()
        .map(|n| format!("{}|{}", n.metric, n.reason))
        .collect::<Vec<_>>()
        .join("\n")
}

fn canonical_bytes(map: Map<String, Value>) -> Vec<u8> {
    serde_json::to_vec(&Value::Object(map)).unwrap_or_default()
}

/// Where policy versions live under a repo root.
pub fn policy_dir(repo_root: &std::path::Path) -> std::path::PathBuf {
    repo_root.join(".spec").join("policy")
}

/// Every filed version, oldest file first. Order here means nothing — the tip
/// comes from the chain, and `in_force` is the only thing that decides it.
pub fn load_versions(repo_root: &std::path::Path) -> product_core::error::Result<Vec<Policy>> {
    crate::ratify::load_dir(&policy_dir(repo_root))
}

/// File a new version, sealing its digest.
///
/// Append-only: a version is never rewritten, so the threshold that was in
/// force last month is still readable next to the argument that justified it.
pub fn file(repo_root: &std::path::Path, mut policy: Policy) -> product_core::error::Result<Policy> {
    policy.binds = policy.computed_binds();
    let path = policy_dir(repo_root).join(format!("{}.yml", policy.id));
    if path.exists() {
        return Err(product_core::error::ProductError::ConfigError(format!(
            "policy version {} is already filed — a change is a supersession, not an edit",
            policy.id
        )));
    }
    crate::ratify::write_yaml(&path, &policy)?;
    Ok(policy)
}

#[path = "policy_tests.rs"]
#[cfg(test)]
pub(crate) mod tests;
