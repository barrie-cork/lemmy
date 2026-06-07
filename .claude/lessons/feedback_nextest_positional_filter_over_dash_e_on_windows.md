---
name: nextest-positional-filter-over-dash-e-on-windows
description: On Windows, an interactively-typed nextest filter must be a bare positional substring, not -E "test(...)" — the filterset double-quotes mangle through the bash → cmd //c → batch %* → nextest chain.
type: feedback
---

# nextest positional filter over `-E` on Windows

> **Provenance (2026-06-01):** authored from the advisor session's report of a
> Windows `cmd //c` run during ADR-017 BUG-1 e2e verification (PR #176). The
> mangling was observed on `win32` via the bat-wrapper chain; it was **not**
> reproduced on Linux (where the `cmd //c` layer doesn't exist). Scope the claim
> to the Windows interactive-invocation path accordingly.

## Symptom
`scripts\brehon\cargo-nextest.bat run --workspace --features full --test e2e -E "test(/pat/)"`
selects 0 tests (or errors on an unparseable filterset) even though the pattern is correct.
The harness exit-summary may also disagree with the `E2E_EXIT_0` sentinel — a tell that
arguments were mangled before nextest saw them.

## Cause
The filterset string in `-E "test(...)"` passes through four layers —
bash → `cmd //c` → batch `%*` → nextest — and the double-quotes get re-quoted/stripped
along the way, so nextest receives a broken filterset, not `test(/pat/)`.

## Fix
Use **bare positional substring filters** instead of `-E`. nextest treats trailing
positional args as test-name substrings (OR'd):

    cmd //c "scripts\brehon\cargo-nextest.bat run --workspace --features full --test e2e author_defendant author_appeal"

Two substrings `author_defendant author_appeal` cover all three ADR-017 tests
(round-trip + emergency contain `author_defendant`; the appeal test contains `author_appeal`).

The plain runner is the same: `... cargo-test.bat ... -- author_defendant author_appeal`.

## Scope — this is the interactive `cmd //c` path only, NOT the `e2e_filter` field
This lesson applies to an **advisor typing a `cmd //c` nextest command directly**.
It does **not** contradict `feedback_e2e_nextest_filter_groups.md`'s `-E "test(~name)"`
table: that field (`e2e_filter` in a `validate-pending-laptop` DQ entry) is consumed by
the advisor's validate-pending handler, which builds the nextest command itself and can
quote `-E` correctly in that controlled construction. The mangling bites only when the
four-layer bash → `cmd //c` → batch `%*` → nextest chain is in play. Rule of thumb:
- DQ `e2e_filter` field → keep using the `-E "test(...)"` group expressions.
- Hand-typed `cmd //c` one-off → bare positional substrings.

## Cost when ignored
First ADR-017 e2e run selected 0 tests; caught by the `E2E_EXIT_0` sentinel in ~45s
(before any compile), so cheap — but only because the sentinel was watched. Without it,
"0 selected" reads as a pass.

## See also
- `feedback_windows_e2e_requires_bat_wrapper.md` — the bare-`cargo test` libpq.dll trap;
  same wrapper-chain family.
- `feedback_e2e_nextest_filter_groups.md` — the canonical `-E` group table for the
  `e2e_filter` DQ field (the controlled-construction path the Scope note above carves out).
