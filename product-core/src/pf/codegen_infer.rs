//! Payload/state field inference over a Decider's logic + scenarios.
//!
//! The What graph (v1.7.0) carries no payload schemas on commands or
//! events — payload fields appear only in authored Decider logic (guards,
//! `set`/`with` assignments, CEL expressions) and in concrete scenarios.
//! This module walks both to recover, per aggregate: the state fields, each
//! command's payload fields, and each event's payload fields. Types come
//! from observed concrete `Scalar`s (scenarios, literal assignments);
//! fields only ever seen inside a CEL expression default to `string`.

use std::collections::BTreeMap;

use super::decider::Decider;
use super::decider_logic::{CommandRef, EventRef, Scalar, State};
use super::codegen_ident::CsTy;

/// Field name → inferred type (`None` until a concrete value is observed).
pub type Fields = BTreeMap<String, Option<CsTy>>;

/// The inferred C# shape of one aggregate: state fields (+ defaults from
/// `logic.initial`), and per-command / per-event payload fields.
#[derive(Debug, Default)]
pub struct AggShape {
    pub state: Fields,
    pub state_defaults: State,
    pub commands: BTreeMap<String, Fields>,
    pub events: BTreeMap<String, Fields>,
}

impl AggShape {
    fn command(&mut self, id: &str) -> &mut Fields {
        self.commands.entry(id.to_string()).or_default()
    }
    fn event(&mut self, id: &str) -> &mut Fields {
        self.events.entry(id.to_string()).or_default()
    }
}

/// Infer the aggregate shape for a Decider from its signature, logic, and
/// scenarios, then overlay the What graph's declared payload schemas
/// (§3.2 `fields` — declarations win over inference). Every handled
/// command / emitted / evolved-from event gets an entry even when no
/// payload field is ever observed.
pub fn infer_shape(decider: &Decider, graph: &super::model::DomainGraph) -> AggShape {
    let mut shape = infer(decider);
    for (id, fields) in shape.commands.iter_mut() {
        if let Some(c) = graph.commands.iter().find(|c| &c.id == id) {
            overlay_declared(fields, &c.fields);
        }
    }
    for (id, fields) in shape.events.iter_mut() {
        if let Some(e) = graph.events.iter().find(|e| &e.id == id) {
            overlay_declared(fields, &e.fields);
        }
    }
    shape
}

/// A declared field always exists; a declared *type* overrides inference.
fn overlay_declared(fields: &mut Fields, declared: &[super::model::Attribute]) {
    for a in declared {
        let slot = fields.entry(a.name.clone()).or_insert(None);
        if let Some(t) = super::codegen_ident::attr_cs_ty(a.ty.as_deref()) {
            *slot = Some(t);
        }
    }
}

/// Inference from the Decider alone (no graph overlay) — see [`infer_shape`].
pub fn infer(decider: &Decider) -> AggShape {
    let mut shape = AggShape::default();
    for c in &decider.handles {
        shape.command(c);
    }
    for e in decider.emits.iter().chain(&decider.evolves_from) {
        shape.event(e);
    }
    for r in &decider.reads {
        shape.state.entry(r.clone()).or_insert(None);
    }
    let links = infer_logic(decider, &mut shape);
    for s in &decider.scenarios {
        for ev in &s.given {
            note_event_ref(&mut shape, ev);
        }
        note_command_ref(&mut shape, &s.when);
        for ev in s.then.emit.iter().flatten() {
            note_event_ref(&mut shape, ev);
        }
    }
    resolve_links(&mut shape, &links);
    shape
}

/// A pending `state field ← event.field` type link from an evolve rule whose
/// value is a plain `=event.<f>` copy — resolved once event types are known.
struct Link {
    state_field: String,
    event_id: String,
    event_field: String,
}

fn infer_logic(decider: &Decider, shape: &mut AggShape) -> Vec<Link> {
    let mut links = Vec::new();
    let Some(logic) = &decider.logic else { return links };
    for (field, value) in &logic.initial {
        note(&mut shape.state, field, Some(scalar_ty(value)));
        shape.state_defaults.insert(field.clone(), value.clone());
    }
    for rule in &logic.evolve {
        shape.event(&rule.on);
        for (field, value) in &rule.set {
            note(&mut shape.state, field, literal_ty(value));
            scan_value(shape, &rule.on, value, Binding::Event);
            if let Some(event_field) = plain_copy(value, "event.") {
                links.push(Link {
                    state_field: field.clone(),
                    event_id: rule.on.clone(),
                    event_field,
                });
            }
        }
    }
    for rule in &logic.decide {
        shape.command(&rule.on);
        infer_decide_rule(shape, rule);
    }
    links
}

