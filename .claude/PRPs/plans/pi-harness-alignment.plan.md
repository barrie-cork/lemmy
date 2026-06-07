# Plan — Pi harness alignment + compaction + PMD access

**Status:** EXECUTED — `81f3d2b60` on `governance-v0` 2026-06-07. All streams complete except B3 (deferred until re-run shows B1+B2 insufficient) and open questions 1+3 (B1 field names verified; re-run question left to user).
**Authored:** 2026-06-07 (advisor session, `governance-v0`).
**Trigger to execute:** next convenient session; no hard dependency. Independent of
the m2-late phase lane. Best done when the comparator is otherwise idle.
**Why a separate session:** the diagnosis session was carrying 263 MB of run
forensics + comparator context; this is clean greenfield harness work that
deserves its own context window.

---

## Problem statement

The planning-001 comparator run (GPT-5.5 challenger) exposed that the Brehon
**`.pi/` harness is incompletely wired for headless long-running tasks.** Three
concrete defects, all confirmed by source inspection on the daemon
(`~/.npm-global/lib/node_modules/@earendil-works/pi-coding-agent`):

1. **`.pi/extensions/lemmy-hooks.ts` is not reload-safe.** When Pi's compaction
   fired `ctx.reload()`, the extension's `tool_call` (line ~289) and
   `tool_result` (line ~323) handlers used a **captured `ctx`** that the reload
   invalidated, throwing `"This extension ctx is stale after session replacement
   or reload"`. This cascaded the run death. The fix is mechanical (Pi's own
   error text prescribes it): never use a captured `pi`/`command` ctx after
   `ctx.reload()` / `newSession` / `fork` / `switchSession`; use the ctx passed
   into the handler / `withSession`.

