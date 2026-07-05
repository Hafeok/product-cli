//! Argument-to-fragment handlers for the mutating domain tools.
//!
//! Each `add_*` handler builds a typed node from the JSON arguments, calls the
//! matching `product_core::pf::ops` operation (which validates in-loop), and
//! returns the `{ ok, node, violations[] }` result as JSON.

use product_core::pf::model::*;
use product_core::pf::ops::{self, OpResult};
use product_core::pf::session::DomainSession;
use serde_json::Value;

use super::args::{bool_flag, opt_str, req_str, str_array};

fn to_value(r: OpResult) -> Result<Value, String> {
    serde_json::to_value(r).map_err(|e| format!("serialize result: {}", e))
}

fn attributes(a: &Value) -> Vec<Attribute> {
    attr_list(a, "attributes")
}

/// Parse a `[{name, type?}]` array under `key` (entity attributes, or the
/// §3.2 command/event payload `fields`).
fn attr_list(a: &Value, key: &str) -> Vec<Attribute> {
    a.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|o| {
                    let name = o.get("name").and_then(|v| v.as_str())?.to_string();
                    let ty = o.get("type").and_then(|v| v.as_str()).map(str::to_string);
                    Some(Attribute { name, ty })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn add_bounded_context(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let c = BoundedContext {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        purpose: opt_str(a, "purpose"),
        glossary: str_array(a, "glossary"),
    };
    to_value(ops::add_bounded_context(s, c))
}

pub fn add_entity(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let e = Entity {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        context: req_str(a, "context")?,
        definition: req_str(a, "definition")?,
        identity: opt_str(a, "identity"),
        is_aggregate_root: bool_flag(a, "is_aggregate_root"),
        attributes: attributes(a),
    };
    to_value(ops::add_entity(s, e))
}

pub fn add_value_object(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let v = ValueObject {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        context: req_str(a, "context")?,
        definition: opt_str(a, "definition"),
    };
    to_value(ops::add_value_object(s, v))
}

pub fn add_relation(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let r = Relation {
        id: req_str(a, "id")?,
        label: opt_str(a, "label"),
        from: req_str(a, "from")?,
        to: req_str(a, "to")?,
        cardinality: req_str(a, "cardinality")?,
        rationale: req_str(a, "rationale")?,
    };
    to_value(ops::add_relation(s, r))
}

pub fn add_invariant(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let i = Invariant {
        id: req_str(a, "id")?,
        statement: req_str(a, "statement")?,
        context: opt_str(a, "context"),
        applies_to: opt_str(a, "applies_to"),
    };
    to_value(ops::add_invariant(s, i))
}

pub fn add_context_mapping(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let m = ContextMapping {
        id: req_str(a, "id")?,
        concept_a: req_str(a, "concept_a")?,
        concept_b: req_str(a, "concept_b")?,
        kind: opt_str(a, "kind"),
        rationale: req_str(a, "rationale")?,
    };
    to_value(ops::add_context_mapping(s, m))
}

pub fn add_command(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let c = Command {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        context: req_str(a, "context")?,
        targets: req_str(a, "targets")?,
        emits: str_array(a, "emits"),
        fields: attr_list(a, "fields"),
    };
    to_value(ops::add_command(s, c))
}

pub fn add_event(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let e = Event {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        context: req_str(a, "context")?,
        changes: req_str(a, "changes")?,
        fields: attr_list(a, "fields"),
    };
    to_value(ops::add_event(s, e))
}

pub fn add_read_model(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let r = ReadModel {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        projects: str_array(a, "projects"),
        ..Default::default()
    };
    to_value(ops::add_read_model(s, r))
}

pub fn add_wireframe_step(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let w = WireframeStep {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        triggers: opt_str(a, "triggers"),
        displays: opt_str(a, "displays"),
        ..Default::default()
    };
    to_value(ops::add_wireframe_step(s, w))
}

pub fn add_flow(s: &mut DomainSession, a: &Value) -> Result<Value, String> {
    let f = Flow {
        id: req_str(a, "id")?,
        label: req_str(a, "label")?,
        steps: str_array(a, "steps"),
        ..Default::default()
    };
    to_value(ops::add_flow(s, f))
}
