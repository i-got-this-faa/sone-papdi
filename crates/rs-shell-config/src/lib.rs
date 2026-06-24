//! TOML configuration schema and hot-reload engine for sone-papdi.

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use anyhow::Result;

// ── Top-level config ───────────────────────────────────────

/// Parsed config, cloneable and shared across threads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub bar: BarConfig,
    #[serde(default)]
    pub clipboard: ClipboardConfig,
    #[serde(default)]
    pub launcher: LauncherConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub battery: BatteryConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub wallpaper: WallpaperConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub lockscreen: LockscreenConfig,
    #[serde(default)]
    pub keybinds: KeybindsConfig,
}

impl Default for ShellConfig {
    fn default() -> Self {
        // Parses assets/default-config.toml bundled at compile time
        toml::from_str(include_str!("../../../assets/default-config.toml"))
            .expect("default config must be valid")
    }
}

impl ShellConfig {
    /// Expand tilde paths in all applicable fields.
    pub fn expand_paths(&mut self) {
        if self.wallpaper.path.starts_with('~') {
            self.wallpaper.path = expand_tilde(&self.wallpaper.path);
        }
    }
}

// ── General ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_wm")]
    pub wm: String,
    #[serde(default = "default_terminal")]
    pub terminal: String,
    #[serde(default = "default_file_manager")]
    pub file_manager: String,
    #[serde(default = "default_browser")]
    pub browser: String,
}

fn default_wm() -> String {
    "auto".into()
}
fn default_terminal() -> String {
    "foot".into()
}
fn default_file_manager() -> String {
    "thunar".into()
}
fn default_browser() -> String {
    "firefox".into()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            wm: default_wm(),
            terminal: default_terminal(),
            file_manager: default_file_manager(),
            browser: default_browser(),
        }
    }
}

// ── Bar ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarConfig {
    #[serde(default = "default_bar_position")]
    pub position: String,
    #[serde(default = "default_bar_height")]
    pub height: u32,
    #[serde(default = "default_bar_opacity")]
    pub opacity: f64,
    #[serde(default = "default_bar_font")]
    pub font: String,
    #[serde(default = "default_true")]
    pub exclusive_zone: bool,
    #[serde(default)]
    pub left: BarSectionConfig,
    #[serde(default)]
    pub center: BarSectionConfig,
    #[serde(default)]
    pub right: BarSectionConfig,
    #[serde(default)]
    pub clock: ClockConfig,
}

fn default_bar_position() -> String {
    "top".into()
}
fn default_bar_height() -> u32 {
    32
}
fn default_bar_opacity() -> f64 {
    0.92
}
fn default_bar_font() -> String {
    "JetBrains Mono 10".into()
}
fn default_true() -> bool {
    true
}

impl Default for BarConfig {
    fn default() -> Self {
        Self {
            position: default_bar_position(),
            height: default_bar_height(),
            opacity: default_bar_opacity(),
            font: default_bar_font(),
            exclusive_zone: true,
            left: BarSectionConfig::default(),
            center: BarSectionConfig::default(),
            right: BarSectionConfig::default(),
            clock: ClockConfig::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BarSectionConfig {
    #[serde(default)]
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockConfig {
    #[serde(default = "default_clock_format")]
    pub format: String,
}

fn default_clock_format() -> String {
    "%a %d %b  %H:%M".into()
}

impl Default for ClockConfig {
    fn default() -> Self {
        Self {
            format: default_clock_format(),
        }
    }
}

// ── Clipboard ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    #[serde(default = "default_max_entries")]
    pub max_entries: i64,
    #[serde(default = "default_max_text_bytes")]
    pub max_text_bytes: usize,
    #[serde(default = "default_max_image_bytes")]
    pub max_image_bytes: usize,
    #[serde(default = "default_true")]
    pub dedupe: bool,
    #[serde(default = "default_cleanup_days")]
    pub cleanup_unpinned_after_days: i64,
    #[serde(default = "default_true")]
    pub auto_paste: bool,
    #[serde(default = "default_paste_delay")]
    pub paste_delay_ms: u64,
    #[serde(default = "default_paste_method")]
    pub paste_method: String,
    #[serde(default = "default_true")]
    pub favicon_cache: bool,
    #[serde(default)]
    pub ui: ClipboardUiConfig,
}

fn default_max_entries() -> i64 {
    5000
}
fn default_max_text_bytes() -> usize {
    1048576
}
fn default_max_image_bytes() -> usize {
    10485760
}
fn default_cleanup_days() -> i64 {
    30
}
fn default_paste_delay() -> u64 {
    140
}
fn default_paste_method() -> String {
    "wtype".into()
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            max_text_bytes: default_max_text_bytes(),
            max_image_bytes: default_max_image_bytes(),
            dedupe: true,
            cleanup_unpinned_after_days: 30,
            auto_paste: true,
            paste_delay_ms: 140,
            paste_method: default_paste_method(),
            favicon_cache: true,
            ui: ClipboardUiConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardUiConfig {
    #[serde(default = "default_ui_width")]
    pub window_width: u32,
    #[serde(default = "default_ui_height")]
    pub window_height: u32,
    #[serde(default = "default_ui_opacity")]
    pub background_opacity: f64,
    #[serde(default = "default_true")]
    pub preview_default: bool,
}

fn default_ui_width() -> u32 {
    920
}
fn default_ui_height() -> u32 {
    620
}
fn default_ui_opacity() -> f64 {
    0.70
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

// ── Launcher ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default = "default_launcher_width")]
    pub width: u32,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default = "default_true")]
    pub show_icons: bool,
    #[serde(default = "default_true")]
    pub fuzzy: bool,
    #[serde(default = "default_terminal_prefix")]
    pub terminal_prefix: String,
}

fn default_launcher_width() -> u32 {
    600
}
fn default_max_results() -> usize {
    12
}
fn default_terminal_prefix() -> String {
    "foot -e".into()
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            width: 600,
            max_results: 12,
            show_icons: true,
            fuzzy: true,
            terminal_prefix: default_terminal_prefix(),
        }
    }
}

