# Session retro — 2026-05-25 — t1-validate-gate-cleared

**Harness:** claude-code
**Session window:** ~2026-05-24T18:00Z → 2026-05-25T01:45Z (~7h45m wall-clock, of which ~3h cargo wait, ~1h env discovery, ~3h45m active reasoning)
**Branch at start:** `2f0ab6a87` (`phase-v1-deps-r1`)
**Branch at end:** `265fe3aa7` (`phase-v1-deps-r1`)
**Files touched:** 34 (33 source/Cargo files in T1 + 1 DQ file in two commits)
**Commits:** 3 (all advisor-explicit: `7bd047f2d` T1, `4e9955757` DQ raise, `265fe3aa7` DQ resolve)

## TL;DR

Advisor took over T1 (diesel-async 0.8→0.9 migration) after Junior #454 hit error_max_turns at 223 turns and #455 was cancelled mid-improvise; completed the wrapper rewrite + 38 callsite migration + 30 import collapses + 1 raw `.transaction` Shape D + language.rs OrderDsl reorder via 4 parallel general-purpose Agent subagents pre-compact, then ran the post-commit validate-pending-laptop-e2e gate end-to-end (e2e 109/109 in 35m45s). The single load-bearing finding: **the DQ command list shape encodes commands but not the env that must be set before them** — first runs of `cargo test -p lemmy_api --lib` failed against a host PG the advisor couldn't authenticate to, requiring ~15 min of env discovery (Docker pgautoupgrade on port 5433 + `LEMMY_DATABASE_URL` + `LEMMY_CONFIG_LOCATION`) that was un-cited in the DQ. Top change proposal: extend the validate-pending-laptop DQ schema with optional `env_setup: []` and `harness_required: []` fields so every advisor running this gate gets the env in one read, not three iterations.

---

## What surprised us

