---
role: bm-task
verb: bm-pr
args: (no args; current branch phase-v1-RT-r1)
phase: v1-RT-r1
created: 2026-05-12
---

# Brief — `bm-pr` for phase-v1-RT-r1

## 1. Role + dispatch line

`[role:bm-task] bm-pr phase-v1-RT-r1 — see .claude/PRPs/briefs/rt-r1-bm-pr-1.md`

You are the **bm-task** subagent. Execute the `bm-pr` verb on the current branch `phase-v1-RT-r1`. This opens a PR from `phase-v1-RT-r1` into `governance-v0` on `barrie-cork/lemmy`.

## 2. Scope

**Produce:**
- New PR `phase-v1-RT-r1 → governance-v0` on `barrie-cork/lemmy` (NOT draft per `.claude/rules/phase-branch.md`).
- Title: `Phase v1-RT-r1 — reputation-tuning r1: config consts, ReputationEventSourceType, ENTRY_KIND shims, schema extensions, e2e round-trip probe`.
- Body assembled from plan + commit log (no verify report or retro exists yet at PR-open time).
- Append a `## bm: PR opened` entry to the BM runlog per `bm-pr.md`/`branch-manager.md`.
- Return the standard `/bm-pr complete` summary.

**Do NOT:**
- Open as draft (CR skips drafts).
- Open into `main` (hard refusal).
- Touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, or commit implementation changes.
- Send Telegram pings.

**Commit only:**
- Runlog append(s) required by `bm-pr.md` / `branch-manager.md` on `phase-v1-RT-r1`.

## 3. Required reading

1. `.claude/commands/bm/bm-pr.md` — operational script. Follow Phase 1–7 step-by-step.
2. `.claude/rules/branch-manager.md` — operating rules + hard refusals.
3. `.claude/rules/phase-branch.md` — phase branch flow + PR-not-draft requirement.
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
5. `.claude/agents/bm-task.md` — Junior-dispatched bm-task contract.
6. Source files for PR body assembly:
   - `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` — §1 Summary.
   - `git log governance-v0..phase-v1-RT-r1 --oneline` — 60 commits.
7. Lessons:
   - `feedback_pr_per_phase.md`
   - `feedback_gh_pr_fork_repo_flag.md`
   - `feedback_pipes_mask_exit_codes.md`

## 4. Constraints

- **Pre-conditions at brief-write time:**
  - `origin/phase-v1-RT-r1` HEAD is `0c2e0f3ef` (ci-watcher mutated DQ #209 — pass).
  - Phase branch is 60 commits ahead of `origin/governance-v0`.
  - `gh pr list --repo barrie-cork/lemmy --head phase-v1-RT-r1` expected to return no existing PR.
  Re-verify all preconditions before creating the PR. If drift found, file a pending DQ blocker from `bm`, commit + push, and return `blocked-on-DQ-#<id>`.

- **Body assembly:**
  - `## Summary` — from plan §1: v1-RT-r1 ships the reputation-tuning r1 substrate: 26 new config consts + `SEEDED_KEYS` parity, `ReputationEventSourceType` Rust enum, 7 `ENTRY_KIND_*` consts + api shim re-exports + governance-log entry-kind registry update, schema extensions for `reputation_event` + `sponsor_allowlist` new fields, `ReputationEventInsertForm` struct extension (dedupe_key + source_event_type), all existing callsites padded, and an e2e round-trip migration probe (count bumped to 18).
  - `## Plan reference` = `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md`.
  - `## Completion report` = `Pending — no retro report at PR-open time`.
  - `## Commits` = full `git log governance-v0..phase-v1-RT-r1 --oneline`, one bullet per commit.
  - `## Closes` = `_(none)_` unless commit messages contain `closes #N`.
  - `## Validation` = Phase 1 workspace checks passed: DQ #198/#203/#204/#206/#207/#208/#209 all resolved pass. No Phase 2 e2e yet (runs post-merge per Shape G plan §15).

- **Title is fixed:**
  ```
  Phase v1-RT-r1 — reputation-tuning r1: config consts, ReputationEventSourceType, ENTRY_KIND shims, schema extensions, e2e round-trip probe
  ```

- **No-AskUserQuestion:** any script branch that would ask the user becomes a DQ-stop.
- **`--repo barrie-cork/lemmy`:** mandatory on every `gh pr` invocation.
- **Pipes mask exit codes:** never pipe `gh` through `tail`/`grep` when exit status matters.
- **Runlog commit message:** `chore(bm): record PR-open for phase-v1-RT-r1`.
- **No Telegram this run.**

## 5. Expected output

```markdown
## /bm-pr complete

**PR #<N>:** Phase v1-RT-r1 — reputation-tuning r1: ...
**URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Base ← Head:** governance-v0 ← phase-v1-RT-r1
**Body source:** plan + commits
**Draft?** No (CR-eligible)
```
