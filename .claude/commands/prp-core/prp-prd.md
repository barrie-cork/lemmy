---
description: Generate a sub-PRD for a Brehon v0 feature or deviation that needs its own design document beyond what the numbered design docs already cover
argument-hint: [feature/problem description] (blank = start with questions)
---

# Brehon Sub-PRD Generator

**Input**: $ARGUMENTS

---

## When to Use This

**The Brehon project already has a frozen design suite** under `docs/brehon-law-inspired-network/`:

- `00-README.md` through `07-operations-and-federation.md` — stable
- `99-decisions-and-open-questions.md` — 15 ADRs + 12 open questions
- `IMPLEMENTATION-PLAN-v0.md` — the phase blueprint

**For 95% of v0 work you do NOT need a new PRD.** You need `/prp-plan <phase ref>` to turn an existing design-doc section into an executable plan.

**Use this command only when:**

1. A new capability is being added that isn't covered by any of the numbered docs (rare for v0 — most v0 work is already specified)
2. A blocking open question ([99 OQ-xxx](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)) needs to be resolved with a small design document before coding can proceed
3. A deviation from the plan is large enough to need its own hypothesis + success metric before committing to a `/prp-plan`
4. The user explicitly asks for a PRD

**For everything else, stop and suggest `/prp-plan` instead.**

---

## Your Role

You are a sharp product-minded engineer who:

- Starts with the **design-doc constraint**, not a blank page
- Never re-litigates committed ADRs
- Writes hypotheses tight enough that the next `/prp-plan` can execute them
- Acknowledges uncertainty honestly
- Flags every OQ this sub-PRD touches or depends on

**Anti-pattern**: do not fill sections with fluff. If info is missing, write "TBD — needs resolution on [99 OQ-xxx]" rather than inventing plausible-sounding requirements.

---

## Brehon Context (read every invocation)

- `docs/brehon-law-inspired-network/01-vision-and-principles.md` — the 9 principles and non-goals (hard)
- `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` — v0 scope and v1/v2/v3 staging
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — 15 committed ADRs, 12 open questions
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` — what's already been decomposed to tasks

**Hard constraints**: if this sub-PRD appears to contradict any ADR, STOP. Surface the contradiction, do not silently fix.

---

## Process Overview

```
SCOPE CHECK → FOUNDATION QUESTIONS → ADR/OQ CHECK → TECHNICAL GROUNDING → DECISIONS → GENERATE
```

---

## Phase 1: SCOPE CHECK — Should This Be A Sub-PRD?

**If no input provided**, ask:

> **What problem or capability do you want to document?**
> Be specific — one or two sentences.

**If input provided**, restate:

> I understand you want to document: {restated understanding}
> Is this correct?

Then run this **triage test** before going further:

1. **Is this already in one of the numbered design docs?** If yes, point the user there and stop.
2. **Is this already a task in [IMPLEMENTATION-PLAN-v0.md §3](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)?** If yes, suggest `/prp-plan` instead and stop.
3. **Does this belong to v1/v2/v3 per [ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)?** If yes, explicitly note "deferred to v{N}" and ask whether the user still wants a sub-PRD now (usually no).
4. **Does this contradict a committed ADR?** If yes, stop and surface the contradiction — it's an ADR-supersession question, not a PRD question.
5. **Is this genuinely v0-scope and undocumented?** Proceed.

**GATE**: Wait for user confirmation before continuing.

---

## Phase 2: FOUNDATION — Problem Discovery

Ask these questions (present all at once, user can answer together):

> **Foundation Questions:**
>
> 1. **Who** feels this problem first — which actor in [02 §2](docs/brehon-law-inspired-network/02-domain-model.md) (Visitor / Provisional / Member / Trusted / Juror Eligible / instance admin)?
>
> 2. **What** is the observable pain? Describe behaviour, not assumed need.
>
> 3. **Why can't the existing 11 MVP endpoints / already-planned phases cover it?**
>
> 4. **Why now (for v0)?** What makes this worth building before v1 instead of deferring?
>
> 5. **How will you know it's working?** Something verifiable from the integration test suite — prefer `cargo test --test e2e` outcomes over vague metrics.

**GATE**: Wait for responses.

---

## Phase 3: ADR / OQ CHECK — Constraint Validation

Based on the foundation answers:

### 3.1 List every ADR in [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) that governs this area

Example: a sub-PRD touching jury selection is governed by ADR-005 (reputation), ADR-007 (simplified jury parameters), and possibly OQ-001 (per-community scope), OQ-004 (concurrent assignments).

### 3.2 List every blocking OQ

If any OQ must be resolved before this sub-PRD can be executed, STOP and tell the user:

> **This sub-PRD depends on an unresolved open question:**
>
> - [99 OQ-xxx](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): {question}
> - Current lean: {from the OQ}
>
> We need a decision on this before the PRD can be finalised. Either:
> 1. Resolve the OQ now (and update [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) in the homeserver repo as a separate change)
> 2. Accept the OQ's current lean as a working assumption, and document it as a risk

### 3.3 Contradiction check

If the foundation answers imply contradicting a committed ADR, STOP:

> **Contradiction detected with [99 ADR-xxx](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md):**
>
> - ADR says: {summary}
> - Your framing implies: {summary}
>
> ADRs are append-only. To change direction we need to write a new ADR that supersedes ADR-xxx, not a sub-PRD that ignores it. Do you want to draft the superseding ADR first?

---

## Phase 4: TECHNICAL GROUNDING — Codebase & Design Feasibility

Launch up to 2 `Explore` agents in parallel via `subagent_type="Explore"`:

### Agent 1 — existing Lemmy patterns for the area

```
Explore brehon-fork (Lemmy 1.0-beta workspace at
C:\Users\barri\Developer\brehon-fork\) for anything relevant to: {area}.

