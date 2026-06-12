# Session retro — 2026-06-12 — bridge time pin fix

**Harness:** claude-code
**Session window:** ~2026-06-12T12:00 → 2026-06-12T13:20 (~80 min)
**Branch at start:** `6c8b84770` (`governance-v0`)
**Branch at end:** `cdc97fda5` (`governance-v0`)
**Files touched:** 4 (Cargo.toml, Cargo.lock, feedback_bridge_validates_on_linux_not_windows.md, CLAUDE.md)
**Commits:** 2 (1 prior-session lesson correction + 1 this session fix)

## TL;DR

Session goal: the prior session's warm-up of `services/bridge` via `cargo-linux.sh` had
reported "exit 0" (background task notification), but the in-log marker was
`BRIDGE_WARMUP_EXIT_NONZERO` with 25 `E0119` errors. This session diagnosed
the root cause (ruma-common 0.19.0's `StringEnum` derive conflicting with time 0.3.48+,
made worse by the complete absence of a `Cargo.lock`), corrected the wrong-premise lesson
that said "bridge compiles on Linux, just not Windows", pinned `time = "=0.3.47"`, and
verified the fix via a fresh Docker check → `BRIDGE_FIX_EXIT_0`, 0 errors. The `Cargo.lock`
was generated and committed for the first time. Top change proposal: the **absence of
`Cargo.lock` in a services/ crate should trigger a pre-warmup warning** — a lockfile-less
workspace-excluded crate has unbounded dep resolution and is a latent breakage source
every time a transitive dep ships a minor bump.

---

## What surprised us

**Advisor:**
- The prior session's lesson was authored and committed **on the same day** as the warmup
  failure, yet contained a wrong premise ("compiles on Linux, just not Windows"). The
  lesson had to be corrected within ~1 hour of being written. Root cause: the warmup log
  was parsed from a background task notification ("exit 0") rather than the in-log marker —
  `feedback_background_task_notification_lies` fired again, this time poisoning a
  lesson's founding assumption, not just a task outcome.
- The bridge `Cargo.lock` had never been committed, despite the bridge being an active,
  workspace-excluded binary crate. This is structurally dangerous: every cold Docker check
  re-solves the dep tree against the current crates.io state, which can pick up breaking
  transitive bumps silently. There was no alert, no DQ entry, no lesson — the absent lock
  was invisible until it caused a build failure.
- `ruma-common 0.19.0` has no patch release (`0.19.1`). The E0119 conflict with
  `time 0.3.48+` was introduced by a time minor bump (June 2026), and ruma upstream has
  not shipped a fix. This means the pin is a durable workaround, not a temporary one —
  it will need revisiting when `matrix-sdk` upgrades its ruma dependency.

**Impl:**
- The fix was a single `time = "=0.3.47"` line. The 80-minute session cost was almost
  entirely diagnosis (reading the log, confirming the version, researching upstream) +
  Docker cargo build time, not the change itself.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add a `Cargo.lock` presence check to the bridge DoD smoke-test and to the m2-late-2 pre-queue precheck.** Specifically: before any `cargo-linux.sh` check on `services/bridge`, assert that `services/bridge/Cargo.lock` exists; if absent, fail loudly with "MISSING_BRIDGE_LOCKFILE — commit one before proceeding." The check lives in `.claude/PRPs/plans/m2-late-2.plan.md` §15 (now that we have a lock) and as a note in the bootstrap handover §5 operational rules. | Prevents the next warmup from silently re-resolving the dep tree to a newer breaking version | minor | 1× this session; 0× prior — but consequential enough to act on |
| 2 | **Update the `feedback_bridge_validates_on_linux_not_windows.md` lesson** to document the `time = "=0.3.47"` pin as the durable workaround, and add a "revisit when" condition: "When `matrix-sdk` upgrades its ruma dependency to a version that resolves the E0119 conflict, remove the pin and re-run the Docker check." Also add: "When `Cargo.lock` is absent, the Docker check is NOT a stable baseline — re-running it on different days may give different results." | Future sessions don't repeat the lesson-poisoning incident | minor | Already executed partially (lesson corrected this session); full update needed |
| 3 | **Add a lesson `feedback_workspace_excluded_crate_must_have_lockfile.md`** capturing: any `exclude`-d crate in a Cargo workspace (R9 pattern) should have its `Cargo.lock` committed, period. Without it, every cold check is an unbounded dep-resolution that may break silently on transitive bumps. The rule: "After authoring a new workspace-excluded binary crate, the first cargo check (wherever it runs) MUST produce and commit a `Cargo.lock`; treat a missing lock as a DoD miss." | Prevents the same class of invisible regression for any future workspace-excluded crates | minor | 1× brehon-fork; potentially cross-project — promote to lesson |

