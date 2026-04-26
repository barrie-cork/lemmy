---
name: Clippy uses cargo-clippy.bat, not cargo-check.bat
description: Clippy validation must use scripts/brehon/cargo-clippy.bat — cargo-check.bat runs cargo check, not cargo clippy. Same bug hit Phase 3.
type: feedback
originSessionId: e8d9be45-74d4-44e0-aba4-2baa51128e3f
---
When writing plan validation commands, use the correct wrapper script:
- `cargo check` → `scripts/brehon/cargo-check.bat`
- `cargo clippy` → `scripts/brehon/cargo-clippy.bat`
- `cargo test` → `scripts/brehon/cargo-test.bat`

**Why:** Phase 3 plan had clippy commands using cargo-check.bat, which silently ran `cargo check` instead of `cargo clippy`. The advisor caught the same bug in Phase 4a plan (correction C4). The wrappers have different names for a reason — they invoke different cargo subcommands.

**How to apply:** Every time a plan or ralph loop specifies a clippy validation step, grep the command for `cargo-check` and replace with `cargo-clippy` if the intent is linting. This is especially important in Task 10 / final-validation tasks.
