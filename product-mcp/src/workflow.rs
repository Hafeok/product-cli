//! Phase-gated workflow transport — drive one What → How → Build session.
//!
//! Wraps the stateless [`ToolRegistry`] with a disk-backed
//! [`WorkflowSession`]: `tools/list` is filtered to the current phase, calls to
//! out-of-phase tools are rejected, and the session-control tools advance the
//! phase or close the session. Every tool dispatches against the canonical
//! `.product` graph directly — `product_session_finalize` validates it and
//! marks the session complete.

use std::path::PathBuf;

use product_core::author::domain::session_dir;
use product_core::pf::session::DomainSession;
use product_core::pf::workflow::{Phase, WorkflowSession};
use serde_json::{json, Value};

use super::registry::ToolRegistry;
use super::tools::ToolDef;
use super::{JsonRpcRequest, JsonRpcResponse};

/// The per-session context resolved by the transport for each call.
pub struct WorkflowCtx {
    /// Directory holding `workflow.json` (`.product/sessions/<id>/`).
    pub session_root: PathBuf,
    /// The canonical repo root every tool dispatches against.
    pub canonical: PathBuf,
}

impl WorkflowCtx {
    /// The on-disk layout for a session id under the canonical repo root.
    pub fn resolve(canonical: &std::path::Path, session_id: &str) -> Self {
        let session_root = canonical.join(".product").join("sessions").join(session_id);
        Self { session_root, canonical: canonical.to_path_buf() }
    }
}

/// A JSON-RPC response plus any server-initiated notifications to flush after it
/// (e.g. `notifications/tools/list_changed` on a phase advance).
pub struct Outgoing {
    pub response: Option<JsonRpcResponse>,
    pub notifications: Vec<Value>,
}

impl Outgoing {
    fn resp(r: JsonRpcResponse) -> Self {
        Self { response: Some(r), notifications: Vec::new() }
    }
    fn silent() -> Self {
        Self { response: None, notifications: Vec::new() }
    }
}

const CONTROL_TOOLS: [&str; 3] =
    ["product_workflow_status", "product_workflow_advance", "product_session_finalize"];

/// The home phase a tool belongs to, by name prefix (the single source of truth
/// for gating; control tools are handled separately and are always visible).
pub fn phase_of(name: &str) -> Phase {
    const WHAT: [&str; 6] = [
        "product_product_",
        "product_domain_",
        "product_decider_",
        "product_projector_",
        "product_primitive_",
        // Authoring scopes are an intake / What concept (§14).
        "product_scope_",
    ];
    const HOW: [&str; 8] = [
        "product_how_",
        "product_design_system_",
        "product_blueprint_",
        // Back-compat: the pre-v1.7.0 `product_archetype_*` names still gate to How.
        "product_archetype_",
        "product_deployable_unit_",
        "product_cell_",
        "product_work_unit_",
        "product_worker_",
    ];
    if WHAT.iter().any(|p| name.starts_with(p)) {
        Phase::What
    } else if HOW.iter().any(|p| name.starts_with(p)) {
        Phase::How
    } else {
        // feature / deliverable / release / build_run
        Phase::Build
    }
}

/// A tool is callable in `current` if it is home to that phase, or it is a
/// read-only tool from an earlier phase (so earlier work stays inspectable).
pub fn is_visible(tool: &ToolDef, current: Phase) -> bool {
    let home = phase_of(&tool.name);
    home == current || (home < current && !tool.requires_write)
}

/// The session-control tool definitions, advertised in every phase (the
/// Copilot SDK host also registers them, alongside the registry surface).
pub(crate) fn control_tools() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "product_workflow_status".into(),
            description: "Show the current workflow phase, how far the session may advance, the tools available now, and the journey so far.".into(),
            requires_write: false,
            input_schema: json!({"type": "object", "properties": {}}),
        },
        ToolDef {
            name: "product_workflow_advance".into(),
            description: "Advance the session to the next phase (What→How→Build), or jump to a named phase via `to`. Returns the tools now available.".into(),
            requires_write: true,
            input_schema: json!({"type": "object", "properties": {"to": {"type": "string", "description": "what | how | build"}}}),
        },
        ToolDef {
            name: "product_session_finalize".into(),
            description: "Validate the What graph and, if conformant, stamp provenance and close the session. Writes have already landed in the canonical `.product` graph.".into(),
            requires_write: true,
            input_schema: json!({"type": "object", "properties": {}}),
        },
    ]
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn load_session(ctx: &WorkflowCtx) -> Result<WorkflowSession, String> {
    WorkflowSession::load(&ctx.session_root).map_err(|e| format!("{e}"))
}

