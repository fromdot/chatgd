use serde::Deserialize;

pub mod subprocess;

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub command: Vec<String>,
    pub trigger: Option<String>,
    #[serde(default)]
    pub default: bool,
}

pub fn find_backend(backends: &[BackendConfig], text: &str, is_reply_or_cmd: bool) -> Option<(BackendConfig, String)> {
    // 1. Check for explicit triggers
    for backend in backends {
        if let Some(trigger) = &backend.trigger {
            if text.contains(trigger) {
                let prompt = text.replace(trigger, "").trim().to_string();
                return Some((backend.clone(), prompt));
            }
        }
    }

    // 2. Fallback to default if applicable
    if is_reply_or_cmd {
        if let Some(default_backend) = backends.iter().find(|b| b.default) {
            let prompt = if text.starts_with("/ask") {
                text.replacen("/ask", "", 1).trim().to_string()
            } else {
                text.trim().to_string()
            };
            return Some((default_backend.clone(), prompt));
        }
    }

    None
}