// ── Notifications ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: i64,
    #[serde(default)]
    pub timeout_critical_ms: i64,
    #[serde(default = "default_max_visible")]
    pub max_visible: usize,
    #[serde(default = "default_notif_position")]
    pub position: String,
    #[serde(default = "default_notif_gap")]
    pub gap: u32,
    #[serde(default = "default_notif_width")]
    pub width: u32,
}

fn default_timeout_ms() -> i64 {
    5000
}
fn default_max_visible() -> usize {
    5
}
fn default_notif_position() -> String {
    "top-right".into()
}
fn default_notif_gap() -> u32 {
    8
}
fn default_notif_width() -> u32 {
    360
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            timeout_critical_ms: 0,
            max_visible: 5,
            position: default_notif_position(),
            gap: 8,
            width: 360,
        }
    }
}

// ── Battery ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    #[serde(default = "default_warn_level")]
    pub warn_level: f64,
    #[serde(default = "default_critical_level")]
    pub critical_level: f64,
    #[serde(default = "default_device")]
    pub device: String,
}

fn default_warn_level() -> f64 {
    20.0
}
fn default_critical_level() -> f64 {
    5.0
}
fn default_device() -> String {
    "auto".into()
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

// ── Audio ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_audio_backend")]
    pub backend: String,
    #[serde(default = "default_max_volume")]
    pub max_volume: u32,
    #[serde(default = "default_scroll_step")]
    pub scroll_step: u32,
}

fn default_audio_backend() -> String {
    "auto".into()
}
fn default_max_volume() -> u32 {
    150
}
fn default_scroll_step() -> u32 {
    5
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

// ── Network ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_primary_interface")]
    pub primary_interface: String,
}

fn default_primary_interface() -> String {
    "auto".into()
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            primary_interface: "auto".into(),
        }
    }
}

// ── Wallpaper ───────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperConfig {
    #[serde(default = "default_wallpaper_path")]
    pub path: String,
    #[serde(default = "default_wallpaper_mode")]
    pub mode: String,
    #[serde(default = "default_wallpaper_backend")]
    pub backend: String,
}

fn default_wallpaper_path() -> String {
    "~/.config/rs-shell/wallpaper.jpg".into()
}
fn default_wallpaper_mode() -> String {
    "fill".into()
}
fn default_wallpaper_backend() -> String {
    "internal".into()
}

impl Default for WallpaperConfig {
    fn default() -> Self {
        Self {
            path: default_wallpaper_path(),
            mode: "fill".into(),
            backend: "internal".into(),
        }
    }
}

// ── Theme ───────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub name: String,
    #[serde(default = "default_true")]
    pub dark_mode: bool,
    #[serde(default = "default_icon_theme")]
    pub icon_theme: String,
    #[serde(default = "default_gtk_theme")]
    pub gtk_theme: String,
    #[serde(default = "default_theme_font")]
    pub font: String,
    #[serde(default)]
    pub colors: ThemeColors,
}

