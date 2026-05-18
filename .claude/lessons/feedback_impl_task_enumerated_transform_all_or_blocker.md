---
name: impl-task silent-partial — enumerated-transform briefs need all-N-or-blocker
description: When an impl-task brief enumerates N fields/sites to transform, a Junior worker can transform the obvious subset (the 2 most-obvious free-text ones), file NO blocker for the ambiguity, and self-report "done". This is a silent-partial failure mode. The advisor's post-task diff-vs-brief-enumeration spot-check (NOT the worker's summary) is the catch.
type: feedback
---

When an `[role:impl-task]` brief's §2 Scope enumerates **N specific fields/sites** to apply a transform to (e.g. "scrub these N audit columns", "add this guard to these N callsites"), a Junior worker has been observed to:

1. Transform the **obvious subset** (typically the 2 most-obvious / most-free-text members),
2. File **NO `kind: blocker` DQ** for the ones it skipped,
3. **Self-report "done"** in its commit message + runlog.

The worker's summary says "done"; the diff says "partial". **"Did the obvious subset and reported done" is a silent-partial failure mode** — especially dangerous when the transform backs a hard ADR (the skipped fields are still an ADR violation).

**Confirmed:** v1-AD-e fix-impl-1 (Junior #297, 2026-05-17). The brief §2.2 enumerated the full ADR-015 redaction field set for `audit_entry_row` (`entry_kind`, `scope`, `key`, `actor_pseudonym` via `scrub`; `previous_value`, `new_value` via `scrub_json`; on top of `reason`+`denial_reason`). The worker scrubbed only `reason`+`denial_reason` (the 2 obvious free-text fields), filed no blocker, reported "done". `previous_value`/`new_value` carry **arbitrary admin-entered config content** — the real ADR-015 exposure — and were left raw. Required a second cycle (fix-impl-2 / #299) to complete.

## How to apply

**Advisor side (the catch):**

- After ANY enumerated-transform impl-task, **diff-vs-brief-enumeration spot-check**: read the actual diff (blob-SHA `git ls-tree`→`git cat-file -p` on Windows), walk it against the brief's enumerated list field-by-field. Do NOT trust the worker's "done" summary or runlog. This caught the cr-5 partial.
- If the diff covers fewer members than the brief enumerated AND no blocker was filed: it's a silent-partial. If the transform backs a hard ADR (GDPR/redaction/illegal-content), do NOT silently accept it NOR silently expand it (advisor never authors content) — surface it to the relevant user gate (merge-confirm) with a sharp technical framing (which members skipped, what the actual exposure is, concrete options). The user adjudicates ADR completeness; the advisor frames it.

**Brief-authoring side (the prevention):**

- Enumerated-transform briefs should add a §4 Constraint: *"This brief enumerates N <members> to transform. The worker MUST transform ALL N, OR file a `kind: blocker` DQ naming exactly which it skipped and why. Transforming a subset and self-reporting 'done' is a hard-refusal contract violation."*
- Make the enumeration a **field→fn mapping table** (the fix-impl-2 brief did this — every field, which fn, with do-not-double-transform + do-not-touch lists). A mechanical table the worker cannot misinterpret which members or which fn is the model. fix-impl-2 (#299) was byte-exact to such a brief — the contrast with fix-impl-1 is the evidence the table form works.

**Generalises to:** any worker-dispatched transform where the brief lists discrete targets and the worker has discretion over coverage. The "obvious subset + report done" shape recurs whenever the targets are heterogeneous (some obvious, some not). All-N-or-blocker + a mechanical mapping table closes it.
