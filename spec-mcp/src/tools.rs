//! The tool list: the delegable verbs, and a list of what is deliberately absent.

use product_mcp::tools::ToolDef;
use serde_json::json;

/// Verbs this surface must never carry, each because it names a principal.
///
/// Listed rather than merely omitted, so that adding one is a visible edit to
/// a named constant and the test below fails the moment the list and the
/// registry disagree.
pub const WITHHELD: &[&str] = &["spec_accept", "spec_reject", "spec_close", "spec_policy_set"];

/// The tools a model may call.
pub fn build() -> Vec<ToolDef> {
    vec![
        read(
            "spec_candidates",
            "List import candidates with what was observed at the transport and \
             which slots ratification must still fill. A candidate is a question, \
             not a proposal: it never names an act.",
            json!({
                "type": "object",
                "properties": {
                    "unreviewed": {
                        "type": "boolean",
                        "description": "Only candidates nobody has ratified or refused."
                    }
                }
            }),
        ),
        read(
            "spec_map",
            "Join ratified acts to entry points and report the disagreements: \
             merge, split, unmapped entry point, unmapped act. A work list, not verdicts.",
            json!({"type": "object", "properties": {}}),
        ),
        read(
            "spec_check",
            "Run the gate. Returns structural verdicts and the project's own policy \
             verdicts apart from one another, plus the reported metrics.",
            json!({"type": "object", "properties": {}}),
        ),
        read(
            "spec_records",
            "List act-time records and whether each is still open.",
            json!({
                "type": "object",
                "properties": {
                    "open": {"type": "boolean", "description": "Only records still open."}
                }
            }),
        ),
        read(
            "spec_policy_show",
            "Show the check policy in force: what it gates, on whose word, with what basis, \
             and what it reports and deliberately does not gate.",
            json!({"type": "object", "properties": {}}),
        ),
        ToolDef {
            name: "spec_implement".into(),
            description: "Open an act-time record for a slice built against the specification. \
                          Produces a PENDING record: the closure is a principal's act and cannot \
                          be done from here. Returns the command a person runs to close it."
                .into(),
            requires_write: true,
            input_schema: json!({
                "type": "object",
                "properties": {
                    "slice": {"type": "string", "description": "The slice being built."},
                    "act_ref": {"type": "string", "description": "The ratified act it realises."},
                    "by": {
                        "type": "string",
                        "description": "Who opens it. May be a machine — building is the delegable half."
                    }
                },
                "required": ["slice", "act_ref"]
            }),
        },
    ]
}

fn read(name: &str, description: &str, input_schema: serde_json::Value) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: description.into(),
        requires_write: false,
        input_schema,
    }
}

#[path = "tools_tests.rs"]
#[cfg(test)]
mod tests;
