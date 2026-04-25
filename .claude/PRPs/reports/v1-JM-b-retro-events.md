# v1-JM-b retro events — accumulating during impl

**Purpose:** mid-phase capture of events the eventual `v1-JM-b-retro.md` will
consume at phase close. Advisor (this session) writes; impl reads
optionally; phase-close retro extracts into the canonical structure of
JM-a's retro (§1 worked, §2 surprised, §3 carry-forward, §4 didn't need
fixing).

**Why mid-phase capture, not phase-close-only:** events surfaced during
impl can be lost between mid-phase resume briefs and the eventual retro.
JM-a R5.2 / R5.3 patterns were captured in impl's session-2 + 3 logs
because they happened to be relayed to advisor; without the relay, they
would have rotted. This file is the durable equivalent of an in-flight
relay archive.

**Companion files:**
- `.claude/PRPs/reports/v1-JM-b-advisor-handover.md` — cold-state for
  fresh advisor sessions
- `.claude/PRPs/plans/v1-jury-mechanics-b.plan.md` — plan being executed
- `.claude/PRPs/reports/phase-v1-JM-a-retro.md` — JM-a retro (the lessons
  whose transfer JM-b is testing)

---

## Event 1 — Task 0 clippy baseline check skipped → debt discovered mid-Task-4

**Observed (mid-Task-4, 2026-04-24):** impl ran a baseline clippy check
during Task 4 (per the per-task validate at plan §13 line 1178) and
discovered ~10 pre-existing `-D warnings` errors on `governance-v0` HEAD
(the JM-a merge `e1c22c759`), most notably an unfulfilled
`#[expect(dead_code)]` on `ConfigScope::Community` at
`crates/api/api/src/governance/config.rs:82` (added during v1-AD as
forward-compat, never consumed by a caller). Pre-existing baseline = red
under the workspace `-D warnings` lint config.

**Resolution path (in-flight):** DQ #49 to be filed by impl; advisor
recommendation is hybrid (option α): fix Task-4-introduced lints inline
in the Task 4 commit; land a separate
`chore(lint): clear pre-existing clippy debt for JM-b DoD` commit on
`phase-v1-JM-b` BEFORE Task 9. Documented in this advisor session's
relay (2026-04-24 turn 6).

**Root cause:** plan §13 Task 0 listed pre-flight probes 1-4 (wrapper
sanity + concurrent-PR check + PM hooks + Docker) but **omitted the
`.claude/rules/pre-phase-harness-audit.md` §3 Clippy baseline capture
step.** §3 of that rule is the gate-keeper for the very class of bug
this event embodies: "If the baseline is non-zero, the plan's clippy DoD
is unexecutable before the phase even starts."

The §3 step was present in Task 0 audits for v1-AD-a/b/c/d, v1-JM-a, and
phase 5a/b/c. JM-b's plan §13 Task 0 dropped it — most likely because
the plan author copied probes 1-4 verbatim from v1-JM-a's plan but JM-a
itself relied on the inherited harness rules to enforce §3 implicitly.
The drop is a plan-wording omission, not a deliberate de-scope.

**Plan-amendment for v1-JM-c/d/e and any future Brehon sub-phase:**

1. **Plan §13 Task 0 must explicitly enumerate the §3 clippy baseline
   capture step.** Current Task 0 enumerates probes 1/2/3/4 (wrapper
   probes) + Docker + PM hook grep + concurrent-PR check. **Add a
   probe 5: clippy baseline capture per pre-phase-harness-audit §3.**
   Wording template:

   ```bash
   cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full \
     --no-deps -- -D warnings > .claude/PRPs/debug/v1-<phase>-task0-clippy-baseline.log 2>&1"
   echo "clippy baseline exit: $?"
   tail -40 .claude/PRPs/debug/v1-<phase>-task0-clippy-baseline.log
   ```

   Expected: exit 0. If non-zero: list each error against the rule's
   two-paths fix (in-pre-phase-commit OR narrow-DoD-via-plan-amendment)
   and pick BEFORE Task 1.

