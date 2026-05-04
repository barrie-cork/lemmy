---
description: |
  Comprehensive PR code review for Brehon — cargo validation, ADR compliance, cross-cutting invariants, posts to GitHub
argument-hint: |
  <pr-number|pr-url> [--approve|--request-changes]
---

# PR Code Review (Brehon)

**Input**: $ARGUMENTS

---

## Your Mission

Perform a senior-engineer-level code review on a Brehon PR:

1. **Understand** what the PR tries to accomplish (intent + plan ancestry)
2. **Check** the code against Lemmy patterns, the 15 ADRs, and the cross-cutting invariants
3. **Run** cargo validation (`check`, `clippy`, `test`, `build`)
4. **Identify** issues by severity
5. **Report** findings as a PR comment AND a local file

**Golden Rule**: Be constructive and actionable. Every issue gets a clear recommendation. Acknowledge good work too.

**Hardness**: a PR that contradicts any of the 15 ADRs in [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) must be BLOCKED. No exceptions. The correct path is a superseding ADR, not a silent code change.

---

## Brehon Context (read every invocation)

- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — the 15 ADRs (hard) and 12 OQs
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` — cross-cutting requirements (§4), risk register (§7)
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — authoritative table/struct/enum/DTO definitions
- `docs/brehon-law-inspired-network/06-security-and-threat-model.md` — threat table to cross-check security-relevant changes
- Fork-local `CLAUDE.md` — pinned upstream SHA

---

## Phase 1: FETCH — Get PR Context

### 1.1 Parse Input

| Input Format | Action |
|---|---|
| Number (`123`, `#123`) | Use as PR number |
| URL (`https://github.com/barrie-cork/lemmy/pull/123`) | Extract PR number |
| Branch name (`feature/phase-1-schema`) | Find associated PR |

```bash
# If branch name provided, find the PR
gh pr list --head {branch-name} --json number -q '.[0].number'
```

### 1.2 Get PR Metadata

```bash
gh pr view {NUMBER} --json number,title,body,author,headRefName,baseRefName,state,additions,deletions,changedFiles,files,reviews,comments

gh pr diff {NUMBER}
gh pr diff {NUMBER} --name-only
```

Extract:
- PR number, title, body
- Author
- Base (`governance-v0` for v0 work) + head branches
- Changed files with line counts
- Existing review comments

### 1.3 Checkout PR Branch

```bash
gh pr checkout {NUMBER}
```

### 1.4 Validate PR State

| State | Action |
|---|---|
| `MERGED` | STOP: "PR already merged, nothing to review" |
| `CLOSED` | WARN: "PR closed — review anyway? (historical analysis)" |
| `DRAFT` | NOTE: "Draft PR — focus on direction, not polish" |
| `OPEN` | PROCEED |

