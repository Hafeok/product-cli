// Error handling module
// Convention: all errors use this unified error type

pub struct AppError {
    pub message: String,
    pub code: u16,
}

impl AppError {
    pub fn new(message: &str, code: u16) -> Self {
        AppError { message: message.to_string(), code }
    }
}
