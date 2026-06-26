# Subplan 03 — Clipboard Engine

**Goal:** Port/adapt the rsclip clipboard daemon so it stores history in SQLite, watches `wl-paste`, and emits typed `ShellEvent::Clipboard` events into the shared bus.

**Target deliverable:** `sone-papdi-clipd` runs standalone, stores clipboard entries, and clients can list/search/delete entries via the IPC command set.

---

## Scope

- `crates/sone-papdi-clipboard/src/lib.rs` and supporting modules.
- `crates/sone-papdi-clipboard/src/main.rs` for the `sone-papdi-clipd` binary.
- SQLite schema matching rsclip (text/link/color/image/file/secret).
- `wl-paste --watch` integration.
- Deduplication, pinning, cleanup, and size limits.
- `ShellCommand::Clipboard*` dispatch.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (event types + socket protocol).
- Subplan 02 — Configuration Engine (`[clipboard]` config section).

## Acceptance Criteria

- [ ] `sone-papdi-clipd` binary compiles and starts.
- [ ] SQLite database is created at `~/.local/share/sone-papdi/clipboard.db`.
- [ ] Copying new clipboard content inserts a row and emits `ClipboardEvent::Stored`.
- [ ] Deduplication works as configured.
- [ ] `ShellCommand::ClipboardList` returns entries.
- [ ] `ShellCommand::ClipboardSearch` returns filtered entries.
- [ ] `ShellCommand::ClipboardDelete` removes entries.
- [ ] Pinned entries are not auto-cleaned.
- [ ] Unit tests for database operations pass.

## Task Checklist

- [ ] Define `ClipEntry` and `ClipEntryKind`.
- [ ] Define `ClipboardEvent` variants.
- [ ] Create SQLite schema and migrations.
- [ ] Implement store/insert with MIME detection.
- [ ] Implement deduplication logic.
- [ ] Implement size limits (`max_text_bytes`, `max_image_bytes`).
- [ ] Implement pinning and cleanup of unpinned old entries.
- [ ] Integrate `wl-paste --watch` child process.
- [ ] Spawn `wtype`/`ydotool`/`xdotool` paste automation when requested.
- [ ] Implement command dispatch for `ClipboardList`, `ClipboardCopy`, `ClipboardDelete`, `ClipboardSearch`.
- [ ] Emit `ClipboardEvent::Stored`, `Copied`, `Deleted`, `Cleared`, `PinnedToggled`, `SearchResult`.
- [ ] Add `sone-papdi-clipd` binary entry point.
- [ ] Write unit tests for DB layer using in-memory SQLite.

## Verification

```bash
cargo build -p sone-papdi-clipboard
cargo test -p sone-papdi-clipboard
cargo clippy -p sone-papdi-clipboard -- -D warnings
```

## Notes / Risks

- OCR with tesseract and favicon fetching are optional; scope them out if they block the deliverable.
- Secret encryption belongs in Subplan 19 (Security Model); this subplan can store secrets as a tagged kind without encryption initially.
- The daemon must gracefully handle the absence of `wl-paste` at runtime (log error, retry).
- Avoid taking a runtime dependency on X11 tools unless the configured paste method requires them.
