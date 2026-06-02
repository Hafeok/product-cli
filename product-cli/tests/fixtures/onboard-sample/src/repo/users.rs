// Repository layer — sole owner of database access
use sqlx;

pub async fn find_by_id(pool: &str, id: u64) -> Result<String, String> {
    Ok(format!("user {}", id))
}

pub async fn list_all(pool: &str) -> Result<Vec<String>, String> {
    Ok(vec!["user1".to_string()])
}
