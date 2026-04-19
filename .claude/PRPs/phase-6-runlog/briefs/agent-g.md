# Agent G — Phase 6 Layer 5: Federation round-trip e2e test

**Model:** opus. **Effort:** maximum — deepest reasoning, most thorough exploration.

**Scope:** Task 77 of `.claude/PRPs/plans/phase-6-federation.plan.md`. Add `sanction_notice_round_trip` two-Postgres e2e test to `crates/server/tests/e2e.rs`. Update `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` with four new federation entry kinds.

You are continuing a Brehon governance-fork Phase 6 layered-execution plan. Read `CLAUDE.md`, plan task 77 and pattern `TWO_DB_TEST_PATTERN`, and `.claude/decision-queue.json` DQ-6.4 (leave env-var cleanup as-is). All prior layers shipped.

## Worktree

- Advisor has created `../brehon-fork-agent-g-phase6` on branch `agent-g-phase6` cut from `phase-6` tip (post all prior agents).
- **First commands:** `cd ../brehon-fork-agent-g-phase6 && git log --oneline -10 && git submodule update --init --recursive`. All Phase 6 commits should be visible in the log.

## Task-hopper envelope

```
scripts/brehon/task-hopper.sh start 77 \
  --agent agent-g --kind cargo_test --layer 5 \
  --label "sanction_notice_round_trip two-DB e2e" \
  --worktree "$(pwd)"
```

Note: `cargo_test` has retry cap 2, not 3. Two attempts max before auto-escalation.

## Task 77 — Two-database e2e

Add `#[tokio::test(flavor = "multi_thread")] async fn sanction_notice_round_trip()` to `crates/server/tests/e2e.rs`.

### Outline per plan `TWO_DB_TEST_PATTERN`

1. Boot container A via `governance_fixtures::start_postgres()`, apply schema, build `context_a`.
2. Boot container B, apply schema, build `context_b`.
3. On instance A: seed `Instance`, admin `Person` (via `seed_person`), target `Person`, open a `ModerationCase` directly via `ModerationCase::create` (bypass `create_report` for test speed). Assign a 5-juror panel via `admin_assign_jury`. Accept all 5 (`accept_jury_assignment` loop). Submit 3 `RecommendFederationAction` votes to trip quorum.
4. After `submit_jury_vote`'s post-decision block fires, assert instance A's `sent_activity` table has exactly one row with `activity.type == "PublishSanctionNotice"`.
5. Deserialise `sent_activity.data` (JSONB) into a `PublishSanctionNotice` struct.
6. Call `PublishSanctionNotice::receive(activity, &context_b).await?` **directly** — no HTTP (per IMPLEMENTATION-PLAN-v0.md §3 Phase 6 task 77: "Do not require a real HTTP federation transport for this test — call the inbox function directly").
7. Assert instance B has exactly one `remote_sanction_notice` row with:
   - `local_case_id IS NULL`
   - `action = SanctionAction::FederationQuarantineRecommendation`
   - `scope = SanctionScope::FederatedRecommendation`
   - `target_url` = target person's `ap_id.to_string()` from A
   - `source_instance = "instance-a.test"` (or whatever instance hostname A is seeded with)
   - `summary` non-empty and redacted (no usernames/emails/URLs from A's rationale)
8. Assert instance B's `governance_log` has exactly one `federation_sanction_received` entry for this notice.
9. Containers auto-drop at test end (owned locals).

### Also update

`docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` — append four new entry kinds to §Entry kinds (non-breaking extension per §Stability contract):
- `federation_sanction_sent`
- `federation_sanction_received`
- `federation_attestation_sent`
- `federation_attestation_received`

Single commit for the test + doc update.

### Critical patterns (per plan §TWO_DB_TEST_PATTERN)

- **`LEMMY_DATABASE_URL` env swap** — sequence pool construction strictly. Build pool A fully before swapping to B. Do NOT use `tokio::join!` on pool builds. Sequencing note in-code:
  ```rust
  // Build A's pool+context fully before swapping env to B — the pool reads
  // env at construction and a multi-thread runtime could interleave otherwise.
  ```
- **Env-var cleanup** — DQ-6.4 resolved: leave as-is, mirror existing pattern at e2e.rs:2195+. Document inline.
- **Error bridging** — tests return `Result<(), Box<dyn Error>>`. LemmyError does not implement `std::error::Error`. All handler calls need `.map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?` per existing pattern (see `is_valid_actor_name` bridge in recent commit).
- **Rate-limit** — the test hits ~12 POSTs on instance A (admin_assign_jury + 5×accept + 3×vote + setup). `RateLimit::with_debug_config()` POST bucket is 6/300s per `feedback_rate_limit_debug_config_post_bucket.md`. Either use `set_config` to bump the bucket, or seed the `moderation_case` + `sanction` rows directly via `ModerationCase::create` + `Sanction::create` (bypass handlers) per plan §GOTCHA "Seeding shortcut". The golden-path test in Phase 5c is what proves the handler chain; task 77's job is to prove the **federation** step, so direct seeding is appropriate.
- **Two containers** — cold-boot ~40s. Acceptable for v0.

### Gotchas

- **Seeding shortcut** strongly encouraged. Going `create_report` → `admin_assign_jury` → 5 × accept → 5 × vote is slow. Seed `moderation_case` + `sanction` directly and then call `submit_jury_vote` for the final vote that trips quorum + triggers the federation publish. Same end-to-end behaviour for federation coverage.
- **`target_url` assertion** — the inbound `remote_sanction_notice.target_url` should equal the target person's `ap_id`. Do NOT compare to the local DB person ID.
- **Redaction assertion** — the plan says "assert summary is redacted". Concretely: verify `summary` doesn't contain the admin's username, target's username, or any email/URL patterns. Use a regex check or explicit substring-not-contained asserts.
- **Governance log hash chain** — the existing `governance_log_hash_chain_holds` test should still pass. Your test's writes add to the chain but don't break it; the invariant is append-only.

### Validate

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server sanction_notice_round_trip > .claude/test-task77.log 2>&1"
tail -50 .claude/test-task77.log
echo "test exit: $?"
# Full sweep:
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/test-task77-full.log 2>&1"
tail -30 .claude/test-task77-full.log
echo "full exit: $?"
```

Both 0. `test result: ok. N passed` in the tails.

## Commit

`feat(governance): task 77 — sanction_notice_round_trip e2e` with plan-template body (see plan §task 77 commit template). Push `agent-g-phase6`.

## Catch-fire triggers

- `PublishSanctionNotice::receive` not importable from `e2e.rs` — likely missing `pub` on the Activity newtype; raise DQ and coordinate with Agent C's scope.
- Container boot timeout — governance_fixtures has retry logic; if it still fails, raise DQ.
- Hash-chain test flakes after task 77 merges — investigate; federation writes should not break chain integrity. Likely signature-key env var issue or tx ordering. DQ with evidence.

## When done

Final merge point 4: advisor runs the full e2e suite against phase-6 tip after Agent G merge. If all tests pass, advisor writes phase-6 completion report at `.claude/PRPs/reports/phase-6-complete-report.md` and opens PR `phase-6 → governance-v0`.
