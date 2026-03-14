use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub from: String,
    pub uid: Option<u64>,
    pub username: Option<String>,
    pub text: String,
    pub elapsed_ms: Option<u64>,
}

pub struct SessionManager {
    base_dir: PathBuf,
}

impl SessionManager {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir).ok();
        Self { base_dir }
    }

    pub fn ensure_session_dir(&self, chat_id: i64, backend: Option<&str>) -> PathBuf {
        let mut dir = self.base_dir.join(format!("chat_{}", chat_id));
        if let Some(b) = backend {
            dir = dir.join(b);
        }
        fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn append_log(&self, chat_id: i64, entry: &LogEntry) -> Result<()> {
        let dir = self.ensure_session_dir(chat_id, None);
        let log_file = dir.join("log.jsonl");
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
            
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;
        
        Ok(())
    }

    pub fn collect_context(&self, chat_id: i64, max_messages: usize) -> Vec<LogEntry> {
        let dir = self.ensure_session_dir(chat_id, None);
        let log_file = dir.join("log.jsonl");
        
        let Ok(file) = std::fs::File::open(log_file) else {
            return vec![];
        };
        
        let reader = BufReader::new(file);
        let mut all_entries = Vec::new();
        
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                all_entries.push(entry);
            }
        }
        
        let mut context_entries = Vec::new();
        for entry in all_entries.into_iter().rev() {
            if entry.from != "user" {
                break;
            }
            context_entries.push(entry);
        }
        context_entries.reverse();
        
        let skip = context_entries.len().saturating_sub(max_messages);
        context_entries.into_iter().skip(skip).collect()
    }
}

pub fn format_context(entries: &[LogEntry], prompt: &str) -> String {
    if entries.is_empty() {
        return prompt.to_string();
    }
    
    let mut out = String::from("[채팅 맥락]\n");
    for e in entries {
        let name = e.username.as_deref().unwrap_or("unknown");
        out.push_str(&format!("user({}): {}\n", name, e.text));
    }
    out.push_str(&format!("\n[요청]\n{}", prompt));
    out
}
