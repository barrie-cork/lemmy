# Perplexity research prompt — Rust + Claude Code best practices for Brehon-fork

**Paste the block below into Perplexity (or another deep-research agent).** It is self-contained — Perplexity has no conversation context; the prompt carries everything it needs.

---

I am building **Claude Code agents** that develop a large Rust workspace (~50 crates, a Lemmy 1.0-beta fork at github.com/barrie-cork/lemmy, branch `governance-v0`). I have an existing best-practices report that was written for **Pi Coding CLI** (`badlogic/pi-mono`, `tmustier/pi-extensions`, `pi-rtk-optimizer`, `pi-goal`, `pi-tmux`, `tab-status`, `session-recap`). **None of that Pi-specific tooling applies to my setup** — I need the equivalent best practices specifically for **Anthropic's Claude Code CLI** (`claude` / `claude -p`, subagents via the `Agent` tool, slash commands, hooks, MCP servers, the SDK, headless `-p` mode).

Please research and return a consolidated best-practices report covering the topics below. For each finding, **cite the source** (Anthropic docs, GitHub repos, blog posts, conference talks). Skip anything generic to "AI coding agents" — I want Claude-Code-specific evidence.

## Context about my setup (so suggestions are actionable, not generic)

- **Harness:** Claude Code CLI on Windows 10 (PowerShell + Bash via tool), interactive sessions + headless `claude -p` workers running as a systemd-template daemon on a Linux server (the "Junior" worker pool, one worktree per task).
- **Codebase:** Lemmy 1.0-beta fork — Rust 1.95, ~50-crate workspace, Diesel/PostgreSQL, ActivityPub federation, AGPL. Workspace builds are slow (cold `cargo check --workspace --features full` ≈ 8 min; full e2e ≈ 26 min). The `e2e.rs` integration-test file is ~15,500 lines.
- **Validation modes:** two — (a) GitHub-Actions-side ("Shape G", currently suspended), (b) laptop-side ("validate-pending-laptop") where impl workers raise a DQ entry naming cargo commands verbatim and the advisor runs them locally. Both are active patterns; not all the time, but both real.
- **Role model:** four roles — Advisor (Opus, persistent), Planning (Opus, ephemeral worker), Impl (Sonnet, ephemeral worker), Branch-Manager (Haiku, ephemeral worker). Roles communicate via a JSON decision-queue + plan/brief markdown files committed to a trunk branch. No live chat between workers.
- **Existing infrastructure I want to compare best-practices against:** `CLAUDE.md` at repo root, `.claude/rules/*.md` (auto-loaded at session start), `.claude/agents/*.md` (subagent definitions), `.claude/commands/*.md` (slash commands), `.claude/skills/*/SKILL.md` (user- and project-scope skills), `.claude/PRPs/` (Plan-Research-Proposal artifacts: briefs, plans, reports, templates), `.claude/decision-queue.json` (async coordination), `.claude/lessons/` (markdown lessons indexed into a project-memory MCP SQLite DB), `.mcp.json` (MCP server wiring including a project-memory server, a Junior-worker dispatcher MCP, an `ide` MCP for diagnostics, and a `ref-context` MCP for docs), git hooks (PreToolUse, PostToolUse, Stop) that enforce things like "every Junior task ends with a `memory_write_eval` retro row".
- **What I already do well (do not re-recommend):** precise `CLAUDE.md`; pinned `rust-toolchain.toml`; `cargo check --workspace` before `cargo build`; per-crate test scoping; auto-loaded rule files; a hash-chained governance log; ADR discipline.

## Specific research questions

Please answer each of these explicitly. If a question has no Claude-Code-specific evidence, say so — do not substitute a generic answer.

### A. Context-window economy for Rust compiler output

A1. What is the current Claude Code idiom for **handling multi-thousand-line `cargo check` / `cargo clippy` / `cargo test` output** without flooding the context window? Specific patterns to research: redirecting cargo output to a log file then reading only `tail -N`, using `--message-format=json`, using `cargo check` instead of `build`, capturing exit code separately from output, using the `Bash` tool's `run_in_background` + `Monitor` workflow, using subagents to read large logs and return a synthesis to the parent.

