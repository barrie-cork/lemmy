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
- ⏳ planning-001 CHALLENGER run still in flight on the daemon (see
  `runs/planning-001/challenger/` — trace growing, 370 reads / 66 bash, no
  meta.json yet = not finalized). Eval can't run until this finishes AND the
  7 pieces exist.
- ❌ The 7 eval-harness pieces — your build.

## Related
- `.claude/PRPs/specs/pi-model-comparator.spec.md` — methodology (§7 judge, §8 routing)
- `.claude/PRPs/comparator/planning-001.json` — first experiment config
- `project_minimal_pi_eval_environment_banked.md` (PMD) — the standing-env plan
