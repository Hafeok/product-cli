use serde_json;

// CONSTRAINT: config values come from environment, never hardcoded
// NEVER USE hardcoded secrets or connection strings
pub struct Settings {
    pub database_url: String,
    pub port: u16,
}
