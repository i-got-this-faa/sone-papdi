//! Hyprwire server for internal IPC in sone-papdi.
//!
//! Serves the `sone_papdi_shell_v1` protocol on a Unix socket. Clients
//! subscribe to event topics and send commands. Events from the global
//! [`EventBus`] are forwarded to subscribed clients.

use anyhow::Result;
use sone_papdi_core::bus::EventBus;
use sone_papdi_core::events::{event_topic, ShellEvent};
use sone_papdi_core::ipc::{IpcMessage, IpcResponse};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Generated protocol bindings.
#[allow(clippy::enum_variant_names, clippy::ptr_arg)]
mod shell_proto {
    hyprwire::include_protocol!("sone_papdi_shell_v1");
}

use shell_proto::server::shell_manager_v1::ShellManagerV1;
use shell_proto::server::SonePapdiShellV1Handler;

// ── Shared server state ──────────────────────────────────────

#[derive(Clone)]
struct SharedState {
    inner: Arc<parking_lot::Mutex<ServerInner>>,
}

struct ClientEntry {
    manager: ShellManagerV1,
    subscriptions: HashSet<String>,
}

struct ServerInner {
    clients: Vec<ClientEntry>,
}

/// Match a client entry by comparing hyprwire object pointer equality.
fn find_client<'a>(
    clients: &'a mut [ClientEntry],
    obj: &ShellManagerV1,
) -> Option<&'a mut ClientEntry> {
    clients.iter_mut().find(|c| c.manager == *obj)
}

// ── Handler ──────────────────────────────────────────────────

struct ServerHandler {
    state: SharedState,
    bus: EventBus,
}

impl SonePapdiShellV1Handler for ServerHandler {
    fn bind(&mut self, object: ShellManagerV1) {
        self.state.inner.lock().clients.push(ClientEntry {
            subscriptions: HashSet::new(),
            manager: object,
        });
        tracing::debug!("hyprwire client connected");
    }
}

impl hyprwire::Dispatch<ShellManagerV1> for ServerHandler {
    fn event(
        &mut self,
        object: &ShellManagerV1,
        event: <ShellManagerV1 as hyprwire::Object>::Event<'_>,
    ) {
        match event {
            shell_proto::server::shell_manager_v1::Event::Subscribe { topic } => {
                if let Some(client) = find_client(&mut self.state.inner.lock().clients, object) {
                    client.subscriptions.insert(topic);
                }
            }
            shell_proto::server::shell_manager_v1::Event::Unsubscribe { topic } => {
                if let Some(client) = find_client(&mut self.state.inner.lock().clients, object) {
                    client.subscriptions.remove(&topic);
                }
            }
            shell_proto::server::shell_manager_v1::Event::Command { payload } => {
                let resp = match serde_json::from_str::<IpcMessage>(&payload) {
                    Ok(IpcMessage::Command(cmd)) => {
                        self.bus.publish(ShellEvent::Shell(
                            sone_papdi_core::events::ShellLifecycleEvent::ServiceStarted(
                                format!("cmd:{cmd:?}"),
                            ),
                        ));
                        IpcResponse::Ok
                    }
                    _ => IpcResponse::error("invalid command"),
                };
                let json = serde_json::to_string(&resp).unwrap_or_default();
                object.send_response(json);
            }
            shell_proto::server::shell_manager_v1::Event::Ping => {
                object.send_pong();
            }
        }
    }
}

// ── Broadcast ────────────────────────────────────────────────

fn broadcast_event(state: &SharedState, event: &ShellEvent) {
    let topic = event_topic(event);
    let payload = serde_json::to_string(event).unwrap_or_default();
    let mut inner = state.inner.lock();
    for client in &mut inner.clients {
        if client.subscriptions.is_empty()
            || client.subscriptions.contains(topic)
            || client.subscriptions.contains("*")
        {
            client.manager.send_event(topic, &payload);
        }
    }
}

// ── Entry point ──────────────────────────────────────────────

/// Run the hyprwire IPC server on a dedicated thread.
///
/// Listens on `socket`, dispatches hyprwire protocol traffic, and bridges
/// [`ShellEvent`]s from the [`EventBus`] to subscribed clients.
pub fn run_hyprwire_server(bus: EventBus, socket: PathBuf) -> Result<thread::JoinHandle<()>> {
    let _ = std::fs::remove_file(&socket);

    let mut server = hyprwire::server::Server::bind(&socket)?;
    tracing::info!("hyprwire server: listening on {}", socket.display());

    let state = SharedState {
        inner: Arc::new(parking_lot::Mutex::new(ServerInner {
            clients: Vec::new(),
        })),
    };

    // Everything runs on one thread so the handler reference stays valid.
    let handle = thread::spawn(move || {
        let mut handler = ServerHandler {
            state: state.clone(),
            bus: bus.clone(),
        };

        server.add_implementation::<
            shell_proto::server::SonePapdiShellV1Impl,
            ServerHandler,
        >(&mut handler, 1);

        // Bridge: forward EventBus events to hyprwire clients.
        let bridge_state = state.clone();
        thread::spawn(move || {
            let mut rx = bus.subscribe();
            loop {
                if let Ok(event) = rx.try_recv() {
                    broadcast_event(&bridge_state, &event);
                }
                thread::sleep(Duration::from_millis(10));
            }
        });

        // Dispatch incoming hyprwire messages.
        loop {
            if let Err(e) = server.dispatch_events(&mut handler, true) {
                tracing::warn!("hyprwire dispatch error: {e}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    Ok(handle)
}
