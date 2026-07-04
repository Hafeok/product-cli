//! Conformance-runner emission — the §6.3 wire-protocol console program.
//!
//! `Program.g.cs` implements exactly the protocol `product decider conform
//! --runner` speaks: a JSON array of `{given, when}` requests on stdin, a
//! JSON array of `{emit: [...]}` / `{reject: id}` outcomes on stdout, in
//! scenario order. The decider id arrives as `args[0]` and routes to the
//! aggregate's typed codec + frame, so one runner serves every aggregate
//! in the namespace.

use super::reify_ident::cs_escape;

/// One (decider id, aggregate PascalCase name) routing entry.
pub struct RunnerEntry {
    pub decider_id: String,
    pub agg: String,
}

/// Render `Program.g.cs` for the conformance runner.
pub fn program_file(header: &str, ns: &str, entries: &[RunnerEntry]) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\n");
    s.push_str("using System;\nusing System.Collections.Generic;\nusing System.Linq;\nusing System.Text.Json;\n");
    s.push_str(&format!("using {ns};\n\n"));
    s.push_str(&main_block(entries));
    for e in entries {
        s.push_str(&run_fn(e));
    }
    s
}

fn main_block(entries: &[RunnerEntry]) -> String {
    let default_id = entries.first().map(|e| e.decider_id.as_str()).unwrap_or("");
    let mut s = String::new();
    s.push_str(&format!(
        "var deciderId = args.Length > 0 ? args[0] : \"{}\";\n",
        cs_escape(default_id)
    ));
    s.push_str("var input = Console.In.ReadToEnd();\n");
    s.push_str("using var doc = JsonDocument.Parse(input);\n");
    s.push_str("var responses = new List<Dictionary<string, object?>>();\n");
    s.push_str("foreach (var request in doc.RootElement.EnumerateArray())\n{\n");
    s.push_str("    responses.Add(deciderId switch\n    {\n");
    for e in entries {
        s.push_str(&format!(
            "        \"{}\" => Run{}(request),\n",
            cs_escape(&e.decider_id),
            e.agg
        ));
    }
    s.push_str("        _ => throw new InvalidOperationException($\"unknown decider '{deciderId}'\"),\n");
    s.push_str("    });\n}\n");
    s.push_str("Console.Out.Write(JsonSerializer.Serialize(responses));\nreturn 0;\n\n");
    s
}

fn run_fn(e: &RunnerEntry) -> String {
    let agg = &e.agg;
    let mut s = String::new();
    s.push_str(&format!(
        "static Dictionary<string, object?> Run{agg}(JsonElement request)\n{{\n"
    ));
    s.push_str(&format!(
        "    var given = request.GetProperty(\"given\").EnumerateArray().Select({agg}Wire.EventFromWire);\n"
    ));
    s.push_str(&format!("    var state = {agg}Decider.Fold(given);\n"));
    s.push_str(&format!(
        "    var command = {agg}Wire.CommandFromWire(request.GetProperty(\"when\"));\n"
    ));
    s.push_str(&format!("    var result = {agg}Decider.Decide(state, command);\n"));
    s.push_str("    if (result.IsRejected) return new Dictionary<string, object?> { [\"reject\"] = result.RejectedInvariant };\n");
    s.push_str("    var emit = result.Events.Select(e =>\n    {\n");
    s.push_str("        var entry = new Dictionary<string, object?> { [\"event\"] = e.WireId };\n");
    s.push_str("        var with = e.WirePayload();\n");
    s.push_str("        if (with.Count > 0) entry[\"with\"] = with;\n");
    s.push_str("        return entry;\n    }).ToList();\n");
    s.push_str("    return new Dictionary<string, object?> { [\"emit\"] = emit };\n}\n\n");
    s
}
