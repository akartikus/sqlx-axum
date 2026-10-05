use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
        })
    }
}
