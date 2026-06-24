# Subplan 16 — Shell Daemon

**Goal:** Wire all crates together into the `rs-shelld` orchestrator binary: config, event bus, WM adapter, services, API servers, and lifecycle events.

**Target deliverable:** `rs-shelld` starts, loads config, spawns services, hosts the Unix/D-Bus API, and keeps running until terminated.

---

## Scope

- `crates/rs-shell-daemon/src/main.rs`.
- `rs-shelld` binary.
- Initializes tracing, config manager, event bus.
- Spawns WM adapter and bridges `WindowEvent` → `ShellEvent::Window`.
- Spawns services: battery, network, audio, wireless, notifications, clipboard, wallpaper/theme apply.
- Hosts Unix socket and D-Bus API servers.
- Publishes lifecycle events and monitors service tasks.
- CLI `rs-shell` binary entry point.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus.
- Subplan 02 — Configuration Engine.
- Subplan 03 — Clipboard Engine.
- Subplan 04 — Subscribable API.
- Subplan 05 — WM Bridge.
- Subplan 06 — Battery Service.
- Subplan 07 — Network & Wireless Service.
- Subplan 08 — Audio Service.
- Subplan 09 — Notification Service.
- Subplan 10 — Wallpaper & Theme Service.

## Acceptance Criteria

- [ ] `rs-shelld` compiles and runs.
- [ ] Creates Unix socket at `rs_shell_core::paths::daemon_socket()`.
- [ ] Acquires D-Bus name `org.rs_shell.Shell1` (if session bus available).
- [ ] Spawns all configured services.
- [ ] Bridges WM events onto the global bus.
- [ ] Applies wallpaper and theme at startup.
- [ ] Re-applies wallpaper/theme on config change.
- [ ] Emits `DaemonStarted` and `ServiceFailed` lifecycle events.
- [ ] Handles SIGTERM/SIGINT gracefully.

## Task Checklist

- [ ] Set up `tracing_subscriber` with env filter.
- [ ] Initialize `ConfigManager` and spawn watcher.
- [ ] Initialize `EventBus`.
- [ ] Detect/select WM adapter and spawn event bridge.
- [ ] Spawn battery service task.
- [ ] Spawn network service task.
- [ ] Spawn audio service task.
- [ ] Spawn wireless service task.
- [ ] Spawn notification service task.
- [ ] Spawn clipboard service task.
- [ ] Apply wallpaper at startup.
- [ ] Apply theme at startup.
- [ ] Watch config changes and re-apply wallpaper/theme.
- [ ] Spawn Unix socket API server.
- [ ] Spawn D-Bus API server.
- [ ] Publish `DaemonStarted`.
- [ ] Monitor service join handles and emit `ServiceFailed` on error.
- [ ] Implement graceful shutdown.
- [ ] Add `--config` CLI option.
- [ ] Add `--version` / `--help`.
- [ ] Add smoke test that starts daemon on a private socket/D-Bus and asserts `DaemonStarted`.

## Verification

```bash
cargo build -p rs-shell-daemon
cargo test -p rs-shell-daemon
cargo run -p rs-shell-daemon -- --help
```

## Notes / Risks

- The daemon must not require a display; keep GTK out of this crate.
- Service startup order should not matter; design tasks to be independent.
- D-Bus name acquisition may fail in minimal environments; log and continue.
- A service crash should not crash the daemon; monitor and report only.
- Clipboard service may be feature-gated if it brings heavy dependencies.
