---
name: Sibling-pattern .gitignore audit at feature-author time
description: When adding a feature that creates a runtime artifact directory or file (queue, drain staging, scratch, log dir), grep the sibling `<feature-base-name>` in `.gitignore` BEFORE the first commit that introduces the artifact. If the sibling is already ignored, codify the new path; if not, codify both. Twice-occurring miss across consecutive sessions.
type: feedback
---

When a feature ships a new file or directory that will hold runtime
artifacts (queues, staging dirs, drained-rows ledgers, scratch logs,
per-task caches), check the existing `.gitignore` for sibling
patterns of the feature's base name BEFORE the first commit. If the
sibling is already ignored, codify the new artifact at the same
time; if no sibling is ignored, codify both.

**Why:** Recurred across two consecutive sessions on the same
codebase:

- **2026-05-24 T4a** — `role-signal-utilisation.sh` shipped with
  `.claude/role-signal-queue.jsonl` in `.gitignore` (the queue
  file). The drain script's *staging directory*
  `.claude/role-signal-drain/` was NOT ignored. Lifted to a WATCH
  in MEMORY.md.
- **2026-05-25 (this lesson's trigger)** — the same drain dir was
  ABOUT to be committed accidentally. Caught at retro-time, not
  at commit-time. Recurrence threshold = 2.

Both misses share the same shape: a feature ships its primary
artifact path under gitignore, but adjacent artifacts created by
the same feature (drain staging, ledger files, scratch caches)
are missed because the author thinks one-artifact-per-feature.

**How to apply:**

At the first commit that introduces a runtime-artifact
file/directory, run:

```bash
# Pull the feature's base name from the file you're adding.
FEATURE_BASE="role-signal"  # e.g. for .claude/role-signal-drain/

# 1. Find existing sibling entries in .gitignore
grep -n "$FEATURE_BASE" .gitignore

# 2. If any siblings exist, your new path MUST sit next to them in
#    .gitignore (same commit). If no siblings exist, add both the
#    new path AND any other paths the feature creates at runtime.
```

The commit-body for any commit adding a new runtime-artifact path
should explicitly state: `also gitignored <path>` — that one line
is enough to make the next reviewer / next session catch the
sibling miss in the diff.

**Triggering signatures (when this lesson fires):**

- A commit adds a new `scripts/<feature>/`,
  `.claude/<feature>-<thing>/`, `target/<scratch>/`, or any
  hook/script that opens a writable file path.
- The path lives in a directory that already has sibling gitignored
  entries (the feature's "home" already has runtime artifacts).
- The commit-body doesn't mention the gitignore status of the new
  path.

**Symptom to recognise:**

- `git status` after running a feature for the first time shows a
  new directory/file under `?? ` that obviously belongs in
  gitignore.
- A retro mentions "almost committed runtime artifacts" or "had to
  gitignore X separately" — that's a deferred miss; the right
  catch is at the original commit.
- The MEMORY.md WATCH says "promote if recurs" and this happens
  again — that's THIS lesson firing.

**Generalises to:** any project hygiene check that's mechanical and
repeatable but consistently slips through manual review. Other
candidates of the same shape:

- File mode (`+x`) on new shell scripts
- `*.bak` / `*~` / `*.swp` near edited files
- Build-artifact dirs near new build steps (`dist/`, `target/`)

For each: grep the `.gitignore` for the feature's base name; if
any sibling exists, codify the new path in the same commit.

**Related lessons:**

- `feedback_commit_aggressively_in_shared_repos.md` — companion
  principle (commit when state changes); this lesson narrows it
  with "and grep gitignore first."
- `feedback_settings_local_json_worktree_bootstrap.md` — sibling
  miss class for per-worktree state.
- Session retro `.claude/PRPs/reports/session-retro-2026-05-25-role-signal-hook-gate-fix.md`.
