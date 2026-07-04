//! C# Projector emission — view record, fold frame, projection adapter.
//!
//! The read-side peer of `reify_decider`: per Projector a typed view
//! record (fields inferred from `logic.initial` + apply rules + scenario
//! `then` states), a `static partial` fold frame whose `Apply` the
//! realiser implements (events arrive in wire form — projections consume
//! the stream, so no per-aggregate typed union is needed), a generated
//! `IProjectionAdapter`, and one xUnit fact per scenario asserting the
//! folded view's wire state equals the oracle's (full-state equality,
//! matching `projector_sim`).

use std::collections::BTreeSet;

use super::decider_logic::{Scalar, State};
use super::projector::Projector;
use super::projector_logic::ProjectorScenario;
use super::reify_ident::{cs_escape, method_name, pascal, CsTy};
use super::reify_infer::Fields;
use super::reify_oracle::wire_new;
use super::reify_scenarios::count_assert;
use super::reify_types::scalar_literal;

/// The PascalCase base name for a projector's generated types
/// (`OrderSummary` → `OrderSummaryView` / `OrderSummaryProjector`).
pub fn view_base(projector: &Projector) -> String {
    pascal(&projector.projects_for)
}

/// Infer the view's fields: names/defaults from `logic.initial`, names from
/// apply-rule `set` keys, types from every concrete scenario `then` value.
pub fn infer_view(projector: &Projector) -> (Fields, State) {
    let mut fields = Fields::new();
    let mut defaults = State::new();
    if let Some(logic) = &projector.logic {
        for (k, v) in &logic.initial {
            fields.insert(k.clone(), Some(scalar_ty(v)));
            defaults.insert(k.clone(), v.clone());
        }
        for rule in &logic.apply {
            for k in rule.set.keys() {
                fields.entry(k.clone()).or_insert(None);
            }
        }
    }
    for s in &projector.scenarios {
        for (k, v) in &s.then {
            let slot = fields.entry(k.clone()).or_insert(None);
            *slot = Some(CsTy::merge(*slot, scalar_ty(v)));
        }
    }
    (fields, defaults)
}

fn scalar_ty(s: &Scalar) -> CsTy {
    match s {
        Scalar::Bool(_) => CsTy::Bool,
        Scalar::Int(_) => CsTy::Long,
        Scalar::Str(_) => CsTy::Str,
    }
}

/// Render `<View>Projector.g.cs` — the view record plus the fold frame.
pub fn frame_file(header: &str, ns: &str, projector: &Projector, fields: &Fields, defaults: &State) -> String {
    let base = view_base(projector);
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System.Collections.Generic;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&view_record(&base, projector, fields, defaults));
    s.push('\n');
    s.push_str(&frame(&base, projector));
    s
}

fn view_record(base: &str, projector: &Projector, fields: &Fields, defaults: &State) -> String {
    let mut s = format!(
        "/// <summary>View state for read model '{}' — fields inferred from logic.initial, apply rules, scenario expectations (§3.4).</summary>\npublic sealed record {base}View\n{{\n",
        cs_escape(&projector.projects_for)
    );
    for (name, ty) in fields {
        s.push_str(&view_field(name, *ty, defaults.get(name)));
    }
    s.push_str(&wire_state(fields, defaults));
    s.push_str("}\n");
    s
}

fn view_field(name: &str, ty: Option<CsTy>, default: Option<&Scalar>) -> String {
    let prop = pascal(name);
    match (ty, default) {
        (Some(t), Some(d)) => format!(
            "    public {} {prop} {{ get; init; }} = {};\n",
            t.name(),
            scalar_literal(t, d)
        ),
        (Some(t), None) => format!("    public {}? {prop} {{ get; init; }}\n", t.name()),
        (None, _) => format!("    public string? {prop} {{ get; init; }}\n"),
    }
}

/// `WireState()` mirrors the oracle's `State` semantics exactly: fields from
/// `initial` are always present; apply-only fields appear once set.
fn wire_state(fields: &Fields, defaults: &State) -> String {
    let mut s = String::from(
        "\n    /// <summary>The view in wire form — equals the oracle's folded state (§6.3 full-state equality).</summary>\n    public IReadOnlyDictionary<string, object?> WireState()\n    {\n        var d = new Dictionary<string, object?>();\n",
    );
    for name in fields.keys() {
        let prop = pascal(name);
        if defaults.contains_key(name) {
            s.push_str(&format!("        d[\"{}\"] = {prop};\n", cs_escape(name)));
        } else {
            s.push_str(&format!(
                "        if ({prop} is not null) d[\"{}\"] = {prop};\n",
                cs_escape(name)
            ));
        }
    }
    s.push_str("        return d;\n    }\n");
    s
}

