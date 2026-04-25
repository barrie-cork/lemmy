# Plan: v1-JM-c — submit_jury_vote 9-step handler (snapshot-aware threshold + deadlock + appeal_window)

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata |
| 6 | Relationship to other v1-JM sub-phases |
| 7 | Preflight guardrails inherited from prior phases |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-JM-c |
| 13 | Step-by-step tasks |
| 14 | Testing strategy |
| 15 | Validation commands (DoD) |
| 16 | Acceptance criteria |
| 17 | Completion checklist |
| 18 | Risks and mitigations |
| 19 | Notes |
| 20 | Sub-phase stubs (v1-JM-d/e TOC) |

---

## 1. Summary

v1-JM-c is the **vote-tally sub-phase** of v1 jury-mechanics. It consumes the snapshot fields that v1-JM-b writes onto `moderation_case` at admin-assign time and turns the v0 `submit_jury_vote` handler into a **snapshot-aware**, **deadlock-detecting**, **appeal-window-emitting** handler per PRD §9.1's 9-step pseudocode. It does not touch admin-assign code, does not add new schema, and does not implement appeals (that's JM-d).

Concretely v1-JM-c delivers:

1. **Replacement of the v0 hardcoded constants** `QUORUM: i64 = 3` (`submit_jury_vote.rs:86`) and `APPEAL_WINDOW_DAYS: i64 = 7` (`submit_jury_vote.rs:88`) with **snapshot reads** (`case.quorum_snapshot`, `case.threshold_count_snapshot`) for the threshold check and a **LIVE config read** (`config::get_int(... "appeal.window_days")`) for the appeal window — per PRD §9.1 cross-references and ADR-010.
2. **Step 5 threshold-check rewrite** — the v0 simple-majority `pick_majority` HashMap-max becomes (a) a per-decision vote count, (b) a `>= threshold_count_snapshot` test that picks the first decision meeting threshold, (c) a deadlock fall-through when `vote_count == panel_size_snapshot AND no decision met threshold` → flip `case.status = CaseStatus::AdminReview` + emit `jury_deadlock` governance_log entry + early-return.
3. **Step 7 sponsor-liability TODO insertion-point stub** — a single load-bearing comment at the exact line v1-SL-d will graft the sponsor-liability compute branch (between sanction insert and case-status flip). The comment cites the cross-PRD contract from `v1-sponsor-liability.prd.md §9.1 / §9.3` and serves as the merge anchor for SL-d's diff.
4. **Step 9 appeal-window write** — `appeal_window_expires_at = decided_at + Duration::days(window_days)` written on **both** the no-sponsor `Decided` path AND the (TODO-stubbed) sponsor-liability path. JM-c is the first writer of this column. The LIVE config read at this step is the single deliberate exception to the snapshot-everything rule per PRD §9.1 cross-references (changing `appeal.window_days` mid-flight affects future decisions, not in-flight cases — but the *appeal window itself* is a procedural input that's read at decision time, not jury-seating time).
5. **`ENTRY_KIND_JURY_DEADLOCK` const declaration** — added to `crates/db_schema/src/source/governance/governance_log.rs` and re-exported via the `crates/api/api/src/governance/governance_log.rs` shim. JM-a seeded 6 v1-JM-a entry kinds; JM-c adds 1 (the only new entry kind in this sub-phase). All other governance_log emissions (`case_decided`, `sanction_created`, `public_log_published`, juror reputation events) reuse existing v0 consts.
6. **e2e tests in `tests/e2e.rs`** — six new tests: (a) snapshot-read happy path (panel of 7 with quorum 5 / threshold 5 reads snapshot, decides at 5 votes); (b) deadlock-to-AdminReview (panel of 5 with threshold 3, votes split 2/2/1, casts the 5th vote, asserts deadlock log + AdminReview flip); (c) appeal_window_expires_at populated on no-sponsor path; (d) appeal_window_expires_at populated under config override (live read verifies the read is not snapshotted); (e) `v0_case_completes_under_v0_rules_after_v1_config_flip` (PRD §11 canonical regression); (f) two-vote concurrency test (two jurors race to be the threshold-meeting vote; only one decision fires; the other vote is recorded but the post-decision block runs exactly once per the existing FOR UPDATE + idempotency guard).

This sub-phase's single-slice value: the **decision point itself becomes severity-aware via the snapshot fields JM-b writes**, the **deadlock case has a procedurally clean exit** (the `CaseStatus::AdminReview` variant existed but had no writer), and the **appeal-window column** that JM-a backfilled gets its first live writer — unblocking JM-d (which reads `appeal_window_expires_at > now()` instead of `closed_at IS NULL` for the bounded-window appeal check).

No edits to `admin_assign_jury.rs`, `admin_emergency_remove.rs`, `accept_jury_assignment.rs`, `decline_jury_assignment.rs`, `request_appeal.rs`, `config.rs`. No new migrations. No new seeded config keys. No new helpers in `jury_common.rs`. No background jobs. No federation-outbound changes (the existing `federation_outbox::send_local_sanction_notice` call at `submit_jury_vote.rs:483` stays exactly as-is).

---

## 2. Source

- [../prds/v1-jury-mechanics.prd.md](../prds/v1-jury-mechanics.prd.md) §9.1 (9-step pseudocode — load-bearing cross-PRD shape, user-provided 2026-04-19 B6 resolution), §10 (defaults matrix — JM-c reads `appeal.window_days`), §11 (v0-compat canonical regression test), §12.4 (appeal-rights spoofing protection, partially relevant), §17 row 3 (v1-JM-c task scope)
- [../prds/v1-sponsor-liability.prd.md §9.1 / §9.3](../prds/v1-sponsor-liability.prd.md) — TODO-insertion-point contract for step 7; JM-c's stub comment names this section so SL-d's diff can find the anchor
- [../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ADR-010 (no retroactive invalidation of in-flight juries — load-bearing for snapshot reads), ADR-013 (`CaseStatus::EmergencyRemove` exhaustive match — preserved in idempotency guard), ADR-015 (pseudonymisation — preserved in `actor_pseudonym_helper::get_or_create` calls)
- [v1-JM-b plan](./v1-jury-mechanics-b.plan.md) — canonical structural template (§§1-20 mirrored verbatim); §3.3 hand-off notes for JM-c; §10 patterns for snapshot UPDATE / governance_log emission
- [v1-JM-b retro §3.2 amendments 1-4](../reports/v1-JM-b-retro.md) — five plan-template amendments rolled into JM-c §10 / §13 / §14 (R1: `as`-cast guard; R2: reputation_snapshot seeding; R3: struct-extension grep sweep; R4: lowercase test names; R5/R6: §13 Task 0 + uniform clippy flags from retro-events Event 4 / Event 3)
- [v1-JM-b retro-events Events 1-4](../reports/v1-JM-b-retro-events.md) — gate-keeper-drop pattern fix (§13 Task 0 enumeration), `--no-deps` clippy uniformity, struct-extension chore-commit hygiene
- [PR #95 carry-forward issue #96](https://github.com/barrie-cork/lemmy/issues/96) — 7 findings explicitly OUT of JM-c scope; plan §19 Notes calls out which paths JM-c must NOT silently fold in
- Precedent: [v1-AD-c plan](./v1-admin-dashboard-c.plan.md) for the "first reader of a JM-a-shipped substrate" pattern; v1-JM-b plan for the "first writer of a JM-a column" pattern (the snapshot fields)

---

## 3. Problem statement

Post-v1-JM-b-merge (governance-v0 @ `4d2b93ed9` after BM bundle `2d7002acc`), `submit_jury_vote.rs` is in a transitional state:

- It carries TWO hardcoded v0 constants — `QUORUM: i64 = 3` (line 86) and `APPEAL_WINDOW_DAYS: i64 = 7` (line 88) — that contradict the snapshot machinery JM-a + JM-b shipped. Every call to this handler currently ignores `case.quorum_snapshot`, `case.threshold_count_snapshot`, and `case.severity_tier`, and overwrites `case.closed_at = now + 7d` instead of writing `case.appeal_window_expires_at`. The case row already has these fields populated (JM-a backfill + JM-b live writes), but no reader exists.
- Its `pick_majority` helper (lines 510-518) implements a HashMap-max simple-majority pick — correct for the v0 5-juror / 3-of-5 case but incorrect for the JM-b severity-aware sizing (a 7-juror Severe case needs `ceil(7 * 0.71) = 5` votes for quorum and `ceil(7 * 0.75) = 6` votes for threshold, NOT a simple-majority HashMap pick).
- It has **no deadlock path**. The existing handler always picks a winning decision via `pick_majority` even when the highest-vote-count decision didn't meet a fraction-based threshold. PRD §9.1 step 5 deadlock semantics ("`all_jurors_voted AND no_decision_met_threshold` → flip `CaseStatus::AdminReview` + emit `jury_deadlock`") have no implementation.
- The `CaseStatus::AdminReview` variant (`enums.rs:407`) exists but has **zero writers** — it appears in the idempotency guard (line 271) as a terminal-status match, but no code path currently transitions into it. JM-c is the first writer.
- The `appeal_window_expires_at` column (`moderation_case.rs:76`) was added by JM-a and backfilled with `COALESCE(closed_at, decided_at + 7d)` — but **zero handlers write it post-decision**. JM-c is the first writer; JM-d will be the first reader (via the bounded-window appeal check that replaces `case.closed_at.is_some()` with `appeal_window_expires_at > now()`).
- Its v0 line 191 emits the literal entry kind string `"jury_vote_submitted"` even though `ENTRY_KIND_JURY_VOTED = "jury_voted"` exists at `governance_log.rs:116` and is documented in `.claude/rules/governance-log-entry-kind-registry.md` line 67 as the canonical const for "Individual juror submitted vote". This is a v0 legacy literal — JM-c does NOT change it (would silently break any log consumer reading `entry_kind = 'jury_vote_submitted'`); see §19 for the carry-forward note.

The substrate is in place; only the readers/writers in the vote-tally handler are missing. v1-JM-c is the bridge.

The scope boundary is load-bearing: if JM-c also implements appeals (`request_appeal.rs` rewrite, `select_appeal_panel`, the appeal-window-expiry background job, the new `admin_trigger_appeal_rejury` handler), the ralph loop exceeds the ~10-12 task phase-splitting threshold per the Phase 1 retro and the v1-AD retro. Appeals are JM-d; JM-c stops at the line where `appeal_window_expires_at` is written, before any reader exists.

---

## 4. Solution statement

Keep the v0 `submit_jury_vote` handler **structurally recognisable** — the outer shape (run_transaction wrapper, assignment check, vote insert, assignment-status flip, governance_log emission, vote count load, FOR UPDATE case load with idempotency guard, sanction insert, sponsor_liability call, case-status flip, public_case_log + governance_log + reputation_event + federation publish) stays intact. Five surgical edits.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **The `QUORUM` and `APPEAL_WINDOW_DAYS` consts are DELETED, not retained as fallbacks.** Per ADR-010, the snapshot fields (`quorum_snapshot`, `threshold_count_snapshot`) are the source of truth for any case that reached `JurySelection`. JM-a's backfill set both to 3 for pre-v1 cases, so every code path that reads the snapshot will get a valid value. Carrying a const fallback would be (a) dead code — the snapshot is always populated post-backfill — and (b) misleading — a future reader would assume the snapshot might be NULL, which it's not (the column types are `Option<i32>` for nullability symmetry with the migration but the backfill + JM-b writes make it effectively NOT NULL for any case in `JurySelection` or later). Use `case.quorum_snapshot.ok_or(LemmyErrorType::NoEnoughJurorsAvailable)?` (or the appropriate error variant) at the read site so a NULL snapshot fails loudly rather than silently degrading to v0 behaviour.
- **`appeal.window_days` is read LIVE at step 9, not snapshotted.** Per PRD §9.1 cross-references: "owned by this PRD (§10); read at decision time (step 9), NOT snapshotted, per ADR-010's no-retroactive-invalidation-of-procedural-rules-of-the-*past*-vote reading. Changing `appeal.window_days` mid-flight affects windows for cases decided after the change, not cases already decided." This is the single deliberate exception in JM-c: every other config read is forbidden because the snapshot exists. The `appeal_window_expires_at` is a property of the *decision moment*, not the *jury-seating moment*.
- **The threshold check is a per-decision tally, not a HashMap-max.** PRD §9.1 step 5: "if any decision has >= threshold votes: winning_decision = that decision; continue to step 6." The implementation iterates each `JuryDecision` variant, counts votes for it, and picks the first one (in stable enum-order) that meets `>= threshold_count_snapshot`. Stable enum-order matters because two decisions could in principle BOTH meet threshold (e.g., 7 jurors split 5-Sanction / 5-AdvisoryLabel is mathematically impossible with 7 jurors but with 11 jurors a 6-Sanction / 5-AdvisoryLabel split is possible — both meet threshold=6 if threshold_fraction is 0.5; PRD §10 defaults make threshold > 50% so this rarely happens but the code must not pick non-deterministically). Use the enum-order iteration via `JuryDecision::*` literal match in a fixed sequence; the first decision meeting threshold wins.
- **Deadlock detection requires `vote_count == panel_size_snapshot` AND `no decision met threshold`.** A partial tally (`vote_count < panel_size_snapshot`) where no decision has reached threshold yet is NOT deadlock — it's an early return with `case_decided: false`. Deadlock only fires when the panel is fully voted and the math says no winner. The `panel_size_snapshot` field was written by JM-b at admin-assign time for every case post-JM-a-merge.
- **Step 6 sanction insert preserves the v0 path verbatim.** The `map_decision_to_sanction(winning_decision)` helper at `submit_jury_vote.rs:523-549` already handles all 8 `JuryDecision` variants exhaustively (per ADR-013). JM-c does not touch this helper. The `SanctionInsertForm` construction (lines 282-296) is also unchanged. The `sanction_created` governance_log emission at line 300 is unchanged.
- **Step 7 sponsor-liability TODO stub is a comment, not code.** v0 currently has `sponsor_liability::apply_sponsor_liability(...)` at line 314 (inside the sanction-insert block when `case_row.target_person_id` is `Some`). v1-SL-d will rewrite this call site to compute deltas + flip status to `SponsorLiabilityPending` + emit `sponsor_liability_pending` log entry, all per `v1-sponsor-liability.prd.md §9.1` (compute/fire split). JM-c's TODO comment names that PRD section so SL-d's diff can find the exact anchor:
  ```rust
  // TODO(v1-sponsor-liability-d): replace this v0 apply_sponsor_liability call with the
  // compute/fire split per v1-sponsor-liability.prd.md §9.1 + §9.3:
  //   - compute_sponsor_liability(...) returns deltas (no event rows yet)
  //   - flip case.status = CaseStatus::SponsorLiabilityPending
  //   - set case.grace_expires_at = now + grace_window_for_severity(severity)
  //   - emit governance_log entry sponsor_liability_pending
  //   - notify_sponsor_of_pending_liability(...) for each delta
  //   - DEFER public_case_log + juror reputation_events to scheduler fire/escape time
  // The current v0 apply_sponsor_liability stays in place for JM-c — SL-d is the rewrite.
  ```
  v0 `sponsor_liability::apply_sponsor_liability` stays as-is in JM-c. The TODO is the merge anchor.
- **Step 8 no-sponsor path preserves v0 lifecycle verbatim.** `public_case_log` insert (lines 344-353), juror `reputation_event` writes (lines 391-450), `case_decided` governance_log emission (lines 472-490) — all unchanged. The only edit at this layer is replacing the line 327-335 case UPDATE block to write `appeal_window_expires_at` instead of `closed_at` (see step 9 below).
- **Step 9 appeal_window write fires on BOTH the v0 no-sponsor path AND the (TODO-stubbed) sponsor-liability path.** Both paths reach a "case decided, jury done" terminal moment; both need `appeal_window_expires_at = decided_at + window_days` written before the response returns. JM-c writes it on the no-sponsor path inline (the `closed_at` UPDATE is replaced); SL-d will graft the same write onto its sponsor-liability path. The PRD §9.1 step 9 comment "fires on BOTH the SponsorLiabilityPending path AND the no-sponsor Decided path" is the contract.
- **`closed_at` is NOT written by JM-c.** v0 sets `closed_at = now + 7d` at line 327-335 (the same UPDATE that flips status to Decided). JM-c REMOVES this write entirely. `closed_at` becomes a JM-d concern (the appeal-window-expiry background job will set `closed_at = now()` when it transitions a case from Decided → Closed; until then `closed_at` is NULL). The v0 backfill migration at `2026-04-23-000100_*/up.sql` populated `appeal_window_expires_at = COALESCE(closed_at, decided_at + 7d)` for pre-v1 cases, so any pre-v1 case still has `closed_at` set — that's fine, JM-c just stops writing it for new decisions.
- **`CaseStatus::AdminReview` is the deadlock terminal state — no further automation runs on it.** Per PRD §9.1 step 5 + ADR-010, the case sits in AdminReview until an admin manually intervenes (no JM-c handler for that intervention; that's a JM-d / future-phase concern). The deadlock path emits `jury_deadlock` governance_log + flips status + early-returns; no `case_decided`, no `sanction_created`, no `public_case_log`, no juror reputation events. The deadlock is procedurally distinct from a decision: nothing is "decided," it's "stuck."
- **`ENTRY_KIND_JURY_DEADLOCK` is a NEW const, declared in `governance_log.rs` and re-exported via the api shim.** Per `.claude/rules/governance-log-entry-kind-registry.md`, every governance_log entry kind needs both (a) a const declaration in the canonical source-of-truth file, (b) a registry table row, (c) a re-export through the api shim, (d) a documented call site. JM-c adds the const + the registry row + the re-export + the call site (in `submit_jury_vote.rs::process_vote` deadlock branch). Per JM-b retro §3.3: JM-c does not add `case_decided`, `jury_voted`, `sponsor_liability_pending` — those are existing or sponsor-liability-d territory.
- **The v0 literal `"jury_vote_submitted"` at line 191 stays as-is.** Per the entry-kind registry the canonical const is `ENTRY_KIND_JURY_VOTED = "jury_voted"` — but `submit_jury_vote.rs:191` emits the v0-era literal `"jury_vote_submitted"` which has been the actual emitted entry kind for every juror vote since Phase 4b. Changing it would silently break any log consumer that reads `entry_kind = 'jury_vote_submitted'`. The cleanup is a separate carry-forward (issue #96 has nothing comparable; this would be a new follow-up if anyone wants the cleanup). JM-c does NOT touch line 191. **§19 Notes documents this explicitly so future readers don't confuse it for a JM-c bug.**
- **No edits to the existing `pick_majority` helper at line 510-518 — it gets DELETED.** The v0 simple-majority logic is wholly replaced by the per-decision threshold check in step 5. The dead helper goes with the change. Same for the unused `JURY_RECENCY_DAYS` const if any survives in this file (none seen — keep the audit at task-level).

### 4.2 Rejected alternatives

- **Keep `pick_majority` and use it as a fallback when no decision meets threshold.** Rejected: PRD §9.1 step 5 explicitly defines the no-threshold-met case as **deadlock**, not "fall back to highest-vote-count." A 5-juror panel with 2/2/1 split has no winner under simple-majority either (the v0 HashMap-max would non-deterministically pick one of the two-vote decisions); JM-c's deadlock path is the procedurally clean exit.
- **Snapshot `appeal.window_days` at admin_assign_jury time alongside the panel/quorum/threshold snapshots.** Rejected: ADR-010 is about *invalidating in-flight procedural decisions*. The appeal window is a **future-procedural-input** that's read at the *decision moment* — by then the jury process is done, the snapshot frozen at jury-seat time has already done its job (preserving the rules the panel agreed to play by). Reading `appeal.window_days` at decision time is consistent with reading any other operational config (e.g., the deltas at lines 366-378 which are also live reads, not snapshotted). PRD §9.1 cross-references explicitly call this out.
- **Make the deadlock path optional (controlled by a new `jury.deadlock_path_enabled` config flag).** Rejected: ADR-010 + PRD §9.1 lock in the deadlock path as procedurally mandatory. A config flag would allow operators to turn off a procedurally-required state transition, which is exactly what ADR-010 forbids.
- **Add a new `closed_at_v1` column and write it instead of `closed_at`.** Rejected: there's no semantic difference; `appeal_window_expires_at` IS the new "case is open until N days after decision" semantics. Adding a parallel column duplicates state. The migration backfill already handles the legacy mapping.
- **Bundle JM-d's `select_appeal_panel` helper into JM-c.** Rejected: that's the JM-d core — appeal panel re-selection with `exclude_person_ids = original_jurors` + `role = Appeal` + next-tier threshold. JM-c stops at writing `appeal_window_expires_at`; JM-d reads it.
- **Add the appeal-window-expiry background job in JM-c.** Rejected: same — that's JM-d. The job needs `appeal_window_expires_at` to exist (JM-c's job) but the *scheduler tick* + the *transition logic* are JM-d's scope per PRD §9.5.
- **Rewrite the v0 literal `"jury_vote_submitted"` to use `ENTRY_KIND_JURY_VOTED` const in this commit.** Rejected: the literal has been emitted since Phase 4b; any log consumer reading `entry_kind = 'jury_vote_submitted'` (federation outbound, dashboards, audit tooling) would silently break. Cleanup belongs in a separate `chore(governance-log): align jury vote entry kind to canonical const` PR with a backfill UPDATE that rewrites historical rows. Out of JM-c scope.
- **Read `case.severity_tier` to validate the snapshot read.** Rejected: not needed. The snapshot fields (`quorum_snapshot`, `threshold_count_snapshot`, `panel_size_snapshot`) carry all the data JM-c needs to make the decision. Reading severity_tier additionally would only matter for *logging context* (e.g., including severity in the `case_decided` payload), which is a nice-to-have not load-bearing. Skip; can be added in JM-d if dashboard queries need it.

---

## 5. Metadata

| Field | Value |
|---|---|
| Type | `HANDLER + CROSS_CUTTING` (submit_jury_vote rewrite + 1 new entry-kind const + 1 registry row) |
| Complexity | MEDIUM (handler-only; no schema, no migrations, no new helpers; isolated to one file plus one const declaration) |
| Crates Affected | `lemmy_api` (`crates/api/api/src/governance/submit_jury_vote.rs`, `crates/api/api/src/governance/governance_log.rs` re-export), `lemmy_db_schema` (`crates/db_schema/src/source/governance/governance_log.rs` const + 1 doc-line mention), `lemmy_server` (`crates/server/tests/e2e.rs` test additions) |
| v1 Step | v1-JM sub-phase C (per v1-JM PRD §17 row 3) |
| Dependencies | v1-JM-b merged to `governance-v0` (snapshot machinery + cascade helpers + e2e fixtures `seed_jury_eligible_snapshots` + `seed_founder_event` + `seed_case`) |
| Estimated Tasks | 9 (Task 0 pre-flight + Task 1 const + Task 2 const reads + Task 3 deadlock + Task 4 step 7 TODO + Task 5 step 9 + Task 6 e2e tests + Task 7 full-workspace validation + Task 8 retro) |
| Sub-phase target branch | `phase-v1-JM-c` (cut by BM from `governance-v0` HEAD post-this-plan-PR-merge) |
| Sub-phase target worktree | `../brehon-fork-phase-v1-JM-c` (provisioned by BM post-branch-cut) |
| Confidence target | 9/10 (per JM-b retro §5: "if §3.2 amendments 1+2+3 land in JM-c plan, those sub-phases should hit 9+/10") |

---

## 6. Relationship to other v1-JM sub-phases

| Sub-phase | Status as of this plan | What it ships | Dependency on JM-c |
|---|---|---|---|
| v1-JM-a | MERGED (2026-04-24, PR #92 → governance-v0 @ `e1c22c759`) | Schema + 27 seeded keys + 6 entry-kind consts + backfill | JM-c reads JM-a-shipped consts (`ENTRY_KIND_CASE_DECIDED`, `ENTRY_KIND_PUBLIC_LOG_PUBLISHED`, `ENTRY_KIND_SANCTION_CREATED`); declares 1 NEW const `ENTRY_KIND_JURY_DEADLOCK` |
| v1-JM-b | MERGED (2026-04-25, PR #95 → governance-v0 @ `4d2b93ed9`, then BM bundle `2d7002acc`) | `admin_assign_jury` cascade + diversity + snapshot writes + emergency-remove cascade fix (cr-3) | JM-c reads JM-b-written snapshot fields (`quorum_snapshot`, `threshold_count_snapshot`, `panel_size_snapshot`); reuses JM-b e2e fixtures (`v1_jm_b_fixtures::seed_jury_eligible_snapshots`, `seed_founder_event`, `seed_case`) |
| **v1-JM-c (THIS PLAN)** | NOT YET CUT (plan in primary worktree on `governance-v0`) | `submit_jury_vote` 9-step rewrite (steps 1-6 + step 9 + sponsor-liability TODO at step 7) + `ENTRY_KIND_JURY_DEADLOCK` + 6 e2e tests | — |
| v1-JM-d | NOT YET PLANNED (stubs in JM-b plan §20) | `request_appeal` bounded-window check; `select_appeal_panel`; `admin_trigger_appeal_rejury` handler; appeal-window-expiry background job | JM-d reads `appeal_window_expires_at` (JM-c is the first writer); JM-d reads JM-c-written status `CaseStatus::Decided` + `decided_at` |
| v1-JM-e | NOT YET PLANNED (stubs in JM-b plan §20) | `v0_case_completes_under_v0_rules_after_v1_config_flip` capstone (PRD §11) + cross-sub-phase integration tests + audit-log invariant test | JM-e exercises the full lifecycle JM-c stabilises at the vote-tally point |

**Cross-PRD dependency graph (per JM PRD §17.1):**

- **JM-c MUST merge before SL-d starts impl.** v1-SL-d grafts the sponsor-liability compute branch onto `submit_jury_vote` step 7 — the exact line JM-c stabilises with the TODO stub. Concurrent impl would produce three-way merge conflicts on every commit.
- **JM-c MUST merge before rep-tuning-r3 starts impl.** v1-rep-tuning-r3 adds vote-outcome + evidence-quality emitters at `submit_jury_vote` step 7 (same TODO insertion point SL-d grafts onto). Serial with JM-c; can run parallel with SL-d after JM-c lands.
- **JM-d/e can run in parallel with SL-d and rep-tuning-r3/r4/r5** — different handlers, different files once JM-c has stabilised the submit_jury_vote shape.

JM-c is the **serialisation gate** for JM-d, SL-d, and rep-tuning-r3+. It is the highest-leverage merge in the v1-JM wave.

---

## 7. Preflight guardrails inherited from prior phases

Per JM PRD §17.4 + JM-b retro §3.2 + JM-b retro-events Event 4 — gate-keepers that must explicitly land in JM-c plan §13 Task 0 (NOT inherited implicitly from rules files):

| ID | Source | Gate-keeper | JM-c plan position |
|---|---|---|---|
| **DQ #42** | v1-AD-d retro §2.1 | `/prp-core:prp-implement` §1.4 task-resume safety | Inherited from command template; cited in §13 Task 0 EXPECT block; no inline compensation |
| **DQ #43** | v1-AD-d retro §2.2 | `/prp-core:prp-implement` §4.1.1 HTTP status code audit | Inherited from command template; JM-c does not introduce new 4xx codes (handler returns `LemmyResult` which routes through `LemmyErrorType::status_code()`); no inline compensation |
| **DQ #44** | v1-AD-d retro §2.3 | `docker ps` Probe 0 | Enumerated explicitly in §13 Task 0; ALSO runs in `/prp-core:prp-implement` §4.2.0 before every e2e invocation |
| **DQ #46** | v1-AD-d retro §3.2 | Phase 5 REPORT template "Follow-up GH issues" section | Inherited from command template; cited in §13 Task 9 retro |
| **R5 (NEW from JM-b retro-events Event 4)** | JM-b retro-events Event 1 + Event 4 | Pre-phase clippy-baseline capture per `pre-phase-harness-audit.md §3` | **Enumerated explicitly in §13 Task 0 — Probe 5; NOT inherited implicitly** |
| **R6 (NEW from JM-b retro-events Event 3)** | JM-b retro-events Event 3 | Uniform `--no-deps` clippy flag across §13 + §15 + §16 | All clippy commands in this plan use the same flag set: `--workspace --features full --no-deps -- -D warnings` |
| **R7 (NEW from JM-b retro §2.3)** | JM-b retro §2.3 (JM-a-drift bleed-through) | `cargo test --no-run -p lemmy_server --test e2e` in handler-task DoDs that touch a struct | Added to §13 Task 1 (const + governance_log shim re-export) and §13 Task 6 (e2e tests) — wherever JM-c could surface a struct-mismatch compile error |

JM-b retro §3.2 amendments R1 (as-cast guard) + R2 (reputation_snapshot seeding) + R3 (struct-extension grep sweep) + R4 (lowercase test names) are rolled into §10 patterns + §13 task wording + §14 test wording (see those sections for inline applications).

---

## 8. Flow design

### 8.1 Before state (v0 + JM-a backfill + JM-b admin-assign)

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE (post-JM-b)                                 ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                 ║
║  admin assigns jury (JM-b) → moderation_case row has populated fields:          ║
║    - severity_tier = 'Severe' (set by admin / inferred / DEFAULT 'Minor')       ║
║    - status_tier   = 'Founder' (computed by JM-b)                               ║
║    - panel_size_snapshot = 7   (cascaded by JM-b)                               ║
║    - quorum_snapshot     = 5   (cascaded by JM-b)                               ║
║    - threshold_count_snapshot = 5  (cascaded by JM-b)                           ║
║    - appeal_window_expires_at = NULL  (NOT YET WRITTEN; backfill empty for      ║
║                                         post-JM-a cases that haven't decided)   ║
║                                                                                 ║
║  juror N submits vote → submit_jury_vote.rs:                                    ║
║    - QUORUM = 3 (HARDCODED — IGNORES quorum_snapshot=5)                        ║
║    - vote_count >= 3 → run pick_majority HashMap-max (IGNORES                   ║
║                        threshold_count_snapshot=5)                              ║
║    - pick wins regardless of fraction-based threshold                           ║
║    - case.status = Decided                                                      ║
║    - case.closed_at = now + 7 (HARDCODED APPEAL_WINDOW_DAYS = 7)                ║
║    - appeal_window_expires_at STAYS NULL                                        ║
║    - sanction inserted, public_case_log emitted, reputation_events fire         ║
║                                                                                 ║
║  PAIN_POINTS:                                                                   ║
║    - Severity-aware sizing has NO READER → 7-juror Severe panel decided         ║
║      with 3-of-7 vote (v0 quorum) instead of 5-of-7 (JM-b quorum_snapshot)      ║
║    - Threshold check IGNORES JM-b's threshold_count_snapshot                    ║
║    - Deadlock has NO PATH (5-juror split 2/2/1 picks non-deterministically)     ║
║    - appeal_window_expires_at column NEVER WRITTEN by post-JM-a cases           ║
║    - JM-d cannot ship — its `if case.appeal_window_expires_at < now()` check    ║
║      always returns true (NULL < now() is false; appeal-window logic broken)    ║
║                                                                                 ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### 8.2 After state (v1-JM-c)

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              AFTER (post-JM-c)                                  ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                 ║
║  juror N submits vote → submit_jury_vote.rs (rewritten step 5):                 ║
║    - load case_row WITH FOR UPDATE (unchanged)                                  ║
║    - read quorum_snapshot, threshold_count_snapshot, panel_size_snapshot        ║
║      from case_row (NEW; replaces hardcoded QUORUM)                             ║
║    - vote_count = COUNT(jury_vote WHERE case_id = ?)                            ║
║    - if vote_count < quorum_snapshot → early return (vote_recorded: true,      ║
║                                                       case_decided: false)     ║
║    - per-decision tally: for each JuryDecision variant in stable enum-order:    ║
║         vote_count_for_decision = COUNT(jury_vote WHERE case_id=?               ║
║                                          AND decision=variant)                  ║
║         if vote_count_for_decision >= threshold_count_snapshot:                 ║
║             winning_decision = variant; break                                   ║
║    - if no winning_decision found:                                              ║
║         if vote_count == panel_size_snapshot:                                   ║
║             # DEADLOCK: all jurors voted, no decision met threshold             ║
║             update case SET status = AdminReview                                ║
║             governance_log::append("jury_deadlock", ...)                        ║
║             early return (vote_recorded: true, case_decided: false)             ║
║         else:                                                                   ║
║             # PARTIAL TALLY: some jurors haven't voted yet                      ║
║             early return (vote_recorded: true, case_decided: false)             ║
║    - else: continue to step 6 with winning_decision                             ║
║                                                                                 ║
║  Step 6 (sanction insert): UNCHANGED                                            ║
║                                                                                 ║
║  Step 7 (sponsor-liability): UNCHANGED v0 apply_sponsor_liability call          ║
║    - ADD load-bearing TODO comment at exact graft point for SL-d                ║
║                                                                                 ║
║  Step 8 (no-sponsor / NoAction path): UNCHANGED v0 lifecycle                    ║
║    - public_case_log + juror reputation_events + case_decided governance_log    ║
║    - REMOVE: case.closed_at = now + 7 (deleted)                                 ║
║                                                                                 ║
║  Step 9 (NEW — appeal window write):                                            ║
║    - window_days = config::get_int(... "appeal.window_days").await? (LIVE)      ║
║    - update case SET appeal_window_expires_at = decided_at + Duration::days(    ║
║                                                  window_days)                  ║
║    - (Fires on BOTH no-sponsor Decided path AND TODO-stubbed SponsorLiability   ║
║       path; see §9.1 step 9 cross-reference for SL-d composition with §6.5      ║
║       min(appeal_window, grace_window) semantics.)                              ║
║                                                                                 ║
║  VALUE_ADDS:                                                                    ║
║    - 7-juror Severe panel now requires 5 votes (quorum_snapshot) AND 5 of one   ║
║      decision (threshold_count_snapshot) to decide                              ║
║    - 5-juror panel split 2/2/1 → AdminReview + jury_deadlock log entry          ║
║    - appeal_window_expires_at populated for all post-JM-c decisions             ║
║    - JM-d unblocked (its bounded-window appeal check has data to read)          ║
║    - SL-d unblocked (its sponsor-liability compute has a stable graft point)    ║
║                                                                                 ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### 8.3 Endpoint changes

| Endpoint | Before (v0) | After (v1-JM-c) | Impact |
|---|---|---|---|
| `POST /api/v4/governance/submit-jury-vote` | Hardcoded QUORUM=3 + simple-majority pick + closed_at write | Snapshot-aware threshold + deadlock detection + appeal_window_expires_at write | Decision rule becomes severity-aware; deadlock procedurally clean; JM-d / SL-d unblocked |

No new endpoints. No DTO changes (`SubmitJuryVote` request + `SubmitJuryVoteResponse` response shapes are preserved verbatim — see §4.1 / §10.5 for the InsertForm propagation rule R3 which does NOT trip in this case). No route registration changes.

---

## 9. Mandatory reading

(Implementation agent reads these BEFORE Task 1.)

### 9.1 Brehon design docs (P0)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §9.1 (lines 520-602) | The 9-step pseudocode IS the contract; every JM-c task implements one or more steps |
| P0 | `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §10 (lines 658-700) | `appeal.window_days` knob (default 7, range [1, 90]) — JM-c step 9 reads this |
| P0 | `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §11 (lines 704-716) | v0-compat regression test (`v0_case_completes_under_v0_rules_after_v1_config_flip`) — JM-c is the OWNER per §17 row 3 |
| P0 | `.claude/PRPs/prds/v1-jury-mechanics.prd.md` | §17 row 3 (line 827) | Authoritative JM-c task scope statement |
| P0 | `.claude/PRPs/prds/v1-sponsor-liability.prd.md` | §9.1 + §9.3 | Step 7 TODO stub — JM-c's comment quotes this section so SL-d's diff can find the anchor |
| P0 | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | ADR-010, ADR-013, ADR-015 | Snapshot semantics (load-bearing for step 5); EmergencyRemove exhaustive match (preserved in idempotency guard); pseudonymisation (preserved in actor_pseudonym calls) |
| P0 | `.claude/rules/governance-log-entry-kind-registry.md` | All — §JM-c row to be added | JM-c adds `ENTRY_KIND_JURY_DEADLOCK`; the registry update is part of this plan's Task 1 commit |
| P0 | `.claude/PRPs/reports/v1-JM-b-retro.md` | §3.3 (lines 145-152) | JM-b's hand-off notes addressed directly to JM-c (snapshots populated; cascade helpers live; ConstraintRecord untouched; no new ENTRY_KIND consts to add beyond JURY_DEADLOCK; no handler files outside admin_assign_jury / admin_emergency_remove edited in JM-b) |

### 9.2 Codebase reads (P0 — mirror these patterns)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api/src/governance/submit_jury_vote.rs` | All 593 lines | The file being rewritten. Read end-to-end before editing — every existing call site interacts with one of the 9 steps |
| P0 | `crates/api/api/src/governance/admin_assign_jury.rs` | 90-260, 393-415 | Snapshot WRITE pattern (UPDATE block at 197-206) + governance_log emission pattern (211-224) — JM-c mirrors the emission pattern for `case_decided` (already there) and `jury_deadlock` (new) |
| P0 | `crates/api/api/src/governance/admin_emergency_remove.rs` | 220-260 | Adjacent snapshot-pattern precedent (PR #95 cr-3 fix) — JM-c does NOT touch this file but the pattern is the same shape |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 110-180 + 196-257 | Const declarations + `append` fn signature + `scrub_json` semantics — JM-c adds 1 const here and emits via `append` |
| P0 | `crates/db_schema/src/source/governance/moderation_case.rs` | 50-80, 100-120 | Field types: `quorum_snapshot: Option<i32>`, `threshold_count_snapshot: Option<i32>`, `panel_size_snapshot: Option<i32>`, `appeal_window_expires_at: Option<DateTime<Utc>>` |
| P0 | `crates/api/api/src/governance/governance_log.rs` | 30-75 | Re-export shim — JM-c adds `ENTRY_KIND_JURY_DEADLOCK` to the import list + the `pub use` re-export |
| P0 | `crates/api/api/src/governance/config.rs` | 220-260 | `config::get_int` signature (cache, pool, scope, key) — JM-c step 9 calls this for `appeal.window_days` |
| P0 | `crates/db_schema_file/src/enums.rs` | 393-410 (CaseStatus), 499-509 (JuryDecision), 676-702 (SeverityTier + CaseStatusTier) | Variant inventories — `CaseStatus::AdminReview` is the deadlock terminal state; `JuryDecision` enum-order determines threshold-pick ordering |

### 9.3 Test patterns (P1 — fixture sources)

| Priority | File | Lines | Why |
|---|---|---|---|
| P1 | `crates/server/tests/e2e.rs` | 1061-1510 (`report_to_modlog_golden_path`) | Canonical v0 end-to-end test — JM-c tests mirror the direct-handler-invocation pattern + the DB-assertion pattern |
| P1 | `crates/server/tests/e2e.rs` | 7041-7160 (`v1_jm_b_fixtures`) | `bootstrap`, `seed_user`, `seed_community`, `seed_jurors`, `seed_case`, `seed_jury_eligible_snapshots`, `seed_founder_event` — JM-c reuses ALL of these; per JM-b retro §3.2 R2, do NOT re-derive |
| P1 | `crates/server/tests/e2e.rs` | 4598-4750 (`admin_config_fixtures`) | `admin_set_config` direct-handler-invocation pattern — JM-c tests use this to bump `appeal.window_days` for the live-read assertion |
| P1 | `crates/server/tests/e2e.rs` | 1379-1407 (vote-submission loop in golden path) | The exact loop shape JM-c's tests will mirror for casting N votes from N jurors |
| P1 | `crates/server/tests/e2e.rs` | 1414-1510 (DB assertions in golden path) | The `AsyncPgConnection::establish` + diesel-select + `.first(&mut conn)` assertion pattern |

### 9.4 Rules (P0 — auto-loaded but worth re-reading at session start)

| Priority | File | Why |
|---|---|---|
| P0 | `.claude/rules/cargo-output-capture.md` | Every cargo run in §13 + §15 captures to `.claude/PRPs/debug/v1-JM-c-task*.log` with explicit exit-code preservation |
| P0 | `.claude/rules/no-cargo-output-paste.md` | Tail-only reads (≤30 lines) of cargo logs in conversation; full log stays on disk |
| P0 | `.claude/rules/phase-branch.md` | All commits land on `phase-v1-JM-c`; PR base is `governance-v0`, not `main` |
| P0 | `.claude/rules/decision-queue.md` | If JM-c hits a blocker, write DQ with `answered_by: null` — never self-attribute as advisor |
| P0 | `.claude/rules/pre-phase-harness-audit.md` | §13 Task 0 enumerates ALL 5+ probes from this rule (Docker + 4 wrapper + clippy baseline + DoD smoke) — see R5 in §7 |
| P0 | `.claude/rules/governance-log-entry-kind-registry.md` | Task 1 adds the JM-c row to this registry; format must match existing rows |
| P0 | `.claude/rules/pm-plugin-hooks-stable.md` | Verify at session start that JM-c does NOT touch `private_message/*` paths (it doesn't — submit_jury_vote.rs is governance-only) |

### 9.5 External documentation

JM-c needs no new external dependencies. All crates (`diesel`, `diesel-async`, `chrono`, `tracing`, `serde_json`, `tokio`) are already in the workspace. No docs.rs research required for this sub-phase.

---

## 10. Patterns to mirror

### 10.1 New entry-kind const declaration

**SOURCE:** `crates/db_schema/src/source/governance/governance_log.rs:170-178` (existing v1-JM-a const block)

```rust
// SOURCE: crates/db_schema/src/source/governance/governance_log.rs:170-174
// COPY THIS PATTERN for ENTRY_KIND_JURY_DEADLOCK at lines ~175+:
/// Severity-tier was frozen onto the case at jury-seating time. Emitted by
/// `admin_assign_jury` at the moment the snapshot UPDATE commits. Payload:
/// `{ case_id, severity_tier, status_tier, panel_size_snapshot,
///   quorum_snapshot, threshold_count_snapshot }`. Per ADR-010, this is the
/// canonical "after this point, no procedural change is legitimate" marker
/// for the case lifecycle.
pub const ENTRY_KIND_SEVERITY_TIER_FROZEN: &str = "severity_tier_frozen";
```

**JM-c addition:**

```rust
/// Jury panel reached `panel_size_snapshot` votes but no `JuryDecision`
/// variant met `threshold_count_snapshot`. Case is flipped to
/// `CaseStatus::AdminReview` for human resolution. Payload:
/// `{ case_id, panel_size_snapshot, threshold_count_snapshot,
///   tally: {<JuryDecision>: count, ...} }`. Emitted exactly once per
/// case at the deadlock-detection moment (PRD §9.1 step 5). Per ADR-010,
/// AdminReview is a procedurally-mandatory terminal state; admin
/// intervention is required to resume. No subsequent `case_decided`,
/// `sanction_created`, or `public_log_published` fires for a deadlocked
/// case.
pub const ENTRY_KIND_JURY_DEADLOCK: &str = "jury_deadlock";
```

**API shim re-export at `crates/api/api/src/governance/governance_log.rs`:**

```rust
// SOURCE: crates/api/api/src/governance/governance_log.rs:48-67 (existing import block)
// ADD ENTRY_KIND_JURY_DEADLOCK alphabetically into the use list:
pub use lemmy_db_schema::source::governance::governance_log::{
    append,
    ENTRY_KIND_CASE_DECIDED,
    // ... existing entries ...
    ENTRY_KIND_JURY_DEADLOCK,  // NEW (JM-c)
    ENTRY_KIND_JURY_VOTED,
    // ... rest of list ...
};
```

### 10.2 Snapshot field READ pattern

**SOURCE:** `submit_jury_vote.rs` does not currently read snapshot fields; this is a NEW pattern for JM-c.

**JM-c pattern (in step 5 after FOR UPDATE case load at line 240):**

```rust
// SOURCE: NEW in JM-c (no current reader in workspace)
// Pattern: explicit ok_or for nullable column that's effectively NOT NULL
// post-JM-a-backfill. Failing loudly at NULL is correct — a NULL snapshot
// would mean the case never went through admin_assign_jury (impossible
// for any case in JurySelection or later post-backfill).
let quorum_snapshot: i32 = case_row
    .quorum_snapshot
    .ok_or(LemmyErrorType::NoEnoughJurorsAvailable)?;
let threshold_count_snapshot: i32 = case_row
    .threshold_count_snapshot
    .ok_or(LemmyErrorType::NoEnoughJurorsAvailable)?;
let panel_size_snapshot: i32 = case_row
    .panel_size_snapshot
    .ok_or(LemmyErrorType::NoEnoughJurorsAvailable)?;
```

**GOTCHA (R1 — JM-b retro §3.2 amendment 1):** the snapshot fields are `i32` once unwrapped. The vote-count query returns `i64` (Postgres `COUNT(*)`). The threshold check compares the two:

```rust
// WRONG (trips clippy::as_conversions under -D warnings):
if vote_count_for_decision as i32 >= threshold_count_snapshot { ... }

// WRONG (also trips as_conversions on the inner cast):
if i32::try_from(vote_count_for_decision as i64).map_err(...)? >= threshold_count_snapshot { ... }

// RIGHT (mirror the geographic_diversity_score pattern at admin_assign_jury.rs:708-712):
// Convert threshold_count_snapshot to i64 for the comparison. i32 → i64 widening
// is lossless and not flagged by as_conversions; the comparison is then i64 vs i64.
let threshold_count_i64 = i64::from(threshold_count_snapshot);
if vote_count_for_decision >= threshold_count_i64 { ... }
```

`i64::from(i32)` is the canonical lossless widening — does NOT trip `clippy::as_conversions`. Same shape for `quorum_snapshot` and `panel_size_snapshot` if either is compared against an `i64`-typed count.

### 10.3 Per-decision threshold tally

**SOURCE:** NEW in JM-c (replaces v0 `pick_majority` at `submit_jury_vote.rs:510-518`).

**Pattern (in step 5):**

```rust
// SOURCE: NEW in JM-c
// Iterate JuryDecision variants in stable enum-order; first decision meeting
// threshold wins. PRD §9.1 step 5: "if any decision has >= threshold votes:
// winning_decision = that decision; continue to step 6."
//
// Stable enum-order matters: in the unlikely event two decisions both meet
// threshold (mathematically possible only when threshold_fraction <= 0.5,
// which the PRD §10 defaults forbid), the first variant in JuryDecision
// declaration order wins. This is deterministic by construction.
//
// JuryDecision variants (per enums.rs:499-509):
//   NoAction, AdvisoryLabel, Warning, Cooldown, RemoveContent,
//   SuspendLocalUser, SuspendCommunityMember, RecommendFederationAction
//
// We do NOT iterate via reflection — we hardcode the match arms below to
// avoid pulling in `strum` or another iterator dependency.

let mut winning_decision: Option<JuryDecision> = None;
let mut tally: HashMap<JuryDecision, i64> = HashMap::new();

// Single query: load decisions for this case, count by variant.
let all_decisions: Vec<JuryDecision> = jury_vote::table
    .filter(jury_vote::case_id.eq(data.case_id))
    .select(jury_vote::decision)
    .load::<JuryDecision>(conn)
    .await?;

for decision in &all_decisions {
    *tally.entry(*decision).or_insert(0) += 1;
}

// Stable enum-order iteration (hardcoded to avoid strum dependency).
for candidate in [
    JuryDecision::NoAction,
    JuryDecision::AdvisoryLabel,
    JuryDecision::Warning,
    JuryDecision::Cooldown,
    JuryDecision::RemoveContent,
    JuryDecision::SuspendLocalUser,
    JuryDecision::SuspendCommunityMember,
    JuryDecision::RecommendFederationAction,
] {
    let count = tally.get(&candidate).copied().unwrap_or(0);
    if count >= threshold_count_i64 {
        winning_decision = Some(candidate);
        break;
    }
}
```

**GOTCHA:** the `JuryDecision` enum has 8 variants per `enums.rs:499-509`. If a future PR adds a 9th variant, this hardcoded list will silently miss it. Mitigation: add a compile-time exhaustiveness assertion via a match expression that uses every variant. The pattern is in `submit_jury_vote.rs:523-549` (`map_decision_to_sanction`) — that helper already requires exhaustive match on all 8 variants, so any added variant will fail to compile there first. JM-c's iteration list is therefore safe by mirror — if `map_decision_to_sanction` adds a 9th arm, the JM-c plan author / impl will see the diff and update the iteration list. Document this dependency inline:

```rust
// INVARIANT: this iteration list MUST cover every JuryDecision variant.
// `map_decision_to_sanction` (line ~523) is the canonical exhaustive
// match — any new variant added to the enum will fail to compile in
// that helper first. When extending JuryDecision, update this list too.
```

### 10.4 Deadlock branch + governance_log emission

**SOURCE MIRROR:** `admin_assign_jury.rs:211-224` (`severity_tier_frozen` emission pattern)

**JM-c pattern (in step 5 after the per-decision threshold loop):**

```rust
// SOURCE MIRROR: admin_assign_jury.rs:211-224
// Pattern: explicit governance_log::append with json! payload + actor_pseudonym
// Note: Some(juror_pseudonym) is the CASTING juror — they're the one who
// triggered deadlock detection by being the panel_size-th voter without
// a winner emerging. ADR-015 actor attribution.

if winning_decision.is_none() {
    if vote_count == i64::from(panel_size_snapshot) {
        // DEADLOCK: all jurors voted, no decision met threshold.
        update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
            .set(moderation_case::status.eq(CaseStatus::AdminReview))
            .execute(conn)
            .await?;

        let tally_payload: serde_json::Map<String, Value> = tally
            .iter()
            .map(|(decision, count)| (format!("{:?}", decision), Value::from(*count)))
            .collect();

        governance_log::append(
            &mut conn.into(),
            ENTRY_KIND_JURY_DEADLOCK,
            json!({
                "case_id": data.case_id.0,
                "panel_size_snapshot": panel_size_snapshot,
                "threshold_count_snapshot": threshold_count_snapshot,
                "tally": tally_payload,
            }),
            Some(juror_pseudonym.clone()),
        )
        .await?;

        return Ok(SubmitJuryVoteResponse {
            vote_recorded: true,
            case_decided: false,
            decision: None,
        });
    } else {
        // PARTIAL TALLY: some jurors haven't voted yet.
        return Ok(SubmitJuryVoteResponse {
            vote_recorded: true,
            case_decided: false,
            decision: None,
        });
    }
}

let winning_decision = winning_decision.expect("checked above");
```

**GOTCHA:** the deadlock UPDATE writes `status = AdminReview` ONLY (no `decided_at`, no `closed_at`, no `appeal_window_expires_at`). A deadlocked case is not "decided" — it's "stuck pending admin." The lifecycle terminates here for now. A future admin-intervention handler (out of JM-c scope; not in JM-d either — likely v1.5 territory) will be the path that resumes a deadlocked case.

**GOTCHA (R3 — JM-b retro §3.2 amendment 3):** if extending `SubmitJuryVoteResponse` (e.g., adding a `deadlock: bool` field for client clarity), run the grep sweep first:

```bash
git grep -l 'SubmitJuryVoteResponse {' -- ':!target' ':!.git'
```

Every literal-construction site needs `..Default::default()` propagation OR explicit field naming. JM-c does NOT need to extend the response shape — `case_decided: false, decision: None` already conveys "no decision yet" without ambiguity for the deadlock case. A client that wants to distinguish deadlock-from-partial-tally can read `governance_log` for `entry_kind = 'jury_deadlock'`. **Decision: do NOT extend the response shape; keep it byte-compatible with v0.**

### 10.5 Step 9 appeal-window write

**SOURCE:** `submit_jury_vote.rs:327-335` (the v0 case UPDATE block — being REPLACED)

**v0 (BEING REMOVED):**

```rust
// SOURCE: submit_jury_vote.rs:327-335 (DELETE)
let closed_at = now + Duration::days(APPEAL_WINDOW_DAYS);
update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
        moderation_case::status.eq(CaseStatus::Decided),
        moderation_case::decided_at.eq(Some(now)),
        moderation_case::closed_at.eq(Some(closed_at)),
    ))
    .execute(conn)
    .await?;
```

**JM-c REPLACEMENT (split into two updates — status/decided at step 8, appeal_window at step 9):**

```rust
// JM-c step 8: case-status flip + decided_at — REMOVES closed_at write
update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set((
        moderation_case::status.eq(CaseStatus::Decided),
        moderation_case::decided_at.eq(Some(now)),
    ))
    .execute(conn)
    .await?;

// ... rest of step 8 (sanction was at step 6/7; public_case_log + governance_log
// + reputation_event writes here, all unchanged) ...

// JM-c step 9: appeal-window expiry write — LIVE config read
let window_days_i64 = config::get_int(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "appeal.window_days",
)
.await?;
let appeal_window_expires_at = now + Duration::days(window_days_i64);
update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set(moderation_case::appeal_window_expires_at.eq(Some(appeal_window_expires_at)))
    .execute(conn)
    .await?;
```

**GOTCHA:** `Duration::days` accepts `i64` per `chrono::Duration` API — the LIVE config read returns `i64` already (matches the type signature of `config::get_int`). No cast needed; no R1 trip.

**GOTCHA:** `cache` is the `ConfigCache` already constructed at handler start (per `submit_jury_vote.rs` ConfigCache lifetime — verify at impl time; if there's no existing cache in the function, JM-c needs to construct one). The cache is created once per handler invocation and shared across all reads in that invocation.

**GOTCHA:** the two-UPDATE shape (one at step 8 for status/decided_at, one at step 9 for appeal_window) is two round trips. Acceptable: the post-decision block already does many writes (sanction, public_case_log, multiple governance_log emissions, multiple reputation_event inserts, optional federation publish), so adding one more UPDATE inside the same transaction is not a hot-path concern. The alternative (single UPDATE that sets all four columns at once, requiring `appeal.window_days` to be read before the case-status flip) couples step 8 and step 9 in a way that makes SL-d's graft harder — SL-d's sponsor-liability path will set `case.status = SponsorLiabilityPending` instead of Decided, and SL-d's status flip will need to land BEFORE the appeal_window write because the appeal_window write uses `decided_at` which the sponsor path would set to a different timestamp. Keeping the two writes separate makes SL-d's diff cleaner.

### 10.6 Step 7 sponsor-liability TODO comment (load-bearing for SL-d graft)

**SOURCE:** `submit_jury_vote.rs:309-322` (the v0 `apply_sponsor_liability` call site)

**JM-c addition (immediately ABOVE the existing `apply_sponsor_liability` call):**

```rust
// SOURCE: NEW in JM-c — load-bearing TODO at the SL-d graft point
//
// TODO(v1-sponsor-liability-d): replace this v0 apply_sponsor_liability call with the
// compute/fire split per .claude/PRPs/prds/v1-sponsor-liability.prd.md §9.1 + §9.3:
//   - compute_sponsor_liability(...) returns deltas (no event rows yet)
//   - flip case.status = CaseStatus::SponsorLiabilityPending
//   - set case.grace_expires_at = now + grace_window_for_severity(severity)
//   - emit governance_log entry sponsor_liability_pending
//   - notify_sponsor_of_pending_liability(...) for each delta
//   - DEFER public_case_log + juror reputation_events to scheduler fire/escape time
//
// The current v0 apply_sponsor_liability stays in place for JM-c — SL-d is the rewrite.
// JM-c's appeal_window_expires_at write at step 9 fires on BOTH this v0 path AND the
// (future) sponsor-liability path; SL-d must preserve that semantic.

if let Some(target_id) = case_row.target_person_id {
    sponsor_liability::apply_sponsor_liability(...).await?;
}
```

**GOTCHA:** the comment text is load-bearing — SL-d's plan author will grep for `TODO(v1-sponsor-liability-d)` to find the graft anchor. Do not paraphrase or shorten. Use the exact strings: `TODO(v1-sponsor-liability-d)`, `compute_sponsor_liability`, `SponsorLiabilityPending`, `grace_window_for_severity`, `sponsor_liability_pending`, `notify_sponsor_of_pending_liability`. These are SL-d's named-symbol contracts.

### 10.7 Test fixture reuse (R2 — JM-b retro §3.2 amendment 2)

**SOURCE:** `crates/server/tests/e2e.rs:7098-7160` (`v1_jm_b_fixtures`)

```rust
// SOURCE: crates/server/tests/e2e.rs:7098 (PUBLIC, REUSABLE)
pub async fn seed_jury_eligible_snapshots(
    conn: &mut AsyncPgConnection,
    persons: &[PersonId],
) -> LemmyResult<()> {
    seed_jury_eligible_snapshots_scoped(conn, persons, None).await
}

// SOURCE: crates/server/tests/e2e.rs:7112 (PUBLIC, REUSABLE)
pub async fn seed_jury_eligible_snapshots_scoped(
    conn: &mut AsyncPgConnection,
    persons: &[PersonId],
    community_id: Option<CommunityId>,
) -> LemmyResult<()> {
    for person in persons {
        let form = ReputationSnapshotInsertForm {
            person_id: *person,
            community_id,
            jury_eligible: true,
            reporting_accuracy: 100,
            jury_reliability: 100,
            ..Default::default()
        };
        insert_into(reputation_snapshot::table)
            .values(&form)
            .execute(conn)
            .await?;
    }
    Ok(())
}
```

**JM-c rule (R2 — MANDATORY for every test that asserts on jury-tally outcomes):** any e2e test that exercises the `submit_jury_vote` happy path (where jurors are seated and votes tallied) MUST call `v1_jm_b_fixtures::seed_jury_eligible_snapshots(conn, &juror_ids)` BEFORE the `admin_assign_jury` call. Without this, `admin_assign_jury` triggers the small-pool-fallback path (per JM-b retro §2.2), the panel is seated via legacy fallback, and the snapshot fields get populated with non-steady-state values. JM-c tests that DON'T seed snapshots will hit the same constraint-record-relaxation surprise JM-b retro documented at length.

**Reuse pattern (no re-derivation):**

```rust
// SOURCE: NEW in JM-c — reuse v1_jm_b_fixtures verbatim
use crate::v1_jm_b_fixtures::{
    bootstrap, seed_user, seed_community, seed_jurors, seed_case,
    seed_jury_eligible_snapshots, seed_founder_event,
};

#[tokio::test]
async fn submit_jury_vote_severe_panel_meets_threshold() -> LemmyResult<()> {
    let (_pg, db_url, ctx) = bootstrap().await?;
    let admin = seed_user(&ctx, instance_id, "admin", true).await?;
    let community = seed_community(&ctx, instance_id, "test").await?;
    let target = seed_user(&ctx, instance_id, "target", false).await?;
    let jurors = seed_jurors(&ctx, instance_id, 7).await?;  // 7-juror Severe panel

    // R2: seed reputation_snapshot BEFORE admin_assign_jury
    {
        let mut conn = AsyncPgConnection::establish(&db_url).await?;
        seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
    }

    let case_id = seed_case(&db_url, target.person.id, SeverityTier::Severe).await?;
    admin_assign_jury(Json(AdminAssignJury { case_id }), ctx.clone(), admin_view).await?;

    // Cast 5 Sanction votes (threshold_count_snapshot for Severe = 5).
    for juror_id in &jurors[..5] {
        let juror_view = lookup_local_user_view(&ctx, *juror_id).await?;
        submit_jury_vote(
            Json(SubmitJuryVote {
                case_id,
                decision: JuryDecision::RemoveContent,
                rationale: Some("test".to_string()),
            }),
            ctx.clone(),
            juror_view,
        )
        .await?;
    }

    // Assert decision fired.
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let case_status: CaseStatus = moderation_case::table
        .filter(moderation_case::id.eq(case_id))
        .select(moderation_case::status)
        .first(&mut conn)
        .await?;
    assert_eq!(case_status, CaseStatus::Decided);

    Ok(())
}
```

### 10.8 Concurrency test pattern (NO PRECEDENT — established by JM-c)

**SOURCE:** No existing concurrency test in `crates/server/tests/e2e.rs` (per Explore agent #3 finding 12). JM-c establishes the pattern.

**JM-c pattern (used in test `submit_jury_vote_concurrent_votes_decide_exactly_once`):**

```rust
// SOURCE: NEW in JM-c (no precedent in e2e.rs)
// Pattern: tokio::join! TWO calls to submit_jury_vote that BOTH would meet
// threshold. The FOR UPDATE lock at submit_jury_vote.rs:243 serializes the
// two votes through the post-decision block. Both votes record rows; the
// idempotency guard (line 265) ensures the second vote sees Decided status
// and returns case_decided: true without re-running the post-decision block.
//
// We can assert exactly-once behaviour by counting:
//   - 5 jury_vote rows (both racing votes record)
//   - 1 sanction row (post-decision block ran once)
//   - 1 case_decided governance_log entry
//   - 4 juror reputation_event rows (one per casting juror; reporter event
//     is a separate write — total 5 reputation_event rows for a 5-juror panel
//     when reporter is set; in this test we set reporter to None for clarity)

#[tokio::test]
async fn submit_jury_vote_concurrent_votes_decide_exactly_once() -> LemmyResult<()> {
    let (_pg, db_url, ctx) = bootstrap().await?;
    // ... seed admin, community, target, 5 jurors, snapshots ...
    let case_id = seed_case(&db_url, target.person.id, SeverityTier::Minor).await?;
    admin_assign_jury(...).await?;

    // Cast 3 votes serially (threshold for Minor = 3 via PRD §10).
    for juror_id in &jurors[..2] {
        let juror_view = lookup_local_user_view(&ctx, *juror_id).await?;
        submit_jury_vote(Json(SubmitJuryVote { case_id, decision: JuryDecision::RemoveContent, rationale: None }), ctx.clone(), juror_view).await?;
    }
    // 2 votes cast; threshold not yet met. Now race jurors 3 and 4.

    let ctx_a = ctx.clone();
    let view_a = lookup_local_user_view(&ctx, jurors[2]).await?;
    let ctx_b = ctx.clone();
    let view_b = lookup_local_user_view(&ctx, jurors[3]).await?;

    let (res_a, res_b) = tokio::join!(
        submit_jury_vote(
            Json(SubmitJuryVote { case_id, decision: JuryDecision::RemoveContent, rationale: None }),
            ctx_a,
            view_a,
        ),
        submit_jury_vote(
            Json(SubmitJuryVote { case_id, decision: JuryDecision::RemoveContent, rationale: None }),
            ctx_b,
            view_b,
        ),
    );

    // BOTH calls must succeed — vote records, idempotency guard handles the late one.
    res_a?;
    res_b?;

    // Assert exactly-once post-decision side effects.
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let vote_count: i64 = jury_vote::table.filter(jury_vote::case_id.eq(case_id)).count().get_result(&mut conn).await?;
    let sanction_count: i64 = sanction::table.filter(sanction::case_id.eq(case_id)).count().get_result(&mut conn).await?;
    let case_decided_log_count: i64 = governance_log::table
        .filter(governance_log::entry_kind.eq("case_decided"))
        // ... narrow by case_id via payload JSONB query ...
        .count().get_result(&mut conn).await?;

    assert_eq!(vote_count, 4, "all 4 votes recorded (2 sequential + 2 concurrent)");
    assert_eq!(sanction_count, 1, "post-decision block ran exactly once");
    assert_eq!(case_decided_log_count, 1, "case_decided emitted exactly once");

    Ok(())
}
```

**GOTCHA:** `tokio::join!` runs both futures on the same task by default (cooperative scheduling). The FOR UPDATE lock guarantees serialization at the database layer regardless of task scheduling. If the test is flaky, it's a sign the lock semantics don't hold — escalate to advisor; do NOT add `tokio::time::sleep` workarounds.

**GOTCHA:** `lookup_local_user_view` is a helper to be defined in `v1_jm_b_fixtures` (or in JM-c's test block) — it loads a `LocalUserView` for a given `PersonId`. If not present, JM-c task wording specifies adding it as part of Task 6.

### 10.9 v0-compat regression test pattern (PRD §11 canonical)

**SOURCE:** No existing implementation; JM-c is the OWNER per PRD §11 + §17 row 3. The pattern below is the canonical shape.

**JM-c test (`v0_case_completes_under_v0_rules_after_v1_config_flip`):**

```rust
// SOURCE: NEW in JM-c — owner per PRD §11
// Asserts: a case opened under v0 defaults (panel=5, threshold_count=3,
// appeal_window=7) completes under those snapshot values even after the
// admin flips the config to wildly different values mid-flight.
//
// This test is the load-bearing assertion for ADR-010 (no retroactive
// invalidation of in-flight juries).

#[tokio::test]
async fn v0_case_completes_under_v0_rules_after_v1_config_flip() -> LemmyResult<()> {
    let (_pg, db_url, ctx) = bootstrap().await?;
    let admin = seed_user(&ctx, instance_id, "admin", true).await?;
    let target = seed_user(&ctx, instance_id, "target", false).await?;
    let jurors = seed_jurors(&ctx, instance_id, 5).await?;  // v0 panel size

    {
        let mut conn = AsyncPgConnection::establish(&db_url).await?;
        seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
    }

    // Step 1: open case under v0 defaults. Severity defaults to Minor.
    let case_id = seed_case(&db_url, target.person.id, SeverityTier::Minor).await?;
    admin_assign_jury(Json(AdminAssignJury { case_id }), ctx.clone(), admin_view).await?;

    // Snapshot fields should be: panel=5, quorum=3, threshold_count=3.
    {
        let mut conn = AsyncPgConnection::establish(&db_url).await?;
        let snap: (Option<i32>, Option<i32>, Option<i32>) = moderation_case::table
            .filter(moderation_case::id.eq(case_id))
            .select((moderation_case::panel_size_snapshot, moderation_case::quorum_snapshot, moderation_case::threshold_count_snapshot))
            .first(&mut conn).await?;
        assert_eq!(snap, (Some(5), Some(3), Some(3)), "v0-default snapshot values");
    }

    // Step 2: admin flips config WHILE case is in InReview.
    admin_set_config(Json(AdminSetConfig {
        key: "jury.panel_size.regular.minor".to_string(),
        value_type: "int".to_string(),
        value: json!(11),
        scope: "instance".to_string(),
        community_id: None,
    }), ctx.clone(), admin_view.clone()).await?;
    admin_set_config(Json(AdminSetConfig {
        key: "jury.threshold_fraction.minor".to_string(),
        value_type: "float".to_string(),
        value: json!(0.95),
        scope: "instance".to_string(),
        community_id: None,
    }), ctx.clone(), admin_view.clone()).await?;
    admin_set_config(Json(AdminSetConfig {
        key: "appeal.window_days".to_string(),
        value_type: "int".to_string(),
        value: json!(60),
        scope: "instance".to_string(),
        community_id: None,
    }), ctx.clone(), admin_view.clone()).await?;

    // Step 3: cast 3 RemoveContent votes (v0 threshold = 3, NOT the new 0.95 * 11 = 11).
    for juror_id in &jurors[..3] {
        let juror_view = lookup_local_user_view(&ctx, *juror_id).await?;
        submit_jury_vote(Json(SubmitJuryVote {
            case_id,
            decision: JuryDecision::RemoveContent,
            rationale: None,
        }), ctx.clone(), juror_view).await?;
    }

    // Step 4: assert case decided under v0 rules.
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (status, decided_at, appeal_expires): (CaseStatus, Option<DateTime<Utc>>, Option<DateTime<Utc>>) = moderation_case::table
        .filter(moderation_case::id.eq(case_id))
        .select((moderation_case::status, moderation_case::decided_at, moderation_case::appeal_window_expires_at))
        .first(&mut conn).await?;

    assert_eq!(status, CaseStatus::Decided, "decided despite mid-flight config change");
    assert!(decided_at.is_some(), "decided_at populated");
    let expires = appeal_expires.expect("appeal_window populated");
    let decided = decided_at.expect("decided_at populated");
    let window_diff = expires - decided;
    assert_eq!(window_diff.num_days(), 60, "appeal_window uses LIVE config (60), not snapshotted v0 default (7)");

    Ok(())
}
```

**GOTCHA:** the assertion `appeal_window uses LIVE config (60)` is the load-bearing differentiation between snapshotted-rules (panel/quorum/threshold) and live-read-rules (appeal_window). PRD §9.1 cross-references explicitly carve out this distinction; the test makes it executable.

### 10.10 Governance-log registry update

**SOURCE:** `.claude/rules/governance-log-entry-kind-registry.md` (existing v1-JM-a section)

**JM-c addition (append a new row in the `## §JM-c` section, or extend the `## §v1-JM-a` table with a "JM-c — added" subsection if the rule's structure is per-phase tables — confirm at impl time):**

```markdown
| `ENTRY_KIND_JURY_DEADLOCK` | `jury_deadlock` | v1-JM-c | `crates/api/api/src/governance/submit_jury_vote.rs` (deadlock branch in process_vote) | Jury panel reached `panel_size_snapshot` votes but no `JuryDecision` met `threshold_count_snapshot`. Case flipped to `CaseStatus::AdminReview`. Payload: `{ case_id, panel_size_snapshot, threshold_count_snapshot, tally: {<JuryDecision>: count, ...} }` |
```

Format must mirror existing rows verbatim (column count, ordering, capitalisation).

---

## 11. Files to change

| File | Action | Justification |
|---|---|---|
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | Add `pub const ENTRY_KIND_JURY_DEADLOCK: &str = "jury_deadlock";` with full doc comment per §10.1 |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Add `ENTRY_KIND_JURY_DEADLOCK` to the `pub use` re-export list per §10.1 |
| `crates/api/api/src/governance/submit_jury_vote.rs` | UPDATE | Core JM-c work: delete `QUORUM` + `APPEAL_WINDOW_DAYS` consts; rewrite step 5 (snapshot reads + per-decision threshold + deadlock branch); add step 7 TODO comment; remove `closed_at` write at step 8; add step 9 appeal_window write; delete `pick_majority` helper; preserve everything else verbatim |
| `crates/server/tests/e2e.rs` | UPDATE | Add 6 new tests per §14; add `lookup_local_user_view` helper if not present |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | Add `ENTRY_KIND_JURY_DEADLOCK` row per §10.10 |
| `.claude/PRPs/reports/v1-JM-c-retro.md` | CREATE | End-of-phase retro per JM PRD §17 + JM-b retro template |
| `.claude/PRPs/reports/v1-JM-c-retro-events.md` | CREATE | Mid-phase events log (advisor writes during impl) |

**Files explicitly NOT touched:**

- `crates/api/api/src/governance/admin_assign_jury.rs` — JM-b territory; JM-c reads its outputs only
- `crates/api/api/src/governance/admin_emergency_remove.rs` — JM-b PR #95 cr-3 fix landed; do not re-touch
- `crates/api/api/src/governance/request_appeal.rs` — JM-d territory
- `crates/api/api/src/governance/accept_jury_assignment.rs`, `decline_jury_assignment.rs` — no edits needed (they don't read snapshot fields)
- `crates/api/api/src/governance/config.rs` — no new helpers; JM-c uses existing `config::get_int`
- `crates/api/api/src/governance/jury_common.rs` — no new helpers
- `crates/db_schema/src/source/governance/moderation_case.rs` — schema is JM-a's; JM-c reads existing fields
- `crates/db_schema_file/src/schema.rs` — no migration; no schema regen
- `migrations/**` — JM-c is handler-only; no migrations
- `crates/api/api_common/src/governance.rs` — `SubmitJuryVote` request + `SubmitJuryVoteResponse` response shapes preserved verbatim per §4.1 / §10.4 GOTCHA
- `crates/api/routes/src/governance.rs` — no new endpoints
- `crates/server/src/governance.rs` — no background jobs

---

## 12. NOT building in v1-JM-c

Explicit scope limits — these belong to later sub-phases or are out-of-scope deferrals:

- **Appeal handler rewrites** (`request_appeal.rs` bounded-window check, reporter-rights extension, auto-rejury, `select_appeal_panel`, `admin_trigger_appeal_rejury`, appeal-window-expiry background job) — JM-d territory per PRD §9.3-§9.5.
- **Sponsor-liability compute/fire split** at step 7 — SL-d territory per `v1-sponsor-liability.prd.md §9.1 / §9.3`. JM-c writes the TODO comment marking the graft point; SL-d does the rewrite.
- **Vote-outcome / evidence-quality emitters** at step 7 — rep-tuning-r3 territory.
- **Case-open severity inference for non-emergency paths** — OQ-V1-JM-07 / DQ #47 (planner-pending). v1.5 candidate. JM-c does NOT add `severity_tier` writers; non-emergency cases inherit JM-a DEFAULT 'Minor' until v1.5.
- **Step-up auth for severity-tier changes mid-case** (PRD §12.3) — JM-e territory; JM-c does not introduce `step_up_token` DTO field.
- **Cleaning up the v0 literal `"jury_vote_submitted"`** at line 191 — separate carry-forward; per §4.2 rejected alternative, would silently break log consumers without a coordinated backfill.
- **Cross-instance jury federation** (PRD §17 row 5 / §2 OUT) — v2 territory.
- **Composable diversity constraint priority** via `jury.constraint_priority_list` — v1.5 candidate (carry-forward from JM-b retro §3.1 row 2).
- **`no_same_endorsement_chain` constraint wiring** — v1.5 candidate (carry-forward from JM-b retro §3.1 row 1).
- **Replacement of small-pool fallback with explicit operator notification** — v1.5 candidate (carry-forward from JM-b retro §3.1 row 3).
- **Issue #96 carry-forward findings** (cr-1, cr-2, cr-5, cr-6, cr-7, cr-8, cr-10) — explicitly out of JM-c scope. If JM-c's edits naturally touch the code paths in cr-5 (`config.rs` candidate-level const fallback) or cr-8 (`decline_jury_assignment` ConstraintRecord persist), the impl MUST NOT silently fold them in. See §19 for the exact policy.
- **Adding a `deadlock: bool` field to `SubmitJuryVoteResponse`** — per §10.4 GOTCHA, not needed; existing `case_decided: false, decision: None` already conveys the deadlock case to clients (clients distinguish deadlock from partial-tally via governance_log queries, not via response shape).
- **Migrating reputation deltas to snapshotted reads** — out of scope; deltas at lines 366-378 of submit_jury_vote.rs continue to be LIVE config reads per JM-a precedent.

---

## 13. Step-by-step tasks

Execute in order. One commit per task. Each task has MIRROR refs, exact file paths, and validation commands. **All cargo commands use the canonical flag set `--workspace --features full --no-deps -- -D warnings` for clippy (R6 — JM-b retro-events Event 3) and capture to `.claude/PRPs/debug/v1-JM-c-task<N>-<probe>.log` with explicit exit-code preservation per `.claude/rules/cargo-output-capture.md`.**

### Task 0: Pre-flight harness audit + branch verification + JM-b state confirmation

**Goal:** verify environment is ready for JM-c impl; confirm branch is `phase-v1-JM-c`; confirm JM-b's snapshot machinery is intact on the base; confirm pre-existing clippy baseline is clean.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly, do NOT inherit implicitly per JM-b retro-events Event 4):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-JM-c-task0-probe1.log 2>&1"
echo "probe 1 exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task0-probe1.log
# EXPECT: only lemmy_utils compiles; exit 0

# Probe 2 — wrapper sanity (--features full activation)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-JM-c-task0-probe2.log 2>&1"
echo "probe 2 exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task0-probe2.log
# EXPECT: features full activated; exit 0

# Probe 3 — wrapper sanity (cargo-test honors --test e2e --no-run)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task0-probe3.log 2>&1"
echo "probe 3 exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task0-probe3.log
# EXPECT: only e2e binary builds; exit 0

# Probe 4 — wrapper exit-code propagation (negative test)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-JM-c-task0-probe4-neg.log 2>&1"
echo "probe 4 exit on bogus feature: $?"
# EXPECT: NON-ZERO exit (101 typical); log contains "the package 'lemmy_server' does not contain this feature"

# Probe 5 — clippy baseline (R5: gate-keeper-drop fix per JM-b retro-events Event 1)
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task0-clippy-baseline.log 2>&1"
echo "clippy baseline exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task0-clippy-baseline.log
# EXPECT: exit 0. If non-zero: list each error against the rule's two-paths fix
# (in-pre-phase-commit OR narrow-DoD-via-plan-amendment); pick BEFORE Task 1.

# Probe 6 — DoD smoke: per-task validation commands run against branch HEAD
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task0-dod-check.log 2>&1"
echo "DoD check exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task0-dod-check.log
# EXPECT: exit 0 (JM-b merged green; nothing new yet)

# Probe 7 — PM-plugin-hooks-stable check (no PM hook touches in JM-c scope)
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done
echo "PM hook literals all present"
# EXPECT: all 6 hooks present; JM-c does not touch this file class

# Probe 8 — concurrent-PR check (no other PR touches submit_jury_vote.rs)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files --jq '.[] | select(.files[]?.path | contains("submit_jury_vote.rs")) | {number, title, headRefName}'
# EXPECT: empty output; if non-empty, STOP and reconcile (file ownership conflict)

# Probe 9 — JM-b state confirmation: snapshot fields + entry-kind consts present
git log --oneline governance-v0 -10
# EXPECT: top commits include 4d2b93ed9 (PR #95 merge) and ENTRY_KIND_SEVERITY_TIER_FROZEN

rg -n "ENTRY_KIND_SEVERITY_TIER_FROZEN" crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 1 match (JM-a-shipped const)

rg -n "panel_size_snapshot|quorum_snapshot|threshold_count_snapshot" crates/db_schema/src/source/governance/moderation_case.rs
# EXPECT: 3 matches (JM-a-shipped fields)
```

**EXPECT block:**
- Probes 0, 1, 2, 3, 5, 6, 7, 9 exit 0
- Probe 4 exits NON-ZERO (negative test confirms exit-code propagation)
- Probe 8 returns empty
- If clippy baseline (Probe 5) is non-zero: STOP and either fix in pre-phase commit OR narrow plan DoD before Task 1

**No commit at Task 0** — this is verification only. If anything fails, STOP and surface to advisor before Task 1.

### Task 1: Add `ENTRY_KIND_JURY_DEADLOCK` const + re-export + registry row

**ACTION:** Declare the new const in `crates/db_schema/src/source/governance/governance_log.rs`; re-export through the api shim at `crates/api/api/src/governance/governance_log.rs`; add the registry row in `.claude/rules/governance-log-entry-kind-registry.md`.

**IMPLEMENT (file 1 of 3):** in `crates/db_schema/src/source/governance/governance_log.rs`, add the const declaration alphabetically among the other ENTRY_KIND_* consts (likely after `ENTRY_KIND_JURY_CONSTRAINT_RELAXED`, before `ENTRY_KIND_JURY_VOTED`). Use the full doc comment from §10.1.

**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/governance/governance_log.rs`, add `ENTRY_KIND_JURY_DEADLOCK` to the `use lemmy_db_schema::source::governance::governance_log::{...}` import list (alphabetical, after `ENTRY_KIND_JURY_CONSTRAINT_RELAXED`).

**IMPLEMENT (file 3 of 3):** in `.claude/rules/governance-log-entry-kind-registry.md`, append a row per §10.10 to the v1-JM-a section (or create a `## §v1-JM-c` subsection if the registry uses per-phase grouping — confirm at impl time by reading the existing structure).

**MIRROR:** `crates/db_schema/src/source/governance/governance_log.rs:170-178` (`ENTRY_KIND_SEVERITY_TIER_FROZEN` declaration) for the const + doc-comment shape.

**GOTCHA:** the const declaration and the re-export are two separate files in two separate crates. Both edits land in this single Task 1 commit.

**GOTCHA:** if the registry rule's table structure differs from §10.10's column ordering, follow the existing rule's structure exactly. Format-faithfulness matters for the registry's grep-ability.

**VALIDATE:**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task1-check.log
# EXPECT: exit 0. New const compiles; no other changes touch source.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task1-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task1-clippy.log
# EXPECT: exit 0. No new lint debt.

# R7: also verify test target still compiles (this task touches a re-export module
# that test fixtures import).
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task1-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task1-test-no-run.log
# EXPECT: exit 0. Test target compiles cleanly (no E0432 import errors from registry change).
```

**COMMIT MESSAGE:**
```
feat(v1-JM-c): add ENTRY_KIND_JURY_DEADLOCK const + registry row (task 1)

Declare the canonical entry-kind const for the JM-c deadlock path; re-export through
the api shim; document the call-site contract in the governance-log registry. The const
is consumed by submit_jury_vote.rs::process_vote at the deadlock branch (added in
task 3) when all jurors have voted but no JuryDecision met threshold_count_snapshot.

Per .claude/rules/governance-log-entry-kind-registry.md, every new entry kind needs
both a canonical const + a registry row + a documented call site. The call site lands
in task 3.

Refs: PRD §9.1 step 5 (deadlock semantics); JM-b retro §3.3 (no other ENTRY_KIND
consts to add in JM-c beyond JURY_DEADLOCK).
```

### Task 2: Replace `QUORUM` + `APPEAL_WINDOW_DAYS` consts with snapshot reads

**ACTION:** In `crates/api/api/src/governance/submit_jury_vote.rs`:
1. DELETE `const QUORUM: i64 = 3;` at line 86
2. DELETE `const APPEAL_WINDOW_DAYS: i64 = 7;` at line 88
3. Replace the line 202-207 vote-count gate to read `case.quorum_snapshot` (with `ok_or` per §10.2)
4. The actual case load at line 240 happens AFTER the vote-count gate currently — for JM-c we need to load the case row BEFORE the vote-count gate so the snapshot is available. Move the FOR UPDATE case load to before the vote count, OR add a separate non-FOR-UPDATE snapshot read at the top of the gate. **Decision: load case row WITH FOR UPDATE earlier, before the vote count gate.** This serializes earlier (slightly), but it's necessary to read the snapshot before the gate.
5. Read the three snapshot fields (`quorum_snapshot`, `threshold_count_snapshot`, `panel_size_snapshot`) once after the case load and reuse throughout.

**IMPLEMENT:**
- Move the FOR UPDATE case load (currently lines 240-245) to immediately after the assignment check + vote insert + log emission (before the existing vote-count gate at lines 202-207).
- After the case load, add the three `ok_or` snapshot reads per §10.2.
- Replace the vote-count gate at lines 202-207 from `if vote_count < QUORUM` to `if vote_count < i64::from(quorum_snapshot)`.
- DELETE the two const declarations.
- The existing idempotency guard at lines 265-278 stays exactly as-is (preserves `CaseStatus::EmergencyRemove` exhaustive match per ADR-013).

**MIRROR:** §10.2 pattern verbatim.

**GOTCHA (R1):** the comparison `vote_count < i64::from(quorum_snapshot)` uses `i64::from(i32)` — lossless widening, NOT flagged by `clippy::as_conversions`. Do NOT use `quorum_snapshot as i64` (would trip lint).

**GOTCHA:** moving the case load earlier means the FOR UPDATE lock is held longer (across the entire vote-tally + idempotency-guard + post-decision block). This is intentional — it's the same lock-coverage as v0; we're just acquiring it sooner. Connection-level concurrency on the same case is already serialized by FOR UPDATE.

**GOTCHA:** the v0 code at lines 202-207 returns early if `vote_count < QUORUM` BEFORE checking idempotency. This is a v0 optimization that survives JM-c — partial tallies (`vote_count < quorum_snapshot`) early-return without taking the FOR UPDATE lock... wait, with the move above the FOR UPDATE comes BEFORE the gate. Reconsider: hold the lock for the whole tally path (correct for serialization) OR keep the gate before the lock (correct for partial-tally fast path). Since post-JM-b the snapshot is always populated, we could read snapshot fields without FOR UPDATE — but then we'd need a second non-locking read of the case row just for the snapshot.

**RESOLUTION (in plan):** keep the FOR UPDATE lock at its current position (lines 240-245) AFTER the early-return gate. For the gate, read the case row via a separate non-locking `SELECT moderation_case::quorum_snapshot WHERE id = ?` BEFORE the gate. This preserves the v0 fast-path for partial tallies while still reading the snapshot. Two queries (one to read snapshot, one to FOR UPDATE), but that's the right shape.

**REVISED IMPLEMENT:**
- BEFORE the existing vote-count gate at line 202: add `let quorum_snapshot: i32 = moderation_case::table.filter(moderation_case::id.eq(data.case_id)).select(moderation_case::quorum_snapshot).first::<Option<i32>>(conn).await?.ok_or(...)?;` (non-locking single-column read).
- Replace the gate to `if vote_count < i64::from(quorum_snapshot)`.
- DELETE the QUORUM const.
- The threshold_count_snapshot + panel_size_snapshot reads happen later, after the FOR UPDATE case load — those fields are needed for the per-decision threshold + deadlock path (Task 3).
- DELETE the APPEAL_WINDOW_DAYS const NOW (it's not used anywhere else); the live config read for window_days lands in Task 5.

**VALIDATE:**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task2-check.log
# EXPECT: exit 0.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task2-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task2-clippy.log
# EXPECT: exit 0. No new lint (R1 guarded via i64::from).

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task2-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task2-test-no-run.log
# EXPECT: exit 0. Tests still compile (no JuryAssignmentInsertForm-style drift in JM-c scope).
```

**COMMIT MESSAGE:**
```
feat(v1-JM-c): replace QUORUM/APPEAL_WINDOW_DAYS consts with snapshot reads (task 2)

Delete the v0 hardcoded constants from submit_jury_vote.rs. The vote-count gate now
reads case.quorum_snapshot (written by JM-b at admin_assign_jury time) instead of
the const. APPEAL_WINDOW_DAYS removal is unblocked here; the live config read at
step 9 lands in task 5.

The snapshot read uses ok_or to fail loudly on a NULL snapshot — per ADR-010 + the
JM-a backfill, every case in JurySelection or later has populated snapshot fields,
so a NULL would indicate a process breach (admin_assign_jury didn't run) rather than
a normal code path.

Refs: PRD §9.1 step 5 (snapshot reads); ADR-010 (no retroactive invalidation);
JM-b retro §3.3 (snapshots populated handoff).
```

### Task 3: Step 5 — per-decision threshold + deadlock branch

**ACTION:** Rewrite the v0 vote-tally + decision-pick logic at lines 216-278 (covering the simple-majority HashMap pick + the existing case load + idempotency guard) to:
1. Load case row WITH FOR UPDATE (existing position, lines 240-245 — preserved verbatim)
2. Idempotency guard (lines 265-278) — preserved verbatim, including exhaustive `CaseStatus::EmergencyRemove` match
3. Read `threshold_count_snapshot` + `panel_size_snapshot` from `case_row` after the idempotency guard (snapshot reads per §10.2)
4. Per-decision threshold tally per §10.3 (replaces lines 216-231 + the `pick_majority` call at line 231)
5. Deadlock branch per §10.4 (NEW — sits between the per-decision tally and the existing `winning_decision` propagation to step 6)
6. Delete the v0 `pick_majority` helper at lines 510-518

**IMPLEMENT:** see §10.3 + §10.4 for the exact patterns. The structure at lines 216-278 becomes:
- Lines 216-245: case load + idempotency guard (unchanged)
- After line 278: snapshot reads + per-decision tally + winning_decision result
- If winning_decision is None and vote_count == panel_size_snapshot: deadlock branch (§10.4) → return early
- If winning_decision is None and vote_count < panel_size_snapshot: partial-tally early return
- If winning_decision is Some: continue to step 6 (existing sanction-insert block)
- Delete the helper at lines 510-518

**MIRROR:** §10.3 (per-decision tally) + §10.4 (deadlock + governance_log emission) + admin_assign_jury.rs:211-224 (governance_log::append signature pattern)

**GOTCHA (R1):** every comparison between `i64` (vote count) and the snapshot fields (`i32`) uses `i64::from(...)`. NEVER use bare `as` casts.

**GOTCHA:** the `juror_pseudonym` for the deadlock log entry comes from line 189-194 of v0 (where the `actor_pseudonym_helper::get_or_create` for the casting juror is already done for the `jury_vote_submitted` log entry). Reuse the existing `juror_pseudonym` variable; do not call `get_or_create` twice.

**GOTCHA:** the deadlock UPDATE writes `status = AdminReview` ONLY (no `decided_at`, no `closed_at`, no `appeal_window_expires_at`). Per §10.4 GOTCHA.

**GOTCHA (exhaustiveness):** the per-decision iteration in §10.3 hardcodes 8 `JuryDecision` variants. The compile-time exhaustiveness anchor is `map_decision_to_sanction` at lines 523-549 (which uses an exhaustive match — adding a 9th variant fails to compile there first). Add the comment from §10.3 inline so future maintainers know to update both lists together.

**VALIDATE:**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task3-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task3-check.log
# EXPECT: exit 0.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task3-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task3-clippy.log
# EXPECT: exit 0.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task3-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task3-test-no-run.log
# EXPECT: exit 0.
```

**COMMIT MESSAGE:**
```
feat(v1-JM-c): step 5 — per-decision threshold + deadlock-to-AdminReview (task 3)

Replace the v0 simple-majority HashMap pick with a per-decision threshold tally that
reads case.threshold_count_snapshot (JM-b-written). Iterate JuryDecision variants in
stable enum-order; the first decision meeting threshold wins. When no decision meets
threshold AND all panel members have voted, flip status to AdminReview and emit
ENTRY_KIND_JURY_DEADLOCK governance_log entry.

The deadlock UPDATE writes status only (no decided_at, no closed_at, no
appeal_window_expires_at). Deadlocked cases sit in AdminReview pending human
intervention — no subsequent case_decided / sanction_created / public_log_published
fires for them.

Idempotency guard at the FOR UPDATE case load is preserved verbatim, including the
exhaustive CaseStatus::EmergencyRemove match (ADR-013). The deadlock terminal state
joins the idempotency guard's terminal-status list — late votes on a deadlocked case
short-circuit cleanly.

Refs: PRD §9.1 step 5; ADR-010 (snapshot semantics); ADR-013 (exhaustive CaseStatus
match); JM-b retro §3.2 amendment 1 (R1 — i64::from vs as cast).
```

### Task 4: Step 7 sponsor-liability TODO insertion-point comment

**ACTION:** In `crates/api/api/src/governance/submit_jury_vote.rs`, immediately ABOVE the existing `sponsor_liability::apply_sponsor_liability(...)` call at line ~314, add the load-bearing TODO comment per §10.6.

**IMPLEMENT:** copy §10.6's comment verbatim. Do not paraphrase or shorten.

**MIRROR:** §10.6 pattern.

**GOTCHA:** the v0 `sponsor_liability::apply_sponsor_liability(...)` call STAYS in place — JM-c does NOT delete it. SL-d is the rewrite. JM-c is just adding the comment.

**GOTCHA (R3):** SL-d will likely extend `SubmitJuryVoteResponse` (e.g., adding a `case_status: CaseStatus` field for client clarity on the new SponsorLiabilityPending state). JM-c does NOT extend the response shape — the TODO comment is the only edit at the SL-d graft point. R3 grep sweep does not apply because no struct is being extended.

**VALIDATE:**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task4-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task4-check.log
# EXPECT: exit 0. Comment-only change.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task4-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task4-clippy.log
# EXPECT: exit 0.
```

**COMMIT MESSAGE:**
```
feat(v1-JM-c): step 7 — sponsor-liability TODO insertion-point stub (task 4)

Add a load-bearing TODO comment immediately above the v0 apply_sponsor_liability call
at submit_jury_vote.rs ~line 314. The comment cites v1-sponsor-liability.prd.md §9.1 +
§9.3 verbatim so SL-d's diff can find the graft anchor via:

  git grep "TODO(v1-sponsor-liability-d)"

The v0 apply_sponsor_liability call STAYS in place — SL-d is the rewrite, not JM-c.
The TODO names the symbols SL-d will introduce (compute_sponsor_liability,
SponsorLiabilityPending, grace_window_for_severity, sponsor_liability_pending,
notify_sponsor_of_pending_liability) so SL-d's plan author can grep for them later.

Refs: v1-sponsor-liability.prd.md §9.1 + §9.3 (compute/fire split); PRD §9.1 step 7
(sponsor-liability branch); cross-PRD merge anchor.
```

### Task 5: Step 9 — appeal-window write + close-at removal

**ACTION:** In `crates/api/api/src/governance/submit_jury_vote.rs`:
1. At the v0 case UPDATE block (lines 327-335), REMOVE the `closed_at` field from the `.set((...))` tuple.
2. Add a new UPDATE block AFTER the public_case_log + reputation_event writes (between the existing reputation event loop end and the `case_decided` governance_log emission, OR after the `case_decided` emission — both work; convention follows §10.5 by placing it as step 9 *after* step 8's writes are complete).
3. The new UPDATE reads `appeal.window_days` LIVE via `config::get_int` and writes `appeal_window_expires_at = decided_at + Duration::days(window_days)`.

**IMPLEMENT:** see §10.5 for the exact pattern. The `closed_at` removal at step 8 + the new step-9 UPDATE land in the same commit since they're paired changes (per §10.5 GOTCHA: keeping them as separate UPDATEs makes SL-d's graft cleaner).

**MIRROR:** §10.5 pattern verbatim.

**GOTCHA:** `cache` (the `ConfigCache`) — verify at impl time whether the v0 handler already constructs one. If yes, reuse. If no, construct one at handler entry and pass through. The deltas read at lines 366-378 already use `cache` per the Explore agent #1 finding 11, so the cache is constructed somewhere. Reuse the existing variable.

**GOTCHA:** `Duration::days(window_days_i64)` accepts `i64` directly — no cast needed.

**GOTCHA:** the `now` variable from line 178 (the `now()` from chrono used for `submitted_at`) is reused here — the same timestamp serves as both `decided_at` and the base for `appeal_window_expires_at`. Do not call `Utc::now()` twice in the same handler invocation; consistency is load-bearing for the v0-compat regression test.

**VALIDATE:**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task5-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task5-check.log
# EXPECT: exit 0.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task5-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task5-clippy.log
# EXPECT: exit 0.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task5-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task5-test-no-run.log
# EXPECT: exit 0.
```

**COMMIT MESSAGE:**
```
feat(v1-JM-c): step 9 — appeal_window_expires_at write + closed_at removal (task 5)

Replace the v0 closed_at write (now + 7 days) with the new appeal_window_expires_at
write that reads appeal.window_days LIVE from config at decision time. This is the
single deliberate exception to the snapshot-everything rule per PRD §9.1
cross-references — the appeal window is a procedural input read at the *decision
moment*, not the *jury-seating moment*.

JM-c is the first writer of appeal_window_expires_at on post-JM-a cases (the JM-a
backfill handled pre-v1 cases via COALESCE(closed_at, decided_at + 7d)). JM-d will
be the first reader via the bounded-window appeal check.

closed_at is no longer written by submit_jury_vote — it becomes a JM-d concern
(the appeal-window-expiry background job will set closed_at = now() when
transitioning Decided → Closed).

Refs: PRD §9.1 step 9; ADR-010 cross-reference for live-read justification;
JM-b retro §3.3 (no other handlers edit appeal_window_expires_at in JM-c).
```

### Task 6: e2e tests for JM-c (6 new tests + lookup_local_user_view helper)

**ACTION:** In `crates/server/tests/e2e.rs`, add the 6 tests per §14, plus the `lookup_local_user_view` helper if not present.

**IMPLEMENT:** see §14 for the test list + assertion details. Tests reuse `v1_jm_b_fixtures` per §10.7 — do not re-derive any fixture.

**MIRROR:** §10.7 (fixture reuse pattern), §10.8 (concurrency test pattern), §10.9 (v0-compat regression test pattern), `crates/server/tests/e2e.rs:1061-1510` (golden-path test as the canonical end-to-end shape).

**GOTCHA (R2):** EVERY test that exercises submit_jury_vote MUST call `seed_jury_eligible_snapshots(conn, &juror_ids)` BEFORE `admin_assign_jury`. Without this, the small-pool fallback path in admin_assign_jury fires and the snapshot fields end up reflecting the relaxation cascade rather than the steady-state values. Per JM-b retro §3.2 amendment 2.

**GOTCHA (R4):** test names use lowercase snake_case. NO `R1`, `JM_C`, etc. Use `submit_jury_vote_concurrent_votes_decide_exactly_once`, `v0_case_completes_under_v0_rules_after_v1_config_flip`, etc.

**GOTCHA:** `lookup_local_user_view` may not exist as a helper. If absent, add it to `v1_jm_b_fixtures` (or to JM-c's test block). Sample shape:
```rust
async fn lookup_local_user_view(ctx: &LemmyContext, person_id: PersonId) -> LemmyResult<LocalUserView> {
    let mut conn = get_conn(ctx.pool()).await?;
    let local_user = local_user::table
        .filter(local_user::person_id.eq(person_id))
        .select(LocalUser::as_select())
        .first(&mut conn).await?;
    let person = person::table.filter(person::id.eq(person_id)).select(Person::as_select()).first(&mut conn).await?;
    Ok(LocalUserView { local_user, person, /* other fields */ })
}
```

**VALIDATE:**
```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task6-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task6-check.log
# EXPECT: exit 0.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task6-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-JM-c-task6-clippy.log
# EXPECT: exit 0.

# Run JM-c's new tests by name pattern.
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server submit_jury_vote_severe_panel_meets_threshold > .claude/PRPs/debug/v1-JM-c-task6-test-1.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-test-1.log
# EXPECT: exit 0; 1 passed.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server submit_jury_vote_deadlock_flips_to_admin_review > .claude/PRPs/debug/v1-JM-c-task6-test-2.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-test-2.log
# EXPECT: exit 0; 1 passed.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server submit_jury_vote_writes_appeal_window_default > .claude/PRPs/debug/v1-JM-c-task6-test-3.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-test-3.log
# EXPECT: exit 0; 1 passed.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server submit_jury_vote_writes_appeal_window_live_config > .claude/PRPs/debug/v1-JM-c-task6-test-4.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-test-4.log
# EXPECT: exit 0; 1 passed.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server v0_case_completes_under_v0_rules_after_v1_config_flip > .claude/PRPs/debug/v1-JM-c-task6-test-5.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-test-5.log
# EXPECT: exit 0; 1 passed.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server submit_jury_vote_concurrent_votes_decide_exactly_once > .claude/PRPs/debug/v1-JM-c-task6-test-6.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-test-6.log
# EXPECT: exit 0; 1 passed.
```

**COMMIT MESSAGE:**
```
test(v1-JM-c): 6 new e2e tests for snapshot-aware threshold + deadlock + appeal_window (task 6)

Added tests:
1. submit_jury_vote_severe_panel_meets_threshold — 7-juror Severe panel decides at 5
   votes (threshold_count_snapshot=5)
2. submit_jury_vote_deadlock_flips_to_admin_review — 5-juror panel split 2/2/1 →
   AdminReview + jury_deadlock log entry
3. submit_jury_vote_writes_appeal_window_default — appeal_window_expires_at populated
   on no-sponsor path with default appeal.window_days = 7
4. submit_jury_vote_writes_appeal_window_live_config — appeal_window_expires_at
   reflects mid-flight config bump (asserts LIVE read, not snapshotted)
5. v0_case_completes_under_v0_rules_after_v1_config_flip — PRD §11 canonical
   regression; case decided under v0 snapshot values despite mid-flight config flip;
   appeal_window uses LIVE config (the deliberate exception)
6. submit_jury_vote_concurrent_votes_decide_exactly_once — two jurors race to
   threshold-meeting vote; FOR UPDATE + idempotency guard ensure exactly-once
   side-effects

All tests reuse v1_jm_b_fixtures::{bootstrap, seed_user, seed_community, seed_jurors,
seed_case, seed_jury_eligible_snapshots} per JM-b retro §3.2 amendment 2 (R2). New
helper lookup_local_user_view added to v1_jm_b_fixtures (or inline if module access
restricted).

Refs: PRD §9.1 (9-step pseudocode); PRD §11 (v0-compat invariant); JM-b retro §3.2
amendments R2 (snapshot seeding) + R4 (lowercase test names).
```

### Task 7: Full-workspace validation pass

**Goal:** confirm all JM-c changes integrate cleanly across the workspace; confirm zero new clippy debt; confirm full e2e suite green; confirm no regression in pre-existing tests.

**Probes:**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-task7-check.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task7-check.log
# EXPECT: exit 0; full workspace compiles.

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-task7-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task7-clippy.log
# EXPECT: exit 0; zero new lint debt.

cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task7-e2e.log 2>&1"
echo "exit: $?"
tail -60 .claude/PRPs/debug/v1-JM-c-task7-e2e.log
# EXPECT: exit 0; 6 new tests passed; pre-existing tests still passing; report N
# passed / 0 failed / 3 ignored (the GH #43 + GH #45 + GH #42 carriers).

# Verify no struct-extension drift (R3 sweep — JM-c does NOT extend any struct,
# but verify the response shape didn't accidentally change).
git grep -l 'SubmitJuryVoteResponse {' -- ':!target' ':!.git'
# EXPECT: no new file matches beyond pre-existing call sites.
```

**EXPECT block:**
- `cargo check --workspace --features full` exit 0
- `cargo clippy --workspace --features full --no-deps -- -D warnings` exit 0
- `cargo test --test e2e -p lemmy_server` exit 0 with 6 new tests passing + all pre-existing tests passing + 3 ignored (GH #43 / #45 / #42 carriers)

**No commit at Task 7** — validation only. If anything fails, do not advance to Task 8 retro.

### Task 8: Retrospective + plan-amendment proposals for JM-d/e

**ACTION:** Write `.claude/PRPs/reports/v1-JM-c-retro.md` per the JM-b retro template + PRP `/prp-implement` Phase 5 REPORT shape. The retro documents:
- §1 What worked (handover briefs if any, plan §10 patterns, snapshot-read shape, cold-resume task-detection)
- §2 What surprised — advisor-actionable (collect all in-flight events from `v1-JM-c-retro-events.md` if advisor wrote any during impl)
- §3 What to carry forward (follow-up GH issue sketches per DQ #46; plan amendments for JM-d/e; handoff notes for JM-d)
- §4 What did NOT need fixing (worth preserving)
- §5 Quantified outcomes vs confidence score (target 9/10)
- §6 Tool-use self-assessment (Read:Edit ratio, Grep usage, Agent/Explore underuse if any)
- §7 CR finding quality (deferred to PR open per JM-b retro §7 pattern)
- §8 Suggested action items for advisor
- §9 Carry-forward specifics for JM-d/e

**MIRROR:** `.claude/PRPs/reports/v1-JM-b-retro.md` structure verbatim.

**VALIDATE:**
```bash
wc -l .claude/PRPs/reports/v1-JM-c-retro.md
# EXPECT: 200-400 lines (JM-b retro is 289 lines).

# Confirm retro file is staged for the commit.
git status --short
# EXPECT: M .claude/PRPs/reports/v1-JM-c-retro.md
```

**COMMIT MESSAGE:**
```
docs(v1-JM-c): retrospective + Task 7 validation gate (task 8)

Retrospective for v1-JM-c — submit_jury_vote 9-step rewrite. Includes:
- §1-2: what worked + what surprised during impl (if any in-flight events)
- §3: carry-forward to JM-d/e plan templates
- §5: quantified outcomes vs confidence score (target 9/10)
- §6: tool-use self-assessment
- §9: handoff notes for JM-d (appeal_window_expires_at is the new bounded-window
  appeal-check input; appeal-window-expiry background job is JM-d scope)

Task 7 full-workspace validation results recorded inline (cargo check / clippy /
e2e all exit 0; N tests passed; 3 ignored carriers preserved).

Refs: PRD §17 row 3 close; v1-JM-b retro template; DQ #46 (follow-up GH issue
sketches); JM-b retro §3.3 (handoff completed).
```

---

## 14. Testing strategy

### 14.1 Tests to add (6 new tests in `crates/server/tests/e2e.rs`)

| # | Test name | Coverage | Fixtures used | Assertion shape |
|---|---|---|---|---|
| 1 | `submit_jury_vote_severe_panel_meets_threshold` | 7-juror Severe panel decides at 5 votes (threshold=5) | `v1_jm_b_fixtures::{bootstrap, seed_user, seed_community, seed_jurors, seed_jury_eligible_snapshots, seed_case}`; `seed_case(_, _, SeverityTier::Severe)` | After 5 RemoveContent votes: `case.status == Decided`; `decided_at.is_some()`; `appeal_window_expires_at.is_some()`; `sanction_count == 1`; `case_decided governance_log entry exists` |
| 2 | `submit_jury_vote_deadlock_flips_to_admin_review` | 5-juror Minor panel split 2/2/1 → AdminReview + jury_deadlock log | `v1_jm_b_fixtures` + `seed_case(_, _, SeverityTier::Minor)` | Jurors vote: 2 RemoveContent, 2 NoAction, 1 AdvisoryLabel. After 5th vote: `case.status == AdminReview`; `decided_at.is_none()`; `appeal_window_expires_at.is_none()`; `sanction_count == 0`; `governance_log entry_kind == "jury_deadlock" exists`; `case_decided NOT emitted`; `public_case_log NOT created`; `0 reputation_event rows for source_case_id` |
| 3 | `submit_jury_vote_writes_appeal_window_default` | No-sponsor path with default `appeal.window_days = 7` | `v1_jm_b_fixtures` + `seed_case(_, _, SeverityTier::Minor)` | After case decided: `(appeal_window_expires_at - decided_at).num_days() == 7`; `closed_at.is_none()` (JM-c removed the close write) |
| 4 | `submit_jury_vote_writes_appeal_window_live_config` | Live-read assertion: bump `appeal.window_days` to 30 BEFORE the threshold-meeting vote, assert appeal_window reflects the bumped value | `v1_jm_b_fixtures` + `admin_set_config` | After admin bumps `appeal.window_days = 30` and threshold-meeting vote casts: `(appeal_window_expires_at - decided_at).num_days() == 30` |
| 5 | `v0_case_completes_under_v0_rules_after_v1_config_flip` | PRD §11 canonical regression — full snapshot semantic | `v1_jm_b_fixtures` + `admin_set_config` × 3 | Snapshot fields = (5, 3, 3) at admin_assign time; admin flips `jury.panel_size.regular.minor=11`, `jury.threshold_fraction.minor=0.95`, `appeal.window_days=60` mid-case; 3 votes still decide the case (v0 threshold_count_snapshot=3 honored, NOT new fraction); appeal_window uses LIVE 60 (deliberate exception) |
| 6 | `submit_jury_vote_concurrent_votes_decide_exactly_once` | Concurrency: tokio::join! 2 votes that BOTH would meet threshold; FOR UPDATE serialization + idempotency guard ensure exactly-once side-effects | `v1_jm_b_fixtures` + `tokio::join!` | After race: `vote_count == 4`; `sanction_count == 1`; `case_decided log count == 1`; both `submit_jury_vote` calls return Ok |

### 14.2 Edge cases covered

- [x] Snapshot-aware threshold honored (test 1)
- [x] Deadlock detection at full-panel-no-winner (test 2)
- [x] Deadlock does NOT emit case_decided / sanction / public_log / reputation_events (test 2)
- [x] Appeal window populated on default config (test 3)
- [x] Appeal window populated under config override — proves LIVE read (test 4)
- [x] v0 snapshot rules survive mid-flight config changes (test 5)
- [x] Live `appeal.window_days` deliberately overrides snapshot semantic (test 5)
- [x] Concurrent votes serialize cleanly (test 6)
- [x] `closed_at` no longer written by JM-c (test 3 explicit assertion)
- [x] `CaseStatus::EmergencyRemove` exhaustive match preserved (compile-time via existing v0 idempotency guard at line 265)

### 14.3 Edge cases NOT covered (out of JM-c scope, JM-d / JM-e territory)

- Appeal window expiry → Closed transition (JM-d background job)
- Reporter-rights extension on appeal (JM-d)
- Auto-rejury on appeal acceptance (JM-d)
- Sponsor-liability compute/fire branch from step 7 (SL-d)
- Vote-outcome / evidence-quality emitters at step 7 (rep-tuning-r3)

### 14.4 Pre-existing tests preserved

JM-c does NOT modify any existing test. The golden-path test `report_to_modlog_golden_path` (line 1061) continues to assert the v0 5-juror / 3-of-5 / closed_at pattern — but JM-c's snapshot reads + appeal_window write change the case's terminal state slightly. Per Task 7 validation: confirm `report_to_modlog_golden_path` still passes by reading its assertions and verifying the pre-existing test seeded a v0-style case (panel=5, snapshot=3) — which it does, since the JM-a backfill set `quorum_snapshot=3` for pre-v1 cases and `seed_case` (used by golden path) creates new cases that go through `admin_assign_jury` post-JM-b, getting the JM-b snapshot writes (panel=5, quorum=3 for default Minor severity).

**Risk to assess at Task 7:** the golden-path test asserts `closed_at.is_some()` somewhere in its assertion block (verify at impl time at line 1414+). JM-c removes the `closed_at` write at step 8, so that assertion may fail. **If so:** this is a pre-existing test that JM-c MUST update — change the assertion from `closed_at.is_some()` to `appeal_window_expires_at.is_some()`. Document this in the Task 7 retro. Do NOT silently delete the assertion; the test should still verify the case has a window populated, just under the new column name.

---

## 15. Validation commands (DoD)

Per `.claude/rules/cargo-output-capture.md` + R6 (uniform `--no-deps` per JM-b retro-events Event 3): all clippy invocations use `--workspace --features full --no-deps -- -D warnings`.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-JM-c-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-JM-c-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (R7 — per task that touches a struct OR a re-export)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-JM-c-<task>-test-no-run.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e test execution (Task 6 + Task 7)

```bash
# Run individual JM-c tests by name (Task 6)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server <test_name> > .claude/PRPs/debug/v1-JM-c-task6-<test>.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-JM-c-task6-<test>.log
# EXPECT: exit 0; "1 passed; 0 failed"

# Run full e2e suite (Task 7)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/PRPs/debug/v1-JM-c-task7-e2e.log 2>&1"
echo "exit: $?"
tail -60 .claude/PRPs/debug/v1-JM-c-task7-e2e.log
# EXPECT: exit 0; "N passed; 0 failed; 3 ignored"
```

### 15.5 Cross-cutting verification (Task 7)

- [ ] Every new governance_log emission calls `governance_log::append(...)` (not direct INSERT)
- [ ] Every governance_log payload string passes through `redaction::scrub` if it contains user-derived content (the JM-c new emissions — `jury_deadlock` payload — contain only system-derived values: case_id, snapshot ints, decision enum names; no scrub needed)
- [ ] `actor_pseudonym_helper::get_or_create` is called for the casting juror in the deadlock path (reuses existing `juror_pseudonym` variable per Task 3 GOTCHA)
- [ ] No raw `person_id` or username written to `governance_log` payloads (the JM-c additions write `case_id` and snapshot ints + decision-enum-keyed counts only — no PII)
- [ ] `CaseStatus::EmergencyRemove` exhaustive match preserved in the idempotency guard at submit_jury_vote.rs:265
- [ ] Hash-chain test (`governance_log_hash_chain_holds` if it exists in e2e.rs) still passes
- [ ] R1: every i32 ↔ i64 comparison uses `i64::from(...)`, never `as` cast
- [ ] R2: every JM-c test calls `seed_jury_eligible_snapshots` before `admin_assign_jury`
- [ ] R3: no struct extension in JM-c scope; grep sweep is empty
- [ ] R4: every JM-c test name uses lowercase snake_case
- [ ] R5: Task 0 enumerated all 5+ probes (Docker + 4 wrapper + clippy baseline + DoD smoke)
- [ ] R6: all clippy invocations use `--no-deps` uniformly
- [ ] R7: Tasks 1, 2, 3, 5, 6 ran `cargo test --no-run -p lemmy_server --test e2e` after the change

---

## 16. Acceptance criteria

- [ ] All 8 tasks (Task 0 pre-flight + 1-6 implementation + 7 validation + 8 retro) completed in dependency order
- [ ] §15.1 (cargo check) exit 0 after every task
- [ ] §15.2 (cargo clippy with `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after every task that touches a struct or re-export
- [ ] §15.4 (e2e tests) — 6 new JM-c tests pass; all pre-existing tests still pass; 3 ignored carriers preserved
- [ ] §15.5 (cross-cutting verification) — all 13 boxes ticked
- [ ] No contradictions with the 15 ADRs (ADR-010 + ADR-013 + ADR-015 explicitly verified)
- [ ] No new schema, no new migrations, no new seeded config keys (handler-only sub-phase)
- [ ] No edits to files outside §11 list
- [ ] Retro file written + validation logs captured
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

---

## 17. Completion checklist

- [ ] Task 0 audit complete (probes 0-9 confirmed)
- [ ] Task 1 (entry-kind const + re-export + registry row) committed
- [ ] Task 2 (snapshot-aware vote-count gate) committed
- [ ] Task 3 (per-decision threshold + deadlock branch) committed
- [ ] Task 4 (sponsor-liability TODO comment at step 7) committed
- [ ] Task 5 (appeal_window write + closed_at removal) committed
- [ ] Task 6 (6 e2e tests) committed
- [ ] Task 7 full-workspace validation green
- [ ] Task 8 retro committed
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] Post-merge JM-c branch retained for retro reads (per JM-b precedent)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Snapshot field is `Option<i32>` and a NULL slips through (e.g., a case manually inserted without going through admin_assign_jury) | LOW | HIGH | `ok_or(LemmyErrorType::NoEnoughJurorsAvailable)` per §10.2 fails loudly; tests assert non-NULL |
| Per-decision tally hardcodes 8 `JuryDecision` variants; future variant addition silently misses the iteration | LOW | MED | Inline comment per §10.3 + compile-time anchor at `map_decision_to_sanction` (exhaustive match — adds 9th variant fails to compile there first) |
| `as` cast slips through (R1) — `clippy::as_conversions` lint trips | LOW | LOW | §10.2 GOTCHA names `i64::from(...)` as the canonical pattern; per-task clippy validation catches |
| Test author forgets `seed_jury_eligible_snapshots` (R2) — test asserts on small-pool-fallback values | MED | MED | §10.7 + §14 require explicit fixture call before every `admin_assign_jury` |
| Concurrency test (#6) flakes due to `tokio::join!` non-determinism | MED | LOW | FOR UPDATE lock at submit_jury_vote.rs:243 serializes at DB layer regardless of task scheduling; assertion is on exactly-once side-effects (idempotent) — escalate to advisor if flaky |
| `closed_at` removal breaks pre-existing `report_to_modlog_golden_path` assertion | MED | MED | Task 7 validation explicitly checks; if asserted, update the test inline (change `closed_at.is_some()` to `appeal_window_expires_at.is_some()`); document in retro |
| Mid-phase context-window exhaustion (post-Phase-1 ~360k token zone) | LOW | MED | JM-c is ~8 tasks (vs JM-b's 10); explicit handover protocol if mid-phase context >180k; Task 0 enumerates probes to avoid late-phase audit cascades |
| `ENTRY_KIND_JURY_DEADLOCK` already exists somewhere (collision with JM-a's seeded-but-unused consts) | LOW | LOW | Task 0 Probe 9 explicitly checks; if collision found, STOP and reconcile |
| The v0 literal `"jury_vote_submitted"` at line 191 gets accidentally changed during Task 3 rewrite | MED | HIGH | §4.1 + §4.2 + §19 explicitly call this out; the rewrite is around line 216+ (not 191); inline review at impl |
| `cargo test --no-run` passes but a runtime assertion fails because `appeal.window_days` config seed missing | LOW | LOW | `appeal.window_days` was seeded in JM-a per JM-b retro §3.3 + JM PRD §10; verify at Task 0 by reading the migration |
| SL-d's diff misses the TODO anchor because the comment got reformatted | LOW | MED | §10.6 + Task 4 GOTCHA explicit "do not paraphrase or shorten"; CR will catch any change |

---

## 19. Notes

- **PR #95 carry-forward (issue #96) — explicit policy.** 7 findings carried over from PR #95 are scoped to issue #96, NOT to JM-c. The two clusters most likely to graze JM-c's edits are:
  - **cr-5 (`config.rs:439` candidate-level const fallback asymmetry)** — JM-c uses `config::get_int` for `appeal.window_days` per §10.5. The cr-5 finding is about the cascade fallback semantics for missing keys, which `get_int` (the non-cascade function) does NOT exhibit. **JM-c does NOT touch this code path; do NOT silently fold cr-5 into a JM-c commit.**
  - **cr-8 (`decline_jury_assignment.rs:164` ConstraintRecord persist)** — JM-c does NOT touch `decline_jury_assignment.rs`. **Do NOT silently fold cr-8 into a JM-c commit.**
  - If during impl the JM-c author finds that one of the carry-forward findings IS trivially adjacent (e.g., a one-line typo fix in a file JM-c is editing anyway), they MAY include it with a clear `chore(carry-forward):` commit subject and reference to issue #96. This is a deliberate scope decision per JM-b retro §3.1 row policy.
- **The v0 literal `"jury_vote_submitted"` at submit_jury_vote.rs:191 deliberately stays.** The canonical const is `ENTRY_KIND_JURY_VOTED = "jury_voted"` per `.claude/rules/governance-log-entry-kind-registry.md` line 67. The literal at line 191 has been the actual emitted entry kind since Phase 4b; changing it would silently break federation outbound, dashboards, and audit tooling that read `entry_kind = 'jury_vote_submitted'`. The cleanup is a separate `chore(governance-log): align jury vote entry kind to canonical const` PR with a backfill UPDATE that rewrites historical rows. **Out of JM-c scope. Document in retro for future advisor decision.**
- **DQ #47 (OQ-V1-JM-07) is planner-pending and does NOT block JM-c.** The post-JM-b general case-open severity-tier inference question is v1.5 territory. JM-c MUST NOT add any case-open severity_tier writers — non-emergency paths (`create_report`, `threshold_met`) inherit the JM-a DEFAULT 'Minor'. Per JM-b retro §3.4. Plan §12 makes this explicit.
- **`closed_at` becomes JM-d's column.** Post-JM-c, `closed_at` is NULL for all newly-decided cases. JM-d's appeal-window-expiry background job will set `closed_at = now()` when transitioning Decided → Closed. Pre-v1 cases retain the JM-a-backfilled `closed_at` (per `2026-04-23-000100_*/up.sql` migration). Do not assume `closed_at IS NOT NULL` anywhere in post-JM-c code.
- **`compute_status_tier` from `admin_assign_jury.rs:328-331` is NOT called by JM-c.** Status tier is read from the snapshot at admin-assign time (JM-b's contribution). JM-c does not need to recompute or re-read status tier; the snapshot is sufficient.
- **PRD §17 row 3 mentions `v0_case_completes_under_v0_rules_after_v1_config_flip` for JM-c AND PRD §11 mentions it as a separate test invariant.** Both reference the same test. JM-c is the OWNER per §17 row 3; JM-e capstone may extend it with cross-sub-phase assertions but the canonical implementation is in JM-c.
- **Federation publish call at submit_jury_vote.rs:483 is preserved verbatim.** The `federation_outbox::send_local_sanction_notice` invocation fires when the winning decision maps to `SanctionScope::FederatedRecommendation` per `map_decision_to_sanction`. JM-c does not touch this; its position relative to step 8 (sanction insert) and step 9 (appeal_window write) is unchanged.
- **Per-juror reputation_event writes at submit_jury_vote.rs:391-450 are preserved verbatim.** JM-c does not modify the reputation logic. Aligned/outlier deltas continue to be LIVE config reads (NOT snapshotted) per the JM-a comment at lines 89-91. This is consistent with the appeal_window LIVE-read decision.
- **Task 0 probes capture to `.claude/PRPs/debug/v1-JM-c-task*.log` per `cargo-output-capture.md`.** The `.claude/PRPs/debug/` directory is `.gitignore`d (per `.claude/.gitignore:2: *.log`). Logs stay local; no commit at Task 0 or Task 7. Validation results are summarized in the Task 8 retro.
- **JM-c does not introduce any new `ENTRY_KIND_*` consts beyond `ENTRY_KIND_JURY_DEADLOCK`.** Per JM-b retro §3.3: existing v0 kinds (`case_decided`, `sanction_created`, `public_log_published`, `jury_voted`) are reused; sponsor-liability-related kinds are SL-d territory.
- **Confidence score: 9/10.** Per JM-b retro §5: "If §3.2 amendments 1+2+3 land in the JM-c/d/e plans, those sub-phases should hit 9+/10 confidence. The §10 snippet discipline + §14 task wording + chore-commit grep-sweep are all proven now." JM-c plan §10 + §13 + §14 incorporate amendments R1, R2, R3, R4, R5, R6, R7 — exceeding the JM-b retro target. The 1-point discount is for the concurrency test pattern (no precedent in e2e.rs) and the v0-compat regression test (PRD §11 owner — first implementation), both of which carry execution risk that can't be fully de-risked by plan-template improvements.

---

## 20. Sub-phase stubs (v1-JM-d/e TOC only)

Per the v1-AD / v1-JM-a / v1-JM-b precedent: each sub-phase is its own plan file, owns its own PR → `governance-v0`, earns its own CodeRabbit review, and only gets written after the preceding sub-phase merges.

### v1-JM-d — Appeals (bounded window + reporter-rights + auto re-jury + background job)

- **Bounded-window appeal check** in `request_appeal.rs`: replace `if case.closed_at.is_some()` (line 105) with `if case.appeal_window_expires_at.is_none() || case.appeal_window_expires_at.unwrap() < now()`. JM-c is the FIRST writer of `appeal_window_expires_at`; JM-d is the FIRST reader.
- **Reporter-rights extension**: allow reporter to appeal iff `case_decision IN (NoAction, AdvisoryLabel)`. PRD §6.4 / §12.4. Per the recommendation in PRD §9.3, may add `moderation_case.winning_decision JuryDecision NULL` for queryability — that's a JM-d migration if needed.
- **Auto re-jury**: when `appeal.auto_select_on_appeal_acceptance = true`, in same transaction call `select_appeal_panel(case, original_jurors)` that re-runs `select_eligible_jurors` with `exclude_person_ids = original_jurors`, `role = JuryAssignmentRole::Appeal`, next-tier threshold per PRD §6.3 (`appeal.threshold_tier_bump`).
- **New handler `admin_trigger_appeal_rejury`** for the `false` config path. New file `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs`. Admin-only. Same `select_appeal_panel` helper as the auto path.
- **New background job in `crates/server/src/governance.rs`** — periodic `appeal_window_expiry` tick (mirrors existing `expired sanction cleanup` pattern). Finds cases where `status = Decided AND appeal_window_expires_at < now()`, flips to `Closed`, sets `closed_at = now()`, emits `appeal_window_expired` log entry.
- e2e tests: appeal happy path + reporter appeal on NoAction + appeal panel excludes original jurors + appeal-window expiry transition.
- ~7 tasks, blocks JM-e.

### v1-JM-e — Capstone test + cross-sub-phase integration assertions

- **Cross-sub-phase integration tests**: full case lifecycle from report → assign (JM-b) → vote (JM-c) → appeal (JM-d) → re-jury (JM-d) → decision → close. May extend `v0_case_completes_under_v0_rules_after_v1_config_flip` (JM-c-owned) with cross-sub-phase assertions.
- **Audit log invariant test**: every case's `governance_log` sequence matches the expected state-transition shape (severity_tier_frozen → panel_assembled → jury_voted×N → case_decided OR jury_deadlock → public_log_published → appeal_panel_assembled (if appealed) → appeal_decided OR appeal_rejected OR appeal_window_expired).
- **Mid-flight config-churn regression test** — extends JM-c's test 5 with appeal-window churn during the appeal phase.
- **Step-up auth stub for severity-tier changes mid-case** (PRD §12.3) — v1 ships `step_up_token: Option<String>` DTO slot (v1 behaviour = ignore; v2 activates per ADR-010).
- **Constraint-relaxation admin-visibility check** (PRD §12.2) — dashboard query for community-admin role.
- ~3-4 tasks, closes the JM PRD.

---

_Plan author: advisor session 2026-04-25 (cold-resumed from `v1-JM-b-advisor-handover.md` + JM-b retro/retro-events). Confidence 9/10. Plan committed on `governance-v0` primary worktree; BM cuts `phase-v1-JM-c` branch + worktree post-PR-merge. Impl session kicks off with `/prp-core:prp-implement .claude/PRPs/plans/v1-jury-mechanics-c.plan.md` from `brehon-fork-phase-v1-JM-c`._
