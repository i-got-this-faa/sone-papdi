# Subplan 04 — Subscribable API

**Goal:** Provide external tools and internal UI with a typed event stream and command interface over Unix socket and D-Bus, plus a CLI `sone-papdi` binary.

**Target deliverable:** `sone-papdi subscribe battery` streams JSON events; `sone-papdi set-volume 70` works when the daemon is running; D-Bus introspection shows `org.rs_shell.Shell1`.

---

## Scope

- `crates/sone-papdi-api/src/unix_server.rs` — Unix socket server for subscribe/commands.
- `crates/sone-papdi-api/src/dbus_server.rs` — `zbus` D-Bus server.
- `crates/sone-papdi-api/src/cli.rs` or inline in daemon — `sone-papdi` CLI binary.
- Topic filtering (`"*"`, `"battery"`, etc.).
- Command dispatch bridge into the event bus.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`IpcMessage`, `EventBus`, socket paths).
- Subplan 02 — Configuration Engine (optional, for CLI defaults).

## Acceptance Criteria

- [ ] `run_unix_server` accepts connections, handles `Subscribe`, `Command`, `Ping`.
- [ ] Subscribed clients receive only matching events (or all events with `"*"`).
- [ ] Commands return `IpcResponse::Ok`/`OkWith`/`Error`.
- [ ] D-Bus service exports properties, methods, and signals on session bus.
- [ ] D-Bus signals fire when internal events occur.
- [ ] `sone-papdi` CLI supports `subscribe`, one-shot queries, and commands.
- [ ] Unit/integration tests for Unix server and CLI.

## Task Checklist

- [ ] Implement `handle_client` loop with `tokio::select!`.
- [ ] Implement topic matching (`event_topic` helper).
- [ ] Implement `dispatch_command` placeholder/forwarder.
- [ ] Implement D-Bus `ShellInterface` with properties.
- [ ] Implement D-Bus methods: `set_volume`, `toggle_mute`, `set_wallpaper`, `set_theme`, `show_launcher`, `lock_screen`, `switch_workspace`, `reload_config`.
- [ ] Implement D-Bus signals: `battery_changed`, `volume_changed`, `network_changed`, `notification_received`, `theme_changed`.
- [ ] Implement D-Bus bridge loop translating `ShellEvent` to signals.
- [ ] Create `sone-papdi` CLI binary with argument parsing (`clap`).
- [ ] CLI `subscribe <topics...>`: stream events to stdout.
- [ ] CLI one-shot commands: `battery`, `audio volume`, `network status`.
- [ ] CLI imperative commands: `set-volume`, `toggle-mute`, `set-wallpaper`, `set-theme`, `switch-workspace`, `show-launcher`, `lock`, `reload`.
- [ ] Add integration test using temp Unix socket.
- [ ] Add D-Bus integration test using `zbus` in-process connection.

## Verification

```bash
cargo build -p sone-papdi-api
cargo test -p sone-papdi-api
cargo clippy -p sone-papdi-api -- -D warnings
```

## Notes / Risks

- Command dispatch may need to be implemented in `sone-papdi-daemon` rather than `sone-papdi-api`; this subplan should at least define the trait/interface.
- D-Bus session bus tests may need a private bus via `dbus-run-session` or `zbus`'s internal bus.
- Keep the CLI surface minimal for MVP; rich formatting can come later.
- Rate-limiting of commands is deferred to Subplan 19 (Security Model).
