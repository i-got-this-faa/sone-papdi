# Subplan 15 — Quick Settings Panel

**Goal:** Build a slide-down/dropdown panel with sliders and toggles for volume, brightness, network, theme, and power actions.

**Target deliverable:** Clicking the bar's right section (or pressing the keybind) opens a panel with working volume slider and power buttons.

---

## Scope

- `crates/rs-shell-ui/src/quick_settings.rs`.
- Volume slider controlling default sink.
- Brightness slider (via `brightnessctl` or `ddcutil` stub).
- Network/Wi-Fi list and connect toggle.
- Battery status summary.
- Theme dark-mode toggle.
- Lock/logout/power buttons.
- IPC commands to trigger actions.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`AudioEvent`, `NetworkEvent`, `ShellEvent`, `ShellCommand`).
- Subplan 02 — Configuration Engine (`[audio]`, `[theme]`, `[keybinds]`).
- Subplan 06 — Battery Service.
- Subplan 07 — Network & Wireless Service.
- Subplan 08 — Audio Service.
- Subplan 10 — Wallpaper & Theme Service (theme toggle).
- Subplan 11 — UI Components.

## Acceptance Criteria

- [ ] Panel opens as a layer-shell overlay below the bar or top-right.
- [ ] Volume slider sends `SetVolume` command and updates on `AudioEvent`.
- [ ] Brightness slider calls `brightnessctl` (graceful fallback if absent).
- [ ] Network section shows current connectivity and Wi-Fi networks.
- [ ] Theme toggle switches dark mode and reloads CSS.
- [ ] Power buttons call configured commands (lock/logout/poweroff).
- [ ] Panel closes on Escape or focus loss.

## Task Checklist

- [ ] Implement `QuickSettingsWindow`.
- [ ] Implement volume slider widget with mute toggle.
- [ ] Implement brightness slider widget.
- [ ] Implement network status row.
- [ ] Implement Wi-Fi network list row with scan/connect.
- [ ] Implement battery status row.
- [ ] Implement theme/dark-mode toggle.
- [ ] Implement lock/logout/power buttons.
- [ ] Subscribe to relevant service events.
- [ ] Implement IPC command dispatch for actions.
- [ ] Add open/close animations.
- [ ] Write unit tests for command mapping.

## Verification

```bash
cargo build -p rs-shell-ui
cargo test -p rs-shell-ui quick_settings
cargo clippy -p rs-shell-ui -- -D warnings
```

## Notes / Risks

- Brightness control is hardware-dependent; gracefully degrade.
- Wi-Fi connect may require privileged D-Bus calls; start with NetworkManager agent.
- Theme toggle may require restarting UI surfaces; document limitation.
- Avoid blocking GTK on external commands; spawn them asynchronously.
