# carry-forward from PR #95: 7 follow-ups (3 Major, 2 Low, 2 Nit)

PR #95 (Phase v1-JM-b — jury-mechanics handler: cascade + diversity + severity/status snapshot) merged with 2 of 9 actionable findings addressed inline (cr-3 + cr-4) and 7 carried forward to this issue under fast-merge strategy β so v1-JM-c planning could begin.

This issue batches all 7 carry-forwards. Each is independently actionable and can be cherry-picked into a single fix-up PR or split as the next session prefers.

## Cluster 1 — Constraint-record + config-resolver correctness (3 Major)

**cr-5** — `crates/api/api/src/governance/config.rs:439`
**Severity:** Major
**Summary:** Use candidate-level const fallbacks, not the bare namespace only.
**Detail:** The cascade resolver currently falls back only to `const_default_int(namespace)` (and `const_default_float`) after exhausting DB/cache. This ignores candidate-specific const defaults. The fix is to walk the same candidates list (most-specific to least-specific) and call `const_default_int(candidate)` / `const_default_float(candidate)` for each candidate, returning the first `Some` value. Apply the identical fix to the analogous float-handling block (around line 463-491).
**CR comment:** https://github.com/barrie-cork/lemmy/pull/95#discussion_r3141635551

**cr-8** — `crates/api/api/src/governance/decline_jury_assignment.rs:164-186`
**Severity:** Major
**Summary:** Store the replacement pick's ConstraintRecord on `JuryAssignmentInsertForm` (currently dropped to `None`).
**Detail:** Same cluster as cr-3/cr-4 but on the decline-replacement path. `select_eligible_jurors` returns a `ConstraintRecord` alongside the replacement person_ids, but the replacement insert hardcodes `selected_under_constraints: None`. If the replacement needed a relaxation or small-pool fallback, that audit data is lost.
**CR comment:** https://github.com/barrie-cork/lemmy/pull/95#pullrequestreview-4175278495 (outside-diff comment on `decline_jury_assignment.rs`)

**Recommended approach:** Address cr-8 by mirroring the cr-4 fix shape (capture `ConstraintRecord` and pass through to `JuryAssignmentInsertForm.selected_under_constraints` via `Some(record.to_json())`). cr-5 is independent — config-resolver semantics, no shared code path.

## Cluster 2 — Test-fixture cleanup + assertions (2 Low + 2 Nit + 1 Low)

**cr-1** — `.claude/PRPs/reports/v1-JM-b-retro.md:100`
**Severity:** Low
**Summary:** Add a language tag to fenced code block (markdownlint).
**Fix:** ~~~ → ```bash on the `git grep` snippet around line 98-100.

**cr-2** — `crates/api/api/src/governance/admin_assign_jury.rs:714-718`
**Severity:** Nit
**Summary:** No-op `let _ = current_geo_enabled;` binding remains from prior review.
**Fix:** Delete the binding. The comment above it already documents intent.
**CR re-flagged:** Yes (review #2 marked as ♻️ Duplicate; still present).

**cr-6** — `crates/server/tests/e2e.rs:6961-7053`
**Severity:** Nit
**Summary:** Pull `bootstrap`, `seed_user`, `seed_community`, `seed_jurors` helpers into existing `governance_fixtures` module to avoid drift.
**Fix:** Relocate the four functions (and the `SIGNING_SEED_HEX` constant + unsafe env-set logic) into `governance_fixtures`, adjust visibility to `pub`, update call sites.
**CR re-flagged:** Yes (review #2 marked as ♻️ Duplicate).

**cr-7** — `crates/server/tests/e2e.rs:7060-7089`
**Severity:** Low
**Summary:** `seed_case` can produce inconsistent severity vs severity_tier (currently uses `CaseSeverity::default()` instead of deriving from incoming `severity_tier`).
**Fix:** Map `SeverityTier → CaseSeverity` inside `seed_case` and set `form.severity` to the derived value.
**CR re-flagged:** Yes (review #2 marked as ♻️ Duplicate).

**cr-10** — `crates/server/tests/e2e.rs:7820`
**Severity:** Low
**Summary:** Assert `CaseStatus::EmergencyRemove` in `admin_emergency_remove_case_has_severity_tier_severe` test.
**Detail:** Test currently asserts `severity_tier` and snapshot fields but not `status`. Per ADR-013 (EmergencyRemove from day 1), the status field is part of the contract and should be pinned in the test. One-line fix:
```rust
assert_eq!(case.status, CaseStatus::EmergencyRemove);
```
**Note:** Production correctness is unaffected — `admin_emergency_remove.rs:155` explicitly sets this status. This is test-rigor strengthening, not a discovered bug.
**CR comment:** https://github.com/barrie-cork/lemmy/pull/95#discussion_r3141686616

## Suggested ordering (if split into multiple commits)

1. **`fix(jury): persist ConstraintRecord on decline_jury_assignment replacement (cr-8)`** — Major; same shape as cr-4.
2. **`fix(config): walk candidate-level const fallbacks before namespace-only (cr-5)`** — Major; independent.
3. **`chore(lint): clear PR #95 cosmetic carry-forwards (cr-1, cr-2, cr-7, cr-10)`** — bundle the 4 small lint/assertion fixes.
4. **`refactor(test): consolidate e2e fixture helpers into governance_fixtures module (cr-6)`** — Nit; refactor scope.

Or any equivalent grouping. Cluster 1's Majors are recommended for the next bug-fix cycle (pre-JM-c-merge or as a parallel `fix/v1-JM-b-followups` branch). Cluster 2 can wait for a quiet moment.

## References

- **PR:** https://github.com/barrie-cork/lemmy/pull/95
- **Findings YAML (local, gitignored):** `.claude/PRPs/reviews/pr-95-findings.yaml`
- **Phase retro:** `.claude/PRPs/reports/v1-JM-b-retro.md` (on `phase-v1-JM-b` branch / merged trunk)
- **PR digest comment:** https://github.com/barrie-cork/lemmy/pull/95#issuecomment-4318545474
- **Triage strategy:** β fast-merge — fix cr-3 in PR (impl extended scope to also fix cr-4), carry-forward 7, wont-fix 1
- **Wont-fix excluded from this issue:** cr-9 (PR-body template — Brehon shape vs CR template)

## Labels (recommended)

`carry-forward`, `source-coderabbit`, `area:governance/jury`
