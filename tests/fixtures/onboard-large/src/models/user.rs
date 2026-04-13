use serde_json;
use uuid;

// CONVENTION: all models derive Serialize and Deserialize
pub struct User {
    pub id: String,
    pub email: String,
}
