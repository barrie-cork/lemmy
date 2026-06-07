# PMD pheromone relevance-labeling — options analysis (decide-later design doc)

## 1. Authority + provenance

User-initiated 2026-06-07 (Opus advisor session), as the follow-up to the shipped
read-pheromone system (`mcp-pmd-read-pheromone.md`) + its per-event telemetry
(`read_events` table, MCP commit `a59a2e0`). The question: *the telemetry lets us
tune pheromone weights from usage SHAPE, but true optimization needs a relevance
label — was the surfaced memory actually useful? What are the options for getting
that label?*

This is a **decide-later design doc**, not a patch contract. No implementation is
authorized. It exists so a future session — triggered by an actual observed
mis-ranking, or by a deliberate choice to invest in self-optimization — can pick
an approach with the trade-offs already worked out. Per the 2026-06-07 decision:
"spec all options, decide later."

**Status of the thing this would improve:** the pheromone weights
(`LAMBDA=0.0231`, `FREQ_NORM=4.6`, `PHEROMONE_WEIGHT=0.5/RRF_K`) are principled
literature defaults (LRFU / FSRS / MAX-MIN Ant System), the system is
bounded-by-design (pheromone cannot override RRF relevance), and **there is zero
observed evidence of mis-ranking** as of this writing. Building a relevance
pipeline before a single bad ranking is observed would be premature optimization.
The recommended default (§7) is therefore "collect data, don't build yet."

## 2. The core problem: attribution

A search surfaces N memories. Later, the agent writes a commit, authors a plan, or
makes a decision. **Which of those N memories (if any) actually helped?** Reading
and acting are disconnected in time and surface. Every option below is, at heart, a
different way to bridge that gap — and they differ mainly in (a) how directly they
observe "usefulness", (b) how much new instrumentation they need, and (c) how noisy
the resulting label is.

A second, harder truth: **the most valuable reads often leave no trace.** A lesson
that stops you from making a mistake produces no commit, no citation, no
re-query — the dog that didn't bark. No passive signal captures these. This caps
the ceiling of every option short of explicit human judgment.

## 3. The five options

### 3.1 Option 1 — Explicit feedback tool (`memory_mark_useful`)

Add an MCP tool the agent calls when a surfaced memory demonstrably informed its
work: `memory_mark_useful(memory_id, query_context)`. Self-reported relevance.

- **Data quality:** highest per-label confidence (an explicit "this helped").
- **Instrumentation:** one new MCP tool; a `useful_marks` table or a column on
  `read_events`.
- **Cost:** near-zero compute.
- **Weakness (fatal as a primary):** relies on the agent remembering to call it.
  Under-reporting is near-certain and *biased* (you mark the dramatic saves, not
  the routine ones). This is the "please rate this interaction" problem — sparse,
  skewed data. Useful only as an *opt-in supplement*, never the backbone.

### 3.2 Option 2 — Citation mining from commits / plans / retros

Passively parse the artifacts the workflow already produces (commits, plan files,
`.claude/PRPs/reports/*.md`, retros) for memory references — explicit
(`memory_id 533`, `feedback_x.md` filename) or fuzzy (title/content n-gram overlap).
Join back to `read_events` by time window: memory surfaced at T, cited at T+Δ →
positive label.

- **Data quality:** high-confidence where a citation exists (a citation is a strong
  positive); but *sparse* — most useful reads are never explicitly cited.
- **Instrumentation:** an offline miner (reads git log + plan/retro files + the
  `read_events` table). No live-path change.
- **Cost:** low; runs as a periodic batch.
- **Weakness:** attribution is fuzzy (overlap ≠ causation); time-window joins are
  noisy; near-zero recall on the "silent save" class. Best as the
  *high-precision-low-recall* leg of a layered design.

### 3.3 Option 3 — Implicit signal: re-query / dwell

Treat the agent's *behavior after a search* as the label, exactly as web search
ranking learns from clicks and abandonment:
- search → immediately followed by a *refined re-query* (overlapping terms) ⇒ the
  first result set was unsatisfying ⇒ **negative**.
- search → followed by *work* (commits, edits, plan writes) with no re-query ⇒
  **positive**.

- **Data quality:** indirect but *abundant* — every search produces a signal.
  Industry-proven (this is how Google/Bing rank).
- **Instrumentation:** needs a **work-event stream** (what the agent did, when)
  correlated with the `read_events` stream. This is the real build cost — a
  session/action log the correlation can join against.
- **Cost:** low compute; moderate engineering.
- **Weakness:** "re-query" is noisier in an agent context than in human search (an
  agent re-queries for many reasons); needs a session model. **This is the
  durable foundation** — the abundant passive label that makes real optimization
  possible — if the investment is justified.

### 3.4 Option 4 — Offline counterfactual backtest (no new live signal)

Don't label usefulness at all. Replay the *existing* `read_events` history through
the ranking formula with swept weights, scoring against a **proxy** objective.
Candidate proxy: "minimize the total rank of memories that were *re-read soon
after* the read in question" (re-read = soft relevance: you came back to it). Sweep
`LAMBDA × FREQ_NORM × PHEROMONE_WEIGHT`, pick the grid point that best satisfies the
proxy.

