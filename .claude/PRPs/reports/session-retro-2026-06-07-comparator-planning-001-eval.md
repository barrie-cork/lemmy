# Session retro — 2026-06-07 — comparator planning-001 eval

**Harness:** claude-code (advisor, canonical brehon-fork checkout)
**Session window:** ~2026-06-07T16:00 IST → ~2026-06-07T18:20 IST (~140 min)
**Branch at start:** `276032aa3` (`governance-v0`)
**Branch at end:** `9cfb7cfc3` (`governance-v0`)
**Files touched:** 37 (across both session halves)
**Commits:** 4 explicit this context (`82855a132`, `9cfb7cfc3`, plus `b255de508` + `75c56afd6` from resumed prior-context tail); ~14 total across the full session arc

## TL;DR

Session ran the first end-to-end eval of the Pi model comparator (planning-001: Opus ground-truth vs GPT-5.5 challenger). Six distinct blocking issues were hit and fixed before a clean result emerged: ARG_MAX limit on shell-arg plan content, `finish_reason=length` from 50KB judge prompts, position-bias in routing label despite correct scores, markdown-bold regex miss on `OUTCOME:`, pipefail killing `find` on missing dirs, and `tool_calls` int-vs-dict type error. Final result: `CONTROL_WINS` 26/35 vs 7/35. Top carry-forward: the `.pi/prompts/prp-plan.md` 20-section gate now prevents a repeat of the "DoD stub only" failure that caused the low challenger score.

---

## What surprised us

- **ARG_MAX bites shell-passed plan content at 50KB.** Passing full plan text through Python `-c` string args fails with `Argument list too long` (exit 126). Threshold is OS-dependent (~2MB on Linux/Windows Bash) but a 50KB plan is well past what shell arg transport can handle. The fix (write to temp files, pass paths) is universal but not obvious — this class of bug is invisible until you hit it with real data.

- **MiniMax M3 uses markdown bold in routing labels.** The judge emitted `OUTCOME: **CONTROL_WINS**` rather than `OUTCOME: CONTROL_WINS`. The extraction regex `r'OUTCOME:\s*([A-Z_]+)'` silently returned `UNKNOWN` instead of erroring. The fix (`\*{0,2}` around the value) is tiny but the silent failure was costly — the routing consensus was wrong for an entire pass before detection.

- **Position-bias in routing label vs correct numeric scores.** Pass1 judge called `CHALLENGER_WINS` despite scoring Control higher on all 7 dimensions. Position-swapping caught this correctly (pass2 said `CONTROL_WINS`), but the routing consensus logic needed to be `CONTROL_WINS if either says it` — not `MIXED_ROUTING`. The methodology worked as designed; the surprise was that the bias was this strong (wrong routing *label* despite unanimous numeric scores favoring control).

- **Plan digesting cuts judge cost 6×.** Extracting only §1/§4/§5/§13/§15/§16a sections from 50KB plans before sending to the judge reduced message size from 113985 bytes to ~20000 bytes. Both passes went from `finish_reason=length` (hit 16384 token budget mid-scoring) to `finish_reason=stop` (all 7 dimensions scored). This wasn't in the original design — it emerged as a fix but should be part of the spec.

- **trace.jsonl is 276MB on EliteDesk — not copyable for laptop judge runs.** The token-signals extraction was designed to read the raw trace. In practice, the trace is too large to transfer. The fix (pre-extract on daemon, write `token-signals.json` to results dir, mark `TRACE_OPTIONAL=true` in judge script) works but requires a manual SSH step. Future experiments should automate the pre-extraction as part of the challenger run teardown.

- **GPT-5.5 planning-002 (from memory note) showed the inverse failure:** planning-001 the model dove in and wrote a plan before overflowing (too eager); planning-002 after the reload-safety fix it read conservatively for the full budget and committed nothing (too cautious). The fix for one run broke the other. Context budget tuning for the planning role is genuinely unsolved.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add `plan_digest()` to the comparator spec (`.claude/PRPs/specs/pi-model-comparator.spec.md`) as a mandatory pre-judge step, not a bug fix | Next judge run starts with digesting; no rediscovery of the ARG_MAX / token-limit problem | minor | 2× (ARG_MAX + token-length both trace to same root: raw plan content is too large for inline transport) |
| 2 | Add `teardown_challenger_extract_signals.sh` helper (or a `--extract-signals` flag to `comparator-teardown.sh`) that SSHes to daemon, runs token extraction on the trace, copies `token-signals.json` to the results dir | Eliminates manual SSH step that was this session's biggest friction source | medium | 1× this session; 1× prior (planning-001 v1 had the same gap before the pre-computed file was written) |
| 3 | Add a `--dry-run` flag to `comparator-judge.sh` that shows message sizes + digest char counts without calling MiniMax | Makes "will this stay under budget?" a 2-second check before a billed run | minor | 1× (would have caught both the ARG_MAX and the 113985-byte message before billing) |
| 4 | Update routing consensus logic in `comparator-judge.sh` `synthesize_judge_results()` to use score delta as tiebreaker (`CONTROL_WINS if control_total > challenger_total else CHALLENGER_WINS`) rather than label precedence (`CONTROL_WINS if either says it`) | More principled; handles future cases where both passes say CHALLENGER_WINS correctly | minor | 1× (label precedence worked here but is ad-hoc) |
| 5 | Add `PROJECT_CONTEXT.md` verbosity-reduction flag to the challenger runner for a "raw-capability pass" | Isolates harness-confounded failures (context pressure) from model-capability failures | medium | 2× (both planning-001 and planning-002 hit context pressure) |

