//! MCP HTTP transport — Streamable HTTP via axum (ADR-020)
//!
//! Hosts both the MCP JSON-RPC endpoint (`POST /mcp`) and the live What-graph
//! view (`GET /`, `/api/graph`, `/api/events`) on one server, so a team's
//! browser is a live window into the same session an agent drives over MCP.

use std::path::PathBuf;
use std::sync::Arc;

use product_core::error::{ProductError, Result};

use super::registry::ToolRegistry;
use super::watch::ChangeTx;
use super::{JsonRpcRequest, JsonRpcResponse};

/// Shared server state — the tool registry plus the repo/session scope the
/// view routes (`http_view`) resolve against.
pub(crate) struct AppState {
    registry: ToolRegistry,
    token: Option<String>,
    pub(crate) repo_root: PathBuf,
    pub(crate) changes: ChangeTx,
    workflow: bool,
    /// The session this server is scoped to (`--session`); the view and the
    /// default MCP session follow it. `None` = unscoped (canonical / scan).
    pub(crate) session: Option<String>,
}

/// Run MCP server over HTTP
#[allow(clippy::too_many_arguments)]
pub async fn run_http(
    repo_root: PathBuf,
    write_enabled: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    cors_origins: Vec<String>,
    workflow: bool,
    session: Option<String>,
) -> Result<()> {
    use axum::{Router, routing::{get, post}};

    let changes = super::watch::spawn(repo_root.join(".product"));
    let state = Arc::new(AppState {
        registry: ToolRegistry::new(repo_root.clone(), write_enabled),
        token,
        repo_root,
        changes,
        workflow,
        session,
    });

    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .route("/legacy", get(legacy_view_handler))
        .route("/api/graph", get(graph_handler))
        .route("/api/pf", get(pf_handler))
        .route("/api/session", get(session_handler))
        .route("/api/events", get(events_handler))
        // The 1.7.0 explorer UI at `/`, plus every embedded asset it references
        // (data*.js, *.jsx, _ds/**, vendor/**, assets/**) via the fallback.
        .fallback(get(ui_handler))
        .with_state(state.clone());

    let app = with_cors(app, &cors_origins);

    let addr = format!("{}:{}", bind, port);
    eprintln!("Product MCP HTTP server listening on {}", addr);
    eprintln!("  Live view: http://{}/", addr);
    if state.token.is_some() {
        eprintln!("  Authentication: bearer token required (MCP endpoint)");
    } else {
        eprintln!("  Warning: no authentication configured (--token not set)");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        ProductError::IoError(format!("Failed to bind {}: {}", addr, e))
    })?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ProductError::IoError(format!("Server error: {}", e)))?;

    Ok(())
}

/// `POST /mcp` — the JSON-RPC MCP endpoint (bearer-auth when a token is set).
async fn mcp_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::Json(request): axum::Json<JsonRpcRequest>,
) -> (axum::http::StatusCode, axum::Json<JsonRpcResponse>) {
    use axum::{http::StatusCode, Json};
    if let Some(ref expected) = state.token {
        let auth = headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if auth != Some(expected.as_str()) {
            return (StatusCode::UNAUTHORIZED, Json(JsonRpcResponse::error(request.id, -32000, "Unauthorized")));
        }
    }
    // Workflow mode: pick the session from the Mcp-Session-Id header. Without a
    // header (or when workflow is off) fall back to the flat stateless surface.
    // Per-request push notifications are not delivered on a POST response; the
    // advance result carries the now-available tool list instead.
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| state.session.clone());
    if let (true, Some(id)) = (state.workflow, session_id) {
        let ctx = super::workflow::WorkflowCtx::resolve(&state.repo_root, &id);
        let out = state.registry.handle_jsonrpc_workflow(&request, &ctx);
        return match out.response {
            Some(response) => (StatusCode::OK, Json(response)),
            None => (StatusCode::ACCEPTED, Json(JsonRpcResponse::success(None, serde_json::json!(null)))),
        };
    }
    match state.registry.handle_jsonrpc(&request) {
        Some(response) => (StatusCode::OK, Json(response)),
        None => (StatusCode::ACCEPTED, Json(JsonRpcResponse::success(None, serde_json::json!(null)))),
    }
}

use super::http_ui::{legacy_view_handler, ui_handler};
use super::http_view::{events_handler, graph_handler, pf_handler, session_handler};

fn with_cors(app: axum::Router, cors_origins: &[String]) -> axum::Router {
    if cors_origins.is_empty() {
        return app;
    }
    use axum::http::Method;
    use tower_http::cors::{AllowOrigin, CorsLayer};
    let origins: Vec<_> = cors_origins.iter().filter_map(|o| o.parse().ok()).collect();
    app.layer(
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE]),
    )
}

/// Wait for SIGTERM or SIGINT to trigger graceful shutdown
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.ok();
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            sig.recv().await;
        } else {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    eprintln!("Shutdown signal received, draining in-flight requests...");
}
