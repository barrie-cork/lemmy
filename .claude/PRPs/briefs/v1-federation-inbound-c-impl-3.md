---
phase: v1-federation-inbound-c
role: impl-task
task: 3
brief_n: 3
authored: 2026-05-21
plan: .claude/PRPs/plans/v1-federation-inbound-c.plan.md
plan_task: "§13 Task 3 — E2e regression test in `mod v1_federation_inbound_b_fixtures` + workaround-comment update"
parent_phase_tip: 2b55b48dd (phase-v1-federation-inbound-c @ DQ #339 pre-reserve, origin tip)
cohort: "Cohort 2 (Task 3 alone — single-member; requires Tasks 1+2 merged per plan §13 requires:)"
related_dq: "324 (advisor INSERT-vs-UPDATE clarify, resolved — picks INSERT pattern for this test); 325 (workaround-comment plan, resolved — Step (b) implements); 328 (Task 1 validate-pending-laptop, resolved pass); 329 (Task 2 validate-pending-laptop, resolved pass); 338 (DAEMON BUG on gov-v0 — wrong-ref reset, drives §4 pre-push mandate); 339 (BINDING — reserved validate-pending-laptop stub for THIS task)"
reserved_validate_dq: 339
---

# [role:impl-task] v1-federation-inbound-c Task 3 — e2e regression for append-history override + workaround-comment update — see .claude/PRPs/briefs/v1-federation-inbound-c-impl-3.md

> **Cohort context:** Task 3 is the SOLE member of Cohort 2 — it has `requires: - task: 1` AND `requires: - task: 2` per plan §13 lines 492-497. Tasks 1+2 already finalize-merged onto `phase-v1-federation-inbound-c` (commits `bd0af0478` Task 1, `564614559` Task 2, validated locally per DQ #328+#329 both `result: pass`). Forward-merge of `governance-v0` (108 commits, brehon-conformance-audit + rls-r1) landed `7f02254fe`; post-merge cargo-check + clippy `-D warnings` both passed at `7bc103421`. Task 3 is unblocked.
>
> **Pre-reservation:** advisor pre-reserved DQ #339 (kind: `validate-pending-laptop`, from: `advisor`) in `.claude/decision-queue.json` `pending[]` per PRECON-7 Option 3 (`.claude/lessons/feedback_cohort_dq_id_collision.md`). You **mutate that existing entry** in place — do NOT write a new DQ entry. See §5 below for the mutation shape.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch matches `junior/<task-slug>-<task-id>` AND it was forked from `phase-v1-federation-inbound-c` at tip `2b55b48dd` (or any descendant — daemon may have pulled origin between worker-spawn and your read). If `git branch --show-current` shows anything else, or if `git merge-base HEAD phase-v1-federation-inbound-c` is empty → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — if inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) exit non-zero with `FORBIDDEN_WINDOW: <window>`. NOTE: Shape G is SUSPENDED (validate-pending-laptop mode per DQ #229) — cargo runs on the LAPTOP not this worker, so the forbidden-window cargo concern is reduced; keep the self-check anyway.
- **Daemon-bug awareness (DQ #338, gov-v0):** the Junior daemon recently (2026-05-21T20:16Z) issued a wrong-ref reset on `governance-v0` after a planning task's finalize-merge — daemon-local `governance-v0` was reset to `origin/phase-v1-federation-inbound-c`, destroying the plan commit. **Mitigation in §4:** worker pre-pushes its commit to the worker branch BEFORE finalize runs; advisor manually finalize-merges from `origin/junior/<branch>` (the cohort-1 pattern). Do NOT rely on daemon finalize-merge.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-c Task 3 — e2e regression for append-history override + workaround-comment update`

The actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-fed-in-c task 3 — see .claude/PRPs/briefs/v1-federation-inbound-c-impl-3.md
```

## §2 Scope

**Produce** (one commit):

- `crates/server/tests/e2e.rs` — TWO Edit operations inside the existing `mod v1_federation_inbound_b_fixtures`:

  **Step (a)** — Insert one new `#[tokio::test(flavor = "multi_thread")]` between the closing brace of `per_peer_rate_limit_returns_429` (currently line 15628) and the `#[tokio::test(flavor = "multi_thread")]` attribute of `replayed_activity_returns_409` (currently line 15630). The test fn exercises the post-fix append-history override behaviour via `federation.inbound.per_peer_rate_per_hour`. Test body skeleton (per plan §13 Task 3 lines 503-545); you pick the exact fn name from the candidate list in §4 GOTCHAs and use it consistently in Step (b)'s comment:

  ```rust
    #[tokio::test(flavor = "multi_thread")]
    async fn <name>() -> LemmyResult<()> {
      let (_container, fed_cfg, db_url, _peer_id) =
        bootstrap_with_peer("override-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
      let context = fed_cfg.to_request_data();
      let mut conn = AsyncPgConnection::establish(&db_url).await?;
      // governance_config is append-only — INSERT a newer-valid_from row to
      // override the seeded federation.inbound.per_peer_rate_per_hour (seed
      // value_int = 100 per migration 2026-04-18-000000-0000_add_governance_config).
      // Post-v1-federation-inbound-c, get_inbound_config_int reads the latest row;
      // the override cap=2 means the 3rd activity should 429.
      diesel::insert_into(governance_config::table)
        .values((
          governance_config::scope.eq("instance"),
          governance_config::key.eq("federation.inbound.per_peer_rate_per_hour"),
          governance_config::value_type.eq("int"),
          governance_config::value_int.eq(Some(2_i64)),
          governance_config::valid_from.eq(diesel::dsl::now),
        ))
        .execute(&mut conn)
        .await?;
      for i in 0..2 {
        let activity = build_unique_sanction_notice_activity("override-test.test", i)?;
        ActivityTrait::receive(activity, &context).await?;
      }
      let activity3 = build_unique_sanction_notice_activity("override-test.test", 2)?;
      let result = ActivityTrait::receive(activity3, &context).await;
      assert!(result.is_err(), "3rd activity must 429 against override cap=2");
      let err = result.err().unwrap();
      assert!(matches!(err.error_type, LemmyErrorType::FederationPeerRateLimitExceeded));
      assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
      // Optional but recommended: assert the drop log row landed.
      let drop_rows: i64 = federation_inbox_dropped_log::table
        .filter(federation_inbox_dropped_log::source_instance.eq("override-test.test"))
        .filter(federation_inbox_dropped_log::drop_reason.eq("rate_limit_peer"))
        .count()
        .get_result(&mut conn)
        .await?;
      assert_eq!(drop_rows, 1, "exactly one rate_limit_peer drop expected");
      Ok(())
    }
  ```

  **Step (b)** — Replace the pre-fix workaround comment at currently-lines 15605-15610 of `crates/server/tests/e2e.rs` (inside the existing `per_peer_rate_limit_returns_429` test) per DQ #325. Pre-fix comment (to remove):

  ```rust
      // governance_config is append-history with UNIQUE on (scope, key, valid_from)
      // — NOT on (scope, key). The migration 2026-05-17 already seeded this key
      // with value_int=100; raw INSERT would create a second row and the reader
      // (get_inbound_config_int) returns an arbitrary one. UPDATE mutates the
      // existing seed row in place. See migration 2026-04-18 comment "Do NOT use
      // (scope, key) as the conflict target" for the schema invariant.
  ```

  Post-fix comment (to insert in its place — substitute `<new-test-fn-name>` with the name you picked in Step (a)):

  ```rust
      // governance_config is append-history: UPDATE mutates the existing seed row;
      // the post-v1-federation-inbound-c reader (.order_by(valid_from.desc())) makes
      // INSERT-with-newer-valid_from also safe. This test uses UPDATE for historical
      // continuity; new override tests use INSERT (see <new-test-fn-name>).
  ```

  The existing `diesel::sql_query("UPDATE governance_config SET value_int = 2 ...")` body (currently lines 15611-15616 pre-comment-edit) stays UNCHANGED — UPDATE still works post-fix.

`git diff --stat` MUST show: `1 file changed, ~30 insertions(+), ~6 deletions(-)` (approximate — Step (a) inserts ~33 lines, Step (b) is a 6-line comment swap; total net insertion ~30).

**Do NOT** in this task:

- Touch any other test in `mod v1_federation_inbound_b_fixtures` (blocklisted_peer_returns_403, replayed_activity_returns_409, allowlisted_happy_path_persists_advisory_row, moderation_label_handler_persists_and_logs — all stay verbatim).
- Touch the sibling `mod v1_federation_inbound_a_fixtures` or any other mod in `e2e.rs`.
- Touch `crates/apub/activities/src/governance/inbox.rs` or `publish_trust_attestation.rs` (Tasks 1+2 already merged; their edits are the load-bearing reader-side fix that makes Step (a)'s assertion deterministic).
- Touch the test fn name once chosen — Step (b)'s comment must reference the exact name from Step (a).
- Introduce `conn.run_transaction(...)` wrapping the INSERT — the override INSERT is a single statement; `feedback_multi_write_handlers_need_transactions.md` does not fire here.
- Introduce `i32 as i64` casts — `2_i64` literal and `0..2` u32 loop both fit cleanly without casts.

**Commit message** (exactly): `feat(fed-in-c): add e2e regression for append-history override + update workaround comment (task 3)`

**HANDOVER trailer in commit body** (per `feedback_handover_trailer_cohort_propagation.md`; cohort propagation):

```
HANDOVER:
  filesCreated: []
  filesModified: [crates/server/tests/e2e.rs]
  keyDecisions:
    - "test fn name: <chosen-name>"
    - "INSERT-with-newer-valid_from pattern (per DQ #324) — proves Task 1's .order_by(valid_from.desc()) makes append-history overrides effective"
    - "Step (b) post-fix comment cross-references the new test fn name"
  notes: "Cohort 2 single-member; final pre-retro commit on phase-v1-federation-inbound-c. Phase 2 e2e (user gate 4) follows on the post-merge phase tip."
```

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #324 (BINDING — INSERT-vs-UPDATE clarify, advisor-resolved: INSERT-with-newer-valid_from is the pattern for Step (a)); DQ #325 (BINDING — workaround-comment plan, advisor-resolved: Step (b) follows option `update-comment-in-task-3`); DQ #326 (kind: "log" — runlog-deletion RCA, informational); DQ #328 + DQ #329 (Task 1+2 validate-pending-laptop, both `result: pass`); DQ #338 (BINDING context — daemon bug drives §4 pre-push mandate); DQ #339 = your reserved validate-pending-laptop stub; mutate it in §5.
2. **Plan §13 "Task 3"** (lines 482-609 of `.claude/PRPs/plans/v1-federation-inbound-c.plan.md`) — full step (a) + step (b) spec including the verbatim skeleton, comment replacement target, GOTCHAs, and validation block.
3. **MIRROR — canonical sibling** — `crates/server/tests/e2e.rs:15600-15628` (`per_peer_rate_limit_returns_429`, the canonical sibling shape inside `mod v1_federation_inbound_b_fixtures`). Mirror its error-shape (Case A `LemmyResult<()>`), connection acquisition (`AsyncPgConnection::establish(&db_url).await?`), bootstrap call (`bootstrap_with_peer`), activity-send loop, assertion grammar, drop-log assertion pattern. **Canonical-schema-first gate (per `.claude/rules/advisor-orchestrator.md` §3.6):** read this sibling verbatim BEFORE authoring your Edit; the surrounding text + indentation is the contract. Sibling fn signature is `LemmyResult<()>` with bare `?` propagation — Case A enum per `feedback_lemmy_error_no_std_error.md`. NO `Box<dyn Error>` bridges. NO `.map_err(|e| format!("{e}").into())?` annotation closures.
4. **Anchor verification** — `grep -n 'per_peer_rate_limit_returns_429' crates/server/tests/e2e.rs` (plan said line 15600); `grep -n 'replayed_activity_returns_409' crates/server/tests/e2e.rs` (plan said line 15630). Step (a) insertion anchor = between closing `}` of `per_peer_rate_limit_returns_429` and `#[tokio::test(flavor = "multi_thread")]` attribute of `replayed_activity_returns_409`. Step (b) anchor = the `// governance_config is append-history with UNIQUE on (scope, key, valid_from)` comment line inside `per_peer_rate_limit_returns_429`. If real positions differ from plan, anchor by text (the surrounding text is the contract; line numbers are approximations).
5. **Brehon-conformance-audit skill** (NEW — per `.claude/rules/advisor-orchestrator.md` §3.1.1): `crates/server/tests/e2e.rs` is NOT in the audit's tracked file patterns (audit targets `crates/apub/activities/src/governance/**.rs`, `crates/api/api/src/governance/**.rs`, `crates/db_schema/src/source/governance/**.rs`). Audit invocation NOT required for this task — but if your Edit accidentally touches any of those three paths, STOP and file a `kind: "blocker"` DQ — that would be out of scope.
6. **Lessons** (per `.claude/rules/advisor-orchestrator.md` §2.4 file-class table — Task 3 edits `crates/server/tests/e2e.rs`, mandatory injection per row 1; sibling fixtures module exists per Case A mirror rule; this is ≥2 edits per row 2):
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** sibling `mod v1_federation_inbound_b_fixtures` uses Case A (`LemmyResult<()>` outer, bare `?`, helpers also `LemmyResult<T>`). Mirror it verbatim — your test fn signature is `LemmyResult<()>`, `?` propagates without `.map_err` bridges. Read the Case A sub-section before writing.
   - `.claude/lessons/feedback_async_pool_test_pattern.md` — **Why:** sibling uses `AsyncPgConnection::establish(&db_url).await?` (already imported at line 15517, no new use needed). Do NOT introduce `Pool` / `DbPool::Conn` patterns.
   - `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **Why:** `e2e.rs` is 15773 lines; never full-file Edit. Your two Edit operations sit at specific anchors (Step (a) ~33-line insert + Step (b) 6-line comment swap = ≤200-line edit budget per plan §13 Task 3 GOTCHA). Use `Edit` with sufficient surrounding context to disambiguate; if `old_string` isn't unique, increase context — never use `replace_all` on a 15k-line file.
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** the Lemmy workspace clippy config denies `unwrap`/`expect`/`#[allow]` escape-hatches in test code. The sibling uses `result.err().unwrap()` (line 15624) — that's pre-existing test code and clippy allows it inside `#[tokio::test]` fns; your new test fn mirrors the sibling so the same allowance applies. No NEW `unwrap`/`expect` outside the sibling pattern.
   - `.claude/lessons/feedback_features_full_workspace_only.md` — **Why:** §5 validation runs `--workspace --features full --no-deps -- -D warnings`. Your Edit is inside the e2e harness which is compiled under `--features full`.
   - `.claude/lessons/feedback_principles_not_rules.md` — **Why:** if a §2 IMPLEMENT line cite drifted, anchor by surrounding text not line number; do not file a blocker over a plan-time line drift.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — the §5 validate-pending-laptop commands use `> log 2>&1` redirect for safe exit-code capture).

## §3a Handover from prior cohort

From Cohort 1 (Tasks 1+2, both `complete` + validate-pending-laptop `result: pass` + finalize-merged):

```yaml
prior_cohort_tasks:
  - task: 1
    commit: bd0af0478
    branch: junior/role-impl-task-v1-fed-in-c-task-1-see-claude-prps-briefs-v1-federation-inbound-c-impl-1-md-397
    filesCreated: []
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "4-space indentation matches surrounding chain"
      - "no view usage; base-table + .order_by"
    notes: "Single-line insert: .order_by(governance_config::valid_from.desc()) between .select(...) and .first::<Option<i64>>(...). Validated locally via §15.1 cargo-check + §15.2 clippy -D warnings (DQ #328 result: pass)."
  - task: 2
    commit: 564614559
    branch: junior/role-impl-task-v1-fed-in-c-task-2-see-claude-prps-briefs-v1-federation-inbound-c-impl-2-md-398
    filesCreated: []
    filesModified: [crates/apub/activities/src/governance/publish_trust_attestation.rs]
    keyDecisions:
      - "8-space indentation matches nested-block chain depth (NOT 4-space like Task 1)"
      - "actor_cap precondition site only; main publish path stays unchanged"
    notes: "Single-line insert: .order_by(governance_config::valid_from.desc()) at the actor_cap precondition query. Validated locally via §15.1 cargo-check + §15.2 clippy -D warnings (DQ #329 result: pass)."
```

**Forward-merge context:** post-cohort-1 forward-merge of `governance-v0` (108 commits, brehon-conformance-audit + rls-r1) landed `7f02254fe` on phase. Notable additions affecting `e2e.rs`: NONE (the merge added skill files, conformance-audit module-roots in `crates/apub/activities/src/governance/mod.rs` + `crates/api/api/src/governance/mod.rs` + `crates/db_schema/src/source/governance/mod.rs`, and `clippy.toml` workspace-disallow config — none touch `e2e.rs`). Post-merge cargo-check + clippy `-D warnings` both passed (commits `7bc103421`).

**Cohort barrier impact:** your Step (a)'s assertion (3rd activity returns 429 under cap=2 INSERTed via newer-`valid_from`) is **deterministically green only because Task 1 landed**. Without Task 1's `.order_by(valid_from.desc())`, `get_inbound_config_int` would non-deterministically return the seeded value_int=100 or the INSERTed 2 → assertion would flake. Task 2's edit is unrelated to this test's path but the cohort barrier ensured both were merged before Task 3 dispatched.

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-c` (tip `2b55b48dd` or descendant). Forked at task-spawn time.
- **One commit.** (Step (a) + Step (b) ship together; if clippy fails on first attempt, amend/fixup the worker commit — do NOT split into a second commit. The single-commit-per-task invariant feeds the cohort handover trailer mechanism.)
- **CRITICAL — DAEMON-BUG PRE-PUSH MANDATE (per DQ #338 on gov-v0, 2026-05-21T20:16Z):** the Junior daemon's finalize-merge code path is known-buggy as of this task — daemon may issue a wrong-ref reset on a trunk branch after merge. The structural fix is in another advisor session's investigation queue. **To avoid the bug:** after your commit, run `git push origin HEAD:$(git branch --show-current)` to push your worker branch to `origin/junior/...` BEFORE the daemon's finalize step runs. The advisor laptop session will manually finalize-merge from `origin/junior/<branch>` (the cohort-1 pattern per `feedback_junior_finalize_skips_when_worker_pre_pushes.md`). **Do NOT push to `phase-v1-federation-inbound-c` directly** — finalize/advisor handles the merge into the phase branch.
- Mid-task DQ visibility: if you raise a NEW `pending` entry (e.g. a blocker), **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility". For the reserved validate-pending-laptop entry at §5, you MUTATE the existing DQ #339 in place — same atomic push discipline.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`. The validate-pending-laptop entry's `answered_by` is `"advisor-laptop"` (advisor mutates it later — leave the field `null` when you mutate `branch`/`commands`/`phase_task`).

### Mutation discipline (DQ #339 — your reserved entry)

The advisor pre-reserved DQ #339 as a `validate-pending-laptop` stub in `pending[]` with `commands: null`, `branch: null`, `phase_task: 3`, `from: "advisor"`. After your worker commit + push, you mutate that entry in place:

- Set `commands` to the §15.1 + §15.2 + §15.3 verbatim command triplet (see §5 below — Task 3 adds `cargo test --test e2e --no-run` because the harness must re-link after the test fn insertion).
- Set `branch` to your worker branch name (`git rev-parse --abbrev-ref HEAD`).
- Set `from` to `"impl"` (advisor pre-filled `"advisor"` only to make the reservation; you re-attribute on mutation).
- Set `timestamp` to your mutation time (the original was reservation time).
- Leave `phase_task: 3` unchanged.
- Leave `result`, `log_slice`, `failed_commands`, `answered_by`, `resolved_at` as `null` — the advisor-laptop fills those after running the commands.
- Update `context` to a one-line reference to your commit SHA + worker branch (e.g. `"impl-task #N committed <sha> on <branch>; mutated reserved stub to attach commands"`).
- Leave `question`, `options`, `answer` as-is (advisor-laptop reads them later).

**Single atomic mutation:** read DQ → mutate the single entry id=339 in `pending[]` → write back (UTF-8, ensure_ascii=False, indent=2) → `git add .claude/decision-queue.json && git commit && git push origin <worker-branch>`. Commit subject: `chore(decision-queue): impl mutated reserved DQ #339 — validate-pending-laptop commands attached (task 3)`.

### Harness-gap note (per DQ #235 — interim escalation-and-transcribe)

If you need to write a DQ entry to `.claude/decision-queue.json` (or mutate DQ #339) and the Claude Code sensitive-file gate blocks it: (a) write the intended DQ-entry JSON object to a worktree-root file `TASK3_VALIDATE_PENDING.json` or `TASK3_BLOCKER_DQ.json`, (b) write a short `TASK3_ESCALATION.md` naming the issue, (c) commit both at worktree root + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`.

### Task-3 GOTCHAs (from plan §13 Task 3 — load-bearing)

- **Edit budget ≤200 lines insertion** within the sibling module. Never full-file Edit on `e2e.rs` (15773 lines). Your two Edit operations (Step (a) new test fn ~33 lines, Step (b) comment swap ~6 lines net) sit well below budget. Use `Edit` with sufficient surrounding context to disambiguate; `replace_all` is FORBIDDEN on this file.
- **Error shape Case A** per `feedback_lemmy_error_no_std_error.md` — `LemmyResult<()>` outer, bare `?` propagation, NO `Box<dyn Error>` bridges, NO `.map_err(|e| format!("{e}").into())?` annotation closures. Mirror sibling `per_peer_rate_limit_returns_429` verbatim.
- **Connection acquisition** via `AsyncPgConnection::establish(&db_url).await?` per `feedback_async_pool_test_pattern.md`. The sibling module already imports `diesel_async::AsyncPgConnection` at line 15517; no new `use` statement needed.
- **No `conn.run_transaction(...)` wrapper** — the override INSERT is a single statement; activity sends are sequential and each commits its own state. `feedback_multi_write_handlers_need_transactions.md` does not fire here.
- **Test fn name discipline:** the planner did not pick the identifier (canonical-schema-first gate is your responsibility). Recommended candidates (pick one): `appended_config_override_takes_effect_returns_429`, `latest_value_from_wins_over_seeded_baseline`, `per_peer_rate_override_via_append_history_returns_429`. Whatever name you pick updates the workaround-comment reference in Step (b) accordingly. **Name once, use twice** (in Step (a) fn declaration + Step (b) comment).
- **R1 (clippy_test_style):** no `i32 as i64` casts. The `2_i64` literal and `0..2` u32 loop counter both fit cleanly without casts; if you introduce an `i64::from(...)` for any count comparison, that's the canonical form.
- **R6:** clippy invocation at validate uses `--workspace --features full --no-deps -- -D warnings` (per §5.2).
- **Hour-bucket assumption** — the test sends 3 activities synchronously; they will all fall into the same `current_hour_bucket()`. The sibling test `per_peer_rate_limit_returns_429` already relies on this; the new test inherits.
- **Step (b) anchor** — the pre-fix comment to remove is INSIDE the existing `per_peer_rate_limit_returns_429` test fn (currently lines 15605-15610), NOT a top-of-mod or top-of-file comment. The `diesel::sql_query("UPDATE governance_config SET value_int = 2 ...")` body that follows (currently lines 15611-15616) stays UNCHANGED — UPDATE still works post-fix.
- **No new `use` statements needed** — sibling mod already imports all required types: `governance_config` (via `lemmy_db_schema_file::schema::*` — verify with `grep -n governance_config crates/server/tests/e2e.rs` near line 15528), `federation_inbox_dropped_log` (line 15529), `LemmyErrorType` (line 15538), `StatusCode` (line 15514), `AsyncPgConnection` (line 15517), `ExpressionMethods` + `QueryDsl` + `RunQueryDsl` (lines 15516-15517), `FederationPeerTrust` (line 15527), `build_unique_sanction_notice_activity` (parent `mod` helper). If `governance_config` is NOT in the existing import list, add `governance_config` to the `use lemmy_db_schema_file::schema::{...}` block (lines 15528-15536) — verify with `grep -n governance_config crates/server/tests/e2e.rs` first.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

**Shape-G suspended until 2026-06-01** (DQ #229). After committing + pushing your worker branch, mutate the pre-reserved DQ #339 entry's `commands[]` to **verbatim**:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-c-task3-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-c-task3-clippy.log 2>&1"
cmd //c "scripts\brehon\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-c-task3-test-norun.log 2>&1"
```

Note the THIRD command — Task 3 is a test fn insertion, so the harness must re-link to confirm the new fn compiles into the e2e binary. `--no-run` builds the test binary without running it (Phase 2 e2e is a separate later step via user gate 4 — local vs dispatch).

Do **NOT** write `kind: "validate-pending"` (Shape G suspended). Do **NOT** capture a `workflow_run_id` (no GitHub Actions run). The advisor laptop session reads the mutated entry, runs all three commands locally, and mutates the entry to `result: pass|fail` + `answered_by: "advisor-laptop"`.

If the `.claude/decision-queue.json` write is gated by the sensitive-file gate, use the §4 harness-gap escalation path with `TASK3_VALIDATE_PENDING.json` at worktree root.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself (the laptop advisor does) — your job is only to apply the §2 Edits + mutate the reserved DQ #339 with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 3 complete — v1-federation-inbound-c e2e regression for append-history override + workaround-comment update

**Commit:** <sha> on <worker-branch>
**Files changed:**
  - crates/server/tests/e2e.rs (+~33, -~6)
**Test fn name chosen:** <name>
**Anchor verification:**
  - per_peer_rate_limit_returns_429 found at line <N> (plan said ~15600)
  - replayed_activity_returns_409 found at line <M> (plan said ~15630)
  - Step (a) insert position: between lines <N+28> and <M-1>
  - Step (b) anchor (pre-fix comment) at lines <K1>-<K2> (plan said 15605-15610)
**Diff stat:** 1 file changed, ~30 insertions(+), ~6 deletions(-) (Step (a) ~33 lines + Step (b) net -3)
**DQ #339 mutation:** commands[] (3 cmds) attached, branch=<worker-branch>, from=impl; reserved stub now ready for advisor-laptop §15 run
**Worker branch pushed:** YES (DQ #338 daemon-bug mitigation per §4)
**Next:** advisor-laptop runs §5.1 + §5.2 + §5.3 cargo, mutates DQ #339 to result: pass|fail; advisor manually finalize-merges from origin/<worker-branch>; Phase 2 e2e (user gate 4) on post-merge tip; Task 4 retro authoring.
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

It does not — Task 3's scope is exactly plan §13 Task 3 (lines 482-609). This brief adds only:

(a) §0 forbidden-window self-check wording + DAEMON-BUG awareness (per DQ #338 on gov-v0)
(b) §4 PRE-PUSH MANDATE (per DQ #338 daemon wrong-ref reset bug — explicit `git push origin HEAD:$(git branch --show-current)` after commit, advisor manually finalize-merges) + harness-gap interim escalation path + explicit DQ #339 mutation discipline
(c) §5 explicit `validate-pending-laptop` shape per PRECON-3 (Shape G suspended) with THREE commands (test --no-run added for harness re-link)
(d) §3 invokes brehon-conformance-audit skill check (per advisor-orchestrator.md §3.1.1) — concludes not required for this file (e2e.rs is not in audit's tracked patterns) but documents the check
(e) §3 + §3a explicit cohort-1 handover summary + forward-merge context (Tasks 1+2 fixes + 108-commit gov-v0 merge)
(f) §3 canonical-sibling MIRROR ref (per advisor-orchestrator.md §3.6 canonical-schema-first gate) anchoring on `per_peer_rate_limit_returns_429`

Step (a) test body + Step (b) comment swap are §13 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-c-impl-1.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate.

---
