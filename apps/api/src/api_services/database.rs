use qryvanta_core::AppError;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .map_err(|error| AppError::Internal(format!("failed to connect to database: {error}")))?;

    sqlx::migrate!("../../crates/infrastructure/migrations")
        .run(&pool)
        .await
        .map_err(|error| AppError::Internal(format!("failed to run migrations: {error}")))?;

    Ok(pool)
}
