# Subplan 10 — Wallpaper & Theme Service

**Goal:** Apply wallpaper and theme changes from config, supporting external backends and bundled CSS themes.

**Target deliverable:** Calling the wallpaper/theme service applies the configured wallpaper/theme and emits `WallpaperEvent::Changed` / `ThemeEvent::Changed` + `ColorsUpdated`.

---

## Scope

- `crates/sone-papdi-services/src/wallpaper.rs`.
- `crates/sone-papdi-services/src/theme.rs`.
- Backends: `swww`, `swaybg`, `hyprpaper`, and an `internal` placeholder.
- Theme CSS resolution: user dir → bundled dir → color overrides.
- gsettings integration for GTK/icon themes.
- CSS variable extraction.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`WallpaperEvent`, `ThemeEvent`, `ShellEvent`).
- Subplan 02 — Configuration Engine (`WallpaperConfig`, `ThemeConfig`).

## Acceptance Criteria

- [ ] `wallpaper::apply` runs the configured backend.
- [ ] `swww`, `swaybg`, `hyprpaper` backends work by spawning external commands.
- [ ] `internal` backend is a stub or launches a GTK background process.
- [ ] Emits `WallpaperEvent::Changed` on success.
- [ ] `theme::ThemeManager::apply` resolves CSS from bundled or user themes.
- [ ] Applies GTK theme and icon theme via gsettings.
- [ ] Writes active CSS to `~/.config/sone-papdi/active-theme.css`.
- [ ] Emits `ThemeEvent::Changed` and `ColorsUpdated`.
- [ ] Unit tests for CSS variable extraction and theme resolution.

## Task Checklist

- [ ] Implement `expand_tilde` helper.
- [ ] Implement wallpaper backend dispatch (`swww`, `swaybg`, `hyprpaper`, `internal`).
- [ ] Implement `wallpaper::apply`.
- [ ] Define bundled themes list (via `include_str!`).
- [ ] Implement theme resolution: user dir → bundled.
- [ ] Implement color override overlay.
- [ ] Implement CSS variable extraction.
- [ ] Implement gsettings calls for `gtk-theme` and `icon-theme`.
- [ ] Implement `ThemeManager::apply`.
- [ ] Write unit tests for `resolve_css`.
- [ ] Write unit tests for CSS variable extraction.

## Verification

```bash
cargo build -p sone-papdi-services
cargo test -p sone-papdi-services wallpaper
cargo test -p sone-papdi-services theme
cargo clippy -p sone-papdi-services -- -D warnings
```

## Notes / Risks

- External backend commands may not be available in CI; test with stubbed `Command` or unit tests.
- gsettings may fail in minimal containers; failures should be logged, not fatal.
- Bundled themes should be minimal CSS files to keep binary size reasonable.
- The `internal` wallpaper backend is a placeholder unless Subplan 11/UI implements it.
