//! Native `wlr-data-control-unstable-v1` Wayland protocol binding.
//!
//! Runs in a dedicated thread, communicating clipboard content changes and
//! copy-from-history commands with the async main loop via `mpsc` channels.

use anyhow::{Context, Result};
use std::os::unix::io::{AsFd, FromRawFd, OwnedFd, RawFd};
use std::time::Duration;
use tokio::sync::mpsc;
use wayland_client::protocol::wl_registry::{self, WlRegistry};
use wayland_client::{Connection, Dispatch, QueueHandle};

// Generated protocol types from wayland-protocols-wlr.
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_device_v1::{
    Event as DeviceEvent, ZwlrDataControlDeviceV1,
};
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_manager_v1::{
    ZwlrDataControlManagerV1,
};
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_offer_v1::{
    Event as OfferEvent, ZwlrDataControlOfferV1,
};
use wayland_protocols_wlr::data_control::v1::client::zwlr_data_control_source_v1::{
    Event as SourceEvent, ZwlrDataControlSourceV1,
};

// ── Channel types ────────────────────────────────────────────

/// Commands the async main loop sends to the Wayland thread.
#[derive(Debug)]
pub enum WaylandCmd {
    SetClipboard { mime: String, content: Vec<u8> },
    Shutdown,
}

/// Events the Wayland thread sends back to the async main loop.
#[derive(Debug)]
pub enum WaylandEvent {
    Content { mime: String, data: Vec<u8> },
    Error(String),
}

/// Spawn the Wayland clipboard monitoring thread.
///
/// Returns a receiver for clipboard content events and a sender for commands.
pub fn spawn_wayland_thread(
) -> Result<(
    mpsc::UnboundedReceiver<WaylandEvent>,
    mpsc::UnboundedSender<WaylandCmd>,
)> {
    let (content_tx, content_rx) = mpsc::unbounded_channel::<WaylandEvent>();
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<WaylandCmd>();

    std::thread::Builder::new()
        .name("wlr-data-control".into())
        .spawn(move || {
            if let Err(e) = run_wayland_loop(content_tx, cmd_rx) {
                tracing::error!("Wayland clipboard thread exited: {e:#}");
            }
        })
        .context("failed to spawn Wayland clipboard thread")?;

    Ok((content_rx, cmd_tx))
}

// ── MIME preference ──────────────────────────────────────────

fn pick_best_mime(mime_types: &[String]) -> Option<String> {
    if mime_types.is_empty() {
        return None;
    }
    for preferred in &[
        "text/plain",
        "text/plain;charset=utf-8",
        "text/*",
        "image/png",
        "image/jpeg",
        "image/webp",
        "image/*",
        "application/octet-stream",
    ] {
        for mt in mime_types {
            if mt == preferred
                || (preferred.ends_with("/*")
                    && mt.starts_with(&preferred[..preferred.len() - 1]))
            {
                return Some(mt.clone());
            }
        }
    }
    Some(mime_types[0].clone())
}

// ── Main event loop ──────────────────────────────────────────

fn run_wayland_loop(
    content_tx: mpsc::UnboundedSender<WaylandEvent>,
    mut cmd_rx: mpsc::UnboundedReceiver<WaylandCmd>,
) -> Result<()> {
    let conn = Connection::connect_to_env().context("failed to connect to Wayland display")?;
    let mut event_queue = conn.new_event_queue::<WaylandState>();
    let qh = event_queue.handle();
    let registry = conn.display().get_registry(&qh, ());

    let mut state = WaylandState {
        manager: None,
        device: None,
        pending_source: None,
        current_offer: None,
        offer_mime_types: Vec::new(),
        content_tx,
        qh,
        registry: Some(registry),
    };

    // Roundtrip to receive initial global announcements and bind protocols.
    event_queue.roundtrip(&mut state)?;
    conn.flush()?;

    tracing::info!("Wayland clipboard thread started (wlr-data-control)");

    loop {
        event_queue.dispatch_pending(&mut state)?;

        // Process collected offers.
        if let Some(offer) = state.current_offer.take() {
            if !state.offer_mime_types.is_empty() {
                if let Err(e) = process_offer(&state, &offer) {
                    tracing::warn!("failed to process clipboard offer: {e}");
                }
            }
            offer.destroy();
            state.offer_mime_types.clear();
        }

        // Process commands from the async main loop.
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                WaylandCmd::SetClipboard { mime, content } => {
                    set_clipboard_from_history(&mut state, &mime, &content);
                }
                WaylandCmd::Shutdown => {
                    tracing::info!("Wayland clipboard thread shutting down");
                    state.pending_source = None;
                    return Ok(());
                }
            }
        }

        conn.flush()?;
        std::thread::sleep(Duration::from_millis(10));
    }
}

// ── Thread-local state ───────────────────────────────────────

struct WaylandState {
    manager: Option<ZwlrDataControlManagerV1>,
    device: Option<ZwlrDataControlDeviceV1>,
    pending_source: Option<(ZwlrDataControlSourceV1, String, Vec<u8>)>,
    current_offer: Option<ZwlrDataControlOfferV1>,
    offer_mime_types: Vec<String>,
    content_tx: mpsc::UnboundedSender<WaylandEvent>,
    qh: QueueHandle<WaylandState>,
    registry: Option<WlRegistry>,
}

