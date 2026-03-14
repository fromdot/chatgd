use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
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

    pub fn ensure_session_dir(&self, chat_id: i64) -> PathBuf {
        let dir = self.base_dir.join(format!("chat_{}", chat_id));
        fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn append_log(&self, chat_id: i64, entry: &LogEntry) -> Result<()> {
        let dir = self.ensure_session_dir(chat_id);
        let log_file = dir.join("log.jsonl");
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
            
        let json = serde_json::to_string(entry)?;
        writeln!(file, "{}", json)?;
        
        Ok(())
    }
}
