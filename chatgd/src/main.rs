mod adapter;
mod backend;
mod session;

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize)]
struct AppConfig {
    telegram: TelegramConfigSection,
    security: SecurityConfigSection,
    backends: Vec<backend::BackendConfig>,
}

#[derive(Deserialize)]
struct TelegramConfigSection {
    token: String,
}

#[derive(Deserialize)]
struct SecurityConfigSection {
    #[serde(default)]
    allowed_users: Vec<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting chatgd...");

    let config_str = fs::read_to_string("config.toml")
        .context("Failed to read config.toml from current directory")?;
    
    let config: AppConfig = toml::from_str(&config_str)
        .context("Failed to parse config.toml")?;

    let session_manager = Arc::new(session::SessionManager::new("sessions"));

    let tg_config = adapter::telegram::TelegramConfig {
        token: config.telegram.token,
        allowed_users: config.security.allowed_users,
        backends: config.backends,
    };

    adapter::telegram::start(tg_config, session_manager).await?;

    Ok(())
}
