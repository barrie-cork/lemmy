---
description: |
  Investigate a Brehon GitHub issue — codebase analysis across crates/, design-doc cross-check, artifact for /prp-issue-fix
argument-hint: |
  <issue-number|url|"description">
---

# Investigate Issue (Brehon)

**Input**: $ARGUMENTS

---

## Your Mission

Investigate the issue/problem and produce a comprehensive Rust implementation plan that:

1. Can be executed by `/prp-issue-fix`
2. Is posted as a GitHub comment (if GH issue provided)
3. Captures all context needed for one-pass implementation
4. Cross-references the Brehon design docs where the fix touches governance primitives

**Golden Rule**: the artifact IS the specification. The implementing agent should be able to work from it without asking questions.

**Investigation scope**: focus on `crates/` (Rust workspace) and `migrations/` (Diesel SQL). Do NOT look for `src/`, `package.json`, `pyproject.toml` — this is a Rust-only fork.

**Brehon red flags** to look for immediately when reading an issue:
- Symptom mentions "governance log" / "hash chain" / "verification" → [ADR-008](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) territory, high-stakes
- Symptom mentions "username leaked" / "GDPR" / "can't delete user" → [ADR-015](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) territory, GDPR-critical
- Symptom mentions "EmergencyRemove" / "admin removal" → [ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- Symptom mentions "remote sanction" / "federation applied" → [ADR-006](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — inbound must stay advisory
- Symptom mentions "Diesel error" / "migration failed" → schema drift is likely; check `diesel migration redo`
- Symptom mentions "upstream" / "Lemmy 1.0 changed" → likely an upstream rebase broke patterns

---

## Phase 1: PARSE - Understand Input

### 1.1 Determine Input Type

**Check the input format:**

- Looks like a number (`123`, `#123`) → GitHub issue number
- Starts with `http` → GitHub URL (extract issue number)
- Anything else → Free-form description

```bash
# If GitHub issue, fetch it:
gh issue view {number} --json title,body,labels,comments,state,url,author
```

### 1.2 Extract Context

**If GitHub issue:**

- Title: What's the reported problem?
- Body: Details, reproduction steps, expected vs actual
- Labels: bug? enhancement? documentation?
- Comments: Additional context from discussion
- State: Is it still open?

**If free-form:**

- Parse as problem description
- Note: No GitHub posting (artifact only)

### 1.3 Classify Issue Type

| Type          | Indicators                                              |
| ------------- | ------------------------------------------------------- |
| BUG           | "broken", "error", "crash", "doesn't work", stack trace |
| ENHANCEMENT   | "add", "support", "feature", "would be nice"            |
| REFACTOR      | "clean up", "improve", "simplify", "reorganize"         |
| CHORE         | "update", "upgrade", "maintenance", "dependency"        |
| DOCUMENTATION | "docs", "readme", "clarify", "example"                  |

### 1.4 Assess Severity/Priority, Complexity, and Confidence

Each assessment requires a **one-sentence reasoning** explaining WHY you chose that value. This reasoning must be based on concrete findings from your investigation (codebase exploration, git history, integration analysis).

**For BUG issues - Severity:**

| Severity | Criteria                                                            |
| -------- | ------------------------------------------------------------------- |
| CRITICAL | System down, data loss, security vulnerability, no workaround       |
| HIGH     | Major feature broken, significant user impact, difficult workaround |
| MEDIUM   | Feature partially broken, moderate impact, workaround exists        |
| LOW      | Minor issue, cosmetic, edge case, easy workaround                   |

**For ENHANCEMENT/REFACTOR/CHORE/DOCUMENTATION - Priority:**

| Priority | Criteria                                                   |
| -------- | ---------------------------------------------------------- |
| HIGH     | Blocking other work, frequently requested, high user value |
| MEDIUM   | Important but not urgent, moderate user value              |
| LOW      | Nice to have, low urgency, minimal user impact             |

**Complexity** (based on codebase findings):

| Complexity | Criteria                                                                |
| ---------- | ----------------------------------------------------------------------- |
| HIGH       | 5+ files, multiple integration points, architectural changes, high risk |
| MEDIUM     | 2-4 files, some integration points, moderate risk                       |
| LOW        | 1-2 files, isolated change, low risk                                    |

**Confidence** (based on evidence quality):

| Confidence | Criteria                                                     |
| ---------- | ------------------------------------------------------------ |
| HIGH       | Clear root cause, strong evidence, well-understood code path |
| MEDIUM     | Likely root cause, some assumptions, partially understood    |
| LOW        | Uncertain root cause, limited evidence, many unknowns        |

**PHASE_1_CHECKPOINT:**

- [ ] Input type identified (GH issue or free-form)
- [ ] Issue content extracted
- [ ] Type classified
- [ ] Severity (bug) or Priority (other) assessed with reasoning
- [ ] Complexity assessed with reasoning (after Phase 2)
- [ ] Confidence assessed with reasoning (after Phase 3)
- [ ] If GH issue: confirmed it's open and not already has PR

---

## Phase 2: EXPLORE — Codebase Intelligence

**CRITICAL**: launch up to 2 `Explore` agents in parallel via the `Agent` tool with `subagent_type="Explore"`. One for code location, one for code-path analysis.

### 2.1 Agent 1 — Location & Patterns

```
Find all Rust code relevant to this issue in brehon-fork (Lemmy 1.0-beta
workspace at C:\Users\barri\Developer\brehon-fork\):

ISSUE: {title/description}

LOCATE (return file:line refs and actual code snippets, not invented examples):
1. Files directly related — which crates/module(s)?
2. Similar patterns elsewhere to mirror (existing Lemmy handlers, views, schemas)
3. Existing integration test patterns (api_tests/ and tests/e2e.rs)
4. Error handling — LemmyError / LemmyResult<T> usage
5. Relevant Diesel definitions in crates/db_schema/src/schema.rs and src/source/
6. Relevant enums in crates/db_schema/src/source/governance/enums.rs (if exists)
7. Any existing governance code under crates/**/governance/

Categorise by: schema / model / view / DTO / handler / route / test / apub / server-wiring.
```

### 2.2 Agent 2 — Data Flow & Integration

```
Analyse the data flow around this issue in brehon-fork:

ISSUE: {title/description}

TRACE:
1. End-to-end path — which route → which handler → which db call → which response
2. Integration points — what calls this, what it calls
3. Transaction boundaries and side effects
4. Cross-cutting: does this path touch governance_log, actor_pseudonym, redaction,
   or CaseStatus::EmergencyRemove? If yes, call it out with file:line.
5. Error propagation — how do failures surface to the user?

Document with precise file:line references. No suggestions, no improvements.
```

### 2.3 Merge and Document Findings

| Area | File:Lines | Notes |
|---|---|---|
| Core handler | `crates/api/api/src/governance/xx.rs:NN-MM` | Main function affected |
| Callers | `crates/api/routes/src/governance.rs:NN-MM` | Wires the route |
| Schema | `crates/db_schema/src/schema.rs:NN-MM` | Relevant Diesel table |
| Model | `crates/db_schema/src/source/governance/xx.rs:NN-MM` | `Queryable` struct |
| Enums | `crates/db_schema/src/source/governance/enums.rs:NN-MM` | `CaseStatus`, `JuryDecision`, ... |
| Tests | `tests/e2e.rs:NN-MM` | Existing test pattern |
| Similar | `crates/api/api/src/{similar}.rs:NN-MM` | Pattern to mirror |
| Cross-cutting | `crates/api/api/src/governance/governance_log.rs` | Log append helper |

**PHASE_2_CHECKPOINT:**

- [ ] Both `Explore` agents (one for code location, one for code-path analysis) launched in parallel and completed
- [ ] Core files identified with line numbers
- [ ] Integration points mapped with data flow traces
- [ ] Similar patterns found to mirror
- [ ] Test patterns documented

---

## Phase 3: ANALYZE - Form Approach

### 3.1 For BUG Issues - Root Cause Analysis

Apply the 5 Whys:

```
WHY 1: Why does [symptom] occur?
→ Because [cause A]
→ Evidence: `crates/api/api/src/governance/case.rs:123` - {code snippet}

WHY 2: Why does [cause A] happen?
→ Because [cause B]
→ Evidence: {proof}

... continue until you reach fixable code ...

ROOT CAUSE: [the specific code/logic to change]
Evidence: `crates/db_schema/src/source/governance/log.rs:456` - {the problematic code}
```

**Check git history:**

```bash
git log --oneline -10 -- {affected-file}
git blame -L {start},{end} {affected-file}
```

### 3.2 For ENHANCEMENT/REFACTOR Issues

**Identify:**

- What needs to be added/changed?
- Where does it integrate?
- What are the scope boundaries?
- What should NOT be changed?

### 3.3 For All Issues

**Determine:**

- Files to CREATE (new files)
- Files to UPDATE (existing files)
- Files to DELETE (if any)
- Dependencies and order of changes
- Edge cases and risks
- Validation strategy

**PHASE_3_CHECKPOINT:**

- [ ] Root cause identified (for bugs) OR change rationale clear (for enhancements)
- [ ] All affected files listed with specific changes
- [ ] Scope boundaries defined (what NOT to change)
- [ ] Risks and edge cases identified
- [ ] Validation approach defined

---

## Phase 4: GENERATE - Create Artifact

### 4.1 Artifact Path

```bash
mkdir -p .claude/PRPs/issues
```

**Path:** `.claude/PRPs/issues/issue-{number}.md`

If free-form (no issue number): `.claude/PRPs/issues/investigation-{timestamp}.md`

### 4.2 Artifact Template

Write this structure to the artifact file.

**Note on Severity vs Priority:**

- Use **Severity** for BUG type (CRITICAL, HIGH, MEDIUM, LOW)
- Use **Priority** for all other types (HIGH, MEDIUM, LOW)

**Important:** Each assessment must include a one-sentence reasoning based on your investigation findings.

````markdown
# Investigation: {Title}

**Issue**: #{number} ({url})
**Type**: {BUG|ENHANCEMENT|REFACTOR|CHORE|DOCUMENTATION}
**Investigated**: {ISO timestamp}

### Assessment

| Metric     | Value                         | Reasoning                                                                |
| ---------- | ----------------------------- | ------------------------------------------------------------------------ |
| Severity   | {CRITICAL\|HIGH\|MEDIUM\|LOW} | {Why this severity? Based on user impact, workarounds, scope of failure} |
| Complexity | {LOW\|MEDIUM\|HIGH}           | {Why this complexity? Based on files affected, integration points, risk} |
| Confidence | {HIGH\|MEDIUM\|LOW}           | {Why this confidence? Based on evidence quality, unknowns, assumptions}  |

<!-- For non-BUG types, replace Severity row with Priority:
| Priority | {HIGH\|MEDIUM\|LOW} | {Why this priority? Based on user value, blocking status, frequency} |
-->

---

## Problem Statement

{Clear 2-3 sentence description of what's wrong or what's needed}

---

## Analysis

### Root Cause / Change Rationale

{For BUG: The 5 Whys chain with evidence}
{For ENHANCEMENT: Why this change and what it enables}

### Evidence Chain

WHY: {symptom}
↓ BECAUSE: {cause 1}
Evidence: `crates/api/api/src/governance/case.rs:123` - `{code snippet}`

↓ BECAUSE: {cause 2}
Evidence: `crates/db_schema/src/source/governance/log.rs:456` - `{code snippet}`

↓ ROOT CAUSE: {the fixable thing}
Evidence: `crates/api/api/src/governance/case.rs:789` - `{problematic code}`

### Affected Files

| File | Lines | Action | Description |
|---|---|---|---|
| `crates/api/api/src/governance/xx.rs` | 45-60 | UPDATE | {what changes} |
| `crates/db_schema/src/source/governance/xx.rs` | 12-30 | UPDATE | {what changes} |
| `tests/e2e.rs` | NEW | CREATE | {test to add} |

### Integration Points

- `crates/api/routes/src/governance.rs:NN` registers the route
- `crates/server/src/governance.rs:NN` wires the background job (if applicable)
- Cross-cutting: `governance_log::append` / `actor_pseudonym::get_or_create` / `redaction::scrub` (if applicable)
- {other dependencies}

### Git History

- **Introduced**: {commit} - {date} - "{message}"
- **Last modified**: {commit} - {date}
- **Implication**: {regression? original bug? long-standing?}

---

## Implementation Plan

### Step 1: {First change description}

**File**: `crates/api/api/src/governance/xx.rs`
**Lines**: 45-60
**Action**: UPDATE

**Current code:**

```rust
// Line 45-50
{actual current Rust code}
```
````

**Required change:**

```rust
// What it should become
{the Rust fix/change}
```

**Why**: {brief rationale}

---

### Step 2: {Second change description}

{Same structure...}

---

### Step N: Add/Update Integration Tests

**File**: `tests/e2e.rs`
**Action**: {CREATE|UPDATE}

**Test cases to add** (Brehon uses integration-only tests per [IMPLEMENTATION-PLAN-v0.md §5](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) — no unit tests until something breaks twice):

```rust
#[tokio::test]
async fn test_{expected_behavior}() {
    // Spin up Postgres in Docker with --user $(id -u):$(id -g)
    // Seed fixtures, call handler, assert the effect
}

#[tokio::test]
async fn test_{edge_case}() {
    // Verify the fix holds under the specific condition from the RCA
}
```

**Warning**: always launch the test Postgres container with `--user $(id -u):$(id -g)` — otherwise root-owned files block git worktree cleanup.

---

## Patterns to Follow

**From codebase — mirror these exactly:**

```rust
// SOURCE: crates/api/api/src/{similar}.rs:20-30
// Pattern for {what this demonstrates}
{actual Rust snippet from the workspace}
```

---

## Edge Cases & Risks

| Risk/Edge Case | Mitigation      |
| -------------- | --------------- |
| {risk 1}       | {how to handle} |
| {edge case}    | {how to handle} |

---

## Validation

### Automated Checks

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --test e2e {relevant-pattern}
```

If the fix touches migrations:

```bash
diesel migration run
diesel migration redo
```

If it touches governance_log / actor_pseudonym / redaction:

```bash
cargo test --test e2e governance_log_hash_chain_holds
cargo test --test e2e redaction_strips_identifiers
```

### Manual Verification

1. {Step to verify the fix/feature works}
2. {Step to verify no regression}

---

## Scope Boundaries

**IN SCOPE:**

- {what we're changing}

**OUT OF SCOPE (do not touch):**

- {what to leave alone}
- {future improvements to defer}

---

## Metadata

- **Investigated by**: Claude
- **Timestamp**: {ISO timestamp}
- **Artifact**: `.claude/PRPs/issues/issue-{number}.md`

````

**PHASE_4_CHECKPOINT:**
- [ ] Artifact file created
- [ ] All sections filled with specific content
- [ ] Code snippets are actual (not invented)
- [ ] Steps are actionable without clarification

---

## Phase 5: COMMIT - Save Artifact

```bash
git add .claude/PRPs/issues/
git status
````

**If changes to commit:**

```bash
git commit -m "Investigate issue #{number}: {brief title}"
```

**PHASE_5_CHECKPOINT:**

- [ ] Artifact committed to git

---

## Phase 6: POST - GitHub Comment

**Only if input was a GitHub issue (not free-form):**

Format the artifact for GitHub and post:

````bash
gh issue comment {number} --body "$(cat <<'EOF'
## 🔍 Investigation: {Title}

**Type**: `{TYPE}`

### Assessment

| Metric | Value | Reasoning |
|--------|-------|-----------|
| {Severity or Priority} | `{VALUE}` | {one-sentence why} |
| Complexity | `{COMPLEXITY}` | {one-sentence why} |
| Confidence | `{CONFIDENCE}` | {one-sentence why} |

---

### Problem Statement

{problem statement from artifact}

---

### Root Cause Analysis

{evidence chain, formatted for GitHub}

---

### Implementation Plan

| Step | File | Change |
|---|---|---|
| 1 | `crates/api/api/src/governance/xx.rs:45` | {description} |
| 2 | `tests/e2e.rs` | Add integration test for {case} |

<details>
<summary>📋 Detailed Implementation Steps</summary>

{detailed steps from artifact}

</details>

---

### Validation

```bash
cargo check --workspace && cargo clippy --workspace -- -D warnings && cargo test --test e2e {pattern}
````

---

### Next Step

To implement: `/prp-issue-fix {number}`

---

_Investigated by Claude • {timestamp}_
EOF
)"

````

**PHASE_6_CHECKPOINT:**
- [ ] Comment posted to GitHub (if GH issue)
- [ ] Formatting renders correctly

---

## Phase 7: REPORT - Output to User

```markdown
## Investigation Complete

**Issue**: #{number} - {title}
**Type**: {BUG|ENHANCEMENT|REFACTOR|...}

### Assessment

| Metric | Value | Reasoning |
|--------|-------|-----------|
| {Severity or Priority} | {value} | {why - based on investigation} |
| Complexity | {LOW\|MEDIUM\|HIGH} | {why - based on files/integration/risk} |
| Confidence | {HIGH\|MEDIUM\|LOW} | {why - based on evidence/unknowns} |

### Key Findings

- **Root Cause**: {one-line summary}
- **Files Affected**: {count} files
- **Estimated Changes**: {brief scope}

### Files to Modify

| File | Action |
|---|---|
| `crates/api/api/src/governance/xx.rs` | UPDATE |
| `tests/e2e.rs` | {CREATE or UPDATE} |

### Artifact

`.claude/PRPs/issues/issue-{number}.md`

### GitHub

{Posted to issue | Skipped (free-form input)}

### Next Step

Run `/prp-issue-fix {number}` to execute the plan.
````

---

## Handling Edge Cases

### Issue is already closed

- Report: "Issue #{number} is already closed"
- Still create artifact if user wants analysis

### Issue already has linked PR

- Warn: "PR #{pr} already addresses this issue"
- Ask if user wants to continue anyway

### Can't determine root cause

- Document what you found
- Set confidence to LOW
- Note uncertainty in artifact
- Proceed with best hypothesis

### Very large scope

- Suggest breaking into smaller issues
- Focus on core problem first
- Note deferred items in "Out of Scope"

---

## Success Criteria

- **ARTIFACT_COMPLETE**: All sections filled with specific, actionable content
- **EVIDENCE_BASED**: Every claim has file:line reference or proof
- **IMPLEMENTABLE**: Another agent can execute without questions
- **GITHUB_POSTED**: Comment visible on issue (if GH issue)
- **COMMITTED**: Artifact saved in git
