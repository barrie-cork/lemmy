---
name: DQ-v3 — append entries via dq-v3-append-fragment.sh; the printed stdout id is authoritative (the helper renumbers, ignoring the fragment's own id field)
description: dq-v3-append-fragment.sh deliberately overwrites whatever `id` is in your fragment with a freshly-minted `<session>-<seq>` and prints the real id to stdout. If you pre-write your intended ids into the brief/commit before reading stdout, the references will be wrong — append order, not fragment id, determines the assigned id. Append first, read the printed id, THEN write references.
type: feedback
---

`scripts/brehon/dq-v3-append-fragment.sh <fragment.json> [--pending]` is the canonical way to append a DQ entry under schema-v3 (replaces inline `python -c` / heredoc merges; pairs with the Write-tool-fragment pattern in [[feedback_windows_backslash_path_dq_via_write_fragment]] for backslash-path safety). It does five things, and **step 3 is the trap:**

1. Calls `dq-v3-new-entry.sh` to mint a fresh `<session_id>-<seq>` id.
2. Reads your fragment (a single entry object).
3. **Injects the fresh id into the fragment — OVERWRITING whatever `id` you wrote in the fragment.** The `id` field in your fragment file is *ignored*.
4. Appends to `resolved[]` (default) or `pending[]` (`--pending`).
5. **Prints the assigned id to stdout** — this is the authoritative id for your commit message and any prose references.

## The failure mode

You author three fragments `dq-051.json`, `dq-052.json`, `dq-053.json` with `id` fields set to your intended mapping (051=WP-1, 052=WP-2, 053=WP-3). You then write those ids into the brief and the commit message. But the helper assigns ids by **append order**, not by your fragment's `id` field — and if you append them in a different order (e.g. the resolved one first, then the two pending ones), the real mapping becomes 051=WP-3, 052=WP-1, 053=WP-2. Every reference you pre-wrote is now off-by-one-ish and points at the wrong entry. You discover it only when you read the DQ back, and you need a `docs(advisor):`/correction follow-up commit to fix the references.

**Confirmed:** m2-rooms-a clarify pass (2026-06-06). First commit `931af71b8` referenced 051=WP-1/052=WP-2/053=WP-4; the helper had actually assigned 051=WP-4 (appended first), 052=WP-1, 053=WP-2. Required correction commit `28db26a68`. Root cause: pre-wrote intended ids into prose before reading the helper's stdout.

Secondary surprise: calling `dq-v3-new-entry.sh` N times in rapid succession (to "pre-allocate" ids) prints the **same** id each time — the per-session sequence counter does not advance until an append actually lands. So you cannot pre-allocate a block of ids by calling the new-entry script repeatedly. (Harmless if you use the append helper, which mints its own; actively misleading if you try to reserve ids ahead of time.)

## How to apply

1. **Author the fragment with whatever `id` you like (or omit it) — it will be overwritten.** Don't agonise over the `id` field in the fragment; it's discarded.
2. **Append, then capture stdout:** `id=$(bash scripts/brehon/dq-v3-append-fragment.sh frag.json --pending)` — `$id` (or the printed line) is the real id.
3. **Only AFTER reading the printed id**, write it into the brief, the commit message, the workflow-state scratchpad, or any cross-reference. Never write an intended id into prose before the append returns.
4. **If appending multiple entries, append them one at a time and record each printed id in order** — the order you append in is the order ids are assigned. If a specific id↔topic mapping matters (it usually does for readability), either append in the order you want the numbers to fall, or just accept the assigned order and reference the real ids.
5. **Always read the DQ back** (`python -c "import io,json; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); print([(e['id'], e.get('question','')[:40]) for e in d['pending']+d['resolved'][-4:]])"`) before writing any id references downstream. This is the verify-before-trusting step ([[pattern_verify_before_trusting_shell_output]]) applied to DQ ids.

**Generalises to:** any helper that mints an identifier server-side and returns it — never assume the identifier you proposed is the one that was assigned. Read it back from the authoritative source (the helper's stdout, or the written file) before referencing it.
