//! Clipboard daemon binary (`rs-shell-clipd`).

use anyhow::Result;
use rs_shell_config::ClipboardConfig;
use rs_shell_core::bus::EventBus;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = ClipboardConfig::default();
    let bus = EventBus::new();

    tracing::info!("rs-shell-clipd starting");
    rs_shell_clipboard::run(bus, config).await
}
