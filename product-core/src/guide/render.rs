//! Render [`Guidance`] as a terminal-friendly text block.

use super::Guidance;

/// Render guidance: a progress checklist, the headline + concept, and the
/// concrete next command(s). JSON rendering is derived from `Serialize`.
pub fn render_text(g: &Guidance) -> String {
    let mut out = String::from("── Your framework journey ──\n");
    for (label, done) in &g.progress {
        let mark = if *done { "[x]" } else { "[ ]" };
        out.push_str(&format!("  {mark} {label}\n"));
    }
    out.push('\n');
    out.push_str(&g.headline);
    out.push_str("\n\n");
    out.push_str("Why this matters: ");
    out.push_str(&g.concept);
    out.push_str("\n\nNext:\n");
    for step in &g.next_steps {
        out.push_str(&format!("  $ {}\n      {}\n", step.command, step.why));
    }
    out
}
