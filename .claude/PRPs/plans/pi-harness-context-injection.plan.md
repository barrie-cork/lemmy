# Plan — Pi harness context injection: progressive disclosure by Brehon role

**Status:** PLANNING — draft for review  
**Authored:** 2026-06-10  
**Trigger:** Audit of pi context injection in `lemmy-hooks.ts` `before_agent_start`  
**Prerequisite:** `.pi/extensions/lemmy-hooks.ts` — the enforcement + injection hub  
**Dependency on prior plan:** Independent of `pi-harness-alignment.plan.md` (Streams A-D).
  That plan fixed reload safety, compaction, and PMD access. This plan fixes the
  monolithic context dump in `before_agent_start`.

---

## 1. Summary

Progressive disclosure by Brehon role. When a user switches to `/brehon-mode planning`,
`/brehon-mode impl-task`, or `/brehon-mode bm`, the `before_agent_start` handler
currently injects the **same monolithic blob** (full PROJECT_CONTEXT.md, all 25 rule
files, pre-phase audit reminder, coordination state) regardless of role. This wastes
40-60% of the context window with irrelevant instructions and contradicts the
progressive-disclosure principle pi's Agent Skills standard was designed for.

This plan reworks the `before_agent_start` handler to:
1. **Slice PROJECT_CONTEXT.md by role** — each Brehon mode gets only the sections it needs.
2. **Auto-inject the matching `.pi/skills/{mode}/SKILL.md`** — switch-and-go role setup, no
   separate `/skill:planning` step needed.
3. **Filter the rule index** — list only the `.claude/rules/` files relevant to the
   current mode.
4. **Rewrite the three ported skills** (planning, impl-task, bm-task) to remove
   Claude-only tool/daemon references and add pi-native equivalents.
5. **Feed the harness-factory profile** into the injected context (persona + memory).

## 2. Source

- `.pi/extensions/lemmy-hooks.ts` — the single file to change (Tasks 1-4, 7)
- `.pi/harness-factory/profiles/*.json` — 7 profiles, to cross-reference for persona/memory (Task 3)
- `.pi/harness-factory/active.json` — current active profile (Task 3)
- `.pi/skills/planning/SKILL.md` — skill to rewrite (Task 5)
- `.pi/skills/impl-task/SKILL.md` — skill to rewrite (Task 5)
- `.pi/skills/bm-task/SKILL.md` — skill to rewrite (Task 5)
- `.pi/skills/code-reviewer/SKILL.md` — review skill (check for same Claude-only refs, Task 5)
- `.pi/skills/silent-failure-hunter/SKILL.md` — same (Task 5)
- `.pi/PROJECT_CONTEXT.md` — the context to slice (Task 2)
- `pi-harness-alignment.plan.md` — prior plan that fixed reload safety; this plan extends structure
- `docs/extensions.md` — pi's `before_agent_start` API reference
- `docs/skills.md` — pi's skill loading and Agent Skills standard

## 3. Problem statement

**Three distinct defects, all in `lemmy-hooks.ts`:**

### Defect 1 — Monolithic context dump in `before_agent_start`

The `before_agent_start` handler (lines ~118-150 of the file) injects into the
system prompt:

- **Entire PROJECT_CONTEXT.md** (~3 KB) — includes cargo wrapper tables, Rust error
  conventions, pi-specific tactics, PMD topology detail, subagent installation
  instructions. A planning session does not need the cargo table or `unwrap()` rules.
  A BM session does not need pi-specific tactics or PMD topology.
- **All 25 rule file references** — BM only needs ~5 of these; planning only needs ~4.
- **Pre-phase audit reminder** (large block) — on phase branches, this Windows cmd
  block is injected into EVERY mode, even planning and BM which don't run cargo.

**Cost:** Each Brehon role wastes at minimum 40% of injection context on
instructions the agent will never use. Over a long session, compacted summaries
retain this noise.

### Defect 2 — Skills exist but are never auto-loaded per mode

Pi discovers skills (scans directories, extracts name+description, puts them in
system prompt as XML). But **the agent must self-decide to `read` the SKILL.md**
or invoke `/skill:<name>`. The mode instructions in `before_agent_start` tell the
agent WHAT to do ("Read Brehon design docs, ADRs, PRDs") but not HOW — the HOW
lives in skills that never get auto-injected.

