---
description: Render the Brehon v1 roadmap (.claude/PRPs/v1-roadmap.json) as a styled HTML page and open it in the default browser. Refreshes the HTML on every invocation so it always reflects the current roadmap state.
argument-hint: (none)
disable-model-invocation: true
---

# /open-roadmap — render + open the v1 roadmap in browser

Single purpose: run the user-scope render script against the roadmap file in
the current repo, then open the resulting HTML in the default browser.

## Steps

1. **Locate the roadmap.** Default path: `.claude/PRPs/v1-roadmap.json`
   relative to CWD. If missing, STOP and surface the path — caller may be in
   the wrong CWD (e.g. a lane worktree where the file doesn't exist).

2. **Render via the script.** Run, capturing the output path from stdout:

   ```bash
   python C:/Users/barri/.claude/scripts/render-roadmap.py
   ```

   The script writes HTML to `%TEMP%/brehon-roadmap-view.html` and prints
   the absolute path on stdout. Exit codes: 0 success, 2 file missing, 3
   JSON parse error.

3. **Open in browser.** On Windows:

   ```bash
   start "" "<output-path-from-step-2>"
   ```

   On macOS use `open`, on Linux `xdg-open`. The current environment is
   Windows; default to `start`.

4. **Report back.** One line: `opened <output-path>` (so user knows where
   the file lives if they want to bookmark or re-open without re-rendering).

## What gets rendered

A single-page HTML view with these sections in order:

- **In flight now** — sub-phases with `status: in_flight` (both inside
  `lanes.*.sub_phases` and in `what_remains.in_flight_outside_roadmap`).
  Out-of-band lanes get an `out-of-band` badge; user-steered lanes get a
  `user-steered` badge.
- **Next recommended** — from `implementation_steering.next_logical_sub_phase`
  plus any `concurrent_lanes` list.
- **Lanes** — top-level summary table: lane key, status (colored badge),
  next pending sub-phase, evidence summary, PRD link.
- **High-priority unstarted** — from `what_remains.high_priority_unstarted`.
- **Meta lanes** — from `meta_lanes` block.
- **Raw JSON** — collapsed details element with the full file for
  inspection / debugging.

## When NOT to use this

- Editing the roadmap (use direct file edits + commit).
- Cutting a new lane from the roadmap (use `/roadmap-next`).
- Driving an in-flight sub-phase (use `/auto-phase` or `/auto-roadmap` per
  rules in `.claude/refs/auto-roadmap.md`).

This command is read-only — it never writes the roadmap file, never opens a
PR, never queues a Junior task.

## Failure handling

- **Script exits 2 (file missing)** → confirm CWD with `pwd` and `git
  branch --show-current`. The roadmap only exists on `governance-v0` in
  the canonical `brehon-fork` checkout; running from a lane worktree on a
  `phase-v1-*` branch may not see it.
- **Script exits 3 (JSON parse error)** → the roadmap file is malformed;
  surface the parser error from stderr verbatim. Do NOT attempt to
  auto-fix — JSON-fix-up is destructive and the user owns the schema.
- **`start`/`open` fails** → fall back to reporting the file path so the
  user can open it manually.
