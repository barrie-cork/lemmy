# Brief: v1-quality-r3 BM-PR

## 1. Role + dispatch

`[role:bm-task] bm-pr v1-quality-r3 — open PR phase-v1-quality-r3 → governance-v0 — see .claude/PRPs/briefs/v1-quality-r3-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-quality-r3` → `governance-v0` on `barrie-cork/lemmy`.

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

**Title:** `refactor(e2e): v1-quality-r3 — EnvVarGuard C4 sweep (23 raw set_var sites)`

**Body:**
```
## Summary

Wraps all remaining raw `unsafe { std::env::set_var(...) }` calls in `crates/server/tests/e2e.rs` with `EnvVarGuard` RAII guards, completing the C4 EnvVarGuard retrofit started in v1-quality-r2.

- **Task 1** — SAFETY-justification comments above 2 `bootstrap()` fixture sites (`governance_fixtures::bootstrap` ~line 832, `admin_config_fixtures::bootstrap` ~line 6145). These stay raw (option-b) because wrapping at bootstrap() return drops the guard before the test body runs (see `feedback_envvarguard_fixture_lifetime_footgun.md`).
- **Task 2** — 10 test-body sites replaced with `let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1")` / `let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", <value>)` RAII bindings. 19 insertions, 60 deletions.

## Validation

- `cargo check --workspace --features full`: ✓ exit 0
- `cargo clippy --workspace --features full --no-deps -- -D warnings`: ✓ exit 0
- `cargo test --workspace --test e2e --features full`: ✓ **126 passed, 0 failed, 5 ignored** (E2E_EXIT_0; 2716s, advisor-laptop gate)

## Plan reference

`.claude/PRPs/plans/v1-quality-r3.plan.md`

## ADRs honoured

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
- Phase branch tip at dispatch time: `92d94e33e` (or later if advisor pushed more commits)
