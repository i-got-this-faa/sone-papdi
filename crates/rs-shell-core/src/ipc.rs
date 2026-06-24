use crate::events::ShellEvent;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

/// Wire message — sent/received by all shell processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    /// Client → Daemon: subscribe to event topics.
    Subscribe { topics: Vec<String> },
    /// Client → Daemon: unsubscribe from event topics.
    Unsubscribe { topics: Vec<String> },
    /// Client → Daemon: imperative command.
    Command(ShellCommand),
    /// Client → Daemon: keepalive ping.
    Ping,

    /// Daemon → Client: event broadcast.
    Event(ShellEvent),
    /// Daemon → Client: response to a command.
    Response(IpcResponse),
    /// Daemon → Client: keepalive pong.
    Pong,
}

/// Response to a command request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    OkWith(serde_json::Value),
    Error { code: u32, message: String },
}

impl IpcResponse {
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            code: 1,
            message: message.into(),
        }
    }
}

/// Imperative commands clients can send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellCommand {
    // Clipboard
    ClipboardList { limit: Option<usize> },
    ClipboardCopy { id: i64 },
    ClipboardDelete { id: i64 },
    ClipboardSearch { query: String },
    ClipboardClear,
    ClipboardPin { id: i64, pinned: bool },

    // Launcher
    ShowLauncher,
    HideLauncher,
    LauncherSearch { query: String },

    // Audio
    SetVolume { sink: Option<String>, volume: f64 },
    ToggleMute { sink: Option<String> },

    // Wallpaper / Theme
    SetWallpaper { path: String, output: Option<String> },
    SetTheme { name: String },
    ToggleDarkMode,

    // Window
    FocusWindow { id: String },
    CloseWindow { id: String },
    MoveToWorkspace { window_id: String, workspace: String },
    SwitchWorkspace { workspace: String },

    // Shell
    ShowNotificationCenter,
    LockScreen,
    Suspend,
    Logout,
    Reload,
}

/// Send a length-prefixed JSON message over a Unix stream.
pub async fn send_message(stream: &mut UnixStream, msg: &IpcMessage) -> Result<()> {
    let payload = serde_json::to_vec(msg)?;
    let len = payload.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&payload).await?;
    Ok(())
}

/// Receive a length-prefixed JSON message from a Unix stream.
pub async fn recv_message(stream: &mut UnixStream) -> Result<IpcMessage> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(serde_json::from_slice(&payload)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokio::net::UnixListener;

    fn temp_socket() -> std::path::PathBuf {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        std::env::temp_dir().join(format!("rs-shell-core-test-{}.sock", millis))
    }

    #[tokio::test]
    async fn round_trip_ipc_message() {
        let path = temp_socket();
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path).unwrap();

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let msg = recv_message(&mut stream).await.unwrap();
            send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok))
                .await
                .unwrap();
            msg
        });

        let path2 = path.clone();
        let client = tokio::spawn(async move {
            let mut stream = UnixStream::connect(&path2).await.unwrap();
            send_message(
                &mut stream,
                &IpcMessage::Command(ShellCommand::ClipboardList { limit: Some(10) }),
            )
            .await
            .unwrap();
            recv_message(&mut stream).await.unwrap()
        });

        let received = server.await.unwrap();
        let response = client.await.unwrap();

        assert!(matches!(received, IpcMessage::Command(ShellCommand::ClipboardList { .. })));
        assert!(matches!(response, IpcMessage::Response(IpcResponse::Ok)));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn serialize_ipc_message() {
        let msg = IpcMessage::Ping;
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"type":"Ping"}"#);
        let parsed: IpcMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, IpcMessage::Ping));
    }
}
