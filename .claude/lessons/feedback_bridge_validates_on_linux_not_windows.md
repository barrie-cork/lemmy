---
name: services/bridge validates on Linux (Docker), never on the Windows host
description: services/bridge does not compile on the Windows laptop — ruma-common v0.19.0 hits E0119 (conflicting From<...HourBase> impl vs the time crate), a host-toolchain quirk independent of any bridge code change. The bridge's real deploy target is Linux, so ALL bridge cargo (check/clippy/test) runs via scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml (Docker rust:1.95), not the Windows-local `cd services/bridge && cargo` form. Also records the ground truth that services/bridge/Cargo.lock is NOT tracked/present in the repo.
type: feedback
---

> **⚠️ PREMISE CORRECTION (2026-06-12, same day as authoring).** The original
> framing below — "bridge compiles on Linux, just not Windows; route bridge cargo
> to Docker" — is **WRONG**. A `cargo-linux.sh` warm-up (Docker `rust:1.95`, the
> CI mirror) hit the **SAME `ruma-common v0.19.0` E0119 vs `time`** (25 errors;
> log `.claude/PRPs/debug/m2-late-2-bridge-warmup.log` → `BRIDGE_WARMUP_EXIT_NONZERO`).
> So `services/bridge` does **not** compile on **either** target — this is a
> **dependency-resolution / version-pin bug** in the bridge's own untracked,
> unpinned dep tree (likely a transitive `time` 0.3.x bump incompatible with
> `ruma-common 0.19.0`), NOT a Windows-host quirk. The `--manifest-path` Docker
> invocation form below is still correct *mechanically*, but it does NOT make the
> bridge green — the ruma/time conflict must be fixed first (pin `time`/`ruma` in
> `services/bridge/Cargo.toml`, which will create the absent `Cargo.lock`).
> Investigate before relying on any bridge validation. This lesson is being kept
> (not deleted) as the record of the wrong premise + the correction. Per
> `feedback_background_task_notification_lies` — the warm-up's "exit 0"
> notification was the shell wrapper's status, not cargo's; the in-log marker was
> authoritative.

## TL;DR

`services/bridge` (the Brehon Matrix bridge, workspace-excluded per R9) **does
not compile on the Windows laptop host** (and, per the correction above, **not on
Linux either** — the root cause is a ruma/time version conflict, not the host).
`cargo check` from `services/bridge/` exits 101 with `error[E0119]`: conflicting
implementations of `From<...format_item::HourBase>` in `ruma-common v0.19.0` vs
the `time` crate.
This is a **host-toolchain / registry-state quirk**, reproducible on a clean
phase base before any bridge edit — NOT a code defect, NOT introduced by any
sub-phase. The bridge's real deploy target is Linux, so the correct validation
target is Linux, not Windows.

