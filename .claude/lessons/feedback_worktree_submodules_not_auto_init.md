---
name: git worktree does not auto-init submodules
description: git worktree add creates the worktree without running git submodule update --init; on Lemmy fork this breaks cargo check via empty crates/email/translations/backend/
type: feedback
originSessionId: 9a8963c5-7adc-4575-831f-d7a506428c75
---
`git worktree add <path> <branch>` checks out the branch's tree into
the new worktree but does **NOT** run `git submodule update --init`
in the new worktree. Submodule directories appear empty.

**Why:** The Lemmy fork has a submodule at `crates/email/translations`
pointing at `https://github.com/LemmyNet/lemmy-translations.git`. The
`lemmy_email` crate's `build.rs` calls `read_dir("translations/backend/")`
during the build. An empty directory yields
`Os { code: 3, kind: NotFound, message: "The system cannot find the
path specified." }` — the error is "directory exists but is empty for
the glob we expected", not "file missing". `cargo check --workspace`
(or any command that touches `lemmy_email`) fails with exit 101.

**How to apply:** Every new worktree needs this sequence before any
cargo command:

```bash
cd <new-worktree-path>
git submodule update --init --recursive
ls crates/email/translations/backend/ | head -3   # expect *.json locale files
```

Observed 2026-04-19 during Phase 6 pre-flight — probes 2 and 3 failed
with the `lemmy_email` NotFound error until the submodule was
initialised in `../brehon-fork-advisor-phase6`. Every Phase 6 agent
brief (`.claude/PRPs/phase-6-runlog/briefs/agent-<A..G>.md`) now
explicitly includes the submodule-init step as the first worktree
command.