## What to carry forward

- **Position-swap + score-based consensus is load-bearing.** The position-swap correctly detected pass1 label bias. Running a single judge pass and trusting the routing label would have reported `CHALLENGER_WINS` despite all numeric evidence pointing the other way. Always run both passes; always use score delta for consensus, not label precedence.

- **Plan digesting is the correct default for any plan that exceeds ~10KB.** Extract §1 (goal), §4 (watchpoints), §5 (complexity), §13 (tasks), §15 (DoD), §16a (stories) — this captures everything the judge needs to score all 7 dimensions. The 10000-char cap per plan is a reasonable starting default; tighten for cheaper models.

- **Pre-write section gate > post-write review.** Adding the 20-section checklist gate to Phase 6 of `.pi/prompts/prp-plan.md` is the single most targeted fix from this session. The challenger wrote only a DoD section because nothing stopped it from calling the write complete. Hard STOP before a line is written is the right intervention point — post-write section-count is a backstop, not the primary gate.

- **`TRACE_OPTIONAL=true` + pre-computed `token-signals.json` pattern is the right architecture for cross-machine experiments.** The judge should be runnable from the laptop without access to EliteDesk artifacts. Pre-extract the signals on the daemon (small JSON, easy to copy or commit) and let the judge operate from that. Don't require the raw 276MB trace.

- **n=1 is a data point, not a verdict.** The MEMORY.md update correctly marks this as n=1. Resist the temptation to read CONTROL_WINS 26/35 vs 7/35 as a conclusive result — the challenger's failure was almost entirely harness (stub plan from missing section gate + context pressure), not model capability. n≥3 with the gate fix applied before attributing the result to model quality.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Plan digest in `comparator-judge.sh` | 30 | 10 | medium | Reduced 113985→19977 bytes; both passes went from `finish_reason=length` to `finish_reason=stop`. 10 min diagnosing root cause before fix. |
| Position-swap methodology | 20 | 0 | low | Worked exactly as designed; caught pass1 labeling bias cleanly |
| `TRACE_OPTIONAL` + pre-computed token-signals | 15 | 20 | medium | 20 min extracting signals via SSH + writing the JSON manually; should be automated |
| ARG_MAX temp-file fix in `minimax-api.sh` | 0 | 25 | high | Entire class of bug (shell arg transport for large content) was invisible until 50KB plan hit it |
| `comparator-eval-report.sh` pipefail guard | 5 | 15 | medium | Script silent-exit-1 from `find` on missing dir; `isinstance` check for `tool_calls`; three separate root causes for one broken script |
| 20-section gate patch to `.pi/prompts/prp-plan.md` | 40 | 0 | none | Direct fix from eval finding; 5 min to write, 0 wasted |
| Memory update (MEMORY.md + project file) | 5 | 0 | none | Routine; linter auto-upgraded frontmatter (expected) |

## Complexity scores (heavy tasks only)

This session had no impl-task Junior dispatches. Complexity scores apply to the script-fix work done inline by the advisor:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| comparator-judge.sh + minimax-api.sh ARG_MAX + digest fix | 2 | 1 (75c56afd6) | ~45 | ~15 |
| eval-report.sh three-bug fix + judge artifact commit | 3 | 1 (82855a132) | ~30 | ~10 |
| .pi/prompts/prp-plan.md 20-section gate | 1 | 1 (9cfb7cfc3) | ~10 | 0 |

All within envelope. No watchdog risk (advisor session, not Junior worker).

## Decisions to revisit

- **Routing consensus logic**: current `CONTROL_WINS if either says it` is conservative for the challenger (makes it harder for challenger to win). The score-delta tiebreaker in "What to change" #4 is more principled but changes the semantics of the metric. Worth discussing before planning-003.
- **planning-002 inverse failure**: the harness fix (reload-safety + keepRecentTokens=40000) changed GPT-5.5 from "too eager, writes stub" to "too cautious, writes nothing". Tuning the context budget for the planning role is an open problem. One option: a `--context-tokens <n>` flag to the challenger runner that lets the cell start with a smaller PROJECT_CONTEXT injection.
- **Should `comparator-spec.md` encode plan digesting as part of §5 "Eval protocol"?** Currently it's a script detail. Elevating it to the spec makes it visible to anyone reading the spec before implementing a new judge.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **ARG_MAX shell-arg transport pattern** → `.claude/lessons/feedback_shell_arg_transport_size_limit.md`: "Never pass content >10KB as Python -c args or shell heredoc string arguments; write to temp files and pass paths. ARG_MAX is OS-dependent (~2MB) but realistic plan/prompt content hits it before the limit." 1× this session + general footgun class. Promote if recurs.
- [ ] **Plan digest as pre-judge mandatory step** → update `.claude/PRPs/specs/pi-model-comparator.spec.md` §5 to document `digest_plan()` as required, not optional.
- [ ] **`teardown_challenger_extract_signals.sh`** → new script to automate the SSH token-extraction step. Unblock future experiments from manual EliteDesk access during judge runs.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