**Rule:** all bridge cargo — `check`, `clippy`, `test`, in DoD smoke tests, in
plan §15 commands, in `validate-pending-laptop` DQ `commands`, in Task-0 probes,
in §16a checkpoint commands — runs via Docker:

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml
```

The literal `cd services/bridge && cargo <verb>` form is **reference-only** — do
not execute it on Windows; it fails before reaching any bridge code.

## Why `--manifest-path` (not `cd services/bridge`)

`cargo-linux.sh` mounts the repo root at `/work` and runs `cargo $*` from there
(`-w /work`). The bridge is **workspace-excluded** (`Cargo.toml:82
exclude = ["services/bridge"]`), so plain `cargo check` from the repo root won't
see it — `--manifest-path services/bridge/Cargo.toml` is how cargo reaches the
excluded crate from the repo-root working dir. The wrapper is scope-agnostic by
design (`feedback_wrapper_script_flag_silence.md`), so `--manifest-path` is a
plain passthrough — **no wrapper-script change is needed**.

Preconditions:
- Docker Desktop in **Linux-container mode** (`docker info --format '{{.OSType}}'`
  → `linux`; the wrapper preflights this and exits 3 if not).
- **First bridge container run is COLD** (~10–20 min — full `ruma` / `matrix-sdk`
  download + compile in-container). The `brehon-cargo-registry` named volume warms
  it after. Consider a manual warm-up run before an `/auto-phase` session so the
  first bridge validation inside the orchestration loop isn't cold.

Workspace crates (`crates/**`) still validate on the Windows bat wrappers
(`scripts\brehon\cargo-check.bat --workspace --features full`) — they compile
fine on Windows; only the bridge has the ruma/time conflict.

## Ground truth: `services/bridge/Cargo.lock` is NOT tracked

Multiple handovers/briefs claimed "`services/bridge/Cargo.lock` is tracked (bridge
has its own lockfile)". **This is wrong.** Verified 2026-06-12:
`git ls-tree governance-v0 services/bridge/Cargo.lock` and
`git ls-tree origin/phase-m2-late-2 services/bridge/Cargo.lock` both return empty;
`ls services/bridge/Cargo.lock` → No such file. The lockfile is **generated on
first `cargo` build, then cleaned** — it is neither committed nor gitignored,
simply absent. (Origin of the drift: `m1-a-impl-8.md:207` said "scaffolding *may
generate* `services/bridge/Cargo.lock` — commit it if created"; it evidently was
NOT committed, and downstream artifacts assumed it exists.)

**Consequence for the Linux gate:** the `validate-pending-laptop-linux` gate
(`feedback_linux_compile_proof_is_a_gate.md`) was framed as firing on a bridge
`Cargo.lock` *change*. Since the lockfile isn't tracked, that specific trigger
can't fire on a diff. But the Linux path is the bridge's *normal* validation
route here regardless of dep changes — because the bridge **cannot compile on
Windows at all.** So: run bridge cargo on Linux every time, not only when deps
change.

**Verify-before-trusting note:** `git ls-files <path>` printing nothing while
exiting 0 means **not tracked** — do not read empty-output-plus-exit-0 as
"tracked" (per `pattern_verify_before_trusting_shell_output`). Use
`git ls-tree <ref> <path>` for an unambiguous answer.

## How to apply

- **Planner:** any plan whose §15 has bridge cargo MUST write the
  `cargo-linux.sh --manifest-path services/bridge/Cargo.toml` form, not
  `cd services/bridge && cargo`, and tag bridge validation as Linux-only. Do not
  write `EXPECT exit 0 locally` for a bridge cargo command. (See `planning.md`
  bridge watchpoint.)
- **Advisor (§3.4 DoD smoke test):** when a plan's §15 includes `services/bridge`
  cargo, run it via `cargo-linux.sh --manifest-path …`, never the Windows-local
  form. A bridge cargo command in §15 that uses the Windows form is a DoD-issue —
  flag it for the planner to revise.
- **Advisor (validate-pending-laptop handler):** a bridge `validate-pending-laptop`
  entry's `commands` run via the `cargo-linux.sh` form; mutate with
  `answered_by: "advisor-laptop"` as usual.

## Symptom to recognise

`error[E0119]: conflicting implementations of trait
\`From<...HourBase>\` ... conflicting implementation in crate \`time\``
during a bridge `cargo check`/`clippy`/`test` on Windows. The fix is never to
patch ruma or `time` — it is to run the bridge build on Linux (Docker) where the
conflict does not occur.

## See also

- `feedback_linux_compile_proof_is_a_gate.md` — `cargo-linux.sh` is a gate, not a
  remembered command; the diff-scoped `validate-pending-laptop-linux` DQ.
- `feedback_wrapper_script_flag_silence.md` — why `cargo-linux.sh` takes the
  subcommand + flags verbatim (so `--manifest-path` is a clean passthrough).
- `pattern_verify_before_trusting_shell_output` — `git ls-files` empty + exit 0 ≠
  tracked.
- `.claude/PRPs/reports/session-retro-2026-06-12-m2-late-2-plan-approval-handoff.md`
  — the DoD-smoke catch that produced this lesson.
