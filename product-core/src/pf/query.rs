//! Graph queries for the What model.
//!
//! Convenience queries keyed by a node ("what happens to X", "contents of
//! context C", "relations of entity E", "flows in context C") plus a raw
//! SPARQL SELECT escape hatch backed by Oxigraph over the Turtle export.

use serde_json::{json, Value};

use super::ids::NodeKind;
use super::model::DomainGraph;
use super::turtle::to_turtle;
use crate::error::{ProductError, Result};

/// What changes/acts on an entity: events that change it, commands that
/// target it, read models that project it.
pub fn what_happens_to(graph: &DomainGraph, id: &str) -> Value {
    let events: Vec<&str> = graph.events.iter().filter(|e| e.changes == id).map(|e| e.id.as_str()).collect();
    let commands: Vec<&str> = graph.commands.iter().filter(|c| c.targets == id).map(|c| c.id.as_str()).collect();
    let views: Vec<&str> = graph.read_models.iter().filter(|r| r.projects.iter().any(|p| p == id)).map(|r| r.id.as_str()).collect();
    json!({ "about": id, "changedByEvents": events, "targetedByCommands": commands, "projectedByReadModels": views })
}

/// Everything declared inside a bounded context.
pub fn context_contents(graph: &DomainGraph, id: &str) -> Value {
    let entities: Vec<&str> = graph.entities.iter().filter(|e| e.context == id).map(|e| e.id.as_str()).collect();
    let value_objects: Vec<&str> = graph.value_objects.iter().filter(|v| v.context == id).map(|v| v.id.as_str()).collect();
    let commands: Vec<&str> = graph.commands.iter().filter(|c| c.context == id).map(|c| c.id.as_str()).collect();
    let events: Vec<&str> = graph.events.iter().filter(|e| e.context == id).map(|e| e.id.as_str()).collect();
    json!({ "context": id, "entities": entities, "valueObjects": value_objects, "commands": commands, "events": events })
}

/// Relations touching an entity, split by direction.
pub fn entity_relations(graph: &DomainGraph, id: &str) -> Value {
    let outgoing: Vec<Value> = graph.relations.iter().filter(|r| r.from == id)
        .map(|r| json!({ "id": r.id, "to": r.to, "cardinality": r.cardinality })).collect();
    let incoming: Vec<Value> = graph.relations.iter().filter(|r| r.to == id)
        .map(|r| json!({ "id": r.id, "from": r.from, "cardinality": r.cardinality })).collect();
    json!({ "entity": id, "outgoing": outgoing, "incoming": incoming })
}

/// Flows whose steps reference any node living in the given context.
pub fn flows_in_context(graph: &DomainGraph, id: &str) -> Value {
    let in_ctx = |node: &str| -> bool {
        graph.commands.iter().any(|c| c.id == node && c.context == id)
            || graph.events.iter().any(|e| e.id == node && e.context == id)
    };
    let flows: Vec<&str> = graph.flows.iter()
        .filter(|f| f.steps.iter().any(|s| in_ctx(s)))
        .map(|f| f.id.as_str())
        .collect();
    json!({ "context": id, "flows": flows })
}

/// The node's own fields, serialized, if it exists in the graph.
pub fn node_value(graph: &DomainGraph, id: &str) -> Option<Value> {
    macro_rules! find {
        ($vec:expr) => {
            if let Some(n) = $vec.iter().find(|n| n.id == id) {
                return serde_json::to_value(n).ok();
            }
        };
    }
    find!(graph.contexts);
    find!(graph.entities);
    find!(graph.value_objects);
    find!(graph.relations);
    find!(graph.invariants);
    find!(graph.context_mappings);
    find!(graph.commands);
    find!(graph.events);
    find!(graph.read_models);
    find!(graph.wireframe_steps);
    find!(graph.flows);
    find!(graph.systems);
    find!(graph.triggers);
    find!(graph.unreifiable_rules);
    find!(graph.products);
    find!(graph.journeys);
    find!(graph.quality_demands);
    None
}

