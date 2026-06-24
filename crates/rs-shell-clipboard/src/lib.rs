//! Clipboard engine: SQLite history, native `wlr-data-control` integration,
//! and shell event bus publishing.
//!
//! # Architecture
//!
//! Unlike clipman/cliphist/clipse which shell out to `wl-paste --watch`,
//! this engine implements the `wlr-data-control-unstable-v1` protocol
//! directly via `wayland-client` + `wayland-protocols-wlr`. This:
//!
//! - Eliminates the `wl-clipboard` package dependency
//! - Gives native control over clipboard data ownership
//! - Avoids subprocess spawning overhead and race conditions
//! - Allows the engine to act as the persistent clipboard data source
//!
//! # Flow
//!
//! ```text
//!                         ┌──────────────────────┐
//!   Wayland Compositor ── │ wlr-data-control      │
//!   (noctalia / sway /     │ (data_offer / source) │
//!    hyprland / niri)     └────────┬─────────────┘
//!                                  │
//!                  ┌───────────────▼───────────────┐
//!                  │  rs-shell-clipd                │
//!                  │  ┌─────────────────────────┐   │
//!                  │  │ wlr-data-control device │   │
//!                  │  │   → monitors clipboard  │   │
//!                  │  │   → owns clipboard data │   │
//!                  │  └─────────────────────────┘   │
//!                  │  ┌─────────────────────────┐   │
//!                  │  │ SQLite history store    │   │
//!                  │  │   → insert on copy      │   │
//!                  │  │   → query/search        │   │
//!                  │  │   → dedup/cleanup       │   │
//!                  │  └─────────────────────────┘   │
//!                  │  ┌─────────────────────────┐   │
//!                  │  │ IPC socket (daemon)     │   │
//!                  │  │   → receive commands    │   │
//!                  │  │   → publish events      │   │
//!                  │  └─────────────────────────┘   │
//!                  └──────────────┬────────────────┘
//!                                 │ /run/user/UID/rs-shell-clipboard.sock
//!                  ┌──────────────▼────────────────┐
//!                  │  rs-shelld (shell daemon)      │
//!                  │  → dispatches ClipboardCommand │
//!                  │  → relays ClipboardEvent       │
//!                  └────────────────────────────────┘
//! ```
//!
//! # Key Behaviors
//!
//! - **Monitoring**: Binds `zwlr_data_control_device_v1`, receives `data_offer`
//!   events when clipboard changes, reads content via file descriptor
//! - **Ownership**: The device becomes the clipboard data owner by offering
//!   received data back, keeping it alive after source app closes
//! - **Paste from history**: On `ClipboardCopy { id }` command,
//!   creates a `zwlr_data_control_source_v1`, offers the stored MIME types,
//!   then optionally simulates Ctrl+V via `wtype`
//! - **Dedup**: Content hashed and compared against last entry
//! - **MIME awareness**: Stores MIME type, supports text/image/file
//! - **CLIPBOARD_STATE**: Not needed — we get immediate protocol-level events

pub mod commands;
pub mod db;
pub mod hash;
pub mod mime;
pub mod protocol;

use crate::commands::ClipboardCommander;
use crate::db::Database;
use crate::protocol::{spawn_wayland_thread, WaylandEvent};
use anyhow::Result;
use rs_shell_config::ClipboardConfig;
use rs_shell_core::bus::EventBus;
use rs_shell_core::ipc::{send_message, IpcMessage, IpcResponse, ShellCommand};
use rs_shell_core::paths;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Mutex;

