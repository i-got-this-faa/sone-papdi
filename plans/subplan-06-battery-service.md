# Subplan 06 — Battery Service

**Goal:** Monitor the system battery over UPower D-Bus and emit typed `ShellEvent::Battery` events, including low/critical threshold crossings.

**Target deliverable:** When running, every UPower `PropertiesChanged` signal results in a `BatteryEvent::StateChanged` on the event bus; crossing warn/critical levels emits `LevelLow`/`LevelCritical`.

---

## Scope

- `crates/sone-papdi-services/src/battery.rs`.
- UPower display device discovery.
- `zbus` proxy for UPower device.
- State polling fallback if signals are unreliable.
- Threshold configuration from `[battery]`.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`BatteryEvent`, `BatteryState`, `BatteryStatus`, `ShellEvent`).
- Subplan 02 — Configuration Engine (`BatteryConfig`).

## Acceptance Criteria

- [ ] Connects to system D-Bus and discovers the display battery device (or configured path).
- [ ] Emits `BatteryEvent::StateChanged` with percentage, status, time-to-empty/full, voltage, temperature.
- [ ] Emits `LevelLow` when percentage ≤ `warn_level`.
- [ ] Emits `LevelCritical` when percentage ≤ `critical_level`.
- [ ] Emits `PluggedIn`/`Unplugged` on status transitions.
- [ ] Handles missing battery gracefully (desktop PCs).
- [ ] Unit tests for threshold logic.

## Task Checklist

- [ ] Define UPower device proxy with `zbus::proxy`.
- [ ] Implement `find_display_device()`.
- [ ] Implement `read_battery_state()`.
- [ ] Subscribe to `PropertiesChanged` and translate to `BatteryState`.
- [ ] Implement threshold crossing detection with hysteresis.
- [ ] Implement `run(bus, config)` entry point.
- [ ] Add graceful shutdown on bus disconnect.
- [ ] Write unit tests for threshold logic.
- [ ] Write mock D-Bus test if feasible.

## Verification

```bash
cargo build -p sone-papdi-services --features default
cargo test -p sone-papdi-services battery
cargo clippy -p sone-papdi-services -- -D warnings
```

## Notes / Risks

- UPower may not be available in all test environments; design the service to exit cleanly and log.
- Threshold hysteresis prevents spam when percentage hovers at exactly 20%.
- `State` enum mapping must cover UPower's `Charging`, `Discharging`, `FullyCharged`, `PendingCharge`, `PendingDischarge`, `Unknown`.
