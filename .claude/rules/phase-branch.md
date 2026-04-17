# Phase branch + PR flow (Phase 5 onwards)

From Phase 5 onward, every Brehon phase runs on its own `phase-<N>` branch and closes via a PR into `governance-v0`. Phases 1–4 were grandfathered onto direct `governance-v0` commits and are not retroactively PR'd.

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

- Commit directly to `governance-v0` from Phase 5 onwards
- Open PRs against `main` — `main` is reserved for upstream rebases
- Squash the PR at merge — the task-per-commit history is load-bearing for retros
- Skip CodeRabbit because "the diff is small"
