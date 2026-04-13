// Repository layer — database access for orders
use sqlx;

pub async fn find_order(pool: &str, id: u64) -> Result<String, String> {
    Ok(format!("order {}", id))
}
