//! Reviewing candidates: ratifying one, or refusing one.
//!
//! Per candidate, deliberately. Sixty-eight candidates can be reviewed at a
//! desk in an hour; they cannot be ticked. Friction belongs in the deciding,
//! never in the tooling — so a model may draft every field of what follows,
//! and a person still types the verb.

use std::path::Path;

use chrono::Utc;
use clap::Args;
use product_core::error::{ProductError, Result};
use serde_json::json;
use spec_core::ratify::{self, Ratification, Ratified, Refusal, Refused};
use spec_core::store;

use crate::exit;
use crate::render::Report;
use crate::verbs::resolve_identity;

#[derive(Args)]
pub struct CandidatesArgs {
    /// Show only candidates nobody has reviewed.
    #[arg(long)]
    pub unreviewed: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct AcceptArgs {
    /// The candidate to ratify, or `--act-only` for one with no candidate.
    pub candidate: String,
    /// The act's address. Defaults to a slug of the name.
    #[arg(long = "id")]
    pub act_id: Option<String>,
    /// What a principal calls this act. Never derived from a route.
    #[arg(long)]
    pub name: String,
    /// What the act settles.
    #[arg(long)]
    pub settles: String,
    /// Who answers for the ratification. Defaults to the git identity.
    #[arg(long)]
    pub principal: Option<String>,
    #[arg(long)]
    pub json: bool,
}

#[derive(Args)]
pub struct RejectArgs {
    /// The candidate to refuse.
    pub candidate: String,
    /// Why. Prose, read-enforced; it exists so the next reader can see whether
    /// the refusal was reasoned.
    #[arg(long)]
    pub reason: String,
    /// Who answers for the refusal. Defaults to the git identity.
    #[arg(long)]
    pub principal: Option<String>,
    #[arg(long)]
    pub json: bool,
}

/// List candidates with what was observed and what is still unfilled.
pub fn candidates(root: &Path, args: &CandidatesArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    let Some(inventory) = &spec.inventory else {
        return Ok(Report::text(
            exit::CONFORMANT,
            "no inventory — run `specflow import` first, or author acts directly",
        ));
    };

    let shown: Vec<_> = inventory
        .candidates
        .iter()
        .filter(|c| !args.unreviewed || spec.is_unreviewed(&c.id))
        .collect();
    let text = shown
        .iter()
        .map(|c| {
            let state = if spec.is_unreviewed(&c.id) { "unreviewed" } else { "reviewed" };
            let transport = c.observed.get("transport").map_or("—", String::as_str);
            format!("{state:<11} {}\n            {transport}", c.id)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let body = args.json.then(|| json!(shown.iter().map(|c| json!({
        "candidate": c.id,
        "entry_point": c.entry_point,
        "observed": c.observed,
        "unfilled_slots": c.unfilled_slots,
        "unreviewed": spec.is_unreviewed(&c.id),
    })).collect::<Vec<_>>()));
    Ok(Report::text(exit::CONFORMANT, text).with_json(body))
}

/// Ratify a candidate as an act.
pub fn accept(root: &Path, args: &AcceptArgs) -> Result<Report> {
    let spec = store::load_store(root)?;
    require_reviewable(&spec, &args.candidate)?;
    let realised_at = entry_points_for(&spec, &args.candidate);
    let from_candidate = if realised_at.is_empty() { None } else { Some(args.candidate.clone()) };

    let ratification = Ratification {
        act_id: args.act_id.clone().unwrap_or_else(|| format!("act/{}", slugify(&args.name))),
        name: args.name.clone(),
        settles: args.settles.clone(),
        principal: resolve_identity(root, args.principal.as_deref())?,
        at: Utc::now(),
        realised_at,
        from_candidate,
    };
    let act_id = ratification.act_id.clone();

    match ratify::accept(root, ratification)? {
        Ratified::Refused(findings) => {
            Ok(crate::render::refusal(&act_id, "ratify", &findings, args.json))
        }
        Ratified::Filed(act) => {
            let text = format!(
                "ratified {} — {}\n  settles: {}\n  realised at: {}",
                act.id,
                act.name,
                act.settles,
                if act.realised_at.is_empty() { "(nothing yet)".into() } else { act.realised_at.join(", ") }
            );
            let body = args.json.then(|| json!({
                "act": act.id,
                "name": act.name,
                "settles": act.settles,
                "realised_at": act.realised_at,
                "status": "ratified",
            }));
            Ok(Report::text(exit::CONFORMANT, text).with_json(body))
        }
    }
}

/// Refuse a candidate, filing the reason.
pub fn reject(root: &Path, args: &RejectArgs) -> Result<Report> {
    let refusal = Refusal {
        candidate: args.candidate.clone(),
        reason: args.reason.clone(),
        principal: resolve_identity(root, args.principal.as_deref())?,
        at: Utc::now(),
    };
    match ratify::reject(root, refusal)? {
        Refused::Blocked(findings) => {
            Ok(crate::render::refusal(&args.candidate, "refuse", &findings, args.json))
        }
        Refused::Filed(rejection) => {
            let text = format!("refused {} — {}", rejection.candidate, rejection.reason);
            let body = args.json.then(|| json!({
                "candidate": rejection.candidate,
                "reason": rejection.reason,
                "status": "refused",
            }));
            Ok(Report::text(exit::CONFORMANT, text).with_json(body))
        }
    }
}

/// The entry points a candidate stands for, or empty when it names none.
fn entry_points_for(spec: &spec_core::SpecStore, candidate: &str) -> Vec<String> {
    spec.inventory
        .as_ref()
        .map(|inventory| {
            inventory
                .candidates
                .iter()
                .filter(|c| c.id == candidate)
                .map(|c| c.entry_point.clone())
                .collect()
        })
        .unwrap_or_default()
}

/// A readable address from a name a principal typed.
pub fn slugify(name: &str) -> String {
    let mapped: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let joined = mapped.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    if joined.is_empty() {
        "unnamed".to_string()
    } else {
        joined
    }
}

/// Refuse a candidate id the inventory does not carry.
///
/// With no inventory at all this passes: greenfield is this flow with an
/// empty import, and an act authored before any code exists is the normal
/// case there rather than a special path. With an inventory present, an
/// unknown id is a typo, and ratifying a typo files an act realised nowhere.
pub fn require_reviewable(spec: &spec_core::SpecStore, candidate: &str) -> Result<()> {
    let Some(inventory) = &spec.inventory else {
        return Ok(());
    };
    if inventory.candidates.iter().any(|c| c.id == candidate) {
        return Ok(());
    }
    Err(ProductError::NotFound(format!(
        "no candidate `{candidate}` in the inventory — re-run `specflow import`, or check the id"
    )))
}
