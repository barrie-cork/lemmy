# No cargo output paste into conversation

Companion rule to `cargo-output-capture.md`. That rule is about exit-code
safety (pipes mask failures). **This rule is about conversation token
budget** — a separate concern that compounds across a long ralph loop.

## The rule

When a cargo command has been captured to a log file per
`cargo-output-capture.md`, **do not paste the full log into the conversation
unless it is small and load-bearing for the current decision.** Pasting
verbose build output into chat burns the context window for every subsequent
iteration — the captured log is already on disk, and only the parts relevant
to the decision belong in the conversation.

Default pattern for reading a captured cargo log:

```bash
tail -20 .claude/build-taskNN.log
echo "exit: $status"
```

Only the last 20 lines go into the conversation. The full log stays on disk
for diagnosis if a later iteration needs it.

## When the full log IS load-bearing

Rare but legitimate cases where you should read more than the tail:

- A panic or linker error whose context is spread across >20 lines
- A test failure where the assertion is far from the final `error:` line
- A clippy warning cascade where the specific lint location matters

In those cases, use `Read` with `offset`/`limit` to pull the relevant
section, not the whole file. A 4000-line cargo log at 50 tokens per line is
200k tokens — dumping one into the conversation is an instant context crisis.

## What never to do

- ❌ `cat .claude/build-taskNN.log` in Bash and letting all 4000 lines land
  in the tool result
- ❌ `Read` on a full cargo log without `offset`/`limit`
- ❌ Pasting verbatim "for your reference" when the advisor didn't ask for it
- ❌ Re-reading the same log file multiple times in one iteration — the
  first read's content is already in conversation history
- ❌ Pasting cargo output into commit messages (not a token issue but still
  wrong — commit messages are for the *why*, logs are for the *what*)

## Why this matters here

Phase 1 of Brehon ran 14 tasks and ended at ~360k tokens of messages on the
implementer side — past the ~200k effective-reasoning threshold where model
judgment degrades. The dominant cost was cargo output pasted into
conversation, re-reads of files the agent already had in context, and long
stream-of-thought diagnosis blocks.

This rule attacks the first of those three directly. The phase-splitting
rule (no more than ~10–12 tasks per ralph loop) attacks the compound effect.
Together they keep each sub-phase's ralph session in the <200k reasoning
zone where Phase 1 actually went smoothly (the first ~8 tasks) rather than
the >300k zone where cosmetic-churn intercepts became necessary.

This rule is mandatory for `claude -p` mode (PRP commands, ralph loop) and
loaded automatically.

## Related rules

- `cargo-output-capture.md` — the companion rule about redirecting cargo to
  files so exit codes propagate correctly. **Always capture first, then
  decide how much of the capture to read.**
