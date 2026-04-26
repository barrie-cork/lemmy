---
name: CodeRabbit Critical findings should block-merge, not just advise
description: CodeRabbit's value surface is row-scoping + control-flow + AP protocol binding semantics — classes local advisor watchpoints + e2e tests miss because they require reading handlers against real schema OR reading AP trait impls against upstream security model. 3× now.
type: feedback
originSessionId: f3b56297-5809-4cc6-a49a-ca58a2b8b6b0
---
CodeRabbit (Pro, ASSERTIVE profile) has now caught 3 critical bugs in
the Brehon governance fork that the local advisor + e2e harness missed:

1. **Phase 5a PR #4** — sanction-scoping bug
2. **Phase 6 PR #46 #15** — duplicate federation notices (4-phase
   latent: Phase 4 task 43 → Phase 6 federation publish made it
   observable)
3. **Phase 6 PR #46 #19** — AP actor-binding impersonation (Trust
   attestation actively exploitable in v0; sanction notice safe in
   v0 but vulnerable in v1)

**Why CodeRabbit catches these and local review doesn't:**
- **Row-scoping bugs** — require reading handler against real schema
  with real multi-row test data. Advisor reading code mentally tracks
  one happy-path execution; CodeRabbit traces all branches.
- **Control-flow bugs** — require reasoning about every reachable
  path through a function with real inputs (not just the inputs the
  test passes). The Phase 6 #15 bug needed a 5-vote scenario; tests
  used 3.
- **AP protocol binding semantics** — require reading trait impls
  against upstream security model. Local review reads the new code
  and assumes the framework handles binding; CodeRabbit knows the
  framework doesn't.

**Process change:**

CodeRabbit findings labeled **Critical (🔴)** should be **block-merge**,
not flag-and-defer. Treat them as load-bearing infrastructure, not
advisory.

**Critical-finding workflow (per PR):**
1. Read every CodeRabbit Critical comment fully (don't skim).
2. Investigate via Explore agent in parallel if multiple — different
   files, no conflict.
3. Fix in scope of the PR if the issue is in PR-touched code.
4. If the issue is in PR-adjacent code (e.g., #15 was a latent
   Phase-4 bug exposed by Phase 6's federation), fix it anyway —
   CodeRabbit caught it now, defer-and-forget loses the finding.
5. Add a regression test in the same commit. For #15 it was
   exactly-once invariants; for #19 it was a negative-path subtest.
6. Separate commits per finding (cleaner re-review).

**Major findings (🟠)** are negotiable carry-forward. Document as
GH issues, file alongside the PR, link from retro.

**Cost-benefit:**
- Cost: ~30-60 min per Critical finding (investigate + fix + test)
- Benefit: prevents shipping a known production bug that local review
  has demonstrated 3× it cannot catch

**How to apply:**
- Default to "fix CodeRabbit Critical in-PR" rather than "carry-forward"
- Use Explore agent for parallel investigation (#19 + #20 ran
  concurrent on PR #46 — different files)
- Cite the CodeRabbit finding number in the commit message + retro
- Save the lesson + the finding's *class* (not just the specific bug)
  to memory if it's a new class

**Why this rule exists:** PR #46 retro on Phase 6 close had to be
amended because the original retro missed both #15 and #19; CodeRabbit
review caught what advisor + 5 hours of agent execution + e2e suite
all failed to catch. This is now the third PR in a row where CodeRabbit
caught a bug class advisor missed. Pattern is established.
