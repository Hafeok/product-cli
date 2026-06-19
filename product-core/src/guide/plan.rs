//! The pure stage-decision logic: framework state in, guidance out.

use super::{FrameworkState, Guidance, NextStep, Stage};

/// Decide where the user is and what to do next. Pure — no I/O.
///
/// The stages form a strict order (What → conformant What → How → slice →
/// deliverable → build); the first unmet step is the current stage, so the
/// guidance always points at the single next move.
pub fn guide(state: &FrameworkState) -> Guidance {
    let stage = stage_of(state);
    Guidance {
        stage,
        headline: headline(state, stage),
        concept: concept(stage).to_string(),
        next_steps: next_steps(state, stage),
        progress: progress(state),
    }
}

fn stage_of(s: &FrameworkState) -> Stage {
    if s.what_total == 0 {
        Stage::CaptureWhat
    } else if s.violations > 0 {
        Stage::FixWhat
    } else if !s.has_how {
        Stage::AuthorHow
    } else if s.slices == 0 {
        Stage::CarveSlice
    } else if s.deliverables == 0 {
        Stage::WrapDeliverable
    } else {
        Stage::BuildIt
    }
}

fn headline(s: &FrameworkState, stage: Stage) -> String {
    match stage {
        Stage::CaptureWhat => "Start by capturing your product's What — its domain and behaviour.".into(),
        Stage::FixWhat => format!(
            "Your What has {} blocking conformance violation(s) — resolve them before going further.",
            s.violations
        ),
        Stage::AuthorHow => format!(
            "Your What is conformant ({} nodes). Now describe the How that realises it.",
            s.what_total
        ),
        Stage::CarveSlice => "The How is scaffolded. Carve a delivery slice over your event model.".into(),
        Stage::WrapDeliverable => format!(
            "You have {} slice(s). Wrap one as a deliverable with its acceptance.",
            s.slices
        ),
        Stage::BuildIt => format!(
            "You have {} deliverable(s). Make behaviour executable, then build.",
            s.deliverables
        ),
    }
}

fn concept(stage: Stage) -> &'static str {
    match stage {
        Stage::CaptureWhat => "The What is your product's meaning: bounded contexts, the entities inside them, and the behaviour — the commands users issue and the events those cause. It is agreed before any How.",
        Stage::FixWhat => "The What is type-checked: every event must change a real entity, every command must target an aggregate and emit an event. Violations mean behaviour references structure that does not exist.",
        Stage::AuthorHow => "The How realises the What without changing its meaning: decisions and principles (the Why), contracts, and the repository layout model. Same What can drive several Hows.",
        Stage::CarveSlice => "A slice is a named, buildable section of the event model — an anchor (a command, context, or flow) plus its neighbourhood — the unit a deliverable is built from.",
        Stage::WrapDeliverable => "A deliverable is one slice plus its acceptance criteria; it is 'done' (§7.2) only when every criterion has a passing verdict.",
        Stage::BuildIt => "Where behaviour is interesting, a Decider makes it executable and is simulated sound before any code. `build` then assembles the frozen SPMC context and runs the realisation against the verification gates.",
    }
}

fn next_steps(s: &FrameworkState, stage: Stage) -> Vec<NextStep> {
    match stage {
        Stage::CaptureWhat => vec![
            NextStep::new(
                format!("product author domain {}", s.product),
                "Run a facilitated capture session (an LLM scribes your domain into the graph).",
            ),
            NextStep::new(
                "product domain new context <Name> --purpose \"...\"",
                "Or author by hand: start with a bounded context, then entities, then behaviour.",
            ),
        ],
        Stage::FixWhat => vec![NextStep::new(
            "product domain validate",
            "List the violations; add the missing relations (e.g. an event's --changes, a command's --targets/--emits).",
        )],
        Stage::AuthorHow => vec![NextStep::new(
            format!("product how init {}", s.product),
            "Scaffold a starter how-contract.yaml, then `product how add decision|principle|...`.",
        )],
        Stage::CarveSlice => vec![NextStep::new(
            "product slice new <id> --anchor <command|context|flow>",
            "Anchor a slice at a node in your event model (e.g. --anchor a command id).",
        )],
        Stage::WrapDeliverable => vec![NextStep::new(
            "product deliverable new <id> --slice <slice-id>",
            "Wrap a slice as a deliverable, then `product deliverable accept` its criteria.",
        )],
        Stage::BuildIt => vec![
            NextStep::new(
                "product decider derive <aggregate>",
                "Derive a Decider for an aggregate and `product decider simulate` it sound before realisation.",
            ),
            NextStep::new(
                "product build <deliverable>",
                "Assemble the frozen build context and run the realisation against the verification gates.",
            ),
        ],
    }
}

/// The journey checklist in order; each item done once its stage is passed.
fn progress(s: &FrameworkState) -> Vec<(String, bool)> {
    vec![
        ("Captured a What model".into(), s.what_total > 0),
        ("What is conformant".into(), s.what_total > 0 && s.violations == 0),
        ("How contract scaffolded".into(), s.has_how),
        ("Delivery slice carved".into(), s.slices > 0),
        ("Deliverable wrapped".into(), s.deliverables > 0),
    ]
}
