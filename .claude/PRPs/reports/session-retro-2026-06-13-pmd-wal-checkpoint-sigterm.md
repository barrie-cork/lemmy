# Session retro — 2026-06-13 — pmd-wal-checkpoint-sigterm

**Harness:** claude-code
**Session window:** ~2026-06-13 23:30 → 23:45 UTC (~45 min)
**Branch at start:** `e1677fcee` (`governance-v0`)
**Branch at end:** `e1677fcee` (`governance-v0`) — no brehon-fork commits this session
**Files touched:** 7 (5 in the `project-memory-mcp` daemon repo on homeserver; 2 PMD memory `.md` files in the user dir)
**Commits:** 1 (explicit: 1 — `4b7a9ce` in the standalone daemon repo on homeserver; 0 in brehon-fork)

## TL;DR

Discussed and implemented the durable fix for the PMD HTTP daemon's restart-unsafety (issue-note carried since the 2026-06-13 split-store incident). The discussion converged fast on **Option B** (in-daemon SIGTERM WAL-checkpoint) over Option A (systemd `ExecStop=sqlite3 wal_checkpoint`), and grounding the discussion in the *actual source* — not the issue-note's prose — was what made the choice airtight: the daemon holds a single long-lived `better-sqlite3` connection, so an external checkpoint on the path literally cannot flush its frames and can hit the wrong inode after a path swap. Shipped `closeDb()` + a re-entrancy-guarded SIGTERM/SIGINT handler + systemd `KillSignal`/`TimeoutStopSec`, then verified end-to-end with a sentinel-row restart test (row survived in the on-disk DB; WAL truncated to nothing). The top carry-forward: **for a "fix X" issue-note, read the live code/config before picking between the note's pre-written options — the note's own RCA is a hypothesis.**

---

## What surprised us

