---
description: |
  Deep root cause analysis for Brehon Rust bugs — finds the actual cause, not just symptoms
argument-hint: |
  <issue|error|stacktrace> [--quick]
---

# Root Cause Analysis (Brehon)

**Input**: $ARGUMENTS

---

## Your Mission

Find the **actual root cause** — the specific Rust code, SQL, config, or logic that, if changed, would prevent this issue. Not symptoms. Not intermediate failures. The origin.

**The Test**: "If I changed THIS, would the issue be prevented?" If the answer is "maybe" or "partially", you haven't found the root cause yet. Keep digging.

**Brehon-specific first suspects** (check these BEFORE opening the 5 Whys if the symptom matches):

- **Hash-chain integrity error** → check Postgres trigger on `governance_log`, check the signature-update path ([IMPLEMENTATION-PLAN-v0.md §4.1](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md))
- **`actor_pseudonym` missing / reused** → check `actor_pseudonym::get_or_create` — ADR-015 requires regeneration after GDPR delete
- **`non-exhaustive patterns: CaseStatus::EmergencyRemove`** → someone used `_ =>` on a `CaseStatus` match; forbidden by ADR-013
- **Redaction bypass** (username visible in public log) → someone wrote to `public_case_log` without passing through `redaction::scrub` ([ADR-015](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))
- **Remote sanction auto-applied** → someone bypassed the advisory-only rule ([ADR-006](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))
- **Upstream Lemmy rebase broke things** → check `git log upstream/main..HEAD` — a recent rebase can break Diesel schema assumptions

Brehon env vars worth setting when reproducing:

```bash
export RUST_BACKTRACE=1
export RUST_LOG=debug,lemmy_api=trace,lemmy_db_schema=debug
```

---

## Phase 1: CLASSIFY - Parse Input

### 1.1 Determine Input Type

| Type | Description | Action |
|------|-------------|--------|
| Raw symptom | Vague description, error message, stack trace | INVESTIGATE - form hypotheses, test them |
| Pre-diagnosed | Already identifies location/problem | VALIDATE - confirm diagnosis, check for related issues |

### 1.2 Determine Mode

- `--quick` flag present → Surface scan (2-3 Whys, ~5 min)
- No flag → Deep analysis (full 5 Whys, git history required)

### 1.3 Parse the Input

- Stack trace → extract error type, message, call chain
- Error message → identify system, error code, context
- Vague description → identify what's actually being claimed

**Restate the symptom in one sentence. What is actually failing?**

**PHASE_1_CHECKPOINT:**
- [ ] Input type classified
- [ ] Mode determined (quick/deep)
- [ ] Symptom restated clearly

---

## Phase 2: HYPOTHESIZE - Form Theories

### 2.1 Generate Hypotheses

Based on the symptom, generate 2-4 hypotheses. For each:

| Hypothesis | What must be true | Evidence needed | Likelihood |
|------------|-------------------|-----------------|------------|
| {H1} | {conditions} | {proof needed} | HIGH/MED/LOW |
| {H2} | {conditions} | {proof needed} | HIGH/MED/LOW |

### 2.2 Rank and Select

Start with the most probable hypothesis.

**PHASE_2_CHECKPOINT:**
- [ ] 2-4 hypotheses generated
- [ ] Ranked by likelihood
- [ ] Leading hypothesis selected

---

## Phase 3: INVESTIGATE - The 5 Whys

Execute the 5 Whys protocol for your leading hypothesis:

```
WHY 1: Why does [symptom] occur?
→ Because [intermediate cause A]
→ Evidence: [code reference, log, or test that proves this]

WHY 2: Why does [intermediate cause A] happen?
→ Because [intermediate cause B]
→ Evidence: [proof]

WHY 3: Why does [intermediate cause B] happen?
→ Because [intermediate cause C]
→ Evidence: [proof]

WHY 4: Why does [intermediate cause C] happen?
→ Because [intermediate cause D]
→ Evidence: [proof]

WHY 5: Why does [intermediate cause D] happen?
→ Because [ROOT CAUSE]
→ Evidence: [exact file:line reference]
```

### Evidence Standards (STRICT)

| Valid Evidence | Invalid Evidence |
|----------------|------------------|
| `crates/api/api/src/governance/case.rs:123` with actual code snippet | "likely includes...", "probably because..." |
| Command output you actually ran | Logical deduction without code proof |
| Test you executed that proves behavior | Explaining how technology works in general |

**Rules:**
- Stop when you hit code you can change
- Every "because" MUST have evidence
- If evidence refutes a hypothesis, pivot to the next one
- If you hit a dead end, backtrack and try alternative branches

### Investigation Techniques

**For tracing complex code paths**, use the built-in `Explore` subagent (via the `Agent` tool with `subagent_type="Explore"`) to understand how the suspected code works before diving into the 5 Whys:

```
Explore the Rust code around: [suspected area / error location] in brehon-fork
(Lemmy 1.0-beta workspace at C:\Users\barri\Developer\brehon-fork\).

TRACE:
1. How data flows through the affected code path (which crates, which handlers)
2. Entry points — which route handler reaches this code
3. State changes and side effects along the way (DB writes, governance_log appends)
4. Contracts between components (LemmyError propagation, transaction boundaries)
5. Cross-cutting touches: does this touch governance_log, actor_pseudonym,
   redaction, or CaseStatus::EmergencyRemove?

Return file:line references and actual Rust snippets. No suggestions.
```

