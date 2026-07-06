//! xUnit emission — one generated fact per Decider scenario (§6.3).
//!
//! The scenarios are the oracle, authored once and consumed twice: the
//! Rust interpreter simulates them pre-realisation (`decider_sim`), and
//! the generated `<Agg>ScenarioTests.g.cs` replays the very same
//! given/when/then against the realised C# `Decide`/`Evolve` — the
//! post-realisation consumption the spec pins as behavioural conformance.

use std::collections::BTreeSet;

use super::decider::Decider;
use super::decider_logic::{EventRef, Expectation, Payload, Scenario};
use super::codegen_ident::{cs_escape, method_name, pascal, CsTy};
use super::codegen_infer::{AggShape, Fields};
use super::codegen_types::scalar_literal;

/// Render `<Agg>ScenarioTests.g.cs` for a Decider with scenarios.
pub fn tests_file(header: &str, ns: &str, agg: &str, decider: &Decider, shape: &AggShape) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing Xunit;\n");
    s.push_str(&format!("using {ns};\n\nnamespace {ns}.Tests;\n\n"));
    s.push_str(&format!(
        "/// <summary>§6.3 — the '{}' scenarios replayed against the realised Decider.</summary>\n",
        cs_escape(&decider.id)
    ));
    s.push_str(&format!("public class {agg}ScenarioTests\n{{\n"));
    let mut seen = BTreeSet::new();
    for scenario in &decider.scenarios {
        s.push_str(&fact(agg, scenario, shape, &mut seen));
    }
    s.push_str("}\n");
    s
}

fn fact(agg: &str, scenario: &Scenario, shape: &AggShape, seen: &mut BTreeSet<String>) -> String {
    let mut name = method_name(&scenario.name);
    while !seen.insert(name.clone()) {
        name.push('_');
    }
    let mut s = String::new();
    s.push_str(&format!("    [Fact]\n    public void {name}()\n    {{\n"));
    s.push_str(&given_block(agg, &scenario.given, shape));
    s.push_str(&format!(
        "        var result = {agg}Decider.Decide(state, {});\n",
        new_expr(scenario.when.id(), &scenario.when.payload(), shape.commands.get(scenario.when.id()))
    ));
    s.push_str(&expectation_block(&scenario.then, shape));
    s.push_str("    }\n\n");
    s
}

fn given_block(agg: &str, given: &[EventRef], shape: &AggShape) -> String {
    if given.is_empty() {
        return format!("        var state = {agg}Decider.InitialState();\n");
    }
    let mut s = format!("        var state = {agg}Decider.Fold(new I{agg}Event[]\n        {{\n");
    for ev in given {
        s.push_str(&format!(
            "            {},\n",
            new_expr(ev.id(), &ev.payload(), shape.events.get(ev.id()))
        ));
    }
    s.push_str("        });\n");
    s
}

/// A `new <Type>(Field: literal, …)` construction expression, with each
/// literal rendered at the field's inferred C# type.
fn new_expr(id: &str, payload: &Payload, fields: Option<&Fields>) -> String {
    let args: Vec<String> = payload
        .iter()
        .map(|(name, value)| {
            let ty = fields
                .and_then(|f| f.get(name).copied().flatten())
                .unwrap_or(CsTy::Str);
            format!("{}: {}", pascal(name), arg_literal(ty, value))
        })
        .collect();
    format!("new {}({})", pascal(id), args.join(", "))
}

fn arg_literal(ty: CsTy, value: &super::decider_logic::Scalar) -> String {
    use super::decider_logic::Scalar;
    match (ty, value) {
        (CsTy::Long, Scalar::Int(i)) => format!("{i}L"),
        _ => scalar_literal(ty, value),
    }
}

fn expectation_block(then: &Expectation, shape: &AggShape) -> String {
    if let Some(invariant) = &then.reject {
        return format!(
            "        Assert.Equal(\"{}\", result.RejectedInvariant);\n",
            cs_escape(invariant)
        );
    }
    let expected = then.emit.clone().unwrap_or_default();
    let mut s = String::from("        Assert.Null(result.RejectedInvariant);\n");
    s.push_str(&count_assert("result.Events", expected.len()));
    for (i, ev) in expected.iter().enumerate() {
        s.push_str(&emitted_event_asserts(i, ev, shape));
    }
    s
}

fn emitted_event_asserts(i: usize, ev: &EventRef, shape: &AggShape) -> String {
    let mut s = format!(
        "        Assert.Equal(\"{}\", result.Events[{i}].WireId);\n",
        cs_escape(ev.id())
    );
    let payload = ev.payload();
    s.push_str(&format!("        var with{i} = result.Events[{i}].WirePayload();\n"));
    s.push_str(&count_assert(&format!("with{i}"), payload.len()));
    for (name, value) in &payload {
        let ty = shape
            .events
            .get(ev.id())
            .and_then(|f| f.get(name).copied().flatten())
            .unwrap_or(CsTy::Str);
        s.push_str(&payload_assert(i, name, ty, value));
    }
    s
}

/// A collection-size assertion in the form the xUnit analyzers prefer
/// (`Assert.Empty` / `Assert.Single` for 0/1, `Assert.Equal` above).
pub(crate) fn count_assert(expr: &str, len: usize) -> String {
    match len {
        0 => format!("        Assert.Empty({expr});\n"),
        1 => format!("        Assert.Single({expr});\n"),
        n => format!("        Assert.Equal({n}, {expr}.Count);\n"),
    }
}

fn payload_assert(i: usize, name: &str, ty: CsTy, value: &super::decider_logic::Scalar) -> String {
    let key = cs_escape(name);
    format!(
        "        Assert.Equal({}, Assert.IsType<{}>(with{i}[\"{key}\"]));\n",
        arg_literal(ty, value),
        ty.name()
    )
}