- **The bug was live, not historical.** The issue-note read as a past incident with a snapshot-timer mitigation already in place ("important-not-urgent"). But `ls` on the DB dir showed a `memory.db (deleted)` stranded inode *and* a 1.4 MB uncheckpointed live WAL at the moment we looked — i.e. the exact at-risk state was present right then, one `systemctl restart` away from loss. The note's framing under-sold the live exposure.
- **A naive restart onto the new binary would itself have lost data.** The fix's own deployment had a trap: the *currently running* process had no handler, so the restart that loads the new binary SIGTERMs a handler-less process and abandons its WAL. Caught this before restarting and externally checkpointed the live 1.4 MB WAL first (path/inode matched at that instant, so it flushed cleanly). Easy to have missed — the fix could have caused the very loss it prevents, on the transition that installs it.
- **Option A is worse than "weaker" — it's actively misleading.** Expected A to be a partial/degraded version of B. In reality, given the singleton open connection, an external `sqlite3` checkpoint returns a green exit while doing nothing useful (or checkpointing the wrong inode). A "succeeded" exit code that silently no-ops is worse than no fix, because it suppresses the alarm. The issue-note's own parenthetical had flagged this; reading the source confirmed *why*.
- **`db.close()` removing the `-wal` file entirely** (not just truncating it) was the clean verification signal — after restart the `-wal` path 404'd via `stat`, which is an even stronger "handler ran" proof than "WAL size == 0".

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Promote a lesson: **"fix-X" issue-notes carry a pre-written RCA + option list; read the live code/config before choosing — the note's premise is a hypothesis, not a contract.** This is the same discipline as `feedback_falsifiable_hypothesis_before_structural_fix.md` but applied to *self-authored issue-notes*, not just DQ structural-fix entries. `.claude/lessons/feedback_issue_note_options_are_hypotheses.md` | Stops a future session from implementing a pre-written option (e.g. Option A) without verifying its premise against source — would have silently shipped a no-op fix here | minor (one lesson file) | 1× this session + 1× prior (DQ #338 daemon-finalize falsification, `feedback_falsifiable_hypothesis_before_structural_fix.md`) → **threshold met** |
| 2 | When a fix changes a long-running daemon's shutdown behaviour, **checkpoint/flush the live volatile state BEFORE the deploy restart** — the restart that installs a shutdown-handler fix runs against the *old* handler-less process. Add as a one-line note to the issue-note resolution and (if it recurs) a deploy-discipline lesson. | Prevents the install-time data-loss trap; generalises to any "graceful shutdown" fix deployed via restart | minor | 1× this session, 0× prior — **recorded, not yet promoted** |
| 3 | The `project-memory-mcp` daemon repo on homeserver **has no git remote** — `4b7a9ce` exists only on that box. Decide whether to add a remote/mirror so the daemon source is backed up off-host (it's the substrate the whole PMD depends on). | Off-host durability for the PMD daemon source; today a homeserver disk loss takes the only copy of this fix | medium (needs a remote + push policy decision) | 1× this session — **surface to user, decision pending** |

## What to carry forward

- **Ground the discussion in `systemctl cat` + the actual `.ts` source before debating options.** Reading `index.ts`/`db.ts` turned a two-option toss-up into a one-option certainty in ~2 tool calls. The decisive facts (singleton module-level connection, zero signal handling, no `db.close()` anywhere) were invisible from the issue-note alone.
- **Sentinel-row restart test as the proof-of-fix for any WAL/durability change.** Write a uniquely-titled row → confirm it sits uncheckpointed in the WAL → restart → confirm (a) the handler's log line fired, (b) the WAL truncated/vanished, (c) the row reads back from the on-disk main DB via a *fresh* connection. Three independent confirmations; no hand-waving. Reusable verbatim for the deferred write-time-embedding / ExecStop-class changes.
- **Edit-locally-then-`scp`-back for remote source edits.** Pulled `db.ts`/`index.ts` to a temp dir, used precise `Edit` anchors, `scp`'d back, built with `tsc` on the box. Cleaner than `sed`/heredoc surgery over SSH; the temp dir was removed at the end. Used twice cleanly this session.
- **Re-entrancy guard + `unref`'d hard-exit fallback timer is the right shape for a Node signal handler** — a second SIGTERM during shutdown is ignored, and a hung checkpoint still exits before systemd's `TimeoutStopSec → SIGKILL`. Carry this shape to any future Node daemon shutdown work.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`. (Single-thread manual infra session — no four-role split, no Junior workers, so per-role structure is N/A; scoring is per tool-batch / decision.)

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `systemctl cat` + source read (index.ts/db.ts) | 20 | 0 | medium | Converted the A-vs-B debate to a certainty; surfaced the singleton-connection fact that decides everything |
| `AskUserQuestion` (repo-status + next-step) | 3 | 0 | none | Clean fork: confirmed separate-repo + implement-now; avoided guessing at attribution path |
| `ls` on DB dir (pre-restart) | 15 | 0 | high | Surfaced the LIVE 1.4 MB WAL + stranded deleted inode → triggered the pre-restart checkpoint that averted install-time loss |
| Edit-local + scp + `tsc` build | 8 | 0 | none | Precise anchors; clean compile first try |
| Sentinel-row restart test | 10 | 0 | low | Three-way proof; `-wal` vanishing was a stronger signal than expected |
| Manual systemd unit edit (python in-place insert + backup) | 2 | 0 | none | Backup saved; idempotent insert after `Type=simple` |

Net: ~58 min saved (vs cold-deriving the fix + a debugging round had Option A been shipped), ~0 min wasted (no dead ends, no wrong premises pursued).

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. (Metric is designed for Junior impl-task watchdog envelopes; adapted here for the one substantive task. `max-log-silence` is N/A — no Junior worker / telemetry CSV — so recorded as `—`.)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| PMD daemon SIGTERM WAL-checkpoint (db.ts + index.ts + unit + build + verify) | 7 | 1 | ~45 | — (manual session, no worker telemetry) |

No watchdog-envelope flags apply (manual session). Files-touched (7) is just under the >8 threshold and is spread across two repos + a systemd unit; cohesion was genuine (one logical change), not bundling drift.

## Decisions to revisit

- **No git remote on the `project-memory-mcp` daemon repo** (carry-forward #3). The PMD is load-bearing for the whole advisor lesson/retro substrate; its daemon source living single-copy on homeserver is a durability gap worth a deliberate decision.
- The deferred **write-time-embedding spec** (`mcp-write-time-embedding.md`, banked in MEMORY.md) touches the same `db.ts` / `index.ts` surface. When it ships, reuse this session's sentinel-restart test harness and the `closeDb()` seam.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (issue-note options are hypotheses; read live source before choosing): promote to `.claude/lessons/feedback_issue_note_options_are_hypotheses.md` (cross-harness lesson). **Threshold met** (1× here + 1× prior: `feedback_falsifiable_hypothesis_before_structural_fix.md` DQ #338). Could alternatively be a one-paragraph extension to that existing lesson rather than a new file — user's call.
- [ ] **Change #2** (flush volatile daemon state before the deploy restart that installs a shutdown fix): record in the issue-note resolution now; promote to a deploy-discipline lesson only if it recurs (currently 1×).
- [ ] **Change #3** (add a remote/mirror for the `project-memory-mcp` daemon repo): not a lesson — an infra decision for the user.
- [ ] PMD eval write for Change #1's pattern (recurrence ≥ 2 met) — see Step 5 note below.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted —
no `/auto-phase` invocation and no auto-state mutation this session
(leftover JSONs are from prior sessions; Step 0.5 trigger did not fire)._
