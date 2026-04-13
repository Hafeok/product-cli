use serde_json;
use uuid;

// CONVENTION: tokens use RS256 signing, never HS256 in production
// DO NOT USE symmetric keys for token signing
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}
