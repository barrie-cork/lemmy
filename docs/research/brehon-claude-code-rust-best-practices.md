# Claude Code + Rust Best Practices

**For:** Barrie Tyner / `barrie-cork/lemmy@governance-v0` (Lemmy 1.0-beta fork harness)
**Date:** May 2026
**Scope:** Anthropic Claude Code CLI only (`claude`, `claude -p`, subagents, slash commands, hooks, MCP, SDK). Rust-specific evidence preferred. Pi tooling, raw API, non-Anthropic models, frontend/mobile all excluded by request.

Every subsection is cross-referenced against your existing `.claude/` setup (`CLAUDE.md`, 24 rules files, 12 agents, 11 hooks, 15 skills, hash-chained governance log, polling-loop architecture). Where your harness already exceeds community practice, that is stated; where there is something genuinely worth adding, it is flagged for the top-5 list at the end.

The single biggest finding: your harness is more sophisticated than nearly every published community example I found. The gaps are narrow and specific — they live in tooling adoption (nextest, rust-analyzer MCP, CodeRabbit CLI, squeez), in one cache-TTL detail that bites every Junior poll, and in semantic-bug-detection categories where no Claude-Code-specific evidence exists at all.

---

## TL;DR — Top 5 Highest-Leverage Practices You Are Most Likely Missing

These are the five additions that survive scrutiny against everything in your existing harness. Full justification and citations are in the section bodies; this is the short version.

