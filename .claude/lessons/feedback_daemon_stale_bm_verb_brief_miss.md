# BM-verb tasks miss their brief when daemon-local governance-v0 lags origin

**Rule:** Before dispatching any Junior task — impl-task OR bm-task — verify the daemon-local ref
for the base branch is current with `origin/`. BM-verb briefs live on `governance-v0`; if the
daemon's local `governance-v0` lags origin, the bm-task worker branches from a stale tip and
never sees the brief, then either halts or declares false success on a no-op.

**Why:** impl-task briefs are on the phase branch (stale-ref fires there too — see
`feedback_daemon_local_trunk_stale_multi_lane.md`). BM-verb briefs (bm-pr, bm-merge,
bm-poll-cr, bm-triage, fix-in-pr) are on `governance-v0`. Same mechanism, different branch:
daemon-local `governance-v0` drifts whenever the advisor pushes briefs from the laptop but the
daemon hasn't fetched yet.

Confirmed: v1-federation-inbound-d 2026-05-22. Junior #415 (fix-in-pr) and #416 (bm-merge) both
declared false success — daemon-local `governance-v0` lagged origin for both. Two BM-verb types,
same session, same cause. 3rd+ class recurrence across the project.

## How to apply

**Pre-dispatch ref-currency check (mandatory, impl AND bm-verb tasks):**

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin && \
  git log --oneline origin/governance-v0 | head -3"
```

The brief commit must appear in the output before you create the task. If it doesn't, wait and
re-check — the push may not have landed or the daemon fetch hasn't run. Never create the task
until the brief SHA is visible on the daemon-local ref.

For impl-task dispatch, the same probe targets the phase branch:

```bash
ssh homeserver "cd /srv/brehon-fork && git fetch origin && \
  git log --oneline origin/phase-v1-<phase> | head -3"
```

**Recovery when the miss has already fired:**

1. Confirm the brief exists on origin: `git show origin/governance-v0:.claude/PRPs/briefs/<brief>.md`
2. FF the daemon-local ref (lane-safe, no HEAD switch):
   `ssh homeserver "cd /srv/brehon-fork && git fetch origin governance-v0:governance-v0"`
3. Cancel the false-success task if still running; re-dispatch after ref is current.
4. If the BM-verb action (e.g. `gh pr merge`) was already performed incorrectly by the
   no-brief worker, apply the correct action directly from the lane worktree as advisor inline
   execution: `gh pr merge <N> --repo barrie-cork/lemmy --merge` (or the appropriate verb).
   Advisor inline execution is the reliable fallback — does not depend on daemon ref currency.

## Structural fix (unshipped)

Daemon-side: add `git fetch origin <base_branch>:<base_branch>` before `git worktree add`.
This makes ref currency automatic rather than relying on a pre-dispatch probe. DQ #338 tracks
the wrong-ref class broadly; this BM-verb variant is now documented here.

## Relates to

- `feedback_daemon_local_trunk_stale_multi_lane.md` — impl-task variant (phase branch base)
- `feedback_cross_lane_daemon_ref_contamination.md` — multi-lane contamination of daemon-local refs
- `feedback_bm_false_success_advisor_post_condition_catch.md` — BM false-success; post-condition
  verify is the independent catch when the brief-miss leads to a no-op self-report
