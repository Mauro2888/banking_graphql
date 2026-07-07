use std::env;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, String> {
        let url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL env variable is missing".to_string())?;

        let max_connections = env::var("MAX_CONNECTIONS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);

        Ok(Self { url, max_connections })
    }
}