// API handler module
// Convention: all handlers return Result<Json<T>, AppError>
use serde_json;
use crate::error::AppError;

pub fn get_users() -> Result<String, AppError> {
    Ok("users".to_string())
}

pub fn get_user(id: u64) -> Result<String, AppError> {
    Ok(format!("user {}", id))
}