/// The tools callable in the given phase: the filtered families plus controls.
fn visible_tools(registry: &ToolRegistry, phase: Phase) -> Vec<&ToolDef> {
    registry.tool_list().iter().filter(|t| is_visible(t, phase)).collect()
}


fn tool_json(t: &ToolDef) -> Value {
    json!({ "name": t.name, "description": t.description, "inputSchema": t.input_schema })
}

/// Entry point: handle one JSON-RPC request in workflow mode.
pub fn handle(registry: &ToolRegistry, request: &JsonRpcRequest, ctx: &WorkflowCtx) -> Outgoing {
    if request.method.starts_with("notifications/") {
        return Outgoing::silent();
    }
    match request.method.as_str() {
        "initialize" => Outgoing::resp(initialize(request)),
        "tools/list" => Outgoing::resp(tools_list(registry, request, ctx)),
        "tools/call" => tools_call(registry, request, ctx),
        other => Outgoing::resp(JsonRpcResponse::error(
            request.id.clone(),
            -32601,
            &format!("Method not found: {other}"),
        )),
    }
}

fn initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
    JsonRpcResponse::success(
        request.id.clone(),
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": { "listChanged": true } },
            "serverInfo": { "name": product_core::author::MCP_SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
        }),
    )
}

fn tools_list(registry: &ToolRegistry, request: &JsonRpcRequest, ctx: &WorkflowCtx) -> JsonRpcResponse {
    let phase = load_session(ctx).map(|s| s.phase).unwrap_or(Phase::What);
    let mut tools: Vec<Value> = visible_tools(registry, phase).iter().map(|t| tool_json(t)).collect();
    tools.extend(control_tools().iter().map(tool_json));
    JsonRpcResponse::success(request.id.clone(), json!({ "tools": tools }))
}

fn tools_call(registry: &ToolRegistry, request: &JsonRpcRequest, ctx: &WorkflowCtx) -> Outgoing {
    let id = request.id.clone();
    let name = request.params.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    let args = request.params.get("arguments").cloned().unwrap_or(json!({}));

    if CONTROL_TOOLS.contains(&name.as_str()) {
        return control_call(registry, &name, &args, ctx, id);
    }

    // Gate: the tool must be visible in the current phase.
    let session = match load_session(ctx) {
        Ok(s) => s,
        Err(e) => return Outgoing::resp(JsonRpcResponse::error(id, -32603, &e)),
    };
    let Some(tool) = registry.tool_list().iter().find(|t| t.name == name) else {
        return Outgoing::resp(JsonRpcResponse::error(id, -32603, &format!("Tool not found: {name}")));
    };
    if !is_visible(tool, session.phase) {
        let msg = format!(
            "Tool '{name}' belongs to the {} phase, but this session is in the {} phase. Advance with product_workflow_advance.",
            phase_of(&name), session.phase
        );
        return Outgoing::resp(JsonRpcResponse::error(id, -32603, &msg));
    }

    let result = registry.call_tool_at(&name, &args, &ctx.canonical);
    if name == "product_build_run" {
        if let Ok(ref v) = result {
            record_build(&session, ctx, v);
        }
    }
    Outgoing::resp(call_response(id, result))
}

fn call_response(id: Option<Value>, result: Result<Value, String>) -> JsonRpcResponse {
    match result {
        Ok(v) => JsonRpcResponse::success(id, json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&v).unwrap_or_default() }]
        })),
        Err(e) => JsonRpcResponse::error(id, -32603, &e),
    }
}

/// Persist the build verdict onto the session record (best-effort).
fn record_build(session: &WorkflowSession, ctx: &WorkflowCtx, result: &Value) {
    if let Some(verdict) = result.get("verdict").filter(|v| !v.is_null()) {
        if let Ok(v) = serde_json::from_value(verdict.clone()) {
            let mut s = session.clone();
            s.record_build(v);
            let _ = s.save(&ctx.session_root);
        }
    }
}

