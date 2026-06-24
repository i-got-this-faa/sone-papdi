use anyhow::Result;
use rs_shell_core::bus::EventBus;
use rs_shell_core::events::{event_topic, ShellEvent};
use rs_shell_core::ipc::{recv_message, send_message, IpcMessage, IpcResponse, ShellCommand};
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::net::{UnixListener, UnixStream};

/// Run the Unix socket server.
pub async fn run_unix_server(bus: EventBus, socket: PathBuf) -> Result<()> {
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket)?;
    tracing::info!("unix api server listening on {}", socket.display());

    loop {
        let (stream, _) = listener.accept().await?;
        let bus = bus.clone();
        tokio::spawn(handle_client(stream, bus));
    }
}

async fn handle_client(mut stream: UnixStream, bus: EventBus) {
    let mut subscribed: HashSet<String> = HashSet::new();
    let mut event_rx = bus.subscribe();

    loop {
        tokio::select! {
            msg = recv_message(&mut stream) => {
                match msg {
                    Ok(IpcMessage::Subscribe { topics }) => {
                        subscribed.extend(topics);
                        let _ = send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok)).await;
                    }
                    Ok(IpcMessage::Unsubscribe { topics }) => {
                        for t in topics {
                            subscribed.remove(&t);
                        }
                        let _ = send_message(&mut stream, &IpcMessage::Response(IpcResponse::Ok)).await;
                    }
                    Ok(IpcMessage::Command(cmd)) => {
                        let resp = dispatch_command(cmd, &bus).await;
                        let _ = send_message(&mut stream, &IpcMessage::Response(resp)).await;
                    }
                    Ok(IpcMessage::Ping) => {
                        let _ = send_message(&mut stream, &IpcMessage::Pong).await;
                    }
                    Err(_) | Ok(IpcMessage::Pong) | Ok(IpcMessage::Event(_)) | Ok(IpcMessage::Response(_)) => break,
                }
            }
            Ok(event) = event_rx.recv() => {
                let topic = event_topic(&event);
                if subscribed.is_empty() || subscribed.contains(topic) || subscribed.contains("*") {
                    if send_message(&mut stream, &IpcMessage::Event(event)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

async fn dispatch_command(cmd: ShellCommand, bus: &EventBus) -> IpcResponse {
    match cmd {
        ShellCommand::ClipboardList { limit } => {
            // Forward to clipboard via event bus request/response is not implemented in MVP;
            // emit a lifecycle event indicating the command was received.
            bus.publish(ShellEvent::Shell(rs_shell_core::events::ShellLifecycleEvent::ServiceStarted(
                "clipboard-list".into(),
            )));
            IpcResponse::OkWith(serde_json::json!({ "limit": limit }))
        }
        ShellCommand::ShowLauncher => {
            bus.publish(ShellEvent::Shell(rs_shell_core::events::ShellLifecycleEvent::ServiceStarted(
                "show-launcher".into(),
            )));
            IpcResponse::Ok
        }
        ShellCommand::HideLauncher => {
            bus.publish(ShellEvent::Shell(rs_shell_core::events::ShellLifecycleEvent::ServiceStarted(
                "hide-launcher".into(),
            )));
            IpcResponse::Ok
        }
        _ => IpcResponse::error("command not implemented in MVP"),
    }
}
