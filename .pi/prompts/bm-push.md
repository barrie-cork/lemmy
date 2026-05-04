---
description: |
  BM — push the current phase or plan branch to origin (auto, no prompt)
argument-hint: |
  [--force-with-lease] (rare; confirms)
disable-model-invocation: true
---

# /bm-push — push current branch to origin

**Input**: $ARGUMENTS

**Dispatcher → `branch-manager` subagent.** This command delegates
execution to the `branch-manager` subagent, which runs in its own
isolated context window. The subagent reads the operational script
below and follows it step by step.

Invoke:

> Use the `branch-manager` subagent to run `bm-push`. Arguments:
> $ARGUMENTS. Follow the phases in `.claude/commands/bm/bm-push.md`.
> Standard push is auto; if `--force-with-lease` is in arguments, ASK
> the user via AskUserQuestion before pushing. Return a "Pushed" summary
> followed by a "Next suggested" line.

The parent (impl) session should call `Agent(subagent_type="branch-manager", model="haiku", prompt=<the above>)`. Mechanical — one `git push -u`; `--force-with-lease` confirm gate is literal-string match, not judgment.

---

## Operational script (for the subagent)

The BM agent pushes the current branch to `origin` with upstream
tracking. Auto if branch matches `phase-*` or `plan/*`. Per
`.claude/rules/branch-manager.md`, push is in the auto-class because
it doesn't change visible state on its own (no PR yet). The
subsequent `/bm-pr` is also auto.

**Reads:** `.claude/rules/branch-manager.md`.

---

## Phase 1 — Verify branch is push-eligible

```bash
git branch --show-current
```

**Decision tree:**

| Branch | Action |
|---|---|
| `governance-v0` | **STOP**: "Cannot push trunk via BM. Trunk is owned by upstream-rebase routine." |
| `main` | **STOP**: "Cannot push main via BM. Main tracks LemmyNet/lemmy upstream." |
| Matches `phase-v*-*-*` | PROCEED auto |
| Matches `plan/v*-*-*` | PROCEED auto |
| Matches `chore/*` | PROCEED auto |
| Anything else | **STOP**: "Branch name doesn't match BM-managed pattern. Confirm intent." |

---

## Phase 2 — Verify there's something to push

```bash
git status --short
git log origin/$(git branch --show-current)..HEAD --oneline 2>/dev/null
# If branch has no remote yet:
git log governance-v0..HEAD --oneline
```

| State | Action |
|---|---|
| Working tree dirty (uncommitted changes) | **STOP**: "Uncommitted changes — impl session must commit first." |
| No new commits to push | **STOP**: "Nothing to push (already in sync with remote)." |
| New commits exist | PROCEED |

---

## Phase 3 — Push

### Standard push (auto)

```bash
git push -u origin HEAD
```

If push succeeds: log to runlog and report.

### `--force-with-lease` push (CONFIRMS)

If `$ARGUMENTS` contains `--force-with-lease`:

**ASK the user first** — force-push is a manual-class action per the
autonomy table:

```markdown
**Confirm force-push?**

- **Branch:** {current}
- **Reason given:** {derive from recent commits or ask}
- **Risk:** if a PR is open and CR has posted, force-push moves the
  HEAD ref and the discussion-anchor commits may become orphaned. CR
  comments still attach to the original SHAs but the diff view changes.

Reply with `confirm` to proceed, or any other input to abort.
```

If confirmed:

```bash
git push --force-with-lease origin HEAD
```

If the lease check fails (someone else pushed in between):
**STOP**, do NOT retry with `--force` (which would clobber). Report
the lease failure and ask the user.

---

## Phase 4 — Failure handling

| Failure | Action |
|---|---|
| `! [rejected] ... non-fast-forward` | Report; suggest `git fetch origin && git log HEAD..origin/{branch}` to see what's behind |
| `! [remote rejected]` from CI hook | Report the hook output verbatim; do not retry |
| Network error | Retry once; if still failing, report and stop |
| Auth error | Report; tell user to check `gh auth status` |

Never retry-loop. One retry on transient network only. All other
failures surface to user.

---

## Phase 5 — Append to runlog

```markdown
## bm: push — {ISO timestamp}
- **branch:** {current}
- **commits pushed:** {N} ({first-sha}..{last-sha})
- **remote ref:** origin/{current}
- **next:** /bm-pr (if no PR yet) or /bm-poll-cr {N} (if PR exists and CR re-runs on push)
```

---

## Phase 6 — Output

```markdown
## /bm-push complete

**Branch:** {current}
**Pushed:** {N} commit(s) to origin/{current}
**Remote ref:** origin/{current} @ {short-sha}
**Upstream tracking:** set

### If a PR already exists for this branch
{detect via `gh pr view --repo barrie-cork/lemmy --json number 2>/dev/null`}
- CodeRabbit will re-review on push. Run `/bm-poll-cr {N}` in 5–10 min.

### If no PR yet
- Suggested next: `/bm-pr`
```

---

## Refusal cases

- On `governance-v0` or `main` → STOP.
- Working tree dirty → STOP.
- Nothing to push → STOP (informational, not an error).
- `--force-with-lease` requested → ASK user, do not auto.

---

## See also

- `.claude/rules/branch-manager.md` — autonomy bounds
- `.claude/commands/bm/bm-pr.md` — open PR after first push
- `.claude/commands/bm/bm-poll-cr.md` — pick up CR's re-review after push
