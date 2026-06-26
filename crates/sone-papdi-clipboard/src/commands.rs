use crate::db::{Database, NewClipEntry};
use crate::protocol::WaylandCmd;
use anyhow::{Context, Result};
use sone_papdi_config::ClipboardConfig;
use sone_papdi_core::bus::EventBus;
use sone_papdi_core::events::{ClipboardEvent, ShellEvent};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

/// Clipboard command dispatcher.
///
/// Coordinates between IPC command handlers, the SQLite history store,
#[derive(Clone)]
pub struct ClipboardCommander {
    db: Arc<Mutex<Database>>,
    bus: EventBus,
    config: ClipboardConfig,
    /// Channel to send clipboard-set commands to the Wayland thread.
    wayland_tx: mpsc::UnboundedSender<WaylandCmd>,
}

impl ClipboardCommander {
    /// Create a new command dispatcher.
    pub fn new(
        db: Arc<Mutex<Database>>,
        bus: EventBus,
        config: ClipboardConfig,
        wayland_tx: mpsc::UnboundedSender<WaylandCmd>,
    ) -> Self {
        Self {
            db,
            bus,
            config,
            wayland_tx,
        }
    }

    /// Store a new clipboard payload and emit a `Stored` event.
    pub async fn store(&self, mime: String, content: Vec<u8>) -> Result<i64> {
        if content.len() > self.config.max_image_bytes {
            anyhow::bail!("clipboard payload exceeds maximum size");
        }

        let (kind, preview) = crate::mime::classify(&mime, &content);
        let content_str = String::from_utf8_lossy(&content).into_owned();
        let hash = crate::hash::content_hash(&content);

        let entry = NewClipEntry {
            content_hash: hash,
            mime: mime.clone(),
            kind: kind.clone(),
            preview: preview.clone(),
            content: Some(content_str),
            size_bytes: content.len() as i64,
        };

        let db = self.db.lock().await;
        let id = db.upsert_entry(&entry)?;
        drop(db);

        self.bus.publish(ShellEvent::Clipboard(ClipboardEvent::Stored {
            id,
            mime,
            kind,
            preview,
        }));

        Ok(id)
    }

    /// Copy an entry to the Wayland clipboard and emit a `Copied` event.
    pub async fn copy(&self, id: i64) -> Result<()> {
        let db = self.db.lock().await;
        let entry = db
            .get_entry(id)?
            .with_context(|| format!("clipboard entry {id} not found"))?;
        let content = entry.content.unwrap_or_default();
        db.touch_used(id)?;
        drop(db);

        // Send the clipboard data to the Wayland thread for native wlr-data-control.
        self.wayland_tx
            .send(WaylandCmd::SetClipboard {
                mime: entry.mime.clone(),
                content: content.as_bytes().to_vec(),
            })
            .context("failed to send clipboard command to Wayland thread")?;

        self.bus.publish(ShellEvent::Clipboard(ClipboardEvent::Copied { id }));
        Ok(())
    }

    /// Delete an entry and emit a `Deleted` event.
    pub async fn delete(&self, id: i64) -> Result<()> {
        let db = self.db.lock().await;
        db.delete_entry(id)?;
        drop(db);

        self.bus.publish(ShellEvent::Clipboard(ClipboardEvent::Deleted { id }));
        Ok(())
    }

    /// List entries.
    pub async fn list(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<sone_papdi_core::events::ClipEntry>> {
        let limit = limit.unwrap_or(self.config.max_entries as usize);
        let db = self.db.lock().await;
        let results = db.list_entries("", limit, 0)?;
        Ok(results)
    }

    /// Search entries and emit a `SearchResult` event.
    pub async fn search(
        &self,
        query: String,
        limit: Option<usize>,
    ) -> Result<Vec<sone_papdi_core::events::ClipEntry>> {
        let limit = limit.unwrap_or(self.config.max_entries as usize);
        let db = self.db.lock().await;
        let results = db.list_entries(&query, limit, 0)?;

        self.bus
            .publish(ShellEvent::Clipboard(ClipboardEvent::SearchResult {
                query: query.clone(),
                results: results.clone(),
            }));

        Ok(results)
    }

    /// Clear all non-pinned history and emit a `Cleared` event.
    pub async fn clear(&self) -> Result<()> {
        let db = self.db.lock().await;
        db.delete_unpinned_older_than_days(0)?;
        drop(db);

        self.bus.publish(ShellEvent::Clipboard(ClipboardEvent::Cleared));
        Ok(())
    }

    /// Toggle pinned state and emit a `PinnedToggled` event.
    pub async fn set_pinned(&self, id: i64, pinned: bool) -> Result<()> {
        let db = self.db.lock().await;
        db.set_pinned(id, pinned)?;
        drop(db);

        self.bus
            .publish(ShellEvent::Clipboard(ClipboardEvent::PinnedToggled {
                id,
                pinned,
            }));
        Ok(())
    }
}
