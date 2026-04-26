# Design Spec: Haiku 4.5 BM (Branch Manager) Subagent
### Brehon Four-Role Agent Topology — April 2026

***

## Overview

This document specifies the changes required to migrate the BM (Branch Manager) subagent from `claude-sonnet-4-6` to `claude-haiku-4-5`. It covers the rationale, model configuration, subagent frontmatter, effort level, expected cost impact, acceptance criteria, and rollback conditions. It is intended as a hand-off spec for the implementing developer.

**Status:** Recommended for immediate implementation with a 5-task smoke test gate before full rollout.

***

## Rationale

The BM subagent executes nine hard-scripted verbs (`bm-status`, `bm-cut`, `bm-push`, `bm-pr`, `bm-poll-cr`, `bm-prp-review`, `bm-triage`, `bm-merge`, `bm-ping`). Each verb is a 30–100-line script the model reads at invocation start and follows literally. The work is: parse CodeRabbit YAML, run 4–8 `gh`/`git`/`yq` commands, write a YAML/JSON artifact. All hard-refusal boundaries (`never touch crates/**`, `never merge with critical findings open`) are encoded in the script text, not in model judgment.

Anthropic explicitly positions Haiku 4.5 for *"real-time applications, high-volume intelligent processing, cost-sensitive deployments needing strong reasoning, sub-agent tasks"* and names sub-agent tasks as a primary use case. Haiku 4.5 scores 73.3% on SWE-bench Verified and 50.7% on computer-use benchmarks. For literal script-execution with no free-form authorship requirement, this capability level is sufficient — the gap vs Sonnet 4.6 does not materialise in deterministic, bounded workflows.[^1][^2][^3][^4]

**Cost saving:** ~67% reduction in BM spend per sub-phase. At 5 BM tasks × (10K input / 3K output), Sonnet 4.6 costs $0.38/sub-phase; Haiku 4.5 costs $0.12/sub-phase.[^5][^6]

***

## Model Specification

| Parameter | Current | Target |
|-----------|---------|--------|
| Model string | `claude-sonnet-4-6` | `claude-haiku-4-5` |
| Effort level | `high` (default) | `low` |
| Context window | 1M (auto-scaled) | 200K (hard cap — sufficient) |
| Input price | $3.00/MTok | $1.00/MTok[^6] |
| Output price | $15.00/MTok | $5.00/MTok[^6] |
| Cache read price | $0.30/MTok | $0.10/MTok[^6] |

**Effort level justification:** Anthropic's effort documentation specifies `low` as appropriate for *"simple, fast tasks"* that minimise thinking, skips extended reasoning, and use fewest tool calls. BM verbs are exactly this — they are not reasoning tasks, they are command-dispatch tasks. Setting `low` avoids Haiku spending think-tokens reasoning about whether to follow a script it should just follow.[^7]

**Context window:** BM tasks read ~10K input tokens (verb script + brief + YAML artifacts) and produce ~3K output tokens. The 200K ceiling on Haiku 4.5 provides a 20× safety margin.[^8]

***

## Updated Subagent Frontmatter

Replace the current `.claude/agents/bm-task.md` frontmatter block:

```yaml
---
name: bm-task
description: Executes one Brehon Branch Manager verb when dispatched by the advisor
  via Junior. Reads the named BM verb script under .claude/commands/bm/, runs the
  prescribed gh/git/yq commands, writes the required YAML/JSON artifact, and returns
  a structured status. Hard-refusal boundaries are encoded in each verb script —
  never touch crates/**, never merge with critical CodeRabbit findings open, never
  perform confirmation-required actions without writing a DQ pending entry first.
  Pinned to Haiku 4.5 — mechanical script-following with structured output, no
  heavy reasoning required.
tools: Read, Edit, Bash, Glob, Grep
model: claude-haiku-4-5
effort: low
---
```

**No tool changes are required.** The BM subagent's tool set (`Read`, `Edit`, `Bash`, `Glob`, `Grep`) is unchanged. Haiku 4.5 supports all five.

***

## Verb-by-Verb Risk Assessment

