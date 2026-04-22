# v1-AD-c advisor cold-resume brief (CR round 2 — stage: fixes staged, not committed)

**Written**: 2026-04-21T22:30Z — session-close for night, resume morning 2026-04-22
**Purpose**: Self-contained brief for morning resume. PR #81 is at HEAD `6c8fc3211`, CR round-2 findings received, advisor triage finalised, Impl has implemented 3 fixes into the working tree but has NOT staged/committed. Morning session resumes at **validate + stage + commit + push + post rebuttals**.

## TL;DR

- **PR #81 committed HEAD = `6c8fc3211`** (merge-gate CI: GREEN — governance-e2e SUCCESS, red-flag-diff SUCCESS, AI review SUCCESS)
- **CR round 2 triaged**: 3 fixes (B, C, D) + 2 rebuttals (A, E)
- **Fix C**: already applied in round-2 commit (no work needed)
- **Fix B** (snapshot race in `admin_list_rule_sets`): implemented in working tree, `cargo check -p lemmy_api --features full` = 0, 1m42s
- **Fix D** (extract `map_rsv_unique_violation` helper): implemented in working tree, handler refactor `cargo check` = 0, 6.92s. e2e-compile was running at session-close — morning must verify tail.
- **Working tree**: 3 Rust files modified, NOT staged, NOT committed
- **Two advisor-instruction-mismatches** caught correctly by Impl this round (E: CR's suggested diff was based on faulty model of snapshot shape; D: advisor's "drive through real handler" directive was impossible because `AdminCreateRuleSet` has no `version` field). Both corrected. Retro13 + retro14 logged.

## Cold-resume sequence

