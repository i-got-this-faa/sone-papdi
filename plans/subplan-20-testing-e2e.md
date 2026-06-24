# Subplan 20 — E2E Testing

**Goal:** Provide automated end-to-end tests that run the full daemon and UI in a controlled Wayland session, verifying integration between services, API, and widgets.

**Target deliverable:** `cargo xtask test-e2e` runs a headless sway session, starts the daemon, injects events, and asserts expected behavior.

---

## Scope

- `xtask/src/test_e2e.rs`.
- Headless compositor harness (sway with dummy outputs, or cage).
- Test fixtures for config and runtime directories.
- Fake D-Bus services where needed (UPower, NetworkManager).
- E2E tests covering:
  - daemon startup and API connectivity
  - clipboard round-trip
  - notification flow
  - bar visibility (pixel/geometry checks)
  - launcher basic search

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 16 — Shell Daemon (full system to test).
- Subplan 18 — Packaging (`cargo xtask test-e2e` entry point).

## Acceptance Criteria

- [ ] `cargo xtask test-e2e` compiles and runs.
- [ ] A headless Wayland compositor launches for tests.
- [ ] Daemon starts inside the test environment.
- [ ] API client can connect and subscribe.
- [ ] A clipboard event can be injected and observed.
- [ ] A notification can be injected and observed.
- [ ] Bar window is detectable (via `swaymsg` or layer-shell introspection).
- [ ] Tests clean up runtime files on exit.

## Task Checklist

- [ ] Choose compositor for headless tests (sway, cage, or weston).
- [ ] Implement compositor process spawn and readiness wait.
- [ ] Implement isolated runtime/config/data directories.
- [ ] Implement daemon spawn in test environment.
- [ ] Implement bar spawn in test environment.
- [ ] Implement API client helper for assertions.
- [ ] Implement D-Bus mock helpers for UPower/NetworkManager.
- [ ] Write test: daemon starts and emits `DaemonStarted`.
- [ ] Write test: clipboard inject → stored → listed.
- [ ] Write test: notification inject → received → closed.
- [ ] Write test: bar appears as layer-shell surface.
- [ ] Write test: launcher opens and returns search results.
- [ ] Implement cleanup on panic/success.
- [ ] Add E2E test step to CI.

## Verification

```bash
cargo xtask test-e2e
```

## Notes / Risks

- E2E tests are flaky by nature; retry compositor spawn once.
- GTK surfaces need a display; use `xvfb-run` if the compositor does not provide one.
- Tests must not interfere with the host session; use private `XDG_RUNTIME_DIR`.
- Running headless sway may need kernel/dri permissions in CI.
- Some tests may be `#[ignore]` by default if they require special setup.
