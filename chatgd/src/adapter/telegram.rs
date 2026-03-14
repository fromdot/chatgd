use crate::backend::{parse_subcommand, subprocess, BackendConfig};
use crate::session::{format_context, LogEntry, SessionManager};
use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;
use tracing::{error, info};

pub struct TelegramConfig {
    pub token: String,
    pub bot_username: String,
    pub allowed_users: Vec<u64>,
    pub backends: Vec<BackendConfig>,
}

pub async fn start(bot: Bot, config: TelegramConfig, session_manager: Arc<SessionManager>) -> Result<()> {
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

    // 3. Check for bot mention
    let mention_str = format!("@{}", cfg.bot_username);
    if !text.contains(&mention_str) {
        return; // Just logged, no action needed
    }

    let raw_text = text.replace(&mention_str, "").trim().to_string();

    // 4. Handle built-in commands
    if raw_text == "/reset" {
        let session_dir = sm.ensure_session_dir(chat_id, None);
        // Only removing subdirectories (backends) to keep log.jsonl
        if let Ok(entries) = std::fs::read_dir(&session_dir) {
            for entry in entries.flatten() {
                if let Ok(ft) = entry.file_type() {
                    if ft.is_dir() {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }
        let _ = bot.send_message(msg.chat.id, "세션 초기화 완료").await;
        return;
    }

    if raw_text == "/status" {
        let log_count = sm.collect_context(chat_id, 99999).len(); // Unprocessed msgs
        let _ = bot.send_message(msg.chat.id, format!("현재 미처리 메시지 수: {}", log_count)).await;
        return;
    }

    // 5. Parse backend subcommand
    let (backend_opt, prompt) = parse_subcommand(&raw_text, &cfg.backends);
    let backend = match backend_opt {
        Some(b) => b,
        None => match BackendConfig::default_backend(&cfg.backends) {
            Some(b) => b.clone(),
            None => {
                let _ = bot.send_message(msg.chat.id, "기본 백엔드가 설정되어 있지 않습니다.").await;
                return;
            }
        }
    };

    // 6. Collect context & format prompt
    let context_entries = sm.collect_context(chat_id, 50);
    let full_prompt = format_context(&context_entries, &prompt);

    // 7. Execute subprocess
    let session_dir = sm.ensure_session_dir(chat_id, Some(&backend.name));
    
    let response_text = match subprocess::execute(&backend, &full_prompt, &session_dir).await {
        Ok(out) => if out.is_empty() { "[No output]".to_string() } else { out },
        Err(e) => format!("Error: {}", e),
    };

    let elapsed = start_time.elapsed().as_millis() as u64;

    // 8. Log response
    let resp_log = LogEntry {
        ts: chrono::Utc::now(),
        from: backend.name.clone(),
        uid: None,
        username: None,
        text: response_text.clone(),
        elapsed_ms: Some(elapsed),
    };
    let _ = sm.append_log(chat_id, &resp_log);

    // 9. Send response back
    let chunks = response_text.as_bytes().chunks(4000);
    for chunk in chunks {
        let s = String::from_utf8_lossy(chunk);
        if let Err(e) = bot.send_message(msg.chat.id, s.to_string()).await {
            error!("Failed to send message chunk: {}", e);
        }
    }
}