fn default_theme_name() -> String {
    "nonchalant-dark".into()
}
fn default_icon_theme() -> String {
    "Papirus-Dark".into()
}
fn default_gtk_theme() -> String {
    "adw-gtk3-dark".into()
}
fn default_theme_font() -> String {
    "Inter 11".into()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
            dark_mode: true,
            icon_theme: default_icon_theme(),
            gtk_theme: default_gtk_theme(),
            font: default_theme_font(),
            colors: ThemeColors::default(),
        }
    }
}

/// Flat map of color overrides. Missing keys inherit from the named theme.
pub type ThemeColors = HashMap<String, String>;

// ── Lock Screen ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockscreenConfig {
    #[serde(default = "default_lockscreen_command")]
    pub command: String,
    #[serde(default = "default_true")]
    pub blur_background: bool,
}

fn default_lockscreen_command() -> String {
    "swaylock".into()
}

impl Default for LockscreenConfig {
    fn default() -> Self {
        Self {
            command: default_lockscreen_command(),
            blur_background: true,
        }
    }
}

// ── Keybinds ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindsConfig {
    #[serde(default = "default_kb_show_launcher")]
    pub show_launcher: String,
    #[serde(default = "default_kb_show_clipboard")]
    pub show_clipboard: String,
    #[serde(default = "default_kb_show_notifications")]
    pub show_notifications: String,
    #[serde(default = "default_kb_screenshot")]
    pub screenshot: String,
    #[serde(default = "default_kb_lock")]
    pub lock: String,
}

fn default_kb_show_launcher() -> String {
    "Super+Space".into()
}
fn default_kb_show_clipboard() -> String {
    "Super+V".into()
}
fn default_kb_show_notifications() -> String {
    "Super+N".into()
}
fn default_kb_screenshot() -> String {
    "Print".into()
}
fn default_kb_lock() -> String {
    "Super+L".into()
}

impl Default for KeybindsConfig {
    fn default() -> Self {
        Self {
            show_launcher: default_kb_show_launcher(),
            show_clipboard: default_kb_show_clipboard(),
            show_notifications: default_kb_show_notifications(),
            screenshot: default_kb_screenshot(),
            lock: default_kb_lock(),
        }
    }
}

// ── Config Manager ──────────────────────────────────────────

/// Watches a config file for changes, hot-reloads on modification.
pub struct ConfigManager {
    config: Arc<RwLock<ShellConfig>>,
    sender: watch::Sender<ShellConfig>,
    path: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager. Loads the config file or writes defaults.
    /// Returns the manager and a watch channel receiver.
    pub async fn new() -> Result<(Self, watch::Receiver<ShellConfig>)> {
        let path = config_path();
        let config = if path.exists() {
            load_config(&path).await?
        } else {
            let default = ShellConfig::default();
            // Write default config on first run
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

    /// Start the file watcher loop (call in a `tokio::spawn`).
    pub async fn watch(self: Arc<Self>) -> Result<()> {
        use tokio::sync::mpsc;

        let path = self.path.clone();
        let (tx, mut rx) = mpsc::channel::<()>(8);

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if res
                    .map(|e| e.kind.is_modify() || e.kind.is_create())
                    .unwrap_or(false)
                {
                    let _ = tx.try_send(());
                }
            },
            notify::Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::NonRecursive)?;

        // Debounce 150ms
        loop {
            if rx.recv().await.is_none() {
                break;
            }
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

    /// Get the current config.
    pub async fn get(&self) -> ShellConfig {
        self.config.read().await.clone()
    }
}

// ── Helpers ─────────────────────────────────────────────────

/// Load config from a TOML file, merging with defaults.
pub async fn load_config(path: &PathBuf) -> Result<ShellConfig> {
    let text = tokio::fs::read_to_string(path).await?;
    let mut cfg: ShellConfig = toml::from_str(&text)?;
    // Merge with defaults for any missing keys (serde default handles this)
    cfg.expand_paths();
    Ok(cfg)
}

/// Returns the default config file path: `~/.config/rs-shell/config.toml`.
pub fn config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    PathBuf::from(home).join(".config/rs-shell/config.toml")
}

/// Expand tilde in a path string to the user's home directory.
fn expand_tilde(path: &str) -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
    if path.starts_with("~/") {
        format!("{}{}", home.trim_end_matches('/'), &path[1..])
    } else if path == "~" {
        home
    } else {
        path.to_string()
    }
}