| Verb | Work shape | Risk level | Notes |
|------|-----------|------------|-------|
| `bm-status` | Read 3–4 git/gh commands, write YAML | **Low** | Pure read/format; no judgment required |
| `bm-cut` | `git checkout -b`, push, write artifact | **Low** | Deterministic branch naming from brief |
| `bm-push` | `git push origin`, capture output | **Low** | One command; error output is pass-through |
| `bm-pr` | `gh pr create` with templated body | **Medium** | PR body generation requires short prose; monitor for truncated descriptions |
| `bm-poll-cr` | Parse CodeRabbit YAML, bucket by severity | **Medium** | YAML parsing + conditional logic; test severity bucketing explicitly in smoke test |
| `bm-prp-review` | Read plan + CodeRabbit findings, write review artifact | **Medium-High** | Most judgment-heavy verb; if Haiku under-reasons here, fall back to Sonnet for this verb only |
| `bm-triage` | Pick advisor-answer/catch-fire/user-relay from 2–3 options | **Low** | The plan makes the right answer explicit; Haiku's instruction-following is sufficient |
| `bm-merge` | `gh pr merge` after finding checks pass | **Low** | Hard-refusal boundary in script prevents unsafe merges |
| `bm-ping` | Write DQ pending entry, commit-push | **Low** | Structured JSON write; deterministic |

**`bm-prp-review` is the highest-risk verb.** It requires reading a plan section and CodeRabbit findings together and producing a structured review. If the smoke test reveals quality degradation on this verb, it should be pinned to Sonnet 4.6 while the other eight verbs run on Haiku 4.5 — this is straightforward to implement via a separate agent definition file (`bm-prp-review.md` with `model: claude-sonnet-4-6`).

***

## Configuration Change: Junior Worker

Since BM subagents run on the EliteDesk under Junior in `-p` mode, the Junior worker config for BM task types must be updated to pass the correct model flag (or rely on the frontmatter `model:` field if Junior respects it). Confirm which mechanism Junior uses for model selection:

**Option A — Frontmatter respected by Junior:** No Junior config change required; updating the frontmatter `model:` field in `bm-task.md` is sufficient.

**Option B — Junior passes `--model` flag explicitly:** Update the BM task dispatch path in Junior to pass `--model claude-haiku-4-5` (or the equivalent env-var override for the `ANTHROPIC_DEFAULT_HAIKU_MODEL` slot if Junior uses tiered model aliasing).

Verify which option applies before deployment.

***

## Prompt Caching on Haiku 4.5

Haiku 4.5 supports prompt caching at $0.10/MTok cache read and $1.25/MTok cache write (5-minute TTL). For BM tasks dispatched in rapid sequence (e.g., `bm-cut` → `bm-push` → `bm-pr` in one sub-phase), the verb scripts + CLAUDE.md prefix can be cache-marked. Recommended placement:[^6]

1. **First cache breakpoint:** After tool definitions + CLAUDE.md (static across all BM invocations)
2. **Second cache breakpoint:** After the verb script read (static within a verb type, varies between verbs)
3. **Dynamic tail:** Brief + YAML artifacts + task-specific arguments (not cached)

At Haiku 4.5 cache rates, a warm prefix hit on ~5K tokens of tool defs + CLAUDE.md saves ~$0.045/MTok vs uncached — marginal per task but zero-overhead to implement if Junior already applies cache breakpoints on other subagents.

***

## Smoke Test Protocol

Before committing the change to the main Junior config, run the following gate:

### Gate criteria

Run 5 complete BM task sequences (one per verb from the risk table above, covering at minimum: `bm-status`, `bm-cut`, `bm-pr`, `bm-poll-cr`, `bm-prp-review`). For each task:

1. **Artifact schema validity:** The produced YAML/JSON artifact must pass the existing schema validator in `scripts/brehon/validate-bm-artifact.sh` (or equivalent). Zero tolerance — any schema failure is a blocker.
2. **Severity bucketing accuracy:** For `bm-poll-cr`, compare Haiku's severity bucket assignments against Sonnet 4.6's output on the same CodeRabbit YAML input. Accept ≤1 bucket misclassification across the 5 tasks.
3. **Hard-refusal compliance:** Confirm that `bm-merge` refuses to proceed when a synthetic `critical` finding is injected into the CodeRabbit YAML. This is the safety-critical boundary — failure here is a hard blocker regardless of other results.
4. **DQ pending entry on confirmation-required action:** Confirm that `bm-ping` produces a correctly-formed DQ entry with `commit-and-push` rather than proceeding with the confirmation-required action.
5. **No watchdog kills:** All 5 tasks must complete within the 6-minute Junior watchdog timeout. BM tasks have no cargo build dependency and should complete in under 60 seconds each.

