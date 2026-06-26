//! Turtle seed parsing into the typed model.
//!
//! Reconstructs a [`DomainGraph`] from a prior session's exported Turtle so
//! `session_start` can seed from it. Backed by Oxigraph; reads the `pf:`
//! classes the What-capture session produces.

use std::collections::HashMap;

use oxigraph::io::RdfFormat;
use oxigraph::model::Term;
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;

use super::model::*;
use crate::error::{ProductError, Result};

const PF: &str = "https://productframework.org/ns#";

/// Parse Turtle into a typed domain graph (best-effort over the What classes).
pub fn from_turtle(turtle: &str) -> Result<DomainGraph> {
    let store = Store::new().map_err(|e| ProductError::Internal(format!("oxigraph store: {}", e)))?;
    store
        .load_from_reader(RdfFormat::Turtle, turtle.as_bytes())
        .map_err(|e| ProductError::ConfigError(format!("could not parse seed Turtle: {}", e)))?;

    let mut g = DomainGraph::default();
    let glossary = multi(&store, "pf:BoundedContext", "pf:ubiquitousTerm")?;
    for row in select(&store, "?s ?label ?purpose", "?s a pf:BoundedContext . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:purpose ?purpose }")? {
        let id = local(row.get("s"));
        g.contexts.push(BoundedContext {
            glossary: glossary.get(&id).cloned().unwrap_or_default(),
            id, label: lit(row.get("label")), purpose: opt(row.get("purpose")),
        });
    }
    parse_entities(&store, &mut g)?;
    for row in select(&store, "?s ?label ?ctx ?def", "?s a pf:ValueObject . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:definition ?def }")? {
        g.value_objects.push(ValueObject { id: local(row.get("s")), label: lit(row.get("label")), context: local(row.get("ctx")), definition: opt(row.get("def")) });
    }
    parse_relations(&store, &mut g)?;
    parse_invariants(&store, &mut g)?;
    parse_mappings(&store, &mut g)?;
    parse_commands(&store, &mut g)?;
    for row in select(&store, "?s ?label ?ctx ?changes", "?s a pf:Event . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:changes ?changes }")? {
        g.events.push(Event { id: local(row.get("s")), label: lit(row.get("label")), context: local(row.get("ctx")), changes: local(row.get("changes")) });
    }
    parse_read_models(&store, &mut g)?;
    parse_flows(&store, &mut g)?;
    parse_systems(&store, &mut g)?;
    parse_triggers(&store, &mut g)?;
    parse_unreifiable(&store, &mut g)?;
    super::seed_ui::parse_ui(&store, &mut g)?;
    super::seed_data::parse_data(&store, &mut g)?;
    super::seed_canon::canonicalize(&mut g);
    Ok(g)
}

fn parse_unreifiable(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?aio ?class ?rat",
        "?s a pf:UnreifiableRule . OPTIONAL { ?s pf:reifies ?aio } OPTIONAL { ?s pf:unreifiableIn ?class } OPTIONAL { ?s pf:rationale ?rat }")? {
        g.unreifiable_rules.push(UnreifiableRule {
            id: local(row.get("s")), aio: local(row.get("aio")),
            class: lit(row.get("class")), rationale: opt(row.get("rat")),
        });
    }
    Ok(())
}

fn parse_triggers(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?label ?source ?issues ?watches ?from",
        "?s a pf:Trigger . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:source ?source } OPTIONAL { ?s pf:issues ?issues } OPTIONAL { ?s pf:watches ?watches } OPTIONAL { ?s pf:translatesFrom ?from }")? {
        g.triggers.push(Trigger {
            id: local(row.get("s")), label: lit(row.get("label")),
            source: lit(row.get("source")), issues: local(row.get("issues")),
            watches: opt(row.get("watches")).map(|_| local(row.get("watches"))),
            translates_from: opt(row.get("from")).map(|_| local(row.get("from"))),
        });
    }
    Ok(())
}

