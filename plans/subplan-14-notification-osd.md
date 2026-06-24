# Subplan 14 — Notification OSD

**Goal:** Render on-screen notifications and a notification-center drawer from `NotificationEvent` events.

**Target deliverable:** A notification from any D-Bus app appears as an OSD widget; dismissed notifications accumulate in a history drawer.

---

## Scope

- `crates/rs-shell-ui/src/notifications.rs`.
- OSD popup stack (top-right default).
- Per-notification widget: icon, app name, summary, body, actions, dismiss.
- Timeout progress bar and urgency styling.
- Notification center drawer (right-anchored, full height).
- SQLite persistence for history (or in-memory if deferred).

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`NotificationEvent`, `Notification`, `ShellEvent`).
- Subplan 02 — Configuration Engine (`[notifications]` config).
- Subplan 09 — Notification Service (emits events).
- Subplan 11 — UI Components.

## Acceptance Criteria

- [ ] OSD popup appears on `NotificationEvent::Received`.
- [ ] Multiple notifications stack and animate.
- [ ] Urgency levels affect styling and timeout.
- [ ] Critical notifications do not auto-dismiss.
- [ ] Clicking body invokes default action.
- [ ] Dismiss button emits `Closed`.
- [ ] Notification center drawer shows history.
- [ ] "Clear all" dismisses visible history.

## Task Checklist

- [ ] Define `NotificationWidget`.
- [ ] Implement timeout handling with progress bar.
- [ ] Implement urgency-based CSS classes.
- [ ] Implement action buttons and `ActionInvoked` emission.
- [ ] Implement dismiss gesture/button.
- [ ] Implement OSD stack container anchored top-right.
- [ ] Implement notification center drawer.
- [ ] Implement history persistence (SQLite or in-memory list).
- [ ] Subscribe to `NotificationEvent` via daemon socket.
- [ ] Implement `ShowNotificationCenter` IPC command.
- [ ] Write unit tests for timeout logic and history state.

## Verification

```bash
cargo build -p rs-shell-ui
cargo test -p rs-shell-ui notifications
cargo clippy -p rs-shell-ui -- -D warnings
```

## Notes / Risks

- Body markup (`<b>`, etc.) must be sanitized before `set_use_markup`.
- Image data hints are out of scope; icon names only.
- OSD positioning must not overlap the bar; respect config `gap`.
- History persistence should be lightweight; consider a simple JSON file for MVP.
