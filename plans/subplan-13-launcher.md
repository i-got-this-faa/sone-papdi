# Subplan 13 — Launcher

**Goal:** Build an app launcher overlay with .desktop indexing, fuzzy search, icon support, and keyboard navigation.

**Target deliverable:** Pressing the configured keybind opens a centered overlay; typing filters apps; Enter launches the selected app.

---

## Scope

- `crates/rs-shell-ui/src/launcher.rs` (or separate `crates/rs-shell-launcher/`).
- `.desktop` file indexing across `XDG_DATA_DIRS/applications`.
- Fuzzy search scoring.
- GTK4 `GtkListView` virtual scrolling.
- Launch via `gio::AppInfo` or `Command`.
- IPC command `ShowLauncher` / `HideLauncher`.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`ShellCommand`, `ShellEvent`).
- Subplan 02 — Configuration Engine (`[launcher]` config).
- Subplan 11 — UI Components.

## Acceptance Criteria

- [ ] Launcher binary/UI opens as a layer-shell overlay.
- [ ] Indexes `.desktop` files at startup and on directory changes.
- [ ] Fuzzy search returns ranked results.
- [ ] Results display icon, name, and description.
- [ ] Keyboard: Up/Down navigate, Enter launch, Escape hide.
- [ ] Supports terminal apps via configured prefix.
- [ ] Handles missing icons gracefully.

## Task Checklist

- [ ] Define `AppEntry` struct.
- [ ] Implement `.desktop` parser (use `freedesktop-desktop-entry` or custom minimal parser).
- [ ] Implement directory watcher for index updates.
- [ ] Implement fuzzy scorer.
- [ ] Implement `GtkListView` + `GtkSingleSelection`.
- [ ] Implement search entry with instant filtering.
- [ ] Implement keyboard event handling.
- [ ] Implement app launch via `gio::AppInfo::launch_uris` or `Command`.
- [ ] Implement terminal-app prefix.
- [ ] Implement `ShowLauncher`/`HideLauncher` IPC handling.
- [ ] Add icon loading helper.
- [ ] Write unit tests for scorer and parser.

## Verification

```bash
cargo build -p rs-shell-ui  # or rs-shell-launcher if split
cargo test -p rs-shell-ui launcher
cargo clippy -p rs-shell-ui -- -D warnings
```

## Notes / Risks

- If launcher becomes large, consider splitting into its own crate `rs-shell-launcher`.
- `.desktop` parsing can be complex; a minimal parser is fine for MVP.
- Icon theme resolution may need `gtk4::IconTheme`.
- Launching apps inside Flatpak/snaps requires special handling; defer.