LOCATE (return file:line refs and Rust snippets):
1. Existing Lemmy patterns we could mirror
2. Crate boundaries — which crate(s) this would live in
3. Any existing governance work already in crates/**/governance/
4. Dependencies in Cargo.toml that would need to be added

Return ACTUAL snippets. No invented examples.
```

### Agent 2 — design-doc alignment

```
Read these Brehon design docs and report how {area} fits:

- docs/brehon-law-inspired-network/04-data-model-and-api.md
- docs/brehon-law-inspired-network/03-architecture.md (§7 crate layout, §4 plane separation)
- docs/brehon-law-inspired-network/06-security-and-threat-model.md (if security-relevant)

For each relevant section, quote the exact paragraph and cite the doc + section.
Flag any place where the new capability would need new tables, new enum variants,
or new DTOs not currently in [04].
```

**Summarise:**

> **Technical Context:**
> - Feasibility: {HIGH/MEDIUM/LOW} because {reason tied to an existing Lemmy pattern or a missing one}
> - Existing patterns to leverage: {list with file:line}
> - New [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) extensions needed: {list, or "none"}
> - Crate(s) affected: {list}
> - Key technical risk: {main concern}

**GATE**: Brief pause for user input.

---

## Phase 5: DECISIONS — Scope & Approach

Ask:

> **Scope & Approach:**
>
> 1. **Minimal Build**: What's the absolute minimum to validate the hypothesis within v0?
>
> 2. **In scope vs deferred**: What 1–2 capabilities MUST land now? What defers to v1?
>
> 3. **Key Hypothesis**: "We believe {capability} will {achieve outcome} for {actor}. We'll know we're right when {integration test passes / [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) contract is satisfied / measurable signal}."
>
> 4. **Out of Scope**: What we are explicitly NOT building (even if tempting)?
>
> 5. **Blocking dependencies**: What must land first ([IMPLEMENTATION-PLAN-v0.md §3](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) phases, [99 OQs](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))?

**GATE**: Wait for responses.

---

## Phase 6: GENERATE — Write Sub-PRD

**Output path**: `.claude/PRPs/prds/{kebab-case-name}.prd.md`

```bash
mkdir -p .claude/PRPs/prds
```

### Sub-PRD Template

```markdown
# Sub-PRD: {Name}

**Scope**: v0 sub-feature — supplements the numbered design docs. Does NOT replace any ADR.
**Created**: {ISO timestamp}
**Status**: DRAFT

---

## Problem Statement

{2–3 sentences. Who, what pain, why now. Reference the design docs rather than re-explaining the world.}

**Actor(s) affected** ([02 §2](docs/brehon-law-inspired-network/02-domain-model.md)): {Visitor / Provisional / Member / Trusted / Juror Eligible / instance admin}

---

## Evidence

- {User quote, incident, or observation — or "Assumption, needs validation via X"}

---

## ADRs That Govern This

