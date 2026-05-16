# Refactor-tier FINAL retro — PR-1 + PR-2 serial lanes, and the whole tier

> **Scope.** This is the *closing* retro for the entire refactor-tier
> (the strict 5-PR gate that blocked v1 PRD planning). It covers the
> two serial lanes that landed after the parallel lane — **PR-2 #131
> (TOCTOU)** and **PR-1 #132 (e2e error-types)** — and rolls up
> tier-wide lessons.
>
> The parallel lane (PR-4 #128 / PR-5 #129 / PR-6 #130) already has its
> own retro at `.claude/PRPs/reports/refactor-tier-retro.md`. This file
> **references that one and records only the delta** — it does not
> duplicate its per-task tables, its consolidated-lesson list, or its
> Advisor/Planning/Impl/BM/ci-watcher signal sections. Read both for the
> full tier picture.

## TL;DR for the advisor

- **Strict gate 5/5 achieved.** PR-4 #128 (09:04Z 05-15), PR-5 #129
  (11:55Z), PR-6 #130 (13:12Z), PR-2 #131 (16:02Z), PR-1 #132
  (01:20:42Z 05-16). PR-3 was dropped via DQ #214 (audit finding
  3.C.1 reject-finding) — the gate was always "the 5 that remain",
  never 6.
- **v1 PRD planning is now UNBLOCKED but deliberately NOT started**
  per the standing instruction — this retro is the gate before that
  resumes.
- **PR-1 was the expensive lane by an order of magnitude.** Not
  because the refactor was hard, but because it triggered the first
  non-trivial CodeRabbit triage cycle of the whole project, which
  exposed four distinct process lessons (one of which cost a
  fix→revert round-trip and two AskUserQuestion gates to land back
  exactly where the user's ratified code already was).
- **One explicit carry-forward:** deferred audit 3.E.2 / Pass 2
  (fixtures dedup), tracked as advisor task #8. It is **not** a v1-PRD
  blocker and must not be auto-dispatched — it resumes only on
  explicit user instruction in a separate post-gate lane.

## Per-task complexity table

Columns: lane / files / commits-on-merged-range / wall-clock-min /
max-log-silence-min. PR-4/5/6 rows are in the prior retro; repeated
here only as one-line totals for tier rollup.

| Lane | files | commits | wall-min | max-silence-min | notes |
|---|---|---|---|---|---|
| PR-4 #128 | (see prior retro) | — | — | — | parallel lane — `refactor-tier-retro.md` |
| PR-5 #129 | (see prior retro) | — | — | — | parallel lane — `refactor-tier-retro.md` |
| PR-6 #130 | (see prior retro) | — | — | — | parallel lane — `refactor-tier-retro.md` |
| **PR-2 #131** (TOCTOU) | ~1 (handler) | small | ~moderate | n/a (local validate) | clean lane; no CR cycle of note; merged 16:02Z 05-15 same day as dispatch |
| **PR-1 #132** (e2e error-types) | 1 (`crates/server/tests/e2e.rs`, 8945 lines) + DQ + runlog | **12** on merged range | **~6 h 25 m** (Junior #268 19:00Z → merge 01:20Z; ~2 h pure Junior impl+continuation, ~2.5 h CR-triage cycles, rest validate/orchestration) | Junior #268 hit a Gate-2 partial+blocker, dispatched #269 continuation (no single >6-min stdout-silence kill — pre-existing watchdog already at 60 min) | the whole tier's expensive lane |

PR-1 commit timeline (merged range `c4b09b65e^1..ba9177d46`):

```
f258824b5 19:00 chore(test): unify e2e error-type to LemmyResult + split phase1_migrations_round_trip (3.E.1+3.E.3+3.E.4; 3.E.2 deferred)   [Junior #268]
091b83a86 19:02 chore(decision-queue): advisor answered DQ #221 + #222 (user-relayed)
c9a1a316b 19:07 chore(bm): runlog — #268 partial+blockers, continuation #269 dispatched
919fe8400 21:37 chore(test): fix pre-existing e2e.rs clippy lints for Gate 2 (DQ #221)                                                       [Junior #269]
3084eaab4 22:46 chore(decision-queue): advisor-laptop validate-pending PASS (CHECK+CLIPPY+E2E exit 0)
3934ac38b 22:50 chore(bm): runlog — PR #132 opened
6c75ea5ce 23:17 fix(test): address PR #132 CR findings (cr-2/3/4/5) — user-approved triage                                                  [advisor inline]
f02378f13 23:28 revert(test): restore .get(0) + #![expect(clippy::get_first)] — cr-2 disproved by compiler                                  [advisor inline]
072f623e9 23:34 fix(test): clippy if_then_some_else_none in assert_revert_list_matches_disk (cr-5 follow-up)                                 [advisor inline]
1a2833ac4 00:14 chore(decision-queue): advisor-laptop RE-validate #3 PASS (CHECK+CLIPPY+E2E exit 0)
69f88ee8f 00:25 test(e2e): add non-ignored phase1_revert_list_matches_disk guard (cr-6)                                                     [advisor inline]
ba9177d46 01:21 chore(decision-queue): advisor-laptop RE-validate #4 PASS cr-6 (88->89 expected)
```

That's **3 advisor-authored inline fix/revert commits + 1 follow-up +
3 validate-pending cycles** for a single CR review — quantifying why
"first non-trivial CR triage" was the tier's dominant cost.

## Per-role signals (delta only — parallel-lane signals are in the prior retro)

### Advisor (this session — orchestrator)

- **Held all six user gates.** USER GATE 3 (CR triage) fired twice on
  PR-1 (initial cr-2/3/4/5 + the cr-6 re-triage); USER GATE 5 (merge
  confirm) fired once. No gate was self-decided. The user-approved
  *advisor-authored-inline* four-role exception (advisor writes
  `crates/server/tests/e2e.rs` for CR fixes) was scoped to this CR-fix
  cycle only and is now spent.
- **Self-caught the cr-6 Gate-2 invocation-scope error** without user
  prompting — see lesson (iv). Net cost was a wasted ~5-min clippy
  cycle, not a regression.
- **Correctly distinguished cr-6 (runtime-TRUE) from cr-2
  (hypothesis-FALSE).** cr-6's premise (the cr-5 assert never ran
  because it was buried in `#[ignore]`d fns) was *empirically* proven
  by the re-validation-#3 Gate-3 log showing those 3 fns as `ignored`
  and the suite count unchanged at 88 — so no compiler scratch-check
  was needed before bucketing it. cr-2's premise was a trait-resolution
  *claim* and got the scratch-check it required (and failed it). The
  advisor applied the right standard of proof to each.
