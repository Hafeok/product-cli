//! C# type emission — domain records, enums, per-aggregate contracts.
//!
//! `domain_file` renders `Domain.g.cs`: one record per entity/value object
//! (attributes via the §3.1 datatype vocabulary) plus one enum per §3.1
//! reference set. `agg_types_file` renders `<Agg>Types.g.cs`: the sealed
//! command/event interfaces, one record per command/event with nullable
//! payload properties (payloads are subsets on the wire — absent fields
//! stay null and are omitted from `WirePayload`), and the aggregate state
//! record with defaults from `logic.initial`.

use super::decider::Decider;
use super::decider_logic::Scalar;
use super::model::{DomainGraph, Entity, ReferenceSet, ValueObject};
use super::reify_ident::{attr_ty, cs_escape, pascal, CsTy};
use super::reify_infer::{AggShape, Fields};

/// Render `Domain.g.cs` — entities, value objects, reference-set enums.
pub fn domain_file(header: &str, ns: &str, graph: &DomainGraph) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\n");
    s.push_str(&format!("namespace {ns};\n"));
    for e in &graph.entities {
        s.push_str(&entity_record(e));
    }
    for v in &graph.value_objects {
        s.push_str(&value_object_record(v));
    }
    for r in &graph.reference_sets {
        s.push_str(&reference_enum(r));
    }
    s
}

fn entity_record(e: &Entity) -> String {
    let mut s = String::new();
    s.push('\n');
    s.push_str(&doc_comment(&e.definition));
    let name = pascal(&e.id);
    if e.attributes.is_empty() {
        s.push_str(&format!("public sealed record {name};\n"));
        return s;
    }
    let params: Vec<String> = e
        .attributes
        .iter()
        .map(|a| format!("{}? {} = null", attr_ty(a.ty.as_deref()), pascal(&a.name)))
        .collect();
    s.push_str(&format!("public sealed record {name}({});\n", params.join(", ")));
    s
}

fn value_object_record(v: &ValueObject) -> String {
    let mut s = String::new();
    s.push('\n');
    s.push_str(&doc_comment(v.definition.as_deref().unwrap_or(&v.label)));
    s.push_str(&format!("public sealed record {};\n", pascal(&v.id)));
    s
}

fn reference_enum(r: &ReferenceSet) -> String {
    let mut s = String::new();
    s.push('\n');
    let label = r.label.as_deref().unwrap_or(&r.id);
    s.push_str(&doc_comment(&format!(
        "{label} — §3.1 reference set for '{}' (closed value set).",
        r.concept
    )));
    s.push_str(&format!("public enum {}\n{{\n", pascal(&r.id)));
    let mut seen = std::collections::BTreeSet::new();
    for value in &r.values {
        let mut member = pascal(value);
        while !seen.insert(member.clone()) {
            member.push('_');
        }
        s.push_str(&format!("    /// <summary>wire value: \"{}\"</summary>\n", cs_escape(value)));
        s.push_str(&format!("    {member},\n"));
    }
    s.push_str("}\n");
    s
}

/// Render `<Agg>Types.g.cs` — interfaces, command/event records, state.
pub fn agg_types_file(
    header: &str,
    ns: &str,
    agg: &str,
    graph: &DomainGraph,
    decider: &Decider,
    shape: &AggShape,
) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System.Collections.Generic;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&doc_comment(&format!(
        "A command the {agg} Decider handles (§3.3 — derived from the event model)."
    )));
    s.push_str(&format!("public interface I{agg}Command : IDomainCommand {{ }}\n\n"));
    s.push_str(&doc_comment(&format!(
        "An event the {agg} Decider emits or evolves from (§3.3)."
    )));
    s.push_str(&format!("public interface I{agg}Event : IDomainEvent {{ }}\n"));
    for (id, fields) in &shape.commands {
        s.push_str(&command_record(agg, id, fields, graph));
    }
    for (id, fields) in &shape.events {
        s.push_str(&event_record(agg, id, fields, graph));
    }
    s.push_str(&state_record(agg, decider, shape));
    s
}

