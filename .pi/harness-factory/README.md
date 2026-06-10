# Brehon pi-harness-factory profiles

These profiles are a UX layer for `/factory browse`, `/factory preview`, and `/factory use`.
They are **not** the policy authority.

Hard enforcement stays in `.pi/extensions/lemmy-hooks.ts`:

- `/brehon-mode` is the canonical mode switch.
- `pi-harness-factory` active profiles with ids `brehon-*` are mapped back to `/brehon-mode` by `lemmy-hooks.ts`.
- Path policies, Rust plan-file checks, raw-cargo blocking, secret-path blocking, and CI auto-commit suppression are enforced by `lemmy-hooks.ts`.

Use `/factory use brehon-impl-task` for the profile UI; the next prompt/tool call will sync it to `/brehon-mode impl-task`.