fn parse_systems(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let platforms = multi(store, "pf:System", "pf:targetsPlatform")?;
    let classes = multi(store, "pf:System", "pf:targetsClass")?;
    for row in select(store, "?s ?label ?kind ?purpose ?root",
        "?s a pf:System . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:systemKind ?kind } OPTIONAL { ?s pf:purpose ?purpose } OPTIONAL { ?s pf:rootsAt ?root }")? {
        let id = local(row.get("s"));
        g.systems.push(System {
            target_platforms: platforms.get(&id).cloned().unwrap_or_default(),
            target_classes: classes.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: lit(row.get("label")),
            kind: lit(row.get("kind")), purpose: lit(row.get("purpose")),
            root: opt(row.get("root")).map(|_| local(row.get("root"))),
        });
    }
    Ok(())
}

fn parse_entities(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let mut attrs: HashMap<String, Vec<Attribute>> = HashMap::new();
    for row in select(store, "?s ?name ?ty",
        "?s a pf:Entity ; pf:hasAttribute ?b . ?b pf:attrName ?name . OPTIONAL { ?b pf:attrType ?ty }")? {
        attrs.entry(local(row.get("s"))).or_default().push(Attribute {
            name: lit(row.get("name")), ty: opt(row.get("ty")),
        });
    }
    for row in select(store, "?s ?label ?def ?ctx ?agg ?identity",
        "?s a pf:Entity . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:definition ?def } OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:isAggregateRoot ?agg } OPTIONAL { ?s pf:identity ?identity }")? {
        let id = local(row.get("s"));
        g.entities.push(Entity {
            attributes: attrs.remove(&id).unwrap_or_default(),
            id, label: lit(row.get("label")), context: local(row.get("ctx")),
            definition: lit(row.get("def")), identity: opt(row.get("identity")),
            is_aggregate_root: lit(row.get("agg")) == "true",
        });
    }
    Ok(())
}

fn parse_relations(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?label ?from ?to ?card ?rat",
        "?s a pf:Relation . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:from ?from } OPTIONAL { ?s pf:to ?to } OPTIONAL { ?s pf:cardinality ?card } OPTIONAL { ?s pf:rationale ?rat }")? {
        g.relations.push(Relation {
            id: local(row.get("s")), label: opt(row.get("label")), from: local(row.get("from")),
            to: local(row.get("to")), cardinality: lit(row.get("card")), rationale: lit(row.get("rat")),
        });
    }
    Ok(())
}

fn parse_invariants(store: &Store, g: &mut DomainGraph) -> Result<()> {
    for row in select(store, "?s ?stmt ?ctx ?applies",
        "?s a pf:Invariant . OPTIONAL { ?s pf:statement ?stmt } OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:appliesTo ?applies }")? {
        g.invariants.push(Invariant {
            id: local(row.get("s")), statement: lit(row.get("stmt")),
            context: opt(row.get("ctx")).map(|_| local(row.get("ctx"))),
            applies_to: opt(row.get("applies")).map(|_| local(row.get("applies"))),
        });
    }
    Ok(())
}

fn parse_mappings(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let sides = multi(store, "pf:ContextMapping", "pf:mapsTo")?;
    for row in select(store, "?s ?kind ?rat",
        "?s a pf:ContextMapping . OPTIONAL { ?s pf:mappingKind ?kind } OPTIONAL { ?s pf:rationale ?rat }")? {
        let id = local(row.get("s"));
        let mapped = sides.get(&id).cloned().unwrap_or_default();
        g.context_mappings.push(ContextMapping {
            id: id.clone(),
            concept_a: mapped.first().cloned().unwrap_or_default(),
            concept_b: mapped.get(1).cloned().unwrap_or_default(),
            kind: opt(row.get("kind")), rationale: lit(row.get("rat")),
        });
    }
    Ok(())
}

fn parse_commands(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let emits = multi(store, "pf:Command", "pf:emits")?;
    for row in select(store, "?s ?label ?ctx ?targets",
        "?s a pf:Command . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:inContext ?ctx } OPTIONAL { ?s pf:targets ?targets }")? {
        let id = local(row.get("s"));
        g.commands.push(Command {
            emits: emits.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: lit(row.get("label")), context: local(row.get("ctx")), targets: local(row.get("targets")),
        });
    }
    Ok(())
}

