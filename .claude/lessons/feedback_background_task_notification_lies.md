---
name: Background task notification-summary lies about exit codes
description: The Bash tool's run_in_background notification's <summary> field can report exit code 0 when the underlying command actually exited non-zero. Treat the summary as advisory; trust log tails and explicit exit-code captures instead.
type: feedback
originSessionId: 4c75c587-189d-451e-87ef-e7639c56796e
---
When you launch a Bash tool call with `run_in_background: true`, the eventual `<task-notification>` contains a `<summary>` field reporting the command's exit code. **That field can be wrong.**

Discovered 2026-04-17 during Phase 5a plan-prep dry-run on Brehon-fork: a backgrounded `cmd //c cargo-clippy.bat ...` invocation returned to the notification layer with `<summary>exited 0</summary>` while the same clippy run, executed in the foreground with the same wrapper and same args, exited 101 (and the log tail showed the actual `error:` lines). Wrapper-script exit-code propagation was verified clean (`cargo.exe <sub> %*` on Windows correctly inherits the final-command exit). Pipes were not involved (output captured to file, not piped). The bug is in the notification layer itself, not the script and not a `cargo-output-capture.md` pipe-masking case.

**Why:** the run_in_background notification's exit-code field is whatever the harness records when the wrapper process tree terminates. On Windows, the outer `cmd //c` shell can exit 0 (its own success) even when the inner `cargo` exits non-zero, depending on how the wrapper batch handles `%ERRORLEVEL%` propagation in conjunction with how the harness reads the parent process exit. The wrapper IS propagating exit codes correctly to a foreground caller; the notification path is the one that lies.

**How to apply:**

- For DoD-critical and gate-critical commands (anything where exit code controls "did we ship?"), **run in foreground**. Pay the wait cost; get a trustworthy exit code.
- If you must run in background (long-running build, polling), **redirect stdout+stderr to a log file** AND **don't trust the notification summary's exit code**. After completion, grep the log tail for `error:` or `error[EXXXX]:` (cargo) / `Finished` (cargo success) / `FAILED` (test runners) to verify the real outcome.
- The pattern is sibling to `feedback_pipes_mask_exit_codes.md` (pipes masking exit codes) and `cargo-output-capture.md` (the established Brehon-fork rule about always capturing cargo output to file). This memory adds: even when you do everything right per those two memories — no pipes, output captured to file — the background-task notification can still report a wrong exit code. The log tail is the source of truth.
- Most commonly bites Windows + `cmd //c <wrapper>.bat` + cargo. Less likely on Unix shells with direct invocation, but the underlying notification-layer bug is platform-agnostic; assume any `run_in_background` summary's exit code may be unreliable.
- **Before writing any lesson based on a background task outcome (warmup, build, validation), verify the in-log marker first.** A lesson authored from a task-notification exit code rather than the in-log marker may encode a false premise. Origin: 2026-06-12 — the `services/bridge` warm-up notification reported exit 0; the in-log marker was `BRIDGE_WARMUP_EXIT_NONZERO`; a lesson was written the same day with the wrong premise ("bridge compiles on Linux, just not Windows"). Per `feedback_workspace_excluded_crate_must_have_lockfile.md` for the full incident.
