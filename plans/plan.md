# sone-papdi Master Plan

> Derived from `implementation-plan.md`.
> Each linked subplan is an independent deliverable with its own progress tracker.

## Goal

Build **sone-papdi**: a complete Wayland desktop shell (panel, launcher, notification center, clipboard manager, quick settings, wallpaper/theme, lock-screen hooks) using a daemon/UI split, GTK4 layer-shell, typed event bus, Unix-socket + D-Bus API, and WM-agnostic bridge.

## Subplans

| ID | Subplan | Purpose | Depends on | Status |
|---|---|---|---|---|
| 00 | [Scaffold](./subplan-00-scaffold.md) | Workspace, crate skeletons, CI, root manifests | — | ✅ Done |
| 01 | [Core IPC & Event Bus](./subplan-01-core.md) | `ShellEvent` types, `EventBus`, length-prefixed JSON IPC, socket paths | 00 | 🚧 Not started |
| 02 | [Configuration Engine](./subplan-02-config.md) | TOML schema, default config, hot-reload watcher | 00 | 🚧 Not started |
| 03 | [Clipboard Engine](./subplan-03-clipboard.md) | Clipboard daemon, SQLite history, `ShellEvent::Clipboard` | 01, 02 | 🚧 Not started |
| 04 | [Subscribable API](./subplan-04-api.md) | Unix socket server + D-Bus `org.rs_shell.Shell1` + CLI `sone-papdi` | 01, 02 | 🚧 Not started |
| 05 | [WM Bridge](./subplan-05-wm-bridge.md) | `WmAdapter` trait, wlr-foreign-toplevel, Hyprland/sway/niri adapters | 01 | 🚧 Not started |
| 06 | [Battery Service](./subplan-06-battery-service.md) | UPower D-Bus battery state + thresholds | 01, 02 | 🚧 Not started |
| 07 | [Network & Wireless Service](./subplan-07-network-wireless-service.md) | NetworkManager connectivity, interfaces, Wi-Fi scan/connect | 01, 02 | 🚧 Not started |
| 08 | [Audio Service](./subplan-08-audio-service.md) | PipeWire/PulseAudio sink/source events | 01, 02 | 🚧 Not started |
| 09 | [Notification Service](./subplan-09-notification-service.md) | `org.freedesktop.Notifications` server | 01 | 🚧 Not started |
| 10 | [Wallpaper & Theme Service](./subplan-10-wallpaper-theme-service.md) | Wallpaper backends, theme CSS resolution, gsettings | 01, 02 | 🚧 Not started |
| 11 | [UI Components](./subplan-11-ui-components.md) | Shared GTK4 layer-shell helpers, CSS loader, widgets | 00 | 🚧 Not started |
| 12 | [Bar / Panel](./subplan-12-bar.md) | `sone-papdi-bar` binary with modules | 01, 02, 05, 06, 07, 08, 11 | 🚧 Not started |
| 13 | [Launcher](./subplan-13-launcher.md) | App launcher with .desktop indexing & fuzzy search | 01, 02, 11 | 🚧 Not started |
| 14 | [Notification OSD](./subplan-14-notification-osd.md) | On-screen notifications + history drawer | 01, 09, 11 | 🚧 Not started |
| 15 | [Quick Settings Panel](./subplan-15-quick-settings.md) | Volume, brightness, network, theme toggles | 01, 02, 06, 07, 08, 11 | 🚧 Not started |
| 16 | [Shell Daemon](./subplan-16-daemon.md) | `rs-shelld` orchestrator wiring all services | 01–10 | 🚧 Not started |
| 17 | [Theme Assets](./subplan-17-theme-assets.md) | Bundled CSS themes (nonchalant-dark, etc.) | 11 | 🚧 Not started |
| 18 | [Packaging](./subplan-18-packaging.md) | systemd units, AUR PKGBUILD, deb, xtask scripts | 16 | 🚧 Not started |
| 19 | [Security Model](./subplan-19-security.md) | IPC permissions, secret encryption, systemd hardening | 03, 16, 18 | 🚧 Not started |
| 20 | [E2E Testing](./subplan-20-testing-e2e.md) | `cargo xtask test-e2e`, sway test harness | 16 | 🚧 Not started |

## Dependency Graph

```text
00 Scaffold
│
├─► 01 Core ──┬─► 03 Clipboard
│             ├─► 04 API
│             ├─► 05 WM Bridge
│             ├─► 06 Battery Service
│             ├─► 07 Network & Wireless Service
│             ├─► 08 Audio Service
│             ├─► 09 Notification Service
│             ├─► 10 Wallpaper & Theme Service
│             └─► 16 Daemon ◄────┐
│                                │
02 Config ──┬─► 03 Clipboard     │
            ├─► 04 API           │
            ├─► 06 Battery       │
            ├─► 07 Network       │
            ├─► 08 Audio         │
            ├─► 10 Wallpaper     │
            ├─► 12 Bar           │
            ├─► 13 Launcher      │
            ├─► 15 Quick Settings│
            └────────────────────┘

11 UI Components ──┬─► 12 Bar
                   ├─► 13 Launcher
                   ├─► 14 Notification OSD
                   ├─► 15 Quick Settings
                   └─► 17 Theme Assets

05 WM Bridge ──────► 12 Bar
06 Battery ────────► 12 Bar, 15 Quick Settings
07 Network ────────► 12 Bar, 15 Quick Settings
08 Audio ──────────► 12 Bar, 15 Quick Settings
09 Notifications ──► 14 Notification OSD
10 Wallpaper/Theme ─► 15 Quick Settings (theme toggle), daemon startup

16 Daemon ─────────► 18 Packaging, 19 Security, 20 E2E Testing
```

## Phased Roadmap (from implementation-plan.md)

| Phase | Subplans | Milestone |
|---|---|---|
| 0 — Scaffold | 00, 01, 02 | `rs-shelld` boots, socket accepts connections, `sone-papdi subscribe '*'` streams nothing |
| 1 — Clipboard | 03 | Clipboard works via shared bus |
| 2 — Services | 06, 07, 08, 09, 10 | Live battery/audio/network/notification/wallpaper events |
| 3 — WM Bridge | 05 | Window lifecycle events on the bus |
| 4 — Bar MVP | 11, 12, 17 | Functional status bar visible on screen |
| 5 — Launcher + OSD | 13, 14 | Full notification flow; launcher accessible |
| 6 — Quick Settings | 15 | Everything configurable from one panel |
| 7 — Polish & Ship | 18, 19, 20 | v0.1.0 release, AUR-installable |

## Progress Summary

Update this line as subplans complete:

**Completed:** 1 / 21

### Status Key

- 🚧 Not started
- 🔄 In progress
- ✅ Done
- ⏸️ Blocked
- ❌ Failed / abandoned

## How to Use This Plan

1. Pick the next unblocked subplan by dependency order.
2. Open its `subplan-XX-*.md` file and work through the task checklist.
3. When a subplan is done, update its status and this master table.
4. If a subplan discovers a new dependency or needs to split scope, update both files and record the decision in a note.
