//! The terminal rendering of a batched acceptance, as a pasteable block.
//!
//! Kept apart from the plan so `--json` and the text form cannot drift into
//! disagreeing: both are the same [`Outcome`] struct, one serialised and one
//! laid out. The layout is built for the worksheet — what was read, what was
//! signed, how many, under which manifest — because the pass this verb
//! serves is recorded, not merely performed.

use super::{Member, Outcome, Plan, Standing};

/// The whole invocation on screen: the enumeration, the tally, the verdict.
pub fn render(outcome: &Outcome, elapsed: &str) -> String {
    let plan = &outcome.plan;
    let mut out = format!(
        "selection  {} — {} decision(s), as {}\nmanifest   {}\n",
        plan.selector,
        plan.members.len(),
        plan.actor,
        plan.manifest
    );
    for m in &plan.members {
        out.push('\n');
        out.push_str(&member(m));
    }
    out.push('\n');
    out.push_str(&tally(plan));
    out.push_str(&verdict(outcome));
    out.push_str(&format!("elapsed    {elapsed}\n"));
    out
}

fn member(m: &Member) -> String {
    let mut out = format!(
        "  {} [{}] {} → {}\n    {}\n    {}\n",
        m.decision, m.group, m.state, m.standing, m.version, m.statement_head
    );
    if let Some(because) = &m.because {
        out.push_str(&format!("    {because}\n"));
    }
    out
}

fn tally(plan: &Plan) -> String {
    format!(
        "{} signable · {} already signed · {} held · {} forked\n",
        plan.tally(Standing::Signable),
        plan.tally(Standing::AlreadySigned),
        plan.tally(Standing::Held),
        plan.tally(Standing::Forked),
    )
}

/// The one line that says what happened, then whatever detail it owes.
fn verdict(outcome: &Outcome) -> String {
    if let Some(refusal) = &outcome.refusal {
        let mut out = format!("\nrefused — {}\n", refusal.message);
        for line in &refusal.detail {
            out.push_str(&format!("  - {line}\n"));
        }
        return out;
    }
    if outcome.dry_run {
        return format!(
            "\ndry run — nothing written. Having read the above, sign this exact selection with:\n  ledger accept {} --confirm {}\n",
            flags(&outcome.plan.selector),
            outcome.plan.manifest
        );
    }
    let mut out = format!(
        "\nsigned {} decision(s) as {} — one acceptance record each, each signing its own version hash\n",
        outcome.signed.len(),
        outcome.plan.actor
    );
    if let Some(path) = &outcome.filed {
        out.push_str(&format!("filed      {}\n", path.display()));
    }
    out
}

/// The selector's CLI spelling, so the confirm line can be pasted as printed.
fn flags(selector: &str) -> String {
    match selector.split_once(':') {
        Some(("group", g)) => format!("--group {g}"),
        Some(("set", s)) => format!("--set {s}"),
        Some(("decision", d)) => d.to_string(),
        _ => selector.to_string(),
    }
}
