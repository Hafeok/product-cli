//! Emit a self-contained SPMC prompt for an autonomous coding agent (§5).
//!
//! Where `build::assemble` produces the frozen context a dispatched worker
//! consumes, this wraps that context in an operating contract a `claude -p`
//! session executes directly: realise every work unit in dependency order at its
//! declared path, then make the verification pass. Pure — no I/O.

use super::build::assemble;
use super::decider::Decider;
use super::deliverable::Deliverable;
use super::how::HowContract;
use super::model::DomainGraph;
use super::schedule::layers;
use super::slice::Slice;
use super::verify;
use super::work_unit::WorkUnit;

const CONTRACT: &str = "⟦Ω:SPMC⟧ You are a Claude Code session realising ONE delivery feature in this repository. The What and How below are **frozen** — realise them, do not re-decide or re-derive them.

## Operating contract
- Produce each work unit's artifact at its **exact** declared path; never invent a location.
- Obey every Principle, apply the named Patterns, and conform to the Application contract (language, layering).
- Build the work units **in the given order**. Once an artifact is written, treat it as read-only for later units — especially tests/oracles: satisfy them, never edit them.
- Do not edit anything under `.product/` (the spec) or any file outside a work unit's declared path.
- When all artifacts exist, run every Verify command and make it pass before you finish.
";

/// Build a self-contained SPMC prompt for a `claude -p` session to realise the
/// whole deliverable in-repo and self-verify.
pub fn emit_session_spmc(
    d: &Deliverable,
    slice: &Slice,
    graph: &DomainGraph,
    how: Option<&HowContract>,
    deciders: &[Decider],
    units: &[WorkUnit],
    product: &str,
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# SPMC — build deliverable `{}`\n\n", d.id));
    out.push_str(CONTRACT);
    out.push_str("\n---\n\n");

    // The frozen What / How / Behaviour / Acceptance context (shared with dispatch).
    out.push_str(&assemble(d, slice, graph, how, deciders, product));
    out.push_str("\n\n---\n\n");

    out.push_str("## Build plan — produce these artifacts, in order\n\n");
    if units.is_empty() {
        out.push_str("_(no work units — dispatch a cell first: `product cell dispatch --bind …`)_\n");
    } else {
        render_units(units, &mut out);
    }
    out.push_str("\n---\n\n");

    out.push_str("## Verify — every command must exit 0 before you finish\n\n");
    render_verify(d, &mut out);

    out.push_str("\n## Done when\n");
    out.push_str("- every artifact above exists at its declared path, and\n");
    out.push_str("- every Verify command exits 0.\n");
    out
}

fn render_units(units: &[WorkUnit], out: &mut String) {
    for (li, layer) in layers(units).iter().enumerate() {
        out.push_str(&format!("### Layer {} (finish before later layers)\n\n", li + 1));
        for &i in layer {
            let wu = &units[i];
            out.push_str(&format!("#### `{}` → write `{}`\n\n{}\n", wu.id, wu.produces.path, wu.prompt));
            if !wu.applies.is_empty() {
                out.push_str(&format!("\n_apply:_ {}\n", wu.applies.join(", ")));
            }
            if !wu.context.derived_from.is_empty() {
                out.push_str(&format!("_derives from (read-only once written):_ {}\n", wu.context.derived_from.join(", ")));
            }
            out.push('\n');
        }
    }
}

fn render_verify(d: &Deliverable, out: &mut String) {
    let steps = verify::plan(d);
    if steps.is_empty() {
        out.push_str("_(no acceptance criterion carries a runner — bind one with `product deliverable runner …` so the build self-verifies; otherwise these are judged manually:)_\n");
        for a in &d.acceptance {
            out.push_str(&format!("- {}: {}\n", a.id, a.statement));
        }
        return;
    }
    for s in steps {
        out.push_str(&format!("- `{}`: `{} {}`\n", s.criterion, s.program, s.args.join(" ")));
    }
}

#[cfg(test)]
#[path = "build_spmc_tests.rs"]
mod tests;
