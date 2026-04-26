I'm running a four-role agent topology for an autonomous software-development orchestration system on top of Claude Code (Anthropic's CLI). I need a current (April 2026) capability + best-practice review of the Claude 4.x family — specifically **Opus 4.7**, **Sonnet 4.6**, and **Haiku 4.5** — to validate model-to-role assignments. Anthropic recently shipped Sonnet 4.6 and Haiku 4.5; I want to know whether my Sonnet-for-impl, Sonnet-for-BM choice is still right or whether I should move impl up to Opus 4.7 (and whether any role should drop to Haiku 4.5).

## The four roles and what each does

### 1. Advisor (orchestrator) — currently Opus 4.7, 1M context, effort=max

Persistent long-running session. Reads design docs, ADRs, retros. Authors briefs (~200-word task prompts) for subagents. Polls subagent status. Triages decision-queue entries (advisor-answer / catch-fire / user-relay). Authors sub-phase retros. Heavy reasoning + judgment + cross-phase pattern recall via memory search.

**What the advisor does in a typical 10-min poll cycle:**
- `mcp__junior-brehon__list_tasks` → status comparison vs last-known
- On status change only: `show_task` for the changed task
- `git fetch origin` + read `.claude/decision-queue.json` for new pending entries
- DQ triage decision tree: advisor-answer / catch-fire / user-relay
- Token cost per cycle when nothing changed: ~500 tokens. Token cost per cycle when a task transitions: 5-30k tokens.

### 2. Planning subagent — currently Opus 4.7

Dispatched per sub-phase. Reads design docs (10-20 files, ~50k tokens), ADRs, prior plans, retros. Runs Explore subagents for cross-codebase context. Drafts a 1500-1800-line plan file with file:line MIRROR refs, watchpoints, and per-task DoD validation commands. Produces one plan, exits. Heaviest single-task reasoning load in the system.

**Real example output**: a recent plan file (`v1-jury-mechanics-c.plan.md`) is 1738 lines / ~131 KB and includes:
- 20 numbered sections (summary, source, problem statement, solution statement with rejected-alternatives discussion, metadata, sub-phase relationships, preflight guardrails, flow design with before/after ASCII diagrams, mandatory reading, patterns to mirror, files to change, NOT-building list, step-by-step tasks, testing strategy, validation commands, acceptance criteria, completion checklist, risks + mitigations, notes, sub-phase stubs)
- ~10 pattern citations to specific upstream Lemmy file:line ranges (MIRROR refs)
- 9 task definitions with per-task DoD validation commands
- 7+ rejected alternatives with reasoning
- Cross-PRD dependency analysis (which sub-phases serialise on this one)

The planning subagent's frontmatter:
```yaml
---
name: planning
description: Authors a Brehon sub-phase plan from a brief. Use when a Junior task description starts with `[role:planning]`. Reads design docs, PRD, ADRs, prior sub-phase reports under .claude/PRPs/, runs Explore subagents for cross-codebase context, drafts a plan file at .claude/PRPs/plans/<sub-phase>.plan.md following the template in .claude/commands/prp-plan.md. Pinned to Opus 4.7 because plan-shaping is the heaviest reasoning role in the four-role model.
tools: Read, Glob, Grep, Edit, Write, Bash, Agent, LSP, WebFetch, mcp__ref-context__ref_read_url, mcp__ref-context__ref_search_documentation
model: claude-opus-4-7
---
```

### 3. Impl subagent — currently Sonnet 4.6 (the question is whether to upgrade)

Dispatched per plan task (8-14 tasks per sub-phase). Reads one task's MIRROR refs, applies the pattern (Rust + Diesel + Postgres), runs `cargo check` / `cargo test --test e2e`, commits one feature commit. Pattern-following from cited file:line examples — not free-form authorship.

**Real example impl-task DoD validation commands (from the JM-c plan §15):**
```bash
./scripts/brehon/cargo-check.sh --workspace --features full
./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings
./scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server
./scripts/brehon/cargo-test.sh --test e2e -p lemmy_server -- --test-threads=1 v1_jm_c
```

