use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level shell event. Every subsystem emits one of these.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ShellEvent {
    /// Clipboard subsystem event.
    Clipboard(ClipboardEvent),

    /// Window management event.
    Window(WindowEvent),

    /// Battery subsystem event.
    Battery(BatteryEvent),

    /// Network subsystem event.
    Network(NetworkEvent),

    /// Audio subsystem event.
    Audio(AudioEvent),

    /// Wireless subsystem event.
    Wireless(WirelessEvent),

    /// Wallpaper change event.
    Wallpaper(WallpaperEvent),

    /// Theme change event.
    Theme(ThemeEvent),

    /// Desktop notification event.
    Notification(NotificationEvent),

    /// Shell lifecycle event.
    Shell(ShellLifecycleEvent),
}

/// Category/topic name for a shell event, used for subscription filtering.
pub fn event_topic(event: &ShellEvent) -> &'static str {
    match event {
        ShellEvent::Clipboard(_) => "clipboard",
        ShellEvent::Window(_) => "window",
        ShellEvent::Battery(_) => "battery",
        ShellEvent::Network(_) => "network",
        ShellEvent::Audio(_) => "audio",
        ShellEvent::Wireless(_) => "wireless",
        ShellEvent::Wallpaper(_) => "wallpaper",
        ShellEvent::Theme(_) => "theme",
        ShellEvent::Notification(_) => "notification",
        ShellEvent::Shell(_) => "shell",
    }
}

// ── Clipboard ────────────────────────────────────────────────────────────────

/// A single clipboard entry as stored in the database and returned by queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipEntry {
    pub id: i64,
    pub mime: String,
    pub kind: ClipEntryKind,
    pub preview: String,
    pub content: Option<String>,
    pub pinned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipEntryKind {
    Text,
    Link,
    Color,
    Image,
    File,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardEvent {
    Stored {
        id: i64,
        mime: String,
        kind: ClipEntryKind,
        preview: String,
    },
    Copied {
        id: i64,
    },
    Deleted {
        id: i64,
    },
    Cleared,
    PinnedToggled {
        id: i64,
        pinned: bool,
    },
    SearchResult {
        query: String,
        results: Vec<ClipEntry>,
    },
}

// ── Battery ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub percentage: f64,
    pub status: BatteryStatus,
    pub time_to_empty_secs: Option<i64>,
    pub time_to_full_secs: Option<i64>,
    pub voltage: Option<f64>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatteryEvent {
    StateChanged(BatteryState),
    LevelCritical(f64),
    LevelLow(f64),
    PluggedIn,
    Unplugged,
}

// ── Network ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    pub connectivity: NetworkConnectivity,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkConnectivity {
    None,
    Limited,
    Portal,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub kind: InterfaceKind,
    pub ip4: Option<String>,
    pub ip6: Option<String>,
    pub state: InterfaceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceKind {
    Ethernet,
    Wifi,
    Loopback,
    Vpn,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceState {
    Connected,
    Disconnected,
    Connecting,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    StateChanged(NetworkState),
    InterfaceUp(NetworkInterface),
    InterfaceDown(String),
    ConnectivityChanged(NetworkConnectivity),
}

// ── Audio ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioState {
    pub sink_volume: f64,
    pub sink_muted: bool,
    pub source_volume: f64,
    pub source_muted: bool,
    pub default_sink: String,
    pub default_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioEvent {
    VolumeChanged { sink: String, volume: f64 },
    MuteToggled { sink: String, muted: bool },
    DefaultSinkChanged(String),
    DefaultSourceChanged(String),
}

// ── Wireless ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: u8,
    pub secured: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WirelessEvent {
    Connected(WifiNetwork),
    Disconnected,
    SignalStrengthChanged(u8),
    NetworksScanned(Vec<WifiNetwork>),
}

// ── Notification ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: Option<String>,
    pub urgency: NotificationUrgency,
    pub actions: Vec<(String, String)>,
    pub timestamp: DateTime<Utc>,
    pub expire_timeout: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    Received(Notification),
    Closed { id: u32, reason: CloseReason },
    ActionInvoked { id: u32, action_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloseReason {
    Expired,
    Dismissed,
    AppClosed,
    Undefined,
}

// ── Window ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub app_id: String,
    pub workspace: Option<String>,
    pub focused: bool,
    pub state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    Fullscreen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowEvent {
    Opened(WindowInfo),
    Closed(String),
    FocusChanged(String),
    TitleChanged { id: String, title: String },
    WorkspaceChanged { id: String, workspace: String },
    StateChanged { id: String, state: WindowState },
    WorkspaceList(Vec<String>),
    ActiveWorkspaceChanged(String),
}

// ── Wallpaper / Theme ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WallpaperEvent {
    Changed { path: String, output: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeEvent {
    Changed { name: String },
    ColorsUpdated(HashMap<String, String>),
    DarkModeToggled(bool),
}

// ── Lifecycle ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellLifecycleEvent {
    DaemonStarted,
    ServiceStarted(String),
    ServiceFailed { name: String, error: String },
    ShuttingDown,
    LockScreen,
    Unlock,
    Suspend,
    Hibernate,
}
