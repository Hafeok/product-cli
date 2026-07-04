//! Oracle-seam emission — the wire-level conformance surface (§6.3).
//!
//! `Oracle.g.cs` is the protocol-shaped seam between the deterministic
//! oracle & any realisation: `WireEvent`/`WireCommand` (payloads as
//! wire-name → scalar dictionaries), `ConformanceOutcome`, the
//! `IConformanceAdapter` a realiser implements, plus the `PfWire` JSON
//! helpers. In oracle-only mode nothing else is typed: the realiser owns
//! the whole domain design behind a scaffolded-once `ConformanceAdapter`,
//! while the generated scenario tests + runner hold it to the graph.

use super::decider::Decider;
use super::decider_logic::{EventRef, Expectation, Payload, Scalar, Scenario};
use super::reify_ident::{cs_escape, method_name};
use std::collections::BTreeSet;

const ORACLE_CS: &str = r##"#nullable enable

using System.Collections.Generic;
using System.Linq;
using System.Text.Json;

namespace {NS};

/// <summary>An event in wire form (§6.3 protocol): its What-graph id plus the payload fields actually set.</summary>
public sealed record WireEvent(string Id, IReadOnlyDictionary<string, object?> With);

/// <summary>A command in wire form (§6.3 protocol).</summary>
public sealed record WireCommand(string Id, IReadOnlyDictionary<string, object?> With);

/// <summary>The wire-level outcome of one decision: emitted events, or a rejection naming the violated invariant.</summary>
public sealed class ConformanceOutcome
{
    private ConformanceOutcome(IReadOnlyList<WireEvent>? emit, string? reject)
    {
        Emit = emit;
        Reject = reject;
    }

    public IReadOnlyList<WireEvent>? Emit { get; }
    public string? Reject { get; }

    public static ConformanceOutcome Emitted(params WireEvent[] events) => new(events, null);
    public static ConformanceOutcome Rejected(string invariantId) => new(null, invariantId);
}

/// <summary>The seam between the deterministic oracle and a realisation (§6.3):
/// fold <paramref name="given"/> into fresh aggregate state, decide <paramref name="when"/>,
/// answer in wire form. The realiser owns everything behind this interface.</summary>
public interface IConformanceAdapter
{
    ConformanceOutcome Run(string deciderId, IReadOnlyList<WireEvent> given, WireCommand when);
}

/// <summary>JSON codec for the §6.3 conformance wire protocol (scalars: long · bool · string).</summary>
public static class PfWire
{
    public static WireEvent ParseEvent(JsonElement el)
    {
        var (id, with) = Split(el, "event");
        return new WireEvent(id, with);
    }

    public static WireCommand ParseCommand(JsonElement el)
    {
        var (id, with) = Split(el, "command");
        return new WireCommand(id, with);
    }

    private static (string, IReadOnlyDictionary<string, object?>) Split(JsonElement el, string idKey)
    {
        if (el.ValueKind == JsonValueKind.String)
            return (el.GetString() ?? "", new Dictionary<string, object?>());
        var id = el.GetProperty(idKey).GetString() ?? "";
        if (!el.TryGetProperty("with", out var with))
            return (id, new Dictionary<string, object?>());
        var d = new Dictionary<string, object?>();
        foreach (var p in with.EnumerateObject())
            d[p.Name] = p.Value.ValueKind switch
            {
                JsonValueKind.Number => p.Value.GetInt64(),
                JsonValueKind.True => true,
                JsonValueKind.False => false,
                JsonValueKind.String => p.Value.GetString(),
                _ => null,
            };
        return (id, d);
    }

    public static long? GetLong(IReadOnlyDictionary<string, object?> with, string name) =>
        with.TryGetValue(name, out var v) && v is long l ? l : null;

    public static string? GetString(IReadOnlyDictionary<string, object?> with, string name) =>
        with.TryGetValue(name, out var v) && v is string s ? s : null;

    public static bool? GetBool(IReadOnlyDictionary<string, object?> with, string name) =>
        with.TryGetValue(name, out var v) && v is bool b ? b : null;

    /// <summary>Serialize an outcome as a protocol response: {"reject": id} or {"emit": [{"event", "with"?}]}.</summary>
    public static Dictionary<string, object?> ToResponse(ConformanceOutcome outcome)
    {
        if (outcome.Reject is not null)
            return new Dictionary<string, object?> { ["reject"] = outcome.Reject };
        var emit = (outcome.Emit ?? new List<WireEvent>()).Select(e =>
        {
            var entry = new Dictionary<string, object?> { ["event"] = e.Id };
            if (e.With.Count > 0) entry["with"] = e.With;
            return entry;
        }).ToList();
        return new Dictionary<string, object?> { ["emit"] = emit };
    }
}
"##;

const ADAPTER_STUB_CS: &str = r##"// Scaffolded once by `product reify csharp --oracle-only` — never overwritten.
// Implement the §6.3 oracle seam here: fold `given` into your aggregate state,
// decide `when`, and answer in wire form. Your domain model's design (types,
// layering, patterns from the How contract) is entirely yours — the generated
// scenario tests and `product decider conform` hold it to the What graph.
#nullable enable

using System;
using System.Collections.Generic;

namespace {NS};

public sealed class ConformanceAdapter : IConformanceAdapter
{
    public ConformanceOutcome Run(string deciderId, IReadOnlyList<WireEvent> given, WireCommand when)
    {
        // TODO: delegate to your realised domain model, e.g.
        //   var state = OrderAggregate.Fold(given.Select(ToDomainEvent));
        //   return Map(state.Decide(ToDomainCommand(when)));
        throw new NotImplementedException($"realise the conformance adapter for '{deciderId}'");
    }
}
"##;