/// Run the clipboard engine.
///
/// Connects to the Wayland display, binds `wlr-data-control`, listens
/// on the IPC socket for commands from the shell daemon, and stores
/// clipboard history in SQLite.
///
/// Socket: `{runtime_dir}/rs-shell-clipboard.sock`
pub async fn run(bus: &EventBus, config: &ClipboardConfig) -> Result<()> {
    let db_path = paths::data_dir().join("clipboard.db");
    tracing::info!("clipboard database: {}", db_path.display());

    let database = Database::open(&db_path)?;
    let db = Arc::new(Mutex::new(database));

    // Spawn the native wlr-data-control Wayland monitoring thread.
    let (mut wayland_rx, wayland_tx) = spawn_wayland_thread()?;

    let commander = ClipboardCommander::new(
        db.clone(),
        bus.clone(),
        config.clone(),
        wayland_tx,
    );

    // Periodic cleanup of old entries.
    let cleanup_db = db.clone();
    let cleanup_config = config.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            interval.tick().await;
            let db = cleanup_db.lock().await;
            if let Err(e) =
                db.delete_unpinned_older_than_days(cleanup_config.cleanup_unpinned_after_days as u32)
            {
                tracing::warn!("clipboard cleanup failed: {e}");
            }
        }
    });

    // Listen on IPC socket for commands.
    let socket_path = paths::clipboard_socket();
    let _ = std::fs::remove_file(&socket_path);
    let listener = UnixListener::bind(&socket_path)?;
    tracing::info!("clipboard IPC socket: {}", socket_path.display());

    loop {
        tokio::select! {
            // Handle incoming clipboard content from the Wayland thread.
            Some(event) = wayland_rx.recv() => {
                match event {
                    WaylandEvent::Content { mime, data } => {
                        if let Err(e) = commander.store(mime, data).await {
                            tracing::warn!("failed to store clipboard entry: {e}");
                        }
                    }
                    WaylandEvent::Error(msg) => {
                        tracing::error!("Wayland clipboard error: {msg}");
                    }
                }
            }
            // Handle IPC connections.
            result = listener.accept() => {
                let (stream, addr) = result?;
                let cmd = commander.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, cmd).await {
                        tracing::warn!("IPC client {addr:?} error: {e}");
                    }
                });
            }
        }
    }
}

/// Handle a single IPC client connection.
async fn handle_client(
    mut stream: tokio::net::UnixStream,
    commander: ClipboardCommander,
) -> Result<()> {
    loop {
        let msg = match rs_shell_core::ipc::recv_message(&mut stream).await {
            Ok(msg) => msg,
            Err(_) => break, // Connection closed or protocol error
        };

        match msg {
            IpcMessage::Command(ShellCommand::ClipboardList { limit }) => {
                let results = commander.list(limit).await?;
                let resp = IpcResponse::OkWith(serde_json::to_value(&results)?);
                send_message(&mut stream, &IpcMessage::Response(resp)).await?;
            }
            IpcMessage::Command(ShellCommand::ClipboardCopy { id }) => {
                match commander.copy(id).await {
                    Ok(()) => {
                        send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok))
                            .await?;
                    }
                    Err(e) => {
                        let resp = IpcResponse::error(format!("{e}"));
                        send_message(&mut stream, &IpcMessage::Response(resp)).await?;
                    }
                }
            }
            IpcMessage::Command(ShellCommand::ClipboardDelete { id }) => {
                commander.delete(id).await?;
                send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok)).await?;
            }
            IpcMessage::Command(ShellCommand::ClipboardSearch { query }) => {
                let results = commander.search(query, None).await?;
                let resp = IpcResponse::OkWith(serde_json::to_value(&results)?);
                send_message(&mut stream, &IpcMessage::Response(resp)).await?;
            }
            IpcMessage::Command(ShellCommand::ClipboardClear) => {
                commander.clear().await?;
                send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok)).await?;
            }
            IpcMessage::Command(ShellCommand::ClipboardPin { id, pinned }) => {
                commander.set_pinned(id, pinned).await?;
                send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok)).await?;
            }
            IpcMessage::Ping => {
                send_message(&mut stream, &IpcMessage::Pong).await?;
            }
            _ => {
                // Ignore other commands on this socket
            }
        }
    }

    Ok(())
}
