//! Reify tool definitions — verifiable code projections over MCP.
//!
//! Four tools, gated to the Build phase by the `product_reify_` prefix:
//! `backends` and `manifest` are read-only (the manifest hands a session
//! agent the whole oracle by value — shapes, scenarios, baked flow
//! chains, screen facts, graph hash — so it can realise or render
//! without touching the repo); `check` is the read-only drift gate;
//! `emit` writes a built-in backend's tree under the repo.

use super::ToolDef;

pub(super) fn all() -> Vec<ToolDef> {
    vec![backends_tool(), manifest_tool(), check_tool(), emit_tool()]
}

fn backends_tool() -> ToolDef {
    ToolDef {
        name: "product_reify_backends".to_string(),
        description: "List the built-in reify language backends (id, description, tier). External backends run out-of-band via `product reify plugin` — a process consuming the reify manifest.".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({ "type": "object", "properties": {} }),
    }
}

fn manifest_tool() -> ToolDef {
    ToolDef {
        name: "product_reify_manifest".to_string(),
        description: "Return the language-neutral reify manifest: the whole verification oracle by value — per-aggregate payload shapes (declared §3.2 fields over inference), Decider/Projector scenarios, oracle-baked §3.2 flow chains, §4.5 screen facts with projector-derived fixtures, and the pinned graph hash. Everything needed to realise (or generate a verification shell for) any language, with no repo access.".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "namespace": {"type": "string", "description": "Type/package namespace (default: PascalCase of the product name)"},
                "product": {"type": "string"}
            }
        }),
    }
}

fn check_tool() -> ToolDef {
    ToolDef {
        name: "product_reify_check".to_string(),
        description: "Drift gate (§7.3): recompute the graph hash (canonical Turtle + decider/projector YAMLs) and compare it with the hash a reified tree was generated from. Returns { conformant, current, recorded }.".to_string(),
        requires_write: false,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "out": {"type": "string", "description": "The reified tree, relative to the repo root (e.g. reified/<product>/csharp)"},
                "product": {"type": "string"}
            },
            "required": ["out"]
        }),
    }
}

fn emit_tool() -> ToolDef {
    ToolDef {
        name: "product_reify_emit".to_string(),
        description: "Emit a built-in backend's verification shell under the repo — either from a §4.2 realisation declared in the How contract (`realisation`: the captured backend/tier/namespace decision), or ad-hoc via `lang`. Scaffolded realiser-owned files are never overwritten; stale generated files from a previous run are removed. Returns { written, kept, stale, graph_hash, out }.".to_string(),
        requires_write: true,
        input_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "realisation": {"type": "string", "description": "A realisation id from the How contract's §4.2 realisations: block (takes precedence over lang)"},
                "lang": {"type": "string", "description": "Backend id: csharp | kotlin (ad-hoc, when no realisation is given)"},
                "out": {"type": "string", "description": "Output directory relative to the repo root (default: reified/<product>/<lang>)"},
                "namespace": {"type": "string", "description": "Type/package namespace (default: PascalCase of the product name)"},
                "oracle_only": {"type": "boolean", "description": "C# only: emit just the adapter seam + facts (kotlin is always oracle-only)"},
                "product": {"type": "string"}
            },
            "required": []
        }),
    }
}