## What to carry forward

- **In-log marker is authoritative; background task exit code is not.** This session
  correctly applied `feedback_background_task_notification_lies` — read
  `BRIDGE_FIX_EXIT_0` / `BRIDGE_WARMUP_EXIT_NONZERO` from the log, not the task
  notification. But the prior session had failed to apply this discipline during the warmup
  itself, leading to the wrong-premise lesson. The rule needs to be enforced at the
  **warmup stage** not just the fix stage: before writing any lesson based on a background
  task result, verify the in-log marker.
- **One-line in-log markers (`BRIDGE_WARMUP_EXIT_NONZERO`) are load-bearing** — they were
  the only reliable signal in a 38KB log where the actual error appeared at line 329.
  The pattern (`echo EXIT_0 >> <log> || echo EXIT_NONZERO >> <log>`) is already documented
  in `feedback_windows_e2e_requires_bat_wrapper.md`; apply it to ALL advisor-run background
  cargo checks, not just Windows e2e.
- **The research Explore subagent was correctly calibrated** — asked for "the fix command
  or Cargo.toml lines needed" rather than open-ended research. Got a correct pin
  recommendation (Option A: `time = "=0.3.47"`) in one round-trip. This is the right shape
  for narrow dep-conflict questions: known crates, known error, "what's the minimum fix?"

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| PMD `memory_search_hybrid` (initial check) | 3 | 0 | none | Confirmed background-task-lies pattern; no new hits on bridge E0119 (novel issue) |
| Explore subagent (dep-conflict research) | ~25 | 0 | low | Correctly identified `time = "=0.3.47"` fix in one shot; would have taken 15-20 min to search crates.io/GitHub manually |
| `cargo-linux.sh` fix verification | 0 | ~15 | medium | Docker used cached registry from warmup — log was 1 line (just the marker); initially looked alarming until the 0-error grep confirmed success |
| Git log / commit check | 2 | 0 | none | Prior session had already committed the lesson correction; clean handoff |

## Complexity scores (heavy tasks only)

This session had no Junior task dispatches. The fix was advisor-side only.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Bridge time-pin fix (advisor-side, Docker verify) | 2 | 1 | ~15 (Docker build: cached, ~1 min) | ~10 (Docker run in background) |

Both tasks (diagnosis and fix) are below watchdog-risk thresholds. The "runtime" cost was
predominantly the PMD search + Explore subagent round-trip (~20 min) not the Docker build.

## Decisions to revisit

- **`time = "=0.3.47"` pin is a durable workaround, not a permanent fix.** When
  `matrix-sdk` upgrades its ruma dependency chain, revisit. Trigger: a `cargo update` in
  `services/bridge` or a matrix-sdk version bump in Cargo.toml.
- **`ruma-appservice-api = "0.16"` is also pinned loosely.** If it has a patch or minor
  bump that pulls a different ruma-common, the time pin may be insufficient. The generated
  `Cargo.lock` now freezes the full dep tree, which mitigates this — but a future `cargo
  update` would unfreeze it.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **`feedback_workspace_excluded_crate_must_have_lockfile.md`** (new lesson): workspace-excluded binary crates must have `Cargo.lock` committed; absent lock = unbounded dep resolution = latent breakage on next cold check. **1× brehon-fork bridge; pattern cross-projects.** Promote: write `.claude/lessons/feedback_workspace_excluded_crate_must_have_lockfile.md` + `memory_write` to PMD.
- [ ] **`feedback_background_task_notification_lies`** (existing lesson — reinforce): the lesson exists but the prior session failed to apply it before writing a lesson. Proposed amendment: add a line "Before writing any new lesson based on a background task outcome (warmup, build, validation), verify the in-log marker first. A lesson authored from a task-notification exit code rather than the in-log marker may encode a false premise." 1× this session; prior PMD hit confirms recurrence pattern.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
