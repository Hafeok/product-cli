//! The terminal rendering of a [`Screen`], as one aligned block.
//!
//! Kept apart from the assembly so the fields a screen carries are decided
//! in one place and their presentation in another: `--json` serialises the
//! struct directly, so text and machine form cannot drift into disagreeing.

use super::{AcceptanceLine, Edge, Screen};

const LABEL: usize = 12;

/// A whole pass, one screen after another, with a count at the head so a
/// grouped read says how much it covers before it starts.
pub fn render_all(screens: &[Screen]) -> String {
    if screens.is_empty() {
        return "no decisions match\n".to_string();
    }
    let mut out = format!("{} decision(s)\n", screens.len());
    for s in screens {
        out.push('\n');
        out.push_str(&render(s));
    }
    out
}

/// One decision's screen, ready to print.
pub fn render(s: &Screen) -> String {
    let mut out = format!("{} — {} [{}]\n", s.decision, s.state.as_str(), s.group);
    if s.forked {
        out.push_str(
            "  ** the version chain is forked — nothing may be signed until `ledger merge --resolve` arbitrates\n",
        );
    }
    let floor = match &s.set_floor {
        Some(f) => format!("{} (floor {f})", s.set),
        None => format!("{} (undeclared)", s.set),
    };
    field(&mut out, "set", &floor);
    field(&mut out, "statement", &s.statement);
    field(&mut out, "allocation", &allocation_line(s));
    if let Some(exposure) = &s.exposure {
        field(&mut out, "exposure", exposure);
    }
    if let Some(review) = &s.review_by {
        field(&mut out, "review by", review);
    }
    field(&mut out, "tolerance", &tolerance_line(s));
    field(&mut out, "version", &version_line(s));
    if let Some(parent) = &s.parent {
        field(&mut out, "parent", parent);
    }
    edges(&mut out, "based on", &s.based_on, "none — nothing stated as ground");
    // The reopen edges get their own block, never folded into the ground
    // above: the ruling this field implements is precisely that these two
    // mean different things.
    edges(&mut out, "revisit if", &s.revisit_if, "none");
    if let Some(old) = &s.supersedes {
        field(&mut out, "supersedes", old);
    }
    if let Some(by) = &s.superseded_by {
        field(&mut out, "superseded", &format!("by {by}"));
    }
    field(&mut out, "filed", &filing_line(s));
    if let Some(note) = &s.filed.note {
        field(&mut out, "note", &note.trim().replace('\n', &format!("\n{}", pad())));
    }
    out.push_str(&acceptance_block(s));
    out
}

fn pad() -> String {
    " ".repeat(LABEL + 4)
}

fn field(out: &mut String, label: &str, value: &str) {
    out.push_str(&format!("  {label:<LABEL$}  {value}\n"));
}

fn list(out: &mut String, label: &str, items: &[String], empty: &str) {
    if items.is_empty() {
        field(out, label, empty);
        return;
    }
    field(out, label, &items[0]);
    for item in &items[1..] {
        out.push_str(&format!("{}{item}\n", pad()));
    }
}

/// An edge block: the pointer, then the argument it carries indented under
/// it. The type alone (`mandate:`, `claim:`) says what kind of thing the
/// edge is; the argument says what it *asserts*, which is the thing a
/// principal is actually ruling on. When no resolver could supply the
/// argument, the pointer stands alone rather than a blank line implying
/// there is nothing to read.
fn edges(out: &mut String, label: &str, items: &[Edge], empty: &str) {
    if items.is_empty() {
        field(out, label, empty);
        return;
    }
    for (i, edge) in items.iter().enumerate() {
        if i == 0 {
            field(out, label, &edge.pointer);
        } else {
            out.push_str(&format!("{}{}\n", pad(), edge.pointer));
        }
        if let Some(argument) = edge.argument.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
            let wrapped = argument.replace('\n', &format!("\n{}    ", pad()));
            out.push_str(&format!("{}    {wrapped}\n", pad()));
        }
    }
}

fn allocation_line(s: &Screen) -> String {
    let mut line = s.allocation.clone().unwrap_or_else(|| "unallocated".to_string());
    if let Some(actor) = &s.actor {
        line.push_str(&format!(" — actor {actor}"));
    }
    if !s.discharge.is_empty() {
        line.push_str(&format!(" — discharge {}", s.discharge.join(", ")));
    }
    if let Some(stage) = &s.discharge_stage {
        line.push_str(&format!(" at {stage}"));
    }
    line
}

fn tolerance_line(s: &Screen) -> String {
    match &s.tolerance_override {
        Some(o) => format!("{o} (override; floor pinned at {})", s.tolerance_floor_at_creation),
        None => format!("{} (pinned at creation)", s.tolerance_floor_at_creation),
    }
}

fn version_line(s: &Screen) -> String {
    let chain = match s.versions_in_chain {
        1 => "first version".to_string(),
        n => format!("{n} in chain"),
    };
    format!("{} ({chain})", s.hash)
}

fn filing_line(s: &Screen) -> String {
    format!("{} on {} by {}", s.filed.change_set, s.filed.at, s.filed.by)
}

/// The acceptance situation, and — when there is none — the exact command
/// that would create one. Naming it is not performing it.
fn acceptance_block(s: &Screen) -> String {
    let mut out = String::new();
    let lines: Vec<String> = s.acceptances.iter().map(acceptance_line).collect();
    list(&mut out, "acceptances", &lines, "none on record");
    if !s.acceptances.iter().any(|a| a.standing == "live") {
        field(&mut out, "to accept", &format!("ledger accept {}", s.decision));
    }
    out
}

fn acceptance_line(a: &AcceptanceLine) -> String {
    let mut line =
        format!("{} signed {} on {} ({})", a.actor, a.version, a.at, a.standing);
    if let Some(expires) = &a.expires_at {
        line.push_str(&format!(", expires {expires}"));
    }
    line
}
