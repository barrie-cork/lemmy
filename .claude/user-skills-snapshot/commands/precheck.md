Pre-flight check for Brehon Junior worker queueing. Runs 5 parallel checks against the EliteDesk + brehon-fork trunk; reports + halts on any failure.

Always uses `/srv/brehon-fork` and the brehon Junior daemon. For other repos, the queueing rules are different — this command doesn't apply.

## Why this command exists

Junior workers branch from the **committed HEAD of the trunk branch**. Uncommitted edits, unpushed commits, stale trunk, or a forbidden execution window will silently produce wrong results. Confirmed pattern after task #28 (Opus subverted brief, contaminated phase-v1-JM-d via commit `533d6b2`). Per CLAUDE.md "Pre-queue git pre-flight (mandatory)" + `feedback_check_git_before_junior_queue.md` (PMD #155).

## Procedure — 5 parallel checks, then halt-on-fail

Run these in a single message with multiple Bash tool calls:

### Check 1 — Tailscale + SSH up

```bash
tailscale status 2>&1 | head -3 && ssh -o ConnectTimeout=5 homeserver 'echo OK'
```

PASS: Tailscale shows `homeserver` peer active AND ssh returns `OK`.
FAIL: Tailscale stopped → `tailscale up` (may need elevated terminal). SSH timeout → check Tailscale state, or restart Tailscale service via tray.

### Check 2 — Daemon active + patch effective

```bash
ssh homeserver 'systemctl is-active junior@brehon-fork.service'
bash C:/Users/barri/Developer/homeserver/scripts/restore-junior-server-patches.sh --check
```

PASS: `active` AND `[ok] patches already applied`.
FAIL: daemon inactive → `sudo systemctl start junior@brehon-fork.service`. Patch drift → `bash C:/Users/barri/Developer/homeserver/scripts/restore-junior-server-patches.sh` to reapply.

The second command also confirms the four-role tiering patch is in source. The cron healthcheck (every 5 min) verifies the patch is *effective* by reading the newest role-task model field — if drift was detected, a Telegram alert would have already fired. So this check covers source-level drift; the cron covers runtime drift.

### Check 3 — Git state on trunk

```bash
ssh homeserver 'cd /srv/brehon-fork && git status -sb; git fetch origin 2>&1 | tail -3
branch=$(git branch --show-current)
upstream=$(git rev-parse --abbrev-ref --symbolic-full-name @{u} 2>/dev/null || echo "")
if [ -n "$upstream" ]; then
  echo "--- upstream: $upstream ---"
  echo "--- ahead ---"
  git log --oneline @{u}..HEAD 2>/dev/null | head -5 || echo "(none)"
  echo "--- behind ---"
  git log --oneline HEAD..@{u} 2>/dev/null | head -5 || echo "(none)"
else
  echo "--- branch is local-only (no upstream) ---"
  echo "--- last 3 commits on $branch ---"
  git log --oneline -3
fi'
```

PASS conditions:
- `git status -sb` shows only the branch line (or only the known runtime dirs, hidden via `.git/info/exclude`)
- **Branch has upstream** AND empty `--- ahead ---` and `--- behind ---` blocks (perfectly synced), OR
- **Branch has upstream** AND ahead with commits the user explicitly intends to ship (advisor confirms case-by-case)
- **Branch is local-only** (no remote tracking) AND advisor explicitly confirms this is intentional. Phase branches like `phase-v1-JM-d` are typically local-only by design — the BM creates them locally, and they get pushed only when ready for PR. Don't fail-stop on this; surface the fact and let the advisor decide.

FAIL conditions and fixes:
- **Unstaged mods or staged uncommitted changes:** `git add` + `git commit` on trunk and (if branch has upstream) `git push` BEFORE queueing. The worker won't see uncommitted edits.
- **Untracked tracked-class files** (anything other than `.junior/`, `.project-memory/`, `.serena/`): commit or stash. If they're persistent runtime state, add to `.git/info/exclude` (per CLAUDE.md "Untracked runtime dirs").
- **Branch has upstream + behind origin:** `git pull --ff-only` first. Worker basing off stale HEAD will diverge from what BM expects.
- **Branch has upstream + ahead with unpushed work intended for PR:** `git push origin <branch>` first. If smoke/diagnostic, switch to a throwaway branch instead (see "Smoke task workflow" below).

