use anyhow::Context;
use std::env;

pub struct InfraConfig {
    pub database_url: String,
    pub redis_url: String,
}

impl InfraConfig {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL not set")?,
            redis_url: env::var("REDIS_URL").context("REDIS_URL not set")?,
        })
    }
}
