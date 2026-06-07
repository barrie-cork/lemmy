# Handover — MiniMax judge auth verified (2026-06-07)

**From:** advisor session (canonical `brehon-fork`, `governance-v0`)
**For:** the session building the 7 eval-harness pieces (`comparator-judge.sh`,
`minimax-api.sh`, etc.) per the minimal-Pi-eval-environment plan.
**Status:** MiniMax M3 is fully authenticated and reachable. Build the judge
on top of the verified mechanism below — do NOT re-derive it.

VERIFIED_AT: daemon `/srv/brehon-fork`, key in `.env`, Pi probe 2026-06-07 ~15:43 UTC.

---

## TL;DR for `minimax-api.sh` (piece #7)

Use the **native MiniMax chat endpoint**, NOT the Anthropic-compat shim.

| | Endpoint | Auth header | Result |
|---|---|---|---|
| ✅ USE THIS | `POST https://api.minimax.io/v1/text/chatcompletion_v2` | `Authorization: Bearer ${MINIMAX_API_KEY}` | HTTP 200, real M3 response |
| ❌ AVOID | `POST https://api.minimax.io/anthropic/v1/messages` | `x-api-key: ${MINIMAX_API_KEY}` | returns spurious `insufficient balance (1008)` even though the account HAS balance |

The `/anthropic` compat shim reports `insufficient balance (1008)` against a key
that the native endpoint serves fine — it appears to meter on a separate (empty)
billing bucket, or the compat shim is mis-wired on MiniMax's side. **This is a
trap: do not conclude "no balance" from the compat endpoint.** The native
`chatcompletion_v2` endpoint is the source of truth and it works.

### Native endpoint request shape (verified)

```bash
curl -s https://api.minimax.io/v1/text/chatcompletion_v2 \
  -H "Authorization: Bearer ${MINIMAX_API_KEY}" \
  -H "content-type: application/json" \
  -d '{"model":"MiniMax-M3","messages":[{"role":"user","content":"..."}],"max_tokens":N}'
```

Response is OpenAI-chat-shaped: `.choices[0].message.content` for the answer,
`.choices[0].message.reasoning_content` (+ `.reasoning_details[]`) for the
model's thinking trace — **capture `reasoning_content` for the judge**; the judge
methodology wants reasons-before-scores, and M3 emits its reasoning there.
Note: with a tiny `max_tokens` the probe returned `finish_reason:"length"` with
empty `content` but populated `reasoning_content` — give the judge enough
`max_tokens` (the reasoning eats budget first).

## Key location

- **Daemon `.env`:** `MINIMAX_API_KEY` is present (`/srv/brehon-fork/.env`, gitignored, 125 chars).
- **Laptop `.env`:** same key (`C:/Users/barri/Developer/brehon-fork/.env`).
- Source it into any pi/judge invocation with `set -a; source .env; set +a`.
  **Never** pass the key inline on a remote command line (process-table leak —
  the auto-mode classifier will block it; it blocked me once, correctly).
- This is the SAME key as the suspended MiniMax A/B trial (`queue-minimax-task.sh`
  reads it the same way). The trial was suspended for *dispatch-infra* reasons
  (wrong-DB, missing branch), NOT for an auth/balance reason — auth is fine.

## If you wire the judge through Pi instead of raw curl

Pi reaches MiniMax via its native `minimax` provider:

```bash
pi -p --model 'minimax/MiniMax-M3' --no-context-files '<prompt>'
```

Verified working (returned the exact sentinel). **Benign warning to expect:**
`Warning: Model "MiniMax-M3" not found for provider "minimax". Using custom
model id.` — Pi's static catalog predates M3, so it passes the id through as a
custom model. It works; ignore the warning (or suppress it in the wrapper).

**Recommendation for the judge:** prefer the **raw-curl `minimax-api.sh`** path
over the Pi path for the judge step — the judge needs deterministic JSON
(scores + reasoning) and full control of `max_tokens`/temperature, which the
raw endpoint gives directly. Reserve Pi for the *generator* cells (the arms),
not the judge.

## Judge-selection constraint (from the spec)

Per `.claude/PRPs/specs/pi-model-comparator.spec.md` §7: the judge MUST be a
third model, distinct from BOTH arms. For planning-001 the arms are
Opus-4.8 (control) and `openai-codex/gpt-5.5` (challenger), so MiniMax-M3 as
judge is valid (≠ either). If a future experiment uses MiniMax as an *arm*,
pick a different judge.

## What's ready vs not

- ✅ MiniMax auth + endpoint verified (this note).
- ✅ Comparator substrate already committed: `run-comparator.sh`,
  `comparator-code-gates.sh`, spec, `planning-001.json` (commits 9e4aa2ad2, 276032aa3).
