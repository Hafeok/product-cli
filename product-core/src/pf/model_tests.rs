//! Unit tests for the What-graph node model.

use super::*;

#[test]
fn kind_lookup_and_counts() {
    let mut g = DomainGraph::default();
    g.contexts.push(BoundedContext { id: "Ctx".into(), label: "Ctx".into(), ..Default::default() });
    g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Ctx".into(), definition: "d".into(), ..Default::default() });
    assert_eq!(g.kind_of("Task"), Some(NodeKind::Entity));
    assert_eq!(g.kind_of("Ctx"), Some(NodeKind::BoundedContext));
    assert_eq!(g.kind_of("nope"), None);
    assert!(g.is_kind("Task", NodeKind::Entity));
    assert!(!g.is_kind("Task", NodeKind::Event));
    assert_eq!(g.node_count(), 2);
    assert!(g.contains("Ctx"));
}
