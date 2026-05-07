---
role: bm-task
verb: bm-pr
args: (no args; current branch phase-v1-SL-b)
phase: v1-SL-b
created: 2026-05-06
---

# Brief — `bm-pr` for phase-v1-SL-b

## 1. Role + dispatch line

`[role:bm-task] bm-pr phase-v1-SL-b — see .claude/PRPs/briefs/sl-b-bm-pr-1.md`

You are the **bm-task** subagent. Execute the `bm-pr` verb on the current branch `phase-v1-SL-b`. This opens a PR from `phase-v1-SL-b` into `governance-v0` on `barrie-cork/lemmy`.

## 2. Scope

**Produce:**
- New PR `phase-v1-SL-b → governance-v0` on `barrie-cork/lemmy` (NOT draft per `.claude/rules/phase-branch.md`).
- Title: `Phase v1-SL-b — revoke_endorsement handler + DTO + route + 9 integration tests`.
- Body assembled from plan + verify report + commit log. There is no `v1-SL-b-retro.md` and no `v1-SL-b-complete-report.md` at brief-write time; do not invent one.
- Append a `## bm: PR opened` entry to `.claude/runlog/v1-SL-b-runlog.md` or the BM runlog location required by the current `bm-pr.md`/`branch-manager.md` script.
- Return the standard `/bm-pr complete` summary.

**Do NOT:**
- Open as draft (CR skips drafts).
- Open into `main` (hard refusal — trunk for v1 work is `governance-v0`).
- Touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, or commit implementation changes.
- Send Telegram pings. Advisor will queue `bm-ping` separately if wanted.
- Edit sealed implementation commits or rewrite branch history.

**Commit only:**
- Runlog append(s) required by `bm-pr.md` / `branch-manager.md` on `phase-v1-SL-b`.

## 3. Required reading

1. `.claude/commands/bm/bm-pr.md` — operational script. Follow Phase 1–7 step-by-step.
2. `.claude/rules/branch-manager.md` — operating rules + hard refusals.
3. `.claude/rules/phase-branch.md` — phase branch flow + PR-not-draft requirement.
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
5. `.claude/agents/bm-task.md` or `.pi/skills/bm-task/SKILL.md` if present — Junior-dispatched bm-task contract.
6. Source files for PR body assembly:
   - `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` — H1/title + §1 Summary.
   - `.claude/PRPs/reports/v1-SL-b-verify.md` — verify reference, all 3 §16a stories ✓.
   - `git log governance-v0..phase-v1-SL-b --oneline` — commit list.
7. Lesson scan:
   - `feedback_pr_per_phase.md`
   - `feedback_branch_manager_pm_split.md`
   - `feedback_gh_pr_fork_repo_flag.md`
   - `feedback_pipes_mask_exit_codes.md`

## 4. Constraints

- **Pre-conditions at brief-write time:**
  - `origin/phase-v1-SL-b` HEAD is `189755617a3dd781d1735bebc5f76fad8b6991ad` (`docs(advisor): brehon-verify v1-SL-b — all stories ✓`).
  - Working tree in the brief-writing worktree was clean after this brief commit is expected to land.
  - `phase-v1-SL-b` is 49 commits ahead of `origin/governance-v0`.
  - `gh pr list --repo barrie-cork/lemmy --head phase-v1-SL-b` returned no existing PR.
  - `/brehon-verify v1-SL-b` passed: `.claude/PRPs/reports/v1-SL-b-verify.md` reports 3 stories, 3✓, 0 phantom, 0 regression, 0 malformed.
  Re-verify all Phase 1 preconditions before creating the PR. If drift is found, file a pending DQ blocker from `bm`, commit + push, and return `blocked-on-DQ-#<id>`.

- **Body assembly substitutions:**
  - `## Summary` = one concise paragraph from plan §1 Summary: SL-b ships `POST /api/v4/governance/endorsement/revoke`, extends the DTO/response, creates the handler, registers the route, and adds 9 integration tests covering PRD §5 and §5.3 grace-window severance.
  - `## Plan reference` = `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md`.
  - `## Completion report` = `Pending — no completion/retro report exists at PR-open time`.
  - Add `## Verify reference` = `.claude/PRPs/reports/v1-SL-b-verify.md` (all 3 §16a stories ✓).
  - `## Commits` = full `git log governance-v0..phase-v1-SL-b --oneline`, one bullet per commit.
  - `## Closes` = no direct GitHub issue closure unless commit messages contain `closes #N` / `fixes #N`. If none, write `_(none)_`.
  - `## Validation` = mention Shape G validation already recorded in verify report: workspace-check runs `25342530143` and `25346692356` success; e2e run `25352168003` success. Leave room for `/bm-prp-review` to append its own validation summary later if the script expects that.

- **Title is fixed:**
  ```
  Phase v1-SL-b — revoke_endorsement handler + DTO + route + 9 integration tests
  ```

- **No-AskUserQuestion:** any script branch that would ask the user becomes a DQ-stop; do not attempt an interactive prompt in Junior `-p` mode.
- **Linux discipline:** EliteDesk worker. Use `bash`/`git`/`gh`.
- **`--repo barrie-cork/lemmy`:** mandatory on every `gh pr` invocation.
- **Pipes mask exit codes:** never pipe `gh` through `tail`/`grep` when the exit status matters.
- **Runlog commit message:** `chore(bm): record PR-open for phase-v1-SL-b`.
- **No Telegram this run.**

## 5. Expected output

Return the standard 7-section summary, including:

```markdown
## /bm-pr complete

**PR #<N>:** Phase v1-SL-b — revoke_endorsement handler + DTO + route + 9 integration tests
**URL:** https://github.com/barrie-cork/lemmy/pull/<N>
**Base ← Head:** governance-v0 ← phase-v1-SL-b
**Body source:** plan + verify + commits
**Draft?** No (CR-eligible)

### What happens next (automatic)

1. CodeRabbit will review within ~5–10 min.
2. Run `/bm-poll-cr <N>` to ingest findings.
3. Run `/bm-prp-review <N>` to add Brehon ADR + cargo review.
4. Triage with `/bm-triage <N>` → confirm before posting digest.
5. Merge with `/bm-merge <N>` → confirm before merging.

### Telegram ping

NOT sent this run — advisor will queue `/bm-ping pr-ready` separately if wanted.
```
