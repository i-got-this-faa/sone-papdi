//! Integration tests for the sone-papdi API crate.
//!
//! Tests the hyprwire IPC server (subscribe/event flow, ping/pong,
//! command/response) using a temporary socket path.

use anyhow::Result;
use hyprwire::Dispatch;
use sone_papdi_core::bus::EventBus;
use sone_papdi_core::events::{BatteryEvent, BatteryState, BatteryStatus, ShellEvent};
use sone_papdi_core::ipc::{IpcMessage, IpcResponse, ShellCommand};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

// Generated protocol bindings.
#[allow(clippy::enum_variant_names)]
mod shell_proto {
    hyprwire::include_protocol!("sone_papdi_shell_v1");
}

use shell_proto::client::shell_manager_v1::ShellManagerV1;

// ── Test client state ────────────────────────────────────────────

/// State for the test hyprwire client — forwards events/responses via a channel.
struct TestState {
    event_tx: mpsc::Sender<String>,
}

impl Dispatch<ShellManagerV1> for TestState {
    fn event(
        &mut self,
        _object: &ShellManagerV1,
        event: <ShellManagerV1 as hyprwire::Object>::Event<'_>,
    ) {
        match event {
            shell_proto::client::shell_manager_v1::Event::Event { payload, .. } => {
                let _ = self.event_tx.send(payload);
            }
            shell_proto::client::shell_manager_v1::Event::Response { payload } => {
                let _ = self.event_tx.send(payload);
            }
            shell_proto::client::shell_manager_v1::Event::Pong => {}
        }
    }
}

// ── Connection helper ────────────────────────────────────────────

/// Connect to a hyprwire server and return the manager, event receiver,
/// and dispatch thread handle.
fn connect(
    socket: &PathBuf,
) -> Result<(
    ShellManagerV1,
    mpsc::Receiver<String>,
    thread::JoinHandle<()>,
)> {
    let client = hyprwire::client::Client::connect(socket)?;
    let eq = client.new_event_queue();
    let (event_tx, event_rx) = mpsc::channel();
    let mut state = TestState { event_tx };
    eq.wait_for_handshake(&mut state)?;
    let manager: ShellManagerV1 = client.bind(&eq, &mut state, 1)?;
    let handle = thread::spawn(move || loop {
        if let Err(e) = eq.dispatch_events(&mut state, true) {
            eprintln!("dispatch error: {e}");
            thread::sleep(Duration::from_millis(100));
        }
    });
    Ok((manager, event_rx, handle))
}

// ── Helpers ──────────────────────────────────────────────────────

/// Generate a unique temporary socket path.
fn temp_socket() -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("sone-papdi-hyprwire-test-{pid}-{nanos}.sock"))
}

/// Start a hyprwire server on a temp socket and wait for it to bind.
fn start_server(bus: &EventBus, socket: &PathBuf) -> thread::JoinHandle<()> {
    let srv = sone_papdi_api::hyprwire_server::run_hyprwire_server(bus.clone(), socket.clone())
        .expect("start hyprwire server");
    // Give the server a moment to bind and accept connections.
    thread::sleep(Duration::from_millis(200));
    srv
}

// ── Tests ────────────────────────────────────────────────────────

#[test]
fn subscribe_receives_events() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    let _server = start_server(&bus, &socket);

    let (manager, rx, _handle) = connect(&socket).expect("connect hyprwire client");

    // Subscribe to battery events.
    manager.send_subscribe("battery");
    thread::sleep(Duration::from_millis(50));

    // Publish a battery event on the bus.
    bus.publish(ShellEvent::Battery(BatteryEvent::StateChanged(BatteryState {
        percentage: 85.0,
        status: BatteryStatus::Discharging,
        time_to_empty_secs: Some(7200),
        time_to_full_secs: None,
        voltage: None,
        temperature: None,
    })));

    // Read the event from the hyprwire dispatch.
    let json = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("receive event");
    let event: ShellEvent = serde_json::from_str(&json).expect("parse ShellEvent");
    assert!(matches!(event, ShellEvent::Battery(_)));

    let _ = std::fs::remove_file(&socket);
}

#[test]
fn subscribe_wildcard_receives_all() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    let _server = start_server(&bus, &socket);

    let (manager, rx, _handle) = connect(&socket).expect("connect hyprwire client");

    // Subscribe to everything.
    manager.send_subscribe("*");
    thread::sleep(Duration::from_millis(50));

    // Publish a lifecycle event.
    bus.publish(ShellEvent::Shell(
        sone_papdi_core::events::ShellLifecycleEvent::DaemonStarted,
    ));

    let json = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("receive event");
    let event: ShellEvent = serde_json::from_str(&json).expect("parse ShellEvent");
    assert!(matches!(event, ShellEvent::Shell(_)));

    let _ = std::fs::remove_file(&socket);
}

#[test]
fn ping_pong() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    let _server = start_server(&bus, &socket);

    let (manager, _rx, _handle) = connect(&socket).expect("connect hyprwire client");

    // Ping — the server responds with pong; our Dispatch impl ignores it.
    // Success = no error or panic.
    manager.send_ping();
    thread::sleep(Duration::from_millis(100));

    let _ = std::fs::remove_file(&socket);
}

#[test]
fn command_returns_response() {
    let bus = EventBus::new();
    let socket = temp_socket();
    let _ = std::fs::remove_file(&socket);
    let _server = start_server(&bus, &socket);

    let (manager, rx, _handle) = connect(&socket).expect("connect hyprwire client");

    // Send a command.
    let payload =
        serde_json::to_string(&IpcMessage::Command(ShellCommand::ShowLauncher))
            .expect("serialize command");
    manager.send_command(&payload);

    // Read the response.
    let json = rx
        .recv_timeout(Duration::from_secs(2))
        .expect("receive response");
    let resp: IpcResponse = serde_json::from_str(&json).expect("parse IpcResponse");
    assert!(matches!(resp, IpcResponse::Ok));

    let _ = std::fs::remove_file(&socket);
}
