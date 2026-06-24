//! MCP HTTP transport — Streamable HTTP via axum (ADR-020)
//!
//! Hosts both the MCP JSON-RPC endpoint (`POST /mcp`) and the live What-graph
//! view (`GET /`, `/api/graph`, `/api/events`) on one server, so a team's
//! browser is a live window into the same session an agent drives over MCP.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use product_core::author::domain::session_dir;
use product_core::config::ProductConfig;
use product_core::error::{ProductError, Result};
use product_core::pf::session::DomainSession;
use product_core::pf::viz::{to_view_graph, ViewGraph};

use super::registry::ToolRegistry;
use super::watch::ChangeTx;
use super::{JsonRpcRequest, JsonRpcResponse};

struct AppState {
    registry: ToolRegistry,
    token: Option<String>,
    repo_root: PathBuf,
    changes: ChangeTx,
}

/// Run MCP server over HTTP
pub async fn run_http(
    repo_root: PathBuf,
    write_enabled: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    cors_origins: Vec<String>,
) -> Result<()> {
    use axum::{Router, routing::{get, post}};

    let changes = super::watch::spawn(repo_root.join(".product"));
    let state = Arc::new(AppState {
        registry: ToolRegistry::new(repo_root.clone(), write_enabled),
        token,
        repo_root,
        changes,
    });

    let app = Router::new()
        .route("/mcp", post(mcp_handler))
        .route("/", get(index_handler))
        .route("/api/graph", get(graph_handler))
        .route("/api/events", get(events_handler))
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
    match state.registry.handle_jsonrpc(&request) {
        Some(response) => (StatusCode::OK, Json(response)),
        None => (StatusCode::ACCEPTED, Json(JsonRpcResponse::success(None, serde_json::json!(null)))),
    }
}

/// `GET /` — the embedded two-lane view page (no build step, no CDN).
async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("assets/view.html"))
}

/// `GET /api/graph` — the What graph projected to `{nodes, edges, contexts}`,
/// rebuilt from `.product/` on every request (the view is always derived).
async fn graph_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> std::result::Result<axum::Json<ViewGraph>, (axum::http::StatusCode, String)> {
    load_view(&state.repo_root)
        .map(axum::Json)
        .map_err(|e| (axum::http::StatusCode::NOT_FOUND, e))
}

/// `GET /api/events` — an SSE stream that ticks whenever a `.product/` file
/// changes, so the browser re-fetches `/api/graph`.
async fn events_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::response::Sse<impl tokio_stream::Stream<Item = std::result::Result<axum::response::sse::Event, std::convert::Infallible>>> {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use tokio_stream::{wrappers::BroadcastStream, StreamExt};

    let stream = BroadcastStream::new(state.changes.subscribe())
        .map(|_| Ok(Event::default().event("changed").data("changed")));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Load the active product's What graph and project it for the client.
fn load_view(repo_root: &Path) -> std::result::Result<ViewGraph, String> {
    let cfg = ProductConfig::load_from_root(repo_root).map_err(|e| e.to_string())?;
    let product = cfg.name.trim();
    if product.is_empty() {
        return Err("no product configured (set `name` in product.toml)".to_string());
    }
    let session = DomainSession::load(&session_dir(repo_root, product))
        .map_err(|_| format!("no What graph for product '{product}' yet"))?;
    Ok(to_view_graph(&session.graph))
}

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

#[cfg(test)]
mod tests {
    use super::load_view;
    use product_core::author::domain::session_dir;
    use product_core::pf::model::{Command, Entity};
    use product_core::pf::session::DomainSession;
    use product_core::pf::viz::{DOMAIN, EVENT};
    use std::path::Path;

    fn save_graph(root: &Path, entities: &[&str]) {
        let mut s = DomainSession::start("demo", None, vec![], None, "t".into()).expect("start");
        for e in entities {
            s.graph.entities.push(Entity {
                id: (*e).into(), label: (*e).into(), context: "ctx".into(), definition: "d".into(),
                ..Default::default()
            });
        }
        s.graph.commands.push(Command {
            id: "Place".into(), label: "Place".into(), context: "ctx".into(), targets: "Order".into(), emits: vec![],
        });
        s.save(&session_dir(root, "demo")).expect("save");
    }

    /// view-derivation-verified: the projection mirrors the on-disk What graph,
    /// and a fresh load after a disk change reflects it — proving the view is
    /// rebuilt per request with no cache.
    #[test]
    fn view_graph_reflects_disk_each_call() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".product")).expect("mkdir");
        std::fs::write(root.join(".product/config.toml"), "name = \"demo\"\n").expect("config");

        save_graph(root, &["Order"]);
        let v1 = load_view(root).expect("load_view");
        assert!(v1.nodes.iter().any(|n| n.id == "Order" && n.model == DOMAIN), "entity in domain lane");
        assert!(v1.nodes.iter().any(|n| n.id == "Place" && n.model == EVENT), "command in event lane");

        save_graph(root, &["Order", "Item"]);
        let v2 = load_view(root).expect("reload");
        assert!(v2.nodes.iter().any(|n| n.id == "Item"), "new node appears without restart (no cache)");
    }
}