**Real example wrapper script** (`scripts/brehon/cargo-check.sh`):
```bash
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
echo "TOOLCHAIN_OK"
command -v cargo && cargo --version
echo "---"
cd "$REPO_ROOT"
exec cargo check "$@"
```

**Representative lessons the impl subagent reads at start** (~20 `feedback_*.md` files in `.claude/lessons/`, each ~300-500 words):

> *Excerpt from `feedback_clippy_test_style.md`:* The Lemmy workspace `[workspace.lints.clippy]` denies `unwrap_used`, `expect_used`, `allow_attributes` — so `#[allow(clippy::unwrap_used)]` does NOT work as an escape hatch. Two legal escape hatches for tests: (1) `async fn -> LemmyResult<()>` with `?` propagation; (2) `async fn -> Result<(), Box<dyn std::error::Error>>` with `?` propagation. Pattern 3 `#[expect(clippy::unwrap_used, clippy::tests_outside_test_module)]` works only because `expect` attributes are different from `allow` attributes under `allow_attributes = deny`.

The impl subagent's frontmatter:
```yaml
---
name: impl-task
description: Executes one implementation task from a Brehon sub-phase plan. Use when a Junior task description starts with `[role:impl-task]`. Reads the named plan task, reads MIRROR refs, makes the change, runs the per-task validation gate (cargo check / e2e / migration round-trip per the plan), and commits with `feat(scope): <title> (task N)` style. Pinned to Sonnet 4.6 — pattern-following from MIRROR refs, not heavy reasoning.
tools: Read, Edit, Write, Bash, Glob, Grep, LSP, mcp__ref-context__ref_read_url, mcp__ref-context__ref_search_documentation
model: claude-sonnet-4-6
---
```

**Typical impl-task work-shape:**
- Read brief (~200 words) + task section of plan (~150 lines / ~5k tokens) + 2-4 MIRROR file:line ranges (~500 lines total)
- 30-50 tool calls per task: ~10 Read, ~10 Edit, ~5 Bash (cargo wrapper invocations), ~10 Glob/Grep, ~5 LSP if available
- Cargo runs capture full output to `.claude/build-task<N>.log` then `tail -20` of the log into reasoning context (per `feedback_no_cargo_output_paste.md`)
- One feature commit per task: `feat(<scope>): <title> (task <N>)`
- DQ pending entry mid-task if blocked (commit-and-push immediately for advisor visibility)

**Representative impl operations the subagent must get right:**
- Propagate a new `InsertForm` field through 4-6 callsites (pattern: each callsite needs `..Default::default()` for backwards compat; trap class: assuming `Option<_>` makes the field auto-compatible)
- Add a Diesel `belonging_to` query against a join table where the snapshot fields live in a different table (`reputation_snapshot` vs `local_user`)
- Replace a hardcoded const (`QUORUM: i64 = 3`) with a snapshot-field read (`case.quorum_snapshot.ok_or(LemmyErrorType::...)?`) at the exact cite the plan names
- Diff cargo lint output between two clippy invocations to determine which lints are pre-existing upstream debt vs lints the change introduced
- Detect that an e2e test failure is a fixture problem (not a handler bug) by tracing which seed function set up the test row

### 4. BM (Branch Manager) subagent — currently Sonnet 4.6

Dispatched per git operation. Cuts branches, opens PRs (`gh pr create`), parses CodeRabbit YAML findings into severity buckets, runs `gh pr merge`. Mostly mechanical — git/yq/gh ops with hard-refusal boundaries (never touch `crates/**`, never merge with critical findings open).

**The 9 BM verbs:** `bm-status`, `bm-cut`, `bm-push`, `bm-pr`, `bm-poll-cr`, `bm-prp-review`, `bm-triage`, `bm-merge`, `bm-ping`. Each is a 30-100-line script the subagent reads at start and follows literally. Most verbs are "run 4-8 `gh`/`git`/`yq` commands and write a YAML/JSON artifact."

