use serde_json;
use uuid;

// CONVENTION: all models derive Serialize and Deserialize
// MUST NOT store prices as floating point
pub struct Order {
    pub id: String,
    pub total_cents: i64,
}
