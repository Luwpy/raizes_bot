mod config;
mod database;
mod redis;

use anyhow::{Context, Result};
use sqlx::PgPool;

pub struct Infrastructure {
    database: database::Database,
    redis: redis::Redis,
}

impl Infrastructure {
    pub async fn connect() -> Result<Self> {
        let config = config::InfraConfig::load()?;

        let database = database::Database::connect(&config.database_url).await?;

        // If Redis initialization fails, database is dropped automatically.
        let redis = redis::Redis::connect(&config.redis_url).await?;

        Ok(Self { database, redis })
    }

    pub fn database(&self) -> &PgPool {
        self.database.pool()
    }

    pub async fn health_check(&mut self) -> Result<()> {
        sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(self.database.pool())
            .await
            .context("PostgreSQL health check failed")?;

        self.redis.ping().await?;

        Ok(())
    }

    pub async fn shutdown(self) {
        let Self { database, redis } = self;

        database.close().await;
        drop(redis);
    }
}
