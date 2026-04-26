---
name: Task-notification exit summaries can lie for long cargo runs
description: The harness's `<task-notification>...<summary>... (exit code N)</summary>` for background cargo commands can misreport the exit code — verify via log tail every time
type: feedback
originSessionId: 6578d16f-975e-4abd-9206-3028fe863b7e
---
When a long cargo/clippy command is launched via `Bash(run_in_background: true)`, the harness delivers a `<task-notification>` with a summary like `Background command "..." completed (exit code 0)`. **This summary can misreport the exit code** — observed twice in a single v1-AD-c ralph session (2026-04-21):

- Task 1 clippy: summary said "exit code 0" but log tail showed `error: could not compile lemmy_api (lib) due to 1 previous error` from a `map_err_ignore` lint. Real exit was non-zero.
- Task 2 cargo-check: summary said "exit code 0" but log tail showed `error: could not compile lemmy_db_views_reputation` with a red build.

**Why:** The summary appears to be derived from the batch-wrapper's exit code, not cargo's actual exit. This is a distinct failure mode from the older pipe-masking issue in `feedback_cargo_invocations.md`/`cargo-output-capture.md` — the cargo command is already captured to file correctly; it's the *notification line* that lies.

**How to apply:**

1. Never trust the `(exit code N)` in a task-notification summary for cargo/clippy/test commands. Always verify by reading the log tail:
   ```bash
   tail -3 .claude/<log>.log; grep -c "^error: " .claude/<log>.log
   ```
2. Real green = `Finished ... profile` on the last line AND zero `^error:` matches.
3. Real red = `could not compile` or `^error:` matches — regardless of what the notification summary said.

This rule compounds with `cargo-output-capture.md`: that rule makes the log trustworthy; this rule is about not skipping the log read because the notification appeared green.
