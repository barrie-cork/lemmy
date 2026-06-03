# Brief: v1-redaction-r1 BM-PR

## 1. Role + dispatch

`[role:bm-task] bm-pr v1-redaction-r1 — open PR phase-v1-redaction-r1 → governance-v0 — see .claude/PRPs/briefs/v1-redaction-r1-bm-pr-1.md`

## 2. Scope

Open a PR from `phase-v1-redaction-r1` → `governance-v0` on `barrie-cork/lemmy`.

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

**Title:** `feat(redaction): doc-comments, MAX_RECURSION_DEPTH=64, scrub_json_inner, adversarial tests (v1-redaction-r1)`

**Body:**
```
## Summary

v1-redaction-r1: hardening of `crates/db_schema/src/source/governance/redaction.rs`.

- Adds full module-level doc-comments and `## Maintenance invariants` section citing ADR-015,
  GDPR §17, and `MAX_RECURSION_DEPTH`; documents over-scrub bias and v1-redaction-r2 carry-forwards.
- Introduces `MAX_RECURSION_DEPTH: usize = 64` constant and `scrub_json_inner` private helper,
  converting `scrub_json` from unbounded recursion to a depth-capped traversal (returns `Value::Null`
  at cap rather than truncating a partial tree).
- Adds adversarial unit test corpus (11 tests, 2 ignored pending r2): order-dependence, newlines,
  regex special chars, integer-id preservation, Cyrillic + Unicode mentions.
- Adds per-regex inline comments above `mention_regex` and `email_regex` explaining scrub bias.
- fix(reputation): resolves 6 pre-existing clippy lints (`map_or` → `is_some_and`,
  `usize as i64`/`i64 as i32` → `try_from`) in `reputation_snapshot.rs` (introduced by v1-RT-r5,
  not by this phase).

## Validation

- `cargo clippy --workspace --features full --no-deps -- -D warnings`: ✓ exit 0
- `cargo test -p lemmy_db_schema --features full redaction::tests`: ✓ 11 passed; 0 failed; 2 ignored
- `cargo test --workspace --test e2e --features full` (advisor-laptop gate): ✓ 128 passed; 0 failed; 5 ignored (E2E_EXIT_0)
- Verify report: `.claude/PRPs/reports/v1-redaction-r1-verify.md` — 4/4 stories ✓

## Plan reference

`.claude/PRPs/plans/v1-redaction-r1.plan.md`

## ADRs honoured

- ADR-013 (EmergencyRemove): no change to case status handling
- ADR-015 (pseudonymisation): redaction module hardening directly serves GDPR §17; no schema changes
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
- Phase branch tip at dispatch time: `ef6566bab`
