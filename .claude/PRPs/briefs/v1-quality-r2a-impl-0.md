# Brief: impl-task 0 — v1-quality-r2a pre-flight harness audit

**Role:** `[role:impl-task]`
**Phase:** `v1-quality-r2a` (branch stays `phase-v1-quality-r2`; plan filename signals scope)
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base branch (Junior forks from):** `phase-v1-quality-r2`
**Lane mode:** Mode B (mobile remote-control). Junior worker runs on the EliteDesk daemon (Linux).

---

## 1. Role + dispatch

`[role:impl-task] v1-quality-r2a task 0 — pre-flight harness audit — see .claude/PRPs/briefs/v1-quality-r2a-impl-0.md`

---

## 2. Scope

Pre-flight audit only. **No code edits. No commits on success.** Verify the environment is ready for v1-quality-r2a impl tasks.

**Platform note:** The Junior worker runs on a Linux daemon. The plan §13 Task 0 cites `bash scripts/brehon/cargo-*.sh` invocations directly (already Linux-syntax). All probe parameters (flags, log paths, expected exit codes) are byte-identical to the plan.

Run the 9 probes (Probe 0 through Probe 8) from plan §13 Task 0 in order:

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch base confirmation (Junior on junior/*)
git branch --show-current
git merge-base --is-ancestor phase-v1-quality-r2 HEAD && echo "BASE_OK phase-v1-quality-r2 is ancestor" || echo "BASE_MISMATCH — STOP"
# EXPECT: junior/...; BASE_OK
# Per RT-r3 Task 0 DQ 1b8527b076d4-001 methodology gap acceptance — Junior framework always runs on junior/* branch; base IS phase-v1-quality-r2.

# Probe 2 — wrapper sanity (cargo-check honors -p)
bash scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/PRPs/debug/v1-quality-r2a-task0-check-p.log 2>&1
echo "check-p exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2a-task0-check-p.log
# EXPECT: exit 0

# Probe 3 — feature flag activation
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-quality-r2a-task0-check-features.log 2>&1
echo "check-features exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2a-task0-check-features.log
# EXPECT: exit 0

# Probe 4 — negative-feature exit-code propagation
bash scripts/brehon/cargo-check.sh --workspace --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r2a-task0-negative.log 2>&1
echo "negative exit: $?"
# EXPECT: NON-ZERO (typically 101)

# Probe 5 — clippy baseline
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2a-task0-clippy.log 2>&1
echo "clippy exit: $?"
tail -40 .claude/PRPs/debug/v1-quality-r2a-task0-clippy.log
# EXPECT: exit 0

# Probe 6 — confirm 4 named DQ entries exist and are currently back-dated
python3 -c "
import json
d = json.load(open('.claude/decision-queue.json'))
ids = {3999, 4007, 4016, 4035}
hits = [e for e in d.get('resolved', []) if e.get('id') in ids or e.get('id_v1') in ids]
print(f'found {len(hits)} of 4 target entries')
backdated = [e for e in hits if e.get('resolved_at') and e.get('timestamp') and e['resolved_at'] < e['timestamp']]
print(f'{len(backdated)} are currently back-dated')
"
# EXPECT: found 4 of 4; 4 are currently back-dated
# If counts differ, file a kind:"blocker" DQ asking the advisor to confirm
# whether some entries have already been swept (idempotent re-run safe) OR
# whether the issue id list (#157 body) is stale relative to current DQ state.

# Probe 7 — confirm dq-v3-new-entry.sh + dq-v3-append-fragment.sh present
ls -la scripts/brehon/dq-v3-*.sh
# EXPECT: both scripts present + executable

# Probe 8 — confirm no other PR open touches r2a's IMPLEMENT files
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("scripts/brehon/dq-lint|scripts/brehon/precheck|decision-queue\\.json")) | {number, title, headRefName}'
# EXPECT: empty output; if any other lane is touching r2a files, STOP and file kind:"blocker" DQ
```

**EXPECT block:**
- Probes 0, 1, 2, 3, 5, 6, 7, 8 exit 0 (or as documented per probe)
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 1 returns `junior/...` branch + `BASE_OK phase-v1-quality-r2 is ancestor`
- Probe 6: confirms 4 of 4 target entries present + all 4 currently back-dated
- Probe 7: both scripts present
- Probe 8: empty output (no concurrent PR overlap)

**Task 0 produces NO commit** if all probes pass — write results to task output only. If any probe fails: write a `kind: "blocker"` DQ entry to `.claude/decision-queue.json`, commit + push it on the worker branch, and stop.

### 2.1 Common environment bootstrap (per `feedback_phase_lane_worktree_bootstrap_checklist.md`)

Junior worktrees do NOT auto-initialize submodules. Before Probe 2, run:

```bash
git submodule update --init --recursive
```

This fixes the `crates/email/translations` empty-gitlink → `Os { code: 3, kind: NotFound }` failure that breaks any cargo command touching `lemmy_email`. Per RT-r3 Task 0 DQ 1b8527b076d4-001 empirical recovery + Lane A Task 0 (#489) confirmed working.

### 2.2 Pipe-mask-exit-code discipline (per `feedback_pipes_mask_exit_codes.md`)

When capturing the exit code from a wrapper-script invocation, **never pipe the output through `tail`/`head`/`grep`** — the pipe sets `$?` to the final pipeline stage (typically 0 for `tail`), masking the wrapper's actual exit code. Redirect to a log file first, then `tail` the log, then check `$?` from the original invocation:

```bash
# CORRECT (per Probes 2-5 above)
bash scripts/brehon/cargo-check.sh -p lemmy_utils > /tmp/log 2>&1
echo "exit: $?"
tail -20 /tmp/log

# WRONG (pipe masks exit code)
bash scripts/brehon/cargo-check.sh -p lemmy_utils | tail -20
echo "exit: $?"  # always 0 (tail succeeded)
```

Lane A Task 0 (#489) recovered from this defect class mid-task; the brief pre-empts it.

---

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r2a.plan.md` §13 Task 0 (the canonical probe list + EXPECT block; this brief mirrors that block verbatim)
- `.claude/rules/pre-phase-harness-audit.md` (R5: enumerate ALL probes explicitly)
- `.claude/rules/decision-queue.md` (DQ schema-v3 + Junior subagent attribution rules; Hard refusals #1, #6, #8, #9)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` (submodule init pre-empt)
- `.claude/lessons/feedback_pipes_mask_exit_codes.md` (Probe-4 + Probe-5 exit-code discipline)
- `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` (why this brief carries Linux-syntax wrappers verbatim rather than betting on Junior's adaptive recovery)
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` (wrapper scripts pass `$@` literally; flag set IS load-bearing)

---

## 4. Constraints

- **No code edits** — this task is verification only.
- **No commit on success** — audit output goes to task output, not git.
- If any probe fails: file `kind: "blocker"` DQ (use `bash scripts/brehon/dq-v3-new-entry.sh` for the composite id; use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append), commit + push on the worker branch, then stop. Do not proceed to Task 1.
- Branch base MUST be `phase-v1-quality-r2` (Probe 1's `git merge-base --is-ancestor` ancestor check confirms this; if it returns BASE_MISMATCH, file a blocker DQ).
- **Attribution:** the worker's DQ writes use `from: "impl"`. **NEVER write `answered_by: "advisor"` or `answered_by: "user"` or `approved_by: <non-null>`** per `.claude/rules/decision-queue.md` Hard refusals #1 + #8.
- **Mid-task push discipline:** if a blocker DQ fires, commit + push the DQ entry on the worker branch in the SAME atomic sequence (per `decision-queue.md` §"Mid-task visibility"). The advisor's next poll sees the new pending DQ; without the push the advisor cannot see it.
- Shape G is **SUSPENDED** per DQ #229 — this is a pre-Shape-G plan. **r2a touches zero Rust** so Tasks 1+2 have no cargo gates. **Task 0 cargo probes (2, 3, 4, 5) are the exception: they run inline on the EliteDesk worker because Task 0 has no commit/push and the per-task validate-pending pathway doesn't apply to verification-only tasks.** This exception precedent is established by RT-r3 Task 0 (commit `46b73c130`) and re-confirmed by Lane A Task 0 (#489 completed all 13 probes inline successfully).
- **No e2e gate for r2a** — the plan ships zero e2e edits. The e2e gate moves to r2b.

---

## 5. Forbidden-window check (advisor pre-queue)

Per `.claude/rules/advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check IS binding for Task 0 (runs cargo inline on EliteDesk). Advisor verified at queue time (~20:10+ UTC = 20:10 UK; outside all standard forbidden windows). Subagent's task-0 pre-flight refuses with `FORBIDDEN_WINDOW: <window>` if mis-queued.

---

## 6. Context

- **Phase:** v1-quality-r2a (narrowed scope per gate-1 split; closes #157 + records deferral DQ for #158)
- **Plan:** `.claude/PRPs/plans/v1-quality-r2a.plan.md` on `phase-v1-quality-r2` at `8ee61efca`
- **Original full-scope predecessor:** `v1-quality-r2.plan.md` on same branch — historical context for r2b only; NOT read by Junior for r2a tasks
- **Phase branch:** `phase-v1-quality-r2` (cut from governance-v0 at `1edb8b94c`)
- **Base branch for this task:** `phase-v1-quality-r2`
- **Lane mode:** Mode B (no laptop-side phase worktree)
- **Plan-approved at:** 2026-05-28T20:10 UTC (advisor gate-1 sign-off; r2a complexity 1/10 well below split threshold)
- **Concurrent lane activity:** Lane A Task 0 (#489) just COMPLETED all 13 probes successfully (~12 min); Lane A ready for Task 1 dispatch. Zero file overlap (r2a = scripts + DQ JSON; Lane A = `crates/db_schema/src/source/governance/redaction.rs`).
- **After Task 0 passes (all 9 probes EXPECT block satisfied):** advisor reads task output, no merge needed (no commit), queues Task 1 brief author + dispatch.
- **If Task 0 surfaces a blocker DQ:** advisor reads the DQ on next poll, routes per §5.4 DQ triage decision tree.
