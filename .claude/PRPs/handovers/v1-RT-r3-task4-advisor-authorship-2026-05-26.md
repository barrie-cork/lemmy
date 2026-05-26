# Handover — v1-RT-r3 Task 4 advisor-side authorship (post-#477 cycle-count catch-fire)

**Authored:** 2026-05-26
**Authoring session:** brehon-fork canonical (governance-v0)
**Resume in:** new session (lane mode: Mode B mobile remote-control OR Mode A lane worktree — user's choice; see "Lane mode decision" below)

---

## RESUME block (read first; 3 tool calls to operate)

**Current state:**
- `origin/governance-v0` @ `8814ae893` (chore(decision-queue): DQ -032 #476 cancel log)
- `origin/phase-v1-RT-r3` @ `ee04d3328` (merge of `8814ae893` into phase branch via daemon SSH; clean auto-merge)
- Junior tasks #474 #475 #476 #477 all `error_max_turns` on Task 4. Cycle-count meta-rule §5.3 triggered: **4 fails same `(error_max_turns, e2e.rs)`** ≥3 → HARD REFUSAL on further dispatch.
- **User authorised (B) advisor-side authorship** of Task 4 e2e tests as the corrective replan. This is a carve-out from `crates/**` discipline (advisor normally never authors `crates/**`).

**Next concrete action (new session, after the 4 reads below):** start authoring the e2e test module `v1_rt_r3_fixtures` into `crates/server/tests/e2e.rs` per `.claude/PRPs/briefs/v1-RT-r3-impl-4.md`, mirroring `v1_ship_3_fixtures` at `crates/server/tests/e2e.rs:16699-17099`.

**Required first reads (in order):**
1. `.claude/PRPs/briefs/v1-RT-r3-impl-4.md` — the brief Task 4 was supposed to ship. §2.0 (Task-0 pre-flight), §2.1 (canonical sibling mirror), §2.3 (EnvVarGuard RAII), §2.4 (story-to-test mapping 5 stories / 10 tests), §3.2 (6 `DEFAULT_*` consts to import), §4 (constraints — NO unwrap/expect; `LemmyResult<()>` outer; `?` propagation only).
2. `crates/server/tests/e2e.rs:16699-17099` — the canonical sibling `v1_ship_3_fixtures` module to mirror verbatim in error shape (LemmyResult<()> outer + LemmyResult<T> helpers; `.ok_or_else(|| anyhow::anyhow!("..."))?` for `Option → Result`; `.map_err(|e| anyhow::anyhow!("{e}"))?` for foreign errors).
3. `.claude/PRPs/plans/v1-RT-r3.plan.md` §13 Task 4 + §16a Story 1-5 — the plan-level Brief-Scope outputs each test must satisfy (verify gate at end will check these mechanically).
4. `.claude/decision-queue.json` — most recent advisor entries (`a3d0e9941441-025` through `-032`) for audit findings that informed brief tightenings.

**Lane mode decision (read multi-lane-worktree.md §"Lane modes" if uncertain):**
- The carve-out is **advisor-authored on the phase branch directly**. The advisor session MUST be on `phase-v1-RT-r3`, not `governance-v0`. Two ways:
  - **Mode A (RECOMMENDED for this carve-out):** create lane worktree `C:/Users/barri/Developer/brehon-fork-rt-r3`, open new Claude Code session in it, branch = `phase-v1-RT-r3`. Bootstrap: `git worktree add ../brehon-fork-rt-r3 phase-v1-RT-r3` + `cd ../brehon-fork-rt-r3 && git submodule update --init --recursive && cp ../brehon-fork/{.mcp.json,.env,.claude/settings.local.json}` (per `feedback_phase_lane_worktree_bootstrap_checklist.md`). This is the canonical lane for crates/** authorship. Reason for RECOMMENDED: cargo iteration locally (E0271-fix loop) needs the worktree on the phase branch; canonical session can't `git checkout phase-v1-RT-r3` per multi-lane Hard refusal #1.
  - **Mode B (workable but adds friction):** new session in canonical `brehon-fork` on `governance-v0`, author the file delta in memory, then SSH-commit via daemon push proxy. Slower per-iteration; not recommended for crates/** authorship that needs local cargo-check feedback.

---

## Context (what the next session needs to know)

### Why advisor is authoring crates/** (carve-out justification)

Per advisor-orchestrator.md the advisor "never authors content." This carve-out applies once because the cycle-count meta-rule §5.3 HARD REFUSAL fired on Junior dispatch and the user explicitly authorised option (B) "advisor-side authorship" over the other three (A=split, C=narrower, D=catch-fire-only). Record in retro that this was a user-authorised carve-out, not a precedent.

### What Junior #477 produced (lost) — direction-confirming signal

Per the prior session's #477 triage:
- Junior fired **6 Edits** into `e2e.rs` between turn ~10-50 of #477's run (productive phase before the recon-drift). The Edits were: `app_data` context setup, `JuryDecision::NoAction` fixture, `JuryDecision::RemoveContent` fixture, `count_reputation_events_for` helper, plus 2 others (need to read log to recover specifics — log at `homeserver:/srv/brehon-fork/.junior/logs/job-477-run-478.log`, 5.88 MB).
- All 6 Edits **lost** when `error_max_turns` triggered daemon auto-reap of worktree before forensics tar.
- Junior reached **E0271** at `e2e.rs:17432`: `<ReputationEventSourceType as SqlType>::IsNull == IsNullable` expected `IsNullable` found `NotNull` on a `.load(conn)` call. This is a real Diesel type-resolution error — likely needs either (a) the Queryable derive on `ReputationEventSourceType` to flip nullability, or (b) the query to use `select(...).load::<...>(conn)` with explicit type annotation.

**Implication for advisor authorship:** read the canonical sibling shape FIRST, then design the 5-story / 10-test surface against it, then iterate cargo-check locally. Don't try to reconstruct Junior's 6 Edits — they were context-bound and the lossless path forward is start from the canonical sibling.

### Cycle-count meta-rule rationale

4 consecutive `error_max_turns` on the same task is loud structural signal: the brief+dispatch combination does NOT fit through 151 turns regardless of hardening (the dispatch-string Task-0 encoding ALSO failed — see DQ -032 + this session's eval 586 + 587 for the experiment record). Advisor authorship eliminates the turn budget as a binding constraint; the work moves to the advisor's runway.

### Concrete authoring shape

The brief calls for 2 Edits (~150 helpers + ~450 tests). Advisor-authored, this can be a single Write of the full module to a new section of `e2e.rs` between two well-known anchors (read `e2e.rs:17099` first to find where `v1_ship_3_fixtures` ends, then append `v1_rt_r3_fixtures` after a blank line). Total expected diff: ~600 lines.

Validation gate per brief §4 + plan §13 Task 4 DoD:
- `bash scripts/brehon/cargo-check.sh --workspace --features full` exit 0
- `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` runs (full e2e is ~26 min; this is the user-gate-4 local-vs-dispatch decision after the module compiles)

### DQ entries to be aware of (current state)

- `a3d0e9941441-025` through `-030` — audit findings that informed brief tightenings (story-to-test mapping, `DEFAULT_*` const verification, EnvVarGuard pattern, etc.)
- `a3d0e9941441-031` — carry-forward for daemon-side `git worktree add` submodule init (filed last session; structural fix lives in `MCPs/junior-mcp` repo, not this one)
- `a3d0e9941441-032` — log entry documenting #476 cancel + false-promise sub-signature; informs the cycle-count rationale

No DQ entries are *pending* — all are resolved. Carry-forward fix (`-031`) is a separate PR-class in a different repo.

---

## ASSUMES (verify before committing to action)

Per `feedback_handover_assumptions_need_empirical_verification.md`, each ASSUMES line below MUST be verified with the named VERIFY command in ≤5 min before the patch lands.

- **ASSUMES** `crates/server/tests/e2e.rs:16699-17099` still contains the `v1_ship_3_fixtures` module unchanged. **VERIFY:** `git show origin/phase-v1-RT-r3:crates/server/tests/e2e.rs | sed -n '16699,16720p'` shows the module declaration. If the line range has shifted, recompute via `grep -n 'mod v1_ship_3_fixtures' crates/server/tests/e2e.rs`.
- **ASSUMES** the 6 `DEFAULT_*` consts at `crates/api/api/src/governance/config.rs:813-1023` still exist with the names cited in brief §3.2. **VERIFY:** `grep -n 'pub const DEFAULT_DELTAS_PARTICIPATION\|DEFAULT_PARTICIPATION_EVIDENCE' crates/api/api/src/governance/config.rs` returns 6 hits with the expected slugs.
- **ASSUMES** the brief's §2.4 story-to-test mapping (5 stories / 10 tests) is current and not invalidated by any plan §16a edit. **VERIFY:** read `.claude/PRPs/plans/v1-RT-r3.plan.md` §16a Story 1-5 and confirm each story's "Brief-Scope outputs" matches the brief's test-name list. Disagreement = plan drift; resolve before authoring.
- **ASSUMES** Mode A lane worktree at `C:/Users/barri/Developer/brehon-fork-rt-r3` exists (per workflow_state_v1_RT_r3.md noting "Lane worktree brehon-fork-rt-r3 pre-created at eaa4669ea"). **VERIFY:** `ls C:/Users/barri/Developer/brehon-fork-rt-r3 && git -C C:/Users/barri/Developer/brehon-fork-rt-r3 branch --show-current` returns `phase-v1-RT-r3`. If absent or on a different branch, recreate via `git -C C:/Users/barri/Developer/brehon-fork worktree add ../brehon-fork-rt-r3 phase-v1-RT-r3`.
- **ASSUMES** `phase-v1-RT-r3 @ ee04d3328` is the canonical authoring base. **VERIFY:** `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git rev-parse origin/phase-v1-RT-r3` matches `ee04d3328`. If it has advanced, rebase the lane worktree before authoring.

---

## Exit criteria (when Task 4 is done in the new session)

1. `v1_rt_r3_fixtures` module exists in `crates/server/tests/e2e.rs` mirroring `v1_ship_3_fixtures` shape.
2. 10 tests across 5 stories per brief §2.4; each test wired to its `DEFAULT_*` const per brief §3.2.
3. `bash scripts/brehon/cargo-check.sh --workspace --features full` exits 0.
4. Commit on `phase-v1-RT-r3` with subject like `feat(governance): v1-RT-r3 task 4 — e2e tests 10 across 5 stories (advisor-authored carve-out per cycle-count §5.3)` and a HANDOVER trailer naming files created + modified.
5. Push `phase-v1-RT-r3`.
6. Run user-gate-4 local-vs-dispatch e2e (advisor surfaces options; user picks).
7. After e2e passes, advance to `/brehon-verify` → bm-merge gate.

---

## See also

- `.claude/decision-queue.json` resolved entries `a3d0e9941441-025` through `-032`
- `.claude/PRPs/briefs/v1-RT-r3-impl-4.md` (the brief Task 4 was supposed to ship)
- `.claude/PRPs/handovers/v1-RT-r3-bootstrap.md` (lane bootstrap from when the phase opened)
- PMD evals `586` + `587` (this session's retros documenting #476 cancel + #477 error_max_turns + cycle-count trigger)
- `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` (the ASSUMES/VERIFY discipline)
- `.claude/rules/advisor-orchestrator.md` §5.3 cycle-count meta-rule (the discipline that mandated the catch-fire)
- `.claude/rules/multi-lane-worktree.md` §"Lane modes" (Mode A vs Mode B decision)
