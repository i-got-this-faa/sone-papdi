# Subplan 12 — Bar / Panel

**Goal:** Implement the `sone-papdi-bar` binary: a GTK4 layer-shell panel with left/center/right modules reacting to live events.

**Target deliverable:** Running `sone-papdi-bar` shows a top/bottom bar with clock, battery, audio, network, workspaces, and notification bell.

---

## Scope

- `crates/sone-papdi-bar/src/main.rs`.
- Module widgets: workspaces, window title, clock, systray stub, audio, network, battery, notifications.
- Bar layout via `gtk4::CenterBox`.
- IPC client to `rs-shelld` for events.
- Config-driven position, height, modules.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (event types, socket protocol).
- Subplan 02 — Configuration Engine (`[bar]` config).
- Subplan 05 — WM Bridge (workspace/window events).
- Subplan 06 — Battery Service.
- Subplan 07 — Network & Wireless Service.
- Subplan 08 — Audio Service.
- Subplan 11 — UI Components.

## Acceptance Criteria

- [ ] `sone-papdi-bar` binary compiles and starts.
- [ ] Creates a layer-shell surface anchored as configured.
- [ ] Renders left/center/right modules.
- [ ] Clock widget updates every second.
- [ ] Battery/audio/network widgets update on events.
- [ ] Workspaces widget updates on window events.
- [ ] Notification bell shows count and clears on dismiss.
- [ ] Exits cleanly on SIGTERM.

## Task Checklist

- [ ] Set up `gtk4::Application` with app ID.
- [ ] Implement bar window creation with layer-shell anchors.
- [ ] Implement `WorkspacesWidget`.
- [ ] Implement `WindowTitleWidget`.
- [ ] Implement `ClockWidget`.
- [ ] Implement `AudioWidget`.
- [ ] Implement `NetworkWidget`.
- [ ] Implement `BatteryWidget`.
- [ ] Implement `NotificationBellWidget`.
- [ ] Implement `SystrayWidget` stub (full StatusNotifierItem in Phase 7).
- [ ] Implement event subscription to daemon socket.
- [ ] Route events to widgets via `glib::spawn_future_local`.
- [ ] Handle config watch for bar position/height/modules.
- [ ] Add `--config` CLI option.
- [ ] Add basic test running the binary with `--help`.

## Verification

```bash
cargo build -p sone-papdi-bar
cargo test -p sone-papdi-bar
cargo run -p sone-papdi-bar -- --help
```

## Notes / Risks

- Systray full implementation is deferred; provide a placeholder that compiles.
- Bar should not crash if the daemon is unreachable; retry connection.
- Module loading from config could be dynamic; start with a fixed enum of supported modules.
- GTK in CI needs a display; use `xvfb-run` for headless checks.
