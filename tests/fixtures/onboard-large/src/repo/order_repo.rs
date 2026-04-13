use sqlx;
use uuid;

// CONSTRAINT: repos return domain types, never raw sqlx::Row
// BY CONVENTION repos are the only layer that imports sqlx
pub struct OrderRepo {
    pool: sqlx::PgPool,
}
