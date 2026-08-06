//! The `ddd_*` tool definitions — names, schemas, write flags (PRD §8).

use product_mcp::tools::ToolDef;
use serde_json::{json, Value};

fn tool(name: &str, description: &str, write: bool, props: Value, required: &[&str]) -> ToolDef {
    ToolDef {
        name: name.to_string(),
        description: description.to_string(),
        requires_write: write,
        input_schema: json!({
            "type": "object",
            "properties": props,
            "required": required,
        }),
    }
}

fn file_props(extra: Value) -> Value {
    let mut props = json!({
        "file": {"type": "string", "description": "File path, repo-root-relative or absolute"},
        "language": {"type": "string", "description": "Adapter override (csharp | bicep); inferred from the file extension when omitted"},
    });
    if let (Some(base), Some(add)) = (props.as_object_mut(), extra.as_object()) {
        for (k, v) in add {
            base.insert(k.clone(), v.clone());
        }
    }
    props
}

fn position_props() -> Value {
    file_props(json!({
        "line": {"type": "integer", "description": "Zero-based line"},
        "character": {"type": "integer", "description": "Zero-based character"},
    }))
}

/// Every tool `ddd serve` exposes, M3 language intelligence plus M4
/// governance, in the order agents see them.
pub fn build() -> Vec<ToolDef> {
    vec![
        find_symbol(),
        references(),
        hover(),
        diagnostics(),
        signature(),
        rename(),
        apply_edit(),
        warmup(),
        why(),
        graph_query(),
        declare_seam(),
        declare_pattern(),
        accept_risk(),
    ]
}

fn find_symbol() -> ToolDef {
    tool(
        "ddd_find_symbol",
        "Find symbols: in one file (documentSymbol) or across the workspace (query).",
        false,
        file_props(json!({"query": {"type": "string", "description": "Workspace symbol query (used when no file is given)"}})),
        &[],
    )
}

fn references() -> ToolDef {
    tool(
        "ddd_references",
        "References to the symbol at a position.",
        false,
        position_props(),
        &["file", "line", "character"],
    )
}

fn hover() -> ToolDef {
    tool(
        "ddd_hover",
        "Hover information at a position.",
        false,
        position_props(),
        &["file", "line", "character"],
    )
}

fn diagnostics() -> ToolDef {
    tool(
        "ddd_diagnostics",
        "Diagnostics for a file, each joined to its .ddd/ manifest governance by rule id.",
        false,
        file_props(json!({})),
        &["file"],
    )
}

fn signature() -> ToolDef {
    tool(
        "ddd_signature",
        "Signature help at a position.",
        false,
        position_props(),
        &["file", "line", "character"],
    )
}

fn rename() -> ToolDef {
    tool(
        "ddd_rename",
        "Compute (not apply) the workspace edit renaming the symbol at a position; apply through ddd_apply_edit so the interceptor sees it.",
        false,
        file_props(json!({
            "line": {"type": "integer"}, "character": {"type": "integer"},
            "new_name": {"type": "string"},
        })),
        &["file", "line", "character", "new_name"],
    )
}

fn apply_edit() -> ToolDef {
    tool(
        "ddd_apply_edit",
        "Apply an edit to a file through the contract-surface interceptor (PRD §8): non-surface applies; surface needs a same-session seam/pattern declaration or is rejected with a structured demand.",
        true,
        file_props(json!({
            "new_text": {"type": "string", "description": "Full replacement content (or use `edits`)"},
            "edits": {"type": "array", "description": "Range edits [{range:{start:{line,character},end:{line,character}}, new_text}]",
                      "items": {"type": "object"}},
        })),
        &["file"],
    )
}

fn warmup() -> ToolDef {
    tool(
        "ddd_warmup",
        "Start the language hosts, optionally waiting for readiness (wait_ms).",
        false,
        json!({
            "languages": {"type": "array", "items": {"type": "string"}},
            "wait_ms": {"type": "integer", "description": "Block up to this long for readiness"},
        }),
        &[],
    )
}

fn why() -> ToolDef {
    tool(
        "ddd_why",
        "Resolve a diagnostic, pattern, seam, claim, or decision id to its governance chain.",
        false,
        json!({"id": {"type": "string"}}),
        &["id"],
    )
}

fn graph_query() -> ToolDef {
    tool(
        "ddd_graph_query",
        "Read the .ddd/ graph: select predicates | claims | decisions | manifests | patterns | seams | seam-events, filtered by id/status/predicate.",
        false,
        json!({
            "select": {"type": "string"},
            "id": {"type": "string", "description": "Exact id filter"},
            "status": {"type": "string", "description": "Claim status filter"},
            "predicate": {"type": "string", "description": "Predicate id filter (claims)"},
        }),
        &["select"],
    )
}

fn declare_seam() -> ToolDef {
    tool(
        "ddd_declare_seam",
        "File a seam declaration (boundary + verdict_knowledge + contract_location). Empty verdict_knowledge files with a seam-cost-without-demand warning.",
        true,
        json!({
            "id": {"type": "string", "description": "seam/<area>/<name>"},
            "boundary": {"type": "string"},
            "verdict_knowledge": {"type": "string", "description": "What the boundary encodes about the verdict — author this; it is never pre-filled"},
            "contract_location": {"type": "string"},
            "obligations": {"type": "array", "items": {"type": "string"}},
            "symbol": {"type": "string", "description": "The symbol this seam covers (links interceptor demands)"},
            "notes": {"type": "string"},
        }),
        &["id", "boundary", "contract_location"],
    )
}

fn declare_pattern() -> ToolDef {
    tool(
        "ddd_declare_pattern",
        "File a pattern instance against a catalog pattern predicate; every catalog obligation must be answered.",
        true,
        json!({
            "id": {"type": "string", "description": "pat/<pattern>/<instance>"},
            "pattern": {"type": "string", "description": "The catalog pattern predicate id"},
            "instance": {"type": "string", "description": "Where the instance lives"},
            "obligation_answers": {"type": "object", "description": "Answers keyed by the catalog obligation names"},
            "notes": {"type": "string"},
        }),
        &["id", "pattern", "instance"],
    )
}

fn accept_risk() -> ToolDef {
    tool(
        "ddd_accept_risk",
        "File the risk-acceptance record a suppression must cite (M2 UNCITED_SUPPRESSION).",
        true,
        json!({
            "diagnostic_id": {"type": "string"},
            "rationale": {"type": "string"},
            "principal": {"type": "string", "description": "A named human; never a model identity"},
        }),
        &["diagnostic_id", "rationale", "principal"],
    )
}
