# Session retro — 2026-06-12 — m2-late-2-plan-approval-handoff

**Harness:** claude-code
**Session window:** ~2026-06-12 11:40 → 13:10 UTC (~90 min, spanning one compaction at start)
**Branch at start:** `649621dce` (`governance-v0`)
**Branch at end:** `7c9b586bd` (`governance-v0`) + `fdc1cd1c2` (`phase-m2-late-2`)
**Files touched:** 3 (1 new agent, 1 plan rider, 1 bootstrap refresh)
**Commits:** 2 (auto: 0, explicit: 2 advisor commits)

## TL;DR

Resumed an m2-late-2 orchestration thread post-compaction, correctly treated the inherited "next action" as a hypothesis and re-verified live state before acting — which immediately paid off: the planning Junior (#660) had finished and the daemon had merged the plan **daemon-local only**, not pushed to origin (the documented daemon-local-first pattern). The most load-bearing finding was a **DoD smoke-test catch**: `services/bridge` `cargo check` fails on the Windows host (`ruma-common v0.19.0` E0119 conflicting-trait-impl vs `time`) — a host-toolchain quirk, not a code defect, that would have silently broken every bridge `validate-pending-laptop` validation in the upcoming impl phase. User confirmed the Docker/`cargo-linux.sh --manifest-path` route; the decision was made durable in three places (plan §15.4 rider, bootstrap RESUME, watchlist correction) so the new `/auto-phase` session reads it instead of re-deriving the failing path. Top change proposal: **add a DoD-smoke-test pre-flight that runs the bridge cargo on its real target (Docker-Linux), not the Windows host, for any plan touching `services/bridge`** — this session caught it manually; it should be mechanical.

---

## What surprised us

- **The bridge does not compile on the Windows host at all** — `ruma-common v0.19.0` hits E0119 (`From<...HourBase>` conflicting impl vs the `time` crate). This is on the *phase base*, before any m2-late-2 edit, so it's a host-toolchain/registry-state quirk, not introduced code. Surprising because the plan's Task 0 Probe 4 and §15.4 all *EXPECT bridge `cargo check` exit 0 locally* — the planner assumed the bridge builds on the laptop. It doesn't. (The bridge's real deploy target is Linux, so this was never load-bearing for the pilot — but it IS load-bearing for the impl phase's local validation path.)
- **The bootstrap/watchlist claimed `services/bridge/Cargo.lock` is "tracked"** — it isn't. The file doesn't exist in the repo; it's generated on first `cargo` build and cleaned. My own earlier `cargo check` *created* a stray untracked `Cargo.lock` in the validation worktree, which confirmed the generation behaviour and would have leaked into a commit if not caught at `git status` pre-commit.
- **`cargo-linux.sh` already supports the workspace-excluded bridge** via `--manifest-path services/bridge/Cargo.toml` — the "put the Linux bridge in place" work the user anticipated turned out to be ~one rider edit, not a wrapper-script change. The wrapper is scope-agnostic by design (`feedback_wrapper_script_flag_silence.md`), so `--manifest-path` is a plain passthrough.
- **The Windows `git show <branch-with-slashes>:<path>` colon→semicolon mangling fired again** — `origin/phase-m2-late-2:.claude/...` became `origin\phase-m2-late-2;.claude\...` and silently returned a 3-line error as if it were file content. This is a *known* trap (`feedback_windows_bash_python_git_show_tmp_traps.md`) but I hit it before reaching for the helper. The recovery (`git-show-json.sh` + `resolve-dq-canonical.sh`) was clean once invoked.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add a **bridge-on-Linux DoD pre-flight** to the §3.4 smoke-test discipline: when a plan's §15 includes `services/bridge` cargo commands, run them via `cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker), NOT the Windows-local `cd services/bridge && cargo` form. Encode as a row in `advisor-orchestrator.md` §3.4 or a new lesson `feedback_bridge_validates_on_linux_not_windows.md`. | DoD smoke test reflects the bridge's real validation target; no impl-phase surprise when the first bridge `validate-pending-laptop` runs | minor (one lesson + one §3.4 line) | 1× this session + the deploy-target gate already exists (`feedback_linux_compile_proof_is_a_gate.md`) → promote |
| 2 | **Planner should not assume `services/bridge` builds on the laptop host.** Add a planning-side watchpoint/guardrail: any plan whose §15 has bridge cargo must specify the `cargo-linux.sh --manifest-path` form and tag bridge validation as Linux-only. Inject into the planning brief template or `planning.md` agent. | Removes the wrong "EXPECT exit 0 locally" premise at plan-author time, not at DoD-smoke time | minor | 1× this session; the m2-late-2 plan shipped with the wrong premise in 8 sites |
| 3 | **Reach for `git-show-json.sh` / `resolve-dq-canonical.sh` FIRST when reading a JSON/MD file out of a slash-containing ref on Windows** — do not attempt raw `git show <branch>:<path>`. Add a one-line reflex note to `feedback_windows_bash_python_git_show_tmp_traps.md` ("the helper is the first move, not the recovery"). | Saves the ~2-min mangle-then-recover loop that recurs whenever a phase-branch DQ/plan is read from canonical | trivial (one line on an existing lesson) | 1× this session + repeated historically (lesson already exists) |
| 4 | **Correct the bootstrap-handover claim that `services/bridge/Cargo.lock` is tracked** — done for m2-late-2, but the bootstrap *template* (or whatever generated the §4.5 claim) may carry the same error forward. Grep handover/bootstrap generators for "Cargo.lock is tracked" / "bridge ... lockfile". | Prevents the wrong "lockfile-change → Linux gate" trigger logic propagating to the next bridge phase | trivial | 1× this session (the claim appeared in 2 places in one bootstrap) |

## What to carry forward

- **Re-verify inherited "next action" as a hypothesis after compaction.** The compaction summary explicitly flagged its next-action as a hypothesis; re-running `show_task` + `list_tasks` + git/DQ state first caught the unpushed-plan condition before I acted on stale assumptions. Worked cleanly; do it every resume.
- **Daemon-local-first probe order when a Junior task on a trunk/phase branch reports done.** Checked daemon-local `phase-m2-late-2` (had the merge at `c6a22e072`) BEFORE concluding the plan was missing from origin. Matches `advisor-orchestrator.md` §3.1 + `feedback_finalize_merge_where_to_look_first.md`. Avoided a multi-probe hunt.
- **Throwaway validation worktree for phase-branch cargo from canonical (Mode B).** `git worktree add` on `origin/phase-m2-late-2` (never a bare checkout in canonical, per multi-lane hard refusal #1), bootstrapped submodules + `.env` + `.mcp.json`, ran the DoD smoke test, then `git worktree remove`. Clean isolation; the `git status` pre-commit check caught the stray generated `Cargo.lock` before it leaked.
- **Make a session-discovered decision durable in the place the next session reads, not just in chat.** The Docker-bridge decision was written to plan §15.4 (authoritative rider) + bootstrap RESUME + watchlist correction, then the daemon-local phase branch was FF'd so `/auto-phase` workers fork from the rider. A decision that lives only in the conversation dies at the session boundary.
- **Atomic stage-only-mine commits on the shared canonical `.git/`** — staged only `brehon-state-status.md`, verified with `git status --short` + `git show --stat HEAD`, per multi-lane hard refusals #6/#7. No race.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| DoD smoke test (§3.4, manual) | 30+ | 0 | high | Caught the bridge-Windows-compile failure that would have broken every bridge `validate-pending-laptop` in the impl phase. Highest-leverage act of the session. |
| `git-show-json.sh` + `resolve-dq-canonical.sh` | 5 | 2 | low | Clean once invoked; the 2 wasted min were attempting raw `git show <branch>:<path>` first (change #3). |
| Throwaway validation worktree (Mode B) | 10 | 0 | none | Correct isolation for phase-branch cargo from canonical; no contamination. |
| `brehon-state-status` agent (created) | — | 0 | none | Proposal-shaped — not yet invoked. Read-only Haiku probe; sibling-checked against `ci-watcher.md` frontmatter. Future-session leverage, not this session. |
| AskUserQuestion (gate 1 + persist-decision) | 5 | 0 | low | Clean forks: bridge-validation route, plan-approval hold, persist-decision method, scope-Linux-bridge. User chose HOLD — correct gate behaviour, no auto-advance. |
| `§3.5` watchpoint-specificity gate | 3 | 0 | none | Passed — R1/R8/R9 all cite specific file:line, verified against live code. |
| Resume-prompt re-fire reconciliation | 8 | 0 | low | Second identical resume prompt fired mid-session; reconciled against already-done work instead of re-running every step. Avoided redundant DoD re-run. |
| Workspace `cargo check --workspace --features full` (background) | — | 0 | none | exit 0, 15m18s, zero errors. Backgrounded correctly; notified on completion; relayed exit code. |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. No Junior impl-tasks ran this session (it held before `/auto-phase`). The one heavy advisor-side task:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Workspace DoD `cargo check` (validation worktree, background) | 0 (read-only validate) | 0 | 15 | 0 (continuous compile output) |

No task breached the >55min runtime / >40min silence / >8 files thresholds. The 15m workspace check is well inside the envelope.

## Decisions to revisit

- **The `brehon-state-status` agent is unproven** — created this session, not yet invoked. Its real value (keeping git/gh/DQ/Junior probing out of main context) is a hypothesis until a future session dispatches it and reports whether the synthesis is usable on first read. Carry forward to the next resume: invoke it instead of inline probing and score it.
- **First bridge `cargo-linux.sh` run will be cold (~10–20 min)** — the user may want a manual warm-up before the `/auto-phase` session so the first bridge validation isn't cold inside the orchestration loop. Noted in handoff; not yet acted on.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

Boxes unchecked by default; user checks to authorise.

- [ ] Change #1 (bridge-validates-on-Linux-not-Windows): promote to `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` (cross-harness lesson) — meets threshold via the existing `feedback_linux_compile_proof_is_a_gate.md` companion.
- [ ] Change #2 (planner must not assume bridge builds on laptop): update `.claude/agents/planning.md` or the planning brief template with a bridge-cargo-is-Linux watchpoint.
- [ ] Change #3 (helper-first for `git show` on slash refs): one-line append to existing `.claude/lessons/feedback_windows_bash_python_git_show_tmp_traps.md`.
- [ ] Change #4 (correct "Cargo.lock is tracked" in bootstrap generator): grep + fix the handover/bootstrap source of the claim.
- [ ] `brehon-state-status` agent: already created at `.claude/agents/brehon-state-status.md` (committed `7c9b586bd`) — no promotion needed; carry-forward is to *invoke and score* it next session.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section omitted —
session held before `/auto-phase`; no auto-state mutation (Step 0.5 trigger did not fire)._
