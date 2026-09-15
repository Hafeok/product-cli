//! The check policy: showing what is in force, filing a new version.
//!
//! Filing, never editing. A change is a supersession with provenance, so the
//! question *who lowered this, when, and on what basis* has an answer and the
//! old threshold stays readable beside the argument that justified it.

use std::path::Path;

use chrono::Utc;
use clap::Args;
use ledger_core::mint::UlidMint;
use product_core::error::{ProductError, Result};
use serde_json::json;
use spec_core::policy::{self, Policy};
use spec_core::{metrics, policy_check, store};

use crate::exit;
use crate::render::Report;
use crate::verbs::resolve_identity;

#[derive(Args)]
pub struct PolicyShowArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct PolicySetArgs {
    /// A YAML file carrying `policy_verdicts` and `not_gated`.
    pub file: std::path::PathBuf,
    /// Who files it. Defaults to the git identity.
    #[arg(long)]
    pub principal: Option<String>,
    #[arg(long)]
    pub json: bool,
}

/// What a caller writes in the policy file: the parts that are theirs.
///
/// Identity, timestamp, supersession and digest are the store's to assign, so
/// they are not fields a caller can set. A policy that could name its own
/// predecessor could also name none and quietly fork the chain.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyDraft {
    #[serde(default)]
    policy_verdicts: Vec<spec_core::policy::PolicyVerdict>,
    #[serde(default)]
    not_gated: Vec<spec_core::policy::NotGated>,
    #[serde(default)]
    not_gated_asserted_none: bool,
}

/// Show the policy in force, with what it declares it does not gate.
pub fn show(root: &Path, args: &PolicyShowArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let in_force = match spec.policy_in_force() {
        Err(tips) => {
            let ids = tips.iter().map(|p| p.id.as_str()).collect::<Vec<_>>().join(", ");
            return Ok(Report::text(
                exit::FINDINGS,
                format!("the policy chain forks at {ids} — only a recorded supersession settles it"),
            ));
        }
        Ok(None) => {
            return Ok(Report::text(
                exit::CONFORMANT,
                "no policy filed — structural verdicts only.\n\
                 A project with no policy gets verdicts on broken things and nothing else;\n\
                 shipping default thresholds would presume a basis nobody stated.",
            )
            .with_json(args.json.then(|| json!({"in_force": null}))))
        }
        Ok(Some(policy)) => policy,
    };

    let text = render(in_force, &spec);
    let body = args.json.then(|| json!(in_force));
    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}

/// File a new policy version.
pub fn set(root: &Path, args: &PolicySetArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let previous = match spec.policy_in_force() {
        Err(tips) => {
            let ids = tips.iter().map(|p| p.id.as_str()).collect::<Vec<_>>().join(", ");
            return Err(ProductError::ConfigError(format!(
                "the policy chain forks at {ids} — settle it before filing another version"
            )));
        }
        Ok(tip) => tip.map(|p| p.id.clone()),
    };

    let text = std::fs::read_to_string(&args.file)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", args.file.display())))?;
    let draft: PolicyDraft =
        serde_yaml::from_str(&text).map_err(|e| ProductError::ParseError {
            file: args.file.clone(),
            line: e.location().map(|l| l.line()),
            message: e.to_string(),
        })?;

    let candidate = Policy {
        form: policy::POLICY_FORM.to_string(),
        id: UlidMint::system().mint(),
        supersedes: previous,
        filed_by: resolve_identity(root, args.principal.as_deref())?,
        filed_at: Utc::now(),
        policy_verdicts: draft.policy_verdicts,
        not_gated: draft.not_gated,
        not_gated_asserted_none: draft.not_gated_asserted_none,
        binds: String::new(),
    };

    let findings = policy_check::judge_policy(&candidate, &metrics::compute(&spec));
    if !findings.is_empty() {
        return Ok(crate::render::refusal(&candidate.id, "file policy", &findings, args.json));
    }

    let filed = policy::file(root, candidate)?;
    let summary = format!(
        "filed policy {} ({} verdict(s), {} not gated){}",
        filed.id,
        filed.policy_verdicts.len(),
        filed.not_gated.len(),
        filed.supersedes.as_ref().map_or(String::new(), |p| format!("\n  supersedes {p}"))
    );
    let body = args.json.then(|| json!({"policy": filed.id, "status": "filed"}));
    Ok(Report::text(exit::CONFORMANT, summary).with_json(body))
}

fn render(policy: &Policy, spec: &spec_core::SpecStore) -> String {
    let observed = metrics::compute(spec);
    let mut lines = vec![format!(
        "policy {} — filed by {} on {}",
        policy.id,
        policy.filed_by.as_str(),
        policy.filed_at.date_naive()
    )];

    lines.push("\ngated:".into());
    if policy.policy_verdicts.is_empty() {
        lines.push("  (nothing — structural verdicts only)".into());
    }
    for verdict in &policy.policy_verdicts {
        let value = observed.get(&verdict.metric).map_or("—".into(), |m| m.render());
        lines.push(format!(
            "  {} {}  (observed {})\n    principal: {}\n    basis: {}",
            verdict.metric,
            verdict.fires_when,
            value,
            verdict.principal.render(),
            verdict.basis.trim()
        ));
    }

    lines.push("\nreported, deliberately not gated:".into());
    if policy.not_gated_asserted_none {
        lines.push("  asserted-none — every metric reported is gated".into());
    }
    for entry in &policy.not_gated {
        lines.push(format!("  {} — {}", entry.metric, entry.reason.trim()));
    }
    lines.join("\n")
}
