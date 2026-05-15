---
phase: chore/refactor-e2e-error-types
role: bm-task
task: bm-pr
brief_n: 2
authored: 2026-05-15
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.E.1 (rank 1 — CRITICAL) + 3.E.3 (rank 7 — MAJOR) + 3.E.4 (rank 8 — MAJOR); audit 3.E.2 (rank 2 — CRITICAL, Pass 2 fixtures dedup) DEFERRED to a separate post-gate lane (advisor task #8)
note: "Run INLINE by advisor (L15 precedent + L3 — Junior bm-task base_branch=chore/* fails on daemon worktree-ref resolution, proven PR-4 #264 + PR-5 #265 + PR-6 #130 + PR-2 #131). This brief documents intent regardless of executor. L5: this brief lives ONLY on governance-v0 — it is NOT cherry-picked onto chore/refactor-e2e-error-types (cherry-picking a governance-v0 brief onto a chore branch caused the PR #129 CONFLICTING incident when an advisor edit made the duplicate diverge). PR-1 is the LAST (5th) of 5 refactor-tier PRs and the HIGHEST-RISK lane (e2e.rs 8945-line corpus reshape)."
---

# [role:bm-task] bm-pr chore/refactor-e2e-error-types — see .claude/PRPs/briefs/refactor-e2e-error-types-bm-pr.md

## §1 Role + dispatch

`[role:bm-task] bm-pr chore/refactor-e2e-error-types — open PR into governance-v0`

## §2 Scope

Open a PR from `chore/refactor-e2e-error-types` into `governance-v0`
per `.claude/commands/bm/bm-pr.md` (Phase 1 → 7). Auto, no prompt
(autonomy table: open PR into governance-v0 is Auto).

**Branch state (prepared by advisor before bm-pr — do NOT re-cut):**

The advisor applied the L1 daemon-base root fix first (the daemon
`/srv/brehon-fork` `governance-v0` was STALE at `08a9069ff` after the
PR-2 #131 squash advanced trunk to `1fed06e16`; reset to
`origin/governance-v0` `1fed06e16`), cut `chore/refactor-e2e-error-types`
off `governance-v0` @ `1fed06e16` (post-PR-2 #131 merge + strict-gate
4/5), dispatched impl-task Junior #268 with `base_branch=governance-v0`
(NOT `base_branch=chore/*` — daemon cannot resolve chore refs, per
PR-4 #264 + PR-5 #265 + PR-6 #130 + PR-2 #131), then cherry-picked the
worker's error-type-unification commit (`f258824b5` on chore: Case C →
Case A `LemmyResult<()>` reshape of the e2e.rs Brehon corpus +
`phase1_migrations_round_trip` split into 3 independent fns +
`PHASE_1_MIGRATION_COUNT` replaced by named `MIGRATIONS_TO_REVERT_PHASE_1`
list; audit 3.E.2 / Pass 2 fixtures dedup explicitly DEFERRED). #268
raised TWO blockers (DQ #221 — pre-existing e2e.rs Gate-2 clippy debt
never green before this refactor; DQ #222 — Pass 2 not done) which the
advisor user-relayed via two AskUserQuestion rounds; authoritative
answers committed on the chore branch at `091b83a86` (#221 →
fix-all-e2e-lints; #222 → defer Pass 2 to advisor task #8). A
continuation impl-task Junior #269 produced the Gate-2 lint cleanup
(`a82769876` on its worker branch); the advisor cherry-picked ONLY the
`crates/server/tests/e2e.rs` delta (dropping #269's decision-queue.json
churn so advisor retains DQ attribution) onto the chore branch as
`919fe8400`. The advisor then ran the full §5.2 advisor-laptop
validation and backfilled the validate-pending DQ (#223, advisor-laptop,
result:pass) at `3084eaab4`. By bm-pr time the branch carries: the
`f258824b5` error-type commit + the `091b83a86` authoritative DQ
#221/#222 answers + the `c9a1a316b` #268 runlog block + the `919fe8400`
Gate-2 lint-cleanup commit + the `3084eaab4` validate-pending DQ
backfill + the lane runlog. **The bm-pr brief is NOT on this branch
(L5) — it lives only on governance-v0.**

- Chore branch — no `.plan.md`. bm-pr retro gate + Phase 1c e2e
  gate correctly skip for chore branches (plan-aware script).
- Phase 1b historical-fail sweep: the LOAD-BEARING signal here is the
  full local e2e suite (a test-corpus reshape; `cargo test --no-run`
  is insufficient per brief §2.7). The advisor ran the full suite
  locally — 88 passed / 0 failed / 5 pre-existing GH-tracked ignored
  deflakes — so the sweep is satisfied by DQ #223.

**Title** (chore/test → `chore(test): <prose>`): `chore(test): unify e2e error-types to LemmyResult + split round-trip + lint cleanup (audit 3.E.1/3.E.3/3.E.4)`

**PR body** — per bm-pr Phase 3. No completion report, no plan. Body
MUST include:

- `## Summary` — Reshapes the Brehon-authored test corpus in
  `crates/server/tests/e2e.rs` (8945 lines) to a uniform
  `LemmyResult<()>` error-handling shape (Case C → Case A per
  `feedback_lemmy_error_no_std_error.md` — eliminates the mixed
  `Box<dyn Error>` / `LemmyResult` divergence that made the corpus
  inconsistent and `?`-propagation fragile; audit §3.E.1, rank 1,
  CRITICAL). Splits the 447-line `phase1_migrations_round_trip` into
  3 independent test fns (`test_phase1_migrations_forward` /
  `_reapply` / `_revert`) so a single migration regression no longer
  masks the other two and each is independently runnable (audit
  §3.E.3, rank 7, MAJOR). Replaces the opaque `PHASE_1_MIGRATION_COUNT`
  integer with a named `MIGRATIONS_TO_REVERT_PHASE_1` list so the
  revert set is self-documenting and drift-resistant (audit §3.E.4,
  rank 8, MAJOR). Closes the pre-existing Gate-2 clippy debt on
  e2e.rs: ~13 lint classes fixed PER-SITE (map_err_ignore 6 sites
  `|_|`→`|_e|`; as_conversions 2 sites `as i64`→`i64::try_from().expect()`;
  needless_ok_wrapping ×2; ref_patterns ×1; iter_copied_collect ×1;
  redundant_type_annotation + unused `VerifyingKey` import removed;
  redundant_closure ×1; type_complexity ×2 stmt-level;
  too_many_arguments ×2 fn-level; unused_async ×1;
  doc_list_item_without_indentation ×1; incidental `create_report.rs`
  `#[allow]`→`#[expect]` accepted per user) **PLUS 7 SELF-CLEANING
  file-level `#![expect(...)]` with mandatory `reason=` strings**
  (`tests_outside_test_module`, `items_after_statements`,
  `expect_used`, `unwrap_used`, `indexing_slicing`, `unreachable`,
  `get_first`).

  **Deliberate, USER-RATIFIED deviation from the literal DQ #221
  wording ("fix each properly, no blanket masking"):** the 7
  file-level attributes use `#![expect]`, **not** `#![allow]`.
  `#![expect]` is categorically different from the `#![allow]`-spam
  anti-pattern the brief forbids: the build **FAILS** via
  `unfulfilled_lint_expectations` if any expected lint stops firing,
  forcing the attribute's removal — it is self-cleaning, not debt-
  hiding. Each carries a `reason=` string. The set is justified
  engineering judgment, ratified by the user across two
  AskUserQuestion rounds (2026-05-15):
  - `tests_outside_test_module` — false-positive on Cargo
    integration-test files: a top-level `async fn test_*` IS the
    idiomatic structure for `tests/e2e.rs` (there is no enclosing
    `#[cfg(test)] mod` in an integration test target).
  - `get_first` — `Vec::first()` collides with Diesel
    `RunQueryDsl::first()`; rewriting to `[0]` would reintroduce
    `indexing_slicing`. Genuine trait-ambiguity, not laziness.
  - `expect_used` / `unwrap_used` / `indexing_slicing` /
    `unreachable` — idiomatic in test assertions where a panic IS
    the failure signal; per-site `.get()? + expect-with-message`
    rewrites of ~hundreds of assertion sites would be pure churn
    with zero behavior change and worse test-failure diagnostics.
  - `items_after_statements` — fixture helper-fn definitions
    interleaved with setup statements; relocating them would
    fragment the fixtures modules that Pass 2 (DEFERRED) will
    restructure anyway.

  ZERO behavior change in any test (no removed assertions, no changed
  seed data, no `#[ignore]` added). Sibling phase fixtures modules
  (`v1_sl_b/c/d/e_fixtures`) all pass unchanged.
- `## Plan reference` — `Ad-hoc — no plan file (chore branch;
  audit-driven refactor-tier PR-1 of 5 — the LAST, serial dedicated
  lane Step-2a — per
  .claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md;
  PR-3 was dropped via DQ #214 so the gate is 5 PRs not 6;
  user-chosen sequencing: PR-2 first, then PR-1 last; PR-1 dispatch
  was explicitly user-gated as the highest-risk lane)`
- `## Closes` — `Closes audit findings 3.E.1
  (v1-code-quality-audit-2026-05-14.md, rank 1, severity CRITICAL),
  3.E.3 (rank 7, MAJOR), 3.E.4 (rank 8, MAJOR).` **EXPLICITLY note:**
  `audit 3.E.2 (rank 2, CRITICAL — Pass 2: consolidate the ~70%
  duplication across v1_jm_b/e + v1_sl_b/c fixtures modules into a
  shared governance_test_helpers module) is DEFERRED to its own
  dedicated post-gate lane (tracked as advisor task #8) per the
  user's 2026-05-15 decision (DQ #222). The 4 fixtures modules are
  UNCHANGED on this branch. 3.E.2 is a tracked carry-forward and
  does NOT block v1 PRD planning (strict-gate stays 5 PRs, not 6).`
- `## Validation` — Workspace check via Shape-G: note that this cycle
  the Shape-G `cargo-validate-workspace` workflow DID trigger on the
  `junior/*` push (run `25935134971`) — contrast the persistent
  stuck-runner pattern seen on PR-4/5/6/2. Regardless, for a
  test-corpus reshape the LOAD-BEARING signal is the FULL local e2e
  suite (per brief §2.7 + advisor-orchestrator.md §5.2 — `cargo test
  --no-run` is insufficient). State the advisor-laptop §5.2 results
  with EXPLICIT exit codes: `cargo-check.bat --workspace --features
  full` → CHECK_EXIT_0 (12m39s); `cargo-clippy.bat --workspace
  --features full --test e2e --no-deps -- -D warnings` → CLIPPY_EXIT_0
  (7m56s, 0 warnings / 0 errors — clippy scoped to `--test e2e` per
  the DQ #221 user-authorised fallback for the out-of-scope
  reputation_snapshot.rs / admin_config.rs cross-crate lints; the
  clean run confirms the 7 `#![expect]` set is complete AND self-
  cleaning-valid — neither incomplete nor over-broad);
  `cargo-test.bat --workspace --features full --test e2e` →
  E2E_EXIT_0 (finished in 1864.47s ≈ 31m4s: **88 passed; 0 failed;
  5 ignored** — the 5 ignored are pre-existing GH-tracked deflakes
  GH #42/#43×3/#45, NOT regressions; #43×3 = the now-correctly-split
  `test_phase1_migrations_{forward,reapply,revert}`). validate-pending
  DQ #223 (advisor-laptop, result:pass). Test count 69 → 71 (the
  round-trip split adds +2 fns). ZERO behavior change. (Submodule
  `crates/email/translations` was init'd pre-emptively per L6 so no
  `lemmy_email build.rs read_dir` failure recurs.)

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` (operational script — Phase 1→7)
- `.claude/rules/branch-manager.md` (file-ownership; autonomy table)
- `.claude/rules/phase-branch.md` (PR into governance-v0, not main; not draft)
- `.claude/rules/gh-pr-fork-target.md` (`--repo barrie-cork/lemmy` mandatory)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` (PR-1 of 5 — the LAST, serial Step-2a; strict-gate)

## §4 Constraints

- **NEVER** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**`.
- **NEVER** open PR into `main` — base `governance-v0`.
- **NEVER** open as draft (CR skips drafts).
- **NEVER** re-cut or force-push the branch.
- **NEVER** cherry-pick this brief onto `chore/refactor-e2e-error-types`
  (L5 — PR #129 CONFLICTING root cause). The brief is referenced
  by path; it stays on governance-v0 only.
- `--repo barrie-cork/lemmy` on EVERY `gh pr` subcommand.
- Do NOT post a PR comment / submit a review (Manual/ask per
  autonomy table — bm-pr only opens the PR; the triage digest
  comment is a separate user-gated step).
- Append to runlog `.claude/runlog/chore-refactor-e2e-error-types.md`
  (already exists — lane was prepared).
- Return PR number + URL in the final summary.

## §5 Concurrency note

Sequential lane — PR-4 (valid-from) merged (#128, strict-gate 1/5),
PR-5 (diesel-errors) merged (#129, 2/5), PR-6 (seed-tests) merged
(#130, 3/5), PR-2 (TOCTOU create_report) merged (#131, 4/5). PR-1
(e2e error-types reshape, ranks 1/2/7/8, HIGHEST RISK) is the LAST
(Step-2a, user-gated explicitly before dispatch). On PR-1 merge →
**strict-gate 5/5** → the autonomous loop STOPS → author the 4-role
refactor-tier FINAL retro → surface its path → WAIT for user
sign-off. Do NOT auto-start the deferred 3.E.2 / Pass 2 lane (advisor
task #8 — user-instructed-only). Do NOT proceed to v1 PRD planning /
`/brehon-phase-transition` until strict-gate 5/5 + the FINAL retro +
user sign-off. Zero cross-lane file overlap (PR-1 touches only
`crates/server/tests/e2e.rs` + the incidental `create_report.rs`
`#[allow]`→`#[expect]` already merged-context — handover doc line 89).
