# feedback: daemon refspec filter excludes meta-phase branches (until 2026-05-20 fix)

## TL;DR

The Junior daemon's `/srv/brehon-fork/.git/config` had a `phase-v1-*` fetch refspec filter that excluded meta-phase branches like `phase-brehon-conformance-audit`. Fixed 2026-05-20 by adding a broader `+refs/heads/phase-*:refs/remotes/origin/phase-*` entry. **Status: PATCHED (live daemon config edit).** A tracked-patch via `homeserver/scripts/restore-junior-shims.ps1` is still TODO so the fix survives shim-restore cycles.

## Why this mattered

The daemon's filter was:

```
[remote "origin"]
    url = https://github.com/barrie-cork/lemmy.git
    fetch = +refs/heads/governance-v0:refs/remotes/origin/governance-v0
    fetch = +refs/heads/phase-v1-*:refs/remotes/origin/phase-v1-*
```

`phase-brehon-conformance-audit` did not match `phase-v1-*`. So `git fetch origin` (the daemon's standard fetch) skipped creating `refs/remotes/origin/phase-brehon-conformance-audit` even though origin had the ref. Symptoms:

- `git log origin/phase-brehon-conformance-audit` → "fatal: ambiguous argument" / "unknown revision".
- Workaround required: `git fetch origin <branch>:refs/remotes/origin/<branch>` (explicit ref-creation per fetch).

This blocked the standard advisor-side `git fetch origin` + ff-daemon-trunk routine for meta-phase branches.

## When to apply

Whenever a phase branch name does NOT match `phase-v1-*` — e.g. meta-tooling sub-phases like:

- `phase-brehon-conformance-audit`
- `phase-brehon-harness-*` (future)
- `phase-v2-*` (future major version)

V1 sub-phase names (`phase-v1-SL-c-2`, `phase-v1-federation-inbound-b`, `phase-v1-ship-1`, etc.) continued to work under the old narrow refspec.

## How the fix applies

2026-05-20 fix executed via:

```bash
ssh homeserver "cd /srv/brehon-fork && git config --add remote.origin.fetch '+refs/heads/phase-*:refs/remotes/origin/phase-*'"
```

Result (`.git/config` `[remote "origin"]` block now):

```
fetch = +refs/heads/governance-v0:refs/remotes/origin/governance-v0
fetch = +refs/heads/phase-v1-*:refs/remotes/origin/phase-v1-*
fetch = +refs/heads/phase-*:refs/remotes/origin/phase-*
```

The narrow `phase-v1-*` line is now redundant but harmless. Subsequent `git fetch origin` calls now create `refs/remotes/origin/phase-<any>` automatically.

## Pending: tracked-patch via Junior-shim restore

The fix is a live `.git/config` edit. If `homeserver/scripts/restore-junior-shims.ps1` (or equivalent reset path per `project_junior_shim_patches_untracked.md`) ever re-creates `/srv/brehon-fork/.git/config` from a template, the broader refspec will be reverted.

**TODO** (Task 13 retro or sooner): track the broadened refspec via the shim-patch manifest so it survives restore cycles. Pattern reference: `project_junior_four_role_tiering_patch.md` shows the env-based ANTHROPIC_MODEL override pattern; the refspec is a similar "config-side persistent patch".

## Cross-references

- `.claude/rules/multi-lane-worktree.md` §"Daemon side (EliteDesk)".
- `feedback_junior_292_stale_base_recover_recipe.md` — adjacent recovery (worker stale-base recover via update-ref).
- `feedback_daemon_local_trunk_stale_multi_lane.md` — daemon-local ff discipline.
- `project_junior_shim_patches_untracked.md` — TODO target for tracked-patch.
- Session retro: `.claude/PRPs/reports/session-retro-2026-05-20-brehon-conformance-audit-bootstrap.md` §2.7 + §4.1.
