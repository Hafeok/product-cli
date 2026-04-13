// API middleware
// Convention: all handlers return Result<Json<T>, AppError>
use serde_json;
use crate::error::AppError;

pub fn auth_middleware() {
    // All requests must pass through this middleware
    println!("auth check");
}
