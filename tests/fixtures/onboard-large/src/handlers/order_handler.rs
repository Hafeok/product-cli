use tracing;
use serde_json;

// CONVENTION: handlers log at info level on entry
pub async fn create_order() {
    tracing::info!("create_order called");
}