### Pass criteria

All 5 tasks pass artifact schema validation, severity bucketing has ≤1 misclassification, hard-refusal compliance is 100%, no watchdog kills. If `bm-prp-review` fails quality review (assessed by human inspection of the review artifact), exempt it from Haiku migration and pin it to Sonnet 4.6 via a dedicated frontmatter file.

***

## Rollback Conditions

Revert `model: claude-haiku-4-5` → `model: claude-sonnet-4-6` if any of the following occur in production:

- Hard-refusal boundary failure (merge proceeds with critical findings open)
- DQ pending rate for BM tasks rises above 2× the Sonnet 4.6 baseline
- Artifact schema validation failures in production
- Watchdog kill rate above 5% of BM tasks (Haiku should be faster than Sonnet for this workload, so any watchdog kills are a signal of unexpected behaviour)

The rollback is a one-line frontmatter change and a Junior worker config update — it can be executed in under 5 minutes.

***

## Expected Outcomes

| Metric | Before | After |
|--------|--------|-------|
| BM cost per sub-phase (5 tasks) | ~$0.38 | ~$0.12[^6] |
| BM cost saving | — | ~67% |
| Sub-phase total (Config A → Config C) | ~$3.48 | ~$3.23 |
| Effort level | `high` (default) | `low` |
| Context usage per BM task | ~13K tokens | ~13K tokens (unchanged) |
| Hard-refusal behaviour | Encoded in script | Encoded in script (unchanged) |

The cost saving is modest in absolute terms ($0.25/sub-phase) but is zero-risk on the critical control paths, requires no architectural change, and establishes the pattern for the larger M2.7 impl migration if that proceeds.

---

## References

1. [Claude Sonnet vs Haiku 2026: Which Model Should You Use?](https://serenitiesai.com/articles/claude-sonnet-vs-haiku-2026) - Claude Sonnet vs Haiku — pricing, speed, and capabilities compared. Find out which Claude model fits...

2. [Claude Haiku 4.5: Features, Testing Results, and Use Cases](https://www.datacamp.com/blog/anthropic-claude-haiku-4-5) - In practice, this means Claude can operate tools like a calculator or a notepad independently, and e...

3. [Claude Haiku 4.5 - Anthropic](https://www.anthropic.com/claude/haiku) - Benchmarks. Haiku 4.5 delivers strong performance and speed across coding, tool use, and reasoning t...

4. [Choosing the right model - Claude API Docs](https://platform.claude.com/docs/en/about-claude/models/choosing-a-model) - Claude Sonnet 4.6, Code generation, data analysis, content creation, visual understanding, agentic t...

5. [Claude API Pricing (March 2026): Opus $5/M Tokens, Sonnet $3 ...](https://www.tldl.io/resources/anthropic-api-pricing) - Claude API pricing 2026: Opus $5.00/M input, $25.00/M output. Sonnet $3.00/$15.00, Haiku $0.25/$1.25...

6. [Pricing - Claude API Docs](https://platform.claude.com/docs/en/about-claude/pricing) - Long context pricing. Claude Mythos Preview, Opus 4.7, Opus 4.6, and Sonnet 4.6 include the full 1M ...

7. [Effort - Claude API Docs](https://platform.claude.com/docs/en/build-with-claude/effort) - At high , xhigh , and max effort, Claude almost always thinks deeply. At lower levels, it may skip t...

8. [Best Claude Models in 2026 — Sonnet vs Opus vs Haiku Compared](https://www.remoteopenclaw.com/blog/best-claude-models-2026) - Best Claude models in 2026 compared by benchmarks and real-world use. Opus 4.6, Sonnet 4.6, Haiku 4....

