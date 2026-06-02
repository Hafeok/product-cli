//! MCP HTTP transport — Streamable HTTP via axum (ADR-020)

use crate::error::{ProductError, Result};
use std::path::PathBuf;

use super::registry::ToolRegistry;
use super::{JsonRpcRequest, JsonRpcResponse};

/// Run MCP server over HTTP
pub async fn run_http(
    repo_root: PathBuf,
    write_enabled: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    cors_origins: Vec<String>,
) -> Result<()> {
    use axum::{Router, routing::post, http::{StatusCode, HeaderMap}, Json};
    use std::sync::Arc;

    struct AppState {
        registry: ToolRegistry,
        token: Option<String>,
    }

    let state = Arc::new(AppState {
        registry: ToolRegistry::new(repo_root, write_enabled),
        token,
    });

    let app = Router::new()
        .route("/mcp", post({
            let state = state.clone();
            move |headers: HeaderMap, Json(request): Json<JsonRpcRequest>| {
                let state = state.clone();
                async move {
                    // Auth check
                    if let Some(ref expected) = state.token {
                        let auth = headers.get("authorization")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.strip_prefix("Bearer "));
                        match auth {
                            Some(provided) if provided == expected.as_str() => {}
                            _ => {
                                return (StatusCode::UNAUTHORIZED, Json(JsonRpcResponse::error(
                                    request.id, -32000, "Unauthorized"
                                )));
                            }
                        }
                    }

                    // Notifications return None — respond with 202 Accepted (no body needed but type requires one)
                    match state.registry.handle_jsonrpc(&request) {
                        Some(response) => (StatusCode::OK, Json(response)),
                        None => (StatusCode::ACCEPTED, Json(JsonRpcResponse::success(None, serde_json::json!(null)))),
                    }
                }
            }
        }));

    // Add CORS if configured
    let app = if !cors_origins.is_empty() {
        use tower_http::cors::{CorsLayer, AllowOrigin};
        use axum::http::Method;
        let origins: Vec<_> = cors_origins.iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        app.layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([Method::POST, Method::OPTIONS])
                .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE]),
        )
    } else {
        app
    };

    let addr = format!("{}:{}", bind, port);
    eprintln!("Product MCP HTTP server listening on {}", addr);
    if state.token.is_some() {
        eprintln!("  Authentication: bearer token required");
    } else {
        eprintln!("  Warning: no authentication configured (--token not set)");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        ProductError::IoError(format!("Failed to bind {}: {}", addr, e))
    })?;

    // Graceful shutdown: listen for SIGTERM/SIGINT, complete in-flight requests
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            ProductError::IoError(format!("Server error: {}", e))
        })?;

    Ok(())
}

/// Wait for SIGTERM or SIGINT to trigger graceful shutdown
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .ok();
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