/// Describe a node's links in and out (the `about` form).
pub fn describe(graph: &DomainGraph, id: &str) -> Result<Value> {
    match graph.kind_of(id) {
        Some(NodeKind::Entity) => Ok(json!({
            "id": id, "kind": "Entity",
            "happensTo": what_happens_to(graph, id),
            "relations": entity_relations(graph, id),
        })),
        Some(NodeKind::BoundedContext) => Ok(context_contents(graph, id)),
        Some(kind) => Ok(json!({ "id": id, "kind": kind.class_name() })),
        None => Err(ProductError::NotFound(format!("no node with id {:?} in the graph", id))),
    }
}

/// Run a raw SPARQL SELECT over the Turtle export via Oxigraph.
pub fn sparql(graph: &DomainGraph, product: &str, query: &str) -> Result<Value> {
    use oxigraph::io::RdfFormat;
    use oxigraph::sparql::QueryResults;
    use oxigraph::store::Store;

    let store = Store::new()
        .map_err(|e| ProductError::Internal(format!("oxigraph store: {}", e)))?;
    let ttl = to_turtle(graph, product);
    store
        .load_from_reader(RdfFormat::Turtle, ttl.as_bytes())
        .map_err(|e| ProductError::Internal(format!("load turtle: {}", e)))?;
    let results = store
        .query(query)
        .map_err(|e| ProductError::ConfigError(format!("SPARQL error: {}", e)))?;
    match results {
        QueryResults::Solutions(solutions) => solutions_to_json(solutions),
        QueryResults::Boolean(b) => Ok(json!({ "boolean": b })),
        QueryResults::Graph(_) => Ok(json!({ "note": "graph results are not supported; use SELECT" })),
    }
}

fn solutions_to_json(solutions: oxigraph::sparql::QuerySolutionIter) -> Result<Value> {
    let vars: Vec<String> = solutions.variables().iter().map(|v| v.as_str().to_string()).collect();
    let mut rows = Vec::new();
    for solution in solutions {
        let solution = solution.map_err(|e| ProductError::Internal(format!("sparql row: {}", e)))?;
        let mut obj = serde_json::Map::new();
        for var in &vars {
            let val = solution.get(var.as_str()).map(|t| t.to_string()).unwrap_or_default();
            obj.insert(var.clone(), Value::String(val));
        }
        rows.push(Value::Object(obj));
    }
    Ok(json!({ "vars": vars, "rows": rows }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pf::model::*;

    fn sample() -> DomainGraph {
        let mut g = DomainGraph::default();
        g.contexts.push(BoundedContext { id: "Tasks".into(), label: "Tasks".into(), ..Default::default() });
        g.entities.push(Entity { id: "Task".into(), label: "Task".into(), context: "Tasks".into(), definition: "d".into(), ..Default::default() });
        g.events.push(Event { fields: vec![], id: "Done".into(), label: "Done".into(), context: "Tasks".into(), changes: "Task".into() });
        g.commands.push(Command { fields: vec![], id: "Complete".into(), label: "Complete".into(), context: "Tasks".into(), targets: "Task".into(), emits: vec!["Done".into()] });
        g
    }

    #[test]
    fn what_happens_to_entity() {
        let v = what_happens_to(&sample(), "Task");
        assert_eq!(v["changedByEvents"][0], "Done");
        assert_eq!(v["targetedByCommands"][0], "Complete");
    }

    #[test]
    fn sparql_select_runs() {
        let v = sparql(&sample(), "demo", "SELECT ?s WHERE { ?s a <https://productframework.org/ns#Entity> }").expect("query");
        assert_eq!(v["rows"].as_array().expect("rows").len(), 1);
    }

    #[test]
    fn describe_unknown_errs() {
        assert!(describe(&sample(), "ghost").is_err());
    }
}