- **CodeRabbit auto-pause was correctly detected and surfaced**, not
  silently worked around. CR paused after the commit influx; the
  advisor surfaced the choice (trigger `@coderabbitai review` vs.
  proceed) rather than either (a) silently posting the trigger comment
  or (b) silently skipping the CR gate. User chose proceed-to-merge.

### Planning (the #268 brief, authored pre-tonight)

Two planning-defect lessons, both rooted in the #268 brief:

- **(a) Gate 2 was a must-pass that was never green on `e2e.rs`
  pre-refactor.** The brief specified `cargo clippy ... -D warnings`
  as an acceptance gate, but the pre-existing `e2e.rs` carried clippy
  debt that predated the refactor. Junior #268 hit this as a blocker
  and could not satisfy a gate the brief itself made unsatisfiable
  without out-of-scope cleanup. This is a **brief defect, not an impl
  failure** — the brief should have either (i) baselined the existing
  lint state and scoped the gate to "no *new* lints", or (ii) made the
  pre-existing-lint cleanup an explicit in-scope task. It became DQ
  #221's scope decision (user-ratified: fix-all-e2e-lints, scope the
  clippy invocation to `--test e2e`).
- **(b) The multi-pass brief lacked per-pass completion criteria.**
  A prior session misread "Pass 1" as subsuming "Pass 2" (fixtures
  dedup, audit 3.E.2). Multi-pass briefs MUST carry an explicit
  per-pass DoD so "is Pass N done?" is mechanically answerable, not a
  judgement call. 3.E.2 is now an explicit deferred carry-forward
  (task #8) precisely because this ambiguity bit once already.

### Impl (Junior #268 → #269 continuation)

- #268 delivered the LemmyResult unification + the
  `phase1_migrations_round_trip` split (audit 3.E.1 + 3.E.3 + 3.E.4),
  but partial — Gate 2 blocked on pre-existing lints (planning defect
  (a) above). The advisor dispatched #269 as a continuation rather than
  forcing #268 to exceed its brief scope. Correct call: the blocker
  was a brief defect, and a continuation with the DQ-#221-ratified
  scope was the clean recovery, not a hard refusal or a scope-creep.
- No watchdog kill on either worker (the 60-min watchdog patch held;
  no >6-min stdout-silence event on the merged range).

### BM (bm-pr — run INLINE by advisor per L3/L15)

- bm-pr ran inline (PR #132 opened `3934ac38b` 22:50Z); runlog written
  (`c9a1a316b`, `3934ac38b`). No BM file-ownership breach. The L15
  consolidation (gate-side read-only checks inline in advisor, Junior
  only for the mutating merge) held: the merge itself was a single
  advisor `gh pr merge` post-confirm, no pre-confirm Junior dispatch.

### ci-watcher (Shape-G)

- **Not load-bearing this session.** Per the user directive
  (2026-05-15): `cargo-validate-workspace` runs LOCAL ONLY for the
  rest of the session. No ci-watcher was dispatched for PR-1; all
  workspace validation was advisor-laptop §5.2 (DQ #221/#224/#225,
  `answered_by: advisor-laptop`).
- The Shape-G chore-branch run **`25935134971`** (headBranch
  `junior/role-impl-task-...-269`, createdAt 2026-05-15T18:41:30Z) is
  recorded for completeness: it ultimately reported
  `status=completed conclusion=success`. It is **historical context
  only — not load-bearing**, because local validation was the
  canonical signal per the user instruction. Earlier in the session it
  appeared stuck (the reason the user issued the local-only directive);
  it later completed-success on its own. No action taken on it; it did
  not gate the merge.

## Tier-wide lessons (new this lane — counted; parallel-lane lessons are in the prior retro)

PR #132's CR cycle was the project's **first non-trivial CodeRabbit
triage**. Four distinct lessons, in cost order:

### (i) An automated-reviewer type/trait claim is a HYPOTHESIS until the compiler proves it — *especially* against a user-ratified decision

cr-2 (Copilot): the 3 `.get(0)` sites in `e2e.rs` were post-`.load()`
`Vec`s, so `.first()` "resolves to inherent `Vec::first()`, no Diesel
ambiguity" — recommending removal of the user-ratified file-level
`#![expect(clippy::get_first)]`. The advisor read the call-sites,
concurred, surfaced a fix-in-pr triage; the user approved "fix all 4";
the advisor applied it; **Gate 2 then failed with 9 compile errors**
(`error[E0275]` overflow + `error[E0277]` `Vec: diesel::Table/Query`
not satisfied) at exactly those 3 sites. The element types have Diesel
trait impls in scope via the test's `use` imports, so `.first()`
engages the blanket `RunQueryDsl`/`LimitDsl` recursion that `.get(0)`
(an inherent slice method, no trait dispatch) does not. Copilot's
premise was false; the advisor's concurring code-read was false; the
user's original `.get(0)` + `#![expect]` was correct all along.

**Net cost:** 1 fix commit + 1 revert commit + 1 clippy follow-up + 3
clippy cycles + 2 AskUserQuestion rounds, all to end up byte-identical
to the ratified state. **Lesson saved** as
`feedback_verify_automated_reviewer_claims_against_compiler.md`
(+ MEMORY.md pointer): trait/type/method-resolution claims require a
compiler scratch-check *before* surfacing the triage, and a finding
that contradicts a user-ratified `#![expect]`/ADR/gate decision
inverts the burden of proof — treat it as probably-wrong until
empirically reproduced. Occurrence count: **1** (first; watch for
recurrence — if it happens again, this becomes a promote-to-pattern
candidate).

### (ii) Advisor-authored fix code must itself pass clippy `-D` BEFORE commit

The cr-5 helper (`assert_revert_list_matches_disk`) was advisor-written
and introduced a fresh `clippy::if_then_some_else_none` violation,
caught only by the *next* validate cycle — costing an extra clippy
round (`072f623e9`). This is the **gate-class of
`feedback_fix_impl_pre_push_cargo_check`** applied to advisor-authored
fixes: when the advisor takes the inline-author exception, the
advisor inherits the impl-task discipline of a local
`cargo clippy --features full --test e2e -- -D warnings` BEFORE the
commit, not after. Occurrence count: **1**.

### (iii) An audit-deliverable refactor must be EXECUTABLE, not decorative — and the enforcing assertion must run in the DEFAULT suite

Audit item 3.E.4 (the Phase-1 revert-list parity guard) shipped
**twice as inert**:

- **cr-5 round:** first as a helper that *existed* but was never
  called in a way that ran by default.
- **cr-6 round:** then as `assert_revert_list_matches_disk()` called
  only as a pre-flight inside the 3 `phase1_migrations_*` round-trip
  fns — **all of which are `#[ignore]`d** pending GH issue #43. So the
  parity check still never ran in default CI. CR caught this on
  re-review of its own cr-5 fix; the re-validation-#3 Gate-3 log
  *empirically proved* non-execution (3 fns `ignored`, suite count
  static at 88). Only cr-6's standalone non-`#[ignore]`d
  `#[test] fn phase1_revert_list_matches_disk()` made it actually run
  (88→89, list matches disk).

**Two full rounds** to make one audit deliverable genuinely
executable. **Lesson:** audit-deliverable refactors need
"the enforcing assertion executes in the default suite (not gated
behind `#[ignore]`)" as an explicit brief acceptance criterion. And:
**a CR re-review of a CR-fix is a valuable second pass — budget for
it**; the cr-5→cr-6 chain is the proof that the first fix of an
inertness finding can itself be inert. Occurrence count: **1** (but
note it manifested as 2 sub-rounds within the lane).

### (iv) Re-use the EXACT validated command shape from the prior passing DQ — do NOT broaden a ratified invocation scope

On the cr-6 re-validation the advisor ran Gate 2 as
`cargo-clippy.bat --workspace --features full --tests --test e2e --
-D warnings`. The `--tests` flag pulled in `lemmy_api` lib-test
targets (`reputation_snapshot.rs` / `admin_config.rs`) carrying
pre-existing lint debt that **DQ #221 explicitly ruled OUT OF SCOPE**
and ratified `--test e2e` (no `--tests`) as the canonical scope. It
failed with 8 errors, all in the out-of-scope crate, zero in
`e2e.rs`. Self-caught by reading DQ #221's ratified decision rather
than touching the excluded file; re-ran the canonical
`--features full --test e2e -- -D warnings` (Gate 2-b), passed in
2.31s, no code change. **Net cost:** a wasted ~5-min clippy cycle +
investigation. **Lesson:** a ratified invocation scope is a contract;
copy it verbatim from the prior passing DQ — "improving" or
broadening it is the command-scope analogue of paraphrasing a §G4
canonical recipe. Same discipline-class as
`feedback_fix_impl_pre_push_cargo_check`. Occurrence count: **1**.

### (cr-4 sidebar) Attribution-integrity understatement caught by CR

CR flagged that DQ #221 + #222 carried `answered_by: "advisor"` when
the answers were user decisions relayed by the advisor via
AskUserQuestion. Corrected to `answered_by: "user"` (provenance
retained in the answer body; `resolved_at` unchanged). **Same
rule-class as the Phase-6 DQ #37 incident** — a user-relayed answer is
`answered_by: "user"`, not `"advisor"`. Worth noting CR's value here:
it caught a process-integrity slip on meta-state, not just code.

## What did NOT need fixing (worth preserving)

- The user-ratified `#![expect(clippy::get_first)]` + `.get(0)`
  pattern. cr-2 attacked it; the compiler vindicated it 9×. The
  `reason=` string on that `#![expect]` (it cites the exact Diesel
  trait collision) was a load-bearing breadcrumb — it should never
  have been second-guessed on a code-read. Preserve such
  `#![expect]` reasons as evidence, not noise.
- The 3 `phase1_migrations_*` round-trip fns staying `#[ignore]`d
  (GH #43). cr-6 did NOT ask to un-ignore them; it asked for a
  *separate* non-ignored guard. The in-fn pre-flight calls were kept
  per the user's explicit "KEEP 3 in-fn pre-flight calls" choice. The
  ignore is correct (the round-trip fns need the federation-table
  revert-list extension first); only the parity guard needed to run.
- The two-session / lane-worktree discipline. PR-1 ran in its own
  `brehon-fork-e2e-error-types` worktree; no cross-lane DQ collision,
  clean `git worktree remove` at ship. The multi-lane rule held.

## Quantified outcomes vs confidence

| Lane | pre-merge confidence | outcome | gap analysis |
|---|---|---|---|
| PR-2 #131 | high | clean merge, no CR cycle of note | confidence well-calibrated |
| PR-1 #132 | "mechanical refactor, low risk" (initial) | 12 commits, 3 CR rounds, 1 fix→revert, 2 extra clippy cycles | **confidence was MIS-calibrated at lane start** — the refactor itself was low-risk, but "touches the 8945-line e2e corpus + is the first real CR triage surface" was the actual risk, and it was not priced in. Lesson for v1 PRD: a large-surface test refactor's risk is dominated by review-cycle cost, not impl complexity. |
| Tier (5/5) | gate would close "soon" | closed, but PR-1 alone took ~6.5 h of the tier's tail | the serial tail (PR-2 then PR-1) was the long pole; the parallel lane (PR-4/5/6) was efficient. Future tiers: front-load the large-surface / CR-heavy lane, don't leave it last. |

## Suggested action items for the advisor (NOT executed — awaiting sign-off)

1. **Brief-template addition:** for any audit-deliverable refactor, add
   an explicit acceptance criterion "the enforcing assertion executes
   in the default test suite (not behind `#[ignore]`/`#[cfg]`)".
   Source: lesson (iii).
2. **Brief-template addition:** for any refactor touching a file with
   pre-existing lint debt under a `-D warnings` gate, the brief MUST
   baseline the existing lint state and either scope the gate to "no
   *new* lints" or make the cleanup an explicit in-scope task. Source:
   planning defect (a) / DQ #221.
3. **Multi-pass briefs:** mandatory per-pass DoD so "is Pass N done?"
   is mechanically answerable. Source: planning defect (b).
4. **Advisor inline-author exception:** when the advisor takes the
   inline-fix exception, codify that the advisor runs the
   DQ-ratified-scope local clippy/check BEFORE the commit (lesson (ii))
   and copies the validated command shape verbatim (lesson (iv)).
5. **CR-cycle budgeting:** treat "CR re-review of a CR-fix" as an
   expected extra pass in any plan that opens a PR with a non-trivial
   diff; the cr-5→cr-6 inertness chain shows the first fix can be
   inert. Source: lesson (iii).
6. **Promote-if-recurs watch:** lesson (i)
   (`feedback_verify_automated_reviewer_claims_against_compiler.md`) is
   at occurrence 1. If a second automated-reviewer type/trait claim is
   accepted on a code-read and later compiler-disproved, promote to a
   confirmed pattern.

These are recommendations only. Per the standing instruction, **v1 PRD
planning is now UNBLOCKED but NOT started — awaiting your retro
sign-off.**

## See also

- `.claude/PRPs/reports/refactor-tier-retro.md` — parallel-lane
  (PR-4/5/6) retro; this file is its delta + tier rollup.
- `feedback_verify_automated_reviewer_claims_against_compiler.md` —
  lesson (i), saved this session.
- DQ #214 (PR-3 drop), #221 (Gate-2 scope + Pass-2 defer, user-ratified),
  #222, #224, #225 (advisor-laptop validate-pending PASSes).
- Task #8 (advisor task list) — deferred audit 3.E.2 / Pass 2 fixtures
  dedup; post-gate, user-instructed-only, NOT a v1-PRD blocker.
