use anyhow::Context;
use redis::Client;

pub struct Redis {
    pub connection: redis::aio::MultiplexedConnection,
}

impl Redis {
    pub async fn connect(redis_url: &str) -> anyhow::Result<Self> {
        let client = Client::open(redis_url).context("Failed to create Redis client")?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to get Redis async connection")?;

        Ok(Self { connection })
    }

    pub async fn ping(&mut self) -> anyhow::Result<()> {
        redis::cmd("PING")
            .query_async(&mut self.connection)
            .await
            .context("Redis ping failed")
    }
}
