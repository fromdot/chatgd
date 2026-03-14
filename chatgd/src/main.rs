mod adapter;
mod backend;
mod session;

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::sync::Arc;
use teloxide::prelude::*;
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

    let token = env::var("CHATGD_TELEGRAM_TOKEN").unwrap_or(config.telegram.token);
    
    let bot = Bot::new(&token);
    let me = bot.get_me().await.context("Failed to get bot info")?;
    let bot_username = me.username.clone().expect("Bot must have a username");
    info!("Bot username: @{}", bot_username);

    let tg_config = adapter::telegram::TelegramConfig {
        token,
        bot_username,
        allowed_users: config.security.allowed_users,
        backends: config.backends,
    };

    adapter::telegram::start(bot, tg_config, session_manager).await?;

    Ok(())
}