A2. Are there community-published Claude Code skills, hooks, or commands that **classify/filter cargo output** before it reaches the model context? (Equivalent to Pi's `pi-rtk-optimizer filterBuildOutput`.)

A3. What is the recommended way to surface a Rust compiler error with **full context** (lifetime annotations, trait bounds, struct definitions of the involved types) without pasting the entire compiler dump? Are there established subagent patterns (e.g. "compile-error-classifier" agents) that take a raw log and return a structured `(error_code, file:line, suggested_fix, relevant_struct_definitions)` tuple?

A4. How are teams using the **1M-context window of Opus 4.7 / Sonnet 4.6** for Rust work — what is genuinely load-bearing vs what just costs cache misses? Specific question: when does it make sense to load the entire workspace's `Cargo.toml` graph + all crate `lib.rs` files vs. lazy-loading per task?

### B. Subagent orchestration patterns

B1. Best practices for **defining a Rust-aware subagent** in `.claude/agents/<name>.md`: which fields in the frontmatter (`model`, `tools`, `description`) matter most; how to write the body so the subagent doesn't re-derive conventions every invocation; tool-allowlist patterns for read-only vs write-capable Rust subagents.

B2. Patterns for **parallel-subagent dispatch** when refactoring across multiple crates safely — how teams avoid the "two subagents edit overlapping files" problem. Specific question: are there established conventions for `Agent` tool batching, per-subagent worktree isolation, or file-ownership manifests that prevent collision?

B3. When to **delegate read work** (large log analysis, multi-file synthesis, codebase Q&A) to an `Explore` / `general-purpose` subagent vs. doing it inline. What heuristics do teams use — token cost, expected output size, whether the parent needs the raw evidence?

B4. **Long-running Rust validation under subagents** — how teams handle a subagent that needs to run a 26-min `cargo test` while the parent continues other work. Patterns for: background-process tracking, completion notification, exit-code reliability (`run_in_background` is known to occasionally lie about exit codes — what is the canonical fix?).

### C. Headless mode (`claude -p`) for Rust agents

C1. Best practices for running `claude -p` as a **dispatched worker** (analogue to my Junior daemon): structured task input format, output capture, retry-on-failure policy, watchdog timeouts. Are there community projects doing this for Rust specifically?

C2. **Stop-hook discipline** for headless workers — how teams ensure a `claude -p` worker writes a useful artifact (eval, retro, plan, code commit) before exiting, vs. exiting clean with no output. Anthropic's hook system has PreToolUse / PostToolUse / Stop / SessionStart / SessionEnd — which are most leveraged in long-lived Rust agent setups?

C3. **Worktree-per-task** isolation patterns for headless workers — git worktree lifecycle (add → work → push → finalize-merge → remove), submodule init gotchas, `.mcp.json` propagation, branch-naming conventions that the daemon can parse mechanically.

C4. **MCP server wiring in headless mode** — known issues with environment-variable propagation, `.mcp.json` resolution from worktree CWD, what causes MCP servers to silently degrade (e.g. ollama embedding server unreachable → FTS5-only fallback in a memory MCP).

### D. Detecting "compiles-clean but semantically wrong" Rust code

D1. **Same-file sibling conformance** — given a new function added next to an existing canonical sibling doing the same job, how do teams detect divergences that compile cleanly but ship latent bugs (e.g. `.unwrap_or_default()` on a required field where the sibling uses `.ok_or_else(...)?`). Specific question: are there agent prompts, audit skills, or static-analysis crates (beyond clippy) that implement this kind of intra-file consistency check?

D2. **Trait-bound completeness for `#[async_trait]`** — patterns for catching missing `Send + Sync + 'static` bounds at plan-authoring time (before code is written) vs. at compile time. Are there clippy lints, MIRROR-stub-compile-check rituals, or planning-agent conventions that catch this earlier?

D3. **Connection-type discipline** in Diesel + diesel-async codebases — preventing `&mut AsyncPgConnection` vs `&mut DbConn<'_>` confusion. Patterns for typed wrappers, lint rules, agent-readable convention docs.

D4. **Trust-boundary input validation** patterns — federation/ActivityPub-adjacent code that validates remote actor data. How teams catch the `.unwrap_or_default()`-instead-of-`.ok_or_else(...)?` class of bug at audit time, not at incident time.

### E. Self-improving audit / agent calibration

E1. **Precision/recall metrics for AI-driven code audits** — how teams measure whether their audit agent's flagged divergences correspond to real bugs vs. false positives. What ground-truth sources are used (CI failures? CR findings? post-merge bugs?). Specific question: are there published examples of an audit agent that tracks per-axis precision/recall across runs and tunes its own detection patterns?

E2. **Eval rubrics for Rust agents** — pass/fail criteria beyond `cargo check` exit code. Specific question: are there community-published `evals/` directories or eval harnesses for Rust+Claude-Code agents that someone has open-sourced?

E3. **Lesson/pattern promotion loops** — how teams convert a one-off observation ("this failed once") into a durable rule ("the agent always checks for X going forward"). Specific question: what is the canonical Claude Code pattern for this (memory MCP, CLAUDE.md amendments, rule files, hooks)?

E4. **Drift detection** in long-lived agent setups — how teams notice when a once-accurate rule file has gone stale (e.g. references a renamed function, cites a removed crate). Specific question: are there hooks or skills that periodically validate rule-file claims against current HEAD?

### F. Token economy under realistic Rust workloads

F1. **Prompt-cache TTL** (5 minutes for Anthropic) and how to schedule `ScheduleWakeup` / long polls around it — specifically the 270s-cache-warm vs 1200s-one-miss tradeoff. Are there established patterns for Rust agents that wait on long cargo runs?

F2. **What blows up token budgets** in Rust + Claude Code that doesn't in Python/TypeScript + Claude Code. Specific candidates to investigate: macro-expanded code in error messages, type-mismatch diagnostics with deep generic chains, `Cargo.lock` re-reads when dependencies change, repeated `schema.rs` re-reads for Diesel projects.

F3. **`Cargo.lock` and lockfile churn** — how teams handle the multi-thousand-line `Cargo.lock` diff that appears whenever any dep is bumped. Patterns for `git diff` suppression in the agent's context, for separating "real dep changes" from "transitive churn".

### G. Tooling integration

G1. **`cargo expand`, `cargo audit`, `cargo deny`, `cargo machete`, `cargo udeps`, `cargo nextest`** — which of these are routinely wired into Claude Code agent workflows and how? Specific question: is `cargo nextest` worth adopting over `cargo test` for agent-driven test runs (faster, JSON output, retry-flaky support)?

G2. **`rust-analyzer` integration via LSP/MCP** — best practices for letting a Claude Code agent query rust-analyzer for "go to definition", "find references", "hover type". Is there a stable MCP server for this, or are teams shelling out to `rust-analyzer` directly?

G3. **`bacon` as a background compile loop** — how (or whether) teams integrate it with Claude Code. Equivalent to Pi's `bacon` + `pi-tmux` pattern, but for Claude Code's `Bash run_in_background` + `Monitor` workflow.

G4. **CodeRabbit / Greptile / similar AI PR reviewers** alongside Claude Code agents — how teams partition responsibilities (CR for triage, Claude Code for fixes? CR for security, Claude Code for refactor?). Specific question: are there `.coderabbit.yaml` patterns that work well alongside a Claude Code agent that responds to CR findings?

### H. Federation / ActivityPub / Lemmy-fork specifics (lower priority but useful)

H1. **Best practices for Rust agents touching ActivityPub code** — JSON-LD pitfalls, signature verification (HTTP Signatures), nonce/replay protection patterns, `activitypub_federation` crate idioms. Are there fork-friendly patterns for governance / moderation extensions on top of vanilla Lemmy?

H2. **Diesel migration discipline under agent control** — staging migrations separately from Rust schema-type changes (this came up in the Pi report as practice #12 and survives the harness change). Specific question: are there Claude Code patterns for "agent writes migration up.sql → human applies it → agent then writes the Rust schema types" that have proven robust?

## Format I want back

For each section A–H above, return:

1. **A short answer** (2–4 sentences) summarising the current consensus, if one exists.
2. **Concrete sources** — links to Anthropic docs, GitHub repos, blog posts, or talks. Skip "I think" or "you could try" answers without a source.
3. **A "no Claude-Code-specific evidence" verdict** where applicable — say so clearly rather than substituting generic AI-agent advice.
4. **Where Anthropic's official guidance differs from community practice**, surface both and flag the difference.

Skip any section where you have nothing substantive to add. At the end, please surface the 5 highest-leverage practices specific to **Rust + Claude Code** that I am most likely missing given the context above.

## Out of scope (don't research these)

- Pi-specific tooling (`pi-rtk-optimizer`, `pi-goal`, `pi-tmux`, `tab-status`, `session-recap`, `tmustier/pi-extensions`) — I have the Pi report already.
- Generic AI-agent advice ("be specific", "use examples") — assume the reader knows this.
- Anthropic API directly (`messages.create`) — I want CLI/agent-level practices, not raw API.
- Non-Anthropic models (GPT, Gemini, local Llama) — Claude only.
- Languages other than Rust.
- Frontend / SPA / mobile work.

---

_End of research prompt. Paste verbatim into Perplexity._