Surface the trunk branch + upstream status in the report so the advisor confirms both match intent (e.g. `phase-v1-JM-d` local-only is fine; `governance-v0` should always have upstream).

### Check 3b — Daemon-local trunk ref synced with origin (MANDATORY in multi-lane)

Pushed-to-origin (Check 3) is necessary but **NOT sufficient**. A Junior worker created via `mcp__junior-brehon__create_task` with `base_branch=<trunk>` branches from the **daemon's LOCAL** `<trunk>` ref in `/srv/brehon-fork`, NOT from `origin/<trunk>`. In multi-lane operation the daemon's single checkout sits on whatever lane's `phase-v1-*` branch most recently used it, and its local trunk ref goes stale silently — a worker that can't see a freshly-pushed brief escalates with NO DQ entry and exits `subtype:"success"` / `status: done`, so the advisor mistakes it for task success (phantom success).

Set `<trunk>` to the exact `base_branch` the upcoming `create_task` will use (`governance-v0` for most briefs/plans; `phase-v1-<lane>` for impl tasks that branch from the phase branch).

Detection:

```bash
ssh homeserver 'cd /srv/brehon-fork && git fetch origin <trunk> --quiet && [ "$(git rev-parse <trunk>)" = "$(git rev-parse origin/<trunk>)" ] && echo SYNC || echo STALE'
```

PASS: `SYNC` (daemon-local `<trunk>` == `origin/<trunk>`).

FAIL (`STALE`): fast-forward the daemon-local ref **without switching the checkout** — refspec-fetch updates the local branch ref while the checkout stays on the other lane's branch:

```bash
ssh homeserver 'cd /srv/brehon-fork && git fetch origin <trunk>:<trunk>'
```

Then VERIFY the refspec-fetch did not disturb the active lane:

```bash
ssh homeserver 'cd /srv/brehon-fork && git branch --show-current && git status --porcelain | head -5'
```

`git branch --show-current` MUST be unchanged from before the fetch, and the working tree MUST be clean.

**HARD REFUSAL:** NEVER `git checkout <trunk>` on the daemon to fix staleness. That switches the shared `/srv/brehon-fork` checkout away from the active lane's branch — a `.claude/rules/multi-lane-worktree.md` hard-refusal-class destructive cross-lane op. The refspec-fetch (`git fetch origin X:X`) is the ONLY correct lane-safe primitive.

