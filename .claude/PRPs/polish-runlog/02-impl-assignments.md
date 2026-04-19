# polish-1/polish-3 — impl assignments

**Created:** 2026-04-19T17:40Z
**Two-impl split.** Phase 6 lessons: file-level claims via runlog, explicit push gates, single source-of-truth for sequencing.

---

## Impl1 (me) — polish-1 critical bugs

**Branch:** `polish/critical-bugs` (cut from `governance-v0` @ `08065e1a1`)

**Scope (4 fixes + 1 test commit):**
1. #48(a) + #35 — `crates/db_schema/src/source/governance/governance_log.rs` — wrap append INSERT+UPDATE in `run_transaction`
2. #48(b) — `crates/api/api/src/governance/submit_jury_vote.rs` — move `case_decided` log above federation send
3. #34 — `crates/api/api_crud/src/governance/request_appeal.rs` — guard compares `closed_at` to `now()` instead of `is_some()`
4. #33 — `crates/api/api/src/governance/decline_jury_assignment.rs` — explicit self-exclude before `select_eligible_jurors`
5. `crates/server/tests/e2e.rs` — regression tests for #34 + #33

**Files owned by Impl1** (DO NOT TOUCH from other impl):
- `crates/db_schema/src/source/governance/governance_log.rs`
- `crates/api/api/src/governance/submit_jury_vote.rs`
- `crates/api/api_crud/src/governance/request_appeal.rs`
- `crates/api/api/src/governance/decline_jury_assignment.rs`
- `crates/server/tests/e2e.rs`

**DQs expected:** if any acceptance test fails, or if `run_transaction` wrapping breaks a caller, file DQ and stop.

**Push gate:** Impl1 pushes `polish/critical-bugs` branch independently. No coordination with Impl2 needed — separate branch, separate PR.

---

## Impl2 — polish-3 docs sweep

**Branch:** `polish/docs-sweep` (cut from `governance-v0` @ `08065e1a1`)

**Scope (1 bundled commit):**
- #38 — `.claude/PRPs/plans/phase-6-federation.plan.md` line 506 + Phase 5c report line 27: relative-path audit (fork-local design-doc citations)
- #39 — `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` — add 4-6 line paragraph on serial-id gap semantics under rollback (near "Catch-up on subscriber start" section)
- #50 — `.claude/PRPs/plans/phase-6-federation.plan.md` MD040 fence language tags (lines 93, 97, 127, 199, 202, 222, 262, 281, 309, 328, 346, 363, 393, 620, 741, 848, 877, 910, 914, 928) + enum typename fixes around line 493 (`*_enum` → plain names)
- #51 — `.claude/decision-queue.json` entry id=38 — update "question" and "answer" text to reflect in-flight-conn resolution instead of fresh-pool-conn language
- #52 — `.claude/rules/task-hopper.md:152` — reconcile agent-edit contradiction ("Agents never hand-edit" vs "Advisor may hand-edit for tuning/recovery")

**Files owned by Impl2** (DO NOT TOUCH from other impl):
- `.claude/PRPs/plans/phase-6-federation.plan.md` (covers #38 + #50)
- `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` (#39)
- `.claude/decision-queue.json` (#51) — UTF-8 encoding on Python open() per `feedback_python_utf8_encoding_windows`
- `.claude/rules/task-hopper.md` (#52)
- `.claude/PRPs/reports/phase-5c-*.md` if #38's broken link lives there

**Commit shape:** one commit `chore(docs): v0-polish docs sweep — GH #38 #39 #50 #51 #52`. ~40 lines across 5 files.

**Validation:** no cargo validation needed (docs-only). `bash -n` if any shell in scope (none here).

**Push gate:** Impl2 pushes `polish/docs-sweep` branch independently.

---

## Coordination

- **No file overlap** between Impl1 and Impl2 — parallel branches, parallel PRs.
- **Runlog is single source of truth** — update `.claude/PRPs/polish-runlog/99-<timestamp>-<impl>-<action>.md` after each meaningful milestone (or append to `03-progress.md` — decide at first commit).
- **If either impl needs to touch a file outside their list**, file a claim note in the runlog BEFORE staging.
- **PR creation:** each impl opens its own PR targeting `governance-v0`. Format follows `phase-branch.md` rule. No draft (CodeRabbit skips drafts).
- **Both PRs merge independently** once reviewed; no merge order constraint between polish-1 and polish-3.

---

## Out-of-scope reminders

- polish-2 Bucket C residual (#2p-7 TOCTOU + #2p-2/#2p-3 defensive) — later PR, not now
- polish-N individual (#36, #47, #37) — later PRs
- v1-tagged issues (#22–#31, #40, #41, #53) — do not pull into v0 polish