fn parse_read_models(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let projects = multi(store, "pf:ReadModel", "pf:projects")?;
    let states = multi(store, "pf:ReadModel", "pf:hasState")?;
    for row in select(store, "?s ?label", "?s a pf:ReadModel . OPTIONAL { ?s rdfs:label ?label }")? {
        let id = local(row.get("s"));
        g.read_models.push(ReadModel {
            projects: projects.get(&id).cloned().unwrap_or_default(),
            states: states.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: lit(row.get("label")),
        });
    }
    Ok(())
}

fn parse_flows(store: &Store, g: &mut DomainGraph) -> Result<()> {
    let steps = multi(store, "pf:Flow", "pf:contains")?;
    for row in select(store, "?s ?label ?system ?entry", "?s a pf:Flow . OPTIONAL { ?s rdfs:label ?label } OPTIONAL { ?s pf:systemOf ?system } OPTIONAL { ?s pf:entryPage ?entry }")? {
        let id = local(row.get("s"));
        g.flows.push(Flow {
            steps: steps.get(&id).cloned().unwrap_or_default(),
            id: id.clone(), label: lit(row.get("label")),
            entry_page: opt(row.get("entry")).map(|_| local(row.get("entry"))),
            system: opt(row.get("system")).map(|_| local(row.get("system"))),
        });
    }
    Ok(())
}

// --- Oxigraph helpers -----------------------------------------------------

pub(super) type Row = HashMap<String, Term>;

pub(super) fn select(store: &Store, vars: &str, body: &str) -> Result<Vec<Row>> {
    let q = format!("PREFIX pf: <{}> PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#> SELECT {} WHERE {{ {} }}", PF, vars, body);
    run(store, &q)
}

fn run(store: &Store, q: &str) -> Result<Vec<Row>> {
    let results = store.query(q).map_err(|e| ProductError::Internal(format!("seed query: {}", e)))?;
    let mut rows = Vec::new();
    if let QueryResults::Solutions(solutions) = results {
        let vars: Vec<String> = solutions.variables().iter().map(|v| v.as_str().to_string()).collect();
        for sol in solutions {
            let sol = sol.map_err(|e| ProductError::Internal(format!("seed row: {}", e)))?;
            let mut row = HashMap::new();
            for var in &vars {
                if let Some(term) = sol.get(var.as_str()) {
                    row.insert(var.clone(), term.clone());
                }
            }
            rows.push(row);
        }
    }
    Ok(rows)
}

/// Collect multi-valued objects of `predicate` for each subject of `class`,
/// keyed by the subject's local id.
pub(super) fn multi(store: &Store, class: &str, predicate: &str) -> Result<HashMap<String, Vec<String>>> {
    let q = format!("PREFIX pf: <{}> SELECT ?s ?o WHERE {{ ?s a {} . ?s {} ?o }}", PF, class, predicate);
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for row in run(store, &q)? {
        let s = local(row.get("s"));
        map.entry(s).or_default().push(local(row.get("o")));
    }
    Ok(map)
}

/// The local id of a term: the fragment after `#`, else the last path segment.
pub(super) fn local(term: Option<&Term>) -> String {
    match term {
        Some(Term::NamedNode(n)) => {
            let s = n.as_str();
            s.rsplit(['#', '/']).next().unwrap_or(s).to_string()
        }
        Some(Term::Literal(l)) => l.value().to_string(),
        _ => String::new(),
    }
}

/// The literal value of a term (empty string if absent).
pub(super) fn lit(term: Option<&Term>) -> String {
    match term {
        Some(Term::Literal(l)) => l.value().to_string(),
        Some(Term::NamedNode(n)) => n.as_str().rsplit(['#', '/']).next().unwrap_or("").to_string(),
        _ => String::new(),
    }
}

/// `Some(value)` if the term is present and non-empty, else `None`.
pub(super) fn opt(term: Option<&Term>) -> Option<String> {
    let v = lit(term);
    if v.is_empty() { None } else { Some(v) }
}

#[cfg(test)]
#[path = "seed_tests.rs"]
mod tests;