// --- session-control tools -------------------------------------------------

fn control_call(registry: &ToolRegistry, name: &str, args: &Value, ctx: &WorkflowCtx, id: Option<Value>) -> Outgoing {
    match name {
        "product_workflow_status" => Outgoing::resp(call_response(id, status(registry, ctx))),
        "product_workflow_advance" => advance(registry, args, ctx, id),
        "product_session_finalize" => Outgoing::resp(call_response(id, finalize(ctx))),
        _ => Outgoing::resp(JsonRpcResponse::error(id, -32603, &format!("unknown control tool: {name}"))),
    }
}

fn status(registry: &ToolRegistry, ctx: &WorkflowCtx) -> Result<Value, String> {
    let session = load_session(ctx)?;
    let mut available: Vec<String> = visible_tools(registry, session.phase).iter().map(|t| t.name.clone()).collect();
    available.extend(CONTROL_TOOLS.iter().map(|s| s.to_string()));
    let mut out = session.status_json();
    if let Value::Object(ref mut m) = out {
        m.insert("availableTools".into(), json!(available));
        m.insert("hint".into(), json!(phase_hint(session.phase)));
    }
    Ok(out)
}

fn phase_hint(phase: Phase) -> String {
    match phase {
        Phase::What => "Author the domain/event model in dependency order — domains → systems → flows: model the domain first (the hardest, and everything references it), then the systems that reference it, then the flows that belong to them (a flow cannot exist without a system, §3.2.5). Every node is a product_domain_new call — there is no per-kind tool: systems → kind=system with system_kind=service|application|website|cli; flows → kind=flow with system=<id>; the product's What-version → kind=product with version=<v>. Make behaviour executable with product_decider_* / product_projector_*. Advance to How (where product_how_set carries the §7.3 realises-version) when product_domain_validate is green.".into(),
        Phase::How => "Author the How: product_how_init scaffolds the contract, product_how_add / product_how_set build the Why cascade (decisions → principles → patterns) plus the application/infrastructure contracts. Inspect with product_blueprint_* / product_work_unit_*. Advance to Build when the architecture is set.".into(),
        Phase::Build => "Run product_build_run on a deliverable. Call product_session_finalize to validate the graph and close the session.".into(),
    }
}

fn advance(registry: &ToolRegistry, args: &Value, ctx: &WorkflowCtx, id: Option<Value>) -> Outgoing {
    let mut session = match load_session(ctx) {
        Ok(s) => s,
        Err(e) => return Outgoing::resp(JsonRpcResponse::error(id, -32603, &e)),
    };
    let to = match args.get("to").and_then(|v| v.as_str()) {
        Some(s) => match Phase::parse(s) {
            Ok(p) => Some(p),
            Err(e) => return Outgoing::resp(JsonRpcResponse::error(id, -32603, &format!("{e}"))),
        },
        None => None,
    };
    let from = session.phase;
    let target = match session.advance(to, now()) {
        Ok(p) => p,
        Err(e) => return Outgoing::resp(JsonRpcResponse::error(id, -32603, &format!("{e}"))),
    };
    if let Err(e) = session.save(&ctx.session_root) {
        return Outgoing::resp(JsonRpcResponse::error(id, -32603, &format!("{e}")));
    }
    let now_available: Vec<String> = visible_tools(registry, target).iter().map(|t| t.name.clone()).collect();
    let result = Ok(json!({
        "from": from, "to": target,
        "nowAvailable": now_available,
        "hint": phase_hint(target),
    }));
    Outgoing {
        response: Some(call_response(id, result)),
        notifications: vec![json!({ "jsonrpc": "2.0", "method": "notifications/tools/list_changed" })],
    }
}

