---
name: fix-impl brief pre-locates verbatim e2e.rs anchors
description: When a fix-impl brief targets `crates/server/tests/e2e.rs`, the advisor MUST pre-locate verbatim `old_string`/`new_string` Edit anchors and paste them into the brief recipe. Leaving "use distinctive anchors" guidance forces Junior to consume its turn budget on Reads before any Edit — recurring `error_max_turns` failure class.
type: feedback
---

# Fix-impl briefs targeting e2e.rs must include pre-located verbatim anchors

When the §G4 classifier (or CR-triage routing) authors a fix-impl-task brief
whose `modifies:` array contains `crates/server/tests/e2e.rs`, the brief
MUST include the **exact `old_string` and `new_string` text** for every Edit
the worker will perform. Pasting "use a distinctive 5-line anchor per
`feedback_clippy_test_style.md`" or "locate the surrounding context via
`grep -n`" forces the Junior worker (Sonnet 4.6 impl-task) to spend its
turn budget on Read + Grep tool calls against a 17,000-line file before
any productive Edit. Empirically, this exhausts the 150-turn budget BEFORE
the worker reaches the Edit phase even when the recipe is correct.

**Why this lesson exists:** Per
`.claude/PRPs/reports/session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail.md`
§"What to change" #1 (recurrence 2× threshold met). Two independent failures
in the same v1-RT-r3 sub-phase, same file, same model:

