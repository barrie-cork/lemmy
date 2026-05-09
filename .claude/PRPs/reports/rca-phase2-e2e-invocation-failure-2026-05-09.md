# RCA: Phase 2 e2e Invocation Failure — 4 Attempts Before Correct Run

**Date:** 2026-05-09  
**Sub-phase:** v1-SL-c-2  
**Attempts wasted:** r1 (bv8jlnd1k), r2 (by21ycs98), r3 (by4tx21kj) — all failed for different reasons  
**Correct run:** r4 (bln2q7xfw) via `cargo-test.bat --workspace --features full`  
**Wall-clock cost:** ~35 min (r1: ~6 min compile + instant fail; r2: ~2 min + instant fail; r3: ~3 min; diagnosis: ~20 min)

---

## What happened (timeline)

| Run | Command | Failure | Root cause |
|-----|---------|---------|------------|
| r1 | `bash: cargo test --workspace --features full --test e2e -- --test-threads=1` | `STATUS_DLL_NOT_FOUND` (0xc0000135) | libpq.dll not on Windows DLL search path — bash PATH doesn't affect Windows DLL loader |
| r2 | Same but with `export PATH="vcpkg/bin:$PATH"` first | Same `STATUS_DLL_NOT_FOUND` | Same root cause — bash PATH export has no effect on the Windows PE DLL loader |
| r3 | `cargo-test.bat -p lemmy_server --features full --test e2e` | `error: the package 'lemmy_server' does not contain this feature: full` | `lemmy_server` doesn't declare `full`; feature lives on workspace members. Wrong scope flag. |
| r4 | `cargo-test.bat --workspace --features full --test e2e` | VCVARS_OK, running | Correct: bat wrapper sets DLL path + `--workspace` propagates feature |

---

## 5 Whys

### Symptom: r1 failed with STATUS_DLL_NOT_FOUND

**Why 1: Why did the test binary fail to load?**  
`libpq.dll` was not on the Windows DLL search path when `e2e-d223f5bfcd0ad348.exe` was launched by cargo's test harness. Windows resolves DLLs at EXE load time using the Windows search order (app dir, System32, PATH from the *Windows environment*, not the current bash session's PATH).

**Why 2: Why was libpq.dll not on the Windows DLL search path?**  
The e2e was launched via `cargo test ...` invoked directly in a bash (`Bash` tool) session. The bash session's `$PATH` is a POSIX-style path list maintained by MSYS2/Git Bash — it does NOT set the Windows `%PATH%` environment variable that the Windows DLL loader reads. The `cargo-test.bat` wrapper exists precisely to prepend `vcpkg\installed\x64-windows\bin` (where `libpq.dll` lives) to the *Windows* `%PATH%` via `set PATH=...` before invoking cargo.

**Why 3: Why was `cargo-test.bat` not used on the first attempt?**  
The advisor followed the command as specified in `advisor-orchestrator.md` §3.2 gate 4:

> `cargo test --workspace --test e2e --features full -- --test-threads=1` in `run_in_background`

The rule names a bare `cargo test` command. It does **not** say "use `cargo-test.bat`". The rule was written assuming the reader would know to substitute the platform-appropriate wrapper — but that substitution logic is documented only in `pre-phase-harness-audit.md` (which says "scripts under `scripts/brehon/` are the *only* supported way to run cargo on this project on Windows") and is not cross-referenced at the Phase 2 e2e gate site.

**Why 4: Why does the rule at the Phase 2 e2e gate not mention the wrapper?**  
The Phase 2 e2e gate text was authored in the context of the `validate-pending-laptop handler` (§5.2), where the `commands[]` field was designed to store "§15 DoD lines verbatim". The §15 DoD lines in plans typically use the wrapper — but the Phase 2 e2e entry is advisor-raised (not impl-raised), so there is no `commands[]` field and no plan §15 DoD line to inherit the wrapper from. The gate text was a copy of the user-gate description (which describes the *conceptual* command) without the execution wrapper that the harness-audit rule mandates.

