//! Open-questions facilitation derivation.
//!
//! The driver that keeps the 60 minutes structured: at any moment the model
//! asks "what's incomplete?" and gets back the exact gaps phrased as
//! questions for the room. Blocking SHACL violations appear here too, but
//! most entries are softer completeness prompts the shapes do not block on.

use serde::Serialize;

use super::ids::NodeKind;
use super::model::DomainGraph;
use super::validate::validate_graph;

/// Which half of the model to surface gaps for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    All,
    Structure,
    Behaviour,
}

impl Focus {
    pub fn parse(s: &str) -> Self {
        match s {
            "structure" => Self::Structure,
            "behaviour" => Self::Behaviour,
            _ => Self::All,
        }
    }
}

/// One question to put to the room.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Question {
    /// `violation` (blocks finalize) or `gap` (completeness prompt).
    pub severity: String,
    /// Which half this belongs to: `structure` | `behaviour`.
    pub half: String,
    /// The node the question is about, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
    /// The question phrased for the facilitator.
    pub question: String,
}

/// Return the open questions, optionally limited to one half of the model.
pub fn open_questions(graph: &DomainGraph, focus: Focus) -> Vec<Question> {
    let mut qs = Vec::new();
    for v in validate_graph(graph) {
        qs.push(Question {
            severity: "violation".into(),
            half: half_of(&v.path),
            focus: Some(v.focus),
            question: v.message,
        });
    }
    structure_gaps(graph, &mut qs);
    behaviour_gaps(graph, &mut qs);
    qs.into_iter().filter(|q| matches(focus, &q.half)).collect()
}

fn matches(focus: Focus, half: &str) -> bool {
    match focus {
        Focus::All => true,
        Focus::Structure => half == "structure",
        Focus::Behaviour => half == "behaviour",
    }
}

fn half_of(path: &str) -> String {
    match path {
        "changes" | "targets" | "emits" | "projects" => "behaviour".into(),
        _ => "structure".into(),
    }
}

fn gap(half: &str, focus: Option<&str>, q: String) -> Question {
    Question { severity: "gap".into(), half: half.into(), focus: focus.map(str::to_string), question: q }
}

fn structure_gaps(graph: &DomainGraph, qs: &mut Vec<Question>) {
    for c in &graph.contexts {
        let has_entity = graph.entities.iter().any(|e| e.context == c.id);
        if !has_entity {
            qs.push(gap("structure", Some(&c.id),
                format!("Context '{}' has no entities yet — what concepts live here?", c.label)));
        }
    }
    for e in &graph.entities {
        let linked = graph.relations.iter().any(|r| r.from == e.id || r.to == e.id);
        if !linked {
            qs.push(gap("structure", Some(&e.id),
                format!("Entity '{}' has no relations — how does it connect to other concepts?", e.label)));
        }
    }
    for r in &graph.relations {
        for end in [&r.from, &r.to] {
            if !graph.is_kind(end, NodeKind::Entity) {
                qs.push(gap("structure", Some(&r.id),
                    format!("Relation '{}' references '{}', which is not an entity — define it or fix the link.", r.id, end)));
            }
        }
    }
    if graph.contexts.len() >= 2 && graph.context_mappings.is_empty() {
        qs.push(gap("structure", None,
            format!("You have {} contexts but no context mappings — do any concepts correspond across them (e.g. is a User a Customer)?", graph.contexts.len())));
    }
}

fn behaviour_gaps(graph: &DomainGraph, qs: &mut Vec<Question>) {
    for e in &graph.entities {
        if e.is_aggregate_root {
            let targeted = graph.commands.iter().any(|c| c.targets == e.id);
            if !targeted {
                qs.push(gap("behaviour", Some(&e.id),
                    format!("Aggregate '{}' has no commands — what intents act on it?", e.label)));
            }
        }
    }
    if !graph.commands.is_empty() && graph.flows.is_empty() {
        qs.push(gap("behaviour", None,
            "You have commands but no flows — walk through the core behaviour as a timeline.".into()));
    }
    for f in &graph.flows {
        let shows_view = f.steps.iter().any(|s| graph.is_kind(s, NodeKind::ReadModel));
        if !shows_view {
            qs.push(gap("behaviour", Some(&f.id),
                format!("Flow '{}' shows no read model — what does the user see along the way?", f.label)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::model::*;

    #[test]
    fn empty_context_surfaces_a_gap() {
        let mut g = DomainGraph::default();
        g.contexts.push(BoundedContext { id: "Billing".into(), label: "Billing".into(), ..Default::default() });
        let qs = open_questions(&g, Focus::All);
        assert!(qs.iter().any(|q| q.question.contains("no entities")));
    }

    #[test]
    fn focus_filters_halves() {
        let mut g = DomainGraph::default();
        g.contexts.push(BoundedContext { id: "Billing".into(), label: "Billing".into(), ..Default::default() });
        g.entities.push(Entity { id: "Inv".into(), label: "Invoice".into(), context: "Billing".into(), definition: "d".into(), is_aggregate_root: true, ..Default::default() });
        let beh = open_questions(&g, Focus::Behaviour);
        assert!(beh.iter().all(|q| q.half == "behaviour"));
        assert!(beh.iter().any(|q| q.question.contains("no commands")));
    }
}
