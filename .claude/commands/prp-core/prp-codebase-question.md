---
description: Research Brehon codebase questions using parallel Explore agents — documents what exists, not what should change
argument-hint: <question or topic> [--web] [--follow-up]
---

# Codebase Research (Brehon)

**Input**: $ARGUMENTS

---

## Your Mission

Answer codebase questions about the Brehon fork (Lemmy 1.0-beta + governance extensions) thoroughly by spawning parallel `Explore` agents, synthesizing their findings, and producing a research document.

**Core Philosophy**: Document what IS, not what SHOULD BE. You are a technical cartographer of the Rust workspace.

**Golden Rule**: Every claim must have a `file:line` reference to an actual file in `crates/`, `migrations/`, `api_tests/`, or `tests/`. No speculation, no suggestions, no critique.

---

## Brehon Context

**The fork lives at `C:\Users\barri\Developer\brehon-fork\` (Lemmy 1.0-beta, working branch `governance-v0`).** The authoritative design docs live in a sibling repo on the same machine, so reference them via absolute Windows paths when research touches governance primitives:

- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — tables, enums, Diesel models, DTOs, routes (LIVING; current v0+v1 schema from live code)
- `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` — 11-endpoint v0 scope
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — 15 ADRs
- `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` — phase blueprint
- Fork-local `CLAUDE.md` and `AGPL-NOTICE.md` at the repo root

Research mode is **description, not design** — if the question happens to touch a committed ADR, quote the ADR verbatim rather than paraphrasing or suggesting alternatives.

---

## CRITICAL: Documentarian Only

- **DO NOT** suggest improvements or changes
- **DO NOT** perform root cause analysis unless explicitly asked
- **DO NOT** propose future enhancements
- **DO NOT** critique implementations or identify problems
- **DO NOT** recommend refactoring or optimization
- **ONLY** describe what exists, where it exists, how it works, and how components interact

---

## Phase 1: PARSE - Understand the Query

### 1.1 Read Mentioned Files

If the user mentions specific files, read them FULLY first (no limit/offset) before any decomposition.

### 1.2 Classify the Query

| Type | Indicators | Explore Agent Brief |
|------|-----------|---------------------|
| **Where** | "where is", "find", "locate" | LOCATE-style brief (find files, extract patterns) |
| **How** | "how does", "trace", "flow" | TRACE-style brief (data flow, integration points) |
| **What** | "what is", "explain", "describe" | Two Explore agents in parallel (one LOCATE, one TRACE) |
| **Pattern** | "how do we", "convention", "examples" | LOCATE-style brief |
| **External** | "docs", "best practice", "API" | Add a WEB-research brief (the built-in `Explore` agent can do `WebFetch` / `WebSearch` as well) |

### 1.3 Determine Scope

- Identify specific components, patterns, or concepts to investigate
- Note any `--web` flag for external research
- Note any `--follow-up` flag for appending to existing research

**PHASE_1_CHECKPOINT:**
- [ ] Mentioned files read in full
- [ ] Query type classified
- [ ] Research scope identified
- [ ] Flags parsed (--web, --follow-up)

---

## Phase 2: DECOMPOSE - Break into Research Areas

### 2.1 Create Research Plan

Break the query into 2-5 composable research areas:

```
RESEARCH QUESTION: {user's question}

AREAS:
1. {Area} → Agent: {which agent}
2. {Area} → Agent: {which agent}
3. {Area} → Agent: {which agent}
```

### 2.2 Agent Selection

All research runs through the **built-in `Explore` subagent** (invoked via the `Agent` tool with `subagent_type="Explore"`). The brief you give it determines its behavior:

| Brief type | Use When |
|------------|----------|
| **LOCATE brief** | Finding WHERE code lives, locating files, extracting patterns, discovering conventions |
| **TRACE brief** | Understanding HOW code works, tracing data flow, mapping integration points, following call chains |
| **WEB brief** | `--web` flag is set or the question explicitly asks for external Rust/Diesel/Lemmy/ActivityPub docs |

**Strategy:**
1. Start with a LOCATE Explore to find what exists in `crates/`
2. Then use a TRACE Explore on the most relevant findings to understand how they work
3. Run up to 3 Explore agents in parallel when they're searching different areas — launch them in a single message with multiple `Agent` tool calls

**PHASE_2_CHECKPOINT:**
- [ ] Query decomposed into 2-5 research areas
- [ ] Agent assigned to each area
- [ ] Parallel vs sequential execution planned

---

## Phase 3: EXPLORE - Spawn Parallel Explore Agents

### 3.1 Launch Codebase Briefs

**Launch Explore agents in parallel using multiple `Agent` tool calls in a single message.** Use `subagent_type="Explore"` for each one.

For each research area, compose the brief from the template below. The Brehon fork is a Rust workspace — briefs should point the agent at `crates/`, `migrations/`, `api_tests/`, and `tests/`, NOT at `src/`, `package.json`, or `pyproject.toml`.

**LOCATE brief template:**

```
Find all Rust code relevant to: {research area} in the Brehon fork at
C:\Users\barri\Developer\brehon-fork\.

LOCATE:
1. {Specific files/modules/types to find} — check crates/db_schema, crates/db_views,
   crates/api/api_common, crates/api/api, crates/api/api_crud, crates/api/routes,
   crates/apub/{objects,activities,apub}, crates/server
2. {Rust patterns or conventions to extract — derive macros, trait impls, Diesel
   schema definitions, AP type mappings}
3. {Related test files} — check api_tests/ and tests/ (governance e2e)
4. {Related SQL} — check migrations/ for Diesel up/down pairs

Categorize findings by crate. Return ACTUAL Rust snippets with precise file:line references.
Thoroughness: medium (balance breadth vs context).

Remember: Document what exists, no suggestions or improvements. Do NOT look for
src/, package.json, pyproject.toml — this is a Rust-only fork.
```

**TRACE brief template:**

```
Analyze the implementation of: {research area} in the Brehon fork at
C:\Users\barri\Developer\brehon-fork\.

TRACE:
1. {Data flow to trace} — follow from route handler → api crate → db_views/db_schema →
   migration. If it crosses the federation boundary, also follow into crates/apub/.
2. {Integration points to document} — LemmyContext wiring in crates/server, Diesel
   connection pool usage, background job registration
3. {Contracts between components} — DTO types in crates/api/api_common, LemmyError
   propagation, transaction boundaries
4. Cross-cutting: does this path touch `governance_log::append`, `actor_pseudonym`,
   `redaction::scrub`, or `CaseStatus::EmergencyRemove`? These are Brehon-specific
   invariants and should be called out if present.

Document what exists with precise file:line references. No suggestions.
Thoroughness: medium.
```

### 3.2 Launch Web Research Brief (if --web or explicitly requested)

**WEB brief template:**

```
Research external documentation for: {topic}. This is for the Brehon fork, a Rust
governance extension to Lemmy 1.0-beta. Context matters: prefer authoritative sources
from Lemmy, Diesel, actix-web, ActivityPub, or the referenced Rust crates over
tutorials or blog posts.

FIND:
1. {Specific documentation needed} — docs.rs, official guides, RFCs
2. {API references or patterns} — version-pin to crate versions used in Cargo.toml
   (e.g. diesel 2.x, extism 1.20, webauthn-rs if relevant) to avoid stale docs

Return findings with direct links, citations, and crate-version context. Use WebFetch
or WebSearch as needed.
```

### 3.3 Wait for All Agents

**IMPORTANT**: Wait for ALL agents to complete before proceeding.

**PHASE_3_CHECKPOINT:**
- [ ] All agents launched (parallel where possible)
- [ ] All agents completed
- [ ] Results collected from each agent

---

## Phase 4: SYNTHESIZE - Merge Findings

### 4.1 Compile Results

- Prioritize live codebase findings as primary source of truth
- Connect findings across different components
- Include specific `file:line` references throughout
- Document patterns, connections, and architectural decisions as they exist

### 4.2 Answer the Question

Map findings back to the user's original question:

| Question Aspect | Finding | Evidence |
|----------------|---------|----------|
| {aspect 1} | {what was found} | `crates/api/api/src/governance/case.rs:123` |
| {aspect 2} | {what was found} | `crates/db_schema/src/source/governance/log.rs:456` |

### 4.3 Identify Gaps

Note any areas that couldn't be fully documented:

- {Area that needs further investigation}
- {Question that remains open}

**PHASE_4_CHECKPOINT:**
- [ ] All agent results synthesized
- [ ] Findings connected across components
- [ ] Original question answered with evidence
- [ ] Gaps identified

---

## Phase 5: DOCUMENT - Generate Research File

### 5.1 Gather Metadata

```bash
date -u +"%Y-%m-%dT%H:%M:%SZ"
git rev-parse --short HEAD
git branch --show-current
basename $(git rev-parse --show-toplevel)
```

### 5.2 Create Research Directory

```bash
mkdir -p .claude/PRPs/research
```

### 5.3 Determine Filename

**If --follow-up**: Append to existing research file instead of creating new one.

**If new research**:

**Path**: `.claude/PRPs/research/{YYYY-MM-DD}-{kebab-case-topic}.md`

Examples:
- `2025-01-08-authentication-flow.md`
- `2025-01-15-database-migration-patterns.md`

### 5.4 Write Research Document

```markdown
---
date: {ISO timestamp with timezone}
git_commit: {short hash}
branch: {branch name}
repository: brehon-fork
upstream_base: Lemmy 1.0-beta @ 811d0d09c
topic: "{User's Question/Topic}"
tags: [research, codebase, {relevant-component-names}]
status: complete
last_updated: {YYYY-MM-DD}
---

# Research: {User's Question/Topic}

**Date**: {ISO timestamp}
**Git Commit**: {short hash}
**Branch**: {branch name} (typically `governance-v0` or a feature branch rebased onto it)
**Repository**: brehon-fork (fork of LemmyNet/lemmy)

## Research Question

{Original user query}

## Summary

{High-level documentation of what was found, answering the question by describing what exists}

## Detailed Findings

### {Component/Area 1}

- Description of what exists (`crates/api/api/src/governance/case.rs:123`)
- How it connects to other crates
- Current implementation details (Diesel schema, route registration, etc.)

### {Component/Area 2}

...

## Code References

| File | Lines | Description |
|------|-------|-------------|
| `crates/db_schema/src/source/governance/log.rs` | 12-45 | {What's there} |
| `crates/api/api/src/governance/case.rs` | 78-120 | {What's there} |
| `migrations/{timestamp}_{name}/up.sql` | 1-30 | {What the migration does} |

## Architecture Documentation

{Current patterns, conventions, and design implementations found. If the findings
touch an ADR from 99-decisions-and-open-questions.md, quote the ADR number and
summarise its constraint.}

## ADR Touchpoints

{List any ADRs from docs/brehon-law-inspired-network/99-decisions-and-open-questions.md that the researched code depends on or enforces, e.g. ADR-008 (hash chain), ADR-013 (EmergencyRemove), ADR-015 (pseudonyms + GDPR)}

## Open Questions

- {Areas that need further investigation}
```

### 5.5 Add GitHub Permalinks (if applicable)

```bash
# Check if on main or pushed
git branch --show-current
gh repo view --json owner,name -q '"\(.owner.login)/\(.name)"'
```

If on main/pushed, replace local file references with:
`https://github.com/{owner}/{repo}/blob/{commit}/{file}#L{line}`

### 5.6 Handle Follow-ups

If `--follow-up` flag and existing research file:

1. Read the existing research file
2. Update frontmatter: `last_updated` and add `last_updated_note`
3. Append new section: `## Follow-up Research {timestamp}`
4. Spawn new agents as needed
5. Save updated document

**PHASE_5_CHECKPOINT:**
- [ ] Metadata gathered
- [ ] Research file created (or existing file updated for follow-up)
- [ ] All sections filled with evidence-based content
- [ ] GitHub permalinks added (if applicable)
- [ ] No placeholder values remain

---

## Phase 6: OUTPUT - Present to User

```markdown
## Research Complete

**Question**: {original question}
**Document**: `.claude/PRPs/research/{filename}.md`

### Summary

{2-3 sentence answer to the question}

### Key Findings

- **{Finding 1}**: {brief} (`crates/api/api/src/governance/case.rs:123`)
- **{Finding 2}**: {brief} (`crates/db_schema/src/source/governance/log.rs:456`)
- **{Finding 3}**: {brief} (`migrations/{timestamp}_{name}/up.sql:12`)

### Architecture

{1-2 sentence description of relevant architecture}

### Open Questions

- {Any unanswered aspects}

### Follow-up

To dig deeper: `/prp-codebase-question --follow-up {topic}`
To include external docs: `/prp-codebase-question --web {topic}`
```

---

## Usage Examples

```bash
# Basic codebase question (Brehon-flavoured)
/prp-codebase-question how does Lemmy 1.0-beta register route handlers in crates/api/routes/

# Include external documentation
/prp-codebase-question --web how does diesel 2.x derive Selectable for joined queries

# Follow up on previous research
/prp-codebase-question --follow-up what transaction boundaries does the comment-create path establish

# Locate governance integration points
/prp-codebase-question where should governance_log::append hook into existing Lemmy post/comment handlers
```

---

## Critical Reminders

1. **Document, don't evaluate.** Describe what IS, never what SHOULD BE. If the question brushes against an ADR, quote the ADR verbatim rather than paraphrasing or proposing alternatives.

2. **Evidence required.** Every claim needs a `file:line` reference to an actual file in `crates/`, `migrations/`, `api_tests/`, or `tests/`.

3. **Explore agents are parallel.** Launch multiple `Agent(subagent_type="Explore", …)` calls in a single message when researching different areas.

4. **Wait for completion.** Never synthesize until ALL agents have returned.

5. **Read first.** If the user mentions files, read them fully (no limit/offset) before spawning agents.

6. **No placeholders.** Every field in the research document must have real values.

7. **Codebase is truth.** Live Rust code always overrides documentation or assumptions. If a finding disagrees with a Brehon design doc, note the discrepancy — the design doc may be stale or the code may have drifted.

8. **Rust-only.** This is a Rust workspace. Do NOT look for `src/`, `package.json`, `pyproject.toml`, TypeScript, or JavaScript.

---

## Success Criteria

- **QUESTION_ANSWERED**: User's question addressed with concrete evidence
- **AGENTS_USED**: Specialized agents spawned for each research area
- **EVIDENCE_COMPLETE**: Every finding has `file:line` references
- **DOCUMENT_CREATED**: Research file saved at `.claude/PRPs/research/`
- **NO_OPINIONS**: Document describes what exists, not what should change
- **PERMALINKS_ADDED**: GitHub links included when possible