- **Task 4 cycle** (Junior #474 #475 #476 #477, 2026-05-26 ~05:00Z–~12:30Z).
  Brief: `.claude/PRPs/briefs/v1-RT-r3-impl-4.md` (10 e2e tests across 5
  stories in `v1_rt_r3_fixtures`). All 4 dispatches `error_max_turns`. Cycle-count
  meta-rule §5.3 HARD REFUSAL fired; user authorised advisor-side carve-out
  per DQ `a3d0e9941441-033`. Advisor authored the module locally instead.
- **fix-impl-2 dispatch** (Junior #479, 2026-05-26T18:18Z–18:56Z, 38 min,
  $8.30, 151 turns). Brief: `.claude/PRPs/briefs/v1-RT-r3-fix-impl-2.md`
  (DQ JSON boundary fix + e2e.rs iso-week guards). `error_max_turns`.
  The `permission_denials` array showed the worker had the EXACT correct
  Edit recipe by turn 150 — it just ran out of turns mid-application. The
  150 turns were consumed Reading helpers (`Post::create`, `Comment::create`,
  `admin_assign_jury`, `governance_log` schema, `ReputationEventSourceType`)
  to confirm context — none of which were strictly in the fix-impl-2 scope.

The defect is brief-design × file-fragility × Sonnet context-cost, NOT
recipe-correctness. A 246-line brief covering 3 file edits with helper
anchors left to the worker to locate is **structurally** too dense for
Sonnet impl-task on a 17,000-line target.

**Mechanical root cause:** `e2e.rs` is the canonical Brehon integration
test file. By v1-RT-r3 it had grown to >17,000 lines with ~20+ `mod *_fixtures`
sibling modules. Each `Read` tool call returns at most 2,000 lines; finding
"the right anchor for Edit N" by Read+Grep against a file this size
typically requires 3-10 round-trips per anchor. At 2-3 anchors per Edit,
that's 6-30 Reads before a single Edit fires. Sonnet's 150-turn budget
covers ~30-40 tool calls (with reasoning interleaved); the e2e.rs anchor-
search alone can consume 60-80% of that before any productive work.

**How to apply:** When authoring a fix-impl-task brief whose `modifies:`
array contains `crates/server/tests/e2e.rs`:

1. **Open `e2e.rs` in the advisor session, `grep -n` the canonical sibling
   module** (e.g. `grep -n 'mod v1_ship_3_fixtures' crates/server/tests/e2e.rs`).
   Confirm the line range.

2. **For every Edit the worker will perform, paste the exact text into the
   brief**:

   ```markdown
   ### Edit N — <one-line summary>

   File: `crates/server/tests/e2e.rs`

   **`old_string`** (paste verbatim — 5-10 lines of surrounding context,
   distinctive enough to be unique in the file):

   ```
   <exact text from e2e.rs at the edit site>
   ```

   **`new_string`** (paste verbatim — the replacement):

   ```
   <exact replacement text>
   ```
   ```

3. **NEVER write** "locate the surrounding context per <lesson>" or "find
   the right anchor near line N" in a fix-impl brief targeting e2e.rs.
   Junior cannot afford the Read budget.

4. **If the Edit is appending** (e.g. a new `mod <phase>_fixtures` after the
   last existing one), the brief MUST cite the exact `old_string` of the
   final existing module's closing `}` + any trailing newlines, so the
   worker can append without ambiguity.

5. **At the end of the brief**, sanity-check: count the Edit anchors. Each
   Edit should compile-cleanly in isolation if applied to the current
   `phase-<X>` tip. If the brief cites 3 Edits and 2 of them depend on the
   first landing, document the order in §2 and ensure each `new_string` is
   computed against the correct intermediate state.

**Edge cases:**

- **The anchor needs to handle Windows CRLF**: `e2e.rs` is text-mode in git;
  on Windows checkouts the file may have CRLF line endings on disk but LF
  in the worker's view (Junior runs on Linux). When pasting `old_string`,
  use LF endings (the default if you copy from a `git show`). The Edit
  tool normalises both sides before comparison; LF in the brief matches LF
  in Junior's view.

- **Module-boundary anchors**: when appending a new fixtures module, the
  `old_string` should end with the prior module's final `}` + a blank line
  + the start of the next sibling (or EOF). Anchoring on the prior
  module's `}` alone is ambiguous because every Rust block ends in `}`.

- **The brief is huge already**: if pre-locating anchors makes the brief
  exceed the cap in `impl-task-brief.template.md` §2 Scope gate (≤150
  lines, ≤2 file edits, ≤2 Edits/file when target is e2e.rs), the brief
  MUST split into multiple narrower fix-impl-Na + fix-impl-Nb briefs.
  Pasting anchors that push the brief over the cap is a signal that the
  fix-impl scope itself is too broad for one dispatch.

- **The findings YAML has 5+ findings all touching e2e.rs**: do NOT bundle
  all 5 into one fix-impl. Split per anchor cluster — typically 1-2
  findings per brief. Per
  `.claude/PRPs/reports/session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail.md`
  §"What to change" #2.

- **The change is structural** (rename, signature change, struct-field
  add): pre-locating anchors is still required, but the brief MUST also
  enumerate all callsites per `feedback_fix_impl_enumerate_all_callsites.md`
  — the two lessons compose.

**Why not "always" pre-locate anchors in every fix-impl brief:** for fix-
impl briefs targeting smaller files (≤2,000 lines, e.g. a single
`crates/api/api/src/governance/<handler>.rs`), the worker can `Read` the
whole file in one tool call; pre-located anchors are redundant. The
discipline applies specifically to e2e.rs and any other file that exceeds
the single-Read threshold.

**Companion lessons:**

- `feedback_fix_impl_pre_push_cargo_check.md` — pre-push cargo-check gate;
  same brief-design discipline; covers the runtime backstop after Edit
  lands. This lesson covers the Edit landing in the first place.
- `feedback_fix_impl_enumerate_all_callsites.md` — enumerate ALL N
  callsites before authoring. Composes with this lesson: enumerate first,
  then pre-locate anchors at each enumerated site.
- `feedback_principles_not_rules.md` — when to extend rules vs. accept the
  cost. The pre-location discipline IS a rule for e2e.rs because the
  failure class has fired twice already.

**Where codified:**

- `.claude/rules/advisor-orchestrator.md` §2.4 file-class lesson injection
  table — extend the `crates/server/tests/e2e.rs` row to also cite this
  lesson when the brief is a fix-impl.
- `.claude/PRPs/templates/impl-task-brief.template.md` §2 Scope gate —
  enforce length + edit-count cap that requires pre-location to fit.
- Every fix-impl brief whose `modifies:` array includes `crates/server/tests/e2e.rs`
  authored after this lesson lands — paste anchors verbatim.
- This lesson file.
