---
name: Cohort [P] shared .git/index.lock contention on single-.git daemon
description: Feedback rule — parallel [P] cohort workers on a single-.git EliteDesk daemon all contend on the same .git/index.lock (not in any FILES YAML), causing D-state git cascades at cohort size >=3; auto-degrade cohorts of >=3 to serial and SSH-tar uncommitted worktrees before cancel
type: feedback
---
# Cohort `[P]` shared `.git/index.lock` contention

**Source:** session retro 2026-05-25 — v1-RT-r3 cohort-2 cascade (#467/#468/#469)

## What happened

Three parallel `[P]` impl-tasks (Tasks 1/2/3) were dispatched simultaneously to the
EliteDesk daemon. All three edited genuinely disjoint Rust files (the YAML overlap check
passed). But every `git add` / `git commit` inside any worker worktree acquires the
SAME `.git/index.lock` because daemon-side worktrees all share one parent `.git/` tree.
With three Sonnet workers running rustc + git simultaneously:

- `#467`: Python `.git/index.lock` removal trick → API retry loop (attempt 8/10)
- `#468`: DNS-resolution timeout (779s) on git push retry → retro-check hook blocking commit
- `#469`: Six `git` processes in D state (uninterruptible disk I/O) competing with parallel rustc

After ~140 min wall-clock all three were cancelled. The daemon's cancel handler reaped
worktree FS state AND `.git/worktrees/<name>/` admin metadata immediately, forfeiting all
three workers' uncommitted code. Zero artifact preservation.

## Root cause

The `[P]` cohort marker + YAML overlap check (`creates:` + `modifies:` arrays) verify
**file-level disjointness** but cannot detect **ambient shared resources**. The
`.git/index.lock` is used by every worker implicitly — it is in no task's FILES YAML.
Three concurrent lock-acquirers on a single daemon `.git/` produce exponentially escalating
contention that degrades to D-state clusters.

Cohort size ≤2 is low-probability for cascade (one lock-acquire blocks, the other proceeds
after a short wait). Cohort size ≥3 produces the cascade empirically.

## Rule (codified in `advisor-orchestrator.md` §4.1 step 5a)

On a single-`.git/` daemon (all EliteDesk daemon worktrees share one `.git/`):
**auto-degrade any `[P]` cohort of size ≥3 to serial**.

This is NOT about file overlap — it is about the hidden ambient lock resource. The YAML
check cannot help; the cap is the only defence.

## Cancel discipline (codified in `advisor-orchestrator.md` §5.6)

Before cancelling any Junior task whose log shows uncommitted code:

```bash
ssh homeserver "tar czf /tmp/job-<id>-recovery-$(date +%s).tar.gz \
  -C /srv/brehon-fork/.junior/worktrees job-<id>" && echo preserved
```

Cancel handler reaps the worktree immediately and completely. The tar step is lossless,
<30s per worker. Without it, uncommitted code is unrecoverable.

## Daemon feature request (tracked)

A `cancel --preserve-worktree` flag on the Junior daemon would eliminate the need for
the manual SSH tar step. Track in `homeserver/scripts/restore-junior-server-patches.sh`
or as an upstream feature request in the Junior MCP repo. Until shipped, the SSH pre-cancel
tar is the mitigation.

## Not applicable to

- Mode A lane worktrees on the laptop (each has its own `.git/`; no shared lock)
- Cohorts of size ≤2 (low-probability; empirically safe)
- GitHub Actions runners (each job runs in an isolated checkout)

## See also

- `advisor-orchestrator.md` §4.1 step 5a (shared-`.git/index.lock` hazard check)
- `advisor-orchestrator.md` §5.6 (pre-cancel SSH inventory row)
- session retro: `.claude/PRPs/reports/session-retro-2026-05-25-rt-r3-cohort-2-git-index-cascade.md`
