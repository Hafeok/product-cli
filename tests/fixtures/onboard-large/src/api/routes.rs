use serde_json;
use axum::Router;

// CONVENTION: all route modules return a Router, never register globally
pub fn routes() -> Router {
    Router::new()
}
