use anyhow::{Context, Result};
use pacesetter::config::DatabaseConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub mod entities;

pub async fn connect_pool(config: DatabaseConfig) -> Result<PgPool, anyhow::Error> {
    let pool = PgPoolOptions::new()
        .connect(config.url.as_str())
        .await
        .context("Failed to connect to database")?;

    Ok(pool)
}

#[cfg(feature = "test-helpers")]
pub mod test_helpers;
