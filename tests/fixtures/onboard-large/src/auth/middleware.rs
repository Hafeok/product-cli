use serde_json;
use tracing;

// CONVENTION: auth middleware rejects with 401, never 403 for missing tokens
// ALWAYS USE the AuthGuard extractor, not manual header parsing
pub struct AuthGuard;