1. Read CLAUDE.md + .claude/rules/*.md (auto-loads in -p mode)
2. Read this file in full
3. Read `.claude/PRPs/v1-AD-c-runlog/00-advisor-notes.md` — risk register + retros (append retro13 + retro14 on first resume write)
4. Read `.claude/PRPs/v1-AD-c-runlog/01-v1-AD-c-progress.md` — last ~40 events for final state (read tail from `21:56:00Z | i | tX | triage` onwards)
5. Verify state:
   ```bash
   git rev-parse HEAD            # expect 6c8fc3211cd0b9696adabffff044013526df883d
   git status --short            # expect 3 Rust files modified (admin_rule_sets.rs, config.rs, e2e.rs), NOT staged
   gh pr view 81 --repo barrie-cork/lemmy --json state,mergeable,headRefOid,statusCheckRollup --jq '{state, mergeable, headRefOid, checks: [.statusCheckRollup[] | {name, status, conclusion}]}'
   ```
6. Append cold-resume event to runlog:
   ```
   <ISO-UTC> | a | tX | meta | advisor-cold-resume-cr-round2-stage=pre-commit-verify pr=81 head=6c8fc3211 working-tree=3files-modified
   ```

## State at handover

- **Branch**: `phase-v1-AD-c` at committed HEAD = `6c8fc3211`, tracking `origin/phase-v1-AD-c`
- **PR**: #81 OPEN, target `governance-v0 ← phase-v1-AD-c`, MERGEABLE, mergeStateStatus=CLEAN
- **URL**: https://github.com/barrie-cork/lemmy/pull/81
- **Working tree**:
  - `M crates/api/api/src/governance/admin_rule_sets.rs` — Fix B (snapshot race, `conn.run_transaction` wrap) + Fix D (extract `pub fn map_rsv_unique_violation` + refactor `process_create_rule_set`'s `UniqueViolation` arm to `.map_err(map_rsv_unique_violation)?`)
  - `M crates/api/api/src/governance/config.rs` — Fix C doc rewrite (already applied in round-2 commit — this diff may be empty or trivial; verify at resume)
  - `M crates/server/tests/e2e.rs` — Fix D test rewrite (`Step 3/4` route DieselError through `lemmy_api::governance::admin_rule_sets::map_rsv_unique_violation`)

## CR round-2 triage (finalised)

Triage matrix reflects the CORRECTED final positions after two advisor-instruction-mismatches caught by Impl:

| CR# | Action | File:Line | Status at session-close |
|---|---|---|---|
| **A** (DTOs expand v0 surface) | **Rebuttal** | api_common/governance.rs:585 | Reply drafted, not yet posted |
| **B** (versions + active_version_id snapshot race) | **Fix** | admin_rule_sets.rs:283-311 | Implemented in working tree, compiled clean (1m42s) |
| **C** (ScopeParseError doc drift) | **Fix** | config.rs:161-163 | Already applied in round-2 commit 6c8fc3211 — no new work |
| **D** (test recreates handler mapping) | **Fix via helper extract (Option 4)** | admin_rule_sets.rs + e2e.rs:5387-5419 | Implemented in working tree, handler refactor compiled clean (6.92s). e2e-compile was running in background at session-close — check tail at resume. |
| **E** (snapshot pin assert missing key) | **Rebuttal** | e2e.rs:5805 | Reply drafted, not yet posted |

## Retro13 + retro14 (append to 00-advisor-notes.md at morning resume)

- **retro13**: advisor prior directive "accept CR's E diff as-is" was a rubber-stamp without reading the snapshot shape. Applying the CR diff verbatim would have made the test FAIL (snap_obj doesn't have `rule_set_version_id` key; it's the 7 `requires_re_jury` keys per `case_open_snapshot.rs:50-58`). `rule_set_version_id` is a sibling column at `create_report.rs:218`, already strictly asserted at `e2e.rs:5779-5783`. Impl correctly flagged the advisor-instruction-mismatch. Pattern fix: advisor CR-suggested-diff acceptances must be marked "code-verified" vs "merits-only-from-comment".
- **retro14**: advisor prior directive "Fix D: drive duplicate-version test through the real handler (option D.i), not extract helper (option D.ii)" was based on not verifying `AdminCreateRuleSet`'s input surface. The DTO has 4 fields (community_id, rule_text, parent_id, reason) — no `version` field — so client cannot force a version-collision through the handler. Handler auto-increments via `lookup_latest_version + 1` at `admin_rule_sets.rs:121-122`. The only path satisfying both CR-5-round-1 (no `tokio::join!` non-determinism) and CR-5-round-2 (regression-safe mapping) is option 4 (extract `pub fn map_rsv_unique_violation`, call from production + test). Impl correctly flagged, advisor reversed, implementation proceeded. Pattern fix: advisor test-shape directives for handler-exercising tests must verify the handler's input DTO surface before prescribing the call shape.

Both retros demonstrate the advisor-instruction-mismatch memory rule working correctly. Record-worthy process wins.

## Morning resume action sequence

1. **Verify state** per cold-resume sequence step 5 above.

2. **Check background e2e-compile result**:
   ```bash
   tail -30 .claude/build-fixD-test-compile.log
   # Expected: e2e test target compiles exit 0. If compile fails, RCA the error.
   ```

3. **Local test gates** (all under real Postgres — docker required):
   ```bash
   # Fix D regression-safety test
   cmd //c "scripts\brehon\cargo-test.bat --test e2e -p lemmy_server admin_create_rule_set_duplicate_version_rejected > .claude/resume-fixD-test.log 2>&1"
   # Fix E rebuttal — existing test must still pass unchanged
   cmd //c "scripts\brehon\cargo-test.bat --test e2e -p lemmy_server case_open_pins_applied_config_snapshot_and_rule_set_version_id > .claude/resume-fixE-passthrough.log 2>&1"
   # Workspace check + clippy sanity
   cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/resume-workspace-check.log 2>&1"
   cmd //c "scripts\brehon\cargo-clippy.bat -p lemmy_api --features full --no-deps -- -D warnings > .claude/resume-clippy.log 2>&1"
   ```
   All must exit 0. If any fails, RCA before commit.

4. **Stage + commit** (single commit covering B + D; C is a no-op since round-2 already applied it):
   ```bash
   git add crates/api/api/src/governance/admin_rule_sets.rs crates/server/tests/e2e.rs
   # Do NOT add config.rs unless diff is non-empty — if it is, investigate why
   git diff --cached --stat
   # Expected: 2 files, admin_rule_sets.rs + e2e.rs; diff ~50-80+ lines total
   ```
   Commit message template at end of this brief.

5. **Post the two rebuttals to PR thread** (as inline comments on the specific file:line):
   - A on `crates/api/api_common/src/governance.rs:585` — text below
   - E on `crates/server/tests/e2e.rs:5805` — text below
   Use `gh api repos/barrie-cork/lemmy/pulls/81/comments -X POST -f ...` or paste manually via GitHub UI.

6. **Push** (non-force) to `phase-v1-AD-c`. CI re-runs; wait for `governance e2e` SUCCESS.

7. **Monitor CI** — merge-gate is `cargo-test-e2e` job. If SUCCESS, PR is ready for user merge authorisation. Do not self-merge.

## Commit message template (for step 4)

```
fix(v1-AD-c): address 3 CR findings on PR #81 (round 3)

- B (admin_rule_sets.rs:283-311): wrap versions + active_version_id
  reads in conn.run_transaction so the list response reflects a single
  DB snapshot. Prevents a concurrent create from splitting the two
  reads across different states — the returned active_version_id now
  consistently references a version present in the returned `versions`
  list.
- C (config.rs:161-163): already applied in round-2 commit 5c04e6ec2
  (ScopeParseError doc comment aligned with enum shape). No further
  work needed this round; noted for completeness.
- D (admin_rule_sets.rs extract map_rsv_unique_violation + e2e.rs:5387-5419
  call it): the duplicate-version test now routes the DB-layer
  UniqueViolation through production's mapping helper instead of
  recreating the mapping inline. Handler's auto-increment version
  (admin_rule_sets.rs:121-122) means a client-driven collision isn't
  reachable through the public API; the helper-extraction pattern gives
  regression safety without reintroducing tokio::join! non-determinism
  (CR-5 round 1) or adding test-only production hooks.

Rebuttals posted on PR thread:
- A (DTOs expand v0 surface): v1-AD-c extends the governance API per
  the v1-AD-c sub-PRD; the v0 11-endpoint guideline is scoped to
  governance-v0 baseline, not v1 branches.
- E (snapshot pin coverage): the pin is on moderation_case.rule_set_version_id
  column, not in applied_config_snapshot JSON. Column-level pin is
  strictly asserted at e2e.rs:5779-5783; adding it to the JSON would
  break the REQUIRES_RE_JURY_KEYS metadata-parity invariant.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

## Rebuttal reply texts (for step 5)

### A — post as inline comment on `crates/api/api_common/src/governance.rs:585`

> The v0 11-endpoint scope limit (`05-mvp-and-delivery-plan.md §2`) governs `governance-v0`. This PR lands on `phase-v1-AD-c` targeting `governance-v0` as the base, but explicitly extends the governance API surface per the v1-AD-c sub-PRD (admin config + rule-set versioning). `AdminCreateRuleSet*`, `AdminListRuleSets*`, and `RuleSetVersionView` are the wire contracts for `POST /admin/rule-sets` and `GET /admin/rule-sets` — v1 endpoints authorised by the sub-PRD, not v0 additions. The coding-guideline sentinel for v0-scope is stale for v1 work on this branch.

### E — post as inline comment on `crates/server/tests/e2e.rs:5805`

> `applied_config_snapshot` is the 7-key JSON payload defined by `CONFIG_KEY_METADATA` entries where `requires_re_jury == true` (see `crates/api/api/src/governance/case_open_snapshot.rs:50-58` and the metadata-parity test in that file's `cfg(test)` module). `rule_set_version_id` is a sibling column on `moderation_case` (pinned at `crates/api/api_crud/src/governance/create_report.rs:218`), not a `requires_re_jury` config key. The existing test asserts the column pin strictly at `e2e.rs:5779-5783`:
>
> ```rust
> assert_eq!(
>   case.rule_set_version_id.map(|r| r.0),
>   Some(v1.rule_set_version_id),
>   "rule_set_version_id pinned to the active community version",
> );
> ```
>
> If the handler stopped writing the pin to the column, `case.rule_set_version_id` would be `None` and this assert fails. The coverage concern is addressed in its real location. Adding `rule_set_version_id` to the JSON snapshot would break the `REQUIRES_RE_JURY_KEYS` metadata-parity invariant — not an acceptable fix path.

## Harness-chore CR (12 findings on `66749bb71`)

Per user directive in prior session ("We can ignore harness CR comments"), the 12 findings on `.claude/hooks/**`, `.claude/channels/**`, `.claude/routines/**` are **out-of-phase, not addressed in PR #81**. File as separate chore issue post-merge (item #5 on carry-forward list — see §below).

## Phase-close followup issues (file AFTER merge, not during)

Pre-authorised by prior advisor reviews:

1. `chore(lint): clear e2e.rs + governance test-target clippy debt (135 errors at v1-AD-c base)` — reference `.claude/audit-clippy-distribution.txt`.
2. `chore: fix lemmy_api_crud reqwest_middleware transitive compile break in OAuth path` — reference `.claude/build-task4-tests.log`.
3. `chore(test): replace tokio::join! race test with deterministic helper` — now resolved within round 2 + 3 (Fix D helper-extract). Can be dropped from list.
4. `chore(admin-config): update admin-config-write.sh to emit previous_value + previous_from pre-INSERT (option A)` — reference PRD §8.4 amended condition 3 in `5c04e6ec2`.
5. **New (from this round)**: `chore(harness): address 12 CR findings on .claude/hooks/ + channels/ + routines/` — reference CR review on `66749bb71`, raw at `.claude/cr-review-pr81-harness.txt`.

## Risk register (carried from 00-advisor-notes.md)

- r1–r8 — all cleared in earlier rounds
- **r9** (new, round 2) — admin_list_rule_sets snapshot race between `versions` + `active_version_id` reads. **Fix B implemented in working tree, not yet committed. CLEARS ON COMMIT.**
- **r10** (new, round 2) — test/handler mapping duplication in `admin_create_rule_set_duplicate_version_rejected`. **Fix D implemented as helper-extract, not yet committed. CLEARS ON COMMIT.**

## Protocol reminders

- Machine-format runlog. Single-line events, pipe-delimited. Write via `Edit` tool, but if file-race collisions recur use `printf ... >> <file>` via Bash. Edit tool's "file modified since read" error has fired 4× this session due to concurrent Impl writes.
- PM + Impl split still active. Impl has autonomy on CR-triage response under the "auto-mode" directive the user established. Advisor stays on process + merits review, does not gate.
- Every runlog entry: `<ISO-UTC> | who | task | event | data`.
- Commit shape: advisor-verified before push; advisor diff-reviews at stage-before-commit.

## Closing state assertions (verify on resume)

- `git branch --show-current` → `phase-v1-AD-c`
- `git rev-parse HEAD` → `6c8fc3211cd0b9696adabffff044013526df883d`
- `git rev-parse origin/phase-v1-AD-c` → same SHA (origin ahead by 0)
- PR #81 state OPEN, target governance-v0, mergeable MERGEABLE, mergeStateStatus CLEAN
- Working tree: 3 Rust files modified (`admin_rule_sets.rs`, `config.rs`, `e2e.rs`), NOT staged
- `.claude/build-fixD-test-compile.log` exists — tail for exit code

## Morning-session first-message template

After executing the resume sequence, report to the user in this shape:

```
Advisor resumed (CR round 2, pre-commit stage). State confirmed: PR #81 OPEN + MERGEABLE at 6c8fc3211, CI merge-gate GREEN on committed HEAD. 3 Rust files modified in working tree from Fix B + Fix D implementation (Fix C no-op — already in round-2 commit), not yet staged.

Final triage: B + D in-phase (implemented, compile green), A + E rebuttals (replies drafted), C already applied. e2e-compile was running at session close — need to verify tail before proceeding to local-test gates.

Ready to: (1) verify e2e compile, (2) run 4 local gates, (3) stage + commit + push, (4) post rebuttals. Awaiting user go or any adjustment.
```
