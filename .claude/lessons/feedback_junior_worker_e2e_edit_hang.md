---
name: Junior worker silently hangs on Edit calls into 8945-line e2e.rs
description: Junior `claude -p` workers stop emitting tool_results when editing the 8945-line crates/server/tests/e2e.rs; log freezes mid-task while CPU stays alive; only the 60-min watchdog or manual cancel ends it
type: feedback
originSessionId: 565fdae3-1248-4e0e-939c-df2c15752f56
---
Junior workers running on `phase-v1-JM-d` reliably hang when issuing `Edit` calls into `crates/server/tests/e2e.rs` (8945 lines as of 2026-04-27). The worker's log file stops growing, but `ps -p <worker-pid>` shows the process alive with non-zero CPU — it's not crashed, it's stuck mid-tool-result. Only the 60-min inactivity watchdog SIGTERMs it, or the advisor cancels.

Confirmed sightings:
- **Task #13** (resume of impl-2): wedged on Edit calls into e2e.rs; advisor pre-edit at commit `1646ddb` flushed partial work (commit body: "task #13 wedged on Edit calls into this 8945-line file (no tool_result returned, watchdog SIGTERM'd 2h later)").
- **Task #14** (deadlock fix #99): worker alive 2h28m with CPU climbing 5%→33% but log frozen at 18:13 UTC. Cancelled, recovered the 200-line patch from `/tmp/`, applied manually, committed as `27b212e`.

**Why:** Best guess is Edit's diff computation against an 8945-line file exceeds some internal timeout or buffer, but the harness doesn't surface the failure as an error — it just stops emitting tool_results. The worker keeps thinking but can't make progress.

**How to apply:**
- For any task whose §15 DoD names edits to `crates/server/tests/e2e.rs`, **never queue a single Junior task that does the work end-to-end**. Either:
  - Pre-edit the file from the advisor session before queueing (clean small edits, well-separated lines — tasks #14 hung even on `sed -i "8797d"` would have been fine; the model picked Edit by default).
  - Or split the task: planning subagent identifies the exact lines, advisor applies the edits via `sed`/`Edit` from the orchestrator session, Junior runs only the validation pass.
- During polling: if a Junior task on `phase-v1-JM-d` shows >30 min log silence AND its DoD touches e2e.rs, **don't wait for the watchdog**. Cancel, recover the patch from the worktree (`git -C /srv/brehon-fork-worktrees/<branch> diff > /tmp/patch.diff`), apply manually.
- Until the file is split or refactored (it's a known JM-e candidate), treat e2e.rs as a no-Junior-Edit zone.

**Symptom to recognise:** worker process alive (non-zero CPU per `ps -p`), log file mtime older than 30 min, last log line ends mid-tool-call (`"name": "Edit"` with no matching tool_result), `tail -f` shows no new output across multiple polls.

**Generalises to:** any large file (>5000 lines) on hot Junior-task paths. Same applies to monolithic schema files, large migration bundles, large frontend route files. The threshold is uncertain — treat 5000 as a yellow flag, 8000+ as red.

**Reconciliation 2026-05-29 (v1-rt-r3-followup — the hang no longer reproduces with pre-located anchors):** this lesson was promoted from the System-1 memory dir into `.claude/lessons/` so the §2.4 mandatory-injection table citation resolves to a real corpus file (it had been cited but never present here — surfaced as a corpus-drift retro item). The original JM-d-era symptom (2026-04-27, 8945-line file) did NOT recur in v1-rt-r3-followup: Junior workers #505 and #506 each made **2 Edits into the now-18,089-line `e2e.rs`** and completed in **2.7–3.75 min with zero hang**. The difference is almost certainly the discipline in `feedback_fix_impl_pre_locate_e2e_anchors.md` — when the brief hands the worker **verbatim `old_string`/`new_string` anchors** (so Edit does an exact-match replace, not a search-and-diff over the whole file), the diff-computation cost that caused the hang is avoided. **Practical takeaway:** the no-Junior-Edit-zone rule is now SOFTENED to: e2e.rs Junior Edits are safe **when** the brief carries pre-located verbatim anchors AND the per-task edit count is ≤2 (the template §2.0 scope gate). End-to-end "go find and fix" e2e tasks without anchors remain a no-Junior-Edit zone. The §2.4 table accordingly cites `feedback_fix_impl_pre_locate_e2e_anchors.md` as the primary e2e-edit lesson; this lesson is the historical-symptom + recovery-recipe companion.
