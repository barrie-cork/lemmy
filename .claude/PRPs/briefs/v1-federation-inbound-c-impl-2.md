---
phase: v1-federation-inbound-c
role: impl-task
task: 2
brief_n: 2
authored: 2026-05-21
plan: .claude/PRPs/plans/v1-federation-inbound-c.plan.md
plan_task: "§13 Task 2 — Add .order_by(valid_from.desc()) in publish_trust_attestation's inline actor_cap block"
parent_phase_tip: aa3c77ab5 (phase-v1-federation-inbound-c @ DQ pre-reserve #328/#329, origin tip)
cohort: "Cohort 1 (Tasks 1+2, 2-way [P]) — dispatched in parallel; zero file overlap (Task 1 = inbox.rs, Task 2 = publish_trust_attestation.rs)"
related_dq: "324 (advisor INSERT-vs-UPDATE clarify, resolved); 325 (workaround-comment plan, resolved); 326 (Junior #393 finalize deletion log); 329 (BINDING — reserved validate-pending-laptop stub for THIS task)"
reserved_validate_dq: 329
---

# [role:impl-task] v1-federation-inbound-c Task 2 — publish_trust_attestation actor_cap reader-side append-history fix — see .claude/PRPs/briefs/v1-federation-inbound-c-impl-2.md

> **Clarify provenance:** parent **planning** brief was clarified via `/brehon-clarify` before the planning task was queued (DQ #324 INSERT-vs-UPDATE for Task 3 e2e + #325 workaround-comment plan — both resolved before plan write). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). This is an impl-task brief; no clarify-DQ gates it directly — clarify gates planning briefs only.

> **Pre-reservation:** advisor pre-reserved DQ #329 (kind: `validate-pending-laptop`, from: `advisor`) in `.claude/decision-queue.json` `pending[]` per PRECON-7 Option 3 (`.claude/lessons/feedback_cohort_dq_id_collision.md`). You **mutate that existing entry** in place — do NOT write a new DQ entry. See §5 below for the mutation shape.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch matches `junior/<task-slug>-<task-id>` AND it was forked from `phase-v1-federation-inbound-c`. If `git branch --show-current` shows anything else, or if `git merge-base HEAD phase-v1-federation-inbound-c` is empty → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: Shape G is SUSPENDED (validate-pending-laptop mode per DQ #229) — cargo runs on the LAPTOP not this worker, so the forbidden-window cargo concern is reduced; keep the self-check anyway.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-c Task 2 — publish_trust_attestation actor_cap .order_by(valid_from.desc()) fix`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-fed-in-c task 2 — see .claude/PRPs/briefs/v1-federation-inbound-c-impl-2.md
```

## §2 Scope

**Produce** (one commit):

- `crates/apub/activities/src/governance/publish_trust_attestation.rs` — one new line inserted between the existing `.select(governance_config::value_int)` (currently line 145) and `.first::<Option<i64>>(conn)` (currently line 146) of the inline `let actor_cap: i64 = { ... }` block (block starts at currently line 139). The new line is `.order_by(governance_config::valid_from.desc())` with **8-space indentation** matching the surrounding chain (one level deeper than Task 1's site because this is inside an inline block).

Post-edit shape (per plan §10.1 lines 220-222 + sibling pattern):

```rust
let actor_cap: i64 = {
    // existing comment block at 135-138 stays
    governance_config::table
        .filter(governance_config::scope.eq(...))
        .filter(governance_config::key.eq(...))
        .select(governance_config::value_int)
        .order_by(governance_config::valid_from.desc())   // NEW (8-space indent)
        .first::<Option<i64>>(conn)
        .await
        // existing error-mapping + null-handling stays
};
```

`git diff --stat` MUST show: `1 file changed, 1 insertion(+), 0 deletions(-)`.

**Do NOT** in this task:

- Edit the helper-duplication comment at lines 135-138 (it remains accurate post-fix — duplication is intentional per PRECON-3; it documents the circular-dep avoidance `lemmy_api -> lemmy_apub -> lemmy_apub_activities`).
- Edit the `subject_url` extraction at lines 123-133 (outside the `actor_cap` block; unrelated).
- Touch the rate-counter increment at lines 154-166 (Gate 5; unrelated).
- Extract a shared helper (PRECON-3 — see Task 1 brief's same gotcha).
- Touch `crates/apub/activities/src/governance/inbox.rs` (that's Task 1 — different worker, parallel cohort, zero overlap).
- Touch `crates/server/tests/e2e.rs` (Task 3, requires Tasks 1+2 merged).
- Introduce `governance_config_current` view usage (the view is not registered in `crates/db_schema_file/src/schema.rs`). Verify: `grep -c governance_config_current crates/db_schema_file/src/schema.rs` → 0.

**Commit message** (exactly): `feat(fed-in-c): add .order_by(valid_from.desc()) in publish_trust_attestation actor_cap (task 2)`

**HANDOVER trailer in commit body** (per `feedback_handover_trailer_cohort_propagation.md`; cohort propagation):

```
HANDOVER:
  filesCreated: []
  filesModified: [crates/apub/activities/src/governance/publish_trust_attestation.rs]
  keyDecisions:
    - "8-space indentation matches surrounding nested chain"
    - "helper-duplication comment at 135-138 stays verbatim (intentional duplication per PRECON-3)"
    - "no view usage; base-table + .order_by"
  notes: "Sibling Task 1 (inbox.rs get_inbound_config_int) mirrors this edit at 4-space depth. No cross-file changes."
```

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #324 (BINDING — INSERT-vs-UPDATE clarify for Task 3 e2e, advisor-resolved; relevant context for the append-history schema invariant), DQ #325 (BINDING — workaround-comment plan, resolved; not a Task 2 edit), DQ #326 (kind: "log" — runlog-deletion RCA, informational). DQ #329 = your reserved validate-pending-laptop stub; mutate it in §5.
2. **Plan §10.1** (lines 163-229 of `.claude/PRPs/plans/v1-federation-inbound-c.plan.md`) — the authoritative MIRROR pattern (`crates/api/api/src/governance/config.rs:740-769`) including the verbatim regression-history comment block; the post-edit shape for **your site** (lines 220-222); and the GOTCHAs that apply to both Task 1 and Task 2.
3. **Plan §13 "Task 2"** (lines 445-480) — step list, action, IMPLEMENT block with exact line cites + indentation rule (8-space, not 4), MIRROR + GOTCHA. **Indentation differs** from Task 1 (8 spaces vs 4 spaces) because the chain is inside an inline block.
4. **MIRROR refs** — (a) `crates/api/api/src/governance/config.rs` lines 740-769 (the canonical sibling) + (b) `crates/apub/activities/src/governance/inbox.rs` lines 421-438 (the **post-Task-1 fix site**, mirror at 4-space; your site mirrors at 8-space depth). `grep -n 'let actor_cap' crates/apub/activities/src/governance/publish_trust_attestation.rs` to find your block's current start line.
5. **Anchor verification** — `grep -n 'let actor_cap: i64' crates/apub/activities/src/governance/publish_trust_attestation.rs` (plan said line 139); `grep -n '.first::<Option<i64>>' crates/apub/activities/src/governance/publish_trust_attestation.rs` (plan said line 146). If real positions differ from plan, the §2 IMPLEMENT lines drifted — proceed with anchor-by-text-match.
6. **Lessons** (per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 2 runs `cargo-clippy.bat ... -D warnings` on the validate step):
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** Lemmy workspace clippy config denies `unwrap`/`expect`/`#[allow]` escape-hatches; even though this is a 1-line insert with no new bindings, the surrounding code is in a `--features full --no-deps -- -D warnings` clippy scope and any incidental whitespace/lint regression must be clean.
   - `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** if a clippy fix is needed, clippy must be re-run; a stale pass is not evidence the fix worked.
   - `.claude/lessons/feedback_features_full_workspace_only.md` — **Why:** §5 validation runs `--workspace --features full`; the surrounding function is compiled under the `full` feature path.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — the §5 validate-pending-laptop commands use `> log 2>&1` redirect for safe exit-code capture).
   - `.claude/lessons/feedback_principles_not_rules.md` — **Why:** if a §2 IMPLEMENT line cite drifted, anchor by surrounding text not line number; do not file a blocker over a 1-line plan-time drift.

## §3a Handover from prior cohort

(none — Cohort 1 is the first impl cohort; Task 0 was a pure read-only probe with no commit and no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-c`. Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-c` directly.
- **One commit.** (Task 2 is single-file 1-line; if clippy fails on first attempt, amend/fixup the worker commit — do NOT split into a second commit.)
- Mid-task DQ visibility: if you raise a NEW `pending` entry (e.g. a blocker), **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility". For the reserved validate-pending-laptop entry at §5, you MUTATE the existing DQ #329 in place — same atomic push discipline.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`. The validate-pending-laptop entry's `answered_by` is `"advisor-laptop"` (advisor mutates it later — leave the field `null` when you mutate `branch`/`commands`/`phase_task`).

### Mutation discipline (DQ #329 — your reserved entry)

The advisor pre-reserved DQ #329 as a `validate-pending-laptop` stub in `pending[]` with `commands: null`, `branch: null`, `phase_task: 2`, `from: "advisor"`. After your worker commit + push, you mutate that entry in place:

- Set `commands` to the §15.1 + §15.2 verbatim command pair (see §5 below).
- Set `branch` to your worker branch name (`git rev-parse --abbrev-ref HEAD`).
- Set `from` to `"impl"` (advisor pre-filled `"advisor"` only to make the reservation; you re-attribute on mutation).
- Set `timestamp` to your mutation time (the original was reservation time).
- Leave `phase_task: 2` unchanged.
- Leave `result`, `log_slice`, `failed_commands`, `answered_by`, `resolved_at` as `null` — the advisor-laptop fills those after running the commands.
- Update `context` to a one-line reference to your commit SHA + worker branch (e.g. `"impl-task #N committed <sha> on <branch>; mutated reserved stub to attach commands"`).
- Leave `question`, `options`, `answer` as-is (advisor-laptop reads them later).

**Single atomic mutation:** read DQ → mutate the single entry id=329 in `pending[]` → write back (UTF-8, ensure_ascii=False, indent=2) → `git add .claude/decision-queue.json && git commit && git push origin <worker-branch>`. Commit subject: `chore(decision-queue): impl mutated reserved DQ #329 — validate-pending-laptop commands attached (task 2)`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` (or mutate DQ #329) and the Claude Code sensitive-file gate blocks it: (a) write the intended DQ-entry JSON object to a worktree-root file `TASK2_VALIDATE_PENDING.json` or `TASK2_BLOCKER_DQ.json`, (b) write a short `TASK2_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`.

### Task-2 GOTCHAs (from plan §13 Task 2 — load-bearing)

- **Indentation is 8 spaces.** The chain inside the inline `let actor_cap: i64 = { ... }` block uses 8-space indentation (one level deeper than Task 1's 4-space site). Task 1 sibling worker uses 4-space — that's NOT your concern. Mirror the surrounding chain's indentation by reading the live file.
- **Anchor by text, not line number.** Plan said line 145/146; if `grep -n` shows different lines, anchor by the actual text (`.select(governance_config::value_int)` immediately followed by `.first::<Option<i64>>(conn)` inside the `actor_cap` block).
- **Helper-duplication comment at lines 135-138 stays verbatim.** It documents the intentional duplication (PRECON-3) on circular-dep grounds. Do not edit or remove.
- **No view usage.** `governance_config_current` is intentionally absent from the Diesel schema. Verify with `grep -c governance_config_current crates/db_schema_file/src/schema.rs` → 0 before editing; if non-zero, the schema has drifted and you should file a `kind: "blocker"` DQ.
- **No shared helper.** The duplication with Task 1's site is intentional (PRECON-3). Do not refactor.
- **Surrounding lines stay unchanged.** Plan §10.1 lists every preserved line (the filter chain, error mapping, null-handling). Your single insertion is between `.select(...)` and `.first(...)`; everything else stays.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229). After committing + pushing your worker branch, mutate the pre-reserved DQ #329 entry's `commands[]` to **verbatim**:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-c-task2-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task2-clippy.log 2>&1"
```

Do **NOT** write `kind: "validate-pending"` (Shape G suspended). Do **NOT** capture a `workflow_run_id` (no GitHub Actions run). The advisor laptop session reads the mutated entry, runs both commands locally, and mutates the entry to `result: pass|fail` + `answered_by: "advisor-laptop"`.

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK2_VALIDATE_PENDING.json` at worktree root.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §2 edit + mutate the reserved DQ #329 with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 2 complete — v1-federation-inbound-c publish_trust_attestation actor_cap append-history fix

**Commit:** <sha> on <worker-branch>
**Files changed:**
  - crates/apub/activities/src/governance/publish_trust_attestation.rs (+1, -0)
**Anchor verification:** let actor_cap: i64 = block found at line <N> (plan said ~139); .first::<Option<i64>>(conn) at line <M> (plan said ~146); 8-space indentation confirmed
**diff stat:** 1 file changed, 1 insertion(+), 0 deletions(-)
**DQ #329 mutation:** commands[] attached, branch=<worker-branch>, from=impl; reserved stub now ready for advisor-laptop §15 run
**Next:** advisor-laptop runs §15.1 + §15.2 cargo, mutates DQ #329 to result: pass|fail; Cohort 1 barrier waits on Task 1 (DQ #328 sibling); Task 3 then dispatches after both finalize-merged + both result: pass.
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

It does not — Task 2's scope is exactly plan §10.1 + §13 Task 2. This brief adds only: (a) §0 forbidden-window self-check wording, (b) §4 harness-gap interim escalation path + the explicit DQ #329 mutation discipline (NEW — first sub-phase using PRECON-7 Option 3 pre-reservation), (c) §5 explicit `validate-pending-laptop` shape per PRECON-3 (Shape G suspended), (d) explicit "anchor by text not line number" reminder + indentation reminder (8 vs 4). The 1-line insert is §10.1 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-impl-1.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate.

---

_Brief author: advisor session (lane CWD `C:/Users/barri/Developer/brehon-fork-fed-in-c` on `phase-v1-federation-inbound-c` @ `aa3c77ab5`). Brief committed on the phase branch BEFORE the Junior Task-2 task is queued so the worker — which branches from the PHASE branch — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-c` SYNC before queue._