**For Rust code issues:**
- Grep for error messages, function names, Diesel queries: `grep -rn 'pattern' crates/`
- Read full context around suspicious code
- `git log --oneline -10 -- <file>` for recent changes
- `git blame -L <start>,<end> <file>` for authorship
- **Run the suspicious code path** with an isolated `#[test]` or via `cargo test --test e2e <pattern>` with a constructed fixture

**For runtime issues:**
- `RUST_BACKTRACE=full cargo test ...` — full backtrace on panic
- `RUST_LOG=trace cargo run` — trace logs across crates
- Check `.env` vs docker-compose env for drift
- Look for initialization order dependencies (especially background jobs in `crates/server/src/governance.rs`)
- Postgres logs — `docker logs <postgres-container>` — often tell you what a Diesel error actually meant
- Check if `diesel migration run` was skipped in the test setup

**For "it worked before" issues:**
```bash
git log --oneline -20
git log --oneline upstream/main..HEAD  # what's ours vs upstream
git diff HEAD~10 -- <suspicious files>
```

**For Diesel-specific issues:**
- `trait bound not satisfied` — mismatch between `schema.rs` and `#[diesel(table_name = ...)]`
- `cannot find macro table!` — `schema.rs` is out of date; regen via `diesel print-schema`
- Nullable vs non-nullable column mismatch — check `Option<T>` wrapping on the struct field

**PHASE_3_CHECKPOINT:**
- [ ] 5 Whys executed (or 2-3 for quick mode)
- [ ] Each step has concrete evidence
- [ ] Root cause identified with file:line reference

---

## Phase 4: VALIDATE - Confirm Root Cause

### 4.1 Three Tests

| Test | Question | Pass? |
|------|----------|-------|
| Causation | Does root cause logically lead to symptom through evidence chain? | Y/N |
| Necessity | If root cause didn't exist, would symptom still occur? | N required |
| Sufficiency | Is root cause alone enough, or are there co-factors? | Document if co-factors |

If any test fails → root cause is incomplete. Go deeper or broader.

### 4.2 Git History (Deep Mode Required)

```bash
git log --oneline -10 -- [affected files]
git blame [affected file] | grep -A2 -B2 [line number]
```

**Document:**
- When was the problematic code introduced?
- What commit/PR added it?
- Has it changed recently or been stable?

### 4.3 Rule Out Alternatives

For deep mode, document why other hypotheses were rejected:

| Hypothesis | Why Ruled Out |
|------------|---------------|
| {H2} | {evidence that disproved it} |
| {H3} | {evidence that disproved it} |

**PHASE_4_CHECKPOINT:**
- [ ] All three tests pass
- [ ] Git history documented (deep mode)
- [ ] Alternative hypotheses ruled out (deep mode)

---

## Phase 5: REPORT - Generate Output

### 5.1 Create Report Directory

```bash
mkdir -p .claude/PRPs/debug
```

### 5.2 Generate Report

**Path**: `.claude/PRPs/debug/rca-{issue-slug}.md`

```markdown
# Root Cause Analysis

**Issue**: {One-line symptom description}
**Root Cause**: {One-line actual cause}
**Severity**: {Critical/High/Medium/Low}
**Confidence**: {High/Medium/Low}

---

## Evidence Chain

WHY: {Symptom occurs}
↓ BECAUSE: {First level cause}
  Evidence: `crates/api/api/src/governance/case.rs:123` - {code snippet}

WHY: {First level cause}
↓ BECAUSE: {Second level cause}
  Evidence: `crates/db_schema/src/source/governance/log.rs:456` - {code snippet}

{...continue...}

↓ ROOT CAUSE: {The fixable thing}
  Evidence: `crates/api/api/src/governance/case.rs:789` - {problematic code}

---

## Git History

- **Introduced**: {commit hash} - {message} - {date}
- **Author**: {who}
- **Recent changes**: {yes/no, when}
- **Type**: {regression / original bug / long-standing}

---

## Fix Specification

### What Needs to Change

{Which files, what logic, what the correct behavior should be}

### Implementation Guidance

```rust
// Current (problematic):
{simplified Rust snippet}

// Required (fixed):
{simplified Rust snippet}
```

### Files to Modify

- `crates/<crate>/src/<path>.rs:LINE` - {why}

### Verification

1. {Test to run}
2. {Expected outcome}
3. {How to reproduce original issue}
```

**PHASE_5_CHECKPOINT:**
- [ ] Report created
- [ ] All sections filled
- [ ] Fix specification is actionable

---

## Phase 6: OUTPUT - Report to User

```markdown
## Root Cause Analysis Complete

**Issue**: {symptom}
**Root Cause**: {cause}
**Confidence**: {High/Medium/Low}

**Report**: `.claude/PRPs/debug/rca-{issue-slug}.md`

### Summary

{2-3 sentence explanation of what was found}

### The Fix

{1-2 sentence description of what needs to change}

### Next Steps

- Review the report for full evidence chain
- Implement the fix following the specification
- Run verification steps to confirm resolution
```

---

## Critical Reminders

1. **Symptoms lie.** The error message tells you what failed, not why.

2. **First explanation is often wrong.** Resist the urge to stop early.

3. **No evidence = no claim.** "Likely", "probably", "may" are not allowed.

4. **Test, don't just read.** Execution proves behavior; reading proves intent.

5. **Git history is mandatory.** In deep mode, you must include when/who/why.

6. **The fix should be obvious.** If your root cause is correct, the fix writes itself.

---

## Success Criteria

- **ROOT_CAUSE_FOUND**: Specific file:line identified
- **EVIDENCE_CHAIN_COMPLETE**: Every step has proof
- **FIX_ACTIONABLE**: Someone could implement from the report
- **VERIFICATION_CLEAR**: How to confirm fix works
