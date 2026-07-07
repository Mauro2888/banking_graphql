use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use crate::config::DatabaseConfig;

pub async fn create_pg_pool(cfg: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect(&cfg.url)
        .await
}