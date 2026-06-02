// API routes
// Convention: all handlers return Result<Json<T>, AppError>
use serde_json;
use crate::error::AppError;

pub fn setup_routes() {
    // CONSTRAINT: all routes must go through the auth middleware
    println!("routes set up");
}