- ✅ planning-001 CHALLENGER run COMPLETE (with caveats — see below). Plan
  recovered + code-gates run. Gradeable artifacts pulled to laptop at
  `.claude/PRPs/comparator/runs/planning-001/` (114K; trace stays daemon-local).
- ❌ The 7 eval-harness pieces — your build. The judge step (MiniMax) is the
  last missing leg; code-gates already done (`eval/gates-*.json`).

## planning-001 challenger OUTCOME (read before building the judge)

**Outcome: `context_overflow_wedge_after_plan_committed`.** GPT-5.5 committed a
complete 736-line / 17-section / 9-task plan to its `ab-cell` branch, THEN blew
past its 272K context window on a later action and the run died. Two findings,
both primary:

1. **272K overflow (the headline result).** GPT-5.5 re-read the harness
   inefficiently — 587 `read` calls, 144× on `crates/server/tests/e2e/governance.rs`,
   82× on the implementation plan, 1039 `bash` calls. Trace hit 263 MB. The model
   returned `"Your input exceeds the context window of this model"` (3×). **This is
   the context-management finding the experiment was designed to surface** — the
   Brehon planning corpus is too large for GPT-5.5's 272K when the model re-reads
   rather than caches. The routing recommendation should weigh this heavily.

2. **`.pi/extensions/lemmy-hooks.ts` is NOT reload-safe (harness bug for YOU).**
   When Pi triggered `compaction_start reason:overflow` → `ctx.reload()`, the
   Brehon hook threw `"This extension ctx is stale after session replacement or
   reload"` (lemmy-hooks.ts:289 + :323) and cascaded the failure. Fix per Pi's
   own error text: move post-replacement work into `withSession` and use the ctx
   passed to it; don't use a captured ctx after `ctx.reload()`. **This will bite
   the eval cells too** — the eval `.pi/agents/eval.md` (#5) must not rely on the
   same captured-ctx pattern.

### Deterministic code-gate scores (challenger vs Opus control)

| Gate | GPT-5.5 | Opus 4.8 | Note |
|---|---|---|---|
| sections | 17 | 21 | Opus richer structure |
| adr015_named | 16 | 11 | GPT-5.5 names it more |
| adr015_callsite | 4 | 7 | Opus binds it to callsites (load-bearing per §2.4a) |
| task_count | 9 | 10 | ~par |
| mirror_refs | 5 | 9 | Opus mirrors more canonical sources |
| **story_signal (§16a)** | **0** | **10** | **decisive — GPT-5.5 omitted §16a stories entirely** |
| cargo_cmds | 8 | 17 | Opus more DoD-complete |
| p_features_footgun | 0 | 1 | GPT-5.5 cleaner here (Opus has 1 `-p+--features full`) |
| **t1_preemption_signal** | **0** | **16** | **decisive — GPT-5.5 made NO mention of lane-mode / validate-pending / Mode-A/B** |

Two decisive deterministic gaps (§16a stories, T1-preemption) plus the overflow.
The judge (MiniMax) adds the nuanced dimensions: task-decomposition quality,
watchpoint specificity (the gates' `watchpoint_cites` grep mis-fires — both read
0, so the judge must score this), ADR-015-callsite *quality* not just count.

### `comparator-code-gates.sh` BUG to fix in your harness

The script's `|| echo 0` fallbacks fire INSIDE the `printf '%s'` for some gates,
emitting a stray `\n0` that produces **invalid JSON** (e.g.
`"watchpoint_cites":0\n0,`). See `eval/gates-*-raw.json`. Replace the
`$(... || echo 0)` pattern with a variable assigned beforehand
(`X=$(...); X=${X:-0}`) so the value is a clean integer before `printf`.

### Run artifacts (laptop)

- `runs/planning-001/challenger/meta.json` — full outcome + both findings
- `runs/planning-001/challenger/challenger-output-plan.md` — the gradeable plan
- `runs/planning-001/eval/control-opus-plan.md` — the Opus ground-truth (from `phase-m2-late-1`)
- `runs/planning-001/eval/gates-{challenger,control}-raw.json` — code-gate output
- Daemon-only (too big for git): `trace.jsonl` (263 MB), `session/`, `pi-run.log`,
  `ab-cell/planning-001-challenger` branch (the plan's source-of-truth commits).

## Related
- `.claude/PRPs/specs/pi-model-comparator.spec.md` — methodology (§7 judge, §8 routing)
- `.claude/PRPs/comparator/planning-001.json` — first experiment config
- `project_minimal_pi_eval_environment_banked.md` (PMD) — the standing-env plan
