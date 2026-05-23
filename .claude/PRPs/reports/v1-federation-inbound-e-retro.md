# Retrospective — v1-federation-inbound-e

**Authored at:** 2026-05-23T08:55 UTC  
**Phase branch:** `phase-v1-federation-inbound-e` @ `1f0af4f4c`  
**Plan:** `.claude/PRPs/plans/v1-federation-inbound-e.plan.md`  
**Goal:** Per-peer ActivityPub inbox storage cap serialised under concurrent receivers — Race A (over-eviction) + Race B (over-insertion) closed via `pg_advisory_xact_lock`.

---

## §1 Four-role signals

### Advisor

**Conformance-audit prevention checkpoint:** Invoked on `inbox.rs` before Task 1 brief authorship per §3.1.1. No Tier-1 findings were surfaced — the TOCTOU defect is a runtime concurrency pattern, not a static-convention violation. Audit confirmed `acquire_evict_lock` shape mirrors `reputation_snapshot.rs:821-848` correctly.

**Cycle-count meta-rule:** Did not fire. Task 1 passed cargo check on first cycle. Task 2 e2e passed on second cycle (first cycle with the null-byte separator fix in place).

**Missing-brief workflow gap:** The planning Junior was queued for this phase without the brief file being committed first (`bootstrap` served as de-facto brief). This re-surfaced from the `kind: "log"` DQ filed at plan commit. Candidate lesson `feedback_advisor_must_commit_brief_before_planning_dispatch.md` confirmed for promotion.

**Null-byte TEXT separator incident (Task 1 amend):** `acquire_evict_lock` originally used `\x00` as the separator in the advisory-lock key `format!("{peer_domain}\x00{table_name}")`. The Postgres `TEXT` type forbids null bytes; the defect was invisible to `cargo check` + `clippy` (no compile-time constraint) and was caught only when Task 2's e2e bound the key to `pg_advisory_xact_lock` via a TEXT bind. User chose Option 1 (amend-in-place 1-char swap `\x00` → `:`). This surfaces a lesson candidate: `feedback_postgres_text_null_byte_forbidden.md`. Promoted below.

**Daemon stale-ref BM-verb pattern:** Did not recur in this phase.

**Gate-1 concurrency-model DQ:** Resolved cleanly by advisor at plan-approval time — planner's recommendation (`pg_advisory_xact_lock` per `(peer_domain, table_name)`) adopted without escalation.

### Planning

**Pattern §10.1 citation:** `acquire_evict_lock` MIRROR ref (`reputation_snapshot.rs:821-848`) was cited correctly in Task 1's impl-task brief. Worker followed it verbatim for function signature + `diesel::dsl::sql<diesel::sql_types::BigInt>` shape.

**Pattern §10.4 citation:** `v1_federation_inbound_e_fixtures` mirroring `v1_federation_inbound_b_fixtures` was cited correctly in Task 2's impl-task brief. Worker appended module in the correct structural position.

**Gate-1 concurrency-model DQ:** Clean resolution — no escalation, no options explored at plan time beyond the planner's recommendation.

**Canonical-schema-first read at Task 2 brief time:** `build_minimal_sanction_notice_activity` helper symbol identified correctly; Task 2 brief included the verbatim import block.

**No §13 task produced spurious `creates:` entries** — Task 1 + Task 2 both `creates: []` (modifies only), correctly reflecting that `inbox.rs` and `e2e.rs` were pre-existing.

### Impl

**Task 1 complexity:** `1 file / 1 commit (+1 amend) / ~15 min impl runtime / ~2 min max log silence`. Divergence: an amend was required after Task 2's e2e discovered the `\x00` separator defect. Net files: 1. Net commits: 2 (original + amend). Plan expected 1. Amend is a narrow deviation; the pre-push `cargo-check.sh` gate passed on the original commit, confirming it's a runtime-only defect class.

**Task 2 complexity:** `1 file / 1 commit / ~50 min runtime (testcontainer cold-start + multi-thread e2e) / ~5 min max log silence`. Within plan expectation (`1/1/<medium>/<short>`).

**Phase-2 e2e complexity:** 104 tests, 0 failed, 5 ignored, finished in 2229s (~37 min). Pre-existing test counts: `v1_federation_inbound_a_fixtures` 2, `v1_federation_inbound_b_fixtures` 6 — both present and passing. No new failures.

**Advisory-lock callsite enumeration:** `rg "acquire_evict_lock(conn," inbox.rs` returns exactly 3 hits (the 3 callers). Bound matches plan §15.7 PRECON assertion.

### BM

**bm-cut:** Executed cleanly from governance-v0 tip (post-fed-in-d-merge) per bootstrap checklist. Phase branch `phase-v1-federation-inbound-e` cut correctly.

**bm-pr / bm-merge / CR triage:** At time of retro authorship, `bm-pr` has not yet been opened — retro is authored pre-bm-pr per the plan §13 Task 3 ordering. BM signals are therefore anticipated/forward-looking:
- Expected: CodeRabbit may flag `acquire_evict_lock` for lock-acquisition discipline (calling outside tx is undefined). Counter-evidence: the plan §18 risk table notes this + the helper validates the lock is always inside `run_transaction` at all 3 call sites.
- Expected: CR counts this as a non-draft PR auto-reviewed; triage should be straightforward.

