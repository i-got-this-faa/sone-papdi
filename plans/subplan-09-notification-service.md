# Subplan 09 — Notification Service

**Goal:** Implement the `org.freedesktop.Notifications` D-Bus service, replacing dunst/mako and feeding `ShellEvent::Notification` events into the bus.

**Target deliverable:** Any app sending a notification via D-Bus appears as a `NotificationEvent::Received`; actions and closes are emitted back.

---

## Scope

- `crates/rs-shell-services/src/notifications.rs`.
- `zbus` interface implementing `org.freedesktop.Notifications`.
- Methods: `Notify`, `CloseNotification`, `GetCapabilities`, `GetServerInformation`.
- Signals: `NotificationClosed`, `ActionInvoked`.
- In-memory history (persistence belongs to notification OSD).

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`Notification`, `NotificationEvent`, `NotificationUrgency`, `CloseReason`, `ShellEvent`).

## Acceptance Criteria

- [ ] Acquires `org.freedesktop.Notifications` name on session bus.
- [ ] `Notify` parses hints, urgency, actions, and emits `Received`.
- [ ] `CloseNotification` emits `Closed` with `AppClosed` reason.
- [ ] `replaces_id` updates existing notifications.
- [ ] `GetCapabilities` returns supported capability list.
- [ ] `GetServerInformation` returns name/vendor/version/spec version.
- [ ] `ActionInvoked` signal fires when an action is triggered.
- [ ] Unit tests for parsing and ID allocation.

## Task Checklist

- [ ] Define `NotificationServer` struct.
- [ ] Implement `zbus` interface with `#[interface]`.
- [ ] Implement urgency parsing from hints.
- [ ] Implement action parsing from flat `Vec<&str>`.
- [ ] Implement ID allocation / replacement.
- [ ] Implement history retention with configurable limit.
- [ ] Emit `NotificationEvent::Received` and `Closed`.
- [ ] Implement D-Bus signal emission for `ActionInvoked`.
- [ ] Implement `run(bus)` entry point.
- [ ] Write unit tests for `Notify` parsing.
- [ ] Write D-Bus integration test using private session bus.

## Verification

```bash
cargo build -p rs-shell-services
cargo test -p rs-shell-services notifications
cargo clippy -p rs-shell-services -- -D warnings
```

## Notes / Risks

- Body markup (`<b>`, `<i>`) should be accepted as a capability but rendering is deferred to the OSD.
- Image data hints (`image-data`) are out of scope for MVP; icon names are enough.
- Notifications from existing apps may rely on specific capability strings; match common ones.
- Ensure only one notification server runs per session to avoid D-Bus name contention.
