# Subplan 11 — UI Components

**Goal:** Build shared GTK4 layer-shell helpers, CSS loading, and reusable widgets used by the bar, launcher, OSD, and quick settings.

**Target deliverable:** A standalone `rs-shell-ui` library compiles; `apply_css_to_screen` loads CSS; shared widgets render without crashing.

---

## Scope

- `crates/rs-shell-ui/src/lib.rs`.
- `crates/rs-shell-ui/src/theme.rs` — CSS provider loading/hot-reload.
- `crates/rs-shell-ui/src/layer_window.rs` — helper to create anchored layer-shell windows.
- Shared widgets: `BarModule`, `IconLabel`, `Meter`, etc. (stubs acceptable).
- Integration with `tokio` broadcast receiver → GTK updates via `glib::spawn_future_local`.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`ShellEvent`).
- Subplan 02 — Configuration Engine (for theme/config watches).

## Acceptance Criteria

- [ ] `rs-shell-ui` crate compiles with GTK4 and `gtk4-layer-shell`.
- [ ] `apply_css_to_screen` loads CSS onto the default display.
- [ ] `watch_and_reload_theme` reacts to config watch changes.
- [ ] Layer-shell window helper supports layer, anchors, margins, exclusive zone.
- [ ] At least one shared widget renders in a simple GTK app test.
- [ ] Unit tests for non-GTK helpers.

## Task Checklist

- [ ] Add GTK4 + layer-shell dependencies.
- [ ] Implement `apply_css_to_screen(css: &str)`.
- [ ] Implement `watch_and_reload_theme(config_rx)`.
- [ ] Implement `LayerWindowBuilder` helper.
- [ ] Implement `BarModule` base widget.
- [ ] Implement `IconLabel` widget.
- [ ] Implement `Meter` / `ProgressBar` styled widget.
- [ ] Implement async event bridge: tokio broadcast → GTK main loop.
- [ ] Add example binary showing a layer-shell window.
- [ ] Write unit tests for helper functions.

## Verification

```bash
cargo build -p rs-shell-ui
cargo test -p rs-shell-ui
cargo clippy -p rs-shell-ui -- -D warnings
```

## Notes / Risks

- GTK tests in CI require a display; use `xvfb-run` or mark as `#[ignore]` for manual run.
- Keep widget surface minimal; full styling comes from theme assets.
- Layer-shell API version compatibility matters; pin to `gtk4-layer-shell 0.8`.
- The `apply_css_to_screen` function must handle multiple displays correctly.