/// `=event.amount` (a bare field copy, nothing else) → `Some("amount")`.
fn plain_copy(v: &Scalar, prefix: &str) -> Option<String> {
    let Scalar::Str(s) = v else { return None };
    let expr = s.strip_prefix('=')?.trim();
    let field = expr.strip_prefix(prefix)?;
    (!field.is_empty() && field.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .then(|| field.to_string())
}

/// Give an untyped state field the type of the event field it plainly copies.
fn resolve_links(shape: &mut AggShape, links: &[Link]) {
    for link in links {
        let ty = shape
            .events
            .get(&link.event_id)
            .and_then(|f| f.get(&link.event_field).copied().flatten());
        if let (Some(t), Some(slot)) = (ty, shape.state.get_mut(&link.state_field)) {
            if slot.is_none() {
                *slot = Some(t);
            }
        }
    }
}

fn infer_decide_rule(shape: &mut AggShape, rule: &super::decider_logic::DecideRule) {
    for g in &rule.guards {
        if let Some(p) = &g.when {
            let ty = p.eq.as_ref().or(p.ne.as_ref()).map(scalar_ty);
            note(&mut shape.state, &p.field, ty);
        }
        if let Some(expr) = &g.expr {
            scan_expr(shape, &rule.on, expr, Binding::Command);
        }
    }
    for ev in &rule.emit {
        shape.event(ev.id());
        for (field, value) in &ev.payload() {
            note(shape.event(ev.id()), field, literal_ty(value));
            scan_value(shape, &rule.on, value, Binding::Command);
        }
    }
}

/// Which non-state binding a CEL expression can see (§3.3 interpreter):
/// `command` inside decide rules, `event` inside evolve rules.
#[derive(Clone, Copy)]
enum Binding {
    Command,
    Event,
}

fn note(fields: &mut Fields, name: &str, ty: Option<CsTy>) {
    let slot = fields.entry(name.to_string()).or_insert(None);
    if let Some(t) = ty {
        *slot = Some(CsTy::merge(*slot, t));
    }
}

fn note_event_ref(shape: &mut AggShape, ev: &EventRef) {
    let fields = shape.event(ev.id());
    for (field, value) in &ev.payload() {
        note(fields, field, Some(scalar_ty(value)));
    }
}

fn note_command_ref(shape: &mut AggShape, cmd: &CommandRef) {
    let fields = shape.command(cmd.id());
    for (field, value) in &cmd.payload() {
        note(fields, field, Some(scalar_ty(value)));
    }
}

fn scalar_ty(s: &Scalar) -> CsTy {
    match s {
        Scalar::Bool(_) => CsTy::Bool,
        Scalar::Int(_) => CsTy::Long,
        Scalar::Str(_) => CsTy::Str,
    }
}

/// The type of a literal assignment value — `None` when it is a `=` CEL
/// expression (its result type is not statically known here).
fn literal_ty(v: &Scalar) -> Option<CsTy> {
    match v {
        Scalar::Str(s) if s.starts_with('=') => None,
        other => Some(scalar_ty(other)),
    }
}

/// If the value is a `=` CEL expression, scan it for field references.
fn scan_value(shape: &mut AggShape, on: &str, v: &Scalar, binding: Binding) {
    if let Scalar::Str(s) = v {
        if let Some(expr) = s.strip_prefix('=') {
            scan_expr(shape, on, expr, binding);
        }
    }
}

/// Collect `state.<f>` / `command.<f>` / `event.<f>` references from a CEL
/// expression by lexical scan (no CEL parse — good enough for field names).
fn scan_expr(shape: &mut AggShape, on: &str, expr: &str, binding: Binding) {
    for field in scan_prefix(expr, "state.") {
        note(&mut shape.state, &field, None);
    }
    let (prefix, id) = match binding {
        Binding::Command => ("command.", on),
        Binding::Event => ("event.", on),
    };
    for field in scan_prefix(expr, prefix) {
        let fields = match binding {
            Binding::Command => shape.command(id),
            Binding::Event => shape.event(id),
        };
        note(fields, &field, None);
    }
}

fn scan_prefix(expr: &str, prefix: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = expr;
    while let Some(pos) = rest.find(prefix) {
        let boundary_ok = pos == 0
            || !rest[..pos]
                .ends_with(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '.');
        let after = &rest[pos + prefix.len()..];
        let field: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if boundary_ok && !field.is_empty() {
            out.push(field);
        }
        rest = &rest[pos + prefix.len()..];
    }
    out
}
