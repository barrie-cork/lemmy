# Brief: v1-quality-r3c BM-PR

## 1. Role + dispatch

`[role:bm-task] bm-pr v1-quality-r3c — open PR phase-v1-quality-r3c → governance-v0 — see .claude/PRPs/briefs/v1-quality-r3c-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-quality-r3c` → `governance-v0` on `barrie-cork/lemmy`.

**Produce:**
- PR opened (not draft) with title and body per §3 below
- PR number recorded in task output

**Do NOT:**
- Merge the PR
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`, `tests/`, `migrations/`
- Push any commits to the phase branch
- Comment on or review the PR (separate confirm-gated verb)
- Send Telegram pings (separate confirm-gated verb)

## 3. PR title + body

**Title:** `fix(governance): quality-r3c — CR endpoint-rule fix + sponsor-allowlist sweep + DISABLE_* guard tests`

**Body:**
```
## Summary

Three quality items requiring no new Rust logic — only targeted edits to config and test files.

- **T1 (Issue #165):** Removes the stale "EXACTLY 11 endpoints" assertion from `.coderabbit.yaml`
  that fired on every v1 PR adding a sanctioned endpoint. Replaced with a v1-aware caveat
  referencing OQ-020 and the v1 PRDs.

- **T2 (Issue #166):** Extends the `all_mvp_endpoints_return_non_404` Phase A sweep in `e2e.rs`
  with the two missing sponsor-allowlist POST routes (`/add` and `/remove`). Route count: 14 → 16.

- **T3:** Adds test coverage for two previously-uncovered `BREHON_DISABLE_*` job guards:
  `BREHON_DISABLE_SNAPSHOT_JOB` and `BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB`.
  Tests mirror the existing `test_brehon_disable_participation_job` shape (EnvVarGuard::set +
  direct function call + DB row-count assertion).

## Validation

- `cargo check --workspace --features full`: ✓ exit 0
- `cargo test --workspace --test e2e --features full`: ✓ **128 passed, 0 failed, 5 skipped**
  (E2E_EXIT_NONZERO is bat-wrapper artifact; test result line is authoritative)
- `grep 'EXACTLY 11' .coderabbit.yaml` = 0 ✓
- `grep -c 'sponsor-allowlist' crates/server/tests/e2e.rs` = 2 ✓
- `test_brehon_disable_snapshot_job` and `test_brehon_disable_fed_replay_cleanup_job` both present ✓

## Plan reference

`.claude/PRPs/plans/v1-quality-r3c.plan.md`

## ADRs honoured

- ADR-010 (v0 scope): T1 correctly scopes the CR rule to v0 only; v1 extensions under OQ-020 are sanctioned
- ADR-013 (EmergencyRemove): no change to case status handling
- ADR-015 (pseudonymisation): no schema changes in this phase
```

## 4. Required reading

- `.claude/rules/branch-manager.md` — file ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — base MUST be `governance-v0`, not draft

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- Base branch: `governance-v0` (NOT `main`)
- Not draft (CodeRabbit skips drafts)
- No force-push
- Record PR number in task output
- Phase branch tip at dispatch time: `0216babc9`
