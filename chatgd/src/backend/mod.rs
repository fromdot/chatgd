use serde::Deserialize;

pub mod subprocess;

#[derive(Debug, Clone, Deserialize)]
pub struct BackendConfig {
    pub name: String,
    pub command: Vec<String>,
    #[allow(dead_code)]
    pub trigger: Option<String>,
    #[serde(default)]
    pub default: bool,
}

impl BackendConfig {
    pub fn default_backend(backends: &[BackendConfig]) -> Option<&BackendConfig> {
        backends.iter().find(|b| b.default)
    }
}

pub fn parse_subcommand(text: &str, backends: &[BackendConfig]) -> (Option<BackendConfig>, String) {
    for backend in backends {
        let prefix = format!("/{}", backend.name);
        if text.starts_with(&prefix) {
            let prompt = text[prefix.len()..].trim().to_string();
            return (Some(backend.clone()), prompt);
        }
    }
    (None, text.to_string())
}