---

## §2 Per-task complexity scores

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|------|-------|---------|---------------|----------------------|-------|
| Task 1 | 1 | 2 (orig + null-byte amend) | ~15 | ~2 | Amend class: runtime Postgres TEXT; invisible to compile-time gates |
| Task 2 | 1 | 1 | ~50 | ~5 | Testcontainer cold-start + multi-thread tokio fan-out; deterministic |
| Phase-2 e2e | — | — | ~37 | — | 104 tests; 2229s wallclock |

**Aggregate:** 2 tasks, 2 files modified, 3 commits, ~2h total wall-clock including e2e validation.

---

## §3 Lessons promoted this phase

### Promoted: `feedback_postgres_text_null_byte_forbidden.md`

**Signal:** Task 1's `acquire_evict_lock` key used `\x00` as separator. Postgres `TEXT` type forbids null bytes at the wire level. Defect is invisible to `cargo check`, `cargo clippy`, and even `cargo test --no-run`. Only a runtime Postgres query bind (Task 2's e2e) surfaces it. The corrective recipe: use a printable ASCII separator (`:` chosen; alternatives: `/`, `|`). Lesson body published below.

**Promoted to:** `.claude/lessons/feedback_postgres_text_null_byte_forbidden.md`

**Mandatory lesson table addition (advisor-orchestrator.md §2.4):** add to the file-pattern → lesson table row for any `pg_advisory_xact_lock` or `void PG function call` (already exists as `feedback_pg_advisory_xact_lock_void_decode.md`); the null-byte lesson is a companion for the lock-key construction pattern.

### Candidate (not yet promoted): `feedback_advisor_must_commit_brief_before_planning_dispatch.md`

**Signal (recurrence):** planning Junior queued for fed-in-e without brief file being committed; bootstrap served as de-facto brief. Same pattern fired in an earlier phase (referenced in plan §19). Two occurrences. Candidate for lesson promotion at 3rd occurrence or user judgment.

**Not promoted yet** — awaiting user judgment per the plan §19 note: "Retro proposes candidate lesson... may promote at retro time if user judges evidence sufficient." Carry-forward item tracks it.

---

## §4 Carry-forward items

### Mandatory (from bootstrap)

1. **governance-v0 divergence check before gate 5:** before bm-merge, advisor runs `git log origin/governance-v0 ^phase-v1-federation-inbound-e` — non-empty requires checkout + merge + push. Standard procedure.
2. **PRE-PUSH MANDATE on BM-verb briefs:** every bm-task brief must cite the PRE-PUSH MANDATE line from `.claude/rules/advisor-orchestrator.md §2.1`. Not a regression this phase; reminder for next BM brief.
3. **Daemon pre-dispatch ref check via ssh:** `ssh homeserver git -C /srv/brehon-fork fetch origin phase-v1-<next>` before any Junior impl-task dispatch, to ensure daemon-local ref is not stale. Recurred in v1-AD-e (5×); no recurrence this phase.

### This phase's contribution

4. **Brief-skip workflow gap:** planning Junior was queued without the brief file committed. Candidate lesson `feedback_advisor_must_commit_brief_before_planning_dispatch.md` — second occurrence; promote at third or by user decision. Structural fix candidate: pre-queue guard in advisor-orchestrator.md §2 that checks the brief file exists on the target branch before creating the task.

5. **Postgres TEXT null-byte lesson:** `feedback_postgres_text_null_byte_forbidden.md` promoted (see §3). Add to advisor-orchestrator.md §2.4 mandatory lesson injection table under `pg_advisory_xact_lock` row.

### Optional follow-ups (post-pilot, deferred from §12)

6. Per-peer in-memory bound (deferred).
7. SHA-256 key-hash for advisory-lock key (deferred).
8. `governance_config` knob for per-peer cap per table (deferred).
9. Postgres-backed per-peer counters (deferred — currently in-memory in `wrap_governance_inbound`).
10. Lock timeout via `pg_try_advisory_xact_lock` if pilot observes connection-pool starvation (deferred).
11. `serial_test` dev-dep if Phase-2 e2e shows tokio multi-thread spawn-ordering flake (not observed; watchpoint remains open).

---

## §5 Watch items (not failures)

- **No new flakes observed** in Phase-2 e2e. `storage_cap_holds_under_concurrent_receivers` ran deterministically with tokio `flavor = "multi_thread"` — no ordering dependency on other tests.
- **`retro_bypass` rate:** 0 bypass events this phase (retro authored before Stop hook could fire, per pre-retro discipline).
- **Shape G SUSPENDED** (DQ #229 pending re-enable 2026-06-01): validate-pending-laptop pathway active; no Shape G events this phase.

---

## §6 Open questions (none blocking merge)

None. Gate-1 concurrency-model DQ resolved at plan-approval. `\x00` separator defect resolved as user-confirmed amend. Phase-2 e2e clean.
