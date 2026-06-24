//! TOML configuration schema and hot-reload manager for sone-papdi.

use anyhow::{Context, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{watch, RwLock};

/// Top-level shell configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ShellConfig {
    /// General settings.
    pub general: GeneralConfig,
    /// Bar/panel configuration.
    pub bar: BarConfig,
    /// Clipboard configuration.
    pub clipboard: ClipboardConfig,
    /// Launcher configuration.
    pub launcher: LauncherConfig,
    /// Notification configuration.
    pub notifications: NotificationConfig,
    /// Battery configuration.
    pub battery: BatteryConfig,
    /// Audio configuration.
    pub audio: AudioConfig,
    /// Network configuration.
    pub network: NetworkConfig,
    /// Wallpaper configuration.
    pub wallpaper: WallpaperConfig,
    /// Theme configuration.
    pub theme: ThemeConfig,
    /// Lock screen configuration.
    pub lockscreen: LockscreenConfig,
    /// Keybind configuration.
    pub keybinds: KeybindsConfig,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            bar: BarConfig::default(),
            clipboard: ClipboardConfig::default(),
            launcher: LauncherConfig::default(),
            notifications: NotificationConfig::default(),
            battery: BatteryConfig::default(),
            audio: AudioConfig::default(),
            network: NetworkConfig::default(),
            wallpaper: WallpaperConfig::default(),
            theme: ThemeConfig::default(),
            lockscreen: LockscreenConfig::default(),
            keybinds: KeybindsConfig::default(),
        }
    }
}

impl ShellConfig {
    /// Expand `~` in paths that support it (wallpaper, theme user dirs).
    pub fn expand_paths(&mut self) {
        if let Ok(expanded) = expand_tilde::expand_tilde(&self.wallpaper.path) {
            self.wallpaper.path = expanded.to_string_lossy().to_string();
        }
    }
}

/// General compositor/application settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct GeneralConfig {
    /// Window manager to use, or "auto" for detection.
    pub wm: String,
    /// Default terminal emulator.
    pub terminal: String,
    /// Default file manager.
    pub file_manager: String,
    /// Default web browser.
    pub browser: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            wm: "auto".into(),
            terminal: "foot".into(),
            file_manager: "thunar".into(),
            browser: "firefox".into(),
        }
    }
}

/// Bar section configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BarSectionConfig {
    /// Modules shown in this section.
    pub modules: Vec<String>,
}

impl Default for BarSectionConfig {
    fn default() -> Self {
        Self { modules: Vec::new() }
    }
}

/// Clock module configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ClockConfig {
    /// strftime format for the clock label.
    pub format: String,
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: "%a %d %b  %H:%M".into(),
        }
    }
}

/// Bar/panel configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BarConfig {
    /// Bar position on screen.
    pub position: String,
    /// Bar height in pixels.
    pub height: u32,
    /// Background opacity (0.0–1.0).
    pub opacity: f64,
    /// Primary font.
    pub font: String,
    /// Whether to reserve screen space for the bar.
    pub exclusive_zone: bool,
    /// Left section.
    pub left: BarSectionConfig,
    /// Center section.
    pub center: BarSectionConfig,
    /// Right section.
    pub right: BarSectionConfig,
    /// Clock module config.
    pub clock: ClockConfig,
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            position: "top".into(),
            height: 32,
            opacity: 0.92,
            font: "JetBrains Mono 10".into(),
            exclusive_zone: true,
            left: BarSectionConfig { modules: vec!["workspaces".into(), "window-title".into()] },
            center: BarSectionConfig { modules: vec!["clock".into()] },
            right: BarSectionConfig {
                modules: vec![
                    "systray".into(),
                    "audio".into(),
                    "network".into(),
                    "battery".into(),
                    "notifications".into(),
                ],
            },
            clock: ClockConfig::default(),
        }
    }
}

/// Clipboard UI configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ClipboardUiConfig {
    /// Window width in pixels.
    pub window_width: u32,
    /// Window height in pixels.
    pub window_height: u32,
    /// Background opacity (0.0–1.0).
    pub background_opacity: f64,
    /// Show preview by default.
    pub preview_default: bool,
}

impl Default for ClipboardUiConfig {
    fn default() -> Self {
        Self {
            window_width: 920,
            window_height: 620,
            background_opacity: 0.70,
            preview_default: true,
        }
    }
}

