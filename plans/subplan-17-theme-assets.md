# Subplan 17 — Theme Assets

**Goal:** Provide the bundled CSS themes and fallback icons referenced by the theme service and UI crates.

**Target deliverable:** `assets/themes/*.css` files exist and are loadable by `sone-papdi-ui`; the default theme matches `nonchalant-dark`.

---

## Scope

- `assets/themes/nonchalant-dark.css`.
- `assets/themes/catppuccin-mocha.css`.
- `assets/themes/catppuccin-latte.css`.
- `assets/themes/tokyo-night.css`.
- `assets/themes/dracula.css`.
- `assets/themes/nord.css`.
- `assets/themes/gruvbox-dark.css`.
- `assets/themes/rose-pine.css`.
- `assets/icons/` fallback icons.
- CSS variable contract documented.

## Dependencies

- Subplan 00 — Scaffold.
- Subplan 11 — UI Components (defines CSS variable contract).

## Acceptance Criteria

- [ ] All 8 bundled themes exist under `assets/themes/`.
- [ ] Each theme defines the full CSS variable contract.
- [ ] `nonchalant-dark` is the default and matches rsclip's design language.
- [ ] Fallback icons exist for common tray/notification use cases.
- [ ] Themes compile into binaries via `include_str!`.
- [ ] No syntax errors in CSS (validated by GTK load).

## Task Checklist

- [ ] Define the CSS variable contract document.
- [ ] Create `assets/themes/nonchalant-dark.css`.
- [ ] Create `assets/themes/catppuccin-mocha.css`.
- [ ] Create `assets/themes/catppuccin-latte.css`.
- [ ] Create `assets/themes/tokyo-night.css`.
- [ ] Create `assets/themes/dracula.css`.
- [ ] Create `assets/themes/nord.css`.
- [ ] Create `assets/themes/gruvbox-dark.css`.
- [ ] Create `assets/themes/rose-pine.css`.
- [ ] Add fallback icons to `assets/icons/`.
- [ ] Ensure themes are referenced by `sone-papdi-services` and `sone-papdi-ui`.
- [ ] Add a CSS lint step or GTK smoke load.

## Verification

```bash
cargo build -p sone-papdi-services
cargo build -p sone-papdi-ui
# Manual: run a small GTK program loading each theme
```

## Notes / Risks

- Theme files are text assets; keep them lean to avoid binary bloat.
- GTK may silently ignore invalid CSS; verify visually.
- The variable contract must be stable before downstream widgets rely on it.
- User themes override bundled ones; document the override path.
