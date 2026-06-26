# Subplan 08 — Audio Service

**Goal:** Monitor and control audio sinks/sources via PipeWire (with optional PulseAudio fallback), emitting `ShellEvent::Audio` events.

**Target deliverable:** Changing the default sink or volume produces `AudioEvent::VolumeChanged`, `MuteToggled`, or `DefaultSinkChanged` on the bus.

---

## Scope

- `crates/sone-papdi-services/src/audio.rs`.
- PipeWire Rust bindings integration.
- Tracking default sink/source via PipeWire metadata.
- Volume/mute change detection.
- Optional PulseAudio fallback behind feature flag.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (`AudioEvent`, `AudioState`, `ShellEvent`).
- Subplan 02 — Configuration Engine (`AudioConfig`).

## Acceptance Criteria

- [ ] Connects to PipeWire and tracks default sink/source.
- [ ] Emits `VolumeChanged` when sink volume changes.
- [ ] Emits `MuteToggled` when mute state changes.
- [ ] Emits `DefaultSinkChanged`/`DefaultSourceChanged`.
- [ ] PulseAudio fallback compiles behind feature flag.
- [ ] PipeWire main loop runs in a dedicated thread; async side receives events via channel.
- [ ] Handles missing PipeWire/PulseAudio gracefully.

## Task Checklist

- [ ] Add `pipewire` dependency behind feature flag.
- [ ] Add `libpulse-binding` fallback dependency behind feature flag.
- [ ] Implement PipeWire thread: `MainLoop`, `Context`, `Core`, `Registry`.
- [ ] Implement node filtering (sink/source).
- [ ] Implement metadata listener for default sink/source.
- [ ] Implement param listener for volume/mute.
- [ ] Forward events via `tokio::sync::mpsc` to async runtime.
- [ ] Implement PulseAudio fallback using `context.subscribe`.
- [ ] Implement `run(bus, config)` entry point.
- [ ] Write unit tests for state-change detection logic.

## Verification

```bash
cargo build -p sone-papdi-services --features pipewire
cargo build -p sone-papdi-services --features pulseaudio
cargo test -p sone-papdi-services audio
cargo clippy -p sone-papdi-services -- -D warnings
```

## Notes / Risks

- PipeWire's Rust API is not fully async; the dedicated thread + channel pattern is required.
- Volume is often reported as cubic volume; convert to linear 0.0–1.0 for events.
- Build-time feature flags prevent forcing PipeWire on systems without it.
- Mocking PipeWire is hard; rely on logic unit tests and manual smoke tests.
