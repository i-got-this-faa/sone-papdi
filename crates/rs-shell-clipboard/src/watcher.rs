use crate::commands::ClipboardCommander;
use crate::db::Database;
use anyhow::Result;
use rs_shell_config::ClipboardConfig;
use rs_shell_core::bus::EventBus;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

/// Poll the Wayland clipboard and store changes.
pub async fn run_watcher(
    db: Arc<Mutex<Database>>,
    bus: EventBus,
    config: ClipboardConfig,
) -> Result<()> {
    let commander = ClipboardCommander::new(db, bus, config);
    let mut last_hash: Option<String> = None;
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(250));

    loop {
        interval.tick().await;

        match poll_clipboard().await {
            Ok((mime, content)) => {
                if content.is_empty() {
                    continue;
                }
                let hash = blake3::hash(&content).to_string();
                if last_hash.as_ref() == Some(&hash) {
                    continue;
                }
                last_hash = Some(hash);

                if let Err(e) = commander.store(mime, content).await {
                    tracing::warn!("failed to store clipboard entry: {e}");
                }
            }
            Err(e) => {
                tracing::debug!("clipboard poll failed: {e}");
            }
        }
    }
}

async fn poll_clipboard() -> Result<(String, Vec<u8>)> {
    // Try text first.
    let output = Command::new("wl-paste")
        .args(["--type", "text/plain", "-n"])
        .output()
        .await?;

    if output.status.success() && !output.stdout.is_empty() {
        return Ok(("text/plain".into(), output.stdout));
    }

    // Fall back to any available MIME type.
    let output = Command::new("wl-paste")
        .arg("-n")
        .output()
        .await?;

    if output.status.success() {
        // wl-paste doesn't tell us the MIME type; infer from content.
        let mime = infer::get(&output.stdout)
            .map(|k| k.mime_type().to_string())
            .unwrap_or_else(|| "application/octet-stream".into());
        return Ok((mime, output.stdout));
    }

    anyhow::bail!("wl-paste failed: {}", String::from_utf8_lossy(&output.stderr))
}
