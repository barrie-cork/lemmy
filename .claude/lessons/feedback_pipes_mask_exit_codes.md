---
name: Pipes mask exit codes
description: Never pipe cargo/npm/docker/long-running build commands through tail/head/grep when you need to know if they succeeded — the pipe reports the right-hand command's exit code, not the build's
type: feedback
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
Never pipe cargo, npm, pnpm, docker, or any long-running build/test/check command through `tail`, `head`, `grep`, `sed`, etc. when you need to know whether the command succeeded. The pipe reports the exit code of the *right-hand* command (tail/head/grep), not the build. The build can fail and the surrounding tooling will report success.

**Why:** Lost an iteration during Brehon Phase 0 (2026-04-14) because an agent ran `cargo test --test e2e --no-run -p lemmy_server 2>&1 | tail -40` and the tool reported "exit code 0" — but cargo had actually failed on a libpq link error. The pipe through `tail` masked it. The agent only caught it because two error lines were visible in the captured tail. False-green is worse than red because automated loops (ralph, Junior) won't self-correct.

**How to apply:**
- Default to capture-to-file: `cargo build > /tmp/cargo.log 2>&1; status=$?; tail -40 /tmp/cargo.log; [ $status -eq 0 ] || exit $status`
- This preserves the build's exit code in `$?`, keeps the full log for diagnosis, and propagates failure.
- Acceptable alternative for inline: `set -o pipefail` immediately before the pipe — propagates leftmost non-zero exit. But it's per-shell and easy to forget.
- Best alternative when output is short: don't pipe at all. The build's exit code becomes the script's exit code automatically.
- Never trust a "exit code 0" summary from a tool that ran the build through a pipe — that exit code came from `tail`/`head`/`grep`, not from the build.

Applies to any project, any language, any long-running build. Especially load-bearing in autonomous loops (PRP, ralph, Junior) where false-green poisons the state file and the loop declares success on a broken build.
