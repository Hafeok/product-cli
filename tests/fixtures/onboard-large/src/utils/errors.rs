use serde_json;

// CONVENTION: errors map to HTTP status codes via IntoResponse
// DO NOT USE panic or unwrap in production code
pub enum AppError {
    NotFound,
    Internal(String),
}
