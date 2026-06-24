//! rs-shell-clipd — Clipboard engine daemon.
//!
//! A standalone binary that:
//! 1. Opens a Wayland connection and binds `wlr-data-control-unstable-v1`
//! 2. Monitors clipboard changes natively (no wl-paste subprocess)
//! 3. Stores history in SQLite
//! 4. Listens on the IPC socket for daemon commands
//! 5. Acts as the persistent clipboard data owner

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("rs-shell-clipd starting");

    // Load config or fall back to defaults.
    let config = if rs_shell_config::config_path().exists() {
        rs_shell_config::load_config(&rs_shell_config::config_path()).await?
    } else {
        tracing::warn!("config not found, using defaults");
        rs_shell_config::ShellConfig::default()
    };

    let bus = rs_shell_core::bus::EventBus::new();

    rs_shell_clipboard::run(&bus, &config.clipboard).await
}
