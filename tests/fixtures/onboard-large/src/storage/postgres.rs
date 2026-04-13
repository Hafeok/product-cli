use sqlx;

// CONSTRAINT: all database access goes through the Pool, never raw connections
// CONSTRAINT: queries must use parameterized statements
pub struct PgStore {
    pool: sqlx::PgPool,
}
