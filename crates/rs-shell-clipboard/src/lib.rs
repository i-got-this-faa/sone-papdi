//! Clipboard history daemon for sone-papdi.
//!
//! Watches `wl-paste --watch`, stores clipboard entries in SQLite, and emits
//! typed `ShellEvent::Clipboard` events on the shared event bus.

use anyhow::{Context, Result};
use rs_shell_config::ClipboardConfig;
use rs_shell_core::bus::EventBus;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod commands;
pub mod db;
pub mod mime;
pub mod watcher;

pub use db::{Database, NewClipEntry};

/// Return the default clipboard database path.
pub fn database_path() -> PathBuf {
    rs_shell_core::paths::data_dir().join("clipboard.db")
}

/// Run the clipboard service.
pub async fn run(bus: EventBus, config: ClipboardConfig) -> Result<()> {
    let path = database_path();
    let db = Database::open(&path)
        .with_context(|| format!("opening clipboard database at {}", path.display()))?;
    let db = Arc::new(Mutex::new(db));

    tracing::info!("clipboard database opened at {}", path.display());

    // Spawn cleanup task.
    {
        let db = db.clone();
        let days = config.cleanup_unpinned_after_days;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
            loop {
                interval.tick().await;
                if let Err(e) = db.lock().await.delete_unpinned_older_than_days(days) {
                    tracing::warn!("clipboard cleanup failed: {e}");
                }
            }
        });
    }

    // Start wl-paste watcher if available.
    watcher::run_watcher(db.clone(), bus.clone(), config.clone()).await
}
