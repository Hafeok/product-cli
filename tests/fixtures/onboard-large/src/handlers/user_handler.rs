use tracing;
use serde_json;

// CONVENTION: handlers log at info level on entry
pub async fn get_user() {
    tracing::info!("get_user called");
}