**Why 5: Why didn't the advisor check `pre-phase-harness-audit.md` before executing?**  
`pre-phase-harness-audit.md` is framed as a "before the first task of any new phase" rule, not as a "before every cargo invocation" rule. The Phase 2 e2e happens mid-phase, after Task 1 completes. The rule's scope ("before task 1") caused the advisor to not re-consult it at the Phase 2 launch point. There is no cross-reference from the Phase 2 e2e gate text to the Windows wrapper requirement.

---

## Root cause (single sentence)

The Phase 2 e2e gate text in `advisor-orchestrator.md` §3.2 specifies a bare `cargo test` command without noting that on Windows, the `cargo-test.bat` wrapper is mandatory for `libpq.dll` to be on PATH, and no cross-reference to `pre-phase-harness-audit.md` exists at that site.

---

## Contributing factors

1. **r2 wasted**: The advisor believed `export PATH=...` in bash would fix the issue — a reasonable assumption for a Linux-trained model but incorrect on Windows where bash PATH ≠ Windows DLL search path. No rule explicitly documents this asymmetry.

2. **r3 wasted**: After switching to the bat wrapper, the advisor used `-p lemmy_server --features full` (from the plan §15 DoD language "cargo test -p lemmy_server --test e2e --features full") but `lemmy_server` doesn't declare `full`. The workspace-level flag `-p lemmy_server` scopes cargo to one crate, but `--features full` only applies to crates that declare it. `--workspace` is required. The two-source conflict (gate text says `--workspace`; plan §15 says `-p lemmy_server`) was not flagged.

3. **cmd //c backgrounding issue**: An earlier attempt (before r1) used `cmd //c "..." run_in_background` which immediately returned exit 0 because cmd.exe detached. This is documented in the session summary but was a third unknown contributing to the confusion.

---

## Solution: two-part fix

### Fix A — Amend `advisor-orchestrator.md` §3.2 gate 4 text (immediate)

Change the Phase 2 e2e option (a) local command from bare `cargo test` to the wrapper form:

**Before:**
```
(a) local — `cargo test --workspace --test e2e --features full -- --test-threads=1` in `run_in_background`
```

**After:**
```
(a) local — `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` (Windows; sets vcpkg libpq.dll PATH via VCVARS). Add `> <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>` for capture. ~26 min, zero billed.
```

Also add a note: "Do NOT use bare `cargo test` on Windows — libpq.dll is only on the Windows DLL search path when the bat wrapper's `set PATH=vcpkg\bin;%PATH%` runs."

### Fix B — Add a new lesson file

Create `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` documenting:
- On Windows, bash PATH export does NOT affect Windows PE DLL search
- `cargo-test.bat` is the mandatory entry point for any test run that links libpq (i.e. any test of `lemmy_server`)
- `--workspace` required when `--features full` targets workspace members (not `lemmy_server` directly)
- Canonical command: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- --test-threads=1 > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"`

### Fix C — Add cross-reference in §5.2 validate-pending-laptop handler

At the Phase 2 e2e paragraph in §5.2, add:
> **Windows note:** use `cmd //c "scripts\\brehon\\cargo-test.bat ..."` not bare `cargo test`. The bat wrapper sets vcpkg libpq.dll on the Windows DLL search path (bash PATH export does not). See `feedback_windows_e2e_requires_bat_wrapper.md`.

---

## What this does NOT change

- The Phase 2 e2e gate itself (user still picks local vs dispatch) — gate 4 is correct
- The DQ #168 entry — it's already raised correctly; just needs mutation after r4 passes
- The `cargo-test.bat` wrapper — it's correct; the problem was not using it

---

## Lesson candidates (for PMD + retro)

1. **Windows DLL search path ≠ bash PATH**: On Windows, a bash `export PATH=...` does not affect the Windows DLL loader. Test EXEs link against `.dll` files found via Windows `%PATH%` (set by `set PATH=...` in cmd.exe or .bat), not POSIX PATH.
2. **`lemmy_server` has no `full` feature**: The `full` feature lives on `lemmy_db_schema`, `lemmy_utils`, and other workspace members. `-p lemmy_server --features full` always errors. Use `--workspace --features full` for any e2e run.
3. **Phase 2 e2e wrapper requirement gap**: The gate text at §3.2 gate 4 does not mention the bat wrapper. The harness-audit rule mentions it but is scoped to "before task 1". Cross-reference gap = advisor re-derives the command from incomplete information.