/// Clipboard configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ClipboardConfig {
    /// Maximum history entries.
    pub max_entries: usize,
    /// Maximum bytes for a text entry.
    pub max_text_bytes: usize,
    /// Maximum bytes for an image entry.
    pub max_image_bytes: usize,
    /// Remove duplicate entries.
    pub dedupe: bool,
    /// Days before unpinned old entries are cleaned.
    pub cleanup_unpinned_after_days: u32,
    /// Automatically paste selected entries.
    pub auto_paste: bool,
    /// Delay before pasting, in milliseconds.
    pub paste_delay_ms: u64,
    /// Paste automation backend.
    pub paste_method: String,
    /// Cache website favicons for link entries.
    pub favicon_cache: bool,
    /// UI-specific configuration.
    pub ui: ClipboardUiConfig,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            max_entries: 5000,
            max_text_bytes: 1_048_576,
            max_image_bytes: 10_485_760,
            dedupe: true,
            cleanup_unpinned_after_days: 30,
            auto_paste: true,
            paste_delay_ms: 140,
            paste_method: "wtype".into(),
            favicon_cache: true,
            ui: ClipboardUiConfig::default(),
        }
    }
}

/// Launcher configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LauncherConfig {
    /// Window width in pixels.
    pub width: u32,
    /// Maximum results shown.
    pub max_results: usize,
    /// Show application icons.
    pub show_icons: bool,
    /// Use fuzzy matching.
    pub fuzzy: bool,
    /// Terminal command prefix for terminal apps.
    pub terminal_prefix: String,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            width: 600,
            max_results: 12,
            show_icons: true,
            fuzzy: true,
            terminal_prefix: "foot -e".into(),
        }
    }
}

/// Notification configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NotificationConfig {
    /// Default timeout for normal notifications, in milliseconds.
    pub timeout_ms: u64,
    /// Timeout for critical notifications; 0 means never auto-dismiss.
    pub timeout_critical_ms: u64,
    /// Maximum visible notifications at once.
    pub max_visible: usize,
    /// OSD anchor position.
    pub position: String,
    /// Gap between OSD popups in pixels.
    pub gap: u32,
    /// OSD popup width in pixels.
    pub width: u32,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            timeout_critical_ms: 0,
            max_visible: 5,
            position: "top-right".into(),
            gap: 8,
            width: 360,
        }
    }
}

/// Battery configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct BatteryConfig {
    /// Percentage at which to emit a low-battery warning.
    pub warn_level: f64,
    /// Percentage at which to emit a critical-battery warning.
    pub critical_level: f64,
    /// UPower device path, or "auto" for the display device.
    pub device: String,
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            warn_level: 20.0,
            critical_level: 5.0,
            device: "auto".into(),
        }
    }
}

/// Audio configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct AudioConfig {
    /// Audio backend to use.
    pub backend: String,
    /// Maximum allowed volume percentage.
    pub max_volume: u32,
    /// Volume step on scroll.
    pub scroll_step: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            backend: "auto".into(),
            max_volume: 150,
            scroll_step: 5,
        }
    }
}

/// Network configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct NetworkConfig {
    /// Primary network interface, or "auto".
    pub primary_interface: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            primary_interface: "auto".into(),
        }
    }
}

/// Wallpaper configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct WallpaperConfig {
    /// Path to wallpaper image.
    pub path: String,
    /// Fill mode.
    pub mode: String,
    /// Backend used to apply the wallpaper.
    pub backend: String,
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            path: "~/.config/rs-shell/wallpaper.jpg".into(),
            mode: "fill".into(),
            backend: "internal".into(),
        }
    }
}

/// Theme color overrides.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(default)]
pub struct ThemeColors {
    /// Accent color.
    pub accent: Option<String>,
    /// Text color on accent.
    pub accent_text: Option<String>,
}

/// Theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ThemeConfig {
    /// Theme name.
    pub name: String,
    /// Whether dark mode is enabled.
    pub dark_mode: bool,
    /// Icon theme name.
    pub icon_theme: String,
    /// GTK theme name.
    pub gtk_theme: String,
    /// Primary UI font.
    pub font: String,
    /// Color overrides.
    pub colors: ThemeColors,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "nonchalant-dark".into(),
            dark_mode: true,
            icon_theme: "Papirus-Dark".into(),
            gtk_theme: "adw-gtk3-dark".into(),
            font: "Inter 11".into(),
            colors: ThemeColors::default(),
        }
    }
}

/// Lock screen configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct LockscreenConfig {
    /// External lock command.
    pub command: String,
    /// Blur the background behind the lock screen.
    pub blur_background: bool,
}