// ── Tests ──────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_parses() {
        // ShellConfig::default() parses the bundled default config
        let config = ShellConfig::default();
        assert_eq!(config.general.wm, "auto");
        assert_eq!(config.general.terminal, "foot");
        assert_eq!(config.bar.position, "top");
        assert_eq!(config.bar.height, 32);
        assert_eq!(config.clipboard.max_entries, 5000);
        assert_eq!(config.theme.name, "nonchalant-dark");
        assert_eq!(config.lockscreen.command, "swaylock");
        assert_eq!(config.keybinds.show_launcher, "Super+Space");
    }

    #[test]
    fn test_config_override_merges_correctly() {
        // A partial config file should merge with defaults for missing keys.
        let partial = r#"
[general]
wm = "sway"
terminal = "alacritty"
"#;
        let mut cfg: ShellConfig = toml::from_str(partial).unwrap();
        cfg.expand_paths();
        assert_eq!(cfg.general.wm, "sway");
        assert_eq!(cfg.general.terminal, "alacritty");
        // Defaults for unspecified fields
        assert_eq!(cfg.general.file_manager, "thunar");
        assert_eq!(cfg.general.browser, "firefox");
        assert_eq!(cfg.bar.position, "top");
        assert_eq!(cfg.clipboard.max_entries, 5000);
        assert_eq!(cfg.theme.name, "nonchalant-dark");
    }

    #[test]
    fn test_partial_bar_config() {
        let partial = r#"
[bar]
height = 48
"#;
        let cfg: ShellConfig = toml::from_str(partial).unwrap();
        assert_eq!(cfg.bar.height, 48);
        assert_eq!(cfg.bar.position, "top"); // default
        assert_eq!(cfg.bar.opacity, 0.92);   // default
}
    #[test]
    fn test_partial_theme_config() {
        let partial = r##"
[theme]
name = "catppuccin-mocha"
[theme.colors]
accent = "#89b4fa"
"##;
        let cfg: ShellConfig = toml::from_str(partial).unwrap();
        assert_eq!(cfg.theme.name, "catppuccin-mocha");
        assert_eq!(cfg.theme.colors.get("accent").unwrap(), "#89b4fa");
        assert!(cfg.theme.dark_mode); // default
    }

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/wallpaper.jpg");
        assert!(!expanded.starts_with('~'), "tilde should be expanded");
        assert!(expanded.ends_with("/wallpaper.jpg"));
    }

    #[test]
    fn test_expand_paths_in_config() {
        let mut cfg = ShellConfig::default();
        assert!(cfg.wallpaper.path.starts_with('~'));
        cfg.expand_paths();
        assert!(!cfg.wallpaper.path.starts_with('~'));
    }

    #[tokio::test]
    async fn test_config_file_write_and_read() {
        // Create a temp dir and write a config file
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let default = ShellConfig::default();
        let toml_str = toml::to_string_pretty(&default).unwrap();
        tokio::fs::write(&config_path, &toml_str)
            .await
            .unwrap();

        // Read it back
        let loaded = load_config(&config_path).await.unwrap();
        assert_eq!(loaded.general.wm, default.general.wm);
        assert_eq!(loaded.bar.height, default.bar.height);
    }

    #[tokio::test]
    async fn test_parse_error_does_not_panic() {
        // Write invalid TOML
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        tokio::fs::write(&config_path, "invalid toml ====")
            .await
            .unwrap();

        let result = load_config(&config_path).await;
        assert!(result.is_err(), "invalid TOML should produce an error");
    }

    #[tokio::test]
    async fn test_config_manager_creates_default() {
        // ConfigManager::new() in a temp dir should write the default config
        let orig_home = std::env::var("HOME").ok();
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", dir.path());

        let (mgr, _rx) = ConfigManager::new().await.unwrap();
        let cfg = mgr.get().await;
        assert_eq!(cfg.general.wm, "auto");

        // Check file was created
        let config_path = dir.path().join(".config/rs-shell/config.toml");
        assert!(config_path.exists(), "default config should be written");

        if let Some(home) = orig_home {
            std::env::set_var("HOME", home);
        }
    }

    #[tokio::test]
    async fn test_config_manager_loads_existing() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().join(".config/rs-shell");
        tokio::fs::create_dir_all(&config_dir).await.unwrap();
        let config_path = config_dir.join("config.toml");

        // Write a modified config
        let custom = r#"
[general]
wm = "hyprland"
terminal = "kitty"

[bar]
height = 40
"#;
        tokio::fs::write(&config_path, custom).await.unwrap();

        let orig_home = std::env::var("HOME").ok();
        std::env::set_var("HOME", dir.path());

        let (mgr, _rx) = ConfigManager::new().await.unwrap();
        let cfg = mgr.get().await;
        assert_eq!(cfg.general.wm, "hyprland");
        assert_eq!(cfg.general.terminal, "kitty");
        assert_eq!(cfg.bar.height, 40);
        // Defaults still apply
        assert_eq!(cfg.theme.name, "nonchalant-dark");

        if let Some(home) = orig_home {
            std::env::set_var("HOME", home);
        }
    }
}
