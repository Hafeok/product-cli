use serde_json;

// CONVENTION: API versions are prefixed with /v1, /v2 etc.
// MUST NOT expose unversioned endpoints in production
pub const API_VERSION: &str = "v1";