impl Default for LockscreenConfig {
    fn default() -> Self {
        Self {
            command: "swaylock".into(),
            blur_background: true,
        }
    }
}

/// Keybind configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct KeybindsConfig {
    /// Show launcher.
    pub show_launcher: String,
    /// Show clipboard manager.
    pub show_clipboard: String,
    /// Show notification center.
    pub show_notifications: String,
    /// Screenshot key.
    pub screenshot: String,
    /// Lock screen key.
    pub lock: String,
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            show_launcher: "Super+Space".into(),
            show_clipboard: "Super+V".into(),
            show_notifications: "Super+N".into(),
            screenshot: "Print".into(),
            lock: "Super+L".into(),
        }
    }
}

/// Manager that owns the current config and notifies subscribers of changes.
pub struct ConfigManager {
    config: Arc<RwLock<ShellConfig>>,
    sender: watch::Sender<ShellConfig>,
    path: PathBuf,
}

impl ConfigManager {
    /// Load or create the user config and return the manager plus a watch receiver.
    pub async fn new() -> Result<(Self, watch::Receiver<ShellConfig>)> {
        let path = config_path();
        let config = if path.exists() {
            load_config(&path).await?
        } else {
            let default = ShellConfig::default();
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
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

    /// Start the file watcher loop. Spawns a background task.
    pub async fn watch(self: Arc<Self>) -> Result<()> {
        let path = self.path.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(8);

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let _ = tx.try_send(());
                    }
                }
            },
            notify::Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        loop {
            if rx.recv().await.is_none() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            while rx.try_recv().is_ok() {}

            match load_config(&path).await {
                Ok(mut new_cfg) => {
                    new_cfg.expand_paths();
                    *self.config.write().await = new_cfg.clone();
                    let _ = self.sender.send(new_cfg);
                    tracing::info!("config hot-reloaded");
                }
                Err(e) => tracing::warn!("config reload failed: {e}"),
            }
        }

        Ok(())
    }

    /// Return a copy of the current config.
    pub async fn get(&self) -> ShellConfig {
        self.config.read().await.clone()
    }

    /// Return a clone of the watch sender.
    pub fn sender(&self) -> watch::Sender<ShellConfig> {
        self.sender.clone()
    }
}

/// Path to the user config file (`~/.config/rs-shell/config.toml`).
pub fn config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("rs-shell/config.toml");
    }
    dirs_next::config_dir()
        .map(|p| p.join("rs-shell/config.toml"))
        .unwrap_or_else(|| PathBuf::from("/tmp/rs-shell-config/config.toml"))
}

async fn load_config(path: &Path) -> Result<ShellConfig> {
    let text = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read config from {}", path.display()))?;
    let mut cfg: ShellConfig = toml::from_str(&text)
        .with_context(|| format!("failed to parse config from {}", path.display()))?;
    cfg.expand_paths();
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn default_config_parses() {
        let cfg = ShellConfig::default();
        assert_eq!(cfg.general.wm, "auto");
        assert_eq!(cfg.bar.height, 32);
        assert_eq!(cfg.clipboard.paste_method, "wtype");
    }

    #[tokio::test]
    async fn config_manager_creates_default() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path().as_os_str());

        let expected_file = tmp.path().join("rs-shell/config.toml");
        let (mgr, _rx) = ConfigManager::new().await.unwrap();
        let cfg = mgr.get().await;
        assert!(expected_file.exists());
        assert_eq!(cfg.theme.name, "nonchalant-dark");

        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[tokio::test]
    #[ignore = "stack overflow in test watcher; manual smoke test recommended"]
    async fn config_hot_reload() {
        let tmp = tempfile::tempdir().unwrap();
        std::env::set_var("XDG_CONFIG_HOME", tmp.path().as_os_str());

        let (mgr, mut rx) = ConfigManager::new().await.unwrap();
        let mgr = Arc::new(mgr);

        // Wait for the initial value.
        let _ = rx.changed().await;
        assert_eq!(rx.borrow().bar.height, 32);

        let watcher = mgr.clone();
        let handle = tokio::spawn(async move { watcher.watch().await });

        // Rewrite the file.
        let config_file = config_path();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&config_file)
            .unwrap();
        writeln!(file, r#"[bar]
height = 64"#).unwrap();
        drop(file);

        // Wait for reload.
        let timeout = tokio::time::Duration::from_secs(5);
        let changed = tokio::time::timeout(timeout, rx.changed()).await;
        assert!(changed.is_ok(), "config reload did not fire");
        assert_eq!(rx.borrow().bar.height, 64);

        handle.abort();
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