const ORACLE_CSPROJ: &str = r##"<Project Sdk="Microsoft.NET.Sdk">

  <!-- Scaffolded once by `product reify csharp` in oracle mode (never
       overwritten): add a ProjectReference to your domain implementation below. -->
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <LangVersion>latest</LangVersion>
    <Nullable>enable</Nullable>
    <ImplicitUsings>disable</ImplicitUsings>
  </PropertyGroup>

</Project>
"##;

/// `Oracle.g.cs` for the given namespace (emitted in both modes).
pub fn oracle_file(header: &str, ns: &str) -> String {
    format!("{header}{}", ORACLE_CS.replace("{NS}", ns))
}

/// The scaffolded-once `ConformanceAdapter.cs` stub (oracle-only mode).
pub fn adapter_stub(ns: &str) -> String {
    ADAPTER_STUB_CS.replace("{NS}", ns)
}

/// The scaffolded-once Conformance csproj (oracle-only mode — the realiser
/// adds their own domain ProjectReference).
pub fn oracle_csproj() -> String {
    ORACLE_CSPROJ.to_string()
}

/// Render `<Agg>ScenarioTests.g.cs` at the wire level (oracle-only mode):
/// each fact drives the scaffolded `ConformanceAdapter` with wire payloads.
pub fn wire_tests_file(header: &str, ns: &str, agg: &str, decider: &Decider) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System;\nusing System.Collections.Generic;\nusing Xunit;\n");
    s.push_str(&format!("using {ns};\n\nnamespace {ns}.Tests;\n\n"));
    s.push_str(&format!(
        "/// <summary>§6.3 — the '{}' scenarios replayed through the conformance adapter (wire level).</summary>\n",
        cs_escape(&decider.id)
    ));
    s.push_str(&format!("public class {agg}ScenarioTests\n{{\n"));
    let mut seen = BTreeSet::new();
    for scenario in &decider.scenarios {
        s.push_str(&wire_fact(&decider.id, scenario, &mut seen));
    }
    s.push_str("}\n");
    s
}

fn wire_fact(decider_id: &str, scenario: &Scenario, seen: &mut BTreeSet<String>) -> String {
    let mut name = method_name(&scenario.name);
    while !seen.insert(name.clone()) {
        name.push('_');
    }
    let mut s = String::new();
    s.push_str(&format!("    [Fact]\n    public void {name}()\n    {{\n"));
    s.push_str("        var adapter = new ConformanceAdapter();\n");
    s.push_str(&format!(
        "        var outcome = adapter.Run(\"{}\", {}, {});\n",
        cs_escape(decider_id),
        given_expr(&scenario.given),
        wire_new("WireCommand", scenario.when.id(), &scenario.when.payload())
    ));
    s.push_str(&wire_expectation(&scenario.then));
    s.push_str("    }\n\n");
    s
}

fn given_expr(given: &[EventRef]) -> String {
    if given.is_empty() {
        return "Array.Empty<WireEvent>()".to_string();
    }
    let items: Vec<String> = given
        .iter()
        .map(|ev| format!("            {},\n", wire_new("WireEvent", ev.id(), &ev.payload())))
        .collect();
    format!("new[]\n        {{\n{}        }}", items.join(""))
}

/// `new WireEvent("id", new Dictionary<string, object?> { ["f"] = v })`.
fn wire_new(ty: &str, id: &str, payload: &Payload) -> String {
    if payload.is_empty() {
        return format!("new {ty}(\"{}\", new Dictionary<string, object?>())", cs_escape(id));
    }
    let fields: Vec<String> = payload
        .iter()
        .map(|(k, v)| format!("[\"{}\"] = {}", cs_escape(k), wire_scalar(v)))
        .collect();
    format!(
        "new {ty}(\"{}\", new Dictionary<string, object?> {{ {} }})",
        cs_escape(id),
        fields.join(", ")
    )
}

fn wire_scalar(v: &Scalar) -> String {
    match v {
        Scalar::Bool(b) => b.to_string(),
        Scalar::Int(i) => format!("{i}L"),
        Scalar::Str(s) => format!("\"{}\"", cs_escape(s)),
    }
}

fn wire_expectation(then: &Expectation) -> String {
    if let Some(invariant) = &then.reject {
        return format!("        Assert.Equal(\"{}\", outcome.Reject);\n", cs_escape(invariant));
    }
    let expected = then.emit.clone().unwrap_or_default();
    let mut s = String::from("        Assert.Null(outcome.Reject);\n");
    s.push_str(&super::reify_scenarios::count_assert("outcome.Emit!", expected.len()));
    for (i, ev) in expected.iter().enumerate() {
        s.push_str(&format!(
            "        Assert.Equal(\"{}\", outcome.Emit![{i}].Id);\n",
            cs_escape(ev.id())
        ));
        let payload = ev.payload();
        s.push_str(&super::reify_scenarios::count_assert(&format!("outcome.Emit![{i}].With"), payload.len()));
        for (k, v) in &payload {
            s.push_str(&wire_payload_assert(i, k, v));
        }
    }
    s
}

fn wire_payload_assert(i: usize, name: &str, value: &Scalar) -> String {
    let ty = match value {
        Scalar::Bool(_) => "bool",
        Scalar::Int(_) => "long",
        Scalar::Str(_) => "string",
    };
    format!(
        "        Assert.Equal({}, Assert.IsType<{ty}>(outcome.Emit![{i}].With[\"{}\"]));\n",
        wire_scalar(value),
        cs_escape(name)
    )
}