/// Validate the canonical What graph; on conformance, stamp provenance and mark
/// the session finalized. The session's writes already live in canonical —
/// finalize is a validation gate + completion record, not a promotion.
fn finalize(ctx: &WorkflowCtx) -> Result<Value, String> {
    let mut session = load_session(ctx)?;
    let product = session.product.clone();
    let canon_dir = session_dir(&ctx.canonical, &product);
    let graph = DomainSession::load(&canon_dir).map_err(|e| format!("no What graph: {e}"))?;
    let fin = graph.finalize(now());
    if !fin.ok {
        return Ok(json!({ "ok": false, "violations": fin.violations }));
    }

    // Stamp the validated .ttl + provenance (the completion artifact).
    let mut written = vec![];
    if let Some(ttl) = fin.turtle {
        let p = canon_dir.join(format!("{product}.ttl"));
        product_core::fileops::write_file_atomic(&p, &ttl).map_err(|e| format!("{e}"))?;
        written.push(p.display().to_string());
    }
    if let Some(prov) = fin.provenance {
        if let Ok(text) = serde_json::to_string_pretty(&prov) {
            let p = canon_dir.join(format!("{product}.provenance.json"));
            product_core::fileops::write_file_atomic(&p, &text).map_err(|e| format!("{e}"))?;
            written.push(p.display().to_string());
        }
    }

    session.finalized = true;
    session.save(&ctx.session_root).map_err(|e| format!("{e}"))?;
    Ok(json!({ "ok": true, "product": product, "written": written }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(name: &str, write: bool) -> ToolDef {
        ToolDef { name: name.into(), description: String::new(), requires_write: write, input_schema: json!({}) }
    }

    #[test]
    fn phase_of_maps_families() {
        assert_eq!(phase_of("product_product_new"), Phase::What);
        assert_eq!(phase_of("product_domain_new"), Phase::What);
        assert_eq!(phase_of("product_decider_validate"), Phase::What);
        assert_eq!(phase_of("product_scope_add"), Phase::What);
        assert_eq!(phase_of("product_scope_enforce"), Phase::What);
        assert_eq!(phase_of("product_how_show"), Phase::How);
        assert_eq!(phase_of("product_how_add"), Phase::How);
        assert_eq!(phase_of("product_blueprint_init"), Phase::How);
        assert_eq!(phase_of("product_design_system_bind"), Phase::How);
        assert_eq!(phase_of("product_design_system_show"), Phase::How);
        assert_eq!(phase_of("product_cell_dispatch"), Phase::How);
        assert_eq!(phase_of("product_work_unit_init"), Phase::How);
        assert_eq!(phase_of("product_work_unit_show"), Phase::How);
        assert_eq!(phase_of("product_feature_new"), Phase::Build);
        assert_eq!(phase_of("product_build_run"), Phase::Build);
        // Codegen is realisation — home phase Build (reads stay visible later).
        assert_eq!(phase_of("product_codegen_manifest"), Phase::Build);
        assert_eq!(phase_of("product_codegen_emit"), Phase::Build);
        // Back-compat: the pre-v1.9.1 `product_reify_*` names still gate to Build.
        assert_eq!(phase_of("product_reify_emit"), Phase::Build);
    }

    #[test]
    fn codegen_tools_are_registered_with_the_right_write_gating() {
        let tools = crate::tools::build_tool_list();
        let find = |n: &str| tools.iter().find(|t| t.name == n).unwrap_or_else(|| panic!("{n} missing"));
        assert!(!find("product_codegen_backends").requires_write);
        assert!(!find("product_codegen_manifest").requires_write);
        assert!(!find("product_codegen_check").requires_write);
        assert!(find("product_codegen_emit").requires_write, "emit writes the repo");
    }

    #[test]
    fn write_tools_lock_to_their_home_phase() {
        let new = tool("product_domain_new", true);
        let show = tool("product_domain_show", false);
        // In What, both What tools are visible.
        assert!(is_visible(&new, Phase::What));
        assert!(is_visible(&show, Phase::What));
        // In How, the What read stays but the What write is locked.
        assert!(is_visible(&show, Phase::How));
        assert!(!is_visible(&new, Phase::How));
    }

    #[test]
    fn how_authoring_writes_live_only_in_how() {
        let add = tool("product_how_add", true);
        // Hidden while still authoring the What…
        assert!(!is_visible(&add, Phase::What));
        // …live in How…
        assert!(is_visible(&add, Phase::How));
        // …and frozen once the architecture is set and Build begins.
        assert!(!is_visible(&add, Phase::Build));
    }

    #[test]
    fn later_phase_tools_hidden_earlier() {
        let build = tool("product_build_run", true);
        assert!(!is_visible(&build, Phase::What));
        assert!(!is_visible(&build, Phase::How));
        assert!(is_visible(&build, Phase::Build));
    }
}