Rationale: per `feedback_daemon_local_trunk_stale_multi_lane.md` — the multi-lane variant of `feedback_check_git_before_junior_queue.md`. Caused a phantom bm-cut on 2026-05-16 (task #273: brief pushed to `origin/governance-v0` but daemon-local `governance-v0` stale because the federation-inbound-a lane had the daemon checkout on its phase branch; worker escalated silently; ~15 min lost). MANDATORY whenever ANY other phase lane may be active — which is the **default assumption** now that multi-lane is the norm. When in doubt, run it: it's a cheap ssh + 2 `rev-parse`s.

### Check 4 — Forbidden window (UTC)

```bash
date -u +"%a %H:%M UTC"
```

PASS: current UTC time is OUTSIDE these windows:
- Daily 02:55–04:15 (NAS backup + web-archive govie-search)
- Sunday 01:55–02:35 (HSE crawl + junior-weekly-review)
- Sunday 03:55–04:30 (restore-drill.timer)
- Wednesday 03:55–04:15 (subset of daily — no extra constraint)

FAIL: in a forbidden window. Report the next safe minute (end of current window) and tell the advisor to defer queueing until then. No DQ entry needed for routine deferrals — this is mechanical self-defer per advisor-orchestrator.md.

Brehon execution windows: 16:00–02:30 UTC (primary, evening/overnight) or 04:30–14:59 UTC (secondary, post-crawl).

### Check 5 — Memory headroom

```bash
ssh homeserver 'free -h | head -3 && echo --- && systemctl show -p MemoryAvailable junior@brehon-fork.service --no-pager 2>&1 | head -1'
```

PASS: `available` field in `free -h` is ≥ 3 GB AND no other heavy workload showing in `top`.

FAIL conditions:
- **`available < 3 GB` and the next task is cargo-class** (Brehon `[role:impl-task]` whose §15 DoD names `cargo check`, `cargo clippy --workspace`, or `cargo test --workspace`): offer the user `bash C:/Users/barri/Developer/homeserver/scripts/web-archive-pause.sh` to free ~2.2 GB. State trade-off: search UI returns 502 until unpause; pause across 02:00 UTC misses HSE/Govie crawl.
- **`available < 3 GB` and next task is non-cargo** (doc edits, brief writes, retros, BM verbs, planning): pass advisory note but don't refuse — those workloads peak well under 1 GB.

Per `feedback_resource_budget_pre_queue.md` (PMD #105) — sum Docker ceilings + worker peak BEFORE queueing cargo-heavy tasks. Skipping cost a power cycle 2026-04-27.

## Synthesis

After all 5 checks land, render this format:

```
=== Brehon Junior pre-queue check ===
Time:                <UTC time>  [OK | DEFER until HH:MM]
Tailscale + SSH:     [PASS | FAIL: <reason>]
Daemon + patch:      [PASS | FAIL: <reason + fix>]
Trunk state:         [PASS on phase-v1-JM-X (synced)
                    | FAIL: <reason + fix>]
Forbidden window:    [OK | IN WINDOW (defer to HH:MM UTC)]
Memory headroom:     [<X.X GB available> | LOW (web-archive-pause offered)]

VERDICT: [READY TO QUEUE | NOT READY — fix above before queueing]
```

If VERDICT is READY:
- Note the trunk branch + commit so advisor can verify it matches intent.
- Done. Advisor proceeds with `mcp__junior-brehon__create_task`.

If VERDICT is NOT READY:
- Halt. Do NOT proceed with queueing.
- List the specific fix commands inline, ready to paste.
- Advisor decides whether to apply the fix or wait.

## Smoke task workflow (special case)

For smoke / diagnostic / no-op test tasks where the worker shouldn't write real commits to the phase branch (per `feedback_check_git_before_junior_queue.md` — Opus may interpret a smoke prompt as real work):

1. Run `/precheck` first to confirm trunk is healthy.
2. After PASS, switch to a throwaway branch:
   ```bash
   ssh homeserver 'cd /srv/brehon-fork && git checkout -b smoke-$(date +%Y%m%d-%H%M)'
   ```
3. Queue the smoke task. Worker contamination stays on the smoke branch.
4. After verifying, clean up:
   ```bash
   ssh homeserver 'cd /srv/brehon-fork && git checkout phase-v1-JM-X && \
     git branch -D smoke-… && \
     git worktree remove /srv/brehon-fork/.junior/worktrees/job-N --force && \
     git branch -D junior/role-...-N'
   ```

Tell the advisor when smoke mode applies — it's not the default. Real impl/BM/planning work runs on the phase branch, not a smoke branch.

## What this command does NOT do

- **Doesn't queue anything.** Read-only check; advisor still calls `mcp__junior-brehon__create_task` after PASS.
- **Doesn't auto-fix.** Per user preference 2026-04-28: report failures and the fix, but never auto-apply destructive operations (commit, push, fetch, branch switch). Advisor confirms each fix.
- **Doesn't replace the cron healthcheck.** That runs every 5 min and detects patch drift autonomously. This command is a deliberate user-triggered moment-of-truth check before a specific task.
- **Doesn't apply to non-Brehon repos.** Other Junior daemons (agent-grey, food-producer, etc) have different queueing rules — see CLAUDE.md.
