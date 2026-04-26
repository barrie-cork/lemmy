# Claude 4.x Model Selection for the Brehon Four-Role Agent Topology

*Date-stamped April 26, 2026. Re-run after Anthropic ships new models.*

***

## TL;DR — Recommendations per Role

**Keep your current assignments for Advisor and Planning (Opus 4.7 is correct).** For Impl, the data supports staying on Sonnet 4.6 at `medium` effort — Sonnet 4.6 already scores 79.6% on SWE-bench Verified (only 8 points below Opus 4.7's 87.6%), Anthropic explicitly positions Sonnet 4.6 at `medium` effort as the right default for agentic coding and tool-heavy workflows, and upgrading Impl to Opus 4.7 raises per-sub-phase cost by 40–75% (more with tokenizer inflation). For BM, drop to Haiku 4.5 — it saves ~67% per BM task with no expected quality loss on read-only git/gh/yq ops. The 1M context window is now GA at flat pricing for both Opus 4.7 and Sonnet 4.6, so there is no cost penalty for the Advisor's long sessions. Prompt caching per `claude -p` invocation is ephemeral (5-minute TTL) but can be extended to 1 hour; your subagent lesson-file reads will hit cache within a session but not across daemon restarts.

| Role | Current | Recommendation | Effort |
|------|---------|----------------|--------|
| Advisor (orchestrator) | Opus 4.7 | **Keep Opus 4.7** | `max` (current) |
| Planning subagent | Opus 4.7 | **Keep Opus 4.7** | `xhigh` (switch from default) |
| Impl subagent | Sonnet 4.6 | **Keep Sonnet 4.6** | Switch to `medium` (from `high`) |
| BM subagent | Sonnet 4.6 | **Drop to Haiku 4.5** | `low` or `medium` |

***

## Section A — Sonnet 4.6 vs Opus 4.7 for the Impl Subagent

### Benchmark gap

Claude Sonnet 4.6 scores **79.6% on SWE-bench Verified** and **59.1% on Terminal-Bench** with a $3/$15 per-million-token price. Claude Opus 4.7 scores **87.6% on SWE-bench Verified** and **69.4% on Terminal-Bench 2.0**, released April 16, 2026. That is roughly a 7–10 percentage-point gap across the two benchmarks most relevant to agentic coding.[^1][^2][^3][^4][^5]

For **pattern-following impl work** — reading 1–3 MIRROR refs from a structured plan, applying the change across 4–6 callsites, running cargo check/clippy/e2e — the question is whether the 8-point SWE-bench gap materialises as real failure modes (DQ pending entries, mid-task aborts, wrong callsite propagation) or gets absorbed by the plan's explicit scaffolding.

### Anthropic's "when to upgrade" guidance

Anthropic's model selection matrix positions Opus 4.7 for *"long-horizon agentic coding, large-scale refactoring, complex systems engineering, advanced research, multi-hour autonomous tasks"* and Sonnet 4.6 for *"code generation, agentic tool use, frontier intelligence at scale"*. Critically, Anthropic's own effort documentation states that **`medium` effort is the recommended default for Sonnet 4.6 on "agentic coding, tool-heavy workflows, and code generation"**. This is the exact workload shape of Impl — implying Anthropic does not expect you to reach for Opus on this class of task.[^6][^7]

Community data corroborates: one widely-cited analysis found that low-effort Opus 4.7 is *"roughly equivalent to medium-effort Opus 4.6 according to Hex's CTO"*, which signals that the whole effort scale has shifted upward — Sonnet 4.6 at `medium` may produce output meaningfully closer to Opus 4.7 at `high` than the sticker benchmarks suggest.[^8]

### Tool-call chain quality

On multi-step tool use specifically, Anthropic reports Opus 4.7 delivers a **14% improvement over Opus 4.6 on complex multi-step workflows while producing a third of the tool errors**. This is the strongest quantitative signal for upgrading Impl if your real failure mode is tool-call errors (e.g., reading a file already in context, running a Bash command whose output was predictable). However, Anthropic also documents that Opus 4.7 **makes fewer tool calls by default by leaning more on reasoning**, and that *"raising effort increases tool usage"*. For an impl task with ~30–50 tool calls, this is a meaningful behavioural difference from Sonnet 4.6 — you would need to set `effort: xhigh` on Opus 4.7 to restore comparable tool-call depth, at significant token cost.[^9][^6]

### Verdict

**Stay on Sonnet 4.6 for Impl.** The plan-driven, MIRROR-ref-following nature of the work is exactly what Anthropic means by "agentic coding tool use" — the target use case for Sonnet 4.6 at `medium` effort. Upgrade to Opus 4.7 only if you observe a sustained DQ-pending-rate per impl task above your baseline, or specific failure classes (multi-callsite propagation errors, Diesel query mispatterns) that Sonnet is consistently getting wrong despite explicit plan scaffolding.

**The metric to measure before changing:** DQ pending entries per impl task, categorised by whether the root cause is reasoning failure or tool-call error. If tool-call errors dominate, Opus 4.7's 3× tool-error reduction is the specific feature you need. If reasoning failures dominate, inspect whether better plan specificity (shorter MIRROR ref ranges, explicit `NOT-build` lists) closes the gap before spending on Opus.

***

## Section B — Effort Levels: What They Actually Do

### Official documentation

Anthropic's effort documentation is authoritative and unambiguous. The five levels are:[^6]

| Level | Behavior | Available On |
|-------|----------|-------------|
| `low` | Minimises thinking; skips thinking for simple tasks; fewest tool calls; most efficient | All supported models |
| `medium` | Moderate token savings; may skip thinking for simple queries; balanced | All supported models |
| `high` (default) | Always thinks on complex tasks; equivalent to omitting the parameter | All supported models |
| `xhigh` | Extended capability for long-horizon work; recommended starting point for coding and agentic tasks | **Opus 4.7 only** |
| `max` | Absolute maximum with no token-spend constraints | Mythos Preview, Opus 4.7, Opus 4.6, Sonnet 4.6 |

Effort is described as *"a behavioral signal, not a strict token budget"*. At lower levels Claude still thinks on hard problems — it thinks less than it would at higher effort for the same problem. Critically, effort affects **all tokens in the response** including tool calls and extended thinking, not just prose length.[^6]

### What `--effort max` does on the Advisor

`max` on Opus 4.7 means *"absolute maximum capability with no constraints on token spending"* — the model engages the deepest possible reasoning and most thorough analysis. For the Advisor's DQ triage and retro authorship, where judgment quality has high downstream leverage, this is appropriate. Anthropic's own guidance warns that `max` *"on some structured-output or less intelligence-sensitive tasks it can lead to overthinking"* — but the Advisor's tasks are judgment-heavy, not structured-output-heavy, so `max` is defensible.[^6]

### Sonnet 4.6 + effort max vs Opus 4.7 default

`max` effort is available on Sonnet 4.6. Whether `Sonnet 4.6 + max` approaches `Opus 4.7 + high` quality is not directly benchmarked by Anthropic in public docs. The indirect evidence is mixed: Sonnet 4.6 at `max` will close some of the reasoning gap by spending more thinking tokens, but the 8-point SWE-bench gap and the 10-point Terminal-Bench gap reflect model weights, not just effort. For cost-sensitive workloads, Sonnet 4.6 at `medium` is the documented sweet spot for agentic coding — not `max`.[^6]

### Opus 4.7: switch Planning from `max` to `xhigh`

Anthropic explicitly states *"start with `xhigh` for coding and agentic use cases"* for Opus 4.7, and notes that `max` *"adds significant cost for relatively small quality gains"* on most workloads above `xhigh`. The Planning subagent, which does the heaviest single-task reasoning but is not a genuinely frontier-difficulty problem, is a candidate to move from `max` to `xhigh`. This would reduce planning token cost while still engaging full adaptive thinking. Reserve `max` for the Advisor, where pattern-recognition across dozens of retros and ADRs is the task.[^6]

***

## Section C — Haiku 4.5: Where It Belongs

### Capabilities and positioning

Claude Haiku 4.5 scores **73.3% on SWE-bench Verified** and achieves **50.7% on computer-use benchmarks** (compared to Sonnet 4.6's 72.5%). Augment's agentic coding evaluation found Haiku 4.5 achieves **90% of Sonnet 4.5's performance** in internal tests. On the τ2-bench agentic tool use framework (retail scenarios), Haiku 4.5 scores 83.2% vs Sonnet 4.5's 86.2%. These scores are for full agentic coding work — for mechanical git/gh/yq operations with hard-scripted verbs, the gap will be smaller.[^10][^11][^12][^13]

Anthropic's model selection matrix explicitly lists Haiku 4.5's target as *"real-time applications, high-volume intelligent processing, cost-sensitive deployments needing strong reasoning, sub-agent tasks"* — and specifically names *"sub-agent tasks"* as a primary use case.[^7]

### The BM subagent is an ideal Haiku candidate

The BM's nine verbs (`bm-status`, `bm-cut`, `bm-push`, `bm-pr`, `bm-poll-cr`, `bm-prp-review`, `bm-triage`, `bm-merge`, `bm-ping`) are each 30–100-line scripts the agent reads and follows literally. The work is: parse YAML, run 4–8 `gh`/`git`/`yq` commands, write a YAML/JSON artifact. Hard-refusal boundaries are encoded in the script, not in the model's judgment. This is exactly the structured, low-reasoning-burden, high-instruction-following use case Anthropic targets for Haiku.

The 200K context ceiling on Haiku 4.5 is not a constraint here — BM tasks read ~10K input tokens and produce ~3K output tokens, well within the standard context window.[^14][^15]

### Advisor polling loop

The advisor's no-change poll cycle (~500 tokens when nothing has changed) is a genuine Haiku candidate: compare two JSON arrays, produce no output if nothing changed. However, the advisor session needs to maintain consistent identity and memory across the full 1M context over hours, and switching the advisor to Haiku for "cheap" turns would require session management complexity (knowing when a turn is cheap vs expensive). The simpler answer: `effort: low` on the Advisor for no-change cycles, if Claude Code exposes per-turn effort overrides. This avoids splitting the session across models.[^6]

### DQ triage when the answer is obvious

If a DQ entry has only two options and the plan is explicit, Haiku could handle the triage. In practice, dispatching a separate Haiku invocation for DQ triage adds latency and orchestration complexity that may not be worth the savings for a solo-dev setup. This is a future optimisation, not an immediate priority.

***

## Section D — 1M Context Window: When Is It Load-Bearing?

### Pricing (April 2026)

As of March 13, 2026, the 1M token context window is **generally available at standard per-token pricing** for Opus 4.7, Opus 4.6, and Sonnet 4.6 — the long-context premium (formerly 2× input, 1.5× output above 200K tokens) has been eliminated. A 900K-token request costs the same per token as a 9K request. **Haiku 4.5 does not have a 1M context option** — it is capped at 200K tokens.[^15][^16][^17][^14]

| Model | Context Window | 1M Premium |
|-------|---------------|------------|
| Opus 4.7 | 1M (standard) | None[^17] |
| Sonnet 4.6 | 1M (standard) | None[^16] |
| Haiku 4.5 | 200K (hard cap) | N/A[^15] |

### Per-role context analysis

**Advisor:** 1M is load-bearing. Holding plan files (~130K), retros, ADRs, recent commit history, and the running conversation across hours-long sessions requires the full 1M window. The previous beta premium made this expensive; at standard pricing it is not a cost lever.

**Planning subagent:** 200K likely suffices. Reading ~50K tokens of design docs, generating ~30K tokens of plan output, and running ~20 tool turns fits comfortably within 200K. The Planning subagent does not need to hold accumulated conversation across multiple plans — it exits after one plan. Setting `model: claude-opus-4-7` with no special context config means the session auto-scales but the practical cost is driven by actual usage, not the window ceiling.

**Impl subagent:** 200K is more than sufficient. ~5K tokens of brief + task section + MIRROR refs, ~30–50 tool calls totalling perhaps 15–20K tokens of tool results — this is a 50–80K-token session at most. No 1M context required.

***

## Section E — Subagent Isolation and Prompt-Cache Reuse

### Cache mechanics for `claude -p` invocations

Anthropic's prompt caching uses an **ephemeral cache with a 5-minute TTL** that resets on each hit. There is an extended 1-hour TTL option at 2× the write cost (vs 1.25× for 5-minute writes). The cache is **session-scoped and per-machine** — it is not persistent across process restarts. Each `claude -p` invocation starts a fresh conversation context, but the cache server-side persists as long as the content hash matches and the TTL has not expired.[^18][^17][^19][^20]

**What this means for your setup:** Prompt cache hits *can* occur across separate `-p` invocations on the same machine, provided the second invocation starts within 5 minutes of the last cache hit from the first invocation and sends an identical prefix. With Junior dispatching a queue of impl-tasks sequentially, task N+1 starting while task N's cache is still warm is plausible for short tasks but unreliable for cargo-heavy tasks (where the 6-minute watchdog is already in play).

### Maximising per-subagent cache hit rate

Anthropic's caching documentation prescribes a clear prefix ordering: **tools → system → messages**, and recommends placing static content (tool definitions, system instructions, lesson files) at the beginning of the prompt with `cache_control: {type: "ephemeral"}` marking the end of the stable prefix. The recommended pattern for your subagents:[^21][^22]

1. **Mark the system prompt + CLAUDE.md + tool definitions as the first cache breakpoint.** These do not change between invocations. Cache write is 1.25× input price; subsequent reads are 0.10×.
2. **Mark the lessons directory reads as the second cache breakpoint.** Your ~20 `feedback_*.md` files (~300–500 words each, ~10–15K tokens total) are static within a sub-phase. Place a `cache_control` marker after the last lesson file read.
3. **Let the task-specific content (brief + plan section + MIRROR refs) be the dynamic tail.** This changes per invocation and should not be cached.

With this structure, a warm cache across a queue of impl-tasks (assuming <5 minute inter-invocation gaps) would hit cache on ~15K tokens of lessons + tool definitions, saving ~85% on that portion. At Sonnet 4.6 rates, that is ~$0.045 saved per impl task, or ~$0.45 across 10 tasks per sub-phase.

For the 6-minute watchdog constraint: since each impl task starts a fresh `-p` session, and cargo-check can run 30–60 minutes cold, your best cache-hit scenario is on the system prompt + lessons prefix *within* a single task's multi-turn conversation, not across tasks. Within a single `-p` session, Claude Code automatically manages cache breakpoints so the CLAUDE.md and tool definitions stay cached across all turns.[^20][^23]

### 1-hour TTL option

If tasks run sequentially with gaps under an hour (but sometimes over 5 minutes), the 1-hour cache write (at 2× the standard 5-minute write price) would preserve cache hits between tasks. At Sonnet 4.6 rates: 5-min write is $3.75/MTok vs cache read at $0.30/MTok. The 1-hour write is $6/MTok but allows the $0.30/MTok read rate to persist for an hour. Break-even is 2 cache reads per write for 1-hour TTL. With 10 impl tasks reading the same 15K-token lessons prefix, this is clearly profitable.[^17]

***

## Section F — Tool-Use Chains: Model-Specific Differences

### Anthropic's published data

Anthropic's Opus 4.7 announcement directly addresses multi-step tool use: the model delivers a **14% improvement on complex multi-step workflows** vs Opus 4.6 **while producing a third of the tool errors**. This is the most specific Anthropic-sourced signal on agentic tool-use chain quality between model generations.[^9]

Importantly, Anthropic also documents that Opus 4.7 **makes fewer tool calls by default** compared to Opus 4.6, as it reasons more before acting. For the Impl subagent's 30–50 tool calls per task, this means Opus 4.7 at default effort might produce a *shorter* tool-call chain than Sonnet 4.6 — not because it fails, but because it inlines more reasoning. Whether a shorter chain produces correct output depends on whether the remaining tool calls are the right ones.[^24][^6]

The error types that matter most for your Impl tasks are:
- **Callsite propagation errors** (missing a callsite when propagating `InsertForm` fields): these are reasoning failures, where Opus 4.7's deeper context integration should help.
- **Incorrect Diesel query patterns** (wrong join table, wrong snapshot field): these are knowledge retrieval failures from MIRROR refs, where model weights matter less than context quality.
- **Lint escape hatch errors** (`#[allow(...)]` vs `#[expect(...)]`): these are instruction-following failures, where the explicit lesson files should close the gap.

### Sonnet 4.6 on structured plan-following

Third-party analysis finds that Sonnet 4.6 *"performs strongly in structured workflows but may require more explicit scaffolding to maintain consistency under high cognitive load"*. The Brehon plan already provides this scaffolding (MIRROR refs, per-task DoD validation commands, explicit NOT-building lists). The question is whether your current failure rate exceeds the acceptable DQ-pending threshold before adding scaffolding would help.[^25]

***

## Section G — Cost per Sub-Phase: Three Configurations

### Pricing baseline (Anthropic official, April 2026)

| Model | Input | Output | Cache Read | Cache Write (5 min) |
|-------|-------|--------|------------|---------------------|
| Opus 4.7 | $5.00/MTok | $25.00/MTok | $0.50/MTok | $6.25/MTok |
| Sonnet 4.6 | $3.00/MTok | $15.00/MTok | $0.30/MTok | $3.75/MTok |
| Haiku 4.5 | $1.00/MTok | $5.00/MTok | $0.10/MTok | $1.25/MTok |

Source: Anthropic official pricing docs, confirmed by multiple third-party cross-checks.[^26][^27][^17]

**Note on Opus 4.7 tokenizer:** Opus 4.7 uses a new tokenizer that may produce **up to 35% more tokens for the same text**. The per-token price is identical to Opus 4.6, but the effective cost per request may be 0–35% higher. The calculation below shows both flat-rate and inflated scenarios.[^28][^29]

### Sub-phase cost estimate

Token shapes as specified:

| Task | Count | Input (k) | Output (k) |
|------|-------|-----------|------------|
| Planning | 1 | 50 | 30 |
| Impl-task | 10 | 20 each | 10 each |
| BM-task | 5 | 10 each | 3 each |

| Configuration | No Caching | With 60% Cache Hit |
|---------------|------------|-------------------|
| **A: Current** (Opus planning, Sonnet impl+BM) | **$3.48** | **$2.94** |
| **B: Upgrade impl to Opus 4.7** (flat rate) | $4.88 (+40%) | $4.12 (+40%) |
| **B inflated: Upgrade impl + 35% tokenizer** | $6.10 (+75%) | ~$5.20 (+77%) |
| **C: Drop BM to Haiku 4.5** | $3.23 (−7%) | $2.74 (−7%) |

Config B's Impl uplift is $1.40–$2.63 per sub-phase depending on tokenizer inflation. Config C's BM saving is $0.25 per sub-phase (67% reduction on BM alone). At even one sub-phase per day, Config B adds $420–$780/month; Config C saves ~$7.50/month — modest in absolute terms but meaningful directionally for a solo-dev budget.

The real lever is **prompt caching on impl tasks**: with a warm lessons-file cache (60% cache hit rate assumed), Config A drops from $3.48 to $2.94, saving $0.54 per sub-phase purely from caching the static prefix. Maximising this with 1-hour TTL cache writes is the highest-ROI optimisation before touching model assignments.

***

## Appendix: What to Verify Before Changing Config

These are the things that cannot be determined from public documentation and require measurement against your actual workload:

1. **DQ-pending-rate per impl task, by failure type.** Instrument your `decision-queue.json` with a `failure_class` field. If > X% of DQ entries have `failure_class: reasoning` (as opposed to `tool_call_error` or `blocked_on_human`), the Opus upgrade has a quantified motivation.

2. **Actual cache hit rate for `-p` invocations.** Log `cache_read_input_tokens` and `cache_creation_input_tokens` from `response.usage` for 10 consecutive impl tasks. If the inter-task gap (including cargo build time) regularly exceeds 5 minutes, the lessons-file cache is likely cold on each invocation — in which case, upgrade to 1-hour TTL writes.

3. **Opus 4.7 tokenizer inflation on your Rust codebase.** Run `/v1/messages/count_tokens` on a representative impl task context (brief + plan section + MIRROR refs + ~20 lesson files) for both `claude-sonnet-4-6` and `claude-opus-4-7`. The actual inflation on Rust + YAML + Markdown content may be well below the theoretical 35% maximum.

4. **Haiku 4.5 on BM-triage verb.** Before committing, run 5–10 BM-triage tasks against Haiku 4.5 and compare the produced YAML artifacts against your Sonnet 4.6 baseline. Specific failure modes to watch: incorrect severity bucket classification in CodeRabbit YAML parsing, off-by-one in branch naming conventions, and failure to produce a DQ entry when confirmation is required rather than merging directly.

5. **Sonnet 4.6 `medium` vs `high` on Impl.** Anthropic's effort docs position `medium` as the recommended starting point for agentic coding on Sonnet 4.6, but you are currently running `high` (the default). Run a head-to-head on 5 impl tasks at `medium` vs `high`, measuring tool-call count, DQ-pending-rate, and final cargo lint output. If `medium` produces the same success rate, the token saving (estimated 20–30%) is worth taking.

6. **Advisor `max` vs `xhigh` on Planning.** The Planning subagent (not the Advisor) is the candidate for dropping from `max` to `xhigh`. Anthropic says `xhigh` is the recommended starting point for coding and agentic use cases on Opus 4.7; `max` is reserved for genuinely frontier problems. Run 2–3 planning tasks at `xhigh` and review the plan for: completeness of rejected-alternatives section, correctness of cross-PRD dependency analysis, and quality of MIRROR ref selection. If indistinguishable from `max` output, switch permanently.

7. **Junior's 6-minute watchdog vs Opus 4.7's fewer-tool-calls behaviour.** Opus 4.7's documented tendency to make fewer tool calls by reasoning more could reduce the frequency of 6-minute watchdog kills on long cargo builds — or could increase them if the model pauses longer mid-task. Monitor watchdog kill rate before and after any Opus impl upgrade.

---

## References

1. [Claude Sonnet 4.6 Review: Benchmarks & Pricing 2026 | Serenities AI](https://serenitiesai.com/articles/claude-sonnet-46-benchmarks-review-2026) - Claude Sonnet 4.6 scores 79.6% on SWE-bench Verified, hits 59.1% on Terminal-Bench, achieves 72.5% i...

2. [Claude Sonnet 4.6: 79.6% SWE-bench at $3/MTok - NxCode](https://www.nxcode.io/resources/news/claude-sonnet-4-6-complete-guide-benchmarks-pricing-2026) - Claude Sonnet 4.6 delivers near-Opus performance at 5x lower cost. See full benchmarks, pricing brea...

3. [Introducing Claude Opus 4.7 - Anthropic](https://www.anthropic.com/news/claude-opus-4-7) - Our latest model, Claude Opus 4.7, is now generally available. Opus 4.7 is a notable improvement on ...

4. [Claude Opus 4.7 Benchmarks Explained - Vellum](https://www.vellum.ai/blog/claude-opus-4-7-benchmarks-explained) - SWE-bench tests real-world GitHub issue resolution; SWE-bench Pro raises the bar with multi-language...

5. [Claude Opus 4.7: Benchmarks, Pricing, Context & What's New](https://llm-stats.com/blog/research/claude-opus-4-7-launch) - Claude Opus 4.7 scores 87.6% on SWE-bench Verified, 94.2% on GPQA, 1M token context, 3.3x higher-res...

6. [Effort - Claude API Docs](https://platform.claude.com/docs/en/build-with-claude/effort) - At high , xhigh , and max effort, Claude almost always thinks deeply. At lower levels, it may skip t...

7. [Choosing the right model - Claude API Docs](https://platform.claude.com/docs/en/about-claude/models/choosing-a-model) - Claude Sonnet 4.6, Code generation, data analysis, content creation, visual understanding, agentic t...

8. [Claude Code effort levels explained - what Low/Medium/High/Max ...](https://www.reddit.com/r/ClaudeCode/comments/1soqwfl/claude_code_effort_levels_explained_what/) - Updated v3 with further community corrections. TL;DR. There are 5 levels, not 4: low, medium, high, ...

9. [Claude Opus 4.7 leads on SWE-bench and agentic ... - TNW](https://thenextweb.com/news/anthropic-claude-opus-4-7-coding-agentic-benchmarks-release) - Anthropic's Claude Opus 4.7 scores 64.3% on SWE-bench Pro, adds multi-agent coordination and 3x visi...

10. [Claude Haiku 4.5: Features, Testing Results, and Use Cases](https://www.datacamp.com/blog/anthropic-claude-haiku-4-5) - In practice, this means Claude can operate tools like a calculator or a notepad independently, and e...

11. [What is Claude Haiku 4.5? Full Overview, Specs & Benchmarks](https://chatlyai.app/blog/what-is-claude-haiku-4.5) - Use tool calling for agentic workflows by defining clear tool interfaces. Haiku 4.5 excels at multi-...

12. [Claude Haiku 4.5 - Anthropic](https://www.anthropic.com/claude/haiku) - Benchmarks. Haiku 4.5 delivers strong performance and speed across coding, tool use, and reasoning t...

13. [Introducing Claude Haiku 4.5 - Anthropic](https://www.anthropic.com/news/claude-haiku-4-5)

14. [Claude Sonnet vs Haiku 2026: Which Model Should You Use?](https://serenitiesai.com/articles/claude-sonnet-vs-haiku-2026) - Claude Sonnet vs Haiku — pricing, speed, and capabilities compared. Find out which Claude model fits...

15. [Best Claude Models in 2026 — Sonnet vs Opus vs Haiku Compared](https://www.remoteopenclaw.com/blog/best-claude-models-2026) - Best Claude models in 2026 compared by benchmarks and real-world use. Opus 4.6, Sonnet 4.6, Haiku 4....

16. [Claude's 1M Context Window Now GA - No Premium Pricing](https://awesomeagents.ai/news/anthropic-1m-context-ga-opus-sonnet/) - 1M context now GA for Opus 4.6 ($5/$25 per MTok) and Sonnet 4.6 ($3/$15) - no long-context premium ·...

17. [Pricing - Claude API Docs](https://platform.claude.com/docs/en/about-claude/pricing) - Long context pricing. Claude Mythos Preview, Opus 4.7, Opus 4.6, and Sonnet 4.6 include the full 1M ...

18. [Prompt Caching with OpenAI, Anthropic, and Google Models](https://www.prompthub.us/blog/prompt-caching-with-openai-anthropic-and-google-models) - Learn how prompt caching reduces costs and latency when using LLMs. We compare caching strategies, p...

19. [How Prompt Caching Actually Works in Claude Code](https://www.claudecodecamp.com/p/how-prompt-caching-actually-works-in-claude-code) - The Hidden System That Makes Claude Code 80% Cheaper

20. [Mastering Cache Hits in Claude Code - DEV Community](https://dev.to/kitaekatt/mastering-cache-hits-in-claude-code-5648) - Understanding how caching works behind the scenes so you can reduce costs and get faster responses —...

21. [Prompt caching - Anthropic](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching?e45d281a_page=1&wtime=695s)

22. [Prompt caching - Claude API Docs](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) - Claude API Documentation

23. [Claude Code Prompt Caching: One Rule, Every Feature - PrimeLine](https://primeline.cc/blog/prompt-caching) - Prompt caching drives every Claude Code design decision. Here's how prefix matching works and how to...

24. [Claude Opus 4.7: What Changed, Pricing, and API Name](https://www.remoteopenclaw.com/blog/claude-opus-4-7-what-changed-pricing-api-name) - Claude Opus 4.7 launched on April 16, 2026. Here is what changed, the API model name, pricing, conte...

25. [Claude Sonnet vs Opus (2026) - Emergent](https://emergent.sh/learn/claude-sonnet-vs-opus) - Claude Sonnet 4.6 vs Claude Opus 4.6 compared across reasoning, coding, context limits, speed, prici...

26. [Claude API Pricing (March 2026): Opus $5/M Tokens, Sonnet $3 ...](https://www.tldl.io/resources/anthropic-api-pricing) - Claude API pricing 2026: Opus $5.00/M input, $25.00/M output. Sonnet $3.00/$15.00, Haiku $0.25/$1.25...

27. [Claude Pricing Guide 2026: Opus, Sonnet, Haiku Costs with ...](https://pecollective.com/tools/claude-pricing-guide/) - Complete Claude pricing breakdown for April 2026. Opus 4.6 at $5/$25, Sonnet 4.6 at $3/$15, Haiku 4....

28. [Claude Opus 4.7 Pricing: The Real Cost Story Behind the ... - Finout](https://www.finout.io/blog/claude-opus-4.7-pricing-the-real-cost-story-behind-the-unchanged-price-tag) - Claude Opus 4.7 keeps Anthropic’s $5/$25 per million token pricing, but a new tokenizer can raise ef...

29. [Claude Opus 4.7 Price: 2026 API Rates & Subscription - GlobalGPT](https://www.glbgpt.com/resources/claude-opus-4-7-price/) - Claude 4.7 same price? Discover the 35% tokenizer trap. Compare 2026 API rates & SWE-bench gains. Sl...