// ── Dispatch: WlRegistry ─────────────────────────────────────

impl Dispatch<WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &WlRegistry,
        event: <WlRegistry as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        else {
            return;
        };

        match interface.as_str() {
            "zwlr_data_control_manager_v1" => {
                if let Some(registry) = state.registry.as_ref() {
                    let mgr: ZwlrDataControlManagerV1 =
                        registry.bind(name, version.min(2), &state.qh, ());
                    state.manager = Some(mgr);
                    tracing::debug!("bound zwlr_data_control_manager_v1 v{version}");
                }
            }
            "wl_seat" => {
                if let (Some(manager), Some(registry)) = (&state.manager, &state.registry) {
                    let seat: wayland_client::protocol::wl_seat::WlSeat =
                        registry.bind(name, version.min(7), &state.qh, ());
                    let device = manager.get_data_device(&seat, &state.qh, ());
                    state.device = Some(device);
                    tracing::debug!("created zwlr_data_control_device_v1");
                }
            }
            _ => {}
        }
    }
}

// ── Dispatch: ZwlrDataControlDeviceV1 ────────────────────────

impl Dispatch<ZwlrDataControlDeviceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlDeviceV1,
        event: <ZwlrDataControlDeviceV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            DeviceEvent::DataOffer { .. } => {
                tracing::trace!("data offer received");
            }
            DeviceEvent::Selection { id: Some(offer) } => {
                tracing::debug!("clipboard selection changed");
                state.current_offer = Some(offer);
                state.offer_mime_types.clear();
            }
            DeviceEvent::Selection { id: None } => {
                tracing::debug!("clipboard selection cleared");
            }
            _ => {}
        }
    }
}

// ── Dispatch: ZwlrDataControlOfferV1 ─────────────────────────

impl Dispatch<ZwlrDataControlOfferV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlOfferV1,
        event: <ZwlrDataControlOfferV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let OfferEvent::Offer { mime_type } = event {
            state.offer_mime_types.push(mime_type);
        }
    }
}

// ── Dispatch: ZwlrDataControlManagerV1 (no protocol events) ──

impl Dispatch<ZwlrDataControlManagerV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrDataControlManagerV1,
        _event: <ZwlrDataControlManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // No events defined for the manager.
    }
}

// ── Dispatch: WlSeat (no events we care about) ──────────────

impl Dispatch<wayland_client::protocol::wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &wayland_client::protocol::wl_seat::WlSeat,
        _event: <wayland_client::protocol::wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        // Seat events are not relevant for clipboard management.
    }
}

// ── Dispatch: ZwlrDataControlSourceV1 ────────────────────────

impl Dispatch<ZwlrDataControlSourceV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &ZwlrDataControlSourceV1,
        event: <ZwlrDataControlSourceV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            SourceEvent::Send { mime_type, fd } => {
                if let Some((_, ref src_mime, ref content)) = state.pending_source {
                    if src_mime == &mime_type {
                        tracing::debug!(
                            "writing {} bytes for clipboard source ({mime_type})",
                            content.len()
                        );
                        use std::io::Write;
                        let mut file = std::fs::File::from(fd);
                        let _ = file.write_all(content);
                    }
                }
            }
            SourceEvent::Cancelled => {
                state.pending_source = None;
            }
            _ => {}
        }
    }
}

// ── Processing ───────────────────────────────────────────────

fn process_offer(state: &WaylandState, offer: &ZwlrDataControlOfferV1) -> Result<()> {
    let mime = pick_best_mime(&state.offer_mime_types)
        .context("no suitable MIME type in clipboard offer")?;

    let data = receive_clipboard_content(offer, &mime)?;
    if data.is_empty() {
        return Ok(());
    }

    let _ = state.content_tx.send(WaylandEvent::Content { mime, data });
    Ok(())
}

fn receive_clipboard_content(offer: &ZwlrDataControlOfferV1, mime: &str) -> Result<Vec<u8>> {
    let (read_fd, write_fd) = create_pipe()?;
    // receive takes (mime_type: &str, fd: BorrowedFd<'_>)
    offer.receive(mime.to_string(), write_fd.as_fd());

    let mut file = std::fs::File::from(read_fd);
    let mut data = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn set_clipboard_from_history(state: &mut WaylandState, mime: &str, content: &[u8]) {
    let Some(manager) = state.manager.as_ref() else {
        tracing::warn!("cannot set clipboard: manager not bound");
        return;
    };
    let Some(device) = state.device.as_ref() else {
        tracing::warn!("cannot set clipboard: device not bound");
        return;
    };

    let source = manager.create_data_source(&state.qh, ());
    source.offer(mime.to_string());
    device.set_selection(Some(&source));

    state.pending_source = Some((source, mime.to_string(), content.to_vec()));

    tracing::debug!("clipboard source created ({mime}, {} bytes)", content.len());
}

// ── Helpers ──────────────────────────────────────────────────

fn create_pipe() -> Result<(OwnedFd, OwnedFd)> {
    let mut fds: [RawFd; 2] = [0; 2];
    let ret = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if ret != 0 {
        anyhow::bail!("pipe2 failed: {}", std::io::Error::last_os_error());
    }
    let read_fd = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    Ok((read_fd, write_fd))
}
