---
name: Linux-compile proof is a gate, not a remembered command
description: cargo-linux.sh is the free local replacement for Shape G's Linux deploy-target compile proof, but a tool that nothing invokes is a hope, not a guarantee. Embedded 2026-06-01 as a diff-scoped bm-pr gate via a validate-pending-laptop-linux DQ — fires only on dep/migration/cfg diffs (where Windows-green != Linux-green is plausible), skips pure-logic PRs. Mirrors the working validate-pending-laptop-e2e gate.
type: feedback
---

## TL;DR

`scripts/brehon/cargo-linux.sh` proves the **Linux deploy-target** build
(Docker `rust:1.95` mirror of CI) — the one thing laptop-native Windows cargo
can't, and the free replacement for Shape G's only irreplaceable job. But it
**ran on its own from nothing**: no PR-open trigger, no gate, no DQ. "Run it
before merge" was a discipline, i.e. a hope. A Linux-only break (`cfg(unix)`,
path-separator assumption, libpq linkage difference) could reach merge with
only Windows-compile evidence.

The fix (2026-06-01): make it a **diff-scoped gate** that mirrors the already-
working `validate-pending-laptop-e2e` pattern. A `validate-pending-laptop-linux`
DQ entry must be at `result: pass` before `bm-pr` opens the PR — but only when
the diff is in a scope where Windows-green ≠ Linux-green is actually plausible.

## Why a gate, not a command

A capability that depends on a human (or an agent) *remembering* to invoke it
has the reliability of memory, which is zero under load. The Windows e2e was
already gated (the `validate-pending-laptop-e2e` DQ blocks `bm-pr`); the Linux
compile had the identical risk profile and **no** gate. Asymmetry like that is
exactly where a regression slips through — the gated path gets run, the
ungated path gets skipped "just this once," and the once is the one that breaks
prod.

The bots (CodeRabbit, Copilot, adr-compliance) auto-fire on PR open, so the
*review* half is automatic. The *local-compile* half is not — so "local-first,
then bots" is enforced **only** by the local gate existing. Without it you get
bots-first, local-never: the opposite order.

## The Option-2 scope (the load-bearing decision)

Gating EVERY code PR on a Linux compile is over-broad: pure governance-logic
Rust compiles byte-identically on Windows and Linux, so the gate would add
~4 min of Docker ceremony to every phase close for zero coverage. The gate
fires ONLY when Windows-green ≠ Linux-green is plausible:

- diff touches `Cargo.toml` / `Cargo.lock` (dependency surface — new crate may
  be platform-conditional or have a Linux-only build script)
- diff touches `migrations/**` (the migration round-trip is a CI-Linux concern)
- diff adds `cfg(unix)` / `cfg(windows)` / `cfg(target_os)` /
  `std::path::MAIN_SEPARATOR` (explicit OS divergence)

Everything else skips the gate. Conservative-by-default: when unsure whether a
diff is in scope, raise the DQ + run `cargo-linux.sh` anyway — a green Linux
compile is never wrong, only sometimes redundant. (Validated 2026-06-01:
redaction-r1's diff — comments + a `scrub_json` recursion cap — correctly
scored out-of-scope; the gate did not burden it.)

## How it's wired (all four surfaces — a gate that doesn't load doesn't fire)

A lesson alone changes nothing; the gate is embedded at every surface that
must carry it:

1. **DQ kind** (`.claude/rules/decision-queue.md`): `validate-pending-laptop-linux`
   registered as a sibling of `-e2e`. `commands` = the cargo-linux.sh
   invocation; same mutation contract (`answered_by: "advisor-laptop"`,
   pass→resolved / fail→pending for §G4).
2. **bm-pr precondition** (`.claude/commands/bm/bm-pr.md` Phase-1d): detects the
   scope trigger from the diff, STOPs if in-scope and no passing DQ exists.
   `bm-task` (Haiku) only *checks* — it never runs cargo/Docker.
3. **Advisor stage-shape + handler** (`.claude/rules/advisor-orchestrator.md`
   §3.1 + §5.2): advisor raises the DQ when scoped, runs cargo-linux.sh via the
   laptop handler, mutates to pass before queueing bm-pr.
4. **impl-task brief template** (`.claude/PRPs/templates/impl-task-brief.template.md`
   §5): the impl-task writes the DQ when its own files match the trigger, then
   stops (NO CARGO ON ELITEDESK — the laptop runs it).

Plus the `cargo-linux.sh` header points back at the gate so the tool is
self-documenting.

## Who produces vs who checks (the e2e pattern, reused)

- **Produces the signal:** the lane session (Mode A) runs `cargo-linux.sh`; the
  advisor-laptop handler mutates the DQ to pass. The signal exists *before*
  bm-pr.
- **Checks the signal:** `bm-pr` (bm-task, Haiku) reads the DQ and confirms
  `result:pass`. It cannot produce the signal (no cargo/Docker on Haiku), so
  pushing production into bm-pr would create a stall — production belongs
  upstream, at impl-task / advisor-handler time. This is exactly how the e2e
  gate already works.

## See also

- `.claude/rules/decision-queue.md` "validate-pending-laptop-linux" — the kind
  definition + the Option-2 scope.
- `project_laptop_canonical_cargo_runner.md` (PMD) — the four-tool model;
  cargo-linux.sh is the Linux arm.
- `project_shape_g_suspended_2026_05_16.md` (PMD) — Shape G residual-only;
  cargo-linux.sh is why Shape G's Linux-compile job is now free + local.
- `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` — the
  meta-principle: a footgun lesson on a fixable gap must SHIP the fix, not just
  describe it. This lesson ships the gate.
- `feedback_validate_pending_laptop_write_then_stop.md` — the write-then-stop
  discipline the impl-task follows for all laptop-validation DQ kinds.
