//! D-Bus server exposing `org.sone_papdi.Shell1` on the session bus.
//!
//! Provides methods for external tools to query state and send commands.
//! Emits D-Bus signals when shell events occur.

use anyhow::Result;
use sone_papdi_core::bus::EventBus;
use sone_papdi_core::events::{
    event_topic, AudioEvent, ShellEvent, ShellLifecycleEvent, ThemeEvent,
};

// ── D-Bus interface ──────────────────────────────────────────

struct ShellInterface {
    bus: EventBus,
}

#[zbus::interface(name = "org.sone_papdi.Shell1")]
impl ShellInterface {
    // ── Audio ────────────────────────────────────────────────
    async fn set_volume(&self, sink: String, volume: f64) -> zbus::fdo::Result<()> {
        self.bus.publish(ShellEvent::Audio(AudioEvent::VolumeChanged {
            sink: if sink.is_empty() { "default".into() } else { sink },
            volume,
        }));
        Ok(())
    }

    async fn toggle_mute(&self, sink: String) -> zbus::fdo::Result<()> {
        self.bus.publish(ShellEvent::Audio(AudioEvent::MuteToggled {
            sink: if sink.is_empty() { "default".into() } else { sink },
            muted: true,
        }));
        Ok(())
    }

    // ── Wallpaper / Theme ────────────────────────────────────
    async fn set_wallpaper(&self, path: String, output: String) -> zbus::fdo::Result<()> {
        self.bus.publish(ShellEvent::Wallpaper(
            sone_papdi_core::events::WallpaperEvent::Changed {
                path,
                output: if output.is_empty() { None } else { Some(output) },
            },
        ));
        Ok(())
    }

    async fn set_theme(&self, name: String) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Theme(ThemeEvent::Changed { name }));
        Ok(())
    }

    async fn toggle_dark_mode(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "toggle-dark-mode".into(),
            )));
        Ok(())
    }

    // ── Launcher ─────────────────────────────────────────────
    async fn show_launcher(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "show-launcher".into(),
            )));
        Ok(())
    }

    async fn hide_launcher(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "hide-launcher".into(),
            )));
        Ok(())
    }

    // ── Window ───────────────────────────────────────────────
    async fn switch_workspace(&self, workspace: String) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                format!("switch-workspace:{workspace}"),
            )));
        Ok(())
    }

    // ── Shell ────────────────────────────────────────────────
    async fn show_notification_center(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "show-notification-center".into(),
            )));
        Ok(())
    }

    async fn lock_screen(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "lock-screen".into(),
            )));
        Ok(())
    }

    async fn suspend(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "suspend".into(),
            )));
        Ok(())
    }

    async fn logout(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "logout".into(),
            )));
        Ok(())
    }

    async fn reload(&self) -> zbus::fdo::Result<()> {
        self.bus
            .publish(ShellEvent::Shell(ShellLifecycleEvent::ServiceStarted(
                "reload".into(),
            )));
        Ok(())
    }
}

// ── Signal helpers ───────────────────────────────────────────

/// Map a `ShellEvent` variant to its D-Bus signal name.
fn signal_name(event: &ShellEvent) -> Option<&'static str> {
    match event {
        ShellEvent::Battery(_) => Some("BatteryChanged"),
        ShellEvent::Audio(_) => Some("VolumeChanged"),
        ShellEvent::Network(_) => Some("NetworkChanged"),
        ShellEvent::Notification(_) => Some("NotificationReceived"),
        ShellEvent::Theme(_) => Some("ThemeChanged"),
        ShellEvent::Wallpaper(_) => Some("WallpaperChanged"),
        ShellEvent::Window(_) => Some("WindowChanged"),
        ShellEvent::Shell(_) => Some("ShellEvent"),
        ShellEvent::Clipboard(_) => Some("ClipboardChanged"),
        ShellEvent::Wireless(_) => Some("WirelessChanged"),
    }
}

/// Serialise a `ShellEvent` payload as a JSON string for D-Bus signal emission.
fn event_to_json_value(event: &ShellEvent) -> serde_json::Value {
    serde_json::to_value(event).unwrap_or(serde_json::Value::Null)
}

// ── Server entry point ───────────────────────────────────────

/// Run the D-Bus server, serving `org.sone_papdi.Shell1` on the session bus.
///
/// Returns when the D-Bus connection is closed (or errors). Spawns an internal
/// bridge task that translates `ShellEvent` → D-Bus signals.
pub async fn run_dbus_server(bus: EventBus) -> Result<()> {
    let interface = ShellInterface { bus: bus.clone() };

    let conn = zbus::connection::Builder::session()?
        .name("org.sone_papdi.Shell1".to_owned())?
        .serve_at("/org/sone_papdi/Shell1", interface)?
        .build()
        .await?;

    // Bridge: subscribe to the event bus and emit D-Bus signals.
    let signal_conn = conn.clone();
    tokio::spawn(async move {
        let mut rx = bus.subscribe();
        while let Ok(event) = rx.recv().await {
            let name = match signal_name(&event) {
                Some(n) => n,
                None => continue,
            };
            let payload = event_to_json_value(&event);
            let topic = event_topic(&event);
            // Emit signal with topic and JSON payload.
            let _ = signal_conn
                .emit_signal(
                    None::<&str>,
                    "/org/sone_papdi/Shell1",
                    "org.sone_papdi.Shell1",
                    name,
                    &(topic, &payload.to_string()),
                )
                .await;
        }
    });

    tracing::info!("dbus server running: org.sone_papdi.Shell1");
    // Keep alive forever; the internal executor handles D-Bus dispatch.
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    }
}
