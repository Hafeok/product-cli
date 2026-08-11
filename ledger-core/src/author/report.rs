//! Read-only projections of the log for the terminal.
//!
//! `status` renders [`crate::verify::view::View`] through the seven-state
//! vocabulary — the same derived reading the gate rules use, so status can
//! never disagree with `verify`. `log` walks the change-sets in creation
//! order; `blame` answers who signed what, corroborated by the commit that
//! introduced each acceptance.

use chrono::NaiveDate;

use crate::blame::{introducing_author, Attribution};
use crate::id::DecisionId;
use crate::store::Store;
use crate::verify::state::{self, DispositionState};
use crate::verify::view::View;

/// The `status` text: every decision's state, grouped by set.
pub fn status(store: &Store, today: NaiveDate) -> String {
    let view = View::build(store);
    let rows = state::states(&view, today);
    let mut lines: Vec<String> = Vec::new();
    for set in &store.sets {
        let members: Vec<_> = rows.values().filter(|r| r.set == set.id).collect();
        lines.push(format!(
            "set `{}` — floor {}, {} decision(s)",
            set.id,
            set.tolerance_floor,
            members.len()
        ));
        for row in members {
            lines.push(member_line(&view, row));
        }
    }
    for row in rows.values().filter(|r| store.set(&r.set).is_none()) {
        lines.push(format!(
            "  {} [undeclared set `{}`] {}",
            row.decision,
            row.set,
            row.state.as_str()
        ));
    }
    let awaiting: Vec<_> = rows
        .values()
        .filter(|r| r.state == DispositionState::AwaitingAcceptance)
        .collect();
    if !awaiting.is_empty() {
        lines.push(format!("{} allocated, awaiting acceptance:", awaiting.len()));
        for row in awaiting {
            lines.push(format!("  - {}", row.decision));
        }
    }
    if lines.is_empty() {
        lines.push("the store is empty — `ledger declare` a set, then `ledger add`".to_string());
    }
    lines.join("\n")
}

/// One decision's status line: id, short hash, state, successor if any.
fn member_line(view: &View, row: &crate::verify::state::StateRow) -> String {
    let short = view
        .latest
        .get(&row.decision)
        .and_then(|i| view.versions.get(*i))
        .map(|v| v.raw.hash.short().to_string())
        .unwrap_or_default();
    let mut line = format!("  {} [{}] {}", row.decision, short, row.state.as_str());
    if let Some(by) = &row.superseded_by {
        line.push_str(&format!(" by {by}"));
    }
    line
}

/// The `log` text: change-sets in ULID (creation) order, optionally only
/// those touching one set.
pub fn log(store: &Store, set: Option<&str>) -> String {
    let mut lines = Vec::new();
    for logged in &store.log {
        let cs = &logged.file;
        if let Some(wanted) = set {
            let touches = cs.versions.iter().any(|v| v.set == wanted);
            if !touches {
                continue;
            }
        }
        lines.push(format!("{} — {} by {}", cs.id, cs.created_at.date_naive(), cs.created_by));
        if let Some(note) = &cs.note {
            lines.push(format!("  {}", note.trim().replace('\n', "\n  ")));
        }
        lines.push(format!(
            "  {} decision(s), {} version(s), {} acceptance(s), {} revocation(s)",
            cs.decisions.len(),
            cs.versions.len(),
            cs.acceptances.len(),
            cs.revocations.len()
        ));
    }
    if lines.is_empty() {
        lines.push(match set {
            Some(s) => format!("no change-sets touch set `{s}`"),
            None => "the log is empty".to_string(),
        });
    }
    lines.join("\n")
}

/// The `blame` text: every acceptance of a decision — who signed which
/// version, whether it still stands, and who actually committed it.
pub fn blame(store: &Store, decision: &DecisionId) -> String {
    let view = View::build(store);
    let id = decision.to_string();
    let latest_hash = view
        .latest
        .get(&id)
        .and_then(|i| view.versions.get(*i))
        .map(|v| v.raw.hash.clone());
    let mut lines = vec![format!("{decision}")];
    let mut any = false;
    for viewed in view.acceptances.iter().filter(|a| a.acceptance.decision == *decision) {
        any = true;
        let a = viewed.acceptance;
        let standing = if view.is_revoked(a) {
            "revoked"
        } else if latest_hash.as_ref() == Some(&a.version) {
            "live"
        } else {
            "historical"
        };
        let committed = match introducing_author(&store.root, viewed.path, &a.id.to_string()) {
            Attribution::Author(email) => format!("committed by {email}"),
            Attribution::NotCommitted => "not yet committed".to_string(),
            Attribution::NoRepository => "no repository to corroborate".to_string(),
        };
        lines.push(format!(
            "  {} signed {} at {} ({standing}, {committed})",
            a.actor,
            a.version.short(),
            a.at.date_naive()
        ));
    }
    if !any {
        lines.push("  no acceptances filed — allocated work awaits its signature".to_string());
    }
    lines.join("\n")
}
