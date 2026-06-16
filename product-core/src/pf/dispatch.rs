//! Cell dispatch — instantiate a task type's template cells into work units.
//!
//! Dispatch binds a task type's dual-read slots to concrete values (a domain
//! slot to a real entity), then turns each template cell into a concrete,
//! frozen SPMC [`WorkUnit`]: its `derived_from` slot pointers are resolved to
//! the bound values, its context is frozen and hashed, and a rationale trace is
//! emitted. Bindings are validated against the captured What graph so a cell is
//! only ever dispatched against entities that exist.

use std::collections::BTreeMap;

use super::cell::TaskType;
use super::model::DomainGraph;
use super::provenance::content_hash;
use super::validate::Violation;
use super::work_unit::{Context, Produces, Trace, WorkUnit};

/// The outcome of a dispatch: the instantiated work units and any binding
/// violations (a non-empty blocking set means nothing should be written).
pub struct Dispatched {
    pub work_units: Vec<WorkUnit>,
    pub violations: Vec<Violation>,
}

fn v(focus: &str, path: &str, message: &str) -> Violation {
    sev(focus, path, message, "violation")
}
fn warn(focus: &str, path: &str, message: &str) -> Violation {
    sev(focus, path, message, "warning")
}
fn sev(focus: &str, path: &str, message: &str, severity: &str) -> Violation {
    Violation { focus: focus.into(), path: path.into(), message: message.into(), severity: severity.into() }
}

/// Dispatch `task` with `bindings` (slot → value), validated against `domain`.
pub fn dispatch(task: &TaskType, bindings: &[(String, String)], domain: Option<&DomainGraph>) -> Dispatched {
    let bound: BTreeMap<&str, &str> = bindings.iter().map(|(k, val)| (k.as_str(), val.as_str())).collect();
    let mut violations = validate_bindings(task, &bound, domain);

    let primary = primary_entity(task, &bound);
    let work_units = if violations.iter().any(|x| x.severity == "violation") {
        Vec::new() // don't instantiate against invalid bindings
    } else {
        task.cells.iter().map(|c| instantiate(task, c, &bound, primary.as_deref())).collect()
    };
    if work_units.is_empty() && !task.cells.is_empty() && !violations.iter().any(|x| x.severity == "violation") {
        violations.push(warn(&task.id, "cells", "task type declares no cells to dispatch"));
    }
    Dispatched { work_units, violations }
}

/// Slot names a cell references as `domain:<slot>` — their bound value flows
/// into a domain pointer, so it must be a real entity.
fn entity_slots(task: &TaskType) -> Vec<&str> {
    let mut out = Vec::new();
    for cell in &task.cells {
        for ptr in &cell.derived_from {
            if let Some(rest) = ptr.strip_prefix("domain:") {
                if task.slots.iter().any(|s| s.name == rest) && !out.contains(&rest) {
                    out.push(rest);
                }
            }
        }
    }
    out
}

/// Every binding names a declared slot; every required slot is bound; an
/// entity-referenced slot's value exists in the What graph.
fn validate_bindings(task: &TaskType, bound: &BTreeMap<&str, &str>, domain: Option<&DomainGraph>) -> Vec<Violation> {
    let mut out = Vec::new();
    for slot_name in bound.keys() {
        if !task.slots.iter().any(|s| &s.name == slot_name) {
            out.push(v(&task.id, "bind", &format!("binding '{slot_name}' names no declared slot")));
        }
    }
    for slot in &task.slots {
        if bound.get(slot.name.as_str()).is_none() && slot.required {
            out.push(v(&task.id, "bind", &format!("required slot '{}' is not bound", slot.name)));
        }
    }
    let entity_slots = entity_slots(task);
    for slot_name in &entity_slots {
        if let Some(val) = bound.get(slot_name) {
            match domain {
                Some(g) if !g.contains(val) => out.push(v(&task.id, "bind",
                    &format!("slot '{slot_name}' bound to '{val}', which is not an entity in the What graph"))),
                None => out.push(warn(&task.id, "bind",
                    &format!("slot '{slot_name}' bound to '{val}' — no What graph loaded to verify it"))),
                _ => {}
            }
        }
    }
    out
}

/// The bound value of the first entity-referenced slot, used to key work-unit ids.
fn primary_entity(task: &TaskType, bound: &BTreeMap<&str, &str>) -> Option<String> {
    entity_slots(task).into_iter()
        .find_map(|s| bound.get(s).map(|v| v.to_string()))
}

/// Turn one template cell into a concrete, frozen work unit.
fn instantiate(task: &TaskType, cell: &super::cell::Cell, bound: &BTreeMap<&str, &str>, primary: Option<&str>) -> WorkUnit {
    let kind_of: BTreeMap<&str, &str> = task.slots.iter()
        .map(|s| (s.name.as_str(), s.kind.as_deref().unwrap_or("domain")))
        .collect();
    let derived_from: Vec<String> = cell.derived_from.iter()
        .map(|p| resolve_pointer(p, bound, &kind_of))
        .collect();
    let id = match primary {
        Some(p) => slug(&format!("{}-{}", cell.id, p)),
        None => slug(&format!("{}-{}", task.id, cell.id)),
    };
    let prompt = format!(
        "Produce {} for task type '{}'{}.{}",
        cell.artifact,
        task.id,
        primary.map(|p| format!(" ({p})")).unwrap_or_default(),
        if cell.applies.is_empty() { String::new() } else { format!(" Apply: {}.", cell.applies.join(", ")) },
    );
    let hash = content_hash(&format!("{}|{}|{}", prompt, derived_from.join(","), cell.artifact));
    WorkUnit {
        id,
        schema: format!("shape: {}", cell.artifact),
        prompt,
        model: cell.model.clone(),
        context: Context { derived_from, frozen: true, hash: Some(format!("sha256:{hash}")) },
        produces: Produces { artifact: cell.artifact.clone(), path_hint: None },
        applies: cell.applies.clone(),
        trace: Some(Trace {
            what: primary.map(str::to_string),
            behaviour: None,
            why: cell.applies.clone(),
            structure: None,
        }),
    }
}

/// Resolve a cell `derived_from` pointer: a slot reference becomes its bound
/// value (prefixed by the slot's kind); everything else passes through.
fn resolve_pointer(ptr: &str, bound: &BTreeMap<&str, &str>, kind_of: &BTreeMap<&str, &str>) -> String {
    let name = ptr.strip_prefix("domain:")
        .or_else(|| ptr.strip_prefix("slot:"))
        .or_else(|| ptr.strip_prefix("behaviour:"))
        .unwrap_or(ptr);
    match bound.get(name) {
        Some(val) => {
            let kind = kind_of.get(name).copied().unwrap_or("domain");
            format!("{kind}:{val}")
        }
        None => ptr.to_string(),
    }
}

/// Lowercase + restrict to the work-unit id grammar `^[a-z0-9][a-z0-9-]*$`.
fn slug(s: &str) -> String {
    let mut out: String = s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    while out.contains("--") {
        out = out.replace("--", "-");
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.chars().next().map(|c| c.is_ascii_alphabetic()).unwrap_or(false) {
        trimmed
    } else {
        format!("u-{trimmed}")
    }
}

#[cfg(test)]
#[path = "dispatch_tests.rs"]
mod tests;
