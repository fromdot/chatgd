use crate::backend::{find_backend, subprocess, BackendConfig};
use crate::session::{LogEntry, SessionManager};
use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;
use tracing::{error, info};

pub struct TelegramConfig {
    pub token: String,
    pub allowed_users: Vec<u64>,
    pub backends: Vec<BackendConfig>,
}

pub async fn start(config: TelegramConfig, session_manager: Arc<SessionManager>) -> Result<()> {
    let bot = Bot::new(&config.token);
    let config = Arc::new(config);

    let handler = Update::filter_message().endpoint(
        |bot: Bot, msg: Message, cfg: Arc<TelegramConfig>, sm: Arc<SessionManager>| async move {
            handle_message(bot, msg, cfg, sm).await;
            respond(())
        },
    );

    info!("Starting Telegram polling...");
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![config, session_manager])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn handle_message(
    bot: Bot,
    msg: Message,
    cfg: Arc<TelegramConfig>,
    sm: Arc<SessionManager>,
) {
    let user = match msg.from() {
        Some(u) => u,
        None => return,
    };

    // 1. Check allowed users
    if !cfg.allowed_users.is_empty() && !cfg.allowed_users.contains(&user.id.0) {
        return;
    }

    let text = match msg.text() {
        Some(t) => t,
        None => return,
    };

    let chat_id = msg.chat.id.0;
    
    // 2. Log incoming message
    let start_time = std::time::Instant::now();
    let log_entry = LogEntry {
        ts: chrono::Utc::now(),
        from: "user".to_string(),
        uid: Some(user.id.0),
        username: user.username.clone(),
        text: text.to_string(),
        elapsed_ms: None,
    };
    
    if let Err(e) = sm.append_log(chat_id, &log_entry) {
        error!("Failed to log user message: {}", e);
    }

    // 3. Find backend
    let is_reply_to_bot = msg.reply_to_message().map_or(false, |m| {
        m.from().map_or(false, |u| u.is_bot)
    });
    let is_cmd = text.starts_with("/ask");

    if let Some((backend, prompt)) = find_backend(&cfg.backends, text, is_reply_to_bot || is_cmd) {
        let session_dir = sm.ensure_session_dir(chat_id);
        
        // 4. Execute subprocess
        let response_text = match subprocess::execute(&backend, &prompt, &session_dir).await {
            Ok(out) => if out.is_empty() { "[No output]".to_string() } else { out },
            Err(e) => format!("Error: {}", e),
        };

        let elapsed = start_time.elapsed().as_millis() as u64;

        // Log response
        let resp_log = LogEntry {
            ts: chrono::Utc::now(),
            from: backend.name.clone(),
            uid: None,
            username: None,
            text: response_text.clone(),
            elapsed_ms: Some(elapsed),
        };
        let _ = sm.append_log(chat_id, &resp_log);

        // 5. Send response back (split if > 4096 chars)
        let chunks = response_text.as_bytes().chunks(4000); // safe margin
        for chunk in chunks {
            let s = String::from_utf8_lossy(chunk);
            if let Err(e) = bot.send_message(msg.chat.id, s.to_string()).await {
                error!("Failed to send message chunk: {}", e);
            }
        }
    }
}
