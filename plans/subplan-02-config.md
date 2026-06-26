# Subplan 02 — Configuration Engine

**Goal:** Provide a single TOML configuration source that can be parsed, defaulted, and hot-reloaded at runtime.

**Target deliverable:** `ConfigManager::new()` loads `~/.config/sone-papdi/config.toml` (or writes a default), returns a `watch::Receiver<ShellConfig>`, and file changes emit updated configs after a 150ms debounce.

---

## Scope

- `crates/sone-papdi-config/src/lib.rs` and supporting modules.
- `ShellConfig` struct mirroring the schema in `implementation-plan.md` Section 5.1.
- `ConfigManager` with `notify` file watcher, `tokio::sync::watch`, and `Arc<RwLock<ShellConfig>>`.
- `assets/default-config.toml` bundled via `include_str!`.
- Path expansion (`~` → `$HOME`) for wallpaper/config paths.

## Dependencies

- Subplan 00 — Scaffold.

## Acceptance Criteria

- [ ] `ShellConfig::default()` parses bundled `default-config.toml`.
- [ ] Missing user config is created from defaults on first run.
- [ ] `ConfigManager` returns a `watch::Receiver` that emits on file change.
- [ ] 150ms debounce prevents reload thrashing.
- [ ] Parse errors log a warning but do not crash the watcher.
- [ ] Unit tests pass.

## Task Checklist

- [ ] Define `GeneralConfig` (wm, terminal, file_manager, browser).
- [ ] Define `BarConfig`, `BarSectionConfig`, `BarModuleConfig`, `ClockConfig`.
- [ ] Define `ClipboardConfig`, `ClipboardUiConfig`.
- [ ] Define `LauncherConfig`.
- [ ] Define `NotificationConfig`.
- [ ] Define `BatteryConfig`.
- [ ] Define `AudioConfig`.
- [ ] Define `NetworkConfig`.
- [ ] Define `WallpaperConfig`.
- [ ] Define `ThemeConfig`, `ThemeColors`.
- [ ] Define `LockscreenConfig`.
- [ ] Define `KeybindsConfig`.
- [ ] Assemble `ShellConfig` with serde derive.
- [ ] Implement `Default` via bundled `default-config.toml`.
- [ ] Implement `config_path()` resolving `~/.config/sone-papdi/config.toml`.
- [ ] Implement `load_config` with missing-key merge against defaults.
- [ ] Implement `ConfigManager::new()`.
- [ ] Implement `ConfigManager::watch()` with debounce.
- [ ] Implement `ConfigManager::get()`.
- [ ] Implement `expand_paths()` helper for `~` expansion.
- [ ] Write `assets/default-config.toml`.
- [ ] Write unit tests for parse, default, and hot-reload simulation.

## Verification

```bash
cargo test -p sone-papdi-config
cargo clippy -p sone-papdi-config -- -D warnings
```

## Notes / Risks

- Hot-reload tests must use a temp directory, not the real `~/.config`.
- Consider whether to merge missing keys deeply or shallowly; document the choice.
- `ThemeColors` is a map-like struct; use a dedicated type so overrides are explicit.
- The watcher should survive the config file being replaced by an atomic write (create + rename).
