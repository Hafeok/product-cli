//! C# Decider frame emission — the partial class the realiser implements.
//!
//! Three files per aggregate: `<Agg>Decider.g.cs` (the generated half of a
//! `static partial class` — signature arrays, `InitialState`, `Fold`, and
//! the `Decide`/`Evolve` partial declarations), `<Agg>Decider.cs` (the
//! editable stub, scaffolded once and never overwritten — behaviour is
//! authored, not transpiled), plus `<Agg>Adapter.g.cs` (the typed
//! `IConformanceAdapter` bridging the §6.3 wire protocol to the Decider).

use super::decider::Decider;
use super::reify_ident::{cs_escape, pascal, CsTy};
use super::reify_infer::{AggShape, Fields};

/// Render `<Agg>Decider.g.cs` — the generated half of the partial class.
pub fn frame_file(header: &str, ns: &str, agg: &str, decider: &Decider) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System.Collections.Generic;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&format!(
        "/// <summary>§3.3 Decider frame for aggregate '{}' — the signature is derived from the What graph; implement Decide/Evolve in {agg}Decider.cs.</summary>\n",
        cs_escape(&decider.decides_for)
    ));
    s.push_str(&format!("public static partial class {agg}Decider\n{{\n"));
    s.push_str(&id_array("Handles", &decider.handles));
    s.push_str(&id_array("Emits", &decider.emits));
    s.push_str(&id_array("EvolvesFrom", &decider.evolves_from));
    s.push_str(&id_array("Rejects", &decider.rejects));
    s.push_str(&frame_members(agg));
    s.push_str("}\n");
    s
}

fn id_array(name: &str, ids: &[String]) -> String {
    let items: Vec<String> = ids.iter().map(|i| format!("\"{}\"", cs_escape(i))).collect();
    format!(
        "    /// <summary>§3.3 `{}` — pinned from the What graph.</summary>\n    public static readonly string[] {name} = {{ {} }};\n",
        name.to_ascii_lowercase(),
        items.join(", ")
    )
}

fn frame_members(agg: &str) -> String {
    let mut s = String::new();
    s.push('\n');
    s.push_str(&format!(
        "    /// <summary>Decide a command against current state: reject via an invariant id, or emit sanctioned events (§3.3).</summary>\n    public static partial DecisionResult Decide({agg}State state, I{agg}Command command);\n\n"
    ));
    s.push_str(&format!(
        "    /// <summary>Fold one event into state (§3.3 evolve).</summary>\n    public static partial {agg}State Evolve({agg}State state, I{agg}Event evt);\n\n"
    ));
    s.push_str(&format!(
        "    /// <summary>Initial state — defaults come from the record's initializers (logic.initial).</summary>\n    public static {agg}State InitialState() => new {agg}State();\n\n"
    ));
    s.push_str(&format!(
        "    /// <summary>Replay a history into state.</summary>\n    public static {agg}State Fold(IEnumerable<I{agg}Event> events)\n    {{\n        var state = InitialState();\n        foreach (var e in events) state = Evolve(state, e);\n        return state;\n    }}\n"
    ));
    s
}

/// Render the editable `<Agg>Decider.cs` stub (written only if missing).
pub fn stub_file(ns: &str, agg: &str, decider: &Decider) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "// Editable realisation of the '{}' Decider (§3.3). Scaffolded once by\n// `product reify csharp` and never overwritten — author Decide + Evolve here.\n// Verify with the generated scenario tests, then close the loop with:\n//   product decider conform {} --runner \"dotnet run --project ../{ns}.Conformance -- {}\"\n",
        cs_escape(&decider.decides_for),
        decider.id,
        decider.id
    ));
    s.push_str("#nullable enable\n\nusing System;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&format!("public static partial class {agg}Decider\n{{\n"));
    s.push_str(&format!(
        "    public static partial DecisionResult Decide({agg}State state, I{agg}Command command)\n    {{\n        // TODO: guard each command with the invariants it protects, then emit its events.\n        throw new NotImplementedException(\"realise Decide for '{}'\");\n    }}\n\n",
        cs_escape(&decider.decides_for)
    ));
    s.push_str(&format!(
        "    public static partial {agg}State Evolve({agg}State state, I{agg}Event evt)\n    {{\n        // TODO: fold each event into state.\n        throw new NotImplementedException(\"realise Evolve for '{}'\");\n    }}\n}}\n",
        cs_escape(&decider.decides_for)
    ));
    s
}

/// Render `<Agg>Adapter.g.cs` — the typed [`IConformanceAdapter`] for full
/// mode: wire → typed records → the realised Decider → wire outcome.
pub fn adapter_file(header: &str, ns: &str, agg: &str, shape: &AggShape) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System;\nusing System.Collections.Generic;\nusing System.Linq;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&format!(
        "/// <summary>§6.3 conformance adapter for the {agg} aggregate — bridges the wire protocol to the typed Decider.</summary>\n"
    ));
    s.push_str(&format!("public sealed class {agg}Adapter : IConformanceAdapter\n{{\n"));
    s.push_str(&run_method(agg));
    s.push('\n');
    s.push_str(&from_wire(agg, "Event", &shape.events));
    s.push('\n');
    s.push_str(&from_wire(agg, "Command", &shape.commands));
    s.push_str("}\n");
    s
}

fn run_method(agg: &str) -> String {
    let mut s = String::new();
    s.push_str("    public ConformanceOutcome Run(string deciderId, IReadOnlyList<WireEvent> given, WireCommand when)\n    {\n");
    s.push_str(&format!("        var state = {agg}Decider.Fold(given.Select(ToEvent));\n"));
    s.push_str(&format!("        var result = {agg}Decider.Decide(state, ToCommand(when));\n"));
    s.push_str("        if (result.IsRejected) return ConformanceOutcome.Rejected(result.RejectedInvariant!);\n");
    s.push_str("        return ConformanceOutcome.Emitted(result.Events.Select(e => new WireEvent(e.WireId, e.WirePayload())).ToArray());\n");
    s.push_str("    }\n");
    s
}

fn from_wire(agg: &str, kind: &str, members: &std::collections::BTreeMap<String, Fields>) -> String {
    let wire_ty = if kind == "Event" { "WireEvent" } else { "WireCommand" };
    let mut s = String::new();
    s.push_str(&format!(
        "    private static I{agg}{kind} To{kind}({wire_ty} wire) => wire.Id switch\n    {{\n"
    ));
    for (id, fields) in members {
        s.push_str(&format!(
            "        \"{}\" => new {}({}),\n",
            cs_escape(id),
            pascal(id),
            ctor_args(fields)
        ));
    }
    s.push_str(&format!(
        "        _ => throw new InvalidOperationException($\"unknown {agg} {} '{{wire.Id}}'\"),\n    }};\n",
        kind.to_ascii_lowercase()
    ));
    s
}

fn ctor_args(fields: &Fields) -> String {
    fields
        .iter()
        .map(|(name, ty)| {
            let getter = match ty.unwrap_or(CsTy::Str) {
                CsTy::Bool => "GetBool",
                CsTy::Long => "GetLong",
                CsTy::Str => "GetString",
            };
            format!("{}: PfWire.{getter}(wire.With, \"{}\")", pascal(name), cs_escape(name))
        })
        .collect::<Vec<_>>()
        .join(", ")
}