- **Data quality:** only as good as the proxy; "re-read soon" is a weak relevance
  label. Tunes for self-consistency, not true usefulness.
- **Instrumentation:** **none** — needs only `read_events` history + a script.
- **Cost:** minutes of compute; small one-time script.
- **Weakness:** the proxy can be gamed by the formula it's tuning (a real risk —
  the objective and the thing being optimized share structure). **But it is the
  cheapest possible next step and de-risks everything else:** run it the moment
  there are a few hundred events to learn whether the weights are even in the right
  ballpark, before investing in live labeling.

### 3.5 Option 5 — LLM-as-judge relevance labeling

Periodically, a separate cheap model scores `(query, surfaced_memories,
what-the-agent-did-next)` for "was this memory relevant to that query?", generating
dense labels at scale.

- **Data quality:** dense; handles the fuzzy "did it help" judgment a heuristic
  can't.
- **Instrumentation:** a batch judge job reading `read_events` + work context;
  a `relevance_label` store.
- **Cost:** real — every (or sampled) search judged = ongoing token spend.
- **Weakness:** the judge can be wrong; **circularity risk** if the same model
  family judges its own retrieval (it may rate its own surfacing as good). The
  "dense labels, cost acceptable" upgrade — only after a cheaper option proves
  the weights need moving.

## 4. Comparison

| Option | New instrumentation | Data quality | Recall on "silent saves" | Cost | When it fits |
|---|---|---|---|---|---|
| 1 explicit tool | 1 MCP tool + table | high/label, biased/coverage | ~0 | ~0 | opt-in supplement only |
| 2 citation mining | offline miner | high precision, low recall | ~0 | low | high-confidence leg of a layered design |
| 3 re-query/dwell | **work-event stream** | indirect, abundant | low | low-med | durable foundation, if justified |
| 4 offline backtest | **none** | proxy-only | n/a | minimal | immediate de-risk; do first |
| 5 LLM-judge | batch judge + store | dense | low-med | ongoing $ | dense-label upgrade |

## 5. Recommended layering (when the trigger fires)

These are not mutually exclusive. The strong play is a **layered pairing**:

1. **Option 4 first** — zero instrumentation, runs on existing data, tells you
   whether the weights are even in the right ballpark. De-risks everything else.
2. **Option 3 as the foundation** — the abundant passive label; build the
   work-event stream + correlation if Option 4 shows the weights need real tuning.
3. **Option 2 as the high-precision leg** — sparse but strong positives, cheap to
   add alongside Option 3.
4. **Option 5 only if dense labels are needed** and the cost + circularity are
   acceptable.
5. **Option 1 actively avoided as a primary** — self-report decay makes it the
   weakest backbone despite seeming cleanest. Fine as an opt-in extra.

## 6. The trigger (when to revisit this doc)

Build NOTHING from this doc until one of:
- An **observed mis-ranking**: a search surfaces an obviously-wrong order and the
  `read_events` counterfactual (`rank_with` vs `rank_without`) shows pheromone
  caused or worsened it.
- The `pmd-pheromone-tune.ps1` usage-shape signals show a **gross mis-tuning**
  (weight never reorders / always reorders with large shifts; frequency term never
  saturates; observed re-read intervals off the half-life by an order of magnitude).
- A **deliberate decision** to invest in PMD self-optimization as a feature in its
  own right.

Absent a trigger, the per-event telemetry simply accumulates — that is the correct
state. The weights are reasonable defaults until proven otherwise.

## 7. Recommended default (this session's decision)

**Collect data, defer labeling.** The `read_events` telemetry (shipped) is the
foundation all five options build on; let it accumulate. Run Option 4 (offline
backtest) as the first move *if and when* a trigger fires — it is the cheapest and
de-risks the rest. Do not build a live relevance-labeling pipeline on weights that
have no observed problem.

## 8. Explicitly out of scope

- Any implementation. This is analysis only.
- Changing the shipped pheromone formula or weights.
- A human-rating UI (no human-in-the-loop labeling surface is contemplated; the
  agent IS the only "user" of the PMD).

## 9. See also

- `.claude/PRPs/specs/mcp-pmd-read-pheromone.md` — the shipped pheromone system this
  would tune.
- `read_events` table (MCP `project-memory-mcp` commit `a59a2e0`) — the per-event
  telemetry every option here consumes.
- `.claude/tools/pmd-pheromone-tune.ps1` — the usage-shape analyzer whose caveat
  section points here; its gross-mis-tuning signals are a §6 trigger.
- `.claude/tools/pmd-pheromone-monitor.ps1` — the 24h health monitor (events_total /
  events_reordered columns feed §6's trigger detection).
- LRFU / W-TinyLFU / FSRS / MAX-MIN Ant System — the literature the current weights
  derive from (see `mcp-pmd-read-pheromone.md` §12).
