use tracing;
use serde_json;

// CONVENTION: handlers log at info level on entry
// MUST NOT expose admin endpoints without role check
pub async fn list_users() {
    tracing::info!("list_users called");
}
