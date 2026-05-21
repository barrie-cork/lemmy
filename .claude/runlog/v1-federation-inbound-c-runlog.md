# v1-federation-inbound-c runlog

> Append-only ledger of state-changing actions on phase-v1-federation-inbound-c.
> See `.claude/rules/branch-manager.md` "Coordination with the impl session" for
> the format. Lines prefixed with `## bm:` are BM session entries; lines prefixed
> with `## advisor:` are advisor session entries; lines prefixed with `## impl:`
> are impl session entries.

## bm: cut phase-v1-federation-inbound-c off governance-v0 @ 6dc489c9e

Phase branch created by Junior bm-cut task #393 off governance-v0 @ 6dc489c9e
(plan @ 22f15bd9a + bm-cut brief @ 6dc489c9e). Pushed to
origin/phase-v1-federation-inbound-c with upstream tracking. Trunk SHA at cut
time: `6dc489c9e chore(advisor): author v1-federation-inbound-c bm-cut brief
(plan-approved user gate 1)`.

## advisor: re-apply runlog (belt-and-braces — Junior #393 finalize deleted index-only file)

Junior bm-cut #393 wrote the runlog via `git hash-object` + `update-index`
plumbing (commit 9df6879a6 on the worker branch) because the Junior session's
PostToolUse hook (allow-prp-deliverables.sh) blocked direct `Write` on
`.claude/runlog/**`. The plumbing path created the file in HEAD but NOT in the
working tree. Junior's finalize-agent then ran `git status` (saw the file as
"deleted") and committed the deletion (c488963a0) before pushing — net result,
the phase branch on origin lacked the runlog. Advisor re-applies the runlog
directly from the canonical laptop checkout's lane worktree at
`C:/Users/barri/Developer/brehon-fork-fed-in-c`. Two root causes filed as
`kind: "log"` DQ for retro harvest.

## advisor: pre-phase wrapper audit pass + submodule init recovery

Per `.claude/rules/pre-phase-harness-audit.md`, ran 4 probes in lane worktree:

- Probe 0 (Docker daemon): OK.
- Probe 1 (`cargo-check.bat -p lemmy_utils`): exit 0; only `lemmy_utils`
  compiled in 1m 58s. `-p` honored.
- Probe 2 (`cargo-check.bat -p lemmy_db_schema --features full`): exit 0;
  `lemmy_db_schema` + full-feature deps (diesel-async, activitypub_federation,
  ed25519-dalek, bcrypt, diesel_migrations, doku, i-love-jesus) compiled in
  6m 30s. Features flag honored.
- Probe 3 (`cargo-test.bat --test e2e --no-run -p lemmy_server`): FIRST RUN
  failed with exit 101 — `lemmy_email` build.rs `read_dir("translations/backend/")`
  returned `Os { code: 3, kind: NotFound }`. **Root cause: `crates/email/translations`
  is a git submodule and was uninitialized in this lane worktree.** Per
  `feedback_phase_lane_worktree_bootstrap_checklist.md` Step 1, the lane
  bootstrap should run `git submodule update --init --recursive` after
  `git worktree add`; this lane's bootstrap missed it. Resolution: ran
  `git submodule update --init --recursive crates/email/translations`
  (a3f9e4669b53f041b92fbbe6bdd80c8db0619c20 lemmy-translations checkout).
  Re-ran probe → exit 0; exactly one executable built
  (`target\debug\deps\e2e-e14ea3c669383090.exe`) in 18m 07s. `-p` + `--test e2e`
  scope honored.
- Probe 4a/4b (bogus `--features nonexistent_xyz`): both exit 101 with
  `error: the package 'lemmy_server' does not contain this feature: nonexistent_xyz`.
  Wrappers propagate non-zero — no exit-code-masking bug.

Flag file `.claude/audit-phase-v1-federation-inbound-c-complete.flag` touched.
Audit logs in `.claude/audit-*.log` (gitignored). Submodule init recovery noted
for retro harvest as evidence the lane bootstrap checklist was not applied —
candidate for a `kind: "log"` DQ at next-id walk if user wants explicit retro
signal beyond this runlog entry.

## advisor: retro-candidate — `/tmp` Bash↔Python path mismatch recurred this session

Per `feedback_windows_bash_python_git_show_tmp_traps.md` (written 12 days ago,
2026-05-09), the `/tmp` Bash↔Python divergence is trap #2 of the consolidated
recipe. The lesson exists; the recipe (use `$LOCALAPPDATA/Temp/<name>` or
`.claude/scratch/<name>.json`, never `/tmp/`) is documented; the symptom
(`FileNotFoundError: '/tmp/<file>'` after Bash `ls` shows it exists) is
explicitly listed.

Despite all of that, the trap fired TWICE during the gov-v0 forward-merge
DQ reconcile in this session:

1. First attempt: `tail -20 ... > /tmp/t1c.txt && tail -20 ... > /tmp/t1cl.txt`
   then Python `io.open('/tmp/t1c.txt', ...)` → FileNotFoundError. User-visible
   tool result; resolved by re-issuing Python with direct file reads from the
   `.claude/PRPs/debug/` paths.
2. Second attempt: `git show :2:... > /tmp/dq-phase.json` then Python
   `io.open('/tmp/dq-phase.json', ...)` → FileNotFoundError. User-visible
   tool result; resolved by re-issuing `git show` to
   `C:/Users/barri/AppData/Local/Temp/dq-phase.json`.

User flagged this in-channel: "Note this for retro: /tmp path mismatch again".

**Retro signal:** the lesson exists, is canonical, has a 12-day-old `originSessionId`,
and STILL the trap fires because the advisor session's inline-script
authoring doesn't pre-check the trap list. Two hypotheses for the v1-fed-in-c
retro to consider:

- (a) PMD-search-pre-queue pattern (per `advisor-orchestrator.md` §2.3) is
  designed for brief-authoring, NOT for ad-hoc inline scripts during merge
  reconcile. The lesson is unreachable through the current
  `memory_search_hybrid` workflow because no brief is being written. Need
  either a `/check-tmp-paths` lint or a Bash-tool pre-execution hook that
  flags `/tmp/...` paths and suggests `$LOCALAPPDATA/Temp` / `.claude/scratch/`.
- (b) The advisor's "write a quick Python one-liner inline" muscle memory
  defaults to `/tmp` because Linux Bash works that way; the Windows-specific
  override only kicks in when the FileNotFoundError forces it. A user-scope
  CLAUDE.md instruction "on Windows, never use /tmp for cross-process file
  handoff; default to C:/Users/barri/AppData/Local/Temp" would shift the
  default at brief-time, not error-time.

This recurrence is exactly the kind of "principle vs. rule" gap that
`feedback_principles_not_rules.md` warns about: the principle is clear; the
default behaviour reasserts itself absent active checking.