2. **Plan §13 Task 0 EXPECT block must list "clippy baseline = 0
   errors against -D warnings" as a hard precondition for Task 1
   to start.** Currently the EXPECT block stops at "probes 1+3 exit 0;
   probe 4 exits non-zero." Extend with: "probe 5 (clippy baseline)
   exits 0, OR a `chore(lint)` commit lands before Task 1 cleans the
   debt."

3. **Plan §15.1 DoD must reference the Task 0 baseline check** as the
   reason §15.1 is achievable (i.e. §15.1 inherits a green baseline from
   Task 0; §15.1's exit-0 on `cargo clippy --workspace --features full
   --no-deps -- -D warnings` is therefore "Task-0-baseline-green +
   no new red introduced by Tasks 1-9," not "exit 0 from raw").

**Carry-forward to JM-c plan template.** This is plan-wording row to add
to the JM-a retro §3.2 "Plan amendments" table (currently rows 1-4a).
Row 5: "Plan §13 Task 0 must include pre-phase-harness-audit §3 clippy
baseline capture step." Priority: HIGH for JM-c onwards.

**Memory note suggestion (advisor):** consider adding a new feedback-
memory entry `feedback_pre_phase_clippy_baseline_in_task0.md`. The
existing `feedback_clippy_test_style.md` covers the workspace lint
config; this new entry would cover the pre-phase audit step. Title:
"Plan Task 0 must run clippy baseline capture, not just wrapper sanity."

---

## Event 2 — JM-a R5.1 lesson incompletely transferred to JM-b drift-fix `cb7b6bdb2`

**Observed (Task 4 setup, 2026-04-24):** the drift-fix commit
`cb7b6bdb2` ("add selected_under_constraints to JuryAssignmentInsertForm
DQ #48") added an `Option<Value>` field with `#[derive(Default)]` on the
struct, claiming in the commit body: *"existing v0 call sites that
construct via `..Default::default()` stay source-compatible."*

**That claim was false for tests/e2e.rs callers.** Three sites in
`crates/server/tests/e2e.rs` (lines 917, 3170, 3811) construct
`JuryAssignmentInsertForm` with explicit-field struct literals — NOT
`..Default::default()`. The drift-fix introduced a compile-break across
those 3 sites the moment it landed. The break was discovered only when
impl ran a Task-4 baseline clippy check on the same branch tip.

**Root cause:** R5.1 (`feedback_insertform_default_propagation.md`)
captures exactly this pattern from JM-a's §10.7 GOTCHA: *"adding fields
to a struct-with-Default-derive requires `..Default::default()` at every
caller site; enumerate via `rg -l '<FormName> {' crates/` before
planning."* The drift-fix authored the field correctly (Default-derive
+ Option<_>) but **did not run the `rg -l` enumeration step** on
`crates/`, so the test callers were not surfaced as needing
propagation.

**Resolution path (in-flight):** impl will land a follow-up that adds
the `selected_under_constraints: None` line to the 3 e2e.rs sites
(matching the production-caller propagation Task 4 already did in
admin_assign_jury.rs / admin_emergency_remove.rs / decline_jury_assignment.rs).
Either as part of the chore-clippy commit (Event 1) or as a separate
mechanical chore commit. Advisor recommended option (b) — separate
commit — to keep the drift-fix's audit signal intact.

**This is the SECOND firing of R5.1 in the JM phase.** JM-a R5.1 was
caught by ralph loop's Task 5 cargo check; JM-b R5.1-redux was caught
by impl's Task-4 clippy baseline. The lesson held in JM-a's plan §10.7
GOTCHA wording (after the JM-a retro fix); it did NOT transfer to the
mid-phase drift-fix shape (`chore(v1-JM-a-drift):*` commits don't go
through the plan §10 GOTCHA gate).

**Plan-amendment recommendation:**

