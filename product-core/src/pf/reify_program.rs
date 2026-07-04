//! Conformance-runner emission — the §6.3 wire-protocol console program.
//!
//! `Program.g.cs` implements the protocol both `product decider conform`
//! and `product projector conform` speak: a JSON array of requests on
//! stdin, a JSON array of outcomes on stdout, in scenario order. The id
//! in `args[0]` routes first to a projection adapter (`{given}` requests
//! → view states), else to a decision adapter (`{given, when}` requests →
//! emit/reject responses). Full mode routes to generated typed adapters;
//! oracle-only mode routes to the realiser's scaffolded adapters.

use super::reify_ident::cs_escape;

/// One (id, C# type-name) routing entry — a decider or a projector.
pub struct RunnerEntry {
    pub decider_id: String,
    pub agg: String,
}

/// Render `Program.g.cs`.
pub fn program_file(
    header: &str,
    ns: &str,
    deciders: &[RunnerEntry],
    projections: &[RunnerEntry],
    oracle_only: bool,
) -> String {
    let mut s = String::new();
    s.push_str(header);
    s.push_str("#nullable enable\n\n");
    s.push_str("using System;\nusing System.Collections.Generic;\nusing System.Linq;\nusing System.Text.Json;\n");
    s.push_str(&format!("using {ns};\n\n"));
    s.push_str(&main_block(deciders, projections));
    s.push_str(&resolve_decider_fn(deciders, oracle_only));
    s.push('\n');
    s.push_str(&resolve_projection_fn(projections, oracle_only));
    s
}

fn main_block(deciders: &[RunnerEntry], projections: &[RunnerEntry]) -> String {
    let default_id = deciders
        .first()
        .or(projections.first())
        .map(|e| e.decider_id.as_str())
        .unwrap_or("");
    let mut s = String::new();
    s.push_str(&format!(
        "var id = args.Length > 0 ? args[0] : \"{}\";\n",
        cs_escape(default_id)
    ));
    s.push_str("var projection = ResolveProjection(id);\n");
    s.push_str("var input = Console.In.ReadToEnd();\n");
    s.push_str("using var doc = JsonDocument.Parse(input);\n");
    s.push_str("var responses = new List<object?>();\n");
    s.push_str("foreach (var request in doc.RootElement.EnumerateArray())\n{\n");
    s.push_str("    var given = request.GetProperty(\"given\").EnumerateArray().Select(PfWire.ParseEvent).ToList();\n");
    s.push_str("    if (projection is not null)\n    {\n");
    s.push_str("        responses.Add(projection.Run(id, given));\n        continue;\n    }\n");
    s.push_str("    var when = PfWire.ParseCommand(request.GetProperty(\"when\"));\n");
    s.push_str("    responses.Add(PfWire.ToResponse(ResolveDecider(id).Run(id, given, when)));\n");
    s.push_str("}\nConsole.Out.Write(JsonSerializer.Serialize(responses));\nreturn 0;\n\n");
    s
}

fn resolve_decider_fn(deciders: &[RunnerEntry], oracle_only: bool) -> String {
    if oracle_only {
        return "static IConformanceAdapter ResolveDecider(string id) => new ConformanceAdapter();\n".to_string();
    }
    let mut s = String::from("static IConformanceAdapter ResolveDecider(string id) => id switch\n{\n");
    for e in deciders {
        s.push_str(&format!(
            "    \"{}\" => new {}Adapter(),\n",
            cs_escape(&e.decider_id),
            e.agg
        ));
    }
    s.push_str("    _ => throw new InvalidOperationException($\"unknown decider '{id}'\"),\n};\n");
    s
}

fn resolve_projection_fn(projections: &[RunnerEntry], oracle_only: bool) -> String {
    if projections.is_empty() {
        return "static IProjectionAdapter? ResolveProjection(string id) => null;\n".to_string();
    }
    let mut s = String::from("static IProjectionAdapter? ResolveProjection(string id) => id switch\n{\n");
    for e in projections {
        let target = if oracle_only {
            "ProjectionAdapter".to_string()
        } else {
            format!("{}ProjectionAdapter", e.agg)
        };
        s.push_str(&format!("    \"{}\" => new {target}(),\n", cs_escape(&e.decider_id)));
    }
    s.push_str("    _ => null,\n};\n");
    s
}
