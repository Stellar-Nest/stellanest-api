use anyhow::Result;
use sqlx::postgres::PgPoolOptions;

/// Create a PostgreSQL connection pool.
pub async fn connect(database_url: &str) -> Result<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;
    Ok(pool)
}
