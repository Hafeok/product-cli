//! Turtle projection of a Decider into the What graph (§3.3 / §9 links).
//!
//! Appends the Decider's `decidesFor`/`handles`/`emitsEvent` edges to the What
//! graph projection so the §3.3 conformance rules can cross-check the authored
//! signature against the commands + events the model derives. Reuses the same
//! `d:` instance namespace as `to_turtle` so the references line up.

use super::decider::Decider;
use super::model::DomainGraph;
use super::turtle::to_turtle;

/// Project the What graph plus a Decider into one Turtle document. The Decider
/// triples are appended after `to_turtle`'s prefixes + body, sharing the same
/// `pf:`/`d:` prefixes (ids are emitted raw, exactly as `to_turtle` emits them).
pub fn decider_to_turtle(graph: &DomainGraph, decider: &Decider, product: &str) -> String {
    let mut out = to_turtle(graph, product);
    out.push_str(&format!("d:{} a pf:Decider", decider.id));
    if !decider.decides_for.is_empty() {
        out.push_str(&format!(" ;\n  pf:decidesFor d:{}", decider.decides_for));
    }
    for c in &decider.handles {
        out.push_str(&format!(" ;\n  pf:handles d:{c}"));
    }
    for e in &decider.emits {
        out.push_str(&format!(" ;\n  pf:emitsEvent d:{e}"));
    }
    out.push_str(" .\n\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::model::*;

    #[test]
    fn projects_decider_edges_onto_the_what_graph() {
        let mut g = DomainGraph::default();
        g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "C".into(), definition: "d".into(), ..Default::default() });
        let d = Decider { id: "task-decider".into(), decides_for: "Task".into(), handles: vec!["CompleteTask".into()], emits: vec!["TaskDone".into()], ..Default::default() };
        let ttl = decider_to_turtle(&g, &d, "demo");
        assert!(ttl.contains("d:task-decider a pf:Decider"));
        assert!(ttl.contains("pf:decidesFor d:Task"));
        assert!(ttl.contains("pf:handles d:CompleteTask"));
        assert!(ttl.contains("pf:emitsEvent d:TaskDone"));
        // the What body is still present (combined projection)
        assert!(ttl.contains("d:Task a pf:Entity"));
    }
}
