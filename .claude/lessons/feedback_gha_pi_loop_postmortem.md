---
name: GitHub Actions facts pi/Claude must not re-derive (post-mortem 2026-05-06)
description: Six load-bearing facts about GitHub Actions env vars, secrets, push/PR contexts, comment retriggering, and CI gate construction. Paired with the corrected adr-compliance bypass pattern. Authored after pi spent multiple iterations chasing a non-existent default env var (GITHUB_EVENT_NUMBER) and self-authoring a memory note hardcoding the wrong shape as canonical
type: feedback
---

This lesson exists to interrupt a specific failure mode: an agent
debugging a failing GitHub Actions workflow without a reliable mental
model of what env vars exist on which trigger. On 2026-05-06 a pi
session spent multiple iterations on `adr-compliance.sh` chasing
`GITHUB_EVENT_NUMBER` (which does not exist), self-authoring a memory
note that hardcoded the wrong fallback as canonical, and re-firing the
workflow on each auto-commit until the loop was diagnosed externally.
Read this lesson before authoring or modifying any `.github/workflows/*.yml`
or `.github/scripts/*.sh`.

## The six facts

1. **`GITHUB_EVENT_NUMBER` does not exist** as a default GitHub Actions
   env var. The default-env-var list contains `GITHUB_EVENT_NAME`,
   `GITHUB_EVENT_PATH`, `GITHUB_REPOSITORY`, `GITHUB_REF`, `GITHUB_SHA`,
   `GITHUB_RUN_ID`, etc. — no `_NUMBER` variant. Searching for it harder
   will not help. Reference: GitHub docs `actions/learn-github-actions/variables#default-environment-variables`.

2. **`secrets.*` are unavailable on push triggers.** They resolve only
   for `pull_request` and a few other contexts. Using `${{ secrets.X }}`
   on a push step silently substitutes the empty string; the workflow
   continues but the value is gone. For numeric or non-secret values
   use a hardcoded literal in the workflow `env:` block (e.g.
   `OWNER_ID: '15565016'`), not a secret reference.

3. **`github.event.pull_request.number` is empty on push events.** Push
   event payloads have no `pull_request` object — by event-payload
   design — even when the push lands on a PR's head branch. To resolve
   a PR number on a push event, look it up via the API:

   ```bash
   gh pr list --head "${GITHUB_REF##*/}" --state open --json number -q '.[0].number // empty'
   ```

4. **PR comments do NOT re-trigger CI.** Posting an "acknowledge" comment
   on a PR does not cause a re-run of any workflow. Re-triggers happen
   on `push` (and `synchronize` for `pull_request`-typed workflows). If
   a bypass scheme depends on reading PR comments at scan time, force a
   fresh run by pushing — e.g. an empty commit:

   ```bash
   git commit --allow-empty -m "trigger: re-run after <reason>" && git push
   ```

5. **Gate downstream steps on the script's exit code, not findings
   presence.** When a workflow YAML uses `if:` to fail the job after a
   scanner step, gate it on the script's actual exit code (captured into
   `$GITHUB_OUTPUT` from `$?`), not on whether a findings file is
   non-empty. Otherwise the script's bypass / advisory paths are
   unreachable: the script exits 0 but the YAML still fails the job.

   Bad:
   ```yaml
   if: steps.scan.outputs.violations == 'true'
   ```

   Good:
   ```yaml
   if: steps.scan.outputs.scan_status != '0'
   ```

   Where `scan_status` was captured by the scanner step:
   ```yaml
   set +e
   bash .github/scripts/<scanner>.sh > /tmp/findings.md
   scan_status=$?
   set -e
   echo "scan_status=$scan_status" >> "$GITHUB_OUTPUT"
   ```

6. **The pi auto-commit-per-edit hook is a CI-iteration footgun.** Each
   speculative `Edit`/`Write` from pi triggers `auto(pi): update <file>`
   + push, which re-fires every push-trigger workflow. During CI debug,
   toggle it off with `/ci-debug-mode` (registered by `lemmy-hooks.ts`)
   before iterating, then back on when the fix is real. Without this,
   each iteration on a wrong premise gets a fresh CI failure to react
   to, and the loop accelerates rather than converges.

## Corrected `adr-compliance` bypass pattern

The cb7f33dfa + 5c521dc17 fix establishes the canonical shape. Any
future "improvement" to the bypass MUST preserve these two
properties:

- The script exits 0 on push / `workflow_dispatch` triggers (no PR
  context to attach an `acknowledge` comment to — advisory only).
- The workflow's "Fail the job" `if:` gates on `scan_status`, not on
  `violations`.

Reference commits:
- `.github/scripts/adr-compliance.sh` end-of-script — `cb7f33dfa`
- `.github/workflows/adr-compliance.yml` Fail-the-job step — `5c521dc17`

## Why this lesson is in `.claude/lessons/` rather than a pi-only path

Both harnesses (Claude Code with its memory-injection rule, pi with
PMD search wired by `start-pi.sh`) read this directory. Single source
of truth; no risk of one harness being patched and the other
re-deriving the same wrong facts. The pi-side
`.pi/agents/ci-debug.md` subagent points here as its first read; the
`.claude/agents/` ci-watcher / branch-manager rules already pull from
this directory.

## Related references

- `feedback_ci_silent_failure_pattern.md` — sibling lesson on CI
  steps that succeed despite missing payloads.
- `feedback_gha_action_input_no_bash_expansion.md` — sibling lesson
  on YAML interpolation gotcha.
- `.claude/memory/pi-advisory-bypass-pattern-20260506.md` — superseded
  by this lesson; kept as a one-line pointer for historical
  breadcrumb.