2. **Compaction is fully default and didn't save the run.** Pi's trigger is
   `contextTokens > contextWindow - reserveTokens`. For GPT-5.5 via
   `openai-codex`, Pi reports `contextWindow: 0` (the model is a "custom model
   id" — not in Pi's static catalog), so the threshold math is degenerate and the
   real overflow was detected only *reactively* by the provider's
   context-exceeded error (`compaction_start reason:"overflow"`), by which point
   the input was already over GPT-5.5's true 272K. The default summarization is
   also generic — not tuned for the structure of a Brehon planning task.

3. **Pi has NO PMD/MCP access — by design.** Pi ships with **no built-in MCP**
   (README line 472: *"No MCP. Build CLI tools with READMEs, or build an
   extension that adds MCP support"*). `.pi/settings.json` has no MCP block and
   the settings schema has no MCP key. So every Pi cell (the comparator
   challenger, and any future Pi-driven role) runs **blind to the PMD** — no
   lesson search, no pheromone read, no retro write — while the Claude/Opus
   control has full PMD via `.mcp.json` (`project-memory` HTTP server at
   `100.104.171.26:11435`). **This is an unfair-comparison confound** in the
   comparator AND a capability gap for any production Pi routing.

A fourth, broader task the user requested: **audit ALL remaining `.pi/` files**
(prompts, skills, agents, hook-scripts, PROJECT_CONTEXT) for "Pi-coding working
and aligned" — i.e. that each `.pi/` artifact actually functions under headless
Pi and matches the intent of its `.claude/` counterpart.

---

## Solution statement

Four work-streams, sequenced. Streams A and B are bug/safety fixes (do first).
Stream C is a capability build (the PMD question — has a design fork to resolve).
Stream D is the breadth audit.

### Stream A — Fix lemmy-hooks reload safety (mechanical, do first)

- **File:** `.pi/extensions/lemmy-hooks.ts` (391 lines; 6 registered events:
  `session_start`, `before_agent_start`, `tool_call`, `user_bash`,
  `tool_result`, `session_shutdown`).
- **Defect sites:** the handlers that capture `ctx` and use it after a possible
  reload. Lines ~289 (`tool_call`) and ~323 (`tool_result`) call
  `runHookScript(..., ...ctx.cwd)` / `claudeCompatibleInput(event, ctx.cwd)`
  using the handler's `ctx` — verify whether any of these run across a reload
  boundary. The throw came from `ExtensionRunner.assertActive` reading
  `ctx.cwd` after invalidation.
- **Fix:** audit every `ctx.` access in the 6 handlers; ensure none retains a
  module-scope captured ctx. Where post-reload work is needed, move it into the
  ctx-passing callback. Reference: Pi's `docs/compaction.md` §"Custom
  Summarization via Extensions" and the error string at
  `dist/core/agent-session-runtime.js` (`assertActive` invalidation message).
- **Validation:** re-run a deliberately context-heavy Pi task with
  `keepRecentTokens` lowered to force ≥1 compaction; confirm `compaction_end`
  count == `compaction_start` count (currently 1 start / 0 ends = the wedge
  signature) and no `stale ctx` in `pi-run.log`.

### Stream B — Compaction customization for long planning tasks (DESIGN FORK)

**The user's question: is customizing compaction desirable and valuable for our
long-running planning tasks?** Answer from the diagnosis: **partially — but the
bigger lever is the context-window declaration, not the summary prompt.**

Three sub-decisions:

- **B1 (highest value, CHEAP — verified vs official docs https://pi.dev/docs/latest/models):**
  declare the real context window in a **`models.json`** file
  (`~/.pi/agent/models.json` global, or project `.pi/models.json`). Pi resolves a
  model's `contextWindow` from this config; the bundled-source `contextWindow: 0`
  I saw at runtime is just the undeclared-default sentinel, and the official docs
  confirm a **per-model `contextWindow` field** (docs default 128000 if a model
  entry omits it). **The fix is a config entry, not extension surgery:**
  ```json
  { "id": "openai-codex/gpt-5.5", "contextWindow": 272000, "maxTokens": <N> }
  ```
  With the real 272K declared, the trigger `contextTokens > contextWindow -
  reserveTokens` fires **pre-emptively** (`reason:"threshold"`) instead of
  reactively after the provider's overflow error (`reason:"overflow"`, the wedge
  signature). This is the root fix; any model whose window Pi doesn't know
  defaults to 128000 (too low for some) or behaves degenerately. **Verify the
  exact `models.json` field names at execution** (`id` / `contextWindow` /
  `maxTokens`) against `docs/latest/models` — the file "reloads each time you
  open `/model`".

- **B2 (medium value): tune `keepRecentTokens` / `reserveTokens` for planning.**
  Planning tasks re-read large canonical files (the challenger read
  `e2e/governance.rs` 144×). A larger `reserveTokens` triggers compaction
  earlier (safer for big-context models); a larger `keepRecentTokens` preserves
  more recent planning state verbatim. Current Brehon config:
  `{enabled:true, keepRecentTokens:30000}` (reserveTokens defaults to 16384).
  Propose a planning-tuned profile, e.g. `{reserveTokens: 32000,
  keepRecentTokens: 40000}` — BUT this is moot until B1 (real window) lands.

- **B3 (optional, higher effort): custom `session_before_compact` hook.** Pi
  lets an extension provide a custom summary (`docs/compaction.md` §"Custom
  Summarization via Extensions"). A Brehon-tuned summary could preserve the
  planning-specific structure (the plan template's 20 sections, the §13 task
  list, MIRROR refs, watchpoints) rather than letting generic summarization drop
  them. **Decision gate:** only build B3 if B1+B2 prove insufficient on a
  re-run. The generic summary format (`## Goal / ## Progress / ## Key Decisions
  / ## Next Steps / <read-files>`) is already decent for planning — measure
  before adding a custom hook. The challenger lost its plan to the *reload bug*
  (Stream A), not to bad summarization — so A is the real culprit, and B3 may be
  unnecessary.

**Recommendation:** do B1 (root fix) + B2 (cheap tuning); defer B3 behind a
measured re-run. Customizing the summary *prompt* is the lowest-value lever;
declaring the context window is the highest.

### Stream C — PMD access for Pi cells (CAPABILITY BUILD; design fork)

Pi has no MCP. Two viable patterns (README explicitly names both):

- **C1 — PMD-as-CLI-tool (Pi-native, recommended).** Write a small CLI
  (`scripts/brehon/pmd-query.sh` / `pmd-write.sh`) that curls the PMD HTTP
  endpoint (`http://100.104.171.26:11435/mcp`, Bearer from `.env`), plus a
  README the Pi agent reads (per Pi's Skills pattern). The agent calls it via
  the `bash` tool. Pros: aligns with Pi's philosophy ("build CLI tools with
  READMEs"), no extension complexity, works headless. Cons: the agent must know
  to call it (prompt-level wiring in `.pi/prompts/*` + a `.pi/skills/pmd/`
  README).
- **C2 — PMD-as-extension (API verified vs official docs https://pi.dev/docs/latest/extensions).**
  A `.pi/extensions/pmd.ts` that registers PMD search/write as native Pi tools.
  The API is concrete and confirmed:
  ```typescript
  pi.registerTool({
    name: "pmd_search", label: "PMD Search",
    description: "...", parameters: Type.Object({ query: Type.String() }),
    async execute(toolCallId, params, signal, onUpdate, ctx) {
      const r = await fetch("http://100.104.171.26:11435/mcp",
        { method: "POST", body: JSON.stringify(...), signal: ctx.signal });
      return { content: [{ type: "text", text: ... }] };
    },
  });
  ```
  Extensions make HTTP calls via standard `fetch(url, {signal: ctx.signal})`
  (abort-aware). Pros: surfaces as first-class tools the LLM sees in the system
  prompt; the `ctx.signal` pattern is inherently reload-safe (no captured ctx).
  Cons: more code than C1; PMD is an MCP HTTP server (JSON-RPC, possibly
  stateful initialize handshake) — the extension may need to speak MCP, not plain
  REST.

**Decision gate:** the official docs make **C2 more attractive than first
thought** — `registerTool` + `fetch(signal)` is clean and reload-safe. Still,
**start with C1** (lower risk, philosophy-aligned, no MCP-protocol handshake to
implement). The PMD HTTP API shape is the same one `.mcp.json` uses — verify the
MCP HTTP protocol (JSON-RPC over HTTP, initialize/session) is curl-able from a
shell script, or whether a thin Node client (→ C2) is needed.
**Confound note for the comparator:** until PMD access exists for Pi, every
comparator result carries an asterisk — the challenger plans without lessons the
control had. Either (a) wire C1 before the next experiment, or (b) explicitly
document the PMD-blind handicap in each run's meta.json and routing rec.

**Also wire (cheap, do regardless):** confirm the daemon `.env` `MINIMAX_API_KEY`
(added 2026-06-07) is sourced by the comparator runner; document that Pi reaches
MiniMax via the native `minimax` provider (the `/anthropic` compat shim falsely
reports insufficient-balance — see
`.claude/PRPs/handovers/comparator-minimax-auth-2026-06-07.md`).

### Stream D — Full `.pi/` harness alignment audit

For EACH `.pi/` artifact, confirm it functions under headless Pi and matches its
`.claude/` intent. Inventory (48 files):

- **`.pi/settings.json`** — provider/model/thinking/compaction/extensions/skills/
  prompts. Audit: does `defaultModel: gpt-5.5` match the comparator intent? Is
  `compaction` tuned (Stream B)? Should `enabledModels` be set?
- **`.pi/PROJECT_CONTEXT.md`** (11.6 KB) — the auto-injected context (via
  AGENTS.md). Audit: is it current vs CLAUDE.md? Not stale on phase/branch refs?
- **`.pi/extensions/lemmy-hooks.ts`** — Stream A. Also audit: do the 4
  hook-scripts it shells out to (`worktree-guard.sh`, `observation-capture.sh`,
  `pre-phase-audit.sh`, `retro-check.sh`) exist + work headless?
- **`.pi/prompts/*` (33 files)** — the `/name`-invoked templates. Audit: does
  each have a working `$ARGUMENTS` binding? Do the prp-* and bm-* prompts mirror
  their `.claude/commands/` counterparts? Spot-check prp-plan.md (the one the
  comparator used — already known-good at 738 lines) and 2-3 others.
- **`.pi/skills/* (12 SKILL.md)`** — the skill commands (`enableSkillCommands:
  true`). Audit: do they resolve? Is `planning/SKILL.md` (the Junior path)
  correctly distinct from the direct prp-plan prompt?
- **`.pi/agents/{bm-pi,ci-debug}.md`** — only 2 agents. Audit: are these the
  intended Pi agent set, or should there be an `eval.md` (the other session is
  building it)? Coordinate.
- **`.pi/rust-analyzer-check.jsonl`** (670 KB) — stale artifact? Should it be
  gitignored like the comparator traces?

**Method:** read each, diff intent against the `.claude/` sibling, list
gaps/staleness/breakage in an audit report at
`.claude/PRPs/reports/pi-harness-audit.md`. Fix mechanical issues inline; file
DQ/follow-ups for judgment calls.

---

## Sequencing

1. **Stream A** (reload-safety) — blocks reliable long runs; do first.
2. **Stream B1** (context-window declaration) — the root overflow fix.
3. **Stream C1** (PMD-as-CLI) — removes the comparator confound; enables the
   real first experiment re-run.
4. **Stream B2** (compaction tuning) — cheap, after B1.
5. **Stream D** (breadth audit) — can run in parallel with C; surfaces more work.
6. **Stream B3** (custom compaction summary) — ONLY if a measured re-run shows
   B1+B2 insufficient.

## Validation gates

- A: compaction round-trip (`compaction_start == compaction_end`, no stale-ctx).
- B1: a Pi task that exceeds the window triggers `compaction_start
  reason:"threshold"` (pre-emptive) NOT `reason:"overflow"` (reactive).
- C1: a Pi `bash`-tool call to `pmd-query.sh` returns lesson hits.
- D: audit report committed; mechanical fixes done; judgment calls filed.

## Doc-verification note (2026-06-07)

All bundled `docs/*.md` claims were cross-checked against the official docs at
**https://pi.dev/docs/latest** and matched (compaction formula, settings schema
with no MCP key, `session_before_compact` hook, "No MCP" stance). **Bundled docs
on the daemon are trustworthy as primary;** official is the tiebreaker for
config-surface specifics. The one thing official added: the **`models.json`
per-model `contextWindow` override** (B1 above), which the bundled runtime only
exposed as the `0` sentinel. Official pages to re-read at execution:
`/docs/latest/{compaction,models,extensions,settings,custom-provider}`.

## Open questions (resolve at execution time)

1. **B1 field names** — RESOLVED to a config approach: declare `contextWindow` per
   model in `models.json`. Verify exact field names (`id`/`contextWindow`/
   `maxTokens`) + global-vs-project precedence against `/docs/latest/models`.
2. **C protocol** — is the PMD MCP HTTP endpoint plain JSON-RPC curl-able from a
   shell script (→ C1), or does it need a stateful MCP handshake
   (initialize/session) better handled by a `registerTool` + `fetch` extension
   (→ C2)? This is the C1-vs-C2 decider.
3. **Comparator re-run** — once PMD access lands, should planning-001 be re-run
   with PMD access for a clean (non-confounded) result, or kept as-is with the
   handicap documented? (n=1 either way; the overflow finding stands regardless.)

## Related

- `.claude/PRPs/handovers/comparator-minimax-auth-2026-06-07.md` — MiniMax auth +
  the planning-001 outcome + the lemmy-hooks bug + code-gates bug.
- `.claude/PRPs/specs/pi-model-comparator.spec.md` — comparator architecture.
- `.claude/PRPs/comparator/runs/planning-001/` — the run that surfaced all this.
- `project_minimal_pi_eval_environment_banked.md` (PMD) — the standing-env plan;
  this harness work is its prerequisite.
- Pi docs on daemon: `~/.npm-global/lib/node_modules/@earendil-works/pi-coding-agent/docs/{compaction,settings,models,usage}.md` + `README.md`.
