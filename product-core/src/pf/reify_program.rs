//! Conformance-runner emission — the §6.3 wire-protocol console program.
//!
//! `Program.g.cs` implements exactly the protocol `product decider conform
//! --runner` speaks: a JSON array of `{given, when}` requests on stdin, a
//! JSON array of `{emit: [...]}` / `{reject: id}` outcomes on stdout, in
//! scenario order. Requests are parsed to wire form (`PfWire`) and driven
//! through an [`IConformanceAdapter`]: in full mode a generated typed
//! adapter per aggregate; in oracle-only mode the realiser's scaffolded
//! `ConformanceAdapter` — same runner, different owner of the interior.

use super::reify_ident::cs_escape;

/// One (decider id, aggregate PascalCase name) routing entry.
pub struct RunnerEntry {
    pub decider_id: String,
    pub agg: String,
}

/// Render `Program.g.cs`. `oracle_only` routes every decider to the
/// realiser's `ConformanceAdapter`; full mode routes to typed adapters.
pub fn program_file(header: &str, ns: &str, entries: &[RunnerEntry], oracle_only: bool) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\n");
    s.push_str("using System;\nusing System.Collections.Generic;\nusing System.Linq;\nusing System.Text.Json;\n");
    s.push_str(&format!("using {ns};\n\n"));
    s.push_str(&main_block(entries));
    s.push_str(&resolve_fn(entries, oracle_only));
    s
}

fn main_block(entries: &[RunnerEntry]) -> String {
    let default_id = entries.first().map(|e| e.decider_id.as_str()).unwrap_or("");
    let mut s = String::new();
    s.push_str(&format!(
        "var deciderId = args.Length > 0 ? args[0] : \"{}\";\n",
        cs_escape(default_id)
    ));
    s.push_str("var adapter = ResolveAdapter(deciderId);\n");
    s.push_str("var input = Console.In.ReadToEnd();\n");
    s.push_str("using var doc = JsonDocument.Parse(input);\n");
    s.push_str("var responses = new List<Dictionary<string, object?>>();\n");
    s.push_str("foreach (var request in doc.RootElement.EnumerateArray())\n{\n");
    s.push_str("    var given = request.GetProperty(\"given\").EnumerateArray().Select(PfWire.ParseEvent).ToList();\n");
    s.push_str("    var when = PfWire.ParseCommand(request.GetProperty(\"when\"));\n");
    s.push_str("    responses.Add(PfWire.ToResponse(adapter.Run(deciderId, given, when)));\n");
    s.push_str("}\nConsole.Out.Write(JsonSerializer.Serialize(responses));\nreturn 0;\n\n");
    s
}

fn resolve_fn(entries: &[RunnerEntry], oracle_only: bool) -> String {
    if oracle_only {
        return "static IConformanceAdapter ResolveAdapter(string deciderId) => new ConformanceAdapter();\n".to_string();
    }
    let mut s = String::from("static IConformanceAdapter ResolveAdapter(string deciderId) => deciderId switch\n{\n");
    for e in entries {
        s.push_str(&format!(
            "    \"{}\" => new {}Adapter(),\n",
            cs_escape(&e.decider_id),
            e.agg
        ));
    }
    s.push_str("    _ => throw new InvalidOperationException($\"unknown decider '{deciderId}'\"),\n};\n");
    s
}