1. **`ENABLE_PROMPT_CACHING_1H=1` for all worker sessions.** Subagents always use the 5-minute cache TTL with no override available — confirmed by Boris Cherny (Claude Code lead) on [Hacker News](https://news.ycombinator.com/item?id=47740756). Every Junior poll that crosses the 5-minute mark pays full cache-write cost (1.25× base) on the next turn. For API-key auth the 1-hour TTL is opt-in via this env var; the 2× write premium pays for itself after avoiding a single miss. ([Claude Code prompt-caching docs](https://code.claude.com/docs/en/prompt-caching))

2. **Adopt `cargo nextest` over `cargo test` for the impl-task and CI-watcher agents.** Isolated-process execution, `--message-format libtest-json` for machine-parseable output, first-class `--retries N` with flaky-test classification, and `--rerun` for failed-only re-execution. The Claude Code skill for this is [published on MCP Market](https://mcpmarket.com/tools/skills/cargo-nextest-rust-test-runner). This is a near-pure upgrade over `cargo test` for agent runs.

3. **`rust-analyzer` via MCP (`zeenix/rust-analyzer-mcp`).** Installable with `cargo install rust-analyzer-mcp`. Exposes 10 LSP-grade tools (`definition`, `references`, `hover`, `diagnostics`, `workspace_diagnostics`) over stdio MCP. Adds project-aware semantic navigation that `grep`/`Read` cannot match for D1 sibling-conformance and D2 trait-bound work. Add to your gitignored `.mcp.json`. ([GitHub](https://github.com/zeenix/rust-analyzer-mcp))

4. **CodeRabbit CLI as a generate→review→fix loop, ingesting your existing CLAUDE.md.** Run `coderabbit review --prompt-only --type uncommitted` from inside an agent before commit. CodeRabbit [automatically reads `**/CLAUDE.md`](https://docs.coderabbit.ai/knowledge-base/code-guidelines) as review criteria, so your 24 rules files and CLAUDE.md become the reviewer's spec without duplication. Closes the loop your CR-triage gate currently leaves open. ([CodeRabbit CLI](https://coderabbit.ai/cli))

5. **Clippy `disallowed_methods` + Dylint for D4 federation trust-boundary lints.** Your harness has no static-analysis gate for `.unwrap_or_default()` on remote `Person`/`Activity` fields. Both Clippy ([config docs](https://doc.rust-lang.org/clippy/lint_configuration.html)) and Trail of Bits' [Dylint](https://github.com/trailofbits/dylint) cover this; no Claude-Code-specific tool exists. A per-module `#![deny(clippy::disallowed_methods)]` in `crates/activitypub_federation/src/...` plus a `clippy.toml` entry banning `Option::unwrap_or_default` in federation paths is the cheapest path. ([Sherlock Rust Security Auditing Guide 2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026))

Honourable mentions that did not make the top 5 but are worth considering:
- **`BASH_MAX_OUTPUT_LENGTH` tuning** — your cargo-output rules say "tail to file" but don't pin the env var; default 50 000-char middle-truncation hides test-summary tails when `tail` is forgotten.
- **`squeez` PostToolUse hook** for ~95% Bash-output compression — `cargo install squeez`, hooks in via `squeez setup --host=claude-code`. Less critical given your `check-cargo-pipe.sh`, but free upside for non-cargo Bash.
- **`Cargo.lock` diff suppression** via `.gitattributes` `*.lock diff=hidden` or [`git-prism`](https://dev.to/mikelane/teaching-claude-to-stop-reaching-for-git-diff-git-prism-v070-4nel). One CLAUDE.md line plus a gitattributes entry stops the lockfile-diff context dump.

The remainder of this report is the per-section evidence.

---

## Section A — Context-Window Economy for Rust Compiler Output

### A1. Multi-thousand-line `cargo check`/`clippy`/`test` output

The canonical pattern is: redirect cargo output to a log file, tail the relevant slice, set `BASH_MAX_OUTPUT_LENGTH` to a tolerable cap, and capture exit codes separately. Claude Code middle-truncates Bash output past the limit (default 50 000 chars) — for large test suites this removes the *middle* and keeps head + tail, but for cargo output the diagnostic information is mostly in the tail, so unbounded output without `tail` is risky. `cargo check` rather than `cargo build` cuts output by ~3×.

Your `check-cargo-pipe.sh` already blocks the worst failure mode (pipes that mask exit codes via PIPESTATUS), and your `cargo-output-capture.md` / `no-cargo-output-paste.md` rules already enforce log-to-file. **You are ahead of community practice here.** The one detail not pinned in your rules: `BASH_MAX_OUTPUT_LENGTH` itself. Setting it explicitly in `.claude/settings.json` (e.g. to 30 000 or 50 000) makes the truncation behaviour deterministic across hosts.

- [`BASH_MAX_OUTPUT_LENGTH` reference (Introl)](https://introl.com/blog/claude-code-cli-comprehensive-guide-2025)
- [Stack Overflow on truncation](https://stackoverflow.com/questions/79716276/how-to-make-claude-code-show-the-whole-output-in-the-terminal-without-truncating)
- [Anthropic — Effective Context Engineering for AI Agents](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) (tool-result clearing)

**No Claude-Code-specific evidence** for `cargo --message-format=json` integration as an established idiom. It can be piped through `jq` manually but no published hook or skill does this.

### A2. Community hooks that classify/filter cargo output

The most developed published tools are **squeez** (Rust crate, `cargo install squeez`, `PostToolUse` hook with 4-stage compression pipeline — smart filter → dedup → grouping → truncation, ~95% reduction on verbose output) and **conclaude** (Rust crate, `Stop`-hook orchestrator that runs `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo build` and blocks session completion on non-zero exit).

- [squeez](https://skillsllm.com/skill/squeez) — generic compressor, applies to all Bash output not just cargo
- [conclaude](https://lib.rs/crates/conclaude) — synchronous Stop-hook
- [Anthropic hooks reference](https://docs.anthropic.com/en/docs/claude-code/hooks)

**Vs your harness:** `check-cargo-pipe.sh` + `retro-check.sh` together do the conclaude job and more (your retro gate is branch-scoped and forces a structured retro write, not just an exit-code check). `squeez` is genuinely complementary — it would compress Bash output that isn't cargo (`git log`, `grep`, etc.) before it hits context. Marginal gain but cheap to install.

**No Claude-Code-specific published "compile-error-classifier" hook** has been released in `awesome-claude-code` or any Anthropic repo.

### A3. Surfacing Rust compiler errors with full context

No established compile-error-classifier subagent with a structured schema (`error_code`, `file:line`, `suggested_fix`, `relevant_struct_definitions`) has been published. The closest idiom is the built-in **Explore** subagent (Haiku, read-only) — pipe `cargo check 2>&1` to a file, delegate the `grep`/`cat` work to Explore, receive a summary. Anthropic's docs explicitly recommend this pattern: ["a subagent handles the research in its own separate context window, so the large file reads stay out of yours."](https://code.claude.com/docs/en/context-window)

- [Anthropic — Sub-agents](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
- [Anthropic — Multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system)
- [Anthropic — Effective context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) ("subagents may use tens of thousands of tokens but return 1–2 k token summaries")

**Vs your harness:** your `silent-failure-hunter` and `type-design-analyzer` agents already approximate this pattern. A dedicated `compile-error-classifier` with a hard-pinned output schema (the Sherlock-style severity field plus a citation discipline) would be a small, high-value addition — but it's a "would be nice", not a critical gap.

**No Claude-Code-specific evidence** for a published structured-schema compile-error subagent.

### A4. 1M-context models for Rust workspaces

Lazy-load per task is the documented default. Anthropic explicitly recommends just-in-time loading with file-path identifiers, not upfront bulk loads. Opus 4.7 / Sonnet 4.6 with 1M context is justified only for cross-crate reasoning that cannot be decomposed into per-crate subagent tasks. Sonnet 1M costs 2× per input token ($6/$22.50 vs $3/$15 per M), so the economic case for lazy-load is direct.

- [Reddit on 1M Opus 4.6 in Claude Code](https://www.reddit.com/r/ClaudeAI/comments/1r70xa9/1m_context_window_for_opus_46_is_finally/)
- [Anthropic — effective context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents)
- [Anthropic — multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system) (parallel 200 k subagents beat single 1M agent by 90.2 %)

**Vs your harness:** Advisor on persistent Opus + Planning on Opus worker is correct architecture. You're not in danger of over-loading.

---

## Section B — Subagent Orchestration

### B1. Defining a Rust-aware subagent

Required frontmatter: `name`, `description`. The `description` field controls auto-delegation — write it as a trigger ("Use this agent when…", "Use proactively after…"), not a capability list. The `tools` field is the highest-impact optional field: omitting it inherits *all* tools including Write/Edit. Specify `tools: Read, Glob, Grep, Bash` explicitly for read-only analysis agents. The Markdown body becomes the system prompt — subagents do NOT receive the parent's system prompt, so Rust/cargo conventions must be embedded in the body.

Key fields:
- `model: haiku` (read-only) / `sonnet` (write)
- `tools:` / `disallowedTools:` (denylist alternative)
- `isolation: worktree` (temporary git worktree, auto-cleanup if no changes)
- `permissionMode: plan` (read-only) / `acceptEdits` (write)
- `skills:` preloads at startup

- [Anthropic — sub-agents](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
- [Tembo — Claude Code Subagents practical guide](https://www.tembo.io/blog/claude-code-subagents)
- [PubNub — Best Practices for Claude Code Subagents](https://www.pubnub.com/blog/best-practices-for-claude-code-sub-agents/)

**Vs your harness:** your 12 agents already follow this pattern. No changes needed.

### B2. Parallel subagent dispatch across crates

Canonical mechanism: `isolation: worktree` per subagent, with a file-ownership manifest maintained by the orchestrator. The Task tool was renamed to **Agent** in v2.1.63 (Task alias still works). `run_in_background=true` enables fire-and-monitor dispatch. The most detailed published parallel-dispatch example is [Davis Vaughan's 200-PR Claude Code workflow](https://blog.davisvaughan.com/posts/2026-01-09-claude-200-pull-requests/) — strict 10-concurrency limit, each subprocess writes `status.json` (`fixed`/`failed`/`needs_review`).

- [Anthropic — worktrees](https://code.claude.com/docs/en/worktrees)
- [Reddit — parallel orchestrator with worktrees](https://www.reddit.com/r/ClaudeAI/comments/1s978m3/i_built_a_parallel_agent_orchestrator_for_claude/)

**Vs your harness:** your `worktree-guard.sh` (blocks sudo/symlinks/cross-worktree writes) exceeds community standards. Your polling-loop + status-file architecture maps cleanly onto the Vaughan pattern. No changes needed.

### B3. When to delegate read work to a subagent

Anthropic's heuristic: delegate when read work would "flood your main conversation with search results, logs, or file contents you won't reference again." Community adds three triggers: (1) raw evidence >200 lines, (2) token cost (subagent burns ~10–50× tokens internally, returns <10% to parent), (3) parent needing raw evidence — for the last case, have the subagent *write* a file and return a path, not a narrative summary.

- [Anthropic — sub-agents](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
- [Martin Fowler — Context Engineering for Coding Agents](https://martinfowler.com/articles/exploring-gen-ai/context-engineering-coding-agents.html)

**Vs your harness:** your PMD `memory_search_hybrid` injection in `memory-injection.md` is the canonical version of this. No changes needed.

### B4. Long-running validation (your 26-min `cargo test`)

`run_in_background: true` returns a `shell_id` immediately and notifies on completion. The canonical fix for "exit code occasionally lies" is **always call `BashOutput(shell_id)` and check both the exit-code field and a result-line pattern** (`"test result: ok"` vs `"FAILED"`) — never trust the notification status alone. `BASH_MAX_TIMEOUT_MS` defaults to 600 000 ms (10 min); for a 26-min suite, override to e.g. 2 000 000 (33 min). Threshold from community practice: background only when >30 s AND independent of other work.

- [Zenn — Eliminate build wait times using background bash](https://zenn.dev/ai_eris_log/articles/claude-code-background-bash-20260512?locale=en) (most detailed published treatment)
- [morphllm — Claude Code Hooks (async: true)](https://www.morphllm.com/claude-code-hooks)
- [conclaude — synchronous Stop-hook alternative](https://lib.rs/crates/conclaude)

**Vs your harness:** your CI-watcher (Haiku) polling pattern is the right architecture for a 26-min suite. Worth pinning `BASH_MAX_TIMEOUT_MS` in `.claude/settings.json` if you don't already; check current value.

---

## Section C — Headless Mode (`claude -p`)

### C1. `claude -p` as dispatched worker

Supply task input via stdin (≤10 MB cap, v2.1.128+) or `--append-system-prompt-file`. Capture `session_id` with `--output-format json` for continuations. **Use `--bare`** — it skips auto-discovery of hooks/skills/plugins/MCP/CLAUDE.md, Anthropic says it will become the `-p` default in a future release. Pass MCP explicitly via `--mcp-config <file>`.

For watchdog timeouts: there is **no native watchdog flag**. Outer supervision (`timeout(1)`, `tokio::time::timeout`) is required. `--output-format stream-json` emits `system/api_retry` events with `attempt`, `max_retries`, `retry_delay_ms` — a dispatcher can detect these and impose an outer cap.

Session continuity: `claude -p --output-format json | jq -r '.session_id'`, then `claude --resume <id> -p "..."`.

- [Headless mode docs](https://code.claude.com/docs/en/headless)
- [Common workflows](https://code.claude.com/docs/en/common-workflows)

**No Claude-Code-specific Rust daemon pattern evidence found.** Your Junior daemon on `homeserver` is the most concrete example you'll find — the published patterns are all language-agnostic.

### C2. Stop-hook discipline for headless workers

The `Stop` hook fires once per turn. To block exit, return `{"decision": "block", "reason": "..."}` or exit code 2 with stderr. **Claude Code overrides the stop-hook cap after 8 consecutive blocks without progress**; cap is configurable via `CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`. Always inspect `stop_hook_active` in the JSON input and exit 0 if it's already `true` — this is the documented anti-loop mechanism.

Headless-specific: `PreToolUse` hooks can return `permissionDecision: "defer"`, which exits with `stop_reason: "tool_deferred"` and preserves the pending tool call. Resume via `claude -p --resume <id>`. This is the official HITL-in-headless mechanism.

- [Hooks reference](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [Hooks guide](https://code.claude.com/docs/en/hooks-guide)
- [HN: Claude 4.7 ignoring stop hooks](https://news.ycombinator.com/item?id=47895029) (real-world JSON, exit-code-2 behaviour)

**Vs your harness:** `retro-check.sh` is the exemplary version of this pattern — branch-scoped, time-windowed (30 min for Junior, 60 min for advisor), structured retro enforcement. It's better than anything published.

### C3. Worktree-per-task isolation

Native `--worktree <name>` (or `-w`). Subagents get `isolation: "worktree"` frontmatter. `.worktreeinclude` (gitignore syntax) is the documented mechanism for copying gitignored files (e.g., `.env`, `config/secrets.json`) into new worktrees. Tracked files are never duplicated.

`worktree.baseRef`: `"fresh"` (default, branches from `origin/HEAD`) or `"head"` (local HEAD). Auto-removal on exit if no uncommitted changes / untracked files / new commits; orphaned subagent worktrees swept on startup after `cleanupPeriodDays`.

**`.mcp.json` propagation into worktrees is NOT documented.** Changelog v2.1.63 says "project configs and auto memory are shared across worktrees of the same repository" — implies the main repo root's `.mcp.json` is used regardless of worktree CWD, but explicit resolution behaviour is unspecified. `CLAUDE_PROJECT_DIR` env var (v2.1.139+) is set for MCP stdio servers and gives a stable project root.

- [Worktrees docs](https://code.claude.com/docs/en/worktrees)
- [Claude Code changelog](https://code.claude.com/docs/en/changelog)
- [Parallel vibe coding with worktrees](https://www.dandoescode.com/blog/parallel-vibe-coding-with-git-worktrees)

**Vs your harness:** because your `.mcp.json` is gitignored and host-specific (Ref API key), the worktree propagation question is moot for you — each worktree is on the same host and resolves the same `.mcp.json`. Your `worktree-guard.sh` is more conservative than anything published.

### C4. MCP server wiring in headless mode

`--bare` skips `.mcp.json` auto-discovery; pass MCP explicitly via `--mcp-config`. Several silent-degradation bugs are documented in the changelog and worth being aware of:

| Bug | Fixed in |
|---|---|
| Paginated `tools/list` silently dropped post-page-one tools | v2.1.144 |
| Malformed `.mcp.json` entry silently dropped *other* valid servers | v2.1.141 |
| stdio MCP servers printing non-protocol stdout caused unbounded memory growth | v2.1.132 |
| Subagents failed to inherit dynamically-injected MCP tools | v2.1.101 |
| `${ENV_VAR}` in HTTP/SSE/WebSocket headers not substituted | v2.1.119 |
| POSIX shell parameter expansions (`${var%pattern}`) flagged as missing env vars | v2.1.141 |

`alwaysLoad: true` in a server entry blocks session start until that server connects (5-s timeout). Others connect in background. `--strict-mcp-config` is preserved across `/bg` respawn (v2.1.143).

- [MCP docs](https://docs.anthropic.com/en/docs/claude-code/mcp)
- [Claude Code changelog](https://code.claude.com/docs/en/changelog)

**Anthropic vs community:** official recommendation is `--bare` + explicit `--mcp-config` for CI/scripted calls. Community usually keeps non-bare mode with project-scoped `.mcp.json`, accepting the silent-degradation risk. For your Junior daemon, the official `--bare` + explicit-config path is the safer one.

**No Claude-Code-specific evidence** for the ollama-embedding-unreachable → FTS5 fallback you mentioned. That's third-party MCP server behaviour, not a Claude Code mechanism.

---

## Section D — Detecting "Compiles Clean But Semantically Wrong"

This is the section where the Claude-Code-specific evidence is thinnest. For D1–D4 the tooling path is consistently Clippy `disallowed_methods`/`disallowed_types` + Dylint + audit prompts.

### D1. Same-file sibling conformance

**No Claude-Code-specific audit skill or agent prompt** for intra-file sibling conformance exists. Pattern: ban error-suppressing methods globally via `clippy.toml`, write a Dylint AST lint for true intra-file walks (Clippy operates per-item, not across-items-in-same-file), supplement with a CLAUDE.md rule that asks the model to compare new functions against canonical siblings.

```toml
# clippy.toml
disallowed-methods = [
  {path = "core::option::Option::unwrap_or_default",
   reason = "Use ok_or_else for required fields"}
]
```

The [`sem` CLI](https://www.reddit.com/r/rust/comments/1sdjnc7/a_semantic_diff_in_rust_that_solves_the_missing/) (`sem entities <file>`, `sem context`) gives entity-level diffs that could feed sibling pairs to an audit agent.

- [Clippy lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html)
- [Dylint (Trail of Bits)](https://github.com/trailofbits/dylint)
- [Trail of Bits — write Rust lints without forking Clippy](https://blog.trailofbits.com/2021/11/09/write-rust-lints-without-forking-clippy/)

### D2. `#[async_trait]` Send + Sync + 'static bounds

`#[deny(clippy::future_not_send)]` is the primary compile-time gate. `#[async_trait]` (dtolnay) wraps futures as `Pin<Box<dyn Future + Send + 'async_trait>>` by default — Send is implied. The `?Send` variant is the explicit opt-out. Native `async fn in trait` (Rust 1.75+) does **not** auto-imply Send — this is the footgun. Desugar to `fn foo() -> impl Future<Output = T> + Send + '_` in the trait definition.

For plan-time enforcement (before compile): `static_assert` pattern in a test module:
```rust
fn _assert_send<T: Send>() {}
fn _test_impls() { _assert_send::<MyService>(); }
```

- [baby steps blog — async trait Send bounds intro](https://smallcultfollowing.com/babysteps/blog/2023/02/01/async-trait-send-bounds-part-1-intro/)
- [rust-lang/rust #103854](https://github.com/rust-lang/rust/issues/103854)

**No Claude-Code-specific plan-time enforcement** mechanism exists.

### D3. Diesel + diesel-async connection type discipline

Community pattern: define project-wide aliases `PgPool` and `PgConn<'a>`, never pass `&mut AsyncPgConnection` directly. Primary footgun is importing `diesel::RunQueryDsl` (sync) instead of `diesel_async::RunQueryDsl` (async) — compiles if both are in scope, silently picks the wrong impl. Enforce via `clippy.toml`:

```toml
disallowed-types = [
  {path = "diesel_async::AsyncPgConnection",
   reason = "Use PgConn type alias from db::pool"}
]
disallowed-methods = [
  {path = "diesel::RunQueryDsl::load",
   reason = "Use diesel_async::RunQueryDsl",
   replacement = "diesel_async::RunQueryDsl::load_async"}
]
```

- [bitemyapp — Diesel Async in anger](https://bitemyapp.com/blog/diesel-async-in-anger/) (PgPool / PgConn alias pattern)
- [Rust Users Forum — LoadConnection not satisfied](https://users.rust-lang.org/t/actix-with-diesel-async-loadconnection-is-not-satisfied/128160)

**No Claude-Code-specific guidance.** Community blog only.

### D4. Federation / ActivityPub trust-boundary `.unwrap_or_default()` detection

The `activitypub_federation` crate explicitly states ["never place implicit trust in the security of data received from the Fediverse"](https://docs.rs/activitypub_federation/) and delegates application-level validation entirely to the implementor via `ActivityHandler::verify` and `Object::verify` traits. The crate's own tutorial snippets use `.unwrap()`, which sets a poor precedent for downstream code.

Required fields on remote `Person` (`id`, `preferredUsername`, `inbox`, `publicKey`) must use `.ok_or_else(|| AppError::MissingField(...))?`, not `.unwrap_or_default()`. Static-analysis enforcement:

```toml
# clippy.toml (apply per-module via #![deny(clippy::disallowed_methods)])
[[disallowed-methods]]
path = "core::option::Option::unwrap_or_default"
reason = "Use ok_or_else for required federation fields"
```

Apply `#![deny(clippy::disallowed_methods)]` only in federation-boundary modules to avoid noise elsewhere.

- [activitypub_federation docs](https://docs.rs/activitypub_federation/latest/activitypub_federation/)
- [LemmyNet/activitypub-federation-rust](https://github.com/LemmyNet/activitypub-federation-rust)
- [Sherlock Rust Security & Auditing Guide 2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026) — `serde(deny_unknown_fields)`, missing-field-default-bypass pattern

**No Claude-Code-specific audit skill or hook** for this exists. This is the cleanest single addition: a `clippy.toml` entry plus a per-module deny attribute in your federation modules. It's #5 on the top-5 list above.

---

## Section E — Self-Improving Audit / Agent Calibration

### E1. Precision/recall metrics for AI audits

**No publicly documented Claude-Code-specific harness tracks per-run precision/recall against ground-truth sources** (CI failures, CR findings, post-merge bugs). The closest published numbers are from Anthropic's own alignment-auditing work ([July 2025 paper](https://alignment.anthropic.com/2025/automated-auditing/)):

| Metric | Value |
|---|---|
| Evaluation agent success rate | 88 % |
| Investigator agent win rate (realistic affordances) | 10–13 % |
| Super-agent win rate (best-of-N=10) | 42 % |
| Behaviours discovered (best run) | 52 |

The grader is Claude Sonnet 4. A "fake target" test (told the agent a baseline model was quirky) did not increase the false-positive rate.

The shipping [Claude Code Review product](https://code.claude.com/docs/en/code-review) exposes per-severity JSON (`{"normal": 2, "nit": 1, "pre_existing": 0}`) but no per-axis precision/recall is exposed. `REVIEW.md` in the repo lets you raise the verification bar (e.g., require file:line citation for every Important finding).

Community pattern for high precision: 13-agent CLI ([Ship Safe, r/ClaudeCode](https://www.reddit.com/r/ClaudeCode/comments/1rs64ot/asking_claude_to_find_security_bugs_gives_too/)) where each agent targets exactly one concern. General-purpose "find security bugs" prompts produce too many false positives.

**Vs your harness:** your `harness-audit` skill with `scoring-matrix.md` and `retro-harvest/evals/evals.json` is more rigorous than anything published. Your evaluation-calibration anti-inflation rules (scores cluster 0.60–0.75, never 1.0) are the published-research-grade discipline. **No genuine gap here.**

**Verdict: no Claude-Code-specific evidence** for ground-truth precision/recall tracking specifically tied to CI/CR/post-merge data.

### E2. Eval rubrics for Rust agents

The [`eval-harness` skill on ClaudePluginHub](https://www.claudepluginhub.com/skills/affaan-m-everything-claude-code/eval-harness) formalises `.claude/evals/*.md` + `*.log` + `baseline.json`, three grader types (code/CI, model-as-judge, human), two eval flavours (capability vs. regression), and thresholds (`pass@3 > 90%` capability, `pass^3 = 100%` regression). It's a convention, not a metrics framework.

The [Rust Foundation Mythos harness](https://www.reddit.com/r/rust/comments/1su53vz/standard_library_unsoundness_found_by_claude/) uses per-file parallelism against stdlib — but it's raw API, not Claude Code CLI.

Beyond `cargo check`, the community layers `cargo clippy --deny warnings` → `cargo test` → property-based (`proptest`, `quickcheck`) → code contracts (preconditions/postconditions as `#[cfg(test)]` runtime asserts).

**Vs your harness:** your `evals.json` assertion-based rubric is the formalised version of this. Your scoring matrix has weights validated against the 2026-05-09 trim outcome — that's calibration with empirical backing. Better than the published `eval-harness` skill.

**No published open-source `evals/` directory for Rust + Claude Code combining automated precision/recall + Rust-specific oracles.** You may have the most rigorous published instance, if you ever open-source it.

### E3. Lesson / pattern promotion loops

Anthropic's explicit promotion trigger (from [memory docs](https://docs.anthropic.com/en/docs/claude-code/memory)):
> Add to CLAUDE.md when Claude makes the same mistake a second time / a code review catches something Claude should have known / you type the same correction last session / a new teammate would need the same context.

Three-layer funnel:
1. Auto memory (`~/.claude/projects/<project>/memory/MEMORY.md`, ≤200 lines) — Claude-owned, machine-local
2. CLAUDE.md / `.claude/rules/*.md` — human-owned, version-controlled
3. Path-scoped rules (YAML `paths:` frontmatter) — constrain to file types

Community three-stage lifecycle ([youngleaders.tech](https://www.youngleaders.tech/p/how-i-finally-sorted-my-claude-code-memory)): staging (`~/.claude/memory/domain/{name}/`) → promotion (package as plugin/skill) → pointer (memory becomes a pointer to the skill). The [centminmod template](https://github.com/centminmod/my-claude-code-setup) ships a `memory-bank-synchronizer` subagent + `/update-memory-bank` slash command.

**Vs your harness:** your `.claude/rules/` + `lessons/` + PMD MCP with `memory_search_hybrid` is the canonical version of this. Your `memory-injection.md` formalises what most setups do ad-hoc. **No gap.**

### E4. Drift detection in long-lived agents

**No official Anthropic hook or built-in mechanism specifically for drift detection.** Closest official affordances:

| Hook | Use |
|---|---|
| `ConfigChange` | Fires on `.claude/settings.json` / rules-file change |
| `FileChanged` | Watch specific rule files |
| `InstructionsLoaded` | Audit which rules loaded |
| `SessionStart` | Run a startup script against current HEAD |
| `PostToolUse` on Write/Edit | Re-validate rule-file assertions after every edit |

Community pattern: `SessionStart` command-hook that (1) greps function names mentioned in `.claude/rules/*.md`, (2) checks each exists via `grep -r` or `cargo check`, (3) emits warnings as `additionalContext`. The [dosu.dev pattern](https://dosu.dev/blog/how-to-catch-documentation-drift-claude-code-github-actions) runs `claude-code-action` on PR merge against a CLAUDE.md code-to-docs mapping (~$0.50–$2.00 per run at `--max-turns 15`).

Code Review docs note: ["If a PR changes code and makes a CLAUDE.md statement outdated, Claude flags that the documentation needs to be updated."](https://code.claude.com/docs/en/code-review) — applies to CLAUDE.md only, not arbitrary rules files.

**Vs your harness:** 15 ADRs + 24 rules files is a lot of surface area to drift. A `SessionStart` hook that greps for rule-file references against HEAD is a low-cost addition. **No Claude-Code-specific automated drift detector for Rust rule files specifically exists** — this would be a custom build.

---

## Section F — Token Economy

### F1. Prompt-cache TTL

Default 5-minute TTL for API keys / Bedrock / Vertex / subagents. Pro/Max subscriptions get 1-hour automatically. **API users opt in via `ENABLE_PROMPT_CACHING_1H=1` (or beta header `anthropic-beta: prompt-caching-2024-07-31`).**

Cost ratios (Sonnet 4.6):

| | Cost/M tokens |
|---|---|
| Base input | $3.00 |
| Cache hit/read | $0.30 (10 % of base) |
| 5-min cache write | $3.75 (125 % of base) |
| 1-hour cache write | $6.00 (200 % of base) |

A cache hit is 10 % of base input cost; a 5-minute miss costs 12.5× more than a hit on the same context.

**Critical: subagents ALWAYS use 5-minute TTL with no override available.** Confirmed by Boris Cherny (Claude Code lead) on [HN](https://news.ycombinator.com/item?id=47740756). Parent may have 1-hour warm cache; spawned subagent starts cold with 5-minute window.

Patterns:
1. **`ENABLE_PROMPT_CACHING_1H=1`** — 2× write premium pays for itself after one missed 5-minute window
2. **`DISABLE_AUTOUPDATER=1`** — prevents background CC updates from invalidating system-prompt cache layer
3. No model switching mid-session — invalidates entire cache prefix
4. Cache-invalidating events: CC version upgrade, MCP server connect/disconnect, tool deny-rule changes
5. Community "keepalive ping" pattern (~5 pings → ~24 min buffer) — niche

- [Claude Code prompt-caching](https://code.claude.com/docs/en/prompt-caching)
- [Anthropic prompt-caching API](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching)
- [Boris Cherny on subagent 5-min TTL](https://news.ycombinator.com/item?id=47740756)
- [notdiamond cost analysis](https://www.notdiamond.ai/blog/how-to-reduce-claude-code-costs-without-sacrificing-output-quality)

**Vs your harness:** this is the single biggest concrete gap. Your Junior daemon poll cycles routinely cross the 5-minute mark. `ENABLE_PROMPT_CACHING_1H=1` is #1 on the top-5 list.

### F2. What blows up token budgets in Rust + Claude Code

Rust-specific inflators not present in Python/TypeScript:
1. **Macro-expanded type errors** — a type mismatch with deep generic chains (`tokio::sync::RwLock<Arc<dyn SomeService<Error = MyError<InnerError>>>>`) produces 50–200 lines per error
2. **Generated files repeatedly read** — `schema.rs` (Diesel), `build.rs` output, `prost` protobuf stubs, `sqlx` offline query data
3. **`Cargo.lock` diffs** — see F3
4. **Semantic search hitting struct/impl defs multiple times across the workspace**

Community measurements:
- [RTK (Rust Token Killer)](https://iamjeremie.me/post/2026-04/consumming-less-tokens-in-claude-code/) — ~89 % savings across 7000+ commands; `cargo test` compressed from 155 lines to 3 lines (failures only)
- [sqz / sqz-cli](https://www.reddit.com/r/ClaudeCode/comments/1skf4ml/i_built_a_tool_that_turns_repeated_file_reads/) — SHA-256 content cache, 13-token inline reference on repeat read; 86 % fewer tokens on file-heavy tasks
- Semantic search MCP — ~40 % context reduction vs. grep

For Diesel `schema.rs` specifically: add to `.claudeignore`, summarise structure in a path-scoped CLAUDE.md rule, and let Claude `grep` against `schema.rs` rather than reading it whole.

**Vs your harness:** your cargo-output rules already do most of this. RTK/sqz would be marginal additions. Not in the top 5.

### F3. `Cargo.lock` and lockfile churn

A 200+ transitive dep `Cargo.lock` is 5 000–15 000 lines; one direct-dep bump cascades. Claude Code has no built-in suppression. Mitigations:

1. **`.gitattributes`** `*.lock diff=hidden` (+ `~/.gitconfig` `[diff "hidden"] command = /bin/true`)
2. **CLAUDE.md rule:** `Never read Cargo.lock directly. Use 'git diff --stat Cargo.lock' to see if it changed, then 'cargo tree --depth 1' to verify direct deps.`
3. **[`git-prism`](https://dev.to/mikelane/teaching-claude-to-stop-reaching-for-git-diff-git-prism-v070-4nel)** — Rust CLI (`cargo install git-prism`), registers as PreToolUse hook, intercepts `git diff` calls, returns structured JSON manifests with per-file metadata, function-level diffs, blast-radius scores; default 8 192-token cap with progressive trimming (full → signatures-only → bare)

- [thepacketgeek — hide Cargo.lock diff](https://thepacketgeek.com/rust/hide-cargo-lock-diff/)
- [r/ClaudeCode — stopped a 35 % context munch on commits](https://www.reddit.com/r/ClaudeCode/comments/1r39oxv/stopped_a_35_context_munch_on_commits/)

**Vs your harness:** the cheapest fix is the gitattributes line + a CLAUDE.md sentence. `git-prism` is a fuller solution if you want function-level diffs in your CR-triage gate. Honourable mention, not top-5.

---

## Section G — Tooling Integration

### G1. `cargo expand` / `audit` / `deny` / `machete` / `udeps` / `nextest`

**`cargo nextest`** has the most developed Claude Code integration. The [MCP Market skill](https://mcpmarket.com/tools/skills/cargo-nextest-rust-test-runner) (January 2026) installs guidance for isolated-process execution, complex test filters, and CI pipelines with auto-retry. Key advantages over `cargo test`:

- `--message-format libtest-json` / `libtest-json-plus` — machine-parseable
- `cargo nextest list --message-format json` — JSON list of test names; `nextest-metadata` crate is the canonical parser
- `--retries N` (or `NEXTEST_RETRIES`) — per-test retry; flaky tests classified separately, exit code stays 0 if all eventually pass
- Per-test exponential-backoff in `.config/nextest.toml`
- `--rerun` — reruns only previously-failed tests
- Isolated processes — better failure-mode isolation than threaded `cargo test`

This is #2 on the top-5 list.

**`cargo machete`** has a [Smithery skill](https://smithery.ai/skills/laurigates/cargo-machete) encoding a five-step workflow: verify install → run `--with-metadata --workspace` → classify (real / proc-macro FP / build.rs-only) → apply `--fix` or `# machete:ignore` → verify with `cargo check --all-targets`. Skill notes: machete requires stable Rust; `cargo +nightly udeps` is more accurate but slower and nightly-only.

**`cargo audit`, `cargo deny`, `cargo expand`, `cargo udeps`** — no Claude-Code-specific skills, MCP wrappers, or hook patterns. Invoked via plain `Bash` from CLAUDE.md-instructed workflows. A [PostToolUse hook pattern](https://www.linkedin.com/pulse/stop-asking-claude-remember-format-test-your-code-use-a-j-geddes-emzmc) auto-runs `rustfmt` + `cargo test` on edit; same pattern adapts to `cargo audit` / `cargo deny` post-edit.

**No Claude-Code-specific evidence** for `cargo expand` integration in agent workflows.

### G2. `rust-analyzer` via MCP

The community answer is **[zeenix/rust-analyzer-mcp](https://github.com/zeenix/rust-analyzer-mcp)** (v0.2.0, Aug 2025, `cargo install rust-analyzer-mcp`). Stdio MCP, 10 tools:

`rust_analyzer_symbols`, `_definition`, `_references`, `_hover`, `_completion`, `_format`, `_code_actions`, `_diagnostics`, `_workspace_diagnostics`, `_set_workspace`.

`.mcp.json` config:
```json
{
  "mcpServers": {
    "rust-analyzer": { "command": "rust-analyzer-mcp" }
  }
}
```

Requires `rustup component add rust-analyzer`. Known limitation: code actions may return empty arrays before indexing finishes.

Alternative: [`mcp-language-server`](https://github.com/isaacphi/mcp-language-server) (Go, generic LSP-to-MCP) — documented in [Build with Naz tutorial](https://www.youtube.com/watch?v=7iLMdNc-zOs). Custom slash commands like `/ra-rename` can wrap specific tools.

Confirmed by [r/rust thread (June 2025)](https://www.reddit.com/r/rust/comments/1l3gklx/is_there_a_good_rustanalyzer_mcp_out_there/). No Anthropic-official rust-analyzer MCP guidance exists.

This is #3 on the top-5 list — it adds LSP-grade semantic navigation that `grep`/`Read` cannot match, particularly for D1 sibling-conformance and D2 trait-bound work.

### G3. `bacon` as background compile loop

Community practitioners run `bacon` in a separate terminal pane while Claude Code operates in another — not via Claude Code's `run_in_background`. The [Build with Naz tutorial](https://www.youtube.com/watch?v=7iLMdNc-zOs) shows two parallel bacon instances (one `nextest`, one docs) running for developer visibility while Claude Code is the "coding buddy."

**No documented Claude-Code-specific pattern for piping bacon's structured output into an agentic loop.** Claude Code v1.0.71 added `run_in_background` + `/bashes` + `Kill Bash` but the alternative tmux + tmux-MCP approach is what most published workflows use. The closest equivalent in Claude Code is a PostToolUse hook that fires `cargo check` / `cargo nextest run` after each edit.

**Vs your harness:** your CI-watcher + impl-task polling loop is the architectural equivalent. Not a gap.

### G4. CodeRabbit / Greptile

**CodeRabbit has first-class Claude Code integration.** The canonical workflow ([juanma.codes](https://juanma.codes/2025/09/25/coderabbit-cli-catch-issues-locally-before-you-open-a-pr/), [CodeRabbit CLI](https://coderabbit.ai/cli)):

1. Claude writes/edits
2. `git add -A`
3. Inside Claude: `coderabbit review --plain --type uncommitted` (or `--prompt-only` for LLM-optimised output)
4. Claude applies fixes
5. Repeat → commit

`--prompt-only` formats output as `(file, line, severity, suggestion)` designed for LLM parsing.

**CodeRabbit auto-ingests `**/CLAUDE.md`** ([docs](https://docs.coderabbit.ai/knowledge-base/code-guidelines)) — case-sensitive, directory-scoped (a `src/payments/CLAUDE.md` applies only to that subtree). Your 24 rules files and CLAUDE.md become the reviewer's spec without duplication.

`.coderabbit.yaml` path-based instructions complement this:
```yaml
reviews:
  path_instructions:
    - path: "crates/lemmy_apub_objects/**"
      instructions: |
        Focus on HTTP Signature verification, replay attack vectors,
        and JSON-LD context handling. Flag any direct DB access bypassing
        the domain layer.
```

[r/ClaudeAI production team post](https://www.reddit.com/r/ClaudeAI/comments/1mc80q8/how_we_10xd_our_dev_speed_with_claude_code_and/) — 9-step orchestration, CodeRabbit does line-by-line PR review after Claude opens the PR, custom Claude slash command reads CR comments and addresses them; reported 98 % production-ready before human review.

This is #4 on the top-5 list — closes your existing CR-triage gate's loop.

**Greptile:** no Claude-Code-specific integration evidence found.

---

## Section H — Federation / ActivityPub / Lemmy-Fork Specifics

This is the section with the least Claude-Code-specific evidence anywhere in the corpus.

### H1. ActivityPub agent best practices

**No Claude-Code-specific evidence exists.** The `activitypub_federation` crate ([v0.6.5, March 2025](https://github.com/LemmyNet/activitypub-federation-rust)) encapsulates HTTP Signatures, activity dispatching, inbox/outbox handling. The crate's [0.4.0 announcement](https://www.reddit.com/r/rust/comments/11vsfuz/announcing_crate_activitypubfederation_040_major/) states its design goal is to let developers "view federation as just another API."

Recommended constraints for agent work:
1. CLAUDE.md rule: "Do not implement HTTP Signature verification outside `activitypub_federation`'s provided abstractions." Prevents the agent from re-implementing primitives the crate handles internally.
2. Treat the crate as a trust boundary — interact via documented traits (`ActivityHandler::verify`, `Object::verify`) only.
3. Enforce `cargo check` post-edit feedback (you already do this).
4. Fork-friendly governance extensions: implement new `Activity` types and `Actor` traits on top of the crate rather than forking its internals.

**JSON-LD pitfalls** — general Rust pitfall (treating ActivityPub JSON as plain JSON without respecting `@context`); the crate handles this but agent-written deserialization that bypasses the crate's types may silently drop context. No Claude-Code-specific guidance.

### H2. Diesel migration discipline under agent control

The community consensus (transferred from Supabase, no Diesel-specific Claude Code published guidance) is a **human-gated three-phase sequence**:

1. **Block automation:** Add `diesel migration run` to denied commands in `.claude/settings.json`.
2. **Agent writes SQL:** Claude generates `migrations/<timestamp>_<name>/up.sql` + `down.sql`.
3. **Human reviews.**
4. **Human applies** `diesel migration run`.
5. **Agent writes Rust types:** After confirmation, Claude updates `src/schema.rs` (or you run `diesel print-schema > src/schema.rs` as the source of truth) and the model structs.
6. **Cleanup:** Verify `down.sql` with `diesel migration redo`.

`diesel print-schema` as the explicit handoff mechanism prevents the agent from generating types divergent from applied DB state.

[Slop Fork article](https://flori.dev/reads/slop-fork-migrating-python-backend-to-bun-with-claude-code) used 5 parallel Claude Code agents in isolated worktrees for a large migration — schema migrations in a separate worktree from Rust application code, reviewed/applied independently.

**Vs your harness:** your governance-v0 policy of human-applied migrations + agent-written schema is the right pattern. The community published evidence is for Supabase, not Diesel — your setup is the more rigorous one.

[devenv.sh Claude Code integration](https://devenv.sh/integrations/claude-code/) shows `diesel migration run` in a sample `db-migrate` script, but under the community pattern this should be explicitly blocked in permissions.

---

## Section Summary — Evidence Quality

| Section | Claude-Code-specific evidence? | Rust-specific evidence? | Your harness vs published |
|---|---|---|---|
| A1 cargo output | Yes | Yes | Ahead |
| A2 cargo filter hooks | Partial (squeez, conclaude) | Yes | Ahead; squeez complementary |
| A3 compile-error classifier | No published schema | Partial | Approximated by silent-failure-hunter |
| A4 1M context | Yes | Partial | At standard |
| B1 subagent frontmatter | Yes | Partial | At standard |
| B2 parallel dispatch | Yes | Yes | Ahead (worktree-guard) |
| B3 read delegation | Yes | Partial | Ahead (PMD memory) |
| B4 long-running tests | Yes | Yes | At standard; pin `BASH_MAX_TIMEOUT_MS` |
| C1 `claude -p` daemon | No Rust-specific | No | You are the published reference |
| C2 Stop hook | Yes | No | Ahead (retro-check.sh) |
| C3 Worktree isolation | Yes | No | Ahead |
| C4 MCP wiring | Yes | No | At standard |
| D1 sibling conformance | No | Partial (Clippy + Dylint) | Gap, Clippy + Dylint path |
| D2 async_trait bounds | No | Partial (`clippy::future_not_send`) | Gap, compile-time only |
| D3 Diesel conn discipline | No | Partial (community blog) | Gap, Clippy entries |
| D4 Federation `.unwrap_or_default()` | No | Partial | **Top-5 #5** |
| E1 precision/recall | Partial | No | Ahead (calibrated matrix) |
| E2 eval rubrics | Partial | Partial | Ahead (evals.json) |
| E3 lesson promotion | Yes | No | Ahead (PMD + lessons/) |
| E4 drift detection | No automated | No | Gap (custom SessionStart hook) |
| F1 cache TTL | Yes | Partial | **Top-5 #1** |
| F2 token blowout | Partial | Yes | At standard |
| F3 Cargo.lock churn | Partial | Yes | Honourable mention |
| G1 nextest | Yes | Yes | **Top-5 #2** |
| G1 machete | Yes | Yes | Optional |
| G1 audit/deny/expand/udeps | No | Partial | Generic Bash usage |
| G2 rust-analyzer MCP | Yes (community) | Yes | **Top-5 #3** |
| G3 bacon | Partial | Partial | Architectural equivalent exists |
| G4 CodeRabbit CLI | Yes | No | **Top-5 #4** |
| G4 Greptile | No | No | — |
| H1 ActivityPub | None | No | You are the reference |
| H2 Diesel migrations | None (only Supabase analogue) | No | Ahead |

---

## Hard "No Claude-Code-Specific Evidence" Verdicts

These are the categories where genuine community/Anthropic gaps exist, in case you want to publish or fill them:

- **H1**: Claude Code best practices for agents touching ActivityPub code (general Rust patterns only)
- **H2**: Diesel migration agent workflow (Supabase analogue only)
- **D1–D4**: Semantic-bug-detection categories with structured Claude Code skills (Clippy/Dylint route only)
- **A3**: Compile-error-classifier subagent with a structured output schema
- **E1**: Per-axis precision/recall harness specifically for Rust audits
- **E2**: Open-source `evals/` directory for Rust + Claude Code combining all the eval-harness layers with automated metric tracking
- **E4**: Automated drift-detection hook for rule files vs HEAD specifically for Rust
- **F1**: Rust-specific cargo-run scheduling pattern around the 5-min TTL (cargo-aware variant)
- **G1**: `cargo expand`, `cargo audit`, `cargo deny`, `cargo udeps` skills (all generic Bash today)
- **C1**: Rust-specific `claude -p` daemon pattern (yours is the most concrete example)

---

## Sources (master list)

### Anthropic official
- [Sub-agents](https://docs.anthropic.com/en/docs/claude-code/sub-agents) · [Hooks reference](https://docs.anthropic.com/en/docs/claude-code/hooks) · [Hooks guide](https://code.claude.com/docs/en/hooks-guide) · [Headless mode](https://code.claude.com/docs/en/headless) · [Common workflows](https://code.claude.com/docs/en/common-workflows) · [Worktrees](https://code.claude.com/docs/en/worktrees) · [Changelog](https://code.claude.com/docs/en/changelog) · [MCP](https://docs.anthropic.com/en/docs/claude-code/mcp) · [Environment variables](https://code.claude.com/docs/en/env-vars) · [Context window](https://code.claude.com/docs/en/context-window) · [Memory](https://docs.anthropic.com/en/docs/claude-code/memory) · [Prompt-caching (Claude Code)](https://code.claude.com/docs/en/prompt-caching) · [Prompt-caching (API)](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching) · [Code Review](https://code.claude.com/docs/en/code-review) · [Effective context engineering](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) · [Multi-agent research system](https://www.anthropic.com/engineering/multi-agent-research-system) · [Alignment auditing](https://alignment.anthropic.com/2025/automated-auditing/)

### Tools and community projects
- [squeez](https://skillsllm.com/skill/squeez) · [conclaude](https://lib.rs/crates/conclaude) · [git-prism v0.7.0](https://dev.to/mikelane/teaching-claude-to-stop-reaching-for-git-diff-git-prism-v070-4nel) · [RTK](https://iamjeremie.me/post/2026-04/consumming-less-tokens-in-claude-code/) · [sqz](https://www.reddit.com/r/ClaudeCode/comments/1skf4ml/i_built_a_tool_that_turns_repeated_file_reads/) · [cargo-nextest skill](https://mcpmarket.com/tools/skills/cargo-nextest-rust-test-runner) · [cargo-machete skill](https://smithery.ai/skills/laurigates/cargo-machete) · [zeenix/rust-analyzer-mcp](https://github.com/zeenix/rust-analyzer-mcp) · [mcp-language-server](https://github.com/isaacphi/mcp-language-server) · [CodeRabbit CLI](https://coderabbit.ai/cli) · [CodeRabbit CLAUDE.md ingestion](https://docs.coderabbit.ai/knowledge-base/code-guidelines) · [eval-harness skill](https://www.claudepluginhub.com/skills/affaan-m-everything-claude-code/eval-harness) · [dosu.dev drift](https://dosu.dev/blog/how-to-catch-documentation-drift-claude-code-github-actions) · [centminmod CC setup](https://github.com/centminmod/my-claude-code-setup)

### Rust ecosystem
- [Clippy lint configuration](https://doc.rust-lang.org/clippy/lint_configuration.html) · [Dylint (Trail of Bits)](https://github.com/trailofbits/dylint) · [TrailofBits — write Rust lints without forking Clippy](https://blog.trailofbits.com/2021/11/09/write-rust-lints-without-forking-clippy/) · [diesel-async](https://docs.rs/diesel-async) · [bitemyapp Diesel Async in anger](https://bitemyapp.com/blog/diesel-async-in-anger/) · [activitypub_federation docs](https://docs.rs/activitypub_federation/latest/activitypub_federation/) · [LemmyNet/activitypub-federation-rust](https://github.com/LemmyNet/activitypub-federation-rust) · [Sherlock Rust Security Auditing Guide 2026](https://sherlock.xyz/post/rust-security-auditing-guide-2026) · [baby steps — async trait Send bounds](https://smallcultfollowing.com/babysteps/blog/2023/02/01/async-trait-send-bounds-part-1-intro/) · [rust-lang #103854](https://github.com/rust-lang/rust/issues/103854) · [thePacketGeek — hide Cargo.lock diff](https://thepacketgeek.com/rust/hide-cargo-lock-diff/) · [sem semantic diff](https://www.reddit.com/r/rust/comments/1sdjnc7/a_semantic_diff_in_rust_that_solves_the_missing/)

### Community write-ups
- [Tembo subagents guide](https://www.tembo.io/blog/claude-code-subagents) · [PubNub subagent best practices](https://www.pubnub.com/blog/best-practices-for-claude-code-sub-agents/) · [Davis Vaughan 200 PRs](https://blog.davisvaughan.com/posts/2026-01-09-claude-200-pull-requests/) · [Martin Fowler context engineering](https://martinfowler.com/articles/exploring-gen-ai/context-engineering-coding-agents.html) · [Introl CLI reference](https://introl.com/blog/claude-code-cli-comprehensive-guide-2025) · [Zenn — background bash](https://zenn.dev/ai_eris_log/articles/claude-code-background-bash-20260512?locale=en) · [Boris Cherny on subagent TTL](https://news.ycombinator.com/item?id=47740756) · [HN Claude 4.7 stop hooks](https://news.ycombinator.com/item?id=47895029) · [youngleaders.tech memory](https://www.youngleaders.tech/p/how-i-finally-sorted-my-claude-code-memory) · [notdiamond cost analysis](https://www.notdiamond.ai/blog/how-to-reduce-claude-code-costs-without-sacrificing-output-quality) · [Build with Naz tutorial](https://www.youtube.com/watch?v=7iLMdNc-zOs) · [zfhuang99 — 100K lines Rust with CC](https://zfhuang99.github.io/rust/claude%20code/codex/contracts/spec-driven%20development/2025/12/01/rust-with-ai.html) · [parallel vibe coding with worktrees](https://www.dandoescode.com/blog/parallel-vibe-coding-with-git-worktrees)
