---
name: DQ self-resolved entries belong in resolved array, not pending
description: When self-resolving a DQ in the same write that adds the entry, place it directly in the resolved[] array — not pending[]. Confirmed JM-c retro 2026-04-25.
type: feedback
originSessionId: 55c519bc-f059-426e-9313-bef37c7c7bfd
---
When you write a DQ entry AND self-resolve it in the same Edit (the entry has `answer` text, `answered_by: "impl-self-resolved"`, AND `answered_at` populated), put the entry directly into the `resolved[]` array of `.claude/decision-queue.json` — NOT into `pending[]`.

**Why:** the `pending` array is the queue for items awaiting an answer. A fully-resolved entry sitting in `pending` looks unresolved to any future reader (BM, advisor, hook scripts that count `DQ pending: N`). The state inside the entry says resolved but the array placement says pending — those two signals contradict.

Confirmed JM-c retro 2026-04-25: I added DQ #49 (Test 6 deadlock) to `pending[]` despite filling in the answer fields. User flagged at PR-prep time as a "minor process slip" requiring `chore(decision-queue): move resolved #49 to resolved array` or fold-into-bundle-commit fix. The attribution itself was correct (`impl-self-resolved`, not `advisor`) — only the array placement was wrong.

**How to apply:** when self-resolving in-channel, the Edit that adds the entry should target the closing `]` of `resolved[]`, not `pending[]`. Place the new entry as the FIRST element of `resolved[]` (most-recent-first ordering matches the existing convention — DQ #48 sits at the top of resolved per the JM-c-era file). The hook context line `DQ pending: N [#X, #Y]` will then correctly drop the resolved entry from its count, signalling closure to the next session.

The pending-array placement only makes sense when the entry is genuinely awaiting an answer (`answer: null`, `answered_by: null`).
