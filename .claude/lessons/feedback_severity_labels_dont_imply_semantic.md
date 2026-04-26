---
name: CodeRabbit severity label does not imply semantic impact
description: A 🟠 Major rename of an unused-looking parameter is cosmetic, not semantic — accepting it as "fixing something" masks live bugs
type: feedback
originSessionId: e8d77f73-4996-415b-810b-3f0a24b2ddf1
---
A CodeRabbit severity badge (🔴 Critical / 🟠 Major / 🔵 Trivial) describes
how CodeRabbit classifies the *finding*, not whether the proposed *fix*
will change runtime behaviour. A "Major" rename or a "Major" style
suggestion is just as cosmetic as a "Trivial" one — the badge reflects
reviewer confidence, not code-change blast radius.

**Why:** PR #76 shipped `5ceffb52d` which renamed `_admin_id` → `admin_id`
in response to CodeRabbit's 🟠 Major finding. The rename was mechanical;
the parameter was already referenced by `updated_by: Some(_admin_id)`. It
changed nothing semantic. But because it was labelled "Major" I triaged
it as "real" and didn't re-investigate whether the test it was supposed
to unblock was actually fixed. The same test then failed a 2nd time in CI
at the same line — because the real bug was in the test's key choice
(`liability.regular_multiplier` is `Instance`-scoped, not `Both`), not in
the handler parameter binding. The user flagged it on the 2nd failure.

**How to apply:** Before committing any CodeRabbit-prompted fix, answer:
"If I revert this fix, would the symptom come back?" If no — the fix is
cosmetic and does not close the underlying failure mode. If there's an
associated failing test, don't claim it's fixed without re-running.
Refactor / style findings from ANY severity class are cosmetic by
definition — they don't earn the "this closes the red test" claim no
matter what colour badge they carry.

Pattern tie-in: Complements `feedback_pr_review_triage_pattern.md`
(4-bucket split) and `feedback_coderabbit_triage_four_buckets_confirmed.md`
(the split held on PR #76) — but adds a missing step: after the split,
validate that each "Mechanical fix" bucket entry actually moves behaviour,
not just formatting.
