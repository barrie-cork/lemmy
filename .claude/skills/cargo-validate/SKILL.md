---
name: cargo-validate
description: Run cargo via Brehon Windows wrapper, capture to log, return exit + tail-20.
user-invocable: true
---

# `/cargo-validate` — capture-then-tail cargo runner

Run a cargo command through the Brehon Windows wrapper, capture full output to a log file, return only exit code + last 20 lines. Used in place of inline `cargo ... 2>&1 | tail -40`, which masks exit codes (per `.claude/rules/cargo-output-capture.md`) and floods context with full logs (per `.claude/rules/no-cargo-output-paste.md`).

## When to invoke

- After any per-task code edit, before committing
- At plan DoD checkpoints (§15.x in JM-b plan)
- When verifying a fix landed before declaring task done

Skip when: the cargo run is itself a long-running background job (`cargo test --test e2e` full sweep) — use the `cargo-runner` background subagent instead.

## Inputs

The user (or invoking command) provides one of:

- A bare cargo verb: `check`, `clippy`, `test --no-run`, `test --test e2e --no-run`
- A full cargo command line: `clippy --workspace --features full --no-deps -- -D warnings`
- A scope: `-p lemmy_api`, `-p lemmy_server`, `--workspace`

Default scope when unspecified: `--workspace --features full`. Default verb: `check`.

## Procedure

1. **Resolve the wrapper.** Brehon mandates wrapper scripts on Windows (libpq + vcvars). Map verb → wrapper:
   - `check` → `scripts\\brehon\\cargo-check.bat`
   - `clippy` → `scripts\\brehon\\cargo-clippy.bat`
   - `test` → `scripts\\brehon\\cargo-test.bat`
   - Anything else: STOP and ask the user (no `cargo` invocation outside the wrappers).

2. **Resolve the log path.** Default: `.claude/build-<verb>-<short-context>.log` (e.g., `.claude/build-clippy-task5.log`). If the user/plan named a path, use it. Use `.claude/PRPs/debug/v1-JM-<sub>-<verb>-<task>.log` if invoked from a phase-branch worktree to keep build artifacts co-located with phase debug logs.

3. **Run with capture.** The single canonical pattern:

   ```bash
   cmd //c "scripts\\brehon\\cargo-<verb>.bat <args> > <log_path> 2>&1"
   status=$?
   tail -20 <log_path>
   echo "exit: $status"
   ```

   The `> <log_path> 2>&1` redirect preserves cargo's exit code in `$?`. Never pipe through `tail`/`head`/`grep` — that masks the exit code (per cargo-output-capture.md). Never run cargo bare (no wrapper); the wrapper is load-bearing on Windows.

4. **Verify exit-code propagation.** If the wrapper itself is suspect (recent change, or first invocation in a session), run a known-bad sanity probe:
   ```bash
   cmd //c "scripts\\brehon\\cargo-<verb>.bat --features nonexistent_xyz > /tmp/sanity.log 2>&1"
   echo "sanity exit: $?"
   ```
   Expect non-zero (typically 101). If 0, the wrapper is masking exit codes — STOP, surface to user, do NOT proceed (per `feedback_batch_goto_eof_clobbers_errorlevel.md`). This probe matches `pre-phase-harness-audit.md §1 probe 4`.

5. **Report.** Return ONLY:
   - The verb and full command run
   - The log path
   - The exit code
   - The last 20 lines of the log

   Do NOT paste larger excerpts unless the user explicitly asks. If exit ≠ 0 and the failure context is spread across more than 20 lines, name the log path and offer to read a specific `offset/limit` range — don't dump the file.

## Reporting format (user-facing)

```
verb: clippy
cmd:  scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings
log:  .claude/build-clippy-task5.log
exit: 0

<last 20 lines of log>
```

When exit ≠ 0:
```
verb: clippy   ✗ FAILED (exit 101)
cmd:  ...
log:  .claude/build-clippy-task5.log
hint: full output at log; use Read with offset to view earlier sections

<last 20 lines>
```

## What this skill never does

- Pipe cargo output through `tail`/`head`/`grep`/`sed`. Always capture-to-file first, then `tail` the file.
- Run cargo bare (no wrapper) on Windows. The wrappers handle libpq + vcvars.
- Paste >20 lines of cargo output into the conversation. The log is on disk for diagnosis.
- Edit code or commit. This skill is read-only validation; fixes are the caller's job.
- Modify the wrapper scripts. Wrapper bugs go to a `chore(scripts):` commit, not patched inline.
- Skip the negative-probe (step 4) on a session's first cargo run if the wrapper hasn't been validated this session.

## Related rules (auto-loaded; do not re-Read)

- `.claude/rules/cargo-output-capture.md` — capture-to-file pattern, why pipes mask exit codes
- `.claude/rules/no-cargo-output-paste.md` — token-budget rule for log tails
- `.claude/rules/pre-phase-harness-audit.md` — probe 4 negative-test for exit-code propagation

## Memory references

- `feedback_batch_goto_eof_clobbers_errorlevel.md` — Issue #8 root cause; why the negative probe matters
- `feedback_pq_sys_wrapper_env_propagation.md` — wrapper env-var hygiene
- `feedback_clippy_vs_check_wrapper.md` — cargo-check.bat ≠ cargo-clippy.bat
- `feedback_windows_tmp_path_unreliable.md` — prefer `.claude/` over `/tmp/` for log paths
- `feedback_task_notification_exit_summary_unreliable.md` — never trust background-completion summaries; always verify the captured log tail
