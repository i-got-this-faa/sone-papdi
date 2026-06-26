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

- [x] `cargo build --workspace` exits 0.
- [x] `cargo test --workspace` exits 0 (empty test suites are fine).
- [x] `cargo clippy --workspace -- -D warnings` exits 0.
- [x] Directory layout matches the workspace layout from `implementation-plan.md`.
- [x] CI workflow file exists and runs on PR/push.

## Task Checklist

- [x] Create `Cargo.toml` workspace with members and shared dependencies.
- [x] Create `crates/sone-papdi-core/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-config/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-clipboard/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-api/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-wm/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-services/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-ui/Cargo.toml` + `src/lib.rs`.
- [x] Create `crates/sone-papdi-daemon/Cargo.toml` + `src/main.rs`.
- [x] Create `crates/sone-papdi-bar/Cargo.toml` + `src/main.rs`.
- [x] Create `xtask/Cargo.toml` + `src/main.rs`.
- [x] Create `assets/themes/`, `assets/icons/`, `assets/default-config.toml`.
- [x] Create `packaging/systemd/`, `packaging/aur/`, `packaging/deb/`.
- [x] Create `scripts/install.sh`, `scripts/build-release-archive.sh`.
- [x] Add `.github/workflows/ci.yml`.
- [x] Add `.gitignore` and `rustfmt.toml`.
- [x] Run `cargo build --workspace` and fix errors.
- [x] Run `cargo clippy --workspace -- -D warnings` and fix warnings.

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
