use super::BackendConfig;
use anyhow::{Context, Result};
use std::path::Path;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{info, warn};

pub async fn execute(backend: &BackendConfig, prompt: &str, session_dir: &Path) -> Result<String> {
    if backend.command.is_empty() {
        return Err(anyhow::anyhow!("Command array is empty for backend {}", backend.name));
    }

    let mut args = Vec::new();
    for arg in &backend.command {
        args.push(arg.replace("{prompt}", prompt));
    }

    let program = &args[0];
    let cmd_args = &args[1..];

    info!("Executing backend '{}' in {:?}", backend.name, session_dir);

    let child = Command::new(program)
        .args(cmd_args)
        .current_dir(session_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn subprocess")?;

    match timeout(Duration::from_secs(120), child.wait_with_output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                let err = String::from_utf8_lossy(&output.stderr);
                warn!("Subprocess failed: {}", err);
                Err(anyhow::anyhow!("Subprocess error: {}", err))
            }
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("Failed to read output: {}", e)),
        Err(_) => Err(anyhow::anyhow!("Subprocess timed out after 120s")),
    }
}
