use sqlx;

// CONSTRAINT: migrations are forward-only, never delete a migration file
// MUST NOT use DROP TABLE in migrations without approval
pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!().run(pool).await
}
