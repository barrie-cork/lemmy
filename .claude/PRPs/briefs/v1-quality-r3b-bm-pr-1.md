# Brief: v1-quality-r3b BM-PR

## 1. Role + dispatch

`[role:bm-task] bm-pr v1-quality-r3b — open PR phase-v1-quality-r3b → governance-v0 — see .claude/PRPs/briefs/v1-quality-r3b-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-quality-r3b` → `governance-v0` on `barrie-cork/lemmy`.

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

**Title:** `fix(governance): capture DB URL at LemmyContext::create — drop test band-aids (issue #167)`

**Body:**
```
## Summary

Fixes issue #167: `admin_audit_stream` was re-reading `LEMMY_DATABASE_URL` from the live process
environment at handler invocation time. When the test fixture's `EnvVarGuard` drops at fixture
return, the env var is unset and the handler fails with a broken connection string.

- Adds `db_url: String` field to `LemmyContext`, populated by `SETTINGS.get_database_url()` inside
  `create()` before the struct literal.
- Adds `pub fn database_url(&self) -> &str` accessor (called from `lemmy_api`, cross-crate so `pub`
  required).
- Switches `admin_audit_stream.rs:125` from `context.settings().get_database_url()` to
  `context.database_url()`; drops the `&` at the old `tokio_postgres::connect` call site (`&str`
  not `String` — `&&str` would not implement `TryInto<Config>`).
- Removes the 3 per-test `EnvVarGuard::set("LEMMY_DATABASE_URL", …)` band-aids introduced in
  v1-quality-r2 from the three `admin_audit_stream_*` test bodies. The fixture-internal guard at
  `e2e.rs:6128` is preserved — it is what makes `create()`'s internal `get_database_url()` see the
  testcontainer URL.

## Validation

- `cargo check --workspace --features full`: ✓ exit 0
- `cargo clippy --workspace --features full --no-deps -- -D warnings`: ✓ exit 0
- `cargo test --workspace --test e2e --features full admin_audit_stream`: ✓ **3 passed, 0 failed** (exit 0; 70s)
- `cargo test --workspace --test e2e --features full`: ✓ **126 passed, 0 failed, 5 ignored** (E2E_EXIT_0; 2788s, advisor-laptop gate)

## Plan reference

`.claude/PRPs/plans/v1-quality-r3b.plan.md`

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
- Phase branch tip at dispatch time: `7cf2b771f` (or later if advisor pushed more commits)
