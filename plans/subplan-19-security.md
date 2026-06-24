# Subplan 19 — Security Model

**Goal:** Harden IPC, clipboard secrets, and systemd sandboxing so sone-papdi can run safely in a user session.

**Target deliverable:** Unix sockets are `0600`; secret clipboard entries are encrypted; systemd units use hardening directives.

---

## Scope

- `crates/rs-shell-core/src/ipc.rs` socket permissions.
- Secret clipboard encryption in `rs-shell-clipboard`.
- IPC command allowlist and rate limiting.
- Systemd unit hardening.
- Documentation of threat model.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 01 — Core IPC & Event Bus (socket paths).
- Subplan 03 — Clipboard Engine (secret entries).
- Subplan 16 — Shell Daemon (systemd units).
- Subplan 18 — Packaging (unit files).

## Acceptance Criteria

- [ ] Daemon socket is created with mode `0600` and owner UID.
- [ ] Clipboard socket is created with mode `0600`.
- [ ] Secret clipboard entries are encrypted at rest.
- [ ] Secret entries auto-clear from Wayland clipboard after timeout.
- [ ] IPC command allowlist rejects unknown commands.
- [ ] Rate limit: max 100 commands/second per client.
- [ ] Systemd units include `NoNewPrivileges`, `PrivateTmp`, `ProtectSystem`, `ProtectHome`.
- [ ] Security documentation exists.

## Task Checklist

- [ ] Implement socket creation with `fchown`/`chmod 0600`.
- [ ] Choose encryption: AES-256-GCM with key from libsecret/kernel keyring.
- [ ] Implement secret key retrieval/creation.
- [ ] Implement encrypt/decrypt helpers.
- [ ] Encrypt secret entries before SQLite insert.
- [ ] Decrypt secret entries only for paste or explicit reveal.
- [ ] Implement secret auto-clear timeout from Wayland clipboard.
- [ ] Implement IPC command allowlist.
- [ ] Implement per-connection command rate limiter.
- [ ] Update systemd units with hardening.
- [ ] Document security model and threat assumptions.
- [ ] Write unit tests for encryption helpers and rate limiter.

## Verification

```bash
cargo test -p rs-shell-core socket_perms
cargo test -p rs-shell-clipboard secret_encrypt
cargo test -p rs-shell-api rate_limit
cargo build -p rs-shell-daemon
```

## Notes / Risks

- libsecret/keyring availability varies; provide fallback to file-backed key with restricted permissions.
- Encryption adds dependencies (`ring`, `aes-gcm`, or `rustcrypto` crates).
- Rate limiting must not block legitimate rapid commands.
- Systemd hardening must still allow required paths (`~/.config/rs-shell`, `~/.local/share/rs-shell`, `/run/user/<uid>`).
