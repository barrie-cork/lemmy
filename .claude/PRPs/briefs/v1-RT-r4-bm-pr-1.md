# Brief: v1-RT-r4 BM-PR

## 1. Role + dispatch

`[role:bm-task] v1-RT-r4-bm-pr — see .claude/PRPs/briefs/v1-RT-r4-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-RT-r4` → `governance-v0` on `barrie-cork/lemmy`.

**Produce:**
- PR opened (not draft) with title and body per §3 below
- PR number recorded in runlog or task output

**Do NOT:**
- Merge the PR
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`, `tests/`, `migrations/`
- Push any commits to the phase branch
- Comment on or review the PR (separate confirm-gated verb)
- Send Telegram pings (separate confirm-gated verb)

## 3. PR title + body

**Title:** `feat(rt-r4): v1-RT-r4 — sponsor-gate strategies + admin allowlist`

**Body:**
```
## Summary

Adds three new sponsor-gate strategies (`age_or_surety`, `reputation`, `allowlist`) and the admin sponsor-allowlist maintenance endpoints.

- **Task 1** — `sponsor_allowlist_insert/delete/exists` inline db-helpers in `crates/db_schema/src/source/governance/sponsor_allowlist.rs`
- **Task 2** — `AddSponsorAllowlist` / `AddSponsorAllowlistResponse` / `RemoveSponsorAllowlist` / `RemoveSponsorAllowlistResponse` admin DTOs in `crates/api/api_common/src/governance.rs` (derive stack mirrored from `AdminSetConfig`)
- **Task 3** — `SponsorGateStrategy` enum extended (`AgeOrSurety`, `Reputation`, `Allowlist` arms) + parse + label + 3 dispatch arms in `crates/api/api_crud/src/governance/create_endorsement.rs`. GOTCHA-55a preserved (no `_ =>` catchall; `Unknown(String)` final arm)
- **Task 4** — `crates/api/api/src/governance/admin_sponsor_allowlist.rs` `add` + `remove` handlers (mirror `admin_set_config` step ordering; ADR-015 pseudonym-only payloads; `governance_log::append` owns its own tx)
- **Task 5** — routes `/api/v4/governance/admin/sponsor-allowlist/{add,remove}` registered in `crates/api/routes/src/lib.rs`
- **Task 6** — registry rows flipped active in `.claude/rules/governance-log-entry-kind-registry.md` (`sponsor_allowlist_added` / `sponsor_allowlist_removed`; no `(pending)` markers)
- **Task 7** — `mod v1_rt_r4_fixtures` (7 e2e tests) in `crates/server/tests/e2e.rs` covering each strategy pass+deny + admin add/remove round-trip

## Validation

- `cargo check --workspace --features full`: ✓ exit 0
- `cargo clippy --workspace --features full --no-deps -- -D warnings`: ✓ exit 0
- `cargo test --workspace --test e2e --no-run --features full`: ✓ exit 0
- `cargo test --workspace --test e2e --features full`: ✓ **126 passed, 0 failed, 5 ignored** (E2E_EXIT_0; 2506s)

## Verify report

`.claude/PRPs/reports/v1-RT-r4-verify.md` — both §16a stories ✓:
- Story A (strategy arms gate endorsement): all 6 strategy tests pass; enum/parse/label/arms + `sponsor_allowlist_exists` wiring confirmed
- Story B (admin allowlist maintenance): add/remove round-trip passes; handlers + routes + registry rows confirmed

## Plan reference

`.claude/PRPs/plans/v1-RT-r4.plan.md`

## ADRs honoured

- ADR-015 (pseudonymisation): governance-log payloads carry `person_pseudonym` + `*_admin_pseudonym`, never raw `person_id`
- ADR-008 (governance log): all writes via `governance_log::append`, never direct INSERT
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
- Record PR number in `.claude/runlog/v1-RT-r4-runlog.md` or task output
- Note: v1-RT-r4 retro (Task 8) is authored separately and gates `bm-merge`, not this `bm-pr`
