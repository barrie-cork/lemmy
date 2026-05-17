# v1-AD-e — resume handoff prompt (fresh session bootstrap)

**Written:** 2026-05-16 by advisor at session wind-down.
**For:** a NEW Claude Code session resuming v1-AD-e from a cold start (no prior conversation context).
**Authoritative source of truth on resume:** `git` + `origin` + `.claude/runlog/v1-AD-e-runlog.md` + `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`. This file is the bootstrap; verify every claim below against live state before acting (claims here are a 2026-05-16 snapshot and may have drifted).

---

## Copy-paste this as the first message to the new session

```
Resume v1-AD-e (Brehon admin-dashboard page-layer sub-phase).

Read .claude/PRPs/handovers/v1-AD-e-resume-bootstrap.md, then
.claude/runlog/v1-AD-e-runlog.md and
.claude/PRPs/plans/v1-admin-dashboard-e.plan.md.

Run /start-brehon v1-AD-e to re-ground on live state. Gate-1
(plan-approval) is ALREADY COMPLETE — do NOT re-run the §15 DoD
smoke, the watchpoint gate, or re-file DQ #237/#238. The next
action is creating the brehon-fork-ad-e lane worktree, then
dispatching plan §13 Task 0 → Task 1 (serial).

HARD CONSTRAINT — parallel development: phase-v1-federation-inbound-a
is a SEPARATE active lane with its own dedicated session + worktree
(C:/Users/barri/Developer/brehon-fork-federation-inbound-a) + its own
phase-branch decision-queue.json. You are the v1-AD-e lane ONLY. Do
NOT write to, dispatch tasks for, mutate DQ on, or recover anything
for federation-inbound-a. Observe it for daemon-capacity only. Per
.claude/rules/multi-lane-worktree.md.
```

---

## State snapshot (verify before trusting — 2026-05-16T22:00Z)

