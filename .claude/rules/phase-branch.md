# Phase branch + PR flow (Phase 5 onwards)

From Phase 5 onward, every Brehon **sub-phase delivery** runs on its own phase branch and closes via a PR into `governance-v0`. Phases 1–4 were grandfathered onto direct `governance-v0` commits and are not retroactively PR'd.

## What goes through the PR flow vs direct on governance-v0

The fork is private (no AGPL §13 trigger pre-pilot per `project_brehon_agpl_repo_privacy.md`), so the PR flow exists for CodeRabbit review value, not for visibility. CR review value lives in code review — `crates/`, `migrations/`, `tests/` — not in meta-work. So:

**Goes through PR flow (phase branch + CR auto-review):**
- Sub-phase deliverables that touch `crates/**`, `migrations/**`, `tests/**`, `crates/server/tests/**` — these are CR's lane and the review actually catches things
- Any change to `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.coderabbit.yaml` itself
- Any phase deliverable with a corresponding plan file in `.claude/PRPs/plans/`

**Direct on `governance-v0` (no phase branch, no PR):**
- `.claude/` meta-work: lessons (`.claude/lessons/*.md`), agent definitions (`.claude/agents/*.md`), rules (`.claude/rules/*.md`), commands (`.claude/commands/*.md`), briefs, PRPs templates, decision-queue answers
- `docs/` updates that aren't design-doc rewrites: research notes, reports, retros, runlog
- `scripts/` infrastructure: cargo wrappers, hooks, tooling
- `.gitignore`, `.mcp.json.example`, sync manifests
- Any `chore(rls):`, `chore(decision-queue):`, `chore(advisor):`, `chore(lessons):`, `docs(advisor):`, `docs(rules):`, `docs(retro):` commit whose diff stays inside the "Direct" file set above

The litmus test: if CodeRabbit's review would be net-noise rather than net-signal on this diff, commit direct. CR is great at Rust + SQL; mediocre at advisor-prompt prose.

**Mixed diffs go via PR.** A single commit that touches `crates/` and `.claude/lessons/` follows the PR flow — the code half wants review.

## Before the first commit of a new phase

Check the current branch. You must be on `phase-<N>` (or `phase-<N>a` for sub-phases), not `governance-v0`:

```
git branch --show-current
```

If the current branch is `governance-v0`, stop and surface the issue in `.claude/decision-queue.json` as a blocking question to the advisor. Do not branch yourself — branch creation is the advisor's responsibility during the phase-transition handoff. Phase 5 branches from `governance-v0` at the Phase 4→5 transition.

If the current branch is `phase-<N>`, proceed.

## During the phase

All ralph commits land on `phase-<N>`. No deviation. If you need to rebase or pick commits from elsewhere, write a decision-queue question first.

## At phase close

Open a PR from `phase-<N>` into `governance-v0`:

```
gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-<N> \
  --title "Phase <N> — <one-line goal>" \
  --body "$(cat <<'EOF'
## Summary
- <bullet>
- <bullet>

## Completion report
`.claude/PRPs/reports/phase-<N>-complete-report.md`

## Plan reference
`docs/research/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` §Phase <N>
EOF
)"
```

The `--repo barrie-cork/lemmy` flag is mandatory — `gh` defaults to upstream `LemmyNet/lemmy` on forks. See `gh-pr-fork-target.md`.

CodeRabbit auto-reviews PRs into `governance-v0` (see `.coderabbit.yaml`). Do not open the PR as a draft — CodeRabbit skips drafts.

## Do not

- Commit code (`crates/`, `migrations/`, `tests/`, dependency manifests) directly to `governance-v0` — that work goes through the PR flow above
- Open PRs against `main` — `main` is reserved for upstream rebases
- Squash the PR at merge — the task-per-commit history is load-bearing for retros
- Skip CodeRabbit on a code PR because "the diff is small"
- Open a PR for pure meta-work (`.claude/`, `docs/`, `scripts/`, infra) when CR review would be net-noise — direct-commit per the policy above
