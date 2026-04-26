---
name: Pre-phase plan-review DoD smoke test
description: Before signing off on a plan that drives an autonomous loop (ralph, PRP, Junior), run every validation command named in the plan's task definitions-of-done against the starting HEAD. Unexecutable DoDs masked as "will work when the code is written" are the single biggest advisor-side miss.
type: feedback
originSessionId: 50cf5c6f-6b81-45b0-b3b4-45ac46b352d8
---
When reviewing a plan that drives an autonomous loop (`/prp-ralph`, `/prp-implement`, Junior, similar), **run every validation command named in the plan's task definitions-of-done against the starting HEAD** before signing off. Commands that look reasonable in prose are often unexecutable when run literally, because the plan author didn't smoke-test them.

**Why:** Hit during Brehon Phase 2a advisor review (2026-04-15). The plan's §9 task DoDs included `cargo clippy -p <crate> -- -D warnings` as a gate. The advisor signed off on the plan without running the command. When the ralph loop hit the DoD at task 15, it failed for two reasons neither the advisor nor the plan author had anticipated: (a) governance code was behind `#[cfg(feature = "full")]` so without `--features full` the clippy pass saw nothing relevant; (b) without `--no-deps`, clippy linted upstream `lemmy_diesel_utils::pagination` which had pre-existing lint debt under a deny-warnings gate. Both issues are one-flag fixes (`--features full`, `--no-deps`) but they would have been caught in 5 minutes of plan review and instead cost a mid-phase checkpoint-2 cascade, a plan-reframe commit, and ~30 minutes of advisor diagnosis.

**How to apply:**

- **During plan review, before approving the plan,** open the plan file and identify every validation command the plan cites in its DoD sections (§9 in PRP templates, or equivalent).
- **Run each command literally against the prior-phase HEAD.** Capture exit code. Do not assume; run it.
- **Classify the expected result:**
  - Commands that check new-code-under-development → **expected red** (the code doesn't exist yet). Note these as expected-red.
  - Commands that check unchanged code or baselines → **expected green**. If any unchanged-code command is currently red, the plan's DoD is citing unexecutable state.
- **For each currently-red unchanged-code command, decide:**
  - Fix in a pre-phase commit (e.g. clear lint debt in a `chore(lint):` commit on the base branch before the phase starts)
  - OR narrow the plan's DoD (add `--no-deps`, `-p <crate>`, `--features <flag>`) so it doesn't inherit the broken state
- **Specific clippy DoD footguns to check for:**
  - Missing `--features <flag>` where code is feature-gated (code doesn't get compiled, so clippy either passes trivially or fails on unrelated code)
  - Missing `--no-deps` where upstream has pre-existing lint debt (upstream debt blocks your deny-warnings gate)
  - Using `--workspace` where you mean per-crate (masks scope issues, may be overly permissive or overly strict)
- **Specific cargo-test DoD footguns:**
  - Missing `-p <crate>` so cargo builds more than needed
  - Missing `--test <name>` so cargo runs every test target
  - Naming a test binary that doesn't exist yet (this is OK if expected-red, but confirm the test will land in the same task)

**Generalizes to:** any structured plan with explicit validation gates, including PRP plans, Junior task definitions, runbook validation steps, deployment checklists. The pattern is "trust but run" — trust that the plan author meant well, but run the commands literally to catch the cases where "meant well" missed an executable detail.

**Time cost:** ~5-10 minutes per plan for plans with 10-20 validation commands. Saves ~30 minutes to several hours when it prevents a mid-loop checkpoint cascade.

**Symptom to recognise in retrospect:** a mid-loop failure where the agent stops and says "the DoD command exits non-zero on the current state" or "this command isn't executable here," and the fix is a one-flag addition to the command. That failure mode was catchable in plan review if the advisor had run the command.

**Mitigation pattern for the Brehon fork:** captured in `.claude/advisor-context-phase-2b.md §5 rule 11` — mandatory advisor-side plan-review DoD smoke test before approving any ralph-run plan. Generalizes to any project with plan-driven autonomous loops.
