# Subplan 05 — Window Manager Bridge

**Goal:** Abstract compositor-specific window events and commands behind a single `WmAdapter` trait, with auto-detection and implementations for wlr-foreign-toplevel, Hyprland, and sway.

**Target deliverable:** `rs_shell_wm::detect_wm()` returns an adapter; running under a supported compositor emits `WindowEvent` variants for open/close/focus/workspace changes.

---

## Scope

- `crates/rs-shell-wm/src/adapter.rs` — `WmAdapter` trait.
- `crates/rs-shell-wm/src/wlr_toplevel.rs` — generic Wayland adapter.
- `crates/rs-shell-wm/src/hyprland.rs` — Hyprland socket IPC adapter.
- `crates/rs-shell-wm/src/sway.rs` — sway/i3-compatible IPC adapter.
- `crates/rs-shell-wm/src/lib.rs` — detection and adapter factory.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`WindowEvent`, `WindowInfo`, `WindowState`).

## Acceptance Criteria

- [ ] `WmAdapter` trait compiles with async methods.
- [ ] `detect_wm()` chooses Hyprland > sway > wlr fallback based on env vars.
- [ ] Hyprland adapter connects to event socket and emits window events.
- [ ] sway adapter parses i3-IPC events.
- [ ] Generic adapter binds `zwlr_foreign_toplevel_manager_v1`.
- [ ] Commands `focus`, `close`, `move_to_workspace`, `switch_workspace`, `workspaces` work on at least one adapter.
- [ ] Unit tests for parsing logic (no compositor required).

## Task Checklist

- [ ] Define `WmAdapter` async trait.
- [ ] Implement `detect_wm()` and `adapter_for(name)`.
- [ ] Implement Hyprland socket path resolution (`HYPRLAND_INSTANCE_SIGNATURE`).
- [ ] Implement Hyprland command/request socket helper.
- [ ] Implement Hyprland event stream parser.
- [ ] Implement Hyprland `windows()`, `focus()`, `close()`, `move_to_workspace()`, `switch_workspace()`, `workspaces()`.
- [ ] Implement sway socket path resolution (`SWAYSOCK`).
- [ ] Implement sway IPC header parser.
- [ ] Implement sway event subscription and parser.
- [ ] Implement sway command dispatch.
- [ ] Implement wlr-foreign-toplevel Wayland dispatch.
- [ ] Map all adapter outputs to `WindowEvent` and `WindowInfo`.
- [ ] Write parser unit tests for Hyprland and sway event lines.

## Verification

```bash
cargo build -p rs-shell-wm
cargo test -p rs-shell-wm
cargo clippy -p rs-shell-wm -- -D warnings
```

## Notes / Risks

- Wayland client tests cannot run headless easily; rely on parser unit tests.
- Hyprland event format may change between versions; keep parsing defensive.
- `wlr-foreign-toplevel` does not expose workspace names; workspace tracking will need compositor-specific adapters.
- Niri adapter is out of scope for this subplan; can be added later without affecting others.