**The Junior-dispatched BM has no `AskUserQuestion`** (Junior runs in `-p` mode where the tool doesn't bubble back to a user). Confirmation-required actions instead write a DQ pending entry, commit-and-push it, and return `blocked-on-DQ-#<id>`.

The BM subagent's frontmatter:
```yaml
---
name: bm-task
description: Executes one Brehon Branch Manager verb when dispatched by the advisor via Junior. Pinned to Sonnet 4.6 — git/yq/gh ops, no heavy reasoning.
tools: Read, Edit, Bash, Glob, Grep
model: claude-sonnet-4-6
---
```

## Background context that may matter

- **Each subagent runs on a separate machine** (Ubuntu server, the EliteDesk) under a Junior daemon, not in the same conversation as the advisor. Subagent isolation = each gets a fresh CC session with its own conversation context. **Prompt cache reuse across subagents is impossible** (different machines, different sessions).
- **Junior has a 6-minute stdout watchdog** — if a subagent goes 6 min without writing to stdout, Junior kills the worker (exit 143). This affects long cargo builds (cold first build is ~30-60 min) and matters for cargo-heavy impl tasks.
- **Lemmy is a large Rust workspace** — ~100 crates, full `cargo check --workspace` is multi-minute even warm.
- **Solo-dev budget** — these costs come out of one developer's pocket. £100/mo is a meaningful number; £1000/mo is not sustainable.
- **The advisor session uses Claude Code's `--effort max` flag** at the session level. Anthropic's docs list effort levels `low / medium / high / xhigh / max`.
- **`claude-sonnet-4-7` does NOT exist as of 2026-04-26** — confirmed via Claude Code documentation. Latest Sonnet is `claude-sonnet-4-6`. Latest Opus is `claude-opus-4-7`. Latest Haiku is `claude-haiku-4-5`.

## Specific questions I need answered

### A. Sonnet 4.6 vs Opus 4.7 for the Impl subagent

When the work is "read 1-3 MIRROR file:line refs from a 1500-line plan, apply the pattern, run cargo, commit" — does Opus 4.7's heavier reasoning meaningfully reduce error rate vs Sonnet 4.6? Or does Sonnet 4.6 hit a ceiling on multi-step Rust refactors (e.g. propagating an `InsertForm` field through 4-6 callsites with `..Default::default()`)?

Cite Anthropic's own benchmarks (SWE-bench Verified, Aider polyglot, Anthropic's terminal-bench / agent-bench numbers) and any Anthropic guidance on "when to upgrade Sonnet to Opus." Specifically: do they publish guidance on when **multi-step tool use chains** (~30-50 tool calls per task) start showing cliff effects on Sonnet vs Opus?

### B. Effort levels (low/medium/high/xhigh/max) — what do they actually do?

Document what `--effort max` changes inside the model. Is it:
- An extended-thinking token budget knob?
- An aggressive tool-use planning mode?
- Multiple inference passes per turn?
- Something else?

Is `effort: max` worthwhile on a Sonnet 4.6 subagent, or does the effort dial only meaningfully change Opus behaviour? If Sonnet 4.6 + effort=max approaches Opus 4.7 default-effort quality at lower cost, that's the most interesting answer.

Cite Anthropic's effort-level documentation directly.

### C. Haiku 4.5 — is there a role it should take?

None of my four roles currently use Haiku. Is there a class of work in this system where Haiku 4.5 would be cheaper without quality loss?

Specific candidates I'm considering:
- The advisor's polling-loop status comparisons (when nothing has changed, the work is "compare two JSON arrays")
- The BM subagent's read-only verbs (`bm-status`, parts of `bm-poll-cr`)
- Decision-queue triage when the entry has only 2 options and the right answer is obvious from the plan
- A future "retro-skim" subagent that reads completed retros and indexes them

Cite Anthropic's Haiku 4.5 capability docs — what does Anthropic position Haiku 4.5 for? Any benchmark numbers vs Sonnet 4.6 on the workloads above (specifically: long-context comprehension, structured-output extraction, agentic task accuracy)?

### D. 1M-context window — when is it load-bearing?

The advisor uses Opus 4.7 with **1M context** to hold plan files + retros + ADRs + recent commit history simultaneously. Do the Planning and Impl subagents need 1M, or does the standard 200k window suffice?

