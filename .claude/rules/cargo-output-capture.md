---
paths:
  - "crates/**"
  - "scripts/brehon/**"
  - ".github/workflows/**"
---

# Cargo output capture

Never pipe cargo output through `tail`, `head`, `grep`, `sed`, or any other
command when you need to know whether the command succeeded. Pipes mask the
exit code of the upstream command — `cargo build 2>&1 | tail -40` reports
the exit code of `tail`, not `cargo`. The build can fail and the surrounding
tooling will report success.

This rule applies to every long-running build/test/check command, not just
cargo. The same trap exists for `npm test | tail`, `pnpm build | grep`,
`docker build | tail`, etc.

## How to capture cargo output safely

If you need to see only the tail of long output, capture the full output to
a file and read the tail separately:

```bash
cargo test --test e2e --no-run -p lemmy_server > /tmp/cargo.log 2>&1
status=$?
tail -40 /tmp/cargo.log
echo "cargo exit code: $status"
[ $status -eq 0 ] || exit $status
```

Three things to notice:

1. The redirect (`> file 2>&1`) preserves the cargo exit code in `$?`.
2. The full log is on disk for diagnosis if the tail isn't enough.
3. The explicit `exit $status` at the end propagates the failure.

## Alternatives, ranked

1. **Capture to file** (above) — most reliable, keeps full log for diagnosis,
   works in any shell. **Default to this.**
2. **`set -o pipefail`** — propagates the leftmost non-zero exit status
   through pipes. Works in bash, but it's a per-shell setting and easy to
   forget. Acceptable for a single inline invocation if you set it
   immediately before:
   ```bash
   set -o pipefail
   cargo test 2>&1 | tail -40
   ```
3. **No pipe at all** — if the output is short enough to read in full, just
   run cargo without piping. Cargo's exit code is then the script's exit
   code automatically.

## What never to do

- ❌ `cargo build 2>&1 | tail -40` — exit code lost
- ❌ `cargo test --test e2e | grep PASS` — exit code lost AND output filtered
- ❌ `cargo check --workspace 2>&1 | head -100` — exit code lost
- ❌ Trusting a "task notification" or "exit code 0" summary from a tool
  that ran cargo through a pipe — that exit code came from the tail/head/grep
  on the right side of the pipe, not from cargo

## Why this matters here

The Brehon governance fork uses `cargo check --workspace` and `cargo test
--test e2e` as the validation signal for every PRP plan and every ralph
loop iteration. The stop hook on `/prp-ralph` decides whether to emit
`<promise>COMPLETE</promise>` based on those exit codes. If a piped cargo
invocation hides a real failure, the loop will declare success on a broken
build, write the wrong progress log entry, and poison the state file for
the next iteration. False-green is worse than red because the loop won't
self-correct.

This rule is mandatory for any cargo invocation (PRP commands, ralph loop, BM tasks) and is loaded automatically at session start.
