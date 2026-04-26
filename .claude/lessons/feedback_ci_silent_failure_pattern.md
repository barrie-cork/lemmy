---
name: CI silent-failure pattern (set -e + cd + output guard)
description: GitHub Actions run steps default to no-set-e; silent subprocess failures poison downstream reports. Guard with set -euo pipefail + explicit cd + post-step size check.
type: feedback
originSessionId: 50ffed48-0dd6-4010-8ffb-d915ca13314c
---
GitHub Actions `run: |` composite-step blocks default to **no `set -e`**. A silent failure in an embedded heredoc (Python, jq, curl, …) whose stdout redirects to a file produces an empty file, and the step still exits 0.

**Why:** on `plan-drift.yml` (Brehon fork), a `python3 - <<'PY' > /tmp/code.txt` heredoc failed silently on CI (probably a relative-path resolution issue), yielding `/tmp/code.txt = ""` → `wc -l = 0` → the committed drift report said `code_route_count: 0` when the actual router wires 8 routes. Fixed 2026-04-18 at commit `e30f93c8b`.

**How to apply:** every GitHub Actions `run: |` step that:
- runs an embedded script (heredoc, inline Python, awk) AND
- redirects stdout to a file OR captures output into `$GITHUB_OUTPUT`

should start with:

```yaml
run: |
  set -euo pipefail
  cd "$GITHUB_WORKSPACE"   # if the script uses any relative path
  …
  # after the critical output file is produced:
  test -s /tmp/out.txt || { echo "::error::<step> produced no output"; exit 1; }
```

The three guards cover three distinct classes:
1. `set -euo pipefail` — catches Python/jq/curl failures that silently produce empty stdout.
2. Explicit `cd "$GITHUB_WORKSPACE"` — defends against runner pwd drift (some actions change cwd; `actions/checkout` sets it to workspace but that contract isn't guaranteed across every step order).
3. Post-step `test -s` — catches the case where the script ran cleanly but produced an empty file because its input shape changed (e.g. a regex that no longer matches). This is the **domain-knowledge guard** — only you know the file should be non-empty.

Same trap exists in any CI system whose step runner doesn't default to `errexit` (GitLab CI, Jenkins shell blocks, CircleCI `run:` steps). Audit any CI step that commits a derived artifact to the repo — a silent-green step with a wrong artifact is the worst-case: it looks trustworthy and ships false data downstream.