Switching to `/brehon-mode planning` should load the planning skill. Currently it just
changes a label and adds 3 bullet points.

### Defect 3 — Ported skills reference Claude-only infrastructure

All three Brehon roles were ported from `.claude/agents/` with a "Pi migration note"
header but their bodies contain:

- `Agent(subagent_type: "Explore")` — pi has no subagent nesting
- `mcp__ref-context__ref_search_documentation` — Claude MCP tool, doesn't exist in pi
- `LSP` tool — Claude Code built-in, pi doesn't have `goto_definition`
- Junior daemon dispatch conventions (DQ mid-task push, forbidden windows, etc.)
- Shape-G/ci-watcher validation delegation patterns

When pi loads these skills, the agent either hallucinates tool calls or interprets
half the instructions as no-ops.

## 4. Solution statement

Six work-streams, sequenced as tasks in §9. Tasks 1-4 are `lemmy-hooks.ts`
changes (the core). Task 5 is a skill rewrite. Tasks 6-7 are integration/cleanup.

### Stream A — Mode-aware context slices (Task 1)

Define a `MODE_CONTEXT` map in `lemmy-hooks.ts` that for each Brehon mode specifies:

- Which PROJECT_CONTEXT.md sections to include
- A role persona string (harvested from or matching the factory profile)
- Which `.claude/rules/` files to list
- The matching skill path to auto-inject
- Authorized `.claude/` read paths (to override AGENTS.md's blanket restriction for the role)

The `before_agent_start` handler uses this map instead of injecting the whole file.

### Stream B — Auto-inject role skill (Task 2)

In `before_agent_start`, when `brehonMode !== "main-safe"` and the mode has a
mapped skill in `.pi/skills/{mode}/SKILL.md`, **read the skill file and inject
its content** into the returned `systemPrompt`. This makes `/brehon-mode planning`
a one-command full role setup.

### Stream C — Rule index filter (Task 3)

Map each mode to a short list of relevant rules (2-6 files). In `before_agent_start`,
generate the "Available Project Rules" section using only the filtered list for the
current mode. `main-safe` keeps the existing unfiltered index.

### Stream D — Feed factory profile (Task 3)

In `before_agent_start`, read `.pi/harness-factory/active.json`, resolve the matching
profile from `.pi/harness-factory/profiles/`, and merge:
- Profile `persona` into system prompt persona line
- Profile `memory` items as additional instruction bullets
- Profile `workflow` settings propagate to agent (e.g. `runTestsAfterEdit`)

### Stream E — Rewrite skills for pi (Tasks 5 + 6)

For each of the three ported skills (planning, impl-task, bm-task):

1. Remove all Claude-only references (`Agent()`, `Explore`, `LSP`, `mcp__ref-context__`,
   `[role:*]`, DQ attribution rules, daemon dispatch, forbidden windows, Shape-G,
   ci-watcher, etc.)
2. Add pi-native equivalents:
   - Replace `Explore` with `rg` and `find` bash commands
   - Replace `LSP` with `rg "fn |struct |trait |impl"` patterns
   - Replace DQ mid-task push discipline with "use `bash` to append entry"
3. Add a **"When loaded"** preamble that lists the `.claude/` read paths authorized
   for this role (solving the AGENTS.md §2 tension)
4. Replace headless-daemon phrasing with interactive-pi phrasing

Also scrub the other pi skills (code-reviewer, silent-failure-hunter, etc.) for any
Claude-only references that slipped through.

### Stream F — Authorized .claude/ paths in mode instructions (Task 4)

The mode instructions injected by `before_agent_start` should explicitly list the
`.claude/` paths the agent is authorised to read for its current role. This overrides
AGENTS.md's blanket "don't read .claude/" restriction in a targeted way.

## 5. Files to change

| File | Change | Task |
|------|--------|------|
| `.pi/extensions/lemmy-hooks.ts` | Add `MODE_CONTEXT` map, rewrite `before_agent_start`, add profile reader | 1,2,3,4 |
| `.pi/skills/planning/SKILL.md` | Rewrite for pi-native tool set | 5 |
| `.pi/skills/impl-task/SKILL.md` | Rewrite for pi-native tool set | 5 |
| `.pi/skills/bm-task/SKILL.md` | Rewrite for pi-native tool set | 5 |
| `.pi/skills/code-reviewer/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/silent-failure-hunter/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/type-design-analyzer/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/comment-analyzer/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/code-simplifier/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/docs-impact-agent/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/pr-test-analyzer/SKILL.md` | Scrub for Claude-only references | 5 |
| `.pi/skills/ci-watcher/SKILL.md` | Scrub for Claude-only references | 5 |
| `.claude/PRPs/reports/pi-harness-context-injection-retro.md` | Write retro | 8 |

**Not modified:**
- `.pi/PROJECT_CONTEXT.md` — kept as-is; the slicing logic lives in `lemmy-hooks.ts`.
- AGENTS.md — kept as-is; the mode instructions override its restrictions dynamically.
- `.claude/` ownership area — no changes except the retro report.

## 6. Pre-flight checks

**Mode note:** this plan writes to `.claude/PRPs/reports/`. Run in harness-maintenance mode (`/brehon-mode harness-maintenance`) to avoid path-policy blocks on retro writes. If blocked mid-execution, use `/brehon-override <path>` as a one-time escape hatch.

0. **Check PMD for relevant lessons:** `scripts/brehon/pmd-query.sh "<scope keywords>" --limit 5`. Do this before any code read — existing lessons may change the implementation approach or surface known pitfalls. This is a standard pre-flight step for all pi-harness plan execution (per `session-retro-2026-06-10-pi-harness-context-injection.md` §"What surprised us").
1. The full `before_agent_start` handler in `lemmy-hooks.ts` and its current injection structure.
2. The `BREHON_MODES` dictionary to understand the current mode-system shape.
3. The `FACTORY_PROFILE_TO_MODE` mapping to understand the profile→mode translation.
4. The factory profile files in `.pi/harness-factory/profiles/` for persona + memory fields.
5. The `.pi/skills/` directory for all skill files that need scrubbing.

## 7. Validation gates

**Gate 1 (Task 0):** `rg` for `Agent(` / `LSP` / `mcp__ref-context` / `[role:` / `subagent` / `Explore` / `Shape-G` / `forbidden-window` / `ci-watcher` / `validate-pending-laptop` / `EliteDesk` across all `.pi/skills/*/SKILL.md` — capture the full list as a baseline before edits.

**Gate 2 (Tasks 1+3):** bash probe — the system prompt injected for planning mode should not contain cargo-wrapper references. The rule index for BM mode should show 4-6 rules, not 25.

**Gate 3 (Task 2):** after a `/brehon-mode planning` switch, the planning skill's key instructions
(e.g. "complexity score", "FILES YAML block") should appear in the agent's system prompt
without requiring a manual `/skill:planning` command.

**Gate 4 (Task 5):** `rg -n "Agent(\|LSP\|mcp__ref-context\|\[role:\|Explore\|Shape-G\|ci-watcher\|forbidden-window\|validate-pending\|EliteDesk" .pi/skills/ --include="SKILL.md"` returns zero hits for the rewritten skills.

**Gate 5 (Task 4):** after switching to any mode, the system prompt should include an "Authorized .claude/ paths" section listing the files the role can read, with a note that this overrides AGENTS.md's blanket restriction.

## 8. Risk watchpoints

| # | Watchpoint | File | Risk | Mitigation |
|---|---|---|---|---|
| W1 | `before_agent_start` returns `{ systemPrompt: ... }` which is CHAINED — other extensions modify the same prompt | extension API contract | Adding mode-slicing could accidentally override another extension's injection | Always start with `event.systemPrompt` (the chained value), not from scratch |
| W2 | Skill files the auto-loader injects may be LARGE (planning SKILL.md is ~300 lines) | `.pi/skills/planning/SKILL.md` | Auto-injecting a 300-line skill adds context budget pressure | The skill is injected once per turn start (not per tool call), and replaces content that was already being injected (the monolithic PROJECT_CONTEXT blob). Net context change should be neutral or negative. |
| W3 | `.pi/skills/{mode}/SKILL.md` may not exist for some mode | filesystem | Auto-inject errors crash `before_agent_start` | Check `fs.existsSync()` before reading; fall back gracefully to the current behaviour |
| W4 | The factory profile reader adds I/O to every `before_agent_start` call | `active.json` | 3× `readFileSync` per turn adds startup latency | Profile data is session-stable — read it once in `session_start`, cache in a module-scoped variable; only re-read if mtime changed (same pattern as `syncFactoryActiveMode`) |
| W5 | Rewritten skills must still work for Claude Code harness | dual-harness constraint | Removing `[role:*]` dispatch markers, `Explore`, `LSP` breaks the Claude agents these skills were ported from | **Do not delete from `.claude/agents/`.** The pi skills are COPIES, not symlinks. Claude Code files live in `.claude/agents/` and are untouched by this plan. |
| W6 | AGENTS.md says "do not read .claude/rules/*.md" — authorizing them in mode instructions creates inconsistency | AGENTS.md | User confusion: "which rule wins?" | The mode instructions are dynamic and scoped. AGENTS.md is the default static rule. Document this in the mode instructions emitted by `before_agent_start`. |

## 9. Task breakdown

### Task 0 — Pre-flight baseline

- `rg` for all Claude-only patterns across `.pi/skills/` (see Gate 1 above)
- Read the full `before_agent_start` handler current implementation
- Read the `BREHON_MODES` dict
- Read the 7 factory profiles for persona/memory fields
- Record counts in a baseline note

**FILES:**
```yaml
creates: []
modifies: []
```

**ACTION:** `rg -n "Agent(\|LSP\|mcp__ref-context\|\[role:\|Explore\|Shape-G\|ci-watcher\|forbidden-window\|validate-pending\|EliteDesk\|subagent_type" .pi/skills/*/SKILL.md | tee .pi/baseline-claude-refs.log`

### Task 1 — Mode-aware context slices in `lemmy-hooks.ts`

Add a `MODE_CONTEXT` map to `lemmy-hooks.ts` (module scope, near the existing `BREHON_MODES` dict). Structure:

```typescript
const MODE_CONTEXT: Record<BrehonMode, {
  persona: string;
  projectContextSections: string[];  // which parts of PROJECT_CONTEXT.md to include
  ruleFilter: string[];              // .claude/rules/ subset to list
  recommendedSkill: string | null;   // path to matching skill
  authorizedReadPaths: string[];     // .claude/ paths to authorize
  maxRulesToShow: number;            // limit for the rule index
}> = { ... }
```

Then rewrite the context-building logic in `before_agent_start` to:
1. Look up `brehonMode` in `MODE_CONTEXT`
2. Build context from only the specified PROJECT_CONTEXT.md sections
3. Filter the rule list
4. Include the authorized-read-paths notice
5. If `recommendedSkill` is set and the file exists, `readFileSync` and inject into `additions[]`

**FILES:**
```yaml
creates: []
modifies: [.pi/extensions/lemmy-hooks.ts]
```

**IMPLEMENT (file 1 of 1):** in `.pi/extensions/lemmy-hooks.ts`:
- Add `MODE_CONTEXT` dict after `BREHON_MODES` (or as a computed property derived from it)
- Rewrite the context-building section of `before_agent_start` (the section that builds `additions[]`)
- Keep existing fallback behaviour if `MODE_CONTEXT[mode]` is undefined

**Validation:**
- Manual read of the modified `before_agent_start` return value
- Confirm `main-safe` still injects the full PROJECT_CONTEXT.md (default behaviour unchanged)
- Confirm `planning` mode injects only Brehon constraints + ADR doc refs + dual-harness boundary, NOT cargo-wrappers or subagent section

### Task 2 — Auto-inject role skill

Extend Task 1's change. In the `before_agent_start` handler, after building the context additions, if `MODE_CONTEXT[mode].recommendedSkill` is set:
- Read the SKILL.md from the path
- Append a `## Active Skill: <name>` section to the system prompt additions

The injection preamble should say something like:
```
## Active Skill: planning

Auto-loaded because Brehon mode is BREHON:PLAN. The full skill content follows:
```

**FILES:**
```yaml
creates: []
modifies: [.pi/extensions/lemmy-hooks.ts]
```

**IMPLEMENT (file 1 of 1):** same file as Task 1 — merge the edits so the file write accumulates.

**Validation:**
- After Task 2, a pi session on `/brehon-mode planning` should have the planning skill's
  "complexity score" / "FILES YAML block" / "per-task IMPLEMENT discipline" instructions
  visible in the system prompt WITHOUT requiring a `/skill:planning` manual invocation.

### Task 3 — Filtered rule index + factory profile

Extend the `MODE_CONTEXT` entries with `ruleFilter` arrays. For the rule-index section
of the injected context, use the filtered list:

```typescript
// Inside the context-building logic:
const rulesToShow = contextEntry.ruleFilter.length > 0
  ? ruleFiles.filter(f => contextEntry.ruleFilter.some(r => f.endsWith(r)))
  : ruleFiles;  // main-safe: show all
```

**FILES:**
```yaml
creates: []
modifies: [.pi/extensions/lemmy-hooks.ts]
```

**IMPLEMENT (file 1 of 1):** same file, same `before_agent_start` handler — add the rule-filter logic.

Also in this task: add the factory-active-profile reader. In `session_start`, cache the
active profile's persona and memory fields. In `before_agent_start`, if the mode
matches and the profile has persona text, override the generic persona line.

### Task 4 — Authorized `.claude/` read paths in mode instructions

For each mode in `MODE_CONTEXT`, populate the `authorizedReadPaths` array. Then in
`before_agent_start`, generate a note like:

```
**Authorized .claude/ paths for this session:**
- .claude/rules/branch-manager.md
- .claude/rules/decision-queue.md
- .claude/rules/phase-branch.md
- .claude/commands/bm/*.md
- .claude/runlog/

(Override of AGENTS.md's blanket restriction. These are authorised because the
current Brehon mode requires them.)
```

**FILES:**
```yaml
creates: []
modifies: [.pi/extensions/lemmy-hooks.ts]
```

### Task 5 — Rewrite planning/SKILL.md for pi

Rewrite `.pi/skills/planning/SKILL.md` with pi-native patterns:

**Remove:**
- `Agent(subagent_type: "Explore")` — replace with `rg` and `find` bash commands
- `LSP` tool — replace with `rg "fn |struct |trait |impl .* for"` patterns
- `mcp__ref-context__ref_search_documentation` — replace with `read` of crate docs or `bash 'cargo doc ...'`
- `[role:planning]` dispatch references
- DQ pre-seed `answered_by: "planner"` attribution rules
- Forbidden window checks
- Parallel-cohort `[P]` markers (Junior orchestration-specific)
- Shape-G vs Pre-Shape-G validation delegation
- `--features full` incompatibility lesson references (live in `.claude/lessons/`)
- EliteDesk memory constraints / worker discipline

**Keep (pi-relevant):**
- Plan template structure (20 sections)
- Per-task FILES YAML block discipline
- Complexity score computation
- DoD dry-run requirements
- Story-grain verification (§16a)
- Lesson trailer conventions (RLS)

**Add:**
- "When loaded" preamble listing authorized `.claude/` read paths
- pi-native tool replacements (as above)
- A note: "This skill auto-loads when in BREHON:PLAN mode."

**FILES:**
```yaml
creates: []
modifies: [.pi/skills/planning/SKILL.md]
```

### Task 6 — Rewrite impl-task/SKILL.md + bm-task/SKILL.md for pi

Same approach as Task 5, applied to the other two role skills. The Claude-only patterns
to strip are the same ones.

**FILES:**
```yaml
creates: []
modifies:
  - .pi/skills/impl-task/SKILL.md
  - .pi/skills/bm-task/SKILL.md
```

Also scrub the 8 non-role pi skills for Claude-only references (code-reviewer,
code-simplifier, comment-analyzer, docs-impact-agent, pr-test-analyzer,
silent-failure-hunter, type-design-analyzer, ci-watcher). Each likely has at most
a `> Pi migration note` header and a few lines of Claude-era frontmatter to remove.

**FALLBACK:** if a skill has extensive Claude-specific content, replace the Claude
instructions with a clear note: "This skill was ported from Claude Code. Pi uses the
[skill name] tool directly — see `docs/skills.md`." The key fix is removing
instructions that would cause a pi agent to hallucinate tools.

### Task 7 — Integration: verify the full mode-switch flow

End-to-end verification cycle for each mode:

1. `/brehon-mode planning` → confirm system prompt contains:
   - Brehon constraints (no cargo wrappers)
   - Planning skill instructions (complexity score, YAML blocks)
   - Authorized `.claude/` read paths
   - Planning profile's persona text from factory

2. `/brehon-mode impl-task` → confirm system prompt contains:
   - Full Brehon constraints + cargo wrappers + error conventions + Rust tactics
   - Impl-task skill instructions (per-task validation, MIRROR discipline)
   - Authorized `.claude/` read paths
   - Impl-task profile's persona

3. `/brehon-mode bm` → confirm system prompt contains:
   - Brehon constraints only (no cargo wrappers, no Rust tactics)
   - BM-task skill instructions (9 verbs, hard refusals, confirmation protocol)
   - Subagent delegation notice ("prefer delegating to .pi/agents/bm-pi.md")
   - Authorized `.claude/` read paths (branch-manager, commands/bm/, runlog, etc.)

4. `/brehon-mode main-safe` → confirm existing behaviour preserved (full PROJECT_CONTEXT.md, all rules, no skill auto-inject)

**FILES:**
```yaml
creates: []
modifies: []
```
(Verification-only task; may produce a fix commit if issues found)

### Task 8 — Retro

Write `.claude/PRPs/reports/pi-harness-context-injection-retro.md` with:
- What changed (file list)
- What was removed from context (noise reduction counts)
- What was auto-injected (skill lines added)
- Regression risk: any mode-change that broke main-safe behaviour
- Canary: the next planner session using `/brehon-mode planning` — does it correctly surface the planning skill without a manual `/skill:`?

**FILES:**
```yaml
creates: [.claude/PRPs/reports/pi-harness-context-injection-retro.md]
modifies: []
```

## 10. Appendix: Mode context slice map

### PROJECT_CONTEXT.md section inventory

| Section | Lines | Planning | Impl-task | BM | CI-debug | Review | Harness |
|---------|-------|----------|-----------|----|----------|--------|---------|
| Non-negotiable Brehon constraints | ~25 | YES | YES | YES | YES | YES | YES |
| Coding workflow constraints | ~15 | YES | YES | YES | YES | YES | YES |
| Dual-harness boundary | ~15 | YES | YES | YES | YES | YES | YES |
| Recursive Learning System parity | ~15 | NO | NO | NO | NO | NO | YES |
| Pi session Rust quick-reference | ~90 | NO | YES | NO | NO | YES (error conventions only) | NO |
| Project subagents | ~40 | NO | NO | YES | YES | NO | YES |
| Setup decisions log | ~25 | NO | NO | YES | YES | NO | YES |

### Rule filter by mode

| Mode | Rules to list |
|------|---------------|
| main-safe | All (unfiltered) |
| planning | `handover.md`, `pmd-invariants.md`, `pmd-search-strategy.md`, `session-awareness.md` |
| impl-task | `phase-branch.md`, `no-cargo-output-paste.md`, `cargo-output-capture.md`, `view-crate-selectable-template.md`, `pmd-invariants.md` |
| bm | `branch-manager.md`, `gh-pr-fork-target.md`, `decision-queue.md`, `phase-branch.md`, `pmd-invariants.md`, `universal-guards.md` |
| ci-debug | `phase-branch.md`, `pmd-invariants.md` |
| review-readonly | `pi-harness-constraints.md`, `no-destructive-defaults.md`, `session-awareness.md`, `pmd-invariants.md` |
| harness-maintenance | `pi-harness-constraints.md`, `pmd-invariants.md`, `pre-phase-harness-audit.md` |

### Authorized `.claude/` read paths by mode

| Mode | Authorized paths |
|------|------------------|
| planning | `.claude/commands/prp-core/prp-plan.md`, `.claude/lessons/`, `.claude/PRPs/briefs/`, `.claude/PRPs/plans/`, `.claude/PRPs/reports/`, `.claude/PRPs/templates/` |
| impl-task | `.claude/commands/prp-core/prp-implement.md`, `.claude/lessons/`, `.claude/PRPs/plans/`, `.claude/decision-queue.json` |
| bm | `.claude/rules/branch-manager.md`, `.claude/rules/decision-queue.md`, `.claude/rules/phase-branch.md`, `.claude/rules/gh-pr-fork-target.md`, `.claude/commands/bm/*.md`, `.claude/runlog/`, `.claude/PRPs/reviews/` |
| ci-debug | `.claude/lessons/feedback_gha_pi_loop_postmortem.md`, `.claude/rules/` |
| review-readonly | `.claude/rules/` (all) |
| harness-maintenance | `.claude/skills/`, `.claude/lessons/`, `.claude/PRPs/reports/` |
| main-safe | (no override — AGENTS.md §2 applies) |