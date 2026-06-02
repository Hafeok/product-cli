use serde_json;
use axum::Json;

// CONVENTION: health endpoints return JSON with {"status": "ok"}
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}
