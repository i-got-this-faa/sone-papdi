# Subplan 01 — Core IPC & Event Bus

**Goal:** Implement the shared nervous system used by every other crate: typed events, in-process broadcast bus, cross-process Unix socket protocol, and runtime paths.

**Target deliverable:** Other crates can `use rs_shell_core::{events, bus, ipc, paths}`; a standalone test can publish/subscribe events and round-trip an IPC message over a Unix socket.

---

## Scope

- `crates/rs-shell-core/src/events.rs` — `ShellEvent` enum and all payload types.
- `crates/rs-shell-core/src/bus.rs` — `EventBus` wrapper around `tokio::sync::broadcast`.
- `crates/rs-shell-core/src/ipc.rs` — length-prefixed JSON frame protocol + `IpcMessage`/`IpcResponse`/`ShellCommand`.
- `crates/rs-shell-core/src/paths.rs` — XDG runtime dir helpers.
- Unit tests for serialization, bus filtering, and IPC round-trip.

## Dependencies

- Subplan 00 — Scaffold (workspace exists).

## Acceptance Criteria

- [ ] `ShellEvent` and all nested types compile and derive `Serialize`/`Deserialize`.
- [ ] `EventBus::publish`/`subscribe` works; `subscribe_filtered` returns a `Stream`.
- [ ] `send_message`/`recv_message` round-trip over a `tokio::net::UnixStream`.
- [ ] Socket paths resolve under `/run/user/<uid>/`.
- [ ] All public types have doc comments.
- [ ] Unit tests pass.

## Task Checklist

- [ ] Define `ShellEvent` enum with `#[serde(tag = "kind", content = "payload")]`.
- [ ] Define `BatteryState`, `BatteryStatus`, `BatteryEvent`.
- [ ] Define `NetworkState`, `NetworkConnectivity`, `NetworkInterface`, `InterfaceKind`, `InterfaceState`, `NetworkEvent`.
- [ ] Define `AudioState`, `AudioEvent`.
- [ ] Define `WifiNetwork`, `WirelessEvent`.
- [ ] Define `Notification`, `NotificationUrgency`, `NotificationEvent`, `CloseReason`.
- [ ] Define `WindowInfo`, `WindowState`, `WindowEvent`.
- [ ] Define `WallpaperEvent`, `ThemeEvent`, `ShellLifecycleEvent`.
- [ ] Implement `EventBus` with `BUS_CAPACITY = 512`.
- [ ] Implement `publish`, `subscribe`, `subscribe_filtered`.
- [ ] Define `IpcMessage`, `IpcResponse`, `ShellCommand` enums.
- [ ] Implement `send_message` using 4-byte BE length prefix.
- [ ] Implement `recv_message` reading exact length.
- [ ] Implement `runtime_dir()`, `daemon_socket()`, `clipboard_socket()`, `bar_socket()`.
- [ ] Add `mod.rs`/`lib.rs` re-exports.
- [ ] Write unit tests for event serialization.
- [ ] Write unit test for IPC round-trip using temp Unix socket.
- [ ] Write unit test for filtered subscription.

## Verification

```bash
cargo test -p rs-shell-core
cargo clippy -p rs-shell-core -- -D warnings
```

## Notes / Risks

- `WindowInfo.id` is intentionally opaque (`String`); WM adapters decide format.
- `ShellCommand` is large; consider boxing later if compile times suffer.
- Keep `ShellEvent` non-exhaustive externally if we expect to add variants.
- `subscribe_filtered` must handle lagged broadcast errors gracefully.
