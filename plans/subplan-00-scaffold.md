# Subplan 00 — Project Scaffold

**Goal:** Create the repository/workspace skeleton so that every other subplan has a place to land.

**Target deliverable:** `cargo build --workspace` succeeds at the workspace level, all crate directories exist, and CI lint/check passes.

---

## Scope

This subplan is **pure structure**. No functional services, no UI, no D-Bus.

- Root `Cargo.toml` workspace manifest.
- Empty crate skeletons for every planned crate.
- `xtask` scaffolding for build automation.
- `assets/`, `packaging/`, `scripts/` directories.
- A minimal `.gitignore`, `rustfmt.toml`, `clippy` baseline.
- CI workflow (GitHub Actions) that runs `cargo check`, `cargo test`, `cargo clippy`.

## Dependencies

None.

## Acceptance Criteria

- [ ] `cargo build --workspace` exits 0.
- [ ] `cargo test --workspace` exits 0 (empty test suites are fine).
- [ ] `cargo clippy --workspace -- -D warnings` exits 0.
- [ ] Directory layout matches the workspace layout from `implementation-plan.md`.
- [ ] CI workflow file exists and runs on PR/push.

## Task Checklist

- [ ] Create `Cargo.toml` workspace with members and shared dependencies.
- [ ] Create `crates/rs-shell-core/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-config/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-clipboard/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-api/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-wm/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-services/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-ui/Cargo.toml` + `src/lib.rs`.
- [ ] Create `crates/rs-shell-daemon/Cargo.toml` + `src/main.rs`.
- [ ] Create `crates/rs-shell-bar/Cargo.toml` + `src/main.rs`.
- [ ] Create `xtask/Cargo.toml` + `src/main.rs`.
- [ ] Create `assets/themes/`, `assets/icons/`, `assets/default-config.toml`.
- [ ] Create `packaging/systemd/`, `packaging/aur/`, `packaging/deb/`.
- [ ] Create `scripts/install.sh`, `scripts/build-release-archive.sh`.
- [ ] Add `.github/workflows/ci.yml`.
- [ ] Add `.gitignore` and `rustfmt.toml`.
- [ ] Run `cargo build --workspace` and fix errors.
- [ ] Run `cargo clippy --workspace -- -D warnings` and fix warnings.

## Verification

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Notes / Risks

- Keep `Cargo.toml` dependency versions in sync with `implementation-plan.md` Section 17.
- Do not enable heavy features like `gtk4/v4_12` or `pipewire` until needed; empty crates should compile quickly.
- The `xtask` crate may need a small stub command to satisfy `cargo build --workspace`.
