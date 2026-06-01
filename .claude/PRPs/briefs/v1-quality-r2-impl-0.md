# Brief: impl-task 0 — v1-quality-r2 pre-flight harness audit

**Role:** `[role:impl-task]`
**Phase:** `v1-quality-r2` (r2-half: #156/#159/#160; branch `phase-v1-quality-r2`)
**Authored:** 2026-05-29
**Authored by:** advisor (canonical brehon-fork session, on `governance-v0`)
**Base branch (Junior forks from):** `phase-v1-quality-r2`
**Lane mode:** Mode B (mobile remote-control). Junior worker runs on the EliteDesk daemon (Linux).

---

## 1. Role + dispatch

`[role:impl-task] v1-quality-r2 task 0 — pre-flight harness audit — see .claude/PRPs/briefs/v1-quality-r2-impl-0.md`

---

## 2. Scope

Pre-flight audit only. **No code edits. No commits on success.** Verify the environment is ready for v1-quality-r2 impl tasks T3/T4/T5 (all `crates/server/tests/e2e.rs`).

**Platform note:** The Junior worker runs on a Linux daemon. The plan §13 Task 0 cites `cmd //c "scripts\\brehon\\cargo-*.bat ..."` (Windows laptop syntax) — **on the daemon, use the `bash scripts/brehon/cargo-*.sh` siblings** with identical flags. All flags/log-paths/expected-exit-codes below are byte-identical to the plan intent; only the wrapper invocation differs (Linux `.sh`, not Windows `.bat`). Per `feedback_handover_assumptions_need_empirical_verification.md` this brief carries the Linux-syntax wrappers verbatim rather than betting on adaptive recovery.

### 2.0 Common environment bootstrap (run FIRST, per `feedback_phase_lane_worktree_bootstrap_checklist.md`)

Junior worktrees do NOT auto-initialize submodules. Before any cargo probe:

```bash
git submodule update --init --recursive
```

This fixes the `crates/email/translations` empty-gitlink → `Os { code: 3, kind: NotFound }` failure that breaks any cargo command touching `lemmy_email`. Confirmed working on r2a Task 0 + Lane A Task 0 (#489).

### 2.1 Probes (run in order)

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch base confirmation (Junior framework always runs on a junior/* branch)
git branch --show-current
git merge-base --is-ancestor phase-v1-quality-r2 HEAD && echo "BASE_OK phase-v1-quality-r2 is ancestor" || echo "BASE_MISMATCH — STOP"
# EXPECT: junior/...; BASE_OK
# Per RT-r3 Task 0 DQ 1b8527b076d4-001: Junior framework runs on junior/* branch; base IS phase-v1-quality-r2.

# Probe 2 — wrapper sanity (cargo-check honors -p)
bash scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/PRPs/debug/v1-quality-r2-task0-check-p.log 2>&1
echo "check-p exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2-task0-check-p.log
# EXPECT: exit 0

# Probe 3 — feature flag activation (the build the T3/T4/T5 gates use)
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-quality-r2-task0-check-features.log 2>&1
echo "check-features exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2-task0-check-features.log
# EXPECT: exit 0

# Probe 4 — negative-feature exit-code propagation
bash scripts/brehon/cargo-check.sh --workspace --features nonexistent_xyz > .claude/PRPs/debug/v1-quality-r2-task0-negative.log 2>&1
echo "negative exit: $?"
# EXPECT: NON-ZERO (typically 101)

# Probe 5 — test-target compile (R7 — T4 changes boot_context signature; confirm e2e compiles clean on base)
bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server --features full > .claude/PRPs/debug/v1-quality-r2-task0-test-norun.log 2>&1
echo "test-norun exit: $?"
tail -20 .claude/PRPs/debug/v1-quality-r2-task0-test-norun.log
# EXPECT: exit 0

# Probe 6 — clippy baseline on the cut tip (capture pre-existing warnings BEFORE any edit)
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r2-task0-clippy.log 2>&1
echo "clippy exit: $?"
tail -40 .claude/PRPs/debug/v1-quality-r2-task0-clippy.log
# EXPECT: exit 0. If non-zero, capture the warning set verbatim in task output —
# it is the pre-existing baseline T3/T4/T5 must not add to (per feedback_clippy_rerun_after_fix.md).

# Probe 7 — ANCHOR ENUMERATION (the load-bearing drift guard — plan contract = 11 / 14 / 14)
echo "boot_context refs (EXPECT 11 = 1 decl + 10 callsites):"
rg -c "boot_context" crates/server/tests/e2e.rs
echo "LEMMY_DATABASE_URL setters via set_var (EXPECT 14 = 13 in-scope for T5 + 1 in boot_context for T4):"
rg -c 'env::set_var\("LEMMY_DATABASE_URL"' crates/server/tests/e2e.rs
echo "*_fixtures modules (EXPECT 14):"
rg -c "^mod \w+_fixtures \{" crates/server/tests/e2e.rs
echo "EnvVarGuard struct (EXPECT 1, still inside v1_rt_r3_fixtures, NOT yet hoisted):"
rg -n "struct EnvVarGuard" crates/server/tests/e2e.rs
# EXPECT: 11 / 14 / 14 / 1-match-around-line-17185.
# If ANY count differs from 11/14/14, a concurrent lane (redaction-r1) shifted e2e.rs —
# file a kind:"blocker" DQ with the actual counts so the advisor re-derives T3/T4/T5 anchors
# BEFORE dispatch. This is the anchor-drift guard (per r2a Task-0 Probe-6 plan-id-mismatch lesson +
# the advisor's clarify DQ a3d0e9941441-037 which verified 11/14/14 on governance-v0).

# Probe 8 — confirm no OTHER open PR touches crates/server/tests/e2e.rs (overlap with Lane A redaction-r1)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/server/tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output. If any open PR (e.g. a redaction-r1 PR) touches e2e.rs, STOP and file
# kind:"blocker" DQ — the advisor must serialize/rebase before T3/T4/T5 dispatch.
```

**EXPECT block:**
- Probes 0, 1, 2, 3, 5, 6 exit 0
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 1 returns `junior/...` branch + `BASE_OK phase-v1-quality-r2 is ancestor`
- Probe 7: counts are exactly **11 / 14 / 14**; EnvVarGuard present (1 match, ~line 17185, un-hoisted)
- Probe 8: empty output (no concurrent PR touching e2e.rs)

**Task 0 produces NO commit** if all probes pass — write results to task output only. If any probe fails OR Probe 7 counts drift OR Probe 8 finds an overlapping PR: write a `kind: "blocker"` DQ entry, commit + push it on the worker branch, and stop.

### 2.2 Pipe-mask-exit-code discipline (per `feedback_pipes_mask_exit_codes.md`)

Never pipe a wrapper invocation through `tail`/`head`/`grep` to read its exit code — the pipe sets `$?` to the final stage (0 for `tail`), masking the real exit. Redirect to a log first, `echo "exit: $?"` from the original invocation, THEN `tail` the log (as Probes 2/3/5/6 do above).

---

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r2.plan.md` §13 Task 0 (the canonical probe list + EXPECT block; **read the RE-SCOPED banner at the top first** — live scope is T0/T3/T4/T5/T6, Tasks 1+2 shipped in r2a) + §10.7 (the 14 fixtures modules) + §11 (the 13 LEMMY_DATABASE_URL sites + 10 boot_context callsites).
- `.claude/rules/pre-phase-harness-audit.md` (R5: enumerate ALL probes explicitly)
- `.claude/rules/decision-queue.md` (DQ schema-v3 + Junior attribution; Hard refusals #1, #6, #8, #9)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` (submodule init pre-empt)
- `.claude/lessons/feedback_pipes_mask_exit_codes.md` (Probe exit-code discipline)
- `.claude/lessons/feedback_handover_assumptions_need_empirical_verification.md` (why this brief carries Linux-syntax wrappers verbatim)
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` (wrappers pass `$@` literally; the flag set IS load-bearing)

---

## 4. Constraints

- **No code edits** — verification only.
- **No commit on success** — audit output goes to task output, not git.
- If any probe fails / Probe 7 drifts / Probe 8 overlaps: file `kind: "blocker"` DQ (`bash scripts/brehon/dq-v3-new-entry.sh` for the composite id; `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append), commit + push on the worker branch, then stop. Do not proceed.
- Branch base MUST be `phase-v1-quality-r2` (Probe 1's ancestor check confirms; BASE_MISMATCH → blocker DQ).
- **Attribution:** worker DQ writes use `from: "impl"`. **NEVER write `answered_by: "advisor"`/`"user"` or `approved_by: <non-null>`** per `decision-queue.md` Hard refusals #1 + #8.
- **Mid-task push discipline:** a blocker DQ must be committed + pushed on the worker branch in the SAME atomic sequence (per `decision-queue.md` §"Mid-task visibility").
- **Shape G SUSPENDED** (DQ #229). This Task 0 runs cargo probes (2/3/5/6) **inline on the daemon** — the verification-only exception (no commit/push, so the validate-pending pathway doesn't apply). Precedent: RT-r3 Task 0 (`46b73c130`) + r2a Task 0 + Lane A Task 0 (#489), all ran cargo inline successfully. (T3/T4/T5 will use validate-pending-laptop because they commit + push.)

---

## 5. Forbidden-window check (advisor pre-queue)

Per `.claude/rules/advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check IS binding for Task 0 (runs cargo inline on EliteDesk). Advisor verifies the dispatch minute is outside all standard windows at queue time. Subagent's task-0 pre-flight refuses with `FORBIDDEN_WINDOW: <window>` if mis-queued.

---

## 6. Context

- **Phase:** v1-quality-r2 (r2-half; the 3 e2e.rs carry-forwards #156/#159/#160. #157+#158-defer shipped in r2a PR #161.)
- **Plan:** `.claude/PRPs/plans/v1-quality-r2.plan.md` on `phase-v1-quality-r2` (re-scoped; RE-SCOPED banner at top).
- **Phase branch:** `phase-v1-quality-r2`, cut + FF-recovered to `governance-v0` HEAD `99eb3da46` (a stale r2a-era branch was reused by bm-cut #508; advisor FF-recovered it — the branch now == trunk, carries the re-scoped plan).
- **Base branch for this task:** `phase-v1-quality-r2`.
- **Lane mode:** Mode B (no laptop-side phase worktree).
- **Clarify:** DQ `a3d0e9941441-037` verified 11/14/14 anchor counts hold on governance-v0 @74197312d. Probe 7 re-verifies against the actual phase-branch tip (this is the in-band guard).
- **Concurrent lane:** `phase-v1-redaction-r1` (Lane A) edits e2e.rs in a DIFFERENT region (redaction tests). Probe 8 checks for an open redaction PR touching e2e.rs; if found, advisor serializes before T3/T4/T5.
- **After Task 0 passes (all probes EXPECT satisfied):** advisor reads task output, no merge needed (no commit), authors + dispatches T3 (the doc-comment audit — first e2e edit).
- **If Task 0 surfaces a blocker DQ:** advisor reads it on next poll, routes per §5.4 DQ triage decision tree (most likely: anchor-count drift → re-derive T3/T4/T5 anchors).
