# Subplan 18 — Packaging

**Goal:** Provide install scripts, systemd user units, AUR PKGBUILD, Debian packaging, and `cargo xtask` helpers.

**Target deliverable:** `cargo xtask install` builds and installs binaries, systemd units, and default config; AUR package builds.

---

## Scope

- `xtask/src/main.rs` commands: `install`, `package`, `test-e2e`, `clean-all`.
- `packaging/systemd/*.service` user units.
- `packaging/aur/PKGBUILD`.
- `packaging/deb/` control files/rules.
- `scripts/install.sh`.
- `scripts/build-release-archive.sh`.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 16 — Shell Daemon (binaries exist).
- Subplan 17 — Theme Assets (assets to install).

## Acceptance Criteria

- [ ] `cargo xtask install` installs binaries to `~/.cargo/bin`.
- [ ] Systemd user units are installed and reference the correct binaries.
- [ ] Default config is installed to `/usr/share/sone-papdi/default-config.toml`.
- [ ] AUR PKGBUILD produces a valid package.
- [ ] Debian package builds with `dpkg-buildpackage`.
- [ ] `cargo xtask package` creates a release archive.
- [ ] `cargo xtask clean-all` cleans build artifacts and runtime files.

## Task Checklist

- [ ] Implement `xtask` command dispatcher.
- [ ] Implement `cargo xtask install`.
- [ ] Implement `cargo xtask package`.
- [ ] Implement `cargo xtask test-e2e`.
- [ ] Implement `cargo xtask clean-all`.
- [ ] Create `packaging/systemd/rs-shelld.service`.
- [ ] Create `packaging/systemd/sone-papdi-bar.service`.
- [ ] Create `packaging/systemd/sone-papdi-clipboard.service`.
- [ ] Create `packaging/aur/PKGBUILD`.
- [ ] Create `packaging/deb/control`.
- [ ] Create `packaging/deb/rules`.
- [ ] Create `scripts/install.sh`.
- [ ] Create `scripts/build-release-archive.sh`.
- [ ] Write install/uninstall tests (dry-run where possible).

## Verification

```bash
cargo xtask --help
cargo xtask install --dry-run
cargo xtask package --dry-run
```

## Notes / Risks

- Packaging scripts must not hardcode `main`/`master`; detect default branch if git is used.
- Systemd units must include `ConditionEnvironment=WAYLAND_DISPLAY`.
- AUR package should depend on GTK4, layer-shell, wl-clipboard, etc.
- Avoid destructive install scripts; always ask before writing outside `~/.cargo`.
