# v1-quality-r1 retro

> Phase tip at retro authorship: `cea679810` (chore(bm): merge PR #145 complete).
> Plan: `.claude/PRPs/plans/v1-quality-r1.plan.md` (committed `7bb37068c`; resumed from in-flight draft).
> PR: #145 merged `eec20a102` 2026-05-22T16:57:34Z.
> Authored: 2026-05-22 by advisor session.
> Phase wall-clock: audit worktree (`brehon-fork-audit-2026-05-22`) + impl worktree (`brehon-fork-quality-r1`). Audit completed off-session; impl session ~4 hours (resume from pre-pause draft → three phases + DQ entries + PR + prp-review + merge).
> §16a stories: not formally tracked (quality-r1 ran as advisor-driven implementation, not four-role Junior dispatch; plan predates §16a stories convention). All three phases shipped.
> Phase 2 e2e: **103 passed / 0 failed / 5 ignored** (`E2E_FULL_EXIT_0`, 2026-05-22, post-Phase-1.5, all 6 commits).

---

## Advisor signals

The advisor operated in a hybrid mode this phase: directly authored all implementation commits (three phases) rather than dispatching to Junior subagents. This was appropriate given the defect-fix nature of the work (ADR-010 snapshot correction, rustfmt stable/nightly alignment, LazyLock test-poison), the audit-phase pre-work already done in the `brehon-fork-audit-2026-05-22` worktree, and the phase's origins as a resumption of in-flight work from a prior session.

Gates honoured: plan approval (DoD smoke test via pre-push local validation), Phase 2 e2e local vs dispatch (local, user selected), merge confirm (gate 5).

One gate deviation: `/bm-prp-review` substituted for CodeRabbit (billing-blocked >72h). The substitute produced a complete Claude review covering all four ADRs, mechanical reformat verification, and LazyLock helper analysis. Recommendation was APPROVE; one low wont-fix finding (SAFETY comment wording).

---

## What surprised us

**Phase 1.5 — third drifting consumer not in the plan.** Audit 4.1 enumerated 5 `CONFIG_KEY_METADATA` consumers and classified them all as safe. The e2e test `case_open_pins_applied_config_snapshot_and_rule_set_version_id` was NOT enumerated because it doesn't import `CONFIG_KEY_METADATA` — it asserted against a hardcoded literal 7-key list. When Phase 1 expanded the helper to emit 27 keys, the e2e assertion broke. Fix: replace the hardcoded list with a metadata-filter computed at test time, same shape as the in-crate parity test. This was caught by running a local Phase 2 e2e before pushing — the failure was identified before PR open, not by CR or CI.

**Lesson:** future drift audits must enumerate not just `CONFIG_KEY_METADATA` *consumers* but also test-side *assertions* that pin the helper's output shape. Parity-pin audits (Audit 4.2) covered const-vs-source pins but not test-side literal lists shadowing the source. The audit scope was too narrow.

**Clippy exit 0 vs expected exit 101.** The plan warned to expect 8 clippy errors in `admin_config.rs` and `reputation_snapshot.rs`. The bm-prp-review agent's clippy run exited 0 with zero errors. The discrepancy: the plan's audit used `cargo clippy --all-targets` (lints test code); the agent's invocation used `--no-deps` without `--all-targets`, so `#[test]` functions were not linted. The 8 errors are still present and still tracked in DQ `922c8bae61db-001`. Not a regression, but the discrepancy is a calibration failure: the plan said "expect 8 errors" and we needed to explain why zero was not a regression.

**Phase 2 reformat: 255 files vs "50+" estimate.** The plan estimated ~50 files from `cargo fmt --check` reporting 1028 diff sites (rough 20:1 ratio). The actual ratio was ~4 diffs/file giving 255 files. The estimate was 5× off. Not a defect — the reformat was mechanical and pure whitespace — but the estimate methodology was wrong (diff-site-to-file ratio is highly variable; direct file count from `cargo fmt --check --verbose` would have been more accurate).

**CodeRabbit billing-blocked.** CR has not been able to collect payment for >72h. This blocked the standard review gate. The bm-poll-cr verb added a Phase 2.5 CR-ready gate (`49da0d109`) during this phase to handle billing-blocked fast-exit. The substitute review path (bm-prp-review) worked correctly. The billing issue is external; tracked as a project note.

---

## What to change

**Drift-audit scope must include test-side literal assertions.** The Audit 4.1 enumeration methodology was: find all `CONFIG_KEY_METADATA` imports and classify by access pattern. This missed test files that assert against the helper's output shape without importing the metadata. Future audits of this class should add a second grep pass: `rg "snapshot_keys\|applied_config_snapshot\|REQUIRES_RE_JURY"` in test files, classify each hit as "dynamic (metadata-filtered)" vs "static (hardcoded literal)".

**Clippy invocation consistency.** The plan-authoring convention should specify whether clippy runs with or without `--all-targets`. For quality-r1, the plan said "expect 8 errors" based on `--all-targets` but the BM verb uses `--no-deps` without `--all-targets`. Either: (a) the plan should note the `--all-targets` qualifier explicitly, or (b) the BM verb should add `--all-targets` to its standard clippy invocation. The DQ `922c8bae61db-001` carries the full recipe; this is a documentation gap not a defect.

**Phase 2 e2e pre-push is the right place to catch drift.** The Phase 1.5 catch happened because the local Phase 2 e2e ran before pushing. This is evidence for the "local e2e pre-push" pattern as a meaningful quality gate (not just a validation step). The plan already mandated it; the execution confirmed it works.

---

## What to carry forward

**ADR-010 read-side gap (v1-JM-b).** The write-side is now correct (27 keys pinned at case-open). The read-side in `admin_assign_jury.rs`, `submit_jury_vote.rs`, and `sponsor_liability.rs` still reads live config for the 20 v1-JM-a keys. Cases opened BEFORE this PR will continue to use live config for those 20 keys until v1-JM-b ships. Tracked in DQ `922c8bae61db-002`. This is the most material technical debt carried forward.

**Clippy 8 errors deferred.** `admin_config.rs` (3 errors) + `reputation_snapshot.rs` (5 errors) in `#[cfg(test)]` code — test fn return types + `unwrap`/`expect`/`changes[N]`. Fix recipe: migrate to `LemmyResult<()>` + `?` per `feedback_clippy_test_style.md`. Estimated 30 min impl. Tracked in DQ `922c8bae61db-001`.

**`init_test_context()` long-term fix.** Phase 3 added 23 per-test-fn call-sites for `ensure_default_settings()`. The longer-term fix (Path B: modify `LemmyContext::init_test_context()` to call it internally, removing per-test boilerplate) was deliberately deferred to avoid blast radius. Tracked in DQ `922c8bae61db-005`. The per-test-fn approach ships correctly; this is a hygiene carry-forward.

**Audit methodology lesson.** Enumeration audits must cover not just code consumers of a source-of-truth but also test-side assertions that pin derived values. This is a cross-phase process lesson applicable to any future parity-pin audit.

**bm-poll-cr Phase 2.5 CR-ready gate.** The `49da0d109` fix added fast-exit logic when CR has not yet posted (billing-blocked, review in progress, etc.). This landed on `governance-v0` via the phase-v1-quality-r1 merge. It should be treated as a durable improvement to the BM verb, not a one-off patch.

---

## Per-role signals

### Advisor
Hybrid mode (direct impl + orchestration). All three defect fixes were planned and executed correctly. The Phase 1.5 catch (e2e drift) was the most valuable judgment call — the local e2e run surfaced a consumer class the audit missed, and the fix was authored and validated before PR open. Gate handling was correct on all six gates (one deviation: CR billing-blocked → bm-prp-review substitute, handled cleanly).

### Planning
No formal planning Junior dispatched (plan was authored advisor-side from prior audit work). The plan served its purpose: scoped the three phases, enumerated the 27-key dispatch table, specified the `ensure_default_settings()` shape, and pre-identified the three latent-defect audits. The "count is 27, not 26" correction was handled correctly (plan text said "reconcile against source-of-truth at impl time"; impl did that).

### Impl
Not dispatched as a Junior subagent (advisor authored directly). All commits are clean, task-labelled, and logically ordered. The fmt reformat commit (`2f13ffb80`) is the largest (255 files, ~6500 diff lines) but is purely mechanical and correctly isolated. The Phase 1.5 fix (`000c3c063`) was added in-flight after a local e2e catch — correctly sequenced after the Phase 1 commit and before the first push.

### BM
bm-cut, bm-push, bm-pr, bm-poll-cr (×2), bm-prp-review, bm-merge all executed. CR billing-blocked triggered bm-poll-cr fast-exit (new Phase 2.5 gate; first use). bm-prp-review ran long (~35 min for test-compile) but completed cleanly; one stall apparent in session context was the cargo test-compile job running in background — the agent completed correctly. Findings YAML was left unstaged by the agent (gitignored artifact); advisor force-added and committed manually. This is an existing BM process gap: gitignored artifacts need explicit force-add at end of prp-review session.

---

## Complexity / effort

| Phase | Files changed | Commits | Runtime approx |
|---|---|---|---|
| Phase 1 (ADR-010 snapshot fix) | 1 + 1 (e2e follow-up) | 2 | ~30 min |
| Phase 2 (rustfmt) | 256 (1 config + 255 reformatted) | 2 | ~10 min |
| Phase 3 (LazyLock fix) | 14 (1 helper + 13 call-sites) | 1 | ~20 min |
| Phase 4 (audits) | 0 (reports in audit worktree) | 0 | prior session |
| Phase 5 (DQ entries + PR) | 1 DQ + 1 PR | 1 | ~15 min |
| BM review cycle | reviews/ + runlog | 4 | ~60 min |

Total implementation: ~75 min active work. Full session wall-clock (including e2e 41 min + test-compile 23 min): ~4 hours.
