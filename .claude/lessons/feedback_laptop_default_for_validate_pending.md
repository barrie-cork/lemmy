---
name: Laptop default for advisor-driven validate-pending
description: GH workspace-check is ~3x slower than laptop clippy even on cache-hit (21+ min vs ~4 min warm). Default to kind=validate-pending-laptop for advisor-driven supersede / fix-validation; reserve kind=validate-pending (GH-side) for impl-task pushes where the run is canonical PR evidence.
type: feedback
originSessionId: 18bed089-5b3f-400c-953f-ef5bf32cbbee
---
# Laptop default for advisor-driven validate-pending

When the advisor session needs to validate a fix, supersede a stale validate-pending entry, or sanity-check a hygiene cherry-pick before queueing further work, default to `kind: "validate-pending-laptop"` (run cargo locally per the handler in `.claude/rules/advisor-orchestrator.md`). Reserve `kind: "validate-pending"` (GH-Actions-side) for impl-task pushes where the workflow run is the canonical CI evidence trail for a per-phase PR.

**Why:** Observed 2026-04-30 incident. Pushed `78eff0de9` to `junior/jm-d-task-5-workspace-recheck` to retrigger `cargo-validate-workspace.yml` and supersede DQ #90. Run `25181904020` was queued + executed in 21+ minutes wall-clock (failure: `unused-mut` at `admin_assign_jury.rs:653`). Same diff under `cargo clippy --workspace --features full --no-deps -- -D warnings` on the laptop completed in 4 min 12 s (warm cache); per-crate clippy on `lemmy_api` ran 7 min 31 s cold.

The GH workflow has `actions/cache@v4` configured for `~/.cargo/registry`, `~/.cargo/git`, and `target/` with key `cargo-validate-workspace-${{ runner.os }}-${{ hashFiles('Cargo.lock') }}` — and `Cargo.lock` had not changed since the last green run, so the cache *should have* hit. The slowness is GH-runner-side queue overhead + cold-start re-population of the runner image, not a cache miss.

Empirical numbers from the incident:
- GH workspace-check warm-cache: 21+ min queue+run (workspace-check job alone)
- Laptop `cargo clippy -p lemmy_api --features full -- -D warnings`: 7m31s cold
- Laptop `cargo clippy --workspace --features full --no-deps -- -D warnings`: 4m12s warm
- Laptop ratio: ~3x faster on the same diff. Cost: zero billed minutes vs ~5-10 GH-billed minutes per round.

GH-billed minutes are a finite shared resource (Actions monthly budget for the private repo). Using them for routine advisor-side validation rounds means fewer minutes available for big PR runs (cargo-test-e2e, cargo-validate-workspace on real impl-task pushes) where the workflow run *is* the canonical PR evidence and external CI signal.

**How to apply:**
- **Default for advisor-driven validate-pending:** raise `kind: "validate-pending-laptop"` from `from: "advisor"` referencing the trunk SHA (or the worker branch tip). Run the §15 DoD commands locally per `advisor-orchestrator.md` "validate-pending-laptop handler". Mutate the entry in place.
- **Use GH-side validate-pending when:** the `validate-pending` is from `from: "impl"` (a Junior worker push to `junior/*`), the run is canonical evidence for a per-phase PR's CI checks, or the PR's branch protection rule requires `cargo-validate-workspace.yml` to pass before merge. In those cases, the GH run cost is justified because the run *is* the artifact the PR cycle consumes.
- **Don't trigger a `junior/*` push solely to retrigger the workflow** if the advisor's question is "does trunk + my fix pass cargo cleanly?" Run cargo locally instead. The GH path is appropriate when the question is "does the per-phase PR's CI flow correctly identify pass/fail?" — i.e., when GH itself is what's being validated, not when cargo is what's being validated.
- **Pre-flight before laptop run:** verify clean working tree on `governance-v0` (or detached-HEAD on the worker branch per the handler), ensure `cargo` cache directory is healthy (`target/` not corrupted; consider `cargo clean -p <crate>` only if a known-bad cache state is suspected), and on Windows verify libpq.dll PATH if the run includes `cargo test` (use the `.bat` wrapper per `feedback_libpq_path_dll.md` for pre-2026-04-29 worker branches).
- **Mutate the DQ entry the same way ci-watcher would:** `result: "pass"` with `answer` describing the run + wall-clock + commands; or `result: "fail"` with `log_slice: <last 100 lines of failing command>` + `failed_commands: [<command-that-failed>]`. Set `answered_by: "advisor-laptop"` (not `"advisor"` — the laptop variant is named in the orchestrator rule).
- **Cross-reference for retros:** the retro should record validate-pending-laptop runs in §"What worked" alongside GH-side runs; the laptop runs are equally legitimate evidence for "Tasks N+M shipped clean."

**Watch for:**
- Laptop battery + low-power state can degrade cargo throughput (warm 4m can stretch to 7-10m on battery saver). Surface in polling output if relevant.
- Laptop concurrency: do not run two `validate-pending-laptop` rounds in parallel against the same `target/` directory — cargo's per-target lock will serialize them anyway and you'll just thrash the disk. Sequential is the design.
- Windows `cmd.exe /c` quoting for `.bat` wrapper invocation: pass the inner command as one quoted argument.

**Generalises to:** any persistent agent session where a CI-side workflow exists for canonical PR evidence but the agent's question is "does my local diff pass?" The cost-per-round of CI is high enough that local validation should be the default for non-canonical-evidence rounds.

**Brief-authoring constraint (v1-RT-r2, 2026-05-23):** Every `impl-task` brief whose DoD includes an e2e command MUST include this explicit guard in §4 Constraints:

> e2e runs on **laptop only** — after cargo-check/clippy/unit-tests pass, write `kind: "validate-pending-laptop-e2e"` DQ entry (commands array = all 4 validate commands, branch, phase_task) and **stop**. Do NOT run e2e on the EliteDesk worker; the laptop advisor session runs it and mutates the DQ entry.

Without this guard, workers interpret prior until-loop patterns and run e2e on the EliteDesk daemon, blocking until session timeout (v1-RT-r2 Task 2: 65-min session ended with e2e loop still running; impl was correct but commit step was missed). The recovery recipe (read worker worktree diff via SSH, copy file to lane, run all 4 commands on laptop) works but costs ~30 min overhead.

**Related lessons:**
- `feedback_clippy_rerun_after_fix.md` — when running clippy locally, re-run after applying any clippy fix because removing dead code can promote sibling bindings to also-stale state.
- `feedback_e2e_local_or_dispatch_user_choice.md` — for e2e tests specifically, the user picks local-vs-dispatch per session. The cost-asymmetry argument here for clippy/check generalises the same logic to the cargo-validate-workspace dimension.
