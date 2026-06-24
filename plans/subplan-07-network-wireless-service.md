# Subplan 07 — Network & Wireless Service

**Goal:** Monitor NetworkManager connectivity, interfaces, and Wi-Fi access points, emitting `NetworkEvent` and `WirelessEvent` on the bus.

**Target deliverable:** Plug/unplug ethernet, connect/disconnect Wi-Fi, or run a scan and see live events via `rs-shell subscribe network wireless`.

---

## Scope

- `crates/rs-shell-services/src/network.rs`.
- `crates/rs-shell-services/src/wireless.rs`.
- NetworkManager `StateChanged`, `DeviceAdded/Removed`, active connection tracking.
- Wi-Fi device proxy: AP list, signal strength, security flags.
- Wired interface enumeration.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`NetworkEvent`, `NetworkState`, `NetworkInterface`, `WirelessEvent`, `WifiNetwork`, `ShellEvent`).
- Subplan 02 — Configuration Engine (`NetworkConfig`).

## Acceptance Criteria

- [ ] Emits `NetworkEvent::ConnectivityChanged` on NM state changes.
- [ ] Emits `NetworkEvent::StateChanged` with interface list.
- [ ] Emits `InterfaceUp`/`InterfaceDown` on device changes.
- [ ] Wireless service emits `Connected`, `Disconnected`, `SignalStrengthChanged`, `NetworksScanned`.
- [ ] Wi-Fi scan can be triggered via command/DBus.
- [ ] Handles NetworkManager absence gracefully.

## Task Checklist

- [ ] Define NetworkManager root proxy.
- [ ] Define NM device and active connection proxies.
- [ ] Implement connectivity state mapping.
- [ ] Implement active connection → interface resolution.
- [ ] Implement interface enumeration (name, kind, IP, state).
- [ ] Implement wireless device proxy.
- [ ] Implement AP object parsing (SSID, strength, security, connected).
- [ ] Implement scan trigger and result emission.
- [ ] Implement signal strength change detection.
- [ ] Implement `run(bus, config)` for network.
- [ ] Implement `run(bus)` for wireless.
- [ ] Write unit tests for state mapping and AP parsing.

## Verification

```bash
cargo build -p rs-shell-services --features networkmanager
cargo test -p rs-shell-services network
cargo clippy -p rs-shell-services -- -D warnings
```

## Notes / Risks

- SSIDs are byte arrays in D-Bus; convert lossily to `String` for display.
- `NetworkManager` vs `iwd` backend is feature-gated; default to NetworkManager for this subplan.
- Test environments may not have NM; keep parsing logic unit-testable.
- Interface kind detection should prefer device type over name heuristics.