- Planning reads ~50k tokens of design docs at start, generates ~30k tokens of output, doesn't need to hold the conversation across multiple plans. Likely fits 200k.
- Impl reads ~5k tokens (brief + task section + MIRROR refs) and runs ~30-50 tool calls. Likely fits 200k.

Anthropic's pricing tier for 1M context is ~2-3x token cost — this matters for solo-dev budget. Can you confirm the current 1M-tier pricing for Opus 4.7 and whether Sonnet 4.6 / Haiku 4.5 even have a 1M tier?

### E. Subagent isolation + prompt-cache reuse

Each Junior-dispatched subagent runs in a fresh worktree on a separate machine with its own conversation context. Prompt caching across subagents is therefore impossible — but **per-subagent prompt-cache hits** (system prompt + tool definitions + initial brief reads + the ~20 lessons file reads at start) should be high since each subagent re-reads the same `.claude/lessons/` files.

Are there best practices for maximising per-subagent cache hit rate when the subagent is invoked via a daemon (Junior) rather than interactive CLI? Specifically:
- Does Anthropic's prompt-caching support persist a cache across CLI `claude -p` invocations on the same machine, or does each `-p` invocation start a fresh cache?
- If the subagent reads ~50k tokens of brief + lessons + plan section at start, can those reads be cache-marked so the subsequent ~30 tool calls hit the cache?
- What's the right `cache_control` placement for an agent that reads a large constant prefix then issues many tool calls against it?

Cite Anthropic's prompt-caching docs.

### F. Tool-use chains — model-specific differences

The Impl subagent runs ~30-50 tool calls per task (Read, Edit, Bash for cargo, Glob, Grep, optional LSP). Does Opus 4.7 produce shorter, more correct tool-call chains than Sonnet 4.6 on this kind of work?

Specifically: when Sonnet 4.6 makes a tool-use error (e.g., reads a file that already exists in context, or runs a Bash command whose output it could have predicted from prior reads), does that error rate close meaningfully on Opus 4.7? Or is the gap mostly about reasoning quality on the in-context content, not tool-use planning?

Cite any Anthropic guidance on agentic workflows / multi-step tool use that compares 4.x models.

### G. Cost-per-sub-phase rough estimate

Given a sub-phase = 1 planning task + 10 impl tasks + 5 BM tasks, with rough token shapes:

| Task class | Input tokens | Output tokens | Tool-use turns |
|---|---|---|---|
| Planning (1×) | 50k | 30k | 20 |
| Impl-task (10×) | 20k each | 10k each | 35 each |
| BM-task (5×) | 10k each | 3k each | 8 each |

What's the expected $ cost difference between:
- **(a)** keeping Sonnet 4.6 for impl + BM + Opus 4.7 for planning (current config)
- **(b)** upgrading impl to Opus 4.7
- **(c)** dropping BM to Haiku 4.5

Cite current Anthropic pricing for the 4.x family.

## Format the response as

1. **One-paragraph TL;DR** with concrete recommendations per role (model + effort).
2. **Sections A-G answered** with citations to Anthropic primary sources (`docs.anthropic.com`, Anthropic blog posts, model cards). Avoid third-party benchmarks unless Anthropic itself cites them.
3. **A "what to verify yourself before changing config" appendix** — i.e. things Perplexity can't be sure of from public docs that I should test against my actual workload before committing the change.

## Hard constraints on the answer

- Use Anthropic's primary documentation as the source of truth. Where third-party sources contradict Anthropic, flag it.
- Do not recommend models or features that don't exist as of April 2026 (e.g. `claude-sonnet-4-7` does not exist; Opus 4.7 + Sonnet 4.6 + Haiku 4.5 are the current family).
- If a question can't be answered from Anthropic's public docs, say so — don't extrapolate from Claude 3.x or other generations.
- Where a recommendation is "it depends on your workload," name the exact metric I should measure to decide (e.g., "tail-latency on cargo runs," "DQ pending-rate per impl task," "tool-call retry rate per task").