| Item | Value at snapshot | How to re-verify |
|---|---|---|
| `origin/governance-v0` tip | `287ad47e9` | `git ls-remote origin governance-v0` |
| `origin/phase-v1-AD-e` tip | `09e0572cc` (= the bm-cut point, off post-plan-approval trunk) | `git ls-remote origin refs/heads/phase-v1-AD-e` |
| DQ #237 (scope-cut) | RESOLVED — `(a) Dashboard + Audit only`, `answered_by: user` | `.claude/decision-queue.json` resolved[] |
| DQ #238 (engine) | RESOLVED — `(a) maud`, `answered_by: user` | `.claude/decision-queue.json` resolved[] |
| v1-AD-e pending DQ | none (`pending: []` for AD-e; #229 is an unrelated Shape-G-re-enable log) | `/check-dq` or read DQ |
| Lane worktree `brehon-fork-ad-e` | does NOT exist yet | `git worktree list` |
| Gate-1 (plan approval) | COMPLETE (DoD smoke 4/4 green, watchpoint PASS, user-approved) | runlog `## advisor: PARKED` entry |

## Why it was parked (and why the block has largely cleared)

Parked 2026-05-16 "stabilise first" for two reasons:

1. **Daemon saturation** — fed-inbound-a Cohort A had 2 tasks running ~2h + 2 failed. **STATUS NOW: cleared.** As of 2026-05-16T21:24Z all fed-inbound-a Cohort A jobs are terminal: #276/#277 `failed`, #278/#279/#280 `cancelled`. No fed-inbound-a tasks running → daemon has capacity. **RE-VERIFY** on resume: `ssh homeserver 'sqlite3 /srv/brehon-fork/.junior/junior.db "SELECT id,status FROM jobs WHERE status IN (\"running\",\"queued\");"'` — if fed-inbound-a (or any) tasks are running, weigh daemon load before dispatching v1-AD-e impl (bm-cut/Task-0 are cheap; cargo-heavy tasks compete).
2. **Multi-lane shared-`.git/` git-friction** — 3+ concurrent HEAD-moves + a discarded DQ write this session. **STATUS NOW: mitigated.** This session shipped `multi-lane-worktree.md` Hard refusal #6 (atomic read-fetch-mutate-commit-push protocol for canonical-checkout DQ writes). The v1-AD-e lane worktree (created in step 1 below) further isolates its DQ file path. Friction is structurally reduced, not eliminated — follow Hard refusal #6 for any DQ write.

Both halves of the runlog's documented resume trigger are now satisfied. Resume is unblocked.

## Resume checklist (the actual next actions — gate-1 is DONE)

**Step 1 — create the lane-dedicated worktree (CWD-changing; user runs this, or advisor surfaces the commands):**

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin phase-v1-AD-e
git worktree add ../brehon-fork-ad-e phase-v1-AD-e
```

Then **open a NEW Claude Code session with CWD = `C:/Users/barri/Developer/brehon-fork-ad-e`**. That session is the dedicated v1-AD-e lane advisor for the rest of the phase. The canonical `C:/Users/barri/Developer/brehon-fork` checkout stays meta-edit-only (briefs/rules/lessons on governance-v0) per `multi-lane-worktree.md`.

**Step 2 — in the lane session, dispatch plan §13 serially (NO `[P]` — this is a 6-step hard dependency chain):**

- **Task 0** — pre-flight harness audit + branch verification (11 probes; Probe 5/6 check DQ #238/#237 resolved — they ARE, so those probes pass). Non-`[P]`, dispatched alone.
- **Task 1** — add the `maud` HTML engine dep (isolated commit; DQ #238 = maud, so install `maud = { version = "0.26", features = ["actix-web"] }` per plan §11; NOT askama, NO `templates/` dir).
- **Tasks 2→3→4→5** — gather extraction + dashboard handler + audit handler + EventSource script + e2e. Strict serial (3's handler lives in 2's module; 4 edits 3's fn; 5 e2e-tests 2+3's routes). Task 5 is the dedicated e2e append (single edit, `feedback_junior_worker_e2e_edit_hang`).
- **Task 6** — retro.

**Validation mode:** Shape G is suspended until 2026-06-01 → v1-AD-e runs **validate-pending-laptop** (per `advisor-orchestrator.md` §5.2; plan §15.6). impl-task pushes; the lane advisor runs §15.1–15.4 locally; mutates the `kind: "validate-pending-laptop"` DQ entry.

## Hard constraints for the resume session

1. **Gate-1 is COMPLETE — do not repeat it.** The §15 DoD smoke (4/4 green), watchpoint-specificity gate (PASS), and DQ #237/#238 (user-resolved) are all done. Re-running them wastes ~40 min (the e2e baseline alone is ~35 min) and re-surfaces already-answered gates. The runlog `## advisor: PARKED` entry is the proof. Resume starts at **lane-worktree creation**, not gate-1.

2. **Parallel-development isolation (per `.claude/rules/multi-lane-worktree.md`) — ABSOLUTE:**
   - `phase-v1-federation-inbound-a` is a **separate active lane** with its own dedicated session + worktree (`C:/Users/barri/Developer/brehon-fork-federation-inbound-a`) + its own phase-branch `.claude/decision-queue.json`. Its Cohort A failed/cancelled — **that is the fed-inbound-a lane session's problem to triage/replan, NOT yours.**
   - Do NOT: write fed-inbound-a's DQ, dispatch/cancel/retry fed-inbound-a Junior tasks, recover its branch, edit its plan, or `git checkout phase-v1-federation-inbound-a` anywhere. Observe it ONLY for daemon-capacity (is the box free enough to dispatch v1-AD-e cargo work?).
   - `phase-v1-ship-1` is likewise a separate lane (`brehon-fork-ship-1` worktree) — same isolation rule.
   - The v1-AD-e lane writes ONLY `phase-v1-AD-e` DQ entries from the `brehon-fork-ad-e` worktree. Brief authoring / rule / lesson edits go on `governance-v0` from the canonical checkout.

3. **bm-cut already ran — do NOT re-cut.** `phase-v1-AD-e` exists on origin at `09e0572cc` (Junior #282, recovered). Re-running bm-cut would hit the same finalize-merge bug (now documented: `feedback_junior_finalize_merges_bm_cut_branch.md` + bm-cut.md Phase 6). The branch is ready; go straight to the lane worktree.

4. **DQ writes follow Hard refusal #6** (`multi-lane-worktree.md`): fetch → read fresh → recompute next_id across all lanes → mutate → verify → add → commit → push as one uninterrupted sequence. The canonical checkout had a discarded-DQ-write race this session; the lane worktree mostly isolates this but the protocol is still mandatory.

5. **Daemon-side finalize / CC-gate awareness** (apply if any v1-AD-e impl-task is dispatched): impl-task finalize-merge into the phase branch is the normal path; but watch for the gate-blocked-runlog + spurious-merge patterns documented in `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` and `feedback_junior_finalize_merges_bm_cut_branch.md`. Verify daemon-local trunk + phase-branch tips post-task.

## Plan summary (so the resume session has context without re-reading the whole plan)

v1-AD-e = the final page-layer sub-phase of the shipped v1-AD keystone. Ships the **first server-rendered HTML in the workspace**: two admin-gated browser pages (Dashboard + Audit) rendering data the **already-shipped** v1-AD-d endpoints produce. **Zero new DB read paths / DTOs / migrations / config keys.** Complexity 4/10, 7 tasks, strictly serial. Engine = **maud** (DQ #238). Scope = **Dashboard + Audit only** (DQ #237; Config-editor/Single-key/Rule-set-manager deferred to a future v1-AD-f). No ADR contradicted (ADR-010 React=v2 unchanged — this is server-rendered HTML, not SPA). The only structural change is a behaviour-preserving `gather_dashboard` extraction, regression-guarded by the v1-AD-d e2e test.

## Open retro items (context, not blockers for resume)

These shipped as lessons this session; the resume session inherits them as standing discipline, not as work to do:
- `feedback_junior_finalize_merges_bm_cut_branch.md` — bm-cut finalize-merge bug + recovery (already applied to #282).
- `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` — `.claude/**` gate blocks Junior BM writes; advisor-relocate is the expected path.
- `multi-lane-worktree.md` Hard refusal #6 — atomic canonical-checkout DQ-write protocol.
- `feedback_pmd_backfill_after_write.md` — PMD has no write-time embedding; session-retro Step 5.5 / weekly-review Step 1b enforce backfill.

## See also

- `.claude/runlog/v1-AD-e-runlog.md` — the durable per-phase ledger (bm-cut + PARKED entries).
- `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` — the approved plan (§13 task list, §15 DoD, §16a stories).
- `.claude/PRPs/reports/v1-AD-e-plan-approval-gate.md` — the original gate-1 handoff note (now superseded by this file for the resume case; kept for audit).
- `.claude/rules/multi-lane-worktree.md` — the parallel-development discipline this handoff enforces.
- `.claude/rules/advisor-orchestrator.md` §3.1 / §5.2 — stage-shape + validate-pending-laptop mechanics.
- `.claude/PRPs/reports/session-retro-2026-05-16-v1-ad-e-gate-bmcut-finalize-recovery.md` — the session that produced this state + the lessons above.
