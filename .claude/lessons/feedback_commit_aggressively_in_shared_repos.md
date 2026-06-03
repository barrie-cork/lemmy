---
name: Commit aggressively in shared repos
description: Doc-only sessions on governance-v0 must commit before closing — OQ resolutions, design-doc edits, and research files are append-only audit artefacts that can be clobbered by a concurrent session if left uncommitted.
type: feedback
---

In the `brehon-fork` repo, `governance-v0` is a shared trunk. Multiple advisor sessions and tool calls can write to the same files. Any uncommitted edit — even a doc-only OQ resolution — is invisible to other sessions and can be clobbered by a concurrent write or a `git checkout` in any other worktree touching the same file.

**Why:** Session 2026-06-01 made substantive edits to `99-decisions-and-open-questions.md` and `v2-messaging-rtc.prd.md` (three OQ resolutions — OQ-V2-10, OQ-009, OQ-V2-09 lean) and copied a new research file, but produced zero commits. The retro surfaced the gap; the commit only happened at retro time. Had a second session run between the edits and the retro, the working-tree edits could have been lost.

**How to apply:**

- After every OQ resolution, ADR update, or design-doc edit on `governance-v0`, commit immediately — do not batch until the retro.
- Commit subject pattern for doc-only sessions: `docs(<scope>): <what changed>` or `chore(decision-queue): <what changed>`.
- OQ resolutions are explicitly append-only audit artefacts per `99-decisions-and-open-questions.md` — treat them with the same immediacy as DQ commits.
- Research files copied to `docs/research/` should be committed in the same commit as the OQ resolution they support.
- At session close, always run `git status --short` before writing the retro. If there are modified tracked files, commit them first.

**Applies to:** advisor sessions on `governance-v0`; any session making doc-only edits without a cargo gate (no validation step = easy to skip the commit step).