1. **Add a plan-template rule:** *"Any `chore(*-drift):*` commit that
   adds fields to a struct with `derive(Default)` MUST include the
   `rg -l '<FormName> {' crates/` enumeration step in the commit body,
   listing every caller site touched (or explicitly noting 'all callers
   use `..Default::default()` — enumeration empty')."* This pulls the
   plan §10.7 GOTCHA discipline out of the plan file and into the commit
   convention, so off-plan chore commits inherit it.

2. **Update `feedback_insertform_default_propagation.md`** to extend
   the "How to apply" section with: *"This rule applies to off-plan
   `chore(*-drift):*` commits too. The drift-fix author MUST run the
   enumeration step in the same commit message body — listing every
   caller site found, or explicitly noting 'all callers compatible via
   `..Default::default()`'. JM-b R5.1-redux (cb7b6bdb2) is the pattern
   that triggered this clarification."*

**Memory-note suggestion:** the existing `feedback_insertform_default_propagation`
already covers the rule. Update its body with the chore-commit
extension; do NOT create a new file (avoid memory bloat).

---

## Event 3 — Plan §13 Task 4/5 vs Task 9 + DoD clippy-flag inconsistency

**Observed (during Event 1 analysis, 2026-04-24):** plan inspection
revealed that Task 4 (line 1178) and Task 5 (line 1210) inline VALIDATE
clippy commands use:

```bash
scripts\\brehon\\cargo-clippy.bat --workspace --features full -- -D warnings
```

(no `--no-deps`), while Task 9 (line 1292) and DoD §15.1 (line 1386) and
acceptance §16 (line 1463) all use:

```bash
scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings
```

(with `--no-deps`).

The Task 9 GOTCHA explicitly rationalises `--no-deps` as the canonical
form for avoiding upstream-Lemmy lint noise:

> Clippy's --features full + --no-deps per
> .claude/rules/pre-phase-harness-audit.md DoD footguns — or narrow to
> -p lemmy_api if upstream lints are noisy. The workspace-level command
> with --no-deps is the canonical form.

So **Task 4/5's per-task checkpoints are running the WIDER form
(without --no-deps), which catches transitive-dep lints that Task 9 and
DoD §15.1 deliberately suppress.** This means Task 4/5 per-task
checkpoints will sometimes show false-red lints (in upstream-Lemmy
deps) that Task 9 and §15.1 are designed to ignore.

**Why this matters:** in the JM-b clippy-debt situation, the
`--workspace --features full -- -D warnings` baseline at Task 4 might
include transitive-dep lints that aren't in §15.1's `--no-deps` baseline.
Impl's "~10 errors before I added a single line" count might mix
own-debt with transitive-dep noise — the count under `--no-deps` could
be smaller (or different).

**Plan-amendment recommendation (for JM-c/d/e + this JM-b mid-flight):**

1. **Align all §13 Task 2-9 clippy commands + §15.1 + §16 to use the
   SAME flag set:** `--workspace --features full --no-deps -- -D warnings`.
   The `--no-deps` flag should be uniform across per-task checkpoints
   and DoD gates so the per-task signal matches the gate signal.

2. **JM-b mid-flight fix:** before impl runs the next Task 4 baseline
   clippy check, swap the per-task command to add `--no-deps`. This is
   a 0-LoC plan edit (one flag added in 2 places) that doesn't require
   a plan-amendment PR — it's a plan-wording correction inline. Either
   advisor edits the JM-b plan file in a `docs(plan): align Task 4-5
   clippy commands with DoD --no-deps form` commit, OR impl edits in a
   companion commit during the chore-clippy work.

3. **Memory-note (low priority):** consider a new feedback-memory entry
   `feedback_clippy_no_deps_uniform.md` — title "All clippy validate
   commands across §13 + DoD must use the same flag set (--no-deps)".
   Or fold into the existing `feedback_clippy_test_style.md` as an
   addendum.

**Caveat:** I have NOT confirmed by running clippy whether
`--workspace --features full -- -D warnings` and `--workspace --features
full --no-deps -- -D warnings` produce different counts on the JM-b
branch tip. Both might happen to produce the same 10 errors today
(governance-v0 fork avoids upstream-Lemmy lint debt by not modifying
upstream files). So this event is a **plan-hygiene** issue, not a
correctness-bug, until measured.

---

## Event 4 — Meta-pattern: gate-keeper Task 0 steps skipped under plan-write time pressure

**Observation:** Events 1 and 2 above share a structural pattern:

| Phase | Gate-keeper task | Skipped step | Cost when fired |
|---|---|---|---|
| JM-a R10.1 | Pre-Task-10 mig count audit | `git log governance-v0..` to enumerate post-AD-a migrations | ~30 min advisor-relay round trip + R10.1 retro entry |
| JM-b Event 1 | Task 0 clippy baseline | `pre-phase-harness-audit §3` clippy baseline capture | ~1 DQ + chore-commit insertion mid-phase |
| JM-b Event 2 | `chore(*-drift):` commit hygiene | `rg -l '<FormName> {' crates/` enumeration | ~3 caller breakages discovered at Task 4 baseline |

**Pattern:** gate-keeper steps that are *defined* in `.claude/rules/*.md`
or memory entries get **dropped from per-phase plan §13 Task 0** when
the plan author copies a previous phase's Task 0 verbatim and the
inherited rules carry the gate implicitly. Plan-template drift =
gate-keeper drop.

**Pattern-level fix recommendation:**

1. **Hard-code the gate-keepers into a phase-template `.md` file** that
   plan §13 Task 0 references by inclusion. Today, plan §13 Task 0
   spells out probes inline. A better shape would be: "Plan §13 Task 0
   = run all probes in `.claude/rules/pre-phase-harness-audit.md`
   §0/§1/§2/§3/§4 + plan-specific probes." That way the plan author
   cannot accidentally drop a probe by copy-paste error — the
   `pre-phase-harness-audit.md` rule is the canonical list.

2. **Add a "Task 0 audit completeness check" to the `/prp-implement`
   command template.** When impl starts a phase, the command should
   verify that Task 0 enumerates all 5+ probes in
   `pre-phase-harness-audit.md` §0-§4 (Docker, wrapper sanity 1-4,
   clippy baseline, DoD smoke test). If the plan's Task 0 is missing
   any of those, the command flags it and asks for plan amendment
   before Task 1.

3. **Captured for advisor-handover-skill input** (per
   `project_handover_skill_retro_pending.md`): the future `/handover`
   skill should bake "Task 0 completeness check" into its handover
   brief schema — the cold-resuming session sees explicitly which
   gate-keepers have been run and which are pending. Tied to the
   `prd_refs` frontmatter approach noted in JM-a retro §3.2 row 4.

**Priority:** MEDIUM-HIGH. Pattern fired twice in JM-a + JM-b combined;
the JM-b retro should propose at least the rule-template inclusion fix
(option 1 above).

---

## Forward-state for retro author (whoever writes `v1-JM-b-retro.md`)

When this phase closes and someone writes the JM-b retro:

1. Pull all 4 events above into JM-b retro §2 "What surprised."
2. Promote the plan-amendment recommendations into JM-b retro §3.2
   "Plan amendments for JM-c/d/e" (extending the JM-a §3.2 table —
   currently rows 1-4a, JM-b adds rows 5-8 corresponding to events 1-4).
3. Record the meta-pattern observation (Event 4) as a JM-b retro §2
   "Pattern" entry — this is now the second phase in a row exhibiting
   gate-keeper-drop. If JM-c/d/e show the same shape, it's a clear
   carry-forward to a Brehon-wide rule-template rather than per-phase
   plan-amendment.
4. Update memory entries per the per-event suggestions (Event 1: maybe
   new file; Event 2: extend existing; Event 3: skip or fold; Event 4:
   maybe new file for the meta-pattern). Aim for net memory delta ≤ +2
   files.

---

_Maintainer: advisor session(s) write to this file mid-phase. Impl session
reads optionally. Phase-close retro author consumes this file and merges
into `v1-JM-b-retro.md`. Once that retro lands, this file can be
gitignored or removed — its content is fully extracted upward. Don't
delete it pre-merge — JM-b PR review may reference these events._
