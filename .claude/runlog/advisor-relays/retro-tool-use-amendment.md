---
id: retro-tool-use-amendment
from: advisor
to: impl
ts: 2026-04-24T02:20Z
relates_to: phase-v1-JM-a-retro.md, commit 8f50a5e00
decision: amend-retro-with-tool-use-section
---

# Decision
Add a §"Tool-use self-assessment" section to `phase-v1-JM-a-retro.md`. User wants to know whether the agent is using available tooling appropriately. Baseline for v1-JM-a; the same section becomes a plan-template requirement for v1-JM-b onwards.

Scheduling: **can slot in before OR after the lows batch**. Independent work; no dependency. If you're already mid-batch on cr-1..cr-6, finish that first and amend the retro after. Otherwise do the amendment now while context is fresh.

# Instructions

Amend `.claude/PRPs/reports/phase-v1-JM-a-retro.md` (on JM-a worktree) by appending a new section before the existing §"Handoff notes for JM-b" (or wherever fits the existing ordering). Commit subject:

```
docs(v1-JM-a): amend retro — tool-use self-assessment section
```

## Section structure (use this template verbatim — user wants consistency)

```markdown
## §N. Tool-use self-assessment (added 2026-04-24)

### §N.1. Tools used heavily this phase

<Bullet list. Each bullet: tool name + approximate frequency + what it was used for.>
<Examples:>
<- `Read`: ~50 reads. Plan file, PRD, existing crate source, prior retros, findings YAML.>
<- `Bash` (git/gh): ~30 calls. git log, git status, gh pr view, gh api.>
<- `Grep`: ~20 calls. Cross-checking enum vocab, searching for call sites.>

### §N.2. Tools NOT used that would have helped

<Bullet list. Each bullet: tool name + what moment it would have saved effort + why it wasn't used (forgot? not available? preferred alternative?).>
<Be honest — this is where the retro has value. If nothing comes to mind, say "none identified" and note the session's blind spot: "self-observation is lossy; offline transcript analysis may surface opportunities I missed".>
<Candidates to consider:>
<- Agent subagent with subagent_type=Explore for wide codebase searches (when you ran 3+ sequential greps on related terms, that's a signal one Explore call would have been cheaper)>
<- Plan tool (exits plan mode after confirmation) when doing larger design work before code — the `/plan` discipline is in CLAUDE.md but the TOOL itself wasn't necessarily used>
<- Rust LSP (if available in the environment): for type-resolution, go-to-def, find-references. Check whether `mcp__ide__getDiagnostics` or similar was available in your session; if yes, was it used?>
<- WebFetch for PRD § URLs or ADR pages vs local Read>
<- TaskCreate/TaskUpdate for progress tracking within the phase>

### §N.3. Rule-violation near-misses

<Bullet list of near-misses against auto-loaded rules. Self-report honestly — the rules exist because past incidents caused them, and catching near-misses now prevents repeat.>
<Rules to check against:>
<- `cargo-output-capture.md` — did any cargo invocation pipe through tail/head/grep without capturing to file first?>
<- `no-cargo-output-paste.md` — did more than 20 lines of cargo output land in the conversation?>
<- `decision-queue.md §attribution-integrity` — any DQ write where attribution could have been clearer?>
<- `pm-plugin-hooks-stable.md` — any PM-adjacent code touched? (JM-a shouldn't have, but verify)>
<- `pre-phase-harness-audit.md` — did Task 0 audit run all 4 wrapper probes + Docker + DoD smoke + clippy baseline?>
<- `phase-branch.md` — any direct commit to governance-v0? (no, but confirm)>

### §N.4. Context-management signals

<Two or three bullets on how the session managed the context window. Not a rigorous count — just gut impressions with anchoring evidence if possible.>
<- Approximate session token high-water mark (if you can read it from the session, e.g. "~180k at Task 9 park"; if not, say "unknown — no session-level meter")>
<- Re-reads: files you re-read during the phase that could have been kept in context or read once-and-extracted? (e.g. plan file re-read every task start = expected; PRD re-read 6 times for cross-references = maybe extractable)>
<- Cargo output budget: total lines of cargo output that entered the conversation vs stayed in `.claude/*.log` files. The memory `feedback_context_trim_verify_empirically` says anything past ~200k tokens degrades reasoning; cargo output is the dominant cost.>

### §N.5. Agent/subagent use

<One or two bullets. Did the session spawn subagents (Agent tool)? For what? Were they right-sized (Explore for wide search, general-purpose for complex multi-step, specialized types where they matched)?>
<If zero subagents were used: note that and self-assess whether the phase would have benefited. Parallel Explore on large codebases is usually the right call; a single thread grepping is usually the wrong call.>

### §N.6. Lessons for JM-b and the plan template

<2-3 concrete recommendations. Should the plan template require X? Should a rule be added? Should a /handover skill include a tool-use checklist at session start?>
```

## Tone

This section is **self-report**, not defensive. The user is asking a real question — "is Claude using available tooling appropriately" — and a retro that says "I used everything perfectly, no improvements needed" is worthless. Honest self-assessment + one or two concrete lessons is worth more than a long summary.

If you genuinely can't recall tool-use patterns across the whole phase, say so — but try to give concrete numbers on at least the current-session segment (the one that's still in your context). For prior session(s), do a best-effort reconstruction from commit messages + runlog entries + relay files, and label it explicitly as "reconstructed, not directly observed".

## Validation

No new tests. Retro is docs-only. L1 check the commit compiles cleanly (trivial — markdown-only).

# Next

1. Amend retro (before or after lows batch, your call).
2. Commit with subject `docs(v1-JM-a): amend retro — tool-use self-assessment section`.
3. Continue with (or resume) lows batch per `advisor-relays/pr92-lows-batch.md`.
4. When both are done, relay `impl-relays/pr92-lows-complete.md` noting both commits are in.

If you amend the retro BEFORE the lows batch, BM will push it as part of the next `phase-v1-JM-a` push (after lows). One push for both.

# For the /handover skill design

This retro amendment is also a design input for the future `/handover` skill. Per `project_handover_skill_retro_pending.md`, the skill retro at Task 11 close already captured what worked in the park/resume flow. Add an additional bullet to that skill's design spec:

> Handover briefs should include a "tool-use hint" section — a one-liner per task hint naming the tools/subagents that match that task shape (e.g. Task 3: "Rust enum creation — Read pg migration first, Edit Rust file, run cargo-check.bat -p lemmy_db_schema"). This primes the fresh session to reach for the right tools without having to re-derive the mapping each time.

Add this to the retro's §"Handover skill design inputs" section (or wherever the existing skill-design notes live).

# Back-reference

Addresses user request 2026-04-24 via advisor session: "will the retro reflect on tool use? as I would like to know whether the agent is using the available tools appropriately."