fn frame(base: &str, projector: &Projector) -> String {
    let folds: Vec<String> = projector.folds.iter().map(|f| format!("\"{}\"", cs_escape(f))).collect();
    let mut s = format!(
        "/// <summary>§3.4 Projector frame for read model '{}' — implement Apply in {base}Projector.cs; events arrive in wire form.</summary>\npublic static partial class {base}Projector\n{{\n",
        cs_escape(&projector.projects_for)
    );
    s.push_str(&format!(
        "    /// <summary>§3.4 `folds` — pinned from the What graph.</summary>\n    public static readonly string[] Folds = {{ {} }};\n\n",
        folds.join(", ")
    ));
    s.push_str(&format!(
        "    /// <summary>Fold one event into the view (§3.4 apply).</summary>\n    public static partial {base}View Apply({base}View view, WireEvent evt);\n\n"
    ));
    s.push_str(&format!(
        "    /// <summary>Initial view — defaults come from the record's initializers (logic.initial).</summary>\n    public static {base}View InitialView() => new {base}View();\n\n"
    ));
    s.push_str(&format!(
        "    /// <summary>Replay a stream into the view.</summary>\n    public static {base}View Fold(IEnumerable<WireEvent> events)\n    {{\n        var view = InitialView();\n        foreach (var e in events) view = Apply(view, e);\n        return view;\n    }}\n}}\n"
    ));
    s
}

/// Render the editable `<View>Projector.cs` stub (written only if missing).
pub fn stub_file(ns: &str, projector: &Projector) -> String {
    let base = view_base(projector);
    let mut s = format!(
        "// Editable realisation of the '{}' Projector (§3.4). Scaffolded once by\n// `product reify csharp` and never overwritten — author Apply here.\n// Verify with the generated projection tests, then close the loop with:\n//   product projector conform {}\n",
        cs_escape(&projector.projects_for),
        projector.id
    );
    s.push_str("#nullable enable\n\nusing System;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&format!("public static partial class {base}Projector\n{{\n"));
    s.push_str(&format!(
        "    public static partial {base}View Apply({base}View view, WireEvent evt)\n    {{\n        // TODO: fold each event into the view, e.g. reading PfWire.GetLong(evt.With, \"amount\").\n        throw new NotImplementedException(\"realise Apply for '{}'\");\n    }}\n}}\n",
        cs_escape(&projector.projects_for)
    ));
    s
}

/// Render `<View>ProjectionAdapter.g.cs` — the typed [`IProjectionAdapter`].
pub fn adapter_file(header: &str, ns: &str, projector: &Projector) -> String {
    let base = view_base(projector);
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System.Collections.Generic;\n\n");
    s.push_str(&format!("namespace {ns};\n\n"));
    s.push_str(&format!(
        "/// <summary>§6.3 projection adapter for read model '{}' — wire events through the realised fold.</summary>\npublic sealed class {base}ProjectionAdapter : IProjectionAdapter\n{{\n    public IReadOnlyDictionary<string, object?> Run(string projectorId, IReadOnlyList<WireEvent> given) =>\n        {base}Projector.Fold(given).WireState();\n}}\n",
        cs_escape(&projector.projects_for)
    ));
    s
}

/// Render the projection scenario facts. In full mode they drive the typed
/// frame; in oracle-only mode the scaffolded `ProjectionAdapter`.
pub fn tests_file(header: &str, ns: &str, projector: &Projector, oracle_only: bool) -> String {
    let base = view_base(projector);
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\nusing System;\nusing System.Collections.Generic;\nusing Xunit;\n");
    s.push_str(&format!("using {ns};\n\nnamespace {ns}.Tests;\n\n"));
    s.push_str(&format!(
        "/// <summary>§6.3 — the '{}' projection scenarios replayed against the realised fold.</summary>\npublic class {base}ProjectionTests\n{{\n",
        cs_escape(&projector.id)
    ));
    let mut seen = BTreeSet::new();
    for scenario in &projector.scenarios {
        s.push_str(&fact(&base, projector, scenario, oracle_only, &mut seen));
    }
    s.push_str("}\n");
    s
}

fn fact(base: &str, projector: &Projector, scenario: &ProjectorScenario, oracle_only: bool, seen: &mut BTreeSet<String>) -> String {
    let mut name = method_name(&scenario.name);
    while !seen.insert(name.clone()) {
        name.push('_');
    }
    let mut s = format!("    [Fact]\n    public void {name}()\n    {{\n");
    s.push_str(&format!("        var given = {};\n", given_expr(scenario)));
    if oracle_only {
        s.push_str(&format!(
            "        var wire = new ProjectionAdapter().Run(\"{}\", given);\n",
            cs_escape(&projector.id)
        ));
    } else {
        s.push_str(&format!("        var wire = {base}Projector.Fold(given).WireState();\n"));
    }
    s.push_str(&count_assert("wire", scenario.then.len()));
    for (k, v) in &scenario.then {
        s.push_str(&then_assert(k, v));
    }
    s.push_str("    }\n\n");
    s
}

fn given_expr(scenario: &ProjectorScenario) -> String {
    if scenario.given.is_empty() {
        return "Array.Empty<WireEvent>()".to_string();
    }
    let items: Vec<String> = scenario
        .given
        .iter()
        .map(|ev| format!("            {},\n", wire_new("WireEvent", ev.id(), &ev.payload())))
        .collect();
    format!("new[]\n        {{\n{}        }}", items.join(""))
}

fn then_assert(name: &str, value: &Scalar) -> String {
    let (ty, lit) = match value {
        Scalar::Bool(b) => ("bool", b.to_string()),
        Scalar::Int(i) => ("long", format!("{i}L")),
        Scalar::Str(v) => ("string", format!("\"{}\"", cs_escape(v))),
    };
    format!(
        "        Assert.Equal({lit}, Assert.IsType<{ty}>(wire[\"{}\"]));\n",
        cs_escape(name)
    )
}
