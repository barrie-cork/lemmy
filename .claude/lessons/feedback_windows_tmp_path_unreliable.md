---
name: Windows /tmp/ redirect silently loses output
description: cmd //c with > /tmp/file on Windows git-bash may silently drop output; always redirect under .claude/ or $TMPDIR
type: feedback
originSessionId: f54f6612-08ca-4d43-adf5-047a3721012a
---
Never redirect cargo output to `/tmp/...` on Windows from a `cmd //c "..."` invocation. Git-bash's `/tmp` maps to a Cygwin/MSYS-internal path that doesn't exist from cmd.exe's perspective — the `>` redirect silently fails AND cargo still reports its own exit code, so you get "exit: 0" with no log file.

**Why:** The call chain `bash → cmd //c "... > /tmp/file 2>&1"` evaluates the redirect inside cmd.exe, not bash. cmd.exe has no `/tmp` concept. On some Windows configurations it falls back to the current directory with a literal `tmp` folder; on others (notably the Brehon fork dev setup) the redirect silently goes nowhere and cargo's output is lost. Meanwhile cargo itself runs fine and exits with its real status, so the shell's "exit: 0" looks legitimate.

**How to apply:**

- Always redirect to paths under `.claude/` (project-local) or `$TEMP` (Windows-native):
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-taskN.log 2>&1"
  ```
- If you must use a tmp path, use `"$TEMP"` (Windows-native TEMP env var resolved by cmd.exe):
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat ... > %TEMP%\\build-taskN.log 2>&1"
  ```
- Pair with `feedback_task_notification_exit_summary_unreliable.md`: always verify a tail of the log after a background task. If the log file doesn't exist, the redirect failed and the "exit 0" is unverifiable.

Discovered 2026-04-21 during v1-AD-c task 4 disposition: advisor kicked off a background `cargo-test.bat --workspace --features full --lib payload_parity` with `> /tmp/test-workspace-lib.log 2>&1`, got notification "exit code 0", but `ls /tmp/test-workspace-lib.log` returned "No such file or directory". Only the shell wrapper's own stdout ("exit: 0") survived, written to Claude Code's task-output file. Cargo's actual output was lost to the void.

**Also applies to:** any build/test wrapper invoked via `cmd //c`, not just cargo. `docker build`, `npm test`, `pnpm build` all have the same exposure.

Related memories:
- `feedback_task_notification_exit_summary_unreliable.md` — always verify log tails, don't trust notifications
- `feedback_cargo_output_capture.md` via `.claude/rules/cargo-output-capture.md` — pipes mask exit codes (different failure mode, same defense: redirect to file under .claude/)
- `feedback_cmd_c_redirect_exit_code_capture.md` — cmd //c exit code capture trap
