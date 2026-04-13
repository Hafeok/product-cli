use sqlx;
use uuid;

// CONSTRAINT: repos return domain types, never raw sqlx::Row
pub struct UserRepo {
    pool: sqlx::PgPool,
}