/// The `Ty? Name = null` payload parameter list shared by commands + events.
fn payload_params(fields: &Fields) -> String {
    fields
        .iter()
        .map(|(name, ty)| {
            let cs = ty.unwrap_or(CsTy::Str).name();
            format!("{cs}? {} = null", pascal(name))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn command_record(agg: &str, id: &str, fields: &Fields, graph: &DomainGraph) -> String {
    let mut s = String::new();
    s.push('\n');
    if let Some(c) = graph.commands.iter().find(|c| c.id == id) {
        s.push_str(&doc_comment(&format!("{} — targets '{}'.", c.label, c.targets)));
    }
    s.push_str(&format!(
        "public sealed record {}({}) : I{agg}Command\n{{\n",
        pascal(id),
        payload_params(fields)
    ));
    s.push_str(&format!("    public string WireId => \"{}\";\n}}\n", cs_escape(id)));
    s
}

fn event_record(agg: &str, id: &str, fields: &Fields, graph: &DomainGraph) -> String {
    let mut s = String::new();
    s.push('\n');
    if let Some(e) = graph.events.iter().find(|e| e.id == id) {
        s.push_str(&doc_comment(&format!("{} — changes '{}'.", e.label, e.changes)));
    }
    s.push_str(&format!(
        "public sealed record {}({}) : I{agg}Event\n{{\n",
        pascal(id),
        payload_params(fields)
    ));
    s.push_str(&format!("    public string WireId => \"{}\";\n", cs_escape(id)));
    s.push_str("    public IReadOnlyDictionary<string, object?> WirePayload()\n    {\n");
    s.push_str("        var d = new Dictionary<string, object?>();\n");
    for name in fields.keys() {
        s.push_str(&format!(
            "        if ({0} is not null) d[\"{1}\"] = {0};\n",
            pascal(name),
            cs_escape(name)
        ));
    }
    s.push_str("        return d;\n    }\n}\n");
    s
}

fn state_record(agg: &str, decider: &Decider, shape: &AggShape) -> String {
    let mut s = String::new();
    s.push('\n');
    s.push_str(&doc_comment(&format!(
        "Aggregate state for '{}' — fields inferred from logic.initial, evolve rules, guards, `reads`.",
        decider.decides_for
    )));
    s.push_str(&format!("public sealed record {agg}State\n{{\n"));
    for (name, ty) in &shape.state {
        s.push_str(&state_field(name, *ty, shape.state_defaults.get(name)));
    }
    s.push_str("}\n");
    s
}

fn state_field(name: &str, ty: Option<CsTy>, default: Option<&Scalar>) -> String {
    let prop = pascal(name);
    match (ty, default) {
        (Some(t), Some(d)) => format!(
            "    public {} {prop} {{ get; init; }} = {};\n",
            t.name(),
            scalar_literal(t, d)
        ),
        (Some(CsTy::Str), None) => {
            format!("    public string? {prop} {{ get; init; }}\n")
        }
        (Some(t), None) => format!("    public {} {prop} {{ get; init; }}\n", t.name()),
        (None, _) => format!("    public string? {prop} {{ get; init; }}\n"),
    }
}

/// Render an initial-state default as a literal of the merged C# type —
/// when observations widened the field to `string`, quote whatever the
/// default was so the generated code still compiles.
pub(crate) fn scalar_literal(ty: CsTy, s: &Scalar) -> String {
    match (ty, s) {
        (CsTy::Bool, Scalar::Bool(b)) => b.to_string(),
        (CsTy::Long, Scalar::Int(i)) => i.to_string(),
        (CsTy::Str, Scalar::Str(v)) => format!("\"{}\"", cs_escape(v)),
        (CsTy::Str, Scalar::Bool(b)) => format!("\"{b}\""),
        (CsTy::Str, Scalar::Int(i)) => format!("\"{i}\""),
        (CsTy::Bool, _) => "false".to_string(),
        (CsTy::Long, _) => "0".to_string(),
    }
}

fn doc_comment(text: &str) -> String {
    format!("/// <summary>{}</summary>\n", xml_escape(text))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