**PHASE_1_CHECKPOINT:**
- [ ] PR identified and fetched
- [ ] Branch checked out
- [ ] Base branch is `governance-v0` (warn if it's `main` — main tracks upstream)

---

## Phase 2: CONTEXT — Understand the Change

### 2.1 Read Project Rules

```bash
cat CLAUDE.md
ls -la .claude/PRPs/plans/ 2>/dev/null
ls -la .claude/PRPs/plans/completed/ 2>/dev/null
ls -la .claude/PRPs/reports/ 2>/dev/null
```

### 2.2 Find Implementation Context

Look for the plan and report that produced this PR:

```bash
# Implementation report by branch
ls .claude/PRPs/reports/*{branch-slug}*.md 2>/dev/null

# Completed plans
ls .claude/PRPs/plans/completed/*{branch-slug}*.md 2>/dev/null

# Investigations
ls .claude/PRPs/issues/completed/ 2>/dev/null
```

**If implementation report exists:**
1. Read the report — note the predicted complexity vs actual
2. Read the referenced plan
3. **Documented deviations are INTENTIONAL** — not issues
4. Note which [IMPLEMENTATION-PLAN-v0.md §3 Phase](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) this advances

**If no implementation report:**
- PR may have been ad-hoc — review without plan context
- Note in the review that no implementation report was found

### 2.3 Understand PR Intent

From the PR title, body, and implementation report (if any):
- What problem does this solve?
- Which v0 step does it advance?
- Which ADRs govern it?
- What deviations are documented?
- What did the author call out as risks?

### 2.4 Categorise Changed Files

For each file in the diff:

| File Type | Expected Patterns |
|---|---|
| `migrations/{ts}_*/up.sql` or `down.sql` | Diesel migration style; enum creation order; reversibility |
| `crates/db_schema/src/source/governance/*.rs` | `Queryable + Selectable + Identifiable`, `Insertable` forms |
| `crates/db_views/*/src/lib.rs` | Denormalised read structs, query functions |
| `crates/api/api_common/src/governance.rs` | DTO `#[derive(Serialize, Deserialize)]` + optional `ts_rs` |
| `crates/api/api/src/governance/*.rs` | Workflow handlers — call `governance_log::append` + `redaction::scrub` |
| `crates/api/api_crud/src/governance/*.rs` | Simple CRUD handlers |
| `crates/api/routes/src/governance.rs` | Route registration only |
| `crates/apub/objects/src/governance/*.rs` | AP objects — serialisation traits |
| `crates/apub/activities/src/governance/*.rs` | AP activities — `Create`, `Undo` wrappers |
| `crates/apub/apub/src/governance/*.rs` | Inbox/outbox — signature verify |
| `crates/server/src/governance.rs` | Composition root only — NO business logic |
| `tests/e2e.rs` | Integration test; must spin up real Postgres |

**Red flag**: business logic inside `crates/server/src/*.rs` ([03 §11](docs/brehon-law-inspired-network/03-architecture.md) — `server` is composition root only). Flag as HIGH.

**PHASE_2_CHECKPOINT:**
- [ ] Project rules read
- [ ] Implementation artifacts located (or absence noted)
- [ ] PR intent understood, phase identified
- [ ] Changed files categorised by crate type

---

## Phase 3: REVIEW — Analyse the Code

### 3.1 Read Each Changed File in Full

For each file, read the full file (not just the diff) so you understand surrounding context. Then read one similar existing Lemmy file to know what "good" looks like.

### 3.2 Brehon Review Checklist

#### ADR Compliance (BLOCKING if violated)

- [ ] **[ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: no v1/v2/v3 scope leaked into the v0 PR (Keycloak, OPA, Vault, external signer, blockchain anchoring, etc.)
- [ ] **[ADR-007](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: jury params are 5 jurors / quorum 3 / simple majority. Not 7-juror or severity-thresholded
- [ ] **[ADR-008](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: every new write path emits a `governance_log::append` entry **before** returning to the user
- [ ] **[ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: `CaseStatus::EmergencyRemove` is explicitly handled in every match. Zero `_ =>` fallthrough arms on `CaseStatus`
- [ ] **[ADR-015](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: no direct `person_id`, username, email, or display name reaches `governance_log.payload`, `public_case_log.summary`, or `public_case_log.rationale_redacted`. All such writes go through `actor_pseudonym::get_or_create` + `redaction::scrub`
- [ ] **[ADR-006](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: inbound federation signals stored as advisory — `remote_sanction_notice` rows with `local_case_id = NULL` are never auto-applied
- [ ] **[ADR-011](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: no relicensing, no removal of AGPL notice or `LICENSE` file
- [ ] **[ADR-012](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**: Lemmy 1.0-beta conventions respected (`ap_id`, not `actor_id`)

#### Correctness

- [ ] Does the code do what the plan / PR claims?
- [ ] Logic errors or off-by-ones (especially in jury vote tallying, quorum check)
- [ ] Edge cases: zero reports, zero jurors, tied votes (MVP says simple majority — document tie-breaking)
- [ ] Appropriate error handling (`LemmyError` / `LemmyResult<T>`)

#### Rust + Cargo Quality

- [ ] No implicit `pub` where `pub(crate)` is correct
- [ ] No `.unwrap()` / `.expect()` outside tests or where the invariant is proven
- [ ] Lifetimes / borrows sensible (no needless clones, no `Arc<Arc<T>>`)
- [ ] Async signatures match existing Lemmy handlers exactly (parameter order matters)
- [ ] Exhaustive match where the compiler asks for it — do not silence with `_ =>`
- [ ] No new `#[allow(clippy::...)]` without a comment justifying it

#### Diesel Hygiene

- [ ] `schema.rs` regenerated from migrations (or edited by `diesel print-schema`), not hand-edited
- [ ] `#[diesel(table_name = xx)]` matches the schema exactly
- [ ] Insert forms use `Insertable`, not `Queryable`
- [ ] Queries use `.select(XxView::as_select())` where applicable
- [ ] No raw SQL strings without `sql_query` justification

#### Migration Hygiene

- [ ] `up.sql` and `down.sql` both exist
- [ ] `down.sql` cleanly reverses `up.sql` (drop in reverse dependency order)
- [ ] Enum types created before tables that reference them (migration ordering)
- [ ] Indexes match [04 §1.5](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- [ ] Postgres triggers, if added, have tests
- [ ] DB grants on `governance_log` still deny UPDATE/DELETE to the app role

#### Pattern Compliance

- [ ] File locations match [03 §7](docs/brehon-law-inspired-network/03-architecture.md) crate layout
- [ ] Naming follows Lemmy conventions (`snake_case`, `lemmy_` crate prefix)
- [ ] Module exports via `pub mod` + `pub use` in parent `mod.rs`
- [ ] Governance code lives under `governance/` subdirectories, not mixed into existing Lemmy modules

#### Security

- [ ] Any user input validated at handler boundary
- [ ] No secrets in committed files
- [ ] No SQL injection paths (raw `sql_query` with untrusted input)
- [ ] `governance_log` write-path has no way to bypass the hash-chain trigger
- [ ] `actor_pseudonym` deletion (if touched) logs to a separate audit table, not `governance_log` itself

#### Performance (non-blocking but worth flagging)

- [ ] No obvious n+1 in view queries
- [ ] Indexes for any new `WHERE` clauses
- [ ] No unbounded `.load()` on large tables
- [ ] No blocking IO inside an async handler

#### Completeness

- [ ] Integration test(s) in `tests/e2e.rs` for new handlers
- [ ] Migration round-trip tested
- [ ] Cross-cutting invariant tests still pass (hash-chain, redaction)
- [ ] No `TODO` or `FIXME` comments blocking merge (flag as MEDIUM)

### 3.3 Categorise Issues

**Important**: check the implementation report first. Documented deviations are intentional, not issues. Only flag **undocumented** deviations.

| Level | Icon | Criteria | Examples |
|---|---|---|---|
| Critical | RED | Blocks merge | ADR contradiction, security hole, data loss, GDPR-log leak, hash-chain bypass |
| High | ORANGE | Should fix before merge | Missing redaction call, missing `EmergencyRemove` arm, migration not reversible, no integration test |
| Medium | YELLOW | Worth addressing | Pattern inconsistency, missing edge case, undocumented deviation, clippy warning suppressed without justification |
| Low | BLUE | Suggestions | Naming, minor optimisation, documentation, comments |

**PHASE_3_CHECKPOINT:**
- [ ] All changed files reviewed
- [ ] ADR compliance checked against [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- [ ] Cross-cutting invariants checked
- [ ] Issues categorised by severity
- [ ] Implementation-report deviations accounted for
- [ ] Positive aspects noted

---

## Phase 4: VALIDATE — Run Automated Checks

### 4.1 Run Cargo Validation

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --test e2e
cargo build --workspace
```

Capture for each:
- Pass/fail
- Error/warning count
- Specific failures (first 10 lines of rustc output)

### 4.2 Migration Check (if schema touched)

```bash
diesel migration run
diesel migration redo
psql -h localhost -U lemmy -d lemmy_test -c '\d {new_or_changed_table}'
```

### 4.3 Cross-Cutting Spot Checks

```bash
# Exhaustive match guard: every CaseStatus match should reference EmergencyRemove
grep -rn 'match.*\.status\b\|match.*CaseStatus' crates/ | head -30

# No direct person_id in governance_log writes
grep -rn 'governance_log.*person_id\|person_id.*governance_log' crates/ | head

# Every public_case_log write should show up paired with redaction
grep -rn 'public_case_log\|PublicCaseLog' crates/ -l | xargs -I {} grep -L 'redaction::scrub\|scrub(' {} 2>/dev/null
```

If any of the greps returns suspicious results, flag as HIGH.

### 4.4 Regression Check

```bash
# Full integration test suite — not just the new tests
cargo test --test e2e
```

**PHASE_4_CHECKPOINT:**
- [ ] `cargo check` executed
- [ ] `cargo clippy` executed
- [ ] `cargo test --test e2e` executed
- [ ] `cargo build` executed
- [ ] Migration round-trip executed (if applicable)
- [ ] Cross-cutting spot checks run
- [ ] Results captured

---

## Phase 5: DECIDE — Form Recommendation

### 5.1 Decision Logic

**APPROVE** if:
- Zero critical or high issues
- All cargo validation passes
- ADR compliance checks all tick
- Code follows Lemmy patterns
- Changes match PR intent + plan

**REQUEST CHANGES** if:
- Any high issues exist (and they're fixable)
- Validation fails but the failure is clear and fixable
- Missing tests for new functionality
- Undocumented deviations from the plan

**BLOCK** if:
- ANY ADR contradiction (critical)
- Security issue (hash-chain bypass, GDPR leak, auth bypass)
- Data loss potential (non-reversible migration)
- Business logic in `crates/server/` ([03 §11](docs/brehon-law-inspired-network/03-architecture.md))
- Breaking change without migration / without acknowledgement

### 5.2 Special Cases

| Situation | Handling |
|---|---|
| Draft PR | Comment only, no approve/block — focus on direction |
| Large PR (>500 lines) | Note thoroughness limits; suggest splitting per-task |
| Security-sensitive (log, pseudonyms, `EmergencyRemove`) | Extra scrutiny; err on BLOCK |
| Missing tests for new handlers | High priority; may not block if plan says "test in next phase" |
| Upstream rebase in the same PR | Flag as "review upstream diff separately" |

**PHASE_5_CHECKPOINT:**
- [ ] Recommendation determined
- [ ] Rationale tied to ADRs / invariants / test results

---

## Phase 6: REPORT — Generate Review

### 6.1 Create Report Directory

```bash
mkdir -p .claude/PRPs/reviews
```

### 6.2 Generate Report File

**Path**: `.claude/PRPs/reviews/pr-{NUMBER}-review.md`

```markdown
---
pr: {NUMBER}
title: "{TITLE}"
author: "{AUTHOR}"
reviewed: {ISO_TIMESTAMP}
recommendation: {approve|request-changes|block}
---

# PR Review: #{NUMBER} — {TITLE}

**Author**: @{author}
**Branch**: {head} → {base}
**Files Changed**: {count} (+{additions}/-{deletions})
**Advances**: [IMPLEMENTATION-PLAN-v0.md §3 Phase {N}](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) (or "Ad-hoc")

---

## Summary

{2–3 sentences}

---

## Implementation Context

| Artifact | Path |
|---|---|
| Implementation Report | `{path}` or "Not found" |
| Original Plan | `{path}` or "Not found" |
| Documented Deviations | {count} |

{If report exists: brief note about deviation quality}

---

## ADR Compliance

| ADR | Check | Status |
|---|---|---|
| [ADR-006](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Inbound federation advisory-only | PASS / **FAIL** |
| [ADR-007](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Jury params 5/3/majority | PASS / **FAIL** |
| [ADR-008](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Log append before response | PASS / **FAIL** |
| [ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | No v1/v2/v3 scope leak | PASS / **FAIL** |
| [ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | `EmergencyRemove` exhaustive | PASS / **FAIL** |
| [ADR-015](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | Pseudonym + redaction | PASS / **FAIL** |

---

## Changes Overview

| File | Changes | Assessment |
|---|---|---|
| `migrations/.../up.sql` | +{N}/-{M} | PASS / WARN / FAIL |
| `crates/db_schema/...` | +{N}/-{M} | PASS / WARN / FAIL |

---

## Issues Found

### Critical

{If none: "No critical issues found."}

- **`{file}:{line}`** — {description}
  - **Why**: {problem}
  - **Fix**: {specific recommendation}

### High

{issues to fix before merge}

### Medium

{worth addressing, not blocking}

### Suggestions

{nice-to-haves}

---

## Validation Results

| Check | Status | Details |
|---|---|---|
| `cargo check --workspace` | PASS / **FAIL** | {notes} |
| `cargo clippy -- -D warnings` | PASS / **FAIL** | {N} warnings |
| `cargo test --test e2e` | PASS / **FAIL** | {X}/{Y} passed |
| `cargo build --workspace` | PASS / **FAIL** | {notes} |
| Migration round-trip | PASS / ⏭️ / **FAIL** | {notes} |

---

## Cross-Cutting Invariants

- [ ] Hash chain: every new governance write calls `governance_log::append`
- [ ] Redaction: every string to log passes through `redaction::scrub`
- [ ] Pseudonyms: every governance-log actor reference uses `actor_pseudonym::get_or_create`
- [ ] `EmergencyRemove`: exhaustively handled in every `CaseStatus` match
- [ ] AGPL notice: `LICENSE` + `AGPL-NOTICE.md` present and unchanged
- [ ] `crates/server/` has no new business logic

---

## What's Good

{Acknowledge positive aspects — clean Diesel work, thorough tests, good error propagation}

---

## Recommendation

**{APPROVE / REQUEST CHANGES / BLOCK}**

{Rationale tied to specific issues and ADR status}

---

*Reviewed by Claude*
*Report: `.claude/PRPs/reviews/pr-{NUMBER}-review.md`*
```

**PHASE_6_CHECKPOINT:**
- [ ] Report file created
- [ ] All sections populated

---

## Phase 7: PUBLISH — Post to GitHub

### 7.1 Review Action

```bash
# If --approve AND no critical/high
gh pr review {NUMBER} --approve --body-file .claude/PRPs/reviews/pr-{NUMBER}-review.md

# If --request-changes OR high/critical issues found
gh pr review {NUMBER} --request-changes --body-file .claude/PRPs/reviews/pr-{NUMBER}-review.md

# Otherwise just comment
gh pr comment {NUMBER} --body-file .claude/PRPs/reviews/pr-{NUMBER}-review.md
```

**For BLOCK**, always use `--request-changes` with the critical issues section expanded.

### 7.2 Capture the Review URL

```bash
gh pr view {NUMBER} --json reviews,comments --jq '.reviews[-1].url // .comments[-1].url'
```

**PHASE_7_CHECKPOINT:**
- [ ] Review / comment posted
- [ ] URL captured

---

## Phase 8: OUTPUT — Report to User

```markdown
## PR Review Complete

**PR**: #{NUMBER} — {TITLE}
**URL**: {pr_url}
**Recommendation**: {APPROVE / REQUEST CHANGES / BLOCK}

### ADR Compliance

| ADR | Status |
|---|---|
| ADR-007 | {PASS/FAIL} |
| ADR-008 | {PASS/FAIL} |
| ADR-013 | {PASS/FAIL} |
| ADR-015 | {PASS/FAIL} |

### Issues

| Severity | Count |
|---|---|
| Critical | {N} |
| High | {N} |
| Medium | {N} |
| Suggestions | {N} |

### Validation

| Check | Result |
|---|---|
| `cargo check` | {PASS/FAIL} |
| `cargo clippy` | {PASS/FAIL} |
| `cargo test --test e2e` | {PASS/FAIL} |
| `cargo build` | {PASS/FAIL} |

### Artifacts

- Report: `.claude/PRPs/reviews/pr-{NUMBER}-review.md`
- GitHub comment: {comment_url}

### Next Steps

{Based on recommendation:}
- APPROVE: "Ready for human merge"
- REQUEST CHANGES: "Author should address {N} high-priority issues"
- BLOCK: "ADR contradiction or security issue — resolution required before reopening"
```

---

## Critical Reminders

1. **ADRs are hard.** An ADR contradiction is always BLOCK. The superseding-ADR path is the only way to change direction.

2. **Understand before judging.** Read the full file, not just the diff. Read one similar Lemmy file to know what "good" looks like.

3. **Be specific.** "This could be better" is useless. "Use `Uuid::new_v4()` via `actor_pseudonym::get_or_create` at line 45 to satisfy ADR-015" is helpful.

4. **Prioritise honestly.** Not every finding is critical. Use the severity levels as defined.

5. **Acknowledge good work.** If the Diesel pattern is clean or the test is thorough, say so.

6. **Run cargo.** Don't skip automated checks — `cargo check` + `cargo clippy` + `cargo test` are the floor.

7. **Check cross-cutting grep spots.** The redaction + pseudonym + `EmergencyRemove` greps in §4.3 catch classes of issues no human review will.

8. **Documented deviations are not issues.** Check the implementation report first.

9. **Trust the PR doc, verify with the code.** The PR description is authorial intent; the diff is fact. Compare.

---

## Success Criteria

- **CONTEXT_GATHERED**: PR metadata, diff, implementation artifacts reviewed
- **ADR_VERIFIED**: Every relevant ADR checked against the diff
- **CODE_REVIEWED**: All changed files analysed against the checklist
- **VALIDATION_RUN**: `cargo check`/`clippy`/`test`/`build` all executed
- **CROSS_CUTTING_VERIFIED**: Hash chain / pseudonyms / redaction / `EmergencyRemove` spot-checked
- **ISSUES_CATEGORISED**: Findings organised by severity
- **REPORT_GENERATED**: Local review file exists
- **PR_UPDATED**: GitHub review posted
- **RECOMMENDATION_CLEAR**: Approve / request-changes / block with rationale tied to ADRs and validation output
</content>
</invoke>