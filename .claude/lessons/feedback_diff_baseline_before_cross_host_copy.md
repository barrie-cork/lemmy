---
name: Diff the target's baseline before copying code to a host that isn't a git checkout
description: When deploying a code change to a host that is NOT a git checkout (scp a built tree, copy files to a server, push to a box you can't `git log`), you cannot rely on git to tell you what's already there. Before copying, diff the TARGET's current baseline against the SOURCE's PRE-change baseline. If they don't match, the copy is a version-uplift (replacing a different lineage), not a same-lineage patch — and a partial copy will regress the target. The 2026-06-07 read-pheromone session deployed to the EliteDesk PMD assuming "recent copy of the laptop"; the baseline diff revealed a much older lineage with no hybrid search, turning a 2-file patch into a full src-tree uplift (built on-box, backed up first).
type: feedback
---

## TL;DR

Deploying a code change to a host that **isn't a git checkout** (scp'd tree, a copied
directory on a server, a box where you can't `git log`/`git diff` to see what's there)
means git can't tell you the target's state. Before you copy anything, **diff the
target's current baseline against the source's PRE-change baseline**:

- **Match** → the target is the same lineage; your minimal patch (copy the changed
  files) is safe.
- **Mismatch** → the target is a *different* lineage. A partial file-copy will splice
  your new files onto a foreign tree and regress it. The correct move is a deliberate
  *version uplift* (copy the whole coherent tree, build on-box, back up first), not a
  cherry-picked file copy.

The diff is the gate that tells you which of those two deploys you're actually doing.

## Why this matters (read-pheromone session, 2026-06-07)

The read-pheromone patch was implemented in the laptop's MCP source
(`C:/Users/barri/Developer/MCPs/project-memory-mcp`) and needed to also land on the
EliteDesk PMD. The EliteDesk MCP is **not a git checkout** — it's a deployed `dist/`
tree, reached over ssh. The working assumption (from the spec + general intuition) was
"the EliteDesk runs a recent copy of the laptop code; just copy the two changed source
files (or the rebuilt dist) over."

Diffing the EliteDesk's baseline against the laptop's pre-change baseline **before**
copying inverted that:

- The EliteDesk tree was a **much older lineage** — it predated the hybrid-search
  feature entirely. Its `search.ts` had a different shape; the two-file patch wouldn't
  have applied cleanly and, if force-copied, would have spliced pheromone code onto a
  tree missing the functions it assumed.
- A naive "copy the changed files" deploy would have produced a **broken or silently
  regressed** EliteDesk PMD — new columns referenced by code paths that didn't match the
  old handlers.

Because the baseline diff caught it first, the deploy became the correct operation: copy
the **entire** laptop `src/` tree (a coherent, current lineage), **back up the EliteDesk
tree first**, build on-box (`tsc`), and verify. A version uplift, done deliberately —
not a partial patch onto a drifted host.

## When to apply

The gate fires when **all** of these hold:

1. You're about to **copy code/build artifacts to a host** (scp, rsync, `Copy-Item` over
   a mount, "deploy the dist tree").
2. The target is **NOT a git checkout** you can interrogate with `git log`/`git diff`
   (or it is, but it's detached from the source's history — a vendored copy, a deployed
   build, a fork you don't pull).
3. You're tempted to do a **partial** copy ("just the changed files", "just the rebuilt
   dist") on the assumption the rest of the target matches the source.

Does NOT fire when:

- The target IS a git checkout sharing history with the source → use `git` (fetch +
  diff/merge) instead; this lesson is for the no-git-handle case.
- You're doing a **full, deliberate replace** of the whole tree anyway (back up + copy
  everything) → the partial-copy regression risk doesn't exist, though backing up first
  still applies.
- The target is throwaway / recreated from scratch on every deploy (no prior state to
  regress).

## How to apply

Before the copy, establish both baselines and diff them:

1. **Capture the SOURCE's pre-change baseline.** The git SHA (or a hash of the relevant
   files) the source was at *before* your change — i.e. what the target *would* match if
   it were the same lineage. (`git stash` your change or diff against `HEAD~` if already
   committed; or hash the specific files at the pre-change ref.)

2. **Capture the TARGET's current baseline** over the wire — without git, hash the same
   files. Use the quote-safe ssh helper so parens/paths don't get mangled:
   ```powershell
   # per-file size+mtime is a weak signal; content hash is the real check:
   .claude/tools/ssh-sql.ps1 -RemoteHost homeserver `
     -Command "cd /path/to/target && for f in src/db.ts src/tools/search.ts; do sha256sum \$f; done"
   ```
   (Or `md5sum`/`sha256sum` over the files; or `wc -l` + a spot-read of a signature
   function as a cheap first pass — but a hash is definitive.)

3. **Diff.** Source-pre-change hashes == target hashes → same lineage, minimal patch is
   safe. Any mismatch → different lineage; STOP the partial copy.

4. **On mismatch, switch to a deliberate uplift:** back up the target tree first
   (`tar czf /tmp/<name>-backup-<ts>.tar.gz <dir>`), copy the *whole* coherent source
   tree (not cherry-picked files), build on-box, and verify against the live system
   (`feedback_spec_deploy_facts_are_hypotheses.md` — confirm the service, port, counts
   after).

## Hard refusals

- **NEVER partial-copy changed files to a non-git-checkout host without diffing its
  baseline first.** "It's probably a recent copy" is a hypothesis; on a mismatch it
  splices your change onto a foreign tree and ships a silent regression.

- **NEVER overwrite a deployed tree without backing it up first** (`tar`/`Copy-Item` the
  target aside). The target isn't a git checkout — there's no `git reset` to recover it.
  Per `no-destructive-defaults.md` (investigate before overwriting) + the §5.6 catch-fire
  table's uncommitted-code preservation discipline.

- **NEVER trust file size/mtime alone as the "same lineage" signal.** Two trees can have
  same-sized files of different content. Hash the files (or diff them) — same-name ≠
  same-content, the same trap as `feedback_runbook_audit_drift_post_event_check.md`'s
  "identical file" claims.

## Cross-references

- `feedback_spec_deploy_facts_are_hypotheses.md` — the sibling from the SAME session: a
  spec's deploy facts (incl. "host A is a copy of host B") are hypotheses. This lesson is
  the "verify the copy-target's lineage" half; that one is the "verify the runtime facts"
  half.
- `feedback_runbook_audit_drift_post_event_check.md` — parent discipline; "identical
  file" / "same version" claims drift and must be diffed, not assumed.
- `feedback_verify_squash_merge_content_byte_level.md` — sibling: verify a merge/copy
  landed by byte-level content comparison, not by trusting the operation reported success.
- `.claude/tools/ssh-sql.ps1` — the quote-safe remote-command helper to capture the
  target baseline over ssh without Windows-OpenSSH quote-stripping.
- `.claude/PRPs/reports/session-retro-2026-06-07-pheromone-pmd-impl.md` — the EliteDesk
  "uplift not copy" surprise that motivated this lesson.