- **Apub lib tests showed 5 failures that *looked* T1-caused but were pre-existing.** `http::community::test_get_community` failed with `assert_eq!(200, res.status())` returning 308 instead of 200 — a textbook "did the migration break something?" signature. Falsification via stash + `git checkout 2f0ab6a87 -- crates/ Cargo.toml Cargo.lock` + rebuild + rerun-single-test proved the failure exists on pre-T1 source. The honest call ("not T1-caused, log separately as kind:log") only emerges by paying ~5 min for the falsification roundtrip; the cheap call ("looks like a regression, surface to user") is the wrong one.
- **The "task notification reports exit 0" lie reproduces under `cmd //c` wrappers too.** lemmy_api lib first run came back from a backgrounded Bash with `exit code 0` in the task-notification but the log tail said `test result: FAILED. 28 passed; 7 failed`. The exit propagation through `cmd //c "scripts\\brehon\\cargo-test.bat ..." > log 2>&1; echo "...exit: $?"` returned the cmd shell's exit (which gracefully wraps the .bat) rather than cargo's. Reading the log tail caught it. Already a promoted pattern (`pattern_verify_before_trusting_shell_output.md`), reaffirmed.
- **Parallel background lib tests blocked on a shared `target/debug/` file lock.** Dispatched 3 in parallel; only 1 ran cleanly; the other 2 sat at `Blocking waiting for file lock on artifact directory`. Cargo doesn't share build artifacts across concurrent test invocations on the same target dir even when the test binaries are disjoint by crate. Cost: ~3 min of token wait before I had to TaskStop and re-run sequentially.
- **`LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1` is a footgun.** Copied the env var from an apub log message because it appeared to fix the "no config" path; instead it loaded `hostname: "unset"` from defaults.hjson which broke `test_crud` hostname assertions in dbschema and produced 3 false failures. The right knob is `LEMMY_CONFIG_LOCATION="$PWD/config/config.hjson"` — point at the dev config, don't trigger the defaults path. Not surprising in retrospect (the var name says what it does); surprising in the moment because the symptom (3 specific test failures with hostname mismatch) didn't immediately map to "you told it to load defaults".
- **The advisor session actually executes the validate-pending-laptop gate.** Pre-session I had a vague sense that the advisor "queues" or "supervises" this work; the actual handler per `.claude/refs/advisor-validation.md` is **the advisor runs the commands locally** (per the gate's name). No Junior dispatch for validate-pending-laptop. The single-session-driving-end-to-end shape was correct but worth surfacing as carry-forward: the gate is hands-on, not orchestrated.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Extend `kind: "validate-pending-laptop-e2e"` DQ schema with optional `env_setup: []` (shell commands run before `commands[]`) and `harness_required: []` (enum: `docker`, `dev-pg`, `config-file`, …). Brief template §4.4 grows an env block. Document in `.claude/rules/decision-queue.md` §"kind:validate-pending-laptop-e2e". | Every advisor running this gate skips ~15 min env discovery; canonical env always reproducible | minor (schema additive + brief template edit) | 1× this session (T1); zero prior because every prior validate-pending-laptop ran on a session that had already provisioned the dev PG. Watch-1 candidate. Promote on second occurrence. |
| 2 | Add `feedback_lib_tests_need_lemmy_config_location.md` lesson encoding: lib tests under `crates/api/`, `crates/apub/`, `crates/db_schema/` require `LEMMY_DATABASE_URL` + `LEMMY_CONFIG_LOCATION=$PWD/config/config.hjson`; `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1` is a trap (loads `hostname: "unset"` from `defaults.hjson`); ephemeral Docker `pgautoupgrade/pgautoupgrade:18.4-alpine` on port 5433 works with canonical lemmy/password creds. | Next advisor running these tests has the recipe; saves the 3-iteration env discovery | minor (single lesson file) | 1× this session; never seen before because Brehon lib tests rarely run standalone. Conditional: promote after T2/T3 confirm same env need. |
| 3 | Make `scripts/brehon/cargo-test.bat` propagate cargo's exit code through the `cmd //c` layer reliably. Specifically: the current shape allows the wrapper to return 0 when cargo returned non-zero, causing task-notification false-greens. Audit `scripts/brehon/cargo-test.bat`'s exit-handling vs `cargo-check.bat` (the latter passes Probe 4 in pre-phase audit; the former apparently doesn't under all paths). | Eliminate the verify-before-trusting-shell-output discipline as a manual check; `tail` becomes optional not mandatory | medium (wrapper script change + Probe 4 audit) | 2× this session (lemmy_api lib false-green + e2e false-green-but-actually-passed). Promotion candidate. |
| 4 | Document the cargo parallel-test target-lock pattern explicitly: `scripts/brehon/cargo-test.bat -p A` and `... -p B` run serially in a single worktree because they share `target/debug/`. Either run sequentially in the same worktree, OR run in separate worktrees with separate `CARGO_TARGET_DIR`. Add to `.claude/rules/advisor-orchestrator.md` §4 cohort dispatch as a known anti-pattern. | Next parallel cargo dispatch in a single worktree degrades-to-serial up front instead of running into the file-lock retry storm | minor (rule-doc edit) | 1× this session; uncommon — most parallel work is via Junior on separate worktrees. Watch-1 candidate. |
| 5 | Add a falsification recipe to the validate-pending-laptop handler: when a failing test is NOT in T's modified file set, run `git stash -u && git checkout <pre-T> -- crates/ Cargo.toml Cargo.lock && cargo test ... <single-failing-test>` to falsify the "T-caused" hypothesis in ~5 min before logging it as a regression. Codify in `.claude/refs/advisor-validation.md` §"validate-pending-laptop handler" → new sub-§ "Pre-existing vs T-caused triage". | Standardises the falsification discipline at the gate point where it's most valuable; prevents future "looks like a regression, surface to user" wrong calls | minor (refs/ edit) | 1× this session; saves wall-clock + correctness on every future env-adjacent test failure in a validate-pending-laptop gate. High-leverage even at 1× recurrence. |

## What to carry forward

- **Falsification before structural fix continues to deliver.** The 5-min stash+checkout+rebuild+retest discipline prevented an incorrect "T1 regression" surface to user. Per `feedback_falsifiable_hypothesis_before_structural_fix.md`, this is now the second well-documented save (2026-05-21 DQ #338 + 2026-05-25 apub lib).
- **Subagent batching for mechanical sweeps remains the right escape hatch.** Pre-compact, 4 parallel general-purpose Agent subagents shipped the 33-file callsite sweep in ~3 min wall-clock that Junior #454 burned 223 turns failing on. The brief §4.7 carve-out for advisor-side subagent batching is load-bearing.
- **The advisor authoring a HANDOVER trailer per brief §4.5 in the impl commit is the right cadence.** The trailer captured Shape distribution + callsite counts pre/post + cargo status + Cargo.lock delta — exactly what T2's first task will want as context. Brief-template-driven commit trailers continue to be the right cadence for cross-task handover.
- **Reading the log tail before trusting the task notification is non-negotiable.** Three times this session: lemmy_api lib (false-green), e2e (true-green but verified), dbschema (true-fail). Two of the three trustingly-green would have produced wrong gate calls.
- **DQ schema-v3 helper scripts work cleanly.** `dq-v3-new-entry.sh` + Write-fragment + `dq-v3-append-fragment.sh` is the canonical pattern; zero collisions, zero TypeError on mixed-id arithmetic, fragment-via-Write avoids the Windows backslash-path mangling class. Per `feedback_dq_v3_append_via_helper_script.md` — the pattern keeps holding.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| advisor takeover (replacing failed Junior #454/#455) | 60 | 0 | low | Per pre-compact decision; would have gone third-cycle Junior with same error_max_turns risk otherwise |
| parallel general-purpose subagent dispatch (pre-compact, 4 batches) | 30 | 0 | low | Mechanical sweep on 33 files in ~3 min wall-clock |
| local cargo clippy verification (T1 pre-commit) | 6 | 0 | none | Caught nothing (clean) but confirms safe-to-commit before pushing to phase branch |
| HANDOVER-trailer commit with brief §4.5 template | 5 | 0 | none | T2 will read this as §3 handover; saves T2's first-turn re-derivation |
| DQ raise via `dq-v3-*` helper scripts | 4 | 0 | none | Two DQs raised (validate-pending + kind:log); zero collisions |
| ephemeral Docker pgautoupgrade provisioning | 0 | 15 | high | DQ command list didn't encode env need; advisor rediscovered Lemmy's dev-PG pattern from first principles |
| `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1` false-path | 0 | 8 | medium | Copied from log message, produced 3 false dbschema failures, corrected to `LEMMY_CONFIG_LOCATION` |
| parallel background lib-test dispatch (3 in parallel) | 0 | 3 | medium | Cargo target-dir file lock degraded 2 of 3 to "blocking" — TaskStop + sequential rerun |
| pre-T1 falsification (stash + checkout + rebuild + retest) | 30 | 5 | high | 5-min investment prevented a wrong "T1 regression" surface; ~30 min saved vs the post-surface user-investigation cycle |
| `kind: "log"` DQ for apub pre-existing | 3 | 0 | none | Cleanly separated T1 outcome from branch backlog visibility |
| advisor-laptop DQ mutation in place (Python single-entry) | 3 | 0 | none | Per option-2 mutation pattern; pending→resolved on result=pass |
| pre-compact `/compact` discipline (pre-session) | n/a | n/a | n/a | Inherited from pre-compact session; the bootstrap was effective — no re-derivation of state required |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| T1 implementation (post-compact, by advisor) | 33 source + 2 deps (`Cargo.toml`/`Cargo.lock`) = 35 | 1 (`7bd047f2d`) | ~30 (clippy 6m + check 7m + commit/push + verification rgs) | ~13 (clippy run with no log progress) |
| T1 validate-pending-laptop-e2e gate | (test runs only — no source changes) | 0 source + 2 DQ commits | ~80 (lib tests + e2e 36m + env iteration ~15m) | ~36 (e2e silent during testcontainers/Postgres warm-up) |
| Pre-T1 falsification roundtrip | 33 stash + checkout + restore | 0 | ~7 (one targeted rebuild + single-test rerun) | ~5 |

T1 implementation file count (35) exceeds the >8 carry-forward threshold but that's expected for a workspace-wide dependency migration — bundling 33 callsites into one commit is the brief's contract, not a planning drift. T1 gate runtime ~80 min is high but dominated by e2e (36 min) + env iteration (15 min) which are both unavoidable for this gate class. No carry-forward planning signal beyond what's already in §"What to change" #1.

## Decisions to revisit

- **Calibration correction on the post-task-retro eval (ID 550):** I scored 0.85 against the rubric. Per `.claude/rules/evaluation-calibration.md`, scores >0.85 should re-read the rubric, and "0.80: clean completion with all outputs verified, no issues at all" — this session had real env-discovery friction. Honest re-score is **0.78** (goal 0.40 + tests 0.30 + clean 0.08, where 0.08 reflects ~26 minutes of friction across a ~3h45m active-reasoning session). The 0.85 eval is on the record; future calibration drift should treat this as a near-miss for over-inflation.
- **Whether `validate-pending-laptop-e2e` should auto-spin its own dev-PG container in the advisor session.** A more invasive variant of "What to change" #1 — the advisor could maintain a long-lived `lemmy-advisor-pg` container (single instance across all phases) so no DQ ever needs to encode env-setup. Cost: shared-state risk across phases; benefit: zero env friction per phase. Worth a clarify before committing to either pattern.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #3 (cargo-test.bat exit-code propagation): promote to fix the wrapper. **Recurrence: 2× this session.** Watch any future session's `cargo-test.bat` invocation; if false-green reproduces, this becomes urgent.
- [ ] Change #5 (falsification recipe in validate-pending-laptop handler): promote to `.claude/refs/advisor-validation.md`. Recurrence 1× here, but high leverage (single failed-test triage saves 30+ min user-cycle).
- [ ] Change #1 (validate-pending-laptop DQ schema `env_setup`/`harness_required`): wait for T2/T3 to confirm same env need. If T2 also requires `LEMMY_DATABASE_URL` + `LEMMY_CONFIG_LOCATION` on its lib-test commands, promote then.
- [ ] Change #2 (lib-test env recipe lesson): companion to #1; promote on same trigger.
- [ ] Change #4 (cargo parallel-test target-lock anti-pattern): single-occurrence so far; note in rules but don't promote standalone lesson until recurrence 2.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
