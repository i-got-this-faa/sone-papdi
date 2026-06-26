//! Integration tests for the sone-papdi API crate.
//!
//! Tests the Unix socket server (subscribe/command flow) using a temporary
//! socket path. D-Bus tests require a session bus and are gated behind a
//! `dbus_test` cfg-flag to allow running in CI without D-Bus.

use sone_papdi_core::bus::EventBus;
use sone_papdi_core::events::{BatteryEvent, BatteryState, BatteryStatus, ShellEvent};
use sone_papdi_core::ipc::{recv_message, send_message, IpcMessage, IpcResponse, ShellCommand};
use std::path::PathBuf;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::time::sleep;

/// Run a Unix server on a temporary socket and return the path and a join handle.
async fn run_test_server(bus: EventBus, socket: PathBuf) {
    tokio::spawn(async move {
        let _ = sone_papdi_api::unix_server::run_unix_server(bus, socket).await;
    });
    // Give the server a moment to bind.
    sleep(Duration::from_millis(100)).await;
}

/// Generate a unique temporary socket path.
fn temp_socket() -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sone-papdi-test-{pid}-{nanos}.sock"))
}

#[tokio::test]
async fn subscribe_receives_events() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    run_test_server(bus.clone(), socket.clone()).await;

    // Connect as a client.
    let mut stream = UnixStream::connect(&socket).await.unwrap();

    // Subscribe to battery events.
    send_message(
        &mut stream,
        &IpcMessage::Subscribe {
            topics: vec!["battery".into()],
        },
    )
    .await
    .unwrap();

    // Read the Ok response.
    let resp = recv_message(&mut stream).await.unwrap();
    assert!(matches!(resp, IpcMessage::Response(IpcResponse::Ok)));

    // Publish a battery event on the bus.
    bus.publish(ShellEvent::Battery(BatteryEvent::StateChanged(BatteryState {
        percentage: 67.0,
        status: BatteryStatus::Discharging,
        time_to_empty_secs: Some(7200),
        time_to_full_secs: None,
        voltage: None,
        temperature: None,
    })));

    // Read the event from the stream.
    let msg = recv_message(&mut stream).await.unwrap();
    assert!(matches!(msg, IpcMessage::Event(ShellEvent::Battery(_))));

    // Cleanup.
    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn subscribe_wildcard_receives_all() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    run_test_server(bus.clone(), socket.clone()).await;

    let mut stream = UnixStream::connect(&socket).await.unwrap();

    // Subscribe to everything.
    send_message(
        &mut stream,
        &IpcMessage::Subscribe {
            topics: vec!["*".into()],
        },
    )
    .await
    .unwrap();

    let resp = recv_message(&mut stream).await.unwrap();
    assert!(matches!(resp, IpcMessage::Response(IpcResponse::Ok)));

    // Publish a lifecycle event.
    bus.publish(ShellEvent::Shell(
        sone_papdi_core::events::ShellLifecycleEvent::DaemonStarted,
    ));

    let msg = recv_message(&mut stream).await.unwrap();
    assert!(matches!(msg, IpcMessage::Event(ShellEvent::Shell(_))));

    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn ping_pong() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    run_test_server(bus, socket.clone()).await;

    let mut stream = UnixStream::connect(&socket).await.unwrap();

    send_message(&mut stream, &IpcMessage::Ping).await.unwrap();
    let resp = recv_message(&mut stream).await.unwrap();
    assert!(matches!(resp, IpcMessage::Pong));

    let _ = std::fs::remove_file(&socket);
}

#[tokio::test]
async fn command_returns_response() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    run_test_server(bus, socket.clone()).await;

    let mut stream = UnixStream::connect(&socket).await.unwrap();

    send_message(
        &mut stream,
        &IpcMessage::Command(ShellCommand::ShowLauncher),
    )
    .await
    .unwrap();

    let resp = recv_message(&mut stream).await.unwrap();
    assert!(matches!(resp, IpcMessage::Response(_)));

    let _ = std::fs::remove_file(&socket);
}
