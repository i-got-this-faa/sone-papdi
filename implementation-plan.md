# rs-shell — Complete Wayland Desktop Shell
## Super-Detailed Implementation Plan

> **Baseline**: rsclip v0.1.11 — daemon/UI split, GTK4 layer-shell, SQLite, Unix datagram
> socket IPC, TOML hot-reload. Every architectural decision in rs-shell extends that foundation.
>
> **GTK note**: rsclip uses GTK4. rs-shell uses GTK4 throughout.
> GTK3 on Wayland is a regression — no GPU compositing, no fractional scaling, and
> `gtk-layer-shell` (GTK3) is effectively unmaintained compared to `gtk4-layer-shell`.

---

## Table of Contents

1. [Design Philosophy](#1-design-philosophy)
2. [Workspace Layout](#2-workspace-layout)
3. [Crate Catalogue](#3-crate-catalogue)
4. [Core IPC & Event Bus (`rs-shell-core`)](#4-core-ipc--event-bus-rs-shell-core)
5. [Configuration Engine (`rs-shell-config`)](#5-configuration-engine-rs-shell-config)
6. [Clipboard Engine (`rs-shell-clipboard`)](#6-clipboard-engine-rs-shell-clipboard)
7. [Subscribable API (`rs-shell-api`)](#7-subscribable-api-rs-shell-api)
8. [Window Manager Bridge (`rs-shell-wm`)](#8-window-manager-bridge-rs-shell-wm)
9. [Service Layer (`rs-shell-services`)](#9-service-layer-rs-shell-services)
10. [UI Layer (`rs-shell-ui`)](#10-ui-layer-rs-shell-ui)
11. [Shell Daemon (`rs-shell-daemon`)](#11-shell-daemon-rs-shell-daemon)
12. [Theme System](#12-theme-system)
13. [Security Model](#13-security-model)
14. [Performance Architecture](#14-performance-architecture)
15. [Build System & Packaging](#15-build-system--packaging)
16. [Phased Roadmap](#16-phased-roadmap)
17. [Full Dependency Map](#17-full-dependency-map)

---

## 1. Design Philosophy

### Principles

| Principle | Implementation |
|-----------|---------------|
| **Daemon/UI separation** | Headless daemons run from systemd; UI processes start on demand (rsclip model) |
| **Zero-overhead when idle** | All UI is resident-but-hidden or spawned on demand |
| **Single source of truth** | One TOML config file, hot-reloaded by all processes |
| **Typed event bus** | No stringly-typed pub/sub; every event is a Rust enum |
| **Layered IPC** | In-process: tokio broadcast. Cross-process: Unix socket. External tools: D-Bus |
| **WM-agnostic core** | Core works on any `wlr-foreign-toplevel` compositor; per-WM adapters are plug-ins |
| **Batteries included** | Every service ships a sane default; zero config required to get a working desktop |

### What rs-shell Provides Out of the Box

```
┌─────────────────────────────────────────────────────────────┐
│                         rs-shell                            │
│                                                             │
│  ┌──────────┐  ┌────────────┐  ┌──────────────────────────┐│
│  │  Panel   │  │  Launcher  │  │  Notification Center     ││
│  │  (bar)   │  │  (wofi-    │  │  (OSD + history drawer)  ││
│  │          │  │   like)    │  │                          ││
│  └──────────┘  └────────────┘  └──────────────────────────┘│
│                                                             │
│  ┌──────────────┐  ┌────────────┐  ┌──────────────────────┐│
│  │  Clipboard   │  │  Quick     │  │  Lock Screen         ││
│  │  Manager     │  │  Settings  │  │  (optional)          ││
│  │  (rsclip)    │  │  Panel     │  │                      ││
│  └──────────────┘  └────────────┘  └──────────────────────┘│
│                                                             │
│  ────────────── Service Layer (D-Bus) ────────────────────  │
│  Battery │ Network │ Audio │ Wireless │ Wallpaper │ Theme   │
│                                                             │
│  ─────────────── WM Bridge Layer ─────────────────────────  │
│  Hyprland │ sway │ niri │ river │ generic wlr-toplevel     │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Workspace Layout

```
rs-shell/
├── Cargo.toml                   # workspace manifest + shared deps
├── Cargo.lock
├── xtask/                       # build automation (cargo xtask)
│   └── src/main.rs
├── crates/                      # flat crate layout (rust-analyzer style)
│   ├── rs-shell-core/           # shared types, IPC primitives, event bus
│   ├── rs-shell-config/         # TOML config + hot-reload engine
│   ├── rs-shell-clipboard/      # rsclip-based clipboard engine
│   ├── rs-shell-api/            # subscribable API server (Unix + D-Bus)
│   ├── rs-shell-wm/             # WM bridge: wlr-toplevel + per-WM adapters
│   ├── rs-shell-services/       # Battery, Network, Audio, Wireless, Wallpaper, Theme
│   ├── rs-shell-ui/             # GTK4 UI components (layer-shell surfaces)
│   ├── rs-shell-daemon/         # orchestrator binary: starts all services
│   └── rs-shell-bar/            # panel binary (separate process for isolation)
├── assets/
│   ├── themes/                  # bundled themes (.css)
│   ├── icons/                   # fallback icons
│   └── default-config.toml
├── packaging/
│   ├── systemd/                 # user service units
│   ├── aur/                     # AUR PKGBUILD
│   └── deb/                     # Debian packaging
└── scripts/
    ├── install.sh
    └── build-release-archive.sh
```

### Root `Cargo.toml` (workspace)

```toml
[workspace]
resolver = "2"
members = [
    "xtask",
    "crates/rs-shell-core",
    "crates/rs-shell-config",
    "crates/rs-shell-clipboard",
    "crates/rs-shell-api",
    "crates/rs-shell-wm",
    "crates/rs-shell-services",
    "crates/rs-shell-ui",
    "crates/rs-shell-daemon",
    "crates/rs-shell-bar",
]

[workspace.package]
version   = "0.1.0"
edition   = "2021"
license   = "MIT"
authors   = ["rs-shell contributors"]
rust-version = "1.80"

[workspace.dependencies]
# Async
tokio         = { version = "1", features = ["full"] }
tokio-stream  = "0.1"
futures       = "0.3"

# IPC / D-Bus
zbus          = { version = "4", features = ["tokio"] }
zvariant      = "4"

# Serialization
serde         = { version = "1", features = ["derive"] }
serde_json    = "1"
toml          = "0.8"

# Storage
rusqlite      = { version = "0.31", features = ["bundled"] }

# UI
gtk4          = { version = "0.9", features = ["v4_12"] }
gtk4-layer-shell = "0.8"
glib          = "0.20"

# Wayland
wayland-client   = "0.31"
wayland-protocols-wlr = { version = "0.3", features = ["client"] }

# Error handling
anyhow        = "1"
thiserror     = "1"

# Logging
tracing       = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Config & file watching
notify        = "6"

# Misc
once_cell     = "1"
parking_lot   = "0.12"
bytes         = "1"
uuid          = { version = "1", features = ["v4"] }
chrono        = { version = "0.4", features = ["serde"] }

[profile.release]
lto           = "fat"
codegen-units = 1
opt-level     = 3
panic         = "abort"
strip         = "symbols"

[profile.dev]
opt-level     = 1
debug         = 1
```

---

## 3. Crate Catalogue

| Crate | Binary? | Purpose |
|-------|---------|---------|
| `rs-shell-core` | lib | Shared event types, IPC protocol, error types |
| `rs-shell-config` | lib | Config schema, file watcher, hot-reload broadcast |
| `rs-shell-clipboard` | lib + `rsclipd` bin | rsclip clipboard daemon extended for shell |
| `rs-shell-api` | lib | Subscribable API: Unix socket server + D-Bus server |
| `rs-shell-wm` | lib | WM bridge: wlr-toplevel, Hyprland/sway/niri adapters |
| `rs-shell-services` | lib | All system services (Battery, Network, Audio…) |
| `rs-shell-ui` | lib | GTK4 components: panel widgets, launcher, notif OSD |
| `rs-shell-daemon` | `rs-shelld` bin | Orchestrator: spawns services, hosts API |
| `rs-shell-bar` | `rs-shell-bar` bin | Panel process with GTK4 + layer-shell |

---

## 4. Core IPC & Event Bus (`rs-shell-core`)

This crate is the nervous system. Every other crate depends on it. It defines the
canonical event types and IPC primitives.

### 4.1 Event Type Hierarchy

```rust
// crates/rs-shell-core/src/events.rs

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Top-level shell event. Every subsystem emits one of these.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ShellEvent {
    // ── Clipboard ──────────────────────────────────────────
    Clipboard(ClipboardEvent),

    // ── Window management ──────────────────────────────────
    Window(WindowEvent),

    // ── System services ────────────────────────────────────
    Battery(BatteryEvent),
    Network(NetworkEvent),
    Audio(AudioEvent),
    Wireless(WirelessEvent),
    Wallpaper(WallpaperEvent),
    Theme(ThemeEvent),
    Notification(NotificationEvent),

    // ── Shell lifecycle ────────────────────────────────────
    Shell(ShellLifecycleEvent),
}

// ── Battery ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub percentage: f64,           // 0.0–100.0
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
    LevelCritical(f64),   // threshold crossed (< 5%)
    LevelLow(f64),        // threshold crossed (< 20%)
    PluggedIn,
    Unplugged,
}

// ── Network ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkState {
    pub connectivity: NetworkConnectivity,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkConnectivity { None, Limited, Portal, Full }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub kind: InterfaceKind,
    pub ip4: Option<String>,
    pub ip6: Option<String>,
    pub state: InterfaceState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceKind { Ethernet, Wifi, Loopback, Vpn, Other(String) }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceState { Connected, Disconnected, Connecting, Failed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    StateChanged(NetworkState),
    InterfaceUp(NetworkInterface),
    InterfaceDown(String), // interface name
    ConnectivityChanged(NetworkConnectivity),
}

// ── Audio ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioState {
    pub sink_volume: f64,     // 0.0–1.5 (150% max)
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

// ── Wireless ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal_strength: u8, // 0–100
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

// ── Notification ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: Option<String>,
    pub urgency: NotificationUrgency,
    pub actions: Vec<(String, String)>, // (key, label)
    pub timestamp: DateTime<Utc>,
    pub expire_timeout: Option<i32>,   // ms, -1 = server default
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationUrgency { Low, Normal, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationEvent {
    Received(Notification),
    Closed { id: u32, reason: CloseReason },
    ActionInvoked { id: u32, action_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloseReason { Expired, Dismissed, AppClosed, Undefined }

// ── Window ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: String,        // compositor-assigned, opaque handle
    pub title: String,
    pub app_id: String,
    pub workspace: Option<String>,
    pub focused: bool,
    pub state: WindowState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowState { Normal, Minimized, Maximized, Fullscreen }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowEvent {
    Opened(WindowInfo),
    Closed(String),         // window id
    FocusChanged(String),   // window id
    TitleChanged { id: String, title: String },
    WorkspaceChanged { id: String, workspace: String },
    StateChanged { id: String, state: WindowState },
    WorkspaceList(Vec<String>),
    ActiveWorkspaceChanged(String),
}

// ── Wallpaper / Theme ──────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WallpaperEvent {
    Changed { path: String, output: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeEvent {
    Changed { name: String },
    ColorsUpdated(std::collections::HashMap<String, String>),
    DarkModeToggled(bool),
}

// ── Lifecycle ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShellLifecycleEvent {
    DaemonStarted,
    ServiceStarted(String),    // service name
    ServiceFailed { name: String, error: String },
    ShuttingDown,
    LockScreen,
    Unlock,
    Suspend,
    Hibernate,
}
```

### 4.2 In-Process Event Bus

```rust
// crates/rs-shell-core/src/bus.rs

use tokio::sync::broadcast;
use crate::events::ShellEvent;

const BUS_CAPACITY: usize = 512;

/// Global event bus. Clone the sender to publish; subscribe for a receiver.
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<ShellEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BUS_CAPACITY);
        Self { tx }
    }

    /// Publish an event. Returns number of active receivers (0 = no subscribers, event dropped).
    pub fn publish(&self, event: ShellEvent) -> usize {
        self.tx.send(event).unwrap_or(0)
    }

    /// Subscribe to all events.
    pub fn subscribe(&self) -> broadcast::Receiver<ShellEvent> {
        self.tx.subscribe()
    }

    /// Subscribe and filter by a predicate. Returns a filtered stream.
    pub fn subscribe_filtered<F>(
        &self,
        filter: F,
    ) -> impl futures::Stream<Item = ShellEvent>
    where
        F: Fn(&ShellEvent) -> bool + Send + 'static,
    {
        use tokio_stream::wrappers::BroadcastStream;
        use tokio_stream::StreamExt;

        BroadcastStream::new(self.subscribe())
            .filter_map(|r| futures::future::ready(r.ok()))
            .filter(move |e| futures::future::ready(filter(e)))
    }
}
```

### 4.3 Unix Socket IPC Protocol

All cross-process communication uses a length-prefixed JSON frame:

```
┌──────────┬─────────────────────────────────────────────┐
│ 4 bytes  │  N bytes                                    │
│ (BE u32) │  (JSON payload, UTF-8)                      │
└──────────┴─────────────────────────────────────────────┘
```

```rust
// crates/rs-shell-core/src/ipc.rs

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use anyhow::Result;

/// Wire message — sent/received by all shell processes.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IpcMessage {
    // ── Client → Daemon ──────────────────────────────────
    Subscribe { topics: Vec<String> },      // subscribe to event topics
    Unsubscribe { topics: Vec<String> },
    Command(ShellCommand),
    Ping,

    // ── Daemon → Client ──────────────────────────────────
    Event(crate::events::ShellEvent),
    Response(IpcResponse),
    Pong,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    OkWith(serde_json::Value),
    Error { code: u32, message: String },
}

/// Imperative commands clients can send.
#[derive(Debug, Serialize, Deserialize)]
pub enum ShellCommand {
    // Clipboard
    ClipboardList { limit: Option<usize> },
    ClipboardCopy { id: i64 },
    ClipboardDelete { id: i64 },
    ClipboardSearch { query: String },

    // Audio
    SetVolume { sink: Option<String>, volume: f64 },
    ToggleMute { sink: Option<String> },

    // Wallpaper
    SetWallpaper { path: String, output: Option<String> },

    // Theme
    SetTheme { name: String },
    ToggleDarkMode,

    // Window
    FocusWindow { id: String },
    CloseWindow { id: String },
    MoveToWorkspace { window_id: String, workspace: String },
    SwitchWorkspace { workspace: String },

    // Shell
    ShowLauncher,
    HideLauncher,
    ShowNotificationCenter,
    LockScreen,
    Suspend,
    Logout,
    Reload,   // hot-reload config
}

pub async fn send_message(stream: &mut UnixStream, msg: &IpcMessage) -> Result<()> {
    let payload = serde_json::to_vec(msg)?;
    let len = payload.len() as u32;
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&payload).await?;
    Ok(())
}

pub async fn recv_message(stream: &mut UnixStream) -> Result<IpcMessage> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;
    let mut payload = vec![0u8; len];
    stream.read_exact(&mut payload).await?;
    Ok(serde_json::from_slice(&payload)?)
}
```

### 4.4 Socket Paths (XDG-compliant)

```rust
// crates/rs-shell-core/src/paths.rs

use std::path::PathBuf;

pub fn runtime_dir() -> PathBuf {
    let uid = unsafe { libc::getuid() };
    PathBuf::from(format!("/run/user/{}", uid))
}

/// Main daemon socket (subscribable API)
pub fn daemon_socket() -> PathBuf {
    runtime_dir().join("rs-shell.sock")
}

/// Clipboard daemon socket (rsclip compatibility)
pub fn clipboard_socket() -> PathBuf {
    runtime_dir().join("rs-shell-clipboard.sock")
}

/// Bar process activation socket
pub fn bar_socket() -> PathBuf {
    runtime_dir().join("rs-shell-bar.sock")
}
```

---

## 5. Configuration Engine (`rs-shell-config`)

Single config file at `~/.config/rs-shell/config.toml`, hot-reloaded by all processes.

### 5.1 Config Schema

```toml
# ~/.config/rs-shell/config.toml

# ── General ─────────────────────────────────────────────────
[general]
wm = "auto"              # "hyprland" | "sway" | "niri" | "river" | "auto"
terminal = "foot"        # fallback terminal
file_manager = "thunar"
browser = "firefox"

# ── Bar / Panel ─────────────────────────────────────────────
[bar]
position = "top"         # "top" | "bottom"
height = 32
opacity = 0.92
font = "JetBrains Mono 10"
exclusive_zone = true    # reserve screen space

[bar.left]
modules = ["workspaces", "window-title"]

[bar.center]
modules = ["clock"]

[bar.right]
modules = ["systray", "audio", "network", "battery", "notifications"]

[bar.clock]
format = "%a %d %b  %H:%M"

# ── Clipboard ───────────────────────────────────────────────
[clipboard]
max_entries = 5000
max_text_bytes = 1048576
max_image_bytes = 10485760
dedupe = true
cleanup_unpinned_after_days = 30
auto_paste = true
paste_delay_ms = 140
paste_method = "wtype"   # "wtype" | "ydotool" | "xdotool"
favicon_cache = true

[clipboard.ui]
window_width = 920
window_height = 620
background_opacity = 0.70
preview_default = true

# ── Launcher ────────────────────────────────────────────────
[launcher]
width = 600
max_results = 12
show_icons = true
fuzzy = true
terminal_prefix = "foot -e"  # prefix for terminal apps

# ── Notifications ───────────────────────────────────────────
[notifications]
timeout_ms = 5000
timeout_critical_ms = 0   # 0 = never auto-dismiss
max_visible = 5
position = "top-right"    # "top-right" | "top-left" | "bottom-right" | "bottom-left"
gap = 8
width = 360

# ── Battery ─────────────────────────────────────────────────
[battery]
warn_level = 20.0
critical_level = 5.0
device = "auto"           # "auto" | specific UPower path

# ── Audio ───────────────────────────────────────────────────
[audio]
backend = "auto"          # "pipewire" | "pulseaudio" | "auto"
max_volume = 150          # max percent (100 or 150)
scroll_step = 5           # volume step on scroll

# ── Network ─────────────────────────────────────────────────
[network]
primary_interface = "auto"

# ── Wallpaper ───────────────────────────────────────────────
[wallpaper]
path = "~/.config/rs-shell/wallpaper.jpg"
mode = "fill"             # "fill" | "fit" | "center" | "tile"
backend = "internal"      # "internal" | "swww" | "swaybg" | "hyprpaper"

# ── Theme ───────────────────────────────────────────────────
[theme]
name = "nonchalant-dark"  # matches rsclip default
dark_mode = true
icon_theme = "Papirus-Dark"
gtk_theme = "adw-gtk3-dark"
font = "Inter 11"

[theme.colors]
# All optional — missing keys inherit from the named theme
accent    = "#7dcfff"
accent_text = "#1a1b26"

# ── Lock Screen ─────────────────────────────────────────────
[lockscreen]
command = "swaylock"      # external, rs-shell calls it
blur_background = true

# ── Keybinds (shell-level, not WM-level) ────────────────────
[keybinds]
# These are registered via wlr-keyboard-shortcuts-inhibit or the WM IPC
show_launcher      = "Super+Space"
show_clipboard     = "Super+V"
show_notifications = "Super+N"
screenshot         = "Print"
lock               = "Super+L"
```

### 5.2 Hot-Reload Engine

```rust
// crates/rs-shell-config/src/lib.rs

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Parsed config, cloneable and shared across threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    pub general: GeneralConfig,
    pub bar: BarConfig,
    pub clipboard: ClipboardConfig,
    pub launcher: LauncherConfig,
    pub notifications: NotificationConfig,
    pub battery: BatteryConfig,
    pub audio: AudioConfig,
    pub network: NetworkConfig,
    pub wallpaper: WallpaperConfig,
    pub theme: ThemeConfig,
    pub lockscreen: LockscreenConfig,
    pub keybinds: KeybindsConfig,
}

impl Default for ShellConfig {
    fn default() -> Self {
        // Parses assets/default-config.toml bundled at compile time
        toml::from_str(include_str!("../../assets/default-config.toml"))
            .expect("default config must be valid")
    }
}

pub struct ConfigManager {
    config: Arc<RwLock<ShellConfig>>,
    sender: watch::Sender<ShellConfig>,
    path: PathBuf,
}

impl ConfigManager {
    pub async fn new() -> Result<(Self, watch::Receiver<ShellConfig>)> {
        let path = config_path();
        let config = if path.exists() {
            load_config(&path).await?
        } else {
            let default = ShellConfig::default();
            // Write default config on first run
            tokio::fs::create_dir_all(path.parent().unwrap()).await?;
            tokio::fs::write(&path, toml::to_string_pretty(&default)?).await?;
            default
        };

        let (sender, receiver) = watch::channel(config.clone());
        let mgr = Self {
            config: Arc::new(RwLock::new(config)),
            sender,
            path,
        };

        Ok((mgr, receiver))
    }

    /// Start the file watcher loop (call in a tokio::spawn).
    pub async fn watch(self: Arc<Self>) -> Result<()> {
        let path = self.path.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if res.map(|e| e.kind.is_modify() || e.kind.is_create()).unwrap_or(false) {
                    let _ = tx.try_send(());
                }
            },
            notify::Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // Debounce 150ms
        loop {
            if rx.recv().await.is_none() { break; }
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            // Drain any extra events accumulated during debounce
            while rx.try_recv().is_ok() {}

            match load_config(&path).await {
                Ok(new_cfg) => {
                    *self.config.write().await = new_cfg.clone();
                    let _ = self.sender.send(new_cfg);
                    tracing::info!("config hot-reloaded");
                }
                Err(e) => tracing::warn!("config reload failed: {e}"),
            }
        }

        Ok(())
    }

    pub async fn get(&self) -> ShellConfig {
        self.config.read().await.clone()
    }
}

async fn load_config(path: &PathBuf) -> Result<ShellConfig> {
    let text = tokio::fs::read_to_string(path).await?;
    let mut cfg: ShellConfig = toml::from_str(&text)?;
    // Merge with defaults for any missing keys
    cfg.expand_paths();
    Ok(cfg)
}

fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".config/rs-shell/config.toml")
}
```

---

## 6. Clipboard Engine (`rs-shell-clipboard`)

This is rsclip with the rough edges sanded for shell integration. Key changes:

1. Emits typed `ClipboardEvent` into the shell event bus
2. Accepts `ShellCommand::Clipboard*` over the shared IPC socket
3. Config read from `[clipboard]` section of `rs-shell/config.toml`
4. Same SQLite schema + wl-paste watchers as rsclip

### Clipboard Event Types

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardEvent {
    Stored {
        id: i64,
        mime: String,
        kind: ClipEntryKind,
        preview: String,  // truncated text preview or "<image>"
    },
    Copied { id: i64 },
    Deleted { id: i64 },
    Cleared,
    PinnedToggled { id: i64, pinned: bool },
    SearchResult { query: String, results: Vec<ClipEntry> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipEntryKind { Text, Link, Color, Image, File, Secret }
```

### Integration Points

```
[rsclipd watch daemon]
        │ wl-paste --watch
        │
        ▼
[SQLite (XDG_DATA_HOME/rs-shell/clipboard.db)]
        │
        │ Unix datagram socket notification
        ▼
[Shell Event Bus] ──broadcast──► [Bar widgets, Clipboard UI, API subscribers]
```

The `rsclipd` binary is renamed `rs-shell-clipd` and the socket path is unified under the
shell runtime dir. The GTK4 overlay UI becomes `rs-shell-clipboard-ui` and subscribes to
the shared event bus instead of its own socket.

---

## 7. Subscribable API (`rs-shell-api`)

External tools (scripts, status bar plugins, custom widgets, CLI) subscribe to typed
shell events over two interfaces: a Unix socket and D-Bus.

### 7.1 Unix Socket Server

Clients connect, send a `Subscribe` message with a list of topics, then receive
a stream of matching `Event` messages.

```rust
// crates/rs-shell-api/src/unix_server.rs

use tokio::net::{UnixListener, UnixStream};
use std::collections::HashSet;
use rs_shell_core::{bus::EventBus, ipc::*};

pub async fn run_unix_server(bus: EventBus, socket: std::path::PathBuf) {
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).unwrap();

    loop {
        let (stream, _) = listener.accept().await.unwrap();
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
                    Ok(IpcMessage::Command(cmd)) => {
                        let resp = dispatch_command(cmd, &bus).await;
                        let _ = send_message(&mut stream, &IpcMessage::Response(resp)).await;
                    }
                    Ok(IpcMessage::Ping) => {
                        let _ = send_message(&mut stream, &IpcMessage::Pong).await;
                    }
                    Err(_) | Ok(IpcMessage::Pong) => break,
                    _ => {}
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

/// Topic name = snake_case variant name of ShellEvent
fn event_topic(event: &rs_shell_core::events::ShellEvent) -> &'static str {
    use rs_shell_core::events::ShellEvent::*;
    match event {
        Clipboard(_)    => "clipboard",
        Window(_)       => "window",
        Battery(_)      => "battery",
        Network(_)      => "network",
        Audio(_)        => "audio",
        Wireless(_)     => "wireless",
        Wallpaper(_)    => "wallpaper",
        Theme(_)        => "theme",
        Notification(_) => "notification",
        Shell(_)        => "shell",
    }
}
```

### 7.2 CLI Client (`rs-shell` binary, built from `rs-shell-daemon`)

```
# One-shot query
rs-shell battery
rs-shell audio volume
rs-shell network status

# Subscribe to events (streaming JSON to stdout)
rs-shell subscribe battery network
rs-shell subscribe '*'

# Commands
rs-shell set-volume 70
rs-shell toggle-mute
rs-shell set-wallpaper ~/Pictures/bg.jpg
rs-shell set-theme catppuccin-mocha
rs-shell switch-workspace 2
rs-shell show-launcher
rs-shell lock
```

### 7.3 D-Bus Interface

Using `zbus`, the daemon exports `org.rs_shell.Shell1` on the session bus.

```rust
// crates/rs-shell-api/src/dbus_server.rs

use zbus::{interface, Connection};

pub struct ShellInterface {
    bus: rs_shell_core::bus::EventBus,
}

#[interface(name = "org.rs_shell.Shell1")]
impl ShellInterface {
    // ── Properties ──────────────────────────────────────────

    #[zbus(property)]
    async fn battery_percentage(&self) -> f64 { /* read from service state */ 0.0 }

    #[zbus(property)]
    async fn volume(&self) -> f64 { /* read from service state */ 0.0 }

    #[zbus(property)]
    async fn muted(&self) -> bool { false }

    #[zbus(property)]
    async fn network_connectivity(&self) -> String { "Full".to_string() }

    #[zbus(property)]
    async fn active_workspace(&self) -> String { "1".to_string() }

    #[zbus(property)]
    async fn current_theme(&self) -> String { "nonchalant-dark".to_string() }

    // ── Methods ─────────────────────────────────────────────

    async fn set_volume(&self, volume: f64) -> zbus::fdo::Result<()> { Ok(()) }
    async fn toggle_mute(&self) -> zbus::fdo::Result<()> { Ok(()) }
    async fn set_wallpaper(&self, path: &str) -> zbus::fdo::Result<()> { Ok(()) }
    async fn set_theme(&self, name: &str) -> zbus::fdo::Result<()> { Ok(()) }
    async fn show_launcher(&self) -> zbus::fdo::Result<()> { Ok(()) }
    async fn lock_screen(&self) -> zbus::fdo::Result<()> { Ok(()) }
    async fn switch_workspace(&self, workspace: &str) -> zbus::fdo::Result<()> { Ok(()) }
    async fn reload_config(&self) -> zbus::fdo::Result<()> { Ok(()) }

    // ── Signals ─────────────────────────────────────────────

    #[zbus(signal)]
    async fn battery_changed(signal_ctxt: &zbus::SignalContext<'_>, percentage: f64) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn volume_changed(signal_ctxt: &zbus::SignalContext<'_>, volume: f64, muted: bool) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn network_changed(signal_ctxt: &zbus::SignalContext<'_>, connectivity: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn notification_received(signal_ctxt: &zbus::SignalContext<'_>, id: u32, app: &str, summary: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn theme_changed(signal_ctxt: &zbus::SignalContext<'_>, name: &str) -> zbus::Result<()>;
}
```

D-Bus bridge loop: watches the internal event bus and fires D-Bus signals:

```rust
pub async fn run_dbus_bridge(bus: EventBus, conn: &Connection) {
    let mut rx = bus.subscribe();
    let iface_ref = conn.object_server().interface::<_, ShellInterface>("/org/rs_shell/Shell1").await.unwrap();

    while let Ok(event) = rx.recv().await {
        let ctxt = iface_ref.signal_emitter();
        match event {
            ShellEvent::Battery(BatteryEvent::StateChanged(s)) => {
                let _ = ShellInterface::battery_changed(ctxt, s.percentage).await;
            }
            ShellEvent::Audio(AudioEvent::VolumeChanged { volume, .. }) => {
                let _ = ShellInterface::volume_changed(ctxt, volume, false).await;
            }
            ShellEvent::Notification(NotificationEvent::Received(n)) => {
                let _ = ShellInterface::notification_received(ctxt, n.id, &n.app_name, &n.summary).await;
            }
            _ => {}
        }
    }
}
```

---

## 8. Window Manager Bridge (`rs-shell-wm`)

A **trait-based plugin** architecture: one common `WmAdapter` trait, multiple implementations.

### 8.1 The Adapter Trait

```rust
// crates/rs-shell-wm/src/adapter.rs

use async_trait::async_trait;
use rs_shell_core::events::WindowEvent;

#[async_trait]
pub trait WmAdapter: Send + Sync + 'static {
    /// Emitter name for tracing/logging.
    fn name(&self) -> &str;

    /// Stream of window events. Runs until compositor disconnects.
    async fn event_stream(
        &self,
        tx: tokio::sync::broadcast::Sender<WindowEvent>,
    );

    /// List all open windows.
    async fn windows(&self) -> anyhow::Result<Vec<rs_shell_core::events::WindowInfo>>;

    /// Focus a window.
    async fn focus(&self, id: &str) -> anyhow::Result<()>;

    /// Close a window.
    async fn close(&self, id: &str) -> anyhow::Result<()>;

    /// Move window to workspace.
    async fn move_to_workspace(&self, id: &str, workspace: &str) -> anyhow::Result<()>;

    /// Switch active workspace.
    async fn switch_workspace(&self, workspace: &str) -> anyhow::Result<()>;

    /// List workspace names.
    async fn workspaces(&self) -> anyhow::Result<Vec<String>>;
}
```

### 8.2 Generic wlr-foreign-toplevel Adapter

Works with any wlroots compositor:

```rust
// crates/rs-shell-wm/src/wlr_toplevel.rs

use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1,
    zwlr_foreign_toplevel_manager_v1,
};
use wayland_client::{Connection, Dispatch, QueueHandle};

// The wlr_foreign_toplevel adapter tracks window opens/closes/focus
// by binding to zwlr_foreign_toplevel_manager_v1 and dispatching
// zwlr_foreign_toplevel_handle_v1 events into WindowEvent::* variants.
// Full implementation handles: title, app_id, state (maximized/fullscreen/
// minimized/activated), and output association.
```

### 8.3 Hyprland Adapter

Hyprland exposes a Unix socket IPC:

```rust
// crates/rs-shell-wm/src/hyprland.rs

use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::Value;

pub struct HyprlandAdapter;

impl HyprlandAdapter {
    fn socket_path(n: u8) -> std::path::PathBuf {
        let his = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").unwrap();
        let uid = unsafe { libc::getuid() };
        std::path::PathBuf::from(format!(
            "/run/user/{uid}/hypr/{his}/.socket{n}.sock"
        ))
    }

    /// Send a command to socket 1 (request/response).
    async fn command(&self, cmd: &str) -> anyhow::Result<String> {
        let mut stream = UnixStream::connect(Self::socket_path(1)).await?;
        stream.write_all(cmd.as_bytes()).await?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).await?;
        Ok(String::from_utf8(buf)?)
    }
}

#[async_trait::async_trait]
impl super::adapter::WmAdapter for HyprlandAdapter {
    fn name(&self) -> &str { "hyprland" }

    async fn event_stream(
        &self,
        tx: tokio::sync::broadcast::Sender<rs_shell_core::events::WindowEvent>,
    ) {
        // Socket 2: event socket, line-delimited "event>>data\n"
        let mut stream = UnixStream::connect(Self::socket_path(2)).await.unwrap();
        let mut line = String::new();
        let mut reader = tokio::io::BufReader::new(stream);
        use tokio::io::AsyncBufReadExt;

        loop {
            line.clear();
            if reader.read_line(&mut line).await.unwrap_or(0) == 0 { break; }
            if let Some(event) = parse_hyprland_event(line.trim()) {
                let _ = tx.send(event);
            }
        }
    }

    async fn windows(&self) -> anyhow::Result<Vec<rs_shell_core::events::WindowInfo>> {
        let json = self.command("j/clients").await?;
        let clients: Vec<Value> = serde_json::from_str(&json)?;
        Ok(clients.iter().filter_map(hyprland_client_to_window).collect())
    }

    async fn focus(&self, id: &str) -> anyhow::Result<()> {
        self.command(&format!("dispatch focuswindow address:{id}")).await?;
        Ok(())
    }

    async fn close(&self, id: &str) -> anyhow::Result<()> {
        self.command(&format!("dispatch closewindow address:{id}")).await?;
        Ok(())
    }

    async fn switch_workspace(&self, workspace: &str) -> anyhow::Result<()> {
        self.command(&format!("dispatch workspace {workspace}")).await?;
        Ok(())
    }

    async fn workspaces(&self) -> anyhow::Result<Vec<String>> {
        let json = self.command("j/workspaces").await?;
        let ws: Vec<Value> = serde_json::from_str(&json)?;
        Ok(ws.iter().filter_map(|w| w["name"].as_str().map(String::from)).collect())
    }

    // move_to_workspace impl similarly...
}

fn parse_hyprland_event(line: &str) -> Option<rs_shell_core::events::WindowEvent> {
    use rs_shell_core::events::{WindowEvent, WindowInfo, WindowState};
    let (event, data) = line.split_once(">>")?;
    match event {
        "activewindowv2" => Some(WindowEvent::FocusChanged(data.to_string())),
        "openwindow" => {
            // data: "address,workspacename,class,title"
            let parts: Vec<&str> = data.splitn(4, ',').collect();
            if parts.len() == 4 {
                Some(WindowEvent::Opened(WindowInfo {
                    id: parts[0].to_string(),
                    title: parts[3].to_string(),
                    app_id: parts[2].to_string(),
                    workspace: Some(parts[1].to_string()),
                    focused: false,
                    state: WindowState::Normal,
                }))
            } else { None }
        }
        "closewindow"    => Some(WindowEvent::Closed(data.to_string())),
        "workspace"      => Some(WindowEvent::ActiveWorkspaceChanged(data.to_string())),
        "createworkspace"| "destroyworkspace" => None, // handled separately
        _ => None,
    }
}
```

### 8.4 sway Adapter

```rust
// sway IPC: Unix socket, 14-byte header + payload
// MAGIC: b"i3-ipc" + 4-byte payload length + 4-byte message type
// Message types: 0=run_command, 1=get_workspaces, 2=subscribe, 4=get_tree ...
// Event types: 0x80000000|0=workspace, 0x80000000|3=window

pub struct SwayAdapter;
// Implementation follows sway's i3-compatible IPC spec with
// event_stream subscribing to ["window", "workspace"] events.
```

### 8.5 Auto-Detection

```rust
// crates/rs-shell-wm/src/lib.rs

pub fn detect_wm() -> Box<dyn WmAdapter> {
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() {
        return Box::new(HyprlandAdapter);
    }
    if std::env::var("SWAYSOCK").is_ok() {
        return Box::new(SwayAdapter);
    }
    if std::env::var("NIRI_SOCKET").is_ok() {
        return Box::new(NiriAdapter);
    }
    // Generic fallback: wlr-foreign-toplevel via Wayland protocol
    Box::new(WlrToplevelAdapter::new())
}
```

---

## 9. Service Layer (`rs-shell-services`)

Each service is a `tokio::task` that polls/subscribes to a system resource and feeds
typed events into the shared `EventBus`.

### 9.1 Battery Service (UPower D-Bus)

```rust
// crates/rs-shell-services/src/battery.rs

use zbus::Connection;
use rs_shell_core::{bus::EventBus, events::{ShellEvent, BatteryEvent, BatteryState, BatteryStatus}};

pub async fn run(bus: EventBus, config: rs_shell_config::BatteryConfig) -> anyhow::Result<()> {
    let conn = Connection::system().await?;

    // Find the display device (or configured device)
    let device_path = if config.device == "auto" {
        find_display_device(&conn).await?
    } else {
        config.device.clone()
    };

    // Subscribe to PropertiesChanged on the UPower device
    use zbus::proxy;
    use futures::StreamExt;

    // Full implementation:
    // 1. Create UPowerDeviceProxy for the device path
    // 2. Subscribe to PropertiesChanged signals
    // 3. On each change, read Percentage, State, TimeToEmpty, TimeToFull
    // 4. Emit BatteryEvent::StateChanged into the bus
    // 5. Check for low/critical thresholds and emit BatteryEvent::LevelLow / LevelCritical
    // 6. On State → Charging: emit BatteryEvent::PluggedIn
    // 7. On State → Discharging: emit BatteryEvent::Unplugged

    let mut state_stream = /* UPower PropertiesChanged stream */ futures::stream::empty::<()>();

    while let Some(_) = state_stream.next().await {
        let state = read_battery_state(&conn, &device_path).await?;
        bus.publish(ShellEvent::Battery(BatteryEvent::StateChanged(state.clone())));

        if state.percentage <= config.critical_level {
            bus.publish(ShellEvent::Battery(BatteryEvent::LevelCritical(state.percentage)));
        } else if state.percentage <= config.warn_level {
            bus.publish(ShellEvent::Battery(BatteryEvent::LevelLow(state.percentage)));
        }
    }

    Ok(())
}
```

### 9.2 Network Service (NetworkManager D-Bus)

```rust
// crates/rs-shell-services/src/network.rs

// D-Bus: org.freedesktop.NetworkManager
// Key signals:
//   - StateChanged(state: u32) on /org/freedesktop/NetworkManager
//   - DeviceAdded / DeviceRemoved
//   - ActiveConnection PropertiesChanged
//
// Connectivity states (NM):
//   0=Unknown, 1=None, 2=Portal, 3=Limited, 4=Full
//
// Translates NM state → NetworkState and emits NetworkEvent variants.
// Also exposes WifiDeviceProxy for WirelessEvent::NetworksScanned.

pub async fn run(bus: EventBus, _config: rs_shell_config::NetworkConfig) -> anyhow::Result<()> {
    let conn = Connection::system().await?;
    // Subscribe to org.freedesktop.NetworkManager.StateChanged signal
    // and translate NM_STATE_* values to NetworkConnectivity.
    //
    // For interface details, iterate ActiveConnections and resolve each
    // ActiveConnection's Devices to get IP config.
    Ok(())
}
```

### 9.3 Audio Service (PipeWire via pipewire-rs)

```rust
// crates/rs-shell-services/src/audio.rs
// Dependency: pipewire = "0.8" (Rust bindings for libpipewire)
//
// Architecture:
//   - Connect to the PipeWire daemon via MainLoop + Context
//   - Register a Registry listener to track Node objects (sinks/sources)
//   - Track the default sink (via metadata "default.audio.sink")
//   - Subscribe to node param changes for volumes
//   - Emit AudioEvent::VolumeChanged / MuteToggled / DefaultSinkChanged
//
// PulseAudio fallback: libpulse-binding crate, same event model.

use pipewire::{MainLoop, Context};

pub async fn run(bus: EventBus, config: rs_shell_config::AudioConfig) -> anyhow::Result<()> {
    // PipeWire's API is not async-native; run it in a dedicated OS thread
    // and channel events back via tokio::sync::mpsc.
    let (pw_tx, mut pw_rx) = tokio::sync::mpsc::unbounded_channel();

    std::thread::spawn(move || {
        let main_loop = MainLoop::new(None).unwrap();
        let context = Context::new(&main_loop).unwrap();
        let core = context.connect(None).unwrap();
        let registry = core.get_registry().unwrap();

        // Registry listener: track Node, Metadata objects
        let _registry_listener = registry
            .add_listener_local()
            .global(move |obj| {
                // Detect audio sink/source nodes, subscribe to param changes
                // Forward volume/mute changes via pw_tx
            })
            .register();

        main_loop.run();
    });

    // Forward PipeWire events to the shell event bus
    while let Some(audio_event) = pw_rx.recv().await {
        bus.publish(ShellEvent::Audio(audio_event));
    }

    Ok(())
}
```

### 9.4 Wireless Service

Wireless is a specialized view on NetworkManager's WiFi devices:

```rust
// crates/rs-shell-services/src/wireless.rs
//
// Uses org.freedesktop.NetworkManager.Device.Wireless interface:
//   - PropertiesChanged for ActiveAccessPoint
//   - AccessPointAdded / AccessPointRemoved
//   - RequestScan() for manual scan trigger
//
// AccessPoint object provides: Ssid, Strength, Flags (security), Frequency
//
// Emits: WirelessEvent::Connected, Disconnected, SignalStrengthChanged, NetworksScanned
```

### 9.5 Notification Service (org.freedesktop.Notifications)

rs-shell implements the full notification server, replacing dunst/mako.

```rust
// crates/rs-shell-services/src/notifications.rs

use zbus::{interface, SignalContext};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use rs_shell_core::events::{Notification, NotificationUrgency};

static NEXT_ID: AtomicU32 = AtomicU32::new(1);

pub struct NotificationServer {
    bus: rs_shell_core::bus::EventBus,
    history: Arc<Mutex<Vec<Notification>>>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationServer {
    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".into(),
            "body-markup".into(),
            "icon-static".into(),
            "actions".into(),
            "persistence".into(),
            "action-icons".into(),
        ]
    }

    fn notify(
        &self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<&str>,
        hints: std::collections::HashMap<&str, zvariant::Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            NEXT_ID.fetch_add(1, Ordering::SeqCst)
        };

        let urgency = hints.get("urgency")
            .and_then(|v| v.downcast_ref::<u8>())
            .map(|u| match u {
                0 => NotificationUrgency::Low,
                2 => NotificationUrgency::Critical,
                _ => NotificationUrgency::Normal,
            })
            .unwrap_or(NotificationUrgency::Normal);

        let notif = Notification {
            id,
            app_name: app_name.to_string(),
            summary: summary.to_string(),
            body: body.to_string(),
            icon: if app_icon.is_empty() { None } else { Some(app_icon.to_string()) },
            urgency,
            actions: actions.chunks(2).filter_map(|c| {
                Some((c.get(0)?.to_string(), c.get(1)?.to_string()))
            }).collect(),
            timestamp: chrono::Utc::now(),
            expire_timeout: if expire_timeout == -1 { None } else { Some(expire_timeout) },
        };

        self.history.lock().push(notif.clone());
        self.bus.publish(ShellEvent::Notification(
            rs_shell_core::events::NotificationEvent::Received(notif)
        ));

        id
    }

    fn close_notification(&self, id: u32) {
        use rs_shell_core::events::{NotificationEvent, CloseReason};
        self.bus.publish(ShellEvent::Notification(
            NotificationEvent::Closed { id, reason: CloseReason::AppClosed }
        ));
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        ("rs-shell", "rs-shell project", env!("CARGO_PKG_VERSION"), "1.2")
    }

    #[zbus(signal)]
    async fn notification_closed(ctxt: &SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn action_invoked(ctxt: &SignalContext<'_>, id: u32, action_key: &str) -> zbus::Result<()>;
}

pub async fn run(bus: rs_shell_core::bus::EventBus) -> anyhow::Result<()> {
    let conn = zbus::ConnectionBuilder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", NotificationServer {
            bus,
            history: Arc::new(Mutex::new(Vec::new())),
        })?
        .build()
        .await?;

    // Keep connection alive
    std::future::pending::<()>().await;
    Ok(())
}
```

### 9.6 Wallpaper Service

```rust
// crates/rs-shell-services/src/wallpaper.rs

use rs_shell_config::WallpaperConfig;
use rs_shell_core::events::{ShellEvent, WallpaperEvent};

pub async fn apply(config: &WallpaperConfig, bus: &rs_shell_core::bus::EventBus) -> anyhow::Result<()> {
    let path = expand_tilde(&config.path);

    match config.backend.as_str() {
        "swww" => {
            // swww img <path> --transition-type any --transition-fps 60
            tokio::process::Command::new("swww")
                .args(["img", &path,
                       "--transition-type", "any",
                       "--transition-fps", "60"])
                .status().await?;
        }
        "swaybg" => {
            // Kill existing swaybg, start new one
            let _ = tokio::process::Command::new("pkill").arg("swaybg").status().await;
            tokio::process::Command::new("swaybg")
                .args(["-i", &path, "-m", &config.mode])
                .spawn()?;
        }
        "hyprpaper" => {
            // Write hyprpaper.conf and send IPC
            // hyprctl hyprpaper wallpaper ",<path>"
            tokio::process::Command::new("hyprctl")
                .args(["hyprpaper", "wallpaper", &format!(",{path}")])
                .status().await?;
        }
        "internal" | _ => {
            // GTK4 GtkPicture-based rendering on a layer-shell surface at layer BACKGROUND
            // Launched as a separate lightweight process: rs-shell-wallpaper
        }
    }

    bus.publish(ShellEvent::Wallpaper(WallpaperEvent::Changed {
        path,
        output: None,
    }));

    Ok(())
}
```

### 9.7 Theme Service

```rust
// crates/rs-shell-services/src/theme.rs

use rs_shell_config::ThemeConfig;
use rs_shell_core::events::{ShellEvent, ThemeEvent};
use std::path::PathBuf;

const BUNDLED_THEMES: &[(&str, &str)] = &[
    ("nonchalant-dark",   include_str!("../../../assets/themes/nonchalant-dark.css")),
    ("catppuccin-mocha",  include_str!("../../../assets/themes/catppuccin-mocha.css")),
    ("catppuccin-latte",  include_str!("../../../assets/themes/catppuccin-latte.css")),
    ("tokyo-night",       include_str!("../../../assets/themes/tokyo-night.css")),
    ("dracula",           include_str!("../../../assets/themes/dracula.css")),
    ("nord",              include_str!("../../../assets/themes/nord.css")),
    ("gruvbox-dark",      include_str!("../../../assets/themes/gruvbox-dark.css")),
    ("rose-pine",         include_str!("../../../assets/themes/rose-pine.css")),
];

pub struct ThemeManager {
    config: ThemeConfig,
    bus: rs_shell_core::bus::EventBus,
}

impl ThemeManager {
    pub async fn apply(&self) -> anyhow::Result<()> {
        let css = self.resolve_css()?;
        // Write to ~/.config/rs-shell/active-theme.css
        let out = dirs_next::config_dir()
            .unwrap()
            .join("rs-shell/active-theme.css");
        tokio::fs::write(&out, &css).await?;

        // Set GTK theme via gsettings (affects GTK apps system-wide)
        if !self.config.gtk_theme.is_empty() {
            let _ = tokio::process::Command::new("gsettings")
                .args(["set", "org.gnome.desktop.interface", "gtk-theme", &self.config.gtk_theme])
                .status().await;
        }
        if !self.config.icon_theme.is_empty() {
            let _ = tokio::process::Command::new("gsettings")
                .args(["set", "org.gnome.desktop.interface", "icon-theme", &self.config.icon_theme])
                .status().await;
        }

        // Parse and broadcast CSS variables as a colors map
        let colors = extract_css_vars(&css);
        self.bus.publish(ShellEvent::Theme(ThemeEvent::Changed { name: self.config.name.clone() }));
        self.bus.publish(ShellEvent::Theme(ThemeEvent::ColorsUpdated(colors)));

        Ok(())
    }

    fn resolve_css(&self) -> anyhow::Result<String> {
        // 1. Check user theme dir
        let user_theme = dirs_next::config_dir()
            .unwrap()
            .join(format!("rs-shell/themes/{}.css", self.config.name));
        if user_theme.exists() {
            return Ok(std::fs::read_to_string(user_theme)?);
        }

        // 2. Check bundled themes
        for (name, css) in BUNDLED_THEMES {
            if *name == self.config.name.as_str() {
                // Overlay user color overrides
                return Ok(apply_color_overrides(css, &self.config.colors));
            }
        }

        anyhow::bail!("theme '{}' not found", self.config.name)
    }
}
```

---

## 10. UI Layer (`rs-shell-ui`)

All UI surfaces use `gtk4-layer-shell` to anchor them to the Wayland compositor layer.

### 10.1 GTK CSS Variable System (inheriting rsclip's `nonchalant-dark`)

```css
/* assets/themes/nonchalant-dark.css — extends rsclip's design language */
:root {
  --shell-bg:          rgba(22, 22, 30, 0.92);
  --shell-surface:     rgba(32, 32, 42, 0.95);
  --shell-surface-2:   rgba(40, 40, 52, 0.98);
  --shell-border:      rgba(80, 80, 110, 0.4);
  --shell-accent:      #7dcfff;
  --shell-accent-text: #1a1b26;
  --shell-text:        #c0caf5;
  --shell-text-dim:    #9aa5ce;
  --shell-text-muted:  #565f89;
  --shell-red:         #f7768e;
  --shell-green:       #9ece6a;
  --shell-yellow:      #e0af68;
  --shell-orange:      #ff9e64;
  --shell-magenta:     #bb9af7;
  --shell-cyan:        #7dcfff;
  --shell-radius:      10px;
  --shell-radius-sm:   6px;
  --shell-padding:     12px;
  --shell-gap:         8px;
  --shell-font:        "JetBrains Mono";
  --shell-font-ui:     "Inter";
  --bar-height:        32px;
  --bar-bg:            rgba(18, 18, 24, 0.88);
  --bar-padding:       0 8px;
}
```

### 10.2 Panel / Bar

The bar is a separate binary (`rs-shell-bar`) for isolation. It opens a
`gtk4-layer-shell` surface anchored top (or bottom), exclusive zone equal to bar height.

```rust
// crates/rs-shell-bar/src/main.rs

use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use rs_shell_core::events::ShellEvent;

pub fn build_bar(app: &gtk4::Application, bus_receiver: tokio::sync::broadcast::Receiver<ShellEvent>) {
    let window = gtk4::ApplicationWindow::new(app);

    // Layer shell setup
    window.init_layer_shell();
    window.set_layer(Layer::Top);
    window.set_anchor(Edge::Left,   true);
    window.set_anchor(Edge::Right,  true);
    window.set_anchor(Edge::Top,    true); // or Bottom
    window.set_exclusive_zone(32);         // bar height from config
    window.set_namespace("rs-shell-bar");

    // CSS
    apply_theme_css(&window);

    // Layout
    let bar = gtk4::CenterBox::new();
    bar.add_css_class("bar");

    // Left: workspaces + title
    let left = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    left.append(&WorkspacesWidget::new());
    left.append(&WindowTitleWidget::new());
    bar.set_start_widget(Some(&left));

    // Center: clock
    bar.set_center_widget(Some(&ClockWidget::new()));

    // Right: systray, audio, network, battery, notifications bell
    let right = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
    right.append(&SystrayWidget::new());
    right.append(&AudioWidget::new());
    right.append(&NetworkWidget::new());
    right.append(&BatteryWidget::new());
    right.append(&NotificationBellWidget::new());
    bar.set_end_widget(Some(&right));

    window.set_child(Some(&bar));
    window.present();

    // Spawn event watcher
    glib::spawn_future_local(async move {
        update_bar_from_events(bus_receiver, &bar_widgets).await;
    });
}
```

### 10.3 Bar Widgets

Each widget is a GTK4 widget that subscribes to relevant events:

```
WorkspacesWidget     → WindowEvent::WorkspaceList, ActiveWorkspaceChanged
WindowTitleWidget    → WindowEvent::FocusChanged, TitleChanged
ClockWidget          → tick timer (1-second interval via glib::timeout_add_seconds)
BatteryWidget        → BatteryEvent::StateChanged
AudioWidget          → AudioEvent::VolumeChanged, MuteToggled
NetworkWidget        → NetworkEvent::ConnectivityChanged, WirelessEvent::SignalStrengthChanged
NotificationBellWidget → NotificationEvent::Received, Closed
SystrayWidget        → StatusNotifierItem D-Bus watcher (org.kde.StatusNotifierWatcher)
```

### 10.4 App Launcher

```rust
// GTK4 layer-shell surface, Layer::Overlay, centered on screen.
// Components:
//   1. SearchEntry (focused immediately on show)
//   2. ResultsList (virtual-scrolling GtkListView with GtkSignalListItemFactory)
//   3. ItemWidget: icon + app name + description

// App data source: .desktop files from XDG_DATA_DIRS/applications
// Indexed at startup, updated on directory change via inotify.
// Fuzzy matching: custom scorer (trigram similarity + exec bonus).

// Keyboard: Up/Down to navigate, Enter to launch, Escape to close.
// Mouse: click to launch, double-click for terminal apps.
```

### 10.5 Notification OSD

```rust
// Layer: Overlay, anchored top-right (configurable).
// Each notification is a GTK4 widget with:
//   - App icon (themed via icon-theme)
//   - App name + timestamp
//   - Summary (bold)
//   - Body (markup supported via gtk4::Label::set_use_markup)
//   - Action buttons
//   - Dismiss X button
//   - Timeout progress bar (for non-persistent notifications)
//
// Stacking: new notifications animate in from the edge, older ones shift down.
// Click on body → invoke default action.
// Swipe right (gesture) → dismiss.

// Notification Center drawer:
//   - Layer::Overlay, right-anchored, full-height
//   - Shows history of last N notifications (from SQLite log)
//   - "Clear all" button
//   - Per-notification dismiss
```

### 10.6 Quick Settings Panel

```rust
// Triggered by clicking the bar's right section or Super+S.
// Layer::Overlay, top-right anchored, below the bar.
// Sections:
//   ┌─────────────────────────────────┐
//   │ 🔊 Volume         [slider]      │
//   │ 💡 Brightness     [slider]      │
//   │ 🌐 Network        [button/list] │
//   │ 🔋 Battery        [status]      │
//   │ 🎨 Theme          [toggle]      │
//   │ 🖥  Display       [button]      │
//   │ 🔒 Lock  🛑 Logout ⏻ Power     │
//   └─────────────────────────────────┘
```

---

## 11. Shell Daemon (`rs-shell-daemon`)

The orchestrator binary `rs-shelld` starts all services and hosts the API.

```rust
// crates/rs-shell-daemon/src/main.rs

use rs_shell_core::bus::EventBus;
use rs_shell_config::ConfigManager;
use rs_shell_api::{unix_server, dbus_server};
use rs_shell_services::{battery, network, audio, wireless, wallpaper, notifications};
use rs_shell_wm::detect_wm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // ── Config ───────────────────────────────────────────────────────────
    let (config_manager, config_rx) = ConfigManager::new().await?;
    let config_manager = std::sync::Arc::new(config_manager);
    let cfg = config_manager.get().await;

    // Start config hot-reload watcher
    let cm = config_manager.clone();
    tokio::spawn(async move { cm.watch().await });

    // ── Event Bus ────────────────────────────────────────────────────────
    let bus = EventBus::new();

    // ── WM Adapter ───────────────────────────────────────────────────────
    let wm: Box<dyn rs_shell_wm::WmAdapter> = if cfg.general.wm == "auto" {
        rs_shell_wm::detect_wm()
    } else {
        rs_shell_wm::adapter_for(&cfg.general.wm)
    };

    let (wm_tx, _) = tokio::sync::broadcast::channel(256);
    let wm_bus = bus.clone();
    let wm_tx2 = wm_tx.clone();
    tokio::spawn(async move {
        wm.event_stream(wm_tx2).await;
    });
    // Bridge WM events → ShellEvent::Window
    {
        let mut rx = wm_tx.subscribe();
        let b = bus.clone();
        tokio::spawn(async move {
            while let Ok(ev) = rx.recv().await {
                b.publish(rs_shell_core::events::ShellEvent::Window(ev));
            }
        });
    }

    // ── Services ─────────────────────────────────────────────────────────
    let services: Vec<(&str, tokio::task::JoinHandle<anyhow::Result<()>>)> = vec![
        ("battery",       tokio::spawn(battery::run(bus.clone(), cfg.battery.clone()))),
        ("network",       tokio::spawn(network::run(bus.clone(), cfg.network.clone()))),
        ("audio",         tokio::spawn(audio::run(bus.clone(), cfg.audio.clone()))),
        ("wireless",      tokio::spawn(wireless::run(bus.clone()))),
        ("notifications", tokio::spawn(notifications::run(bus.clone()))),
        ("clipboard",     tokio::spawn(rs_shell_clipboard::run(bus.clone(), cfg.clipboard.clone()))),
    ];

    // Apply wallpaper at startup
    {
        let b = bus.clone();
        let wc = cfg.wallpaper.clone();
        tokio::spawn(async move { rs_shell_services::wallpaper::apply(&wc, &b).await });
    }

    // Apply theme at startup
    {
        let b = bus.clone();
        let tc = cfg.theme.clone();
        tokio::spawn(async move {
            rs_shell_services::theme::ThemeManager { config: tc, bus: b }.apply().await
        });
    }

    // Watch config changes and re-apply wallpaper/theme
    {
        let b = bus.clone();
        let mut crx = config_rx;
        tokio::spawn(async move {
            while crx.changed().await.is_ok() {
                let new_cfg = crx.borrow().clone();
                rs_shell_services::wallpaper::apply(&new_cfg.wallpaper, &b).await.ok();
                rs_shell_services::theme::ThemeManager { config: new_cfg.theme, bus: b.clone() }
                    .apply().await.ok();
            }
        });
    }

    // ── API Servers ──────────────────────────────────────────────────────
    let socket_path = rs_shell_core::paths::daemon_socket();
    tokio::spawn(unix_server::run_unix_server(bus.clone(), socket_path));
    tokio::spawn(dbus_server::run_dbus_server(bus.clone()));

    bus.publish(rs_shell_core::events::ShellEvent::Shell(
        rs_shell_core::events::ShellLifecycleEvent::DaemonStarted
    ));

    tracing::info!("rs-shelld started");

    // Monitor service tasks
    for (name, handle) in services {
        let b = bus.clone();
        tokio::spawn(async move {
            match handle.await {
                Ok(Err(e)) => {
                    tracing::error!("service {} failed: {e}", name);
                    b.publish(rs_shell_core::events::ShellEvent::Shell(
                        rs_shell_core::events::ShellLifecycleEvent::ServiceFailed {
                            name: name.to_string(),
                            error: e.to_string(),
                        }
                    ));
                }
                Err(e) => tracing::error!("service {} panicked: {e}", name),
                Ok(Ok(())) => tracing::info!("service {} exited cleanly", name),
            }
        });
    }

    // Block forever
    std::future::pending::<()>().await;
    Ok(())
}
```

---

## 12. Theme System

### Theme File Structure

```css
/* assets/themes/<name>.css */

/* 1. CSS variables (consumed by rs-shell itself) */
:root {
  --shell-bg:     rgba(22, 22, 30, 0.92);
  --shell-accent: #7dcfff;
  /* ... all shell variables ... */
}

/* 2. GTK widget overrides */
.bar                { background: var(--bar-bg); }
.bar-module         { padding: 0 8px; color: var(--shell-text); }
.battery-critical   { color: var(--shell-red); }
.notification       { background: var(--shell-surface); border-radius: var(--shell-radius); }
.launcher-entry     { background: var(--shell-surface-2); }
.launcher-result    { padding: 8px; }
.launcher-result:selected { background: var(--shell-accent); color: var(--shell-accent-text); }
```

### Runtime CSS Application

```rust
// crates/rs-shell-ui/src/theme.rs

use gtk4::{CssProvider, gdk::Display, StyleContext};

pub fn apply_css_to_screen(css: &str) {
    let provider = CssProvider::new();
    provider.load_from_data(css);
    StyleContext::add_provider_for_display(
        &Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

pub fn watch_and_reload_theme(mut config_rx: tokio::sync::watch::Receiver<rs_shell_config::ShellConfig>) {
    glib::spawn_future_local(async move {
        while config_rx.changed().await.is_ok() {
            let css = {
                let cfg = config_rx.borrow();
                resolve_theme_css(&cfg.theme)
            };
            apply_css_to_screen(&css);
        }
    });
}
```

---

## 13. Security Model

### Process Isolation

```
rs-shelld (daemon, no UI, no GTK)
│
├── Clipboard: access to wl-paste, SQLite, network (favicons only)
├── Battery/Network: read-only D-Bus system bus
├── Audio: PipeWire client socket
├── Notifications: org.freedesktop.Notifications D-Bus name
└── API: Unix socket at /run/user/UID/rs-shell.sock (mode 0600)

rs-shell-bar (GTK process, no D-Bus system access)
│
└── Connects to rs-shelld via Unix socket
    Receives events, sends commands only

rs-shell-launcher (spawned on demand, short-lived)
rs-shell-clipboard-ui (resident but idle)
```

### Clipboard Security

- **Secret entries**: stored AES-256-GCM encrypted in SQLite (key in libsecret / kernel keyring)
- **Auto-clear**: secret entries cleared from the Wayland clipboard after configurable timeout
- **MIME allowlist**: configurable per-application MIME type blocking
- **History cap**: prevents unbounded growth; user-configurable

### IPC Security

- Unix socket uses `fchown`/`chmod 0600` — only owner UID can connect
- D-Bus service: session bus only, no system-bus privileged operations
- Commands validated against an allowlist before dispatch
- Rate limiting: max 100 commands/second per client connection

### Systemd Integration

```ini
# packaging/systemd/rs-shelld.service
[Unit]
Description=rs-shell Desktop Shell Daemon
PartOf=graphical-session.target
After=graphical-session.target
ConditionEnvironment=WAYLAND_DISPLAY

[Service]
Type=notify
ExecStart=%h/.cargo/bin/rs-shelld
Restart=on-failure
RestartSec=3
# Hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=%h/.config/rs-shell %h/.local/share/rs-shell /run/user/%U

[Install]
WantedBy=graphical-session.target
```

---

## 14. Performance Architecture

### Startup Sequence

```
t=0ms   rs-shelld starts, reads config
t=5ms   Event bus ready, socket created (systemd sd_notify READY=1)
t=15ms  Battery, Network service D-Bus connections established
t=20ms  Audio PipeWire connection established
t=30ms  Notification server D-Bus name acquired
t=40ms  rs-shell-bar process starts (separate, GTK init)
t=80ms  Bar window visible, first paint
t=90ms  Wallpaper applied (swaybg/swww launched)
t=100ms Launcher pre-indexes .desktop files in background thread
```

### Memory Budget

| Component | Target RSS |
|-----------|-----------|
| rs-shelld | < 12 MB |
| rs-shell-bar | < 25 MB |
| rs-shell-clipboard-ui (idle) | < 18 MB |
| rs-shell-launcher (resident) | < 20 MB |
| **Total** | **< 75 MB** |

### Key Perf Decisions

- **Virtual scrolling** in all list widgets: same technique as rsclip v0.1.9 — only render visible rows
- **GtkListView + GtkSingleSelection** over GtkListBox (10× faster for large lists)
- **Incremental .desktop indexing**: index entries as discovered, show partial results immediately
- **Broadcast channel capacity**: 512 events; back-pressure prevents slow consumers from stalling the bus
- **Config hot-reload debounce**: 150ms — same as rsclip — prevents thrash on rapid saves
- **PipeWire thread isolation**: PipeWire main loop runs in a dedicated OS thread; GTK never blocks on audio
- **LTO + codegen-units=1**: fat LTO in release builds for cross-crate inlining

---

## 15. Build System & Packaging

### Cargo xtask

```rust
// xtask/src/main.rs
// cargo xtask install   — build + install systemd units + binaries
// cargo xtask package   — build release archive for AUR
// cargo xtask test-e2e  — run end-to-end tests in a sway subprocess
// cargo xtask clean-all — clean + remove runtime files
```

### Feature Flags

```toml
# crates/rs-shell-daemon/Cargo.toml
[features]
default    = ["pipewire", "networkmanager"]
pipewire   = ["rs-shell-services/pipewire"]
pulseaudio = ["rs-shell-services/pulseaudio"]
networkmanager = ["rs-shell-services/networkmanager"]
iwd        = ["rs-shell-services/iwd"]   # alternative wifi backend
swww       = ["rs-shell-services/swww"]
```

### AUR Package

```
rs-shell-bin
├── Binaries: rs-shelld, rs-shell-bar, rs-shell-clipboard-ui, rs-shell, rs-shell-launcher
├── Systemd: rs-shelld.service, rs-shell-bar.service, rs-shell-clipboard.service
├── Desktop: rs-shell-launcher.desktop
└── Default config: /usr/share/rs-shell/default-config.toml
```

---

## 16. Phased Roadmap

### Phase 0 — Scaffold (Week 1–2)
- Workspace setup, all crates skeletons
- `rs-shell-core`: event types, event bus, IPC protocol, socket paths
- `rs-shell-config`: schema, hot-reload engine, default config
- `rs-shelld` boots, starts no services, Unix socket accepts connections

**Deliverable**: `rs-shelld` running, `rs-shell subscribe '*'` shows empty stream

### Phase 1 — Clipboard (Week 3–4)
- Fork rsclip into `crates/rs-shell-clipboard/`
- Unify config, socket paths, and event bus integration
- Clipboard events flow through `ShellEvent::Clipboard`
- Clipboard UI subscribes via event bus

**Deliverable**: Clipboard works identically to rsclip but via shared bus

### Phase 2 — Services (Week 5–7)
- Battery (UPower)
- Notifications (org.freedesktop.Notifications D-Bus server)
- Network (NetworkManager basic connectivity)
- Audio (PipeWire)

**Deliverable**: `rs-shell subscribe battery audio notification` shows live events

### Phase 3 — WM Bridge (Week 8–9)
- `wlr-foreign-toplevel` generic adapter
- Hyprland adapter
- Window events (open/close/focus/workspace) flowing to bus

**Deliverable**: `rs-shell subscribe window` shows window lifecycle events

### Phase 4 — Bar MVP (Week 10–12)
- `rs-shell-bar` binary with gtk4-layer-shell
- Working modules: clock, battery, audio, network, workspaces
- Theme CSS loading and hot-reload
- Bar reacts to live service events

**Deliverable**: A functional status bar visible on screen

### Phase 5 — Launcher + Notifications OSD (Week 13–15)
- App launcher with .desktop indexing, fuzzy search, icon support
- Notification OSD with urgency, actions, timeout
- Notification center drawer

**Deliverable**: Full notification flow from any app; launcher accessible via keybind

### Phase 6 — Quick Settings + Wallpaper + Theme (Week 16–17)
- Quick settings panel (volume slider, wifi toggle, dark mode, power buttons)
- Wallpaper service (swww + swaybg + internal backends)
- Full theme switcher with 8 bundled themes
- D-Bus bridge for external clients

**Deliverable**: Everything configured from one panel, theme hot-swappable

### Phase 7 — Polish & Security (Week 18–20)
- Encrypted clipboard secrets
- Systemd service hardening
- Wireless (NetworkManager AP scanning + connect UI)
- Systray (StatusNotifierItem)
- sway + niri adapters
- End-to-end tests via `cargo xtask test-e2e`
- AUR packaging

**Deliverable**: v0.1.0 release, AUR-installable, batteries truly included

---

## 17. Full Dependency Map

```toml
# Across the workspace

# Async runtime
tokio       = "1"          # all async I/O
tokio-stream = "0.1"       # stream adapters for broadcast channels
futures     = "0.3"        # future combinators

# IPC / D-Bus
zbus        = "4"          # battery, network, notifications, audio → D-Bus
zvariant    = "4"          # D-Bus value types

# Serialization
serde       = "1"          # all config + IPC types
serde_json  = "1"          # IPC wire format + Hyprland/sway JSON
toml        = "0.8"        # config file format

# Storage
rusqlite    = "0.31"       # clipboard history (bundled feature = no system SQLite required)

# UI
gtk4        = "0.9"        # all GTK widgets
gtk4-layer-shell = "0.8"  # Wayland layer surfaces for bar, OSD, launcher
glib        = "0.20"       # GLib main loop integration, glib::spawn_future_local

# Wayland protocols (low-level, for wlr-toplevel and wallpaper)
wayland-client        = "0.31"
wayland-protocols-wlr = "0.3"  # zwlr_foreign_toplevel_manager_v1
wayland-protocols     = "0.31" # xdg-shell, etc.

# Audio
pipewire    = "0.8"        # PipeWire Rust bindings (feature-gated)
# libpulse-binding = "2.28" # PulseAudio fallback (feature-gated)

# Config hot-reload
notify      = "6"          # cross-platform file watcher

# Error handling
anyhow      = "1"          # error propagation across async boundaries
thiserror   = "1"          # typed errors in library crates

# Logging
tracing     = "0.1"
tracing-subscriber = "0.3"

# Clipboard runtime tools (external, not Rust deps)
# wl-copy, wl-paste  — from wl-clipboard
# wtype              — from wtype (paste automation)
# tesseract          — OCR (optional)
# swww / swaybg      — wallpaper backends (optional)

# Utilities
once_cell   = "1"          # lazy static initialization
parking_lot = "0.12"       # fast Mutex/RwLock
bytes       = "1"          # zero-copy byte buffers in IPC
uuid        = "1"          # unique IDs for notifications and clipboard entries
chrono      = "0.4"        # timestamps on notifications and clipboard entries
dirs-next   = "2"          # XDG dirs: config_dir(), data_dir(), runtime_dir()
expand-tilde = "0.1"       # ~ expansion in wallpaper/config paths
async-trait = "0.1"        # async fn in trait (WmAdapter)
libc        = "0.2"        # getuid() for socket paths
```

---

*End of rs-shell implementation plan — v0.1 draft*
*Designed to extend rsclip's daemon/UI split, TOML hot-reload, and nonchalant-dark CSS*
*across a full Wayland desktop shell surface area.*