| ADR | Summary | How it constrains us |
|---|---|---|
| [ADR-NNN](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | {short} | {what it forces us to do or avoid} |

**Contradiction check**: {"None found" or "See §Risks"}

---

## Open Questions This Touches

| OQ | Status | Impact on this sub-PRD |
|---|---|---|
| [OQ-NNN](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | {blocking/non-blocking/resolved} | {what we assume or need} |

---

## Proposed Solution

{One paragraph. Which Lemmy patterns we mirror; which crate(s) we touch; which [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) section this extends.}

---

## Key Hypothesis

> We believe {capability} will {achieve outcome} for {actor}.
> We'll know we're right when {integration test assertion or [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) contract}.

---

## What We're NOT Building

- {Out of scope 1 — why}
- {Out of scope 2 — why}
- **Anything deferred to v1/v2/v3 per [ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)**

---

## Success Criteria (verifiable)

| Criterion | How Verified |
|---|---|
| {Criterion} | `cargo test --test e2e {name}` passes |
| {Criterion} | `diesel migration redo` round-trips cleanly |
| {Criterion} | No new `cargo clippy -- -D warnings` failures |

---

## Cross-Cutting Impact ([IMPLEMENTATION-PLAN-v0.md §4](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md))

- [ ] Hash-chain governance log touched? {yes/no — details}
- [ ] `actor_pseudonym` table or redaction service touched? {yes/no}
- [ ] `CaseStatus::EmergencyRemove` affected? {yes/no}
- [ ] AGPLv3 notice / source disclosure affected? {usually no}

---

## Users & Context

**Primary actor**: {from [02 §2](docs/brehon-law-inspired-network/02-domain-model.md) — be specific}
- **Current behaviour**: {what happens today}
- **Trigger**: {what moment makes the need appear}
- **Success state**: {what "done" looks like}

**Non-actors**: {who this is NOT for}

---

## Technical Approach

**Feasibility**: {HIGH / MEDIUM / LOW} — {one-sentence reason tied to Lemmy patterns or missing primitives}

**Crate(s) affected**:
- `crates/db_schema/...`
- `crates/api/...`
- `crates/apub/...` (if federation-relevant)

**Architecture fit** ([03 §4](docs/brehon-law-inspired-network/03-architecture.md)): {how this respects plane separation}

**New dependencies** (if any):
- {crate} = "{version}" — {why}

**Technical risks**:

| Risk | Likelihood | Mitigation |
|---|---|---|
| {risk} | {L/M/H} | {mitigation} |

---

## Implementation Phases (for follow-up `/prp-plan` runs)

<!--
  STATUS: pending | in-progress | complete
  PRP: link to generated plan file once /prp-plan runs on this
-->

| # | Phase | Description | Status | Depends | PRP Plan |
|---|---|---|---|---|---|
| 1 | {Phase name} | {deliverable} | pending | - | - |
| 2 | {Phase name} | {deliverable} | pending | 1 | - |

### Phase Details

**Phase 1: {Name}**
- **Goal**: {what}
- **Scope**: {deliverables}
- **Success signal**: {test or cargo check}

---

## Decisions Log

| Decision | Choice | Alternatives | Rationale |
|---|---|---|---|
| {Decision} | {Choice} | {Options} | {Why this one, citing Lemmy patterns or ADRs} |

---

## Research Summary

**Codebase findings** (from `Explore` agents):
- {file:line} — {pattern}
- {file:line} — {pattern}

**Design-doc alignment**:
- {doc + section} — {what it requires}

---

*Generated: {timestamp}*
*Status: DRAFT — review before running `/prp-plan`*
```

---

## Phase 7: OUTPUT — Summary

```markdown
## Sub-PRD Created

**File**: `.claude/PRPs/prds/{name}.prd.md`

### Summary

**Problem**: {one line}
**Actor**: {from [02 §2](docs/brehon-law-inspired-network/02-domain-model.md)}
**Hypothesis**: {one line}

### ADRs Governing This

- {ADR-NNN}
- {ADR-MMM}

### Blocking Open Questions

- {OQ-NNN — state} (or "None")

### Cross-Cutting Impact

- {Hash chain / pseudonyms / EmergencyRemove / AGPL — which apply, or "None"}

### Validation Status

| Section | Status |
|---|---|
| Problem statement | {Validated / Assumption} |
| ADR alignment | {Clean / Needs ADR supersession} |
| Technical feasibility | {HIGH / MEDIUM / LOW} |
| Success criteria | {Verifiable / Needs tightening} |

### Recommended Next Step

{One of: resolve OQ-NNN, draft superseding ADR, run `/prp-plan`, user review}

### To Execute

Run: `/prp-plan .claude/PRPs/prds/{name}.prd.md`

This will pick the first pending phase and produce an implementation plan.
```

---

## Success Criteria

- **DESIGN_DOC_AWARE**: Sub-PRD references the numbered docs, does not repeat them
- **ADR_COMPLIANT**: No contradictions with the 15 ADRs, or superseding-ADR path flagged
- **OQ_TRIAGED**: Every touched open question is listed with impact
- **HYPOTHESIS_TESTABLE**: Success criteria are verifiable by `cargo test --test e2e` or similar
- **V0_BOUNDED**: No v1/v2/v3 scope leaks into "must build"
- **CROSS_CUTTING_ACKED**: Hash chain / pseudonyms / `EmergencyRemove` touchpoints identified
- **NEXT_STEP_CLEAR**: User knows exactly what happens after the sub-PRD
</content>
</invoke>