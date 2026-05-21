# Plan: v1-rls-r1 — RLS hardening Wave 1 (top-5 RLS-PMD review items)

> **DELIVERY NOTE (planner — 2026-05-21):** this file is the plan
> deliverable. The canonical target path
> `.claude/PRPs/plans/v1-rls-r1.plan.md` is blocked for Junior
> worker writes by Claude Code's built-in sensitive-file protection
> (every `.claude/**` path triggers the prompt regardless of
> `settings.json` `permissions.allow`). Same constraint observed at
> `.claude/PRPs/plans/v1-federation-inbound-a.plan.md`'s delivery
> note (2026-05-16). The advisor laptop session (which has full
> Write authority) MUST move this file to its canonical path BEFORE
> plan approval:
>
> ```
> git mv PLAN_DELIVERABLE.md .claude/PRPs/plans/v1-rls-r1.plan.md
> ```
>
> The plan body is unchanged — only the path changes on `mv`. The
> §15 DoD smoke + §16a story-grain checkpoints apply as written.
>
> **Planner DQ pre-seeds (§19.1) are also blocked** — Junior workers
> cannot write to `.claude/decision-queue.json` either, despite the
> path being explicitly whitelisted in `settings.json`. The four
> `kind: "log"` pre-seeds are described in §19.1 of this plan; the
> advisor must transcribe them to `.claude/decision-queue.json`
> directly when moving this file (or the planner ships them inline
> below — see §19.1 for the verbatim JSON).
>
> **Retro signal for v1-rls-r1 (Task 13):** the `.claude/**` write-
> protection / Junior-worker harness gap recurred again. See the
> v1-fed-in-a retro entry for the prior occurrence. A focused
> harness-side fix (add `.claude/PRPs/plans/**` +
> `.claude/decision-queue.json` to the Junior worker permission
> allowlist) would close this and is itself a candidate v1-rls-r2
> item.

## 1. Summary

v1-rls-r1 ships **five harness-only deliverables** that together close
the load-bearing-on-honesty edges in the brehon-fork Recursive Learning
System (per `docs/research/brehon-rls-pmd-review.md` §2 + §6): **(4.2-
spec)** a tracked `.claude/PRPs/specs/mcp-write-time-embedding.md`
specification of the MCP-server patch that collapses the post-write
embedding window (the patch itself lives in a separate
`MCPs/project-memory-mcp` PR — out of scope here per PRECON-2);
**(4.1)** the tracked `.claude/hooks/pmd-canonical-guard.sh` SessionStart
script that promotes the cross-lane canonical-PMD-path invariant from
documentation to enforcement; **(4.8)** a tracked
`.claude/rules/pmd-invariants.md` rule file consolidating the five PMD-
meta invariants currently scattered as lesson files; **(4.6)** an edit
to `.claude/skills/weekly-review/SKILL.md` adding a new **Step 2c
retro-harvest sweep** that folds the existing read-only retro-harvest
skill into the weekly cadence with output at `.claude/harvest/<iso-
week>.md`; **(4.7)** an additive shell function in
`.claude/hooks/retro-check.sh` that emits a JSONL `retro_bypass` record
on every fail-open path, plus a new dedicated kind-registry doc at
`docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` for
hook-emitted JSONL observability (distinct from the v1 PG-table
governance log).

Each deliverable is a **specification + thin shell/skill artifact**; no
`crates/**`, no migration, no Rust, no Shape-G workflow. The sub-phase
ships under the **`validate-pending-laptop` DoD shape** (per PRECON-7;
Shape G suspended until 2026-06-01). Headline acceptance: `git diff
governance-v0..phase-v1-rls-r1` at plan-approval shows only
`.claude/PRPs/specs/` (new), `.claude/rules/pmd-invariants.md` (new),
`.claude/hooks/pmd-canonical-guard.sh` (new), edits to
`.claude/hooks/retro-check.sh`, edits to
`.claude/skills/weekly-review/SKILL.md`, a new lesson file
`.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`
(authored in this sub-phase to close the brief's PRECON-9 reference;
file currently absent at HEAD `6e16ee94f`), new lessons under
`.claude/lessons/`, edits to `.claude/rules/advisor-orchestrator.md` +
`docs/brehon-law-inspired-network/04-data-model-and-api.md`, the new
`docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`, this
plan file, the dogfood report, and the retro. The §15 `cargo` runs
exit 0 (sanity-only — confirm no incidental Rust breakage).

## 2. Source

- `.claude/PRPs/briefs/v1-rls-r1-planning-1.md` @ phase-v1-rls-r1
  HEAD `91d0eda6f` — the advisor brief (post-`/brehon-clarify`; DQ
  #296–#302 resolved 2026-05-20, binding). The PRECON-1..PRECON-10
  decisions in the brief §0.1 are the contract.
- `docs/research/brehon-rls-pmd-review.md` @ `governance-v0` —
  the RLS-PMD review the brief absorbs. §1 (five-tier architecture),
  §2 (eight-stage retro flow), §3 (PMD pipeline), §4.1 / §4.2 / §4.6 /
  §4.7 / §4.8 (the five hardening specs), §5 (autonomy-readiness
  criteria), §6 (top-5 leverage table — the verbatim source of the
  five-item slate).
- `.claude/PRPs/templates/plan.template.md` — canonical 20-section
  schema. Mirrored verbatim.
- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` — **structural
  mirror for §15 laptop-DoD shape** (per PRECON-7 — Shape G suspended).
  fed-in-a's §15.1/§15.2/§15.3/§15.5 shape is mirrored; §15.4 migration
  round-trip is N/A here (v1-rls-r1 ships zero migrations).
- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — **structural
  mirror for harness-only Track A/B/C task shape** (per `feedback_
  read_canonical_before_writing_spec.md`); read for §13 task body
  conventions (SKILL.md authoring tasks, dogfood task shape, paired-
  lesson task shape).

### Lessons that bind §13 decisions

- `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` — IS
  the spec for Track B Task 3. Lift the script's WARN-not-FAIL
  semantics + path-resolution one-liner verbatim. Closes its own
  `fix status: PENDING` self-confession on ship.
- `feedback_pmd_cross_lane_canonical_db.md` — the v1-ship-1 incident
  driving Track B Task 3. Diagnosis recipe at §"Diagnosis recipe"
  is the manual mitigation the SessionStart hook automates.
- `feedback_pmd_backfill_after_write.md` — the gap Track A Task 1
  spec'd-not-shipped contract closes. Step 5.5 + weekly-review §1b
  references stay in place until the MCP patch ships in a separate
  PR.
- `feedback_pmd_two_memory_systems_distinction.md` — invariant #2 in
  Track A Task 2's rule file.
- `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` —
  the meta-pattern v1-rls-r1 enforces. Every documentary guard in
  this plan pairs with a shipped enforcement OR an explicit
  `enforcement: PENDING` note.
- `feedback_principles_not_rules.md` — WARN-vs-FAIL discipline +
  anti-keyword-stuffing in `description:` fields. Watchpoint #1 and
  #5 enforce.
- `feedback_settings_local_json_worktree_bootstrap.md` — per-worktree
  + gitignored discipline Track B Task 6's wiring follows.
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` — Task 13 retro
  authoring discipline.
- `feedback_complexity_score_pre_split.md` — v1-rls-r1 scores **6**;
  below the Sonnet `> 8` split threshold; no split-DQ filed.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
  FILES YAML (`creates:` + `modifies:` + `requires:` where needed).
- `feedback_parallel_cohort_dispatch.md` — `[P]` markers per disjoint
  FILES YAML. Cohort A (5-way `[P]`) + serial chain after.
- `feedback_dogfood_slash_command_specs.md` — pre-commit walkthrough
  discipline applies to the plan itself; Task 10 dogfoods the three
  behaviour-changing artifacts (4.1 hook, 4.6 skill step, 4.7
  instrumentation).
- `feedback_advisor_watchpoint_specificity.md` — §4 watchpoints cite
  specific file:line / function / table.
- `feedback_read_canonical_before_writing_spec.md` — every impl-task
  worker reads 1-2 canonical siblings before authoring.

### Related prior plans

- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — parallel
  meta-skill plan; structural-shape mirror for Track A/B/C splits.
- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` — laptop-DoD
  mirror per PRECON-7.

### ADRs

**No new ADR.** Per PRECON-5 + brief §2.2 "No new ADR.". This
sub-phase enforces existing invariants only — promoting documentary
guards to shipped enforcement, registering one new hook-emitted JSONL
kind in a dedicated observability doc (NOT the ADR corpus), and folding
an existing skill into a higher-cadence call point. No design
decision is changed; the ADR corpus (1-15) is untouched.

## 3. Problem statement

The RLS architecture is sound (per RLS-PMD review §1 + Appendix); the
execution gaps are concentrated at the edges between tiers, between
systems, and between specification and enforcement. Five concrete edges
are open at HEAD `6e16ee94f`:

1. **`feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
   carries `fix status: PENDING`.** The lesson IS a spec; nothing
   automated reads it. The 2026-05-18 v1-ship-1 incident
   (~27+ false `retro-check.sh` Stop-hook blocks; 21 retros stranded
   in a lane-local PMD; ~27+ token taxes across a whole phase) is
   the historical proof. The documentary guard at
   `.mcp.json.example` (the `_comment_pmd_cross_lane` key, line 16
   at HEAD) does not prevent the failure because nothing re-reads
   it. **Tracks to Task 3 + Task 6.**
2. **`memory_write_eval` writes the FTS5 row but not the vector** —
   verified by source inspection of `MCPs/project-memory-mcp/
   dist/index.js` per `feedback_pmd_backfill_after_write.md`. Every
   Junior retro is FTS5-only until weekly-review's safety net runs
   (worst-case 6.5-day window). `memory_search_hybrid` silently
   FTS5-degrades for unembedded rows; the +62.7% Recall@10
   advantage (per `.claude/rules/pmd-search-strategy.md`) is absent
   during the window. The fix is upstream (MCP server patch) and
   out of scope for v1-rls-r1 (PRECON-2); v1-rls-r1 ships the
   **specification** that the future MCP-side PR implements.
   **Tracks to Task 1.**
3. **48 unchecked retro proposals across 52 retro files** (per
   RLS-PMD review §4.6 evidence; checked 2026-05-16). A
   `cycle_count ≥ 3` catch-fire proposal sat unread for 7 days
   before it would have prevented a ~123-min loss on v1-SL-c-2
   cycle-3 (per
   `feedback_plan_stub_uniformity_with_canonical_sibling.md`
   §"cycles cost ~123 min"). The `.claude/skills/retro-harvest/
   SKILL.md` skill exists (read-only, ~431 lines, fully spec'd) —
   nothing calls it on a cadence. **Tracks to Task 4 + Task 5.**
4. **Five PMD-meta invariants are scattered as lesson files** —
   canonical-path / two-systems / no-write-time-embedding /
   LESSON-trailer / SessionStart-guard. They are **non-negotiable
   system invariants**, not heuristics, but their location
   (`.claude/lessons/feedback_*.md`) classifies them alongside
   recurrence-based pattern guidance. The category confusion makes
   them feel optional. **Tracks to Task 2.**
5. **`retro-check.sh` fail-open after 3 attempts leaves no trail.**
   The 3-attempt cap is by design (loops are real); the
   bypass surface is that a determined agent gets a free third
   pass with no retro written and **no signal**. The fail-open
   branch lives at `.claude/hooks/retro-check.sh` lines 141-144
   (`rm -f "$SESSION_FILE"; exit 0`). For autonomy (per RLS-PMD
   review §5.2: "Autonomy = the bypass rate is monotonically
   decreasing"), this trail must exist. **Tracks to Task 7 + Task
   8 + Task 9.**

These five edges share one structural property: each is an invariant
that EXISTS in the system's documentation, lesson corpus, or
architectural prose — but enforcement is either missing entirely
(items 1, 3, 5), specified in a separate repo (item 2), or scattered
across the wrong memory category (item 4). v1-rls-r1 closes the
spec-to-enforcement boundary for the top-5-leverage items per the
RLS-PMD review's leverage calculation (downstream collapse × autonomy
criterion unblocked ÷ implementation cost).

## 4. Solution statement

Three behaviour-changing tracks plus one closeout track, fused into one
plan, sharing the §15 laptop-DoD validation gate.

**Track A — Doc + spec artifacts (zero compile dependency).**
The `.claude/PRPs/specs/` directory does not exist at HEAD `6e16ee94f`
(verified 2026-05-20). Task 1 establishes the directory with a 5-line
`README.md` describing its purpose ("tracked specifications for
contracts that live outside brehon-fork — e.g. MCP server patches,
external-repo coordination docs"), then authors the MCP write-time
embedding spec. Task 2 authors the consolidating `.claude/rules/pmd-
invariants.md` rule file with the five invariants listed verbatim from
the brief §0.1.5. Both files take effect by being **cited** by other
tracks — they unblock Tracks B+C by providing the single citation
target.

**Track B — SessionStart hook + skill edits (behaviour-changing,
brehon-fork-local).** Task 3 authors `.claude/hooks/pmd-canonical-
guard.sh` — a tracked WARN-not-FAIL hook that compares the running
worktree's `.mcp.json` `PROJECT_MEMORY_DB` against the
git-common-dir → canonical resolution (same source-of-truth the
`retro-check.sh` already uses) and emits a stderr WARN naming both
paths on mismatch; exits 0 always. Task 4 inserts a new **Step 2c
retro-harvest sweep** into `.claude/skills/weekly-review/SKILL.md`
between current Steps 2b and 3 (per DQ #296 — adjacent to "promotion-
candidate surfacing", zero downstream renumber). Task 5 adds
`.claude/harvest/` and `.claude/governance-log/` to `.gitignore`
(per PRECON-9 lean — runtime journal output). Task 6 authors a NEW
`.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`
(absent at HEAD per the verification step; the brief assumes its
presence but the file does not exist on this worktree — the planner
files this as `kind: "log"` so future v1-rls-r1 retro can audit) with
a step `(N+1) wire pmd-canonical-guard.sh SessionStart entry in
.claude/settings.local.json`, including the exact JSON snippet to
paste; documents the manual dual-wire in both the canonical
`brehon-fork` checkout and the lane-dedicated `brehon-fork-rls-r1`
checkout (per DQ #301 resolution).

**Track C — `retro-check.sh` governance-log instrumentation (additive,
narrow scope).** Task 7 APPENDS one shell function
`emit_retro_bypass_log` to `.claude/hooks/retro-check.sh` invoked from
the existing fail-open branch (line 142-143). The function writes
one JSONL record per fail-open to
`.claude/governance-log/retro-bypass.jsonl`. Fields per DQ #297:
`timestamp` (ISO 8601 UTC), `session_id`, `attempt_count`,
`prompt_hash` (SHA-256 of `CLAUDE_PROMPT` truncated to 16 hex chars —
the brief renamed the originally-proposed `last_assistant_message_hash`
because line 20 of `retro-check.sh` reads only `CLAUDE_PROMPT`, not
the assistant message), `branch_at_fail_open`, `kind: "retro_bypass"`.
Task 8 authors a NEW doc
`docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` (per
DQ #302) registering the `retro_bypass` kind in a kind-registry table;
adds a one-sentence cross-link from `04-data-model-and-api.md` near
the redaction-service-contract block (line 405 at HEAD — the closest
governance-log reference in 04) pointing at the new doc and explicitly
distinguishing hook-emitted observability from the v1 PG-table
governance log. Task 9 authors a NEW lesson
`.claude/lessons/feedback_retro_bypass_governance_log.md` cross-linking
the instrumentation, the schema kind, and the autonomy-readiness
criterion 5.2.

**Track D — closeout.** Task 10 runs three dogfood probes (per
PRECON-8) and writes the report at
`.claude/PRPs/reports/v1-rls-r1-dogfood-<YYYY-MM-DD>.md`. Task 11
wires v1-rls-r1's outputs into the advisor stage-shape via two surgical
edits to `.claude/rules/advisor-orchestrator.md` (§1 polling-loop
addendum naming the SessionStart guard + §5 new sub-section §5.5
"Retro-bypass observability"). Task 12 authors two paired lesson files
closing the prior PENDING statuses. Task 13 is the retro.

## 5. Metadata

- **Phase:** `v1-rls-r1`
- **Branch:** `phase-v1-rls-r1` (already cut; advisor brief at HEAD
  `91d0eda6f`; planner forks `junior/v1-rls-r1-planning-1` from
  `6e16ee94f` per brief title-line).
- **Target impl-task model:** `sonnet-4-6` (default — harness-only
  tasks; pattern-following from canonical hook/skill/rule siblings).
- **Estimated tasks:** 14 (Task 0 pre-flight + 13 numbered tasks).
- **Estimated cargo budget:** **0 GB** — v1-rls-r1 ships zero Rust
  changes. §15 cargo invocations are sanity-only against an unchanged
  workspace.
- **Forbidden-window applicability:** non-binding under Shape G;
  PRECON-7 ships under `validate-pending-laptop`; **§15 cargo runs
  execute on the laptop, not the EliteDesk** — so the forbidden
  windows in `.claude/rules/advisor-orchestrator.md` §5.1 apply only
  if the user ad-hoc dispatches a worker. The default flow is
  advisor-laptop §5.2 handler (per `.claude/rules/advisor-
  orchestrator.md` §5.2).
- **Complexity score:** **6/10** — see breakdown below.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Threshold for Sonnet
target: file split-DQ if `score > 8`. v1-rls-r1 scores **6**, **below
threshold** — **no split-DQ filed**.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **+6** | 12 numbered tasks (Tasks 1-12) excluding Task 0 (pre-flight) and Task 13 (retro); 12 - 5 = 7 above threshold; planner caps at +6 because 8 of those tasks (1, 2, 3, 4, 5, 7, 11, 12) are pure documentation/script authoring with no compile dependency and ~30 min wall-clock each. |
| Migrations touched | +2 each | **0** | None. |
| Crates touched | +1 each | **0** | None. v1-rls-r1 ships zero edits under `crates/`. |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **0** | None. |
| New ADR-affecting decisions | +2 each | **0** | Per PRECON-5; no new ADR. |
| Cargo budget peak above 6 GB | +1 per GB | **0** | No Rust changes. |
| **Total** | — | **6** | Threshold `>8`; no split-DQ. |

### 5.2 Per-task complexity ceiling (non-Sonnet target only)

**Not applicable.** Target model is Sonnet. The Sonnet ceiling
(≤4 files per task, ≤2 crates) is the implicit norm; v1-rls-r1's
heaviest task is Task 6 (one new lesson file) and Task 1 (two new
files — `.claude/PRPs/specs/README.md` + the spec doc itself) —
both well below ceiling.

## 6. Relationship to other v1-rls-* sub-phases

- **Prior:** no v1-rls predecessor; v1-rls-r1 is the **first** wave of
  RLS hardening.
- **Follower:** **v1-rls-r2 (planned)** — absorbs the deferred items
  4.3 (`.mcp.json` documentary-guard re-read), 4.4 (Junior post-task-
  retro inline-backfill PostToolUse hook — collapses if MCP write-time
  embedding ships first), 4.5 (`sync-lessons-to-pmd.sh` auto-invoke),
  4.9 (auto-phase 10-category mandatory section), 4.10
  (observation-capture.sh consumer in weekly-review Step 4). Per
  PRECON-5; brief §2.2 enumerates each with rationale.
- **Independent of all active impl sub-phases.** v1-rls-r1 is
  harness-level only. Does not block / is not blocked by:
  - `v1-federation-inbound-c` (the next federation sub-phase) —
    different worktree, different file ownership.
  - `brehon-conformance-audit` (parallel meta-skill plan) — disjoint
    file set; the audit's `axes/4-error-idiom.md` may eventually cite
    `pmd-invariants.md` but no immediate dependency.
- **First production beneficiaries:**
  - The **next lane bootstrap** activates `pmd-canonical-guard.sh` per
    the updated checklist.
  - The **next Sunday weekly-review** (post-merge) fires the new Step
    2c retro-harvest sweep.
  - **Any retro-check.sh fail-open** after this PR merges begins
    accumulating the JSONL trail immediately.

## 7. Preflight guardrails

- **R1: WARN-not-FAIL discipline on hook scripts.** Per
  `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
  §"Why a SessionStart WARN". Every hook v1-rls-r1 ships or modifies
  exits 0 on every path. Watchpoint #1 enforces.
- **R2: The five PMD invariants are FIXED.** Per PRECON-5. Track A
  Task 2's rule file enumerates exactly five — canonical-path /
  two-systems / no-write-time-embedding / LESSON-trailer /
  SessionStart-guard. No sixth, no fourth. Watchpoint #2 enforces.
- **R3: No `cargo` invocation from any v1-rls-r1 ship surface.** Per
  PRECON-7. The §15 cargo runs are sanity-only; the deliverables (the
  hook script, the rule file, the skill step body, the doc, the
  lessons) never call cargo. Watchpoint #3 enforces.
- **R4: Mid-task DQ push.** Per `.claude/rules/decision-queue.md`
  "Mid-task visibility" + DQ #300 resolution. Every DQ entry the
  planner OR impl-task workers file mid-task commits + pushes
  immediately so the advisor's `git fetch` picks them up. Watchpoint
  #4 enforces.
- **R5: No auto-trigger keyword-stuffing in `description:` fields.**
  Per PRECON-3 + `feedback_principles_not_rules.md`. Plain-language
  one-sentence descriptions on the new rule file's frontmatter, the
  hook script's leading comment, and every new lesson's frontmatter.
  Watchpoint #5 enforces.
- **R6: `retro-check.sh` fail-open behaviour is PRESERVED.** Per
  Watchpoint #7 + PRECON-2. Track C Task 7's edit is ADDITIVE only
  (function appended + one function-invoke line in the existing
  fail-open branch). The 3-attempt cap, the fail-open exit code (0),
  and the `rm -f "$SESSION_FILE"` cleanup remain.
- **R7: Documentary guards pair with shipped enforcement.** Per
  `feedback_lesson_must_pair_with_structural_fix_when_fixable.md` +
  PRECON-5/6. Every invariant in Track A Task 2's rule file cites
  the SHIPPED enforcement OR carries explicit
  `enforcement: PENDING — see <future-sub-phase>` text. Watchpoint #8
  enforces.
- **R8: Canonical-sibling-mirror discipline.** Per
  `feedback_read_canonical_before_writing_spec.md` + advisor-
  orchestrator §3.6. Every impl-task worker authoring a new artifact
  Reads 1-2 existing canonical siblings before authoring. See §10
  patterns to mirror for the specific citations.
- **R9: Dogfood is the integration test.** Per PRECON-8. Task 10
  has three sub-runs (4.1 / 4.6 / 4.7) and is non-`[P]` `requires:
  [3, 4, 5, 6, 7, 8, 9]`. Watchpoint #6 enforces.
- **R10: Forbidden-window applicability.** Per PRECON-7 + advisor-
  orchestrator §5.1. Junior task-0 pre-flight refuses to start in a
  forbidden window. Advisor honours when queueing.

## 8. Flow design

### Before v1-rls-r1 (current state at HEAD `6e16ee94f`)

```
SessionStart
   │
   ▼
pre-phase-audit.sh   (runs; no canonical-PMD check)
   │
   ▼
(any session work — write to PMD, retros, etc.)
   │
   ▼  retro-check.sh Stop hook
       ├─ retro found → exit 0 (allow stop)
       ├─ retro missing, attempts < 3 → exit 2 (block + WARN)
       └─ retro missing, attempts == 3 → rm session file, exit 0
                                          ↑ NO TRAIL — bypass invisible

Five PMD-meta invariants live as five disconnected lesson files:
  feedback_pmd_cross_lane_canonical_db.md (canonical-path)
  feedback_pmd_two_memory_systems_distinction.md (two-systems)
  feedback_pmd_backfill_after_write.md (no-write-time-embed)
  feedback_junior_pmd_write_convention.md (LESSON-trailer)
  feedback_mcp_canonical_pmd_path_enforce_at_session_start.md (PENDING)

Weekly review (Sunday 02:00 UTC):
  Step 1  → memory_review + prune
  Step 1b → backfill safety net (embedding)
  Step 2  → promotion candidates (sweep)
  Step 2b → lesson clustering (TH only)
  Step 3  → eval metrics aggregation
                ↑ retro-harvest NOT folded here — ad-hoc only
  Step 4  → git hygiene
  Step 5  → summary
  Step 6  → commit
```

### After v1-rls-r1 (post-merge)

```
SessionStart
   │
   ▼
pmd-canonical-guard.sh   (NEW — wired per lane via settings.local.json
   │                       per updated bootstrap checklist)
   │   ├─ canonical match → exit 0 silent
   │   └─ mismatch → stderr WARN naming both paths + 1-line fix; exit 0
   ▼
pre-phase-audit.sh   (unchanged)
   │
   ▼
(any session work — write to PMD, retros, etc.)
   │
   ▼  retro-check.sh Stop hook
       ├─ retro found → exit 0 (allow stop)
       ├─ retro missing, attempts < 3 → exit 2 (block + WARN)
       └─ retro missing, attempts == 3 → rm session file, exit 0
                                          + emit_retro_bypass_log()  ◀── NEW
                                            writes JSONL to
                                            .claude/governance-log/
                                              retro-bypass.jsonl
                                                                  ▼
Weekly review (Sunday 02:00 UTC):
  Step 1   → memory_review + prune
  Step 1b  → backfill safety net (embedding)
  Step 2   → promotion candidates (sweep)
  Step 2b  → lesson clustering (TH only)
  Step 2c  → retro-harvest sweep   ◀── NEW (folds existing
  Step 3   → eval metrics aggregation        retro-harvest skill into
                                             weekly cadence; output to
                                             .claude/harvest/<iso-week>.md)
  Step 4   → git hygiene
  Step 5   → summary
  Step 6   → commit

PMD-meta invariants:
  .claude/rules/pmd-invariants.md   ◀── NEW (consolidates five invariants
                                            as numbered non-negotiable
                                            statements; auto-loads at
                                            SessionStart; lessons stay
                                            as evidence trail)

MCP write-time embedding patch:
  .claude/PRPs/specs/mcp-write-time-embedding.md   ◀── NEW (contract
                                                          authoring; the
                                                          actual patch
                                                          ships in a
                                                          separate
                                                          MCPs/project-
                                                          memory-mcp PR)

Hook-emitted JSONL kinds registry:
  docs/brehon-law-inspired-network/
    governance-log-kinds-jsonl.md   ◀── NEW (kind registry for
                                            hook-emitted observability;
                                            DISTINCT from v1 PG-table
                                            governance log; first entry:
                                            retro_bypass)
```

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit. Grouped
by purpose:

### Canonical artifacts (sibling-mirror sources)

- `.claude/hooks/retro-check.sh` (156 lines at HEAD `6e16ee94f`) —
  Track C Task 7 EDITS this file. Read in full; lines 95-120 are the
  retro-check SQL branches, lines 132-147 are the fail-open path Task
  7 appends to. Also serves as a canonical hook shape for Track B
  Task 3 (`set -euo pipefail`, `git rev-parse --git-common-dir`
  resolution, sqlite3 absence handling).
- `.claude/hooks/pre-phase-audit.sh` (sequencing-comparison source) —
  Read for SessionStart hook idioms; Track B Task 6's
  `settings.local.json` snippet must sequence `pmd-canonical-guard.sh`
  BEFORE `pre-phase-audit.sh` per the lesson's "exact moment the
  human can act" rationale.
- `.claude/hooks/observation-capture.sh` (84 lines) — Read to
  CONFIRM observation-capture.sh is structurally unsuitable for the
  `retro_bypass` trail (lines 14-18: writes to per-PPID transient
  cache files at `~/.cache/tw-observations/<PPID>.jsonl`, gitignored
  shadow-mode). Track C Task 7 uses a DEDICATED sidecar at
  `.claude/governance-log/retro-bypass.jsonl` instead. Lines 73-82
  ARE the canonical `jq -nc --arg` JSONL emit pattern the new
  function mirrors.
- `.claude/skills/weekly-review/SKILL.md` (152 lines) — Track B Task
  4 EDITS this file. Read in full; the new Step 2c inserts between
  Step 2b (line 77) and Step 3 (line 95).
- `.claude/skills/retro-harvest/SKILL.md` (431 lines) — Track B Task
  4 CONSUMES this skill. Read in full; the new Step 2c body either
  invokes the skill by reference (Skill tool call) OR duplicates the
  core sweep logic inline. Planner lean: invoke-by-reference if the
  skill's invocation contract supports it; verify by reading the
  retro-harvest SKILL.md frontmatter + first 50 lines.
- `.mcp.json.example` (HEAD line 16 + line 22 — the
  `_comment_pmd_cross_lane` marker + the canonical absolute
  `PROJECT_MEMORY_DB`). Track B Task 3's path-comparison logic
  treats this value as ground truth (or, equivalently, derives the
  canonical from `git rev-parse --git-common-dir` since both
  resolve to the same canonical absolute).
- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` Tasks 1-2
  (SKILL.md authoring task shape + paired-lesson shape) — Read to
  mirror Track A Task 1 + Task 2 prose + structure.

### Lessons (gate §13 decisions — see §2 for the bind line)

- `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md` (the
  spec for Track B Task 3).
- `feedback_pmd_cross_lane_canonical_db.md` (the incident driving
  Track B; the diagnosis recipe Task 3 automates).
- `feedback_pmd_backfill_after_write.md` (the gap Track A Task 1's
  spec closes).
- `feedback_pmd_two_memory_systems_distinction.md` (invariant #2 in
  the rule file).
- `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`
  (the meta-pattern v1-rls-r1 enforces).
- `feedback_principles_not_rules.md` (R5 / Watchpoint #1 + #5).
- `feedback_settings_local_json_worktree_bootstrap.md` (Track B Task
  6 follows this discipline).
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` (Task 13 retro
  authoring).
- `feedback_dogfood_slash_command_specs.md` (Task 10 dogfood
  discipline).
- `feedback_advisor_watchpoint_specificity.md` (§4 cite-discipline).
- `feedback_read_canonical_before_writing_spec.md` (R8).
- `feedback_complexity_score_pre_split.md` (§5 mechanical score).
- `feedback_explicit_file_arrays_on_tasks.md` (§13 FILES YAML).
- `feedback_parallel_cohort_dispatch.md` (`[P]` discipline).
- `feedback_schema_changing_spec_retrofit_question.md` (§3.8 gate —
  see §19.4 below).

### Rules

- `.claude/rules/advisor-orchestrator.md` (§1 polling-loop, §3.1
  stage-shape, §3.4 DoD smoke, §3.5 watchpoint-specificity, §3.7
  dogfood, §3.8 schema-retrofit, §3.9 verify, §4.1 cohort dispatch,
  §5 validation/classification) — Task 11 EDITS this file (§1 + §5
  additions).
- `.claude/rules/decision-queue.md` (§"Subagents and attribution",
  §"Mid-task visibility", §"kind: blocker vs log", §"Hard refusals
  write-side") — every §13 task's DQ writes follow these rules.
- `.claude/rules/branch-manager.md` (file-ownership: BM Junior task
  that opens the PR must NOT edit hook/script/skill content).
- `.claude/rules/multi-lane-worktree.md` (§"PMD is cross-lane
  shared") — the invariant Track A Task 2's rule file consolidates.
- `.claude/rules/pmd-search-strategy.md` (Track A Task 1 + Task 2
  use plain-language `description:` per this rule's anti-keyword-
  stuffing).

### Architectural prose (for Task 8 cross-link insertion)

- `docs/brehon-law-inspired-network/03-architecture.md` §6
  "Append-only governance log (interface)" — lines 177-196 at HEAD.
  Task 8's new doc explicitly distinguishes "harness JSONL
  observability" from this v1 PG-table product surface.
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` near
  the redaction-service-contract block (line 405) — Task 8 inserts
  the one-sentence cross-link near this anchor (the closest
  governance-log reference in 04).

### Reports (context)

- `docs/research/brehon-rls-pmd-review.md` (296 lines) — the
  authority for the five-item slate. Re-read §6 (top-5 table) and §5
  (autonomy-readiness criteria 5.2 + 5.3 + 5.4) at task author time.
- `.claude/PRPs/reports/session-retro-2026-05-18-pmd-stranding-
  remediation.md` (if present) — the v1-ship-1 retro driving Track
  B. **NOTE FOR PLANNER:** confirmed present at HEAD; if not, file
  `kind: "log"` and proceed (the lesson alone is sufficient to
  drive Task 3).

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: every entry cites a
specific file:line / function name / table / structural pattern at
HEAD `6e16ee94f`. No concept-only mirrors.

### 10.1 SessionStart hook script shape

**Mirror:** `.claude/hooks/retro-check.sh:1-50` for shebang
(`#!/usr/bin/env bash`) + `set -euo pipefail` + early-exit on missing
inputs + `git rev-parse --git-common-dir` resolution pattern.
Specifically lines 32-41 (the DB-path resolution) — Track B Task 3
mirrors this exact pattern to compute the canonical
`PROJECT_MEMORY_DB` target.

```bash
# From retro-check.sh lines 32-41 (HEAD 6e16ee94f) — the canonical
# git-common-dir → canonical-PMD resolution:
GIT_COMMON=$(git rev-parse --git-common-dir 2>/dev/null || true)
if [ -n "$GIT_COMMON" ] && [ "$GIT_COMMON" != ".git" ]; then
  MAIN_REPO=$(dirname "$GIT_COMMON")
  CANONICAL_DB="${MAIN_REPO}/.project-memory/memory.db"
fi
```

Track B Task 3 (`pmd-canonical-guard.sh`) computes the canonical
target by the same path and compares against the running worktree's
`.mcp.json` `mcpServers."project-memory".env.PROJECT_MEMORY_DB`. By
construction the guard and the hook agree.

### 10.2 `.mcp.json` value extraction (UTF-8-safe Python one-liner)

**Mirror:** `feedback_pmd_cross_lane_canonical_db.md` §"Diagnosis
recipe" step 1 (= the lesson's mechanism citation in
`feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
§"Mechanism" step 1) — the Python one-liner extracting the canonical
`PROJECT_MEMORY_DB`. Track B Task 3 uses this verbatim:

```python
python -c "import json,io;print(json.load(io.open('.mcp.json',encoding='utf-8'))['mcpServers']['project-memory']['env']['PROJECT_MEMORY_DB'])"
```

UTF-8 + io.open is load-bearing on Windows (per
`feedback_windows_bash_python_git_show_tmp_traps.md`).

### 10.3 Frontmatter shape — rule files

**Mirror:** `.claude/rules/decision-queue.md` (no frontmatter — pure
markdown) AND `.claude/rules/advisor-orchestrator.md` (no
frontmatter). Rule files in `.claude/rules/` do NOT carry YAML
frontmatter (they auto-load at SessionStart by file presence; no
`name:` / `description:` keys needed). Track A Task 2's
`pmd-invariants.md` follows this convention — first line is
`# PMD invariants` (H1), no `---` block.

### 10.4 Frontmatter shape — lesson files

**Mirror:** `.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_start.md:1-5`:

```markdown
---
name: <plain-language summary, one sentence>
description: <plain-language description, one or two sentences — search-friendly natural language, NO auto-trigger keyword stuffing>
type: feedback
---
```

Track D Task 12's two paired lesson files MUST follow this shape.
Track C Task 9's `feedback_retro_bypass_governance_log.md` MUST
follow this shape. Track B Task 6's
`feedback_phase_lane_worktree_bootstrap_checklist.md` MUST follow
this shape. Per Watchpoint #5: no "DO use when" / "ALWAYS INVOKE" /
"MUST USE" verbiage in the `description:` field.

### 10.5 Skill body Step insertion shape

**Mirror:** `.claude/skills/weekly-review/SKILL.md:23-65` (Step 1b
PMD embedding backfill) — the existing skill-step pattern. Track B
Task 4's new Step 2c follows this shape:

- `### 2c. Retro-harvest sweep` H3 heading
- One-sentence `**Why:**` paragraph
- Sequenced numbered list of operations (Glob → Read → write
  artifact)
- Output artifact path naming convention:
  `.claude/harvest/<iso-week>.md` (per PRECON-9 lean — gitignored;
  Task 5 adds the `.gitignore` entry)
- Final line: "Manual review thereafter — SURFACING, not auto-
  promoting (per RLS-PMD review §4.6 contract)".

### 10.6 JSONL kind registry doc shape

**Mirror:** the brief's PRECON-10 §0.1.10 section text verbatim —
the new doc's structure is:

1. **Scope** (one paragraph) — explicit distinction from v1 PG-table
   governance log.
2. **Kind registry table** (5-column markdown table: `kind name |
   originating hook | sidecar path | JSONL field schema | consumer`).
3. **`retro_bypass` entry** — first row in the table.

The brief authored the prose; Task 8 lifts it verbatim. No new
prose invented.

### 10.7 Spec doc shape (Track A Task 1)

**Mirror:** the brief §0.1.2 (PRECON-2) prose verbatim — the
contract sections are:

- **(a)** Patch insertion point in `memory_write_eval` (post-FTS5
  insert, pre-commit per RLS-PMD review §4.2 +
  `feedback_pmd_backfill_after_write.md` source inspection).
- **(b)** Graceful-fallback contract (write the row, log the miss,
  let weekly-review §1b safety net catch it).
- **(c)** Verification recipe (3-write probe + `SELECT COUNT(*) FROM
  memory_vectors` equals row count immediately after write).
- **(d)** Downstream-collapse claim (post-patch: delete
  `session-retro` Step 5.5 inline-backfill; `weekly-review` Step 1b
  becomes aspirational; the Junior 7-day window vanishes).

The doc explicitly notes: "actual `dist/index.js` patch ships in a
separate `MCPs/project-memory-mcp` PR; this doc is the contract that
PR implements". Per PRECON-2.

### 10.8 PMD invariants — verbatim text

**Mirror:** the brief §2.1 Track A Task 2 enumeration (already lifted
verbatim from the brief §0.1.5 table). Track A Task 2's `pmd-
invariants.md` body uses this text:

> 1. **Canonical PMD path (absolute, cross-lane).** Every worktree's
>    `.mcp.json` `PROJECT_MEMORY_DB` MUST be
>    `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`
>    — never relative, never per-lane. Detection: `bash
>    .claude/hooks/pmd-canonical-guard.sh` (v1-rls-r1 ships).
> 2. **Two systems, one source of truth.** System 1 (auto-loaded
>    markdown under `~/.claude/projects/.../memory/`) and System 2
>    (queryable SQLite-vec DB at the canonical path) are NOT
>    interchangeable; never write retro content directly to System
>    1; never assume System 2 is loaded at SessionStart.
> 3. **No write-time embedding (yet — see item 4.2-spec).** Every
>    `memory_write_eval` writes the FTS5 row but NOT the vector;
>    `backfill.js` must run between write and the next hybrid search
>    OR the search degrades silently to FTS5-only. Mitigations:
>    session-retro inline backfill (laptop-only); weekly-review §1b
>    safety-net (Junior + safety net for missed). The
>    `mcp-write-time-embedding.md` spec ships in v1-rls-r1; the patch
>    ships separately.
> 4. **LESSON-trailer discipline.** Every learning observation either
>    fires a `LESSON:` trailer (in commit body) OR a `kind: "log"`
>    DQ entry (mid-task push, resolved-immediately). Never both,
>    never neither for a durable observation.
> 5. **SessionStart canonical-PMD guard (post-v1-rls-r1).** The
>    tracked `pmd-canonical-guard.sh` hook runs at every session
>    start; lane drift surfaces as a loud WARN within seconds of
>    session-start, not after a phase of stranding. Per
>    `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`.

Each invariant in the rule file body carries: a one-paragraph **Why
this is non-negotiable** with the originating incident citation; a
**How to apply** line naming the SHIPPED enforcement (or
`enforcement: PENDING + see <future-sub-phase>`); a **See also** block
linking to the originating lesson(s).

### 10.9 Brief-Scope output check pattern

**Mirror:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md`
§16a Story 1 — the "Brief-Scope outputs to verify" bullet list. Each
v1-rls-r1 §16a Story names structural patterns that
`/brehon-verify` can grep for (e.g. `grep -c "^### " .claude/skills/
weekly-review/SKILL.md` returns ≥ 9; `test -x .claude/hooks/
pmd-canonical-guard.sh` exits 0; `head -5 .claude/PRPs/specs/
mcp-write-time-embedding.md` shows the expected heading).

## 11. Files to change

By directory/component, grouped per task. The planner has confirmed
none of these paths require a `cargo metadata` precheck (no
`crates/**` involvement).

### `.claude/PRPs/specs/` (NEW directory)

- `.claude/PRPs/specs/README.md` — 5-line directory description
  (Task 1).
- `.claude/PRPs/specs/mcp-write-time-embedding.md` — the item 4.2-
  spec contract doc (Task 1).

### `.claude/rules/`

- `.claude/rules/pmd-invariants.md` — 5 PMD-meta invariants
  consolidated as numbered non-negotiable statements (Task 2).
- `.claude/rules/advisor-orchestrator.md` — §1 polling-loop one-line
  addendum (SessionStart guard active) + new §5.5 sub-section
  "Retro-bypass observability" (Task 11).

### `.claude/hooks/`

- `.claude/hooks/pmd-canonical-guard.sh` — NEW SessionStart guard
  script (Task 3).
- `.claude/hooks/retro-check.sh` — ADDITIVE function append +
  one-line invocation in existing fail-open branch (Task 7).

### `.claude/skills/`

- `.claude/skills/weekly-review/SKILL.md` — INSERT new Step 2c
  between current Step 2b and current Step 3 (Task 4).

### `.claude/lessons/`

- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`
  — NEW lesson (planner verified absent at HEAD `6e16ee94f`; brief
  assumed presence but file does not exist on this worktree).
  Task 6 authors with: (a) general "lane-bootstrap-checklist as
  durable record" framing per
  `feedback_settings_local_json_worktree_bootstrap.md`, (b) the
  numbered N+1 step "wire `pmd-canonical-guard.sh` SessionStart
  entry in `.claude/settings.local.json`" with the exact JSON
  snippet, (c) documentary note on the manual dual-wire in both
  canonical `brehon-fork` + lane-dedicated `brehon-fork-rls-r1`
  per DQ #301.
- `.claude/lessons/feedback_retro_bypass_governance_log.md` — NEW
  lesson cross-linking the instrumentation (Task 9).
- `.claude/lessons/feedback_pmd_canonical_guard_enforces_invariant.md`
  — NEW lesson closing the prior PENDING status (Task 12).
- `.claude/lessons/feedback_retro_harvest_weekly_cadence.md` — NEW
  lesson closing the retro-harvest ad-hoc gap (Task 12).

### `.gitignore`

- `.gitignore` — ADD `.claude/harvest/` line + `.claude/governance-
  log/` line (Task 5).

### `docs/brehon-law-inspired-network/`

- `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` —
  NEW dedicated doc per DQ #302 (Task 8).
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` —
  ADD one-sentence cross-link near the redaction-service-contract
  block (line 405 at HEAD) (Task 8).

### `.claude/PRPs/`

- `.claude/PRPs/plans/v1-rls-r1.plan.md` — this file (planning
  Junior; pre-move from `PLAN_DELIVERABLE.md` per delivery note).
- `.claude/PRPs/reports/v1-rls-r1-dogfood-<YYYY-MM-DD>.md` — Task
  10 dogfood report.
- `.claude/PRPs/reports/v1-rls-r1-retro.md` — Task 13 retro.

### Manual hand-off (NOT in any §13 task's FILES YAML — out-of-band)

Per PRECON-9 / DQ #301 + `feedback_settings_local_json_worktree_
bootstrap.md`, the two `settings.local.json` files are gitignored
per-worktree config and CANNOT propagate via git. Task 6's body
documents the exact JSON snippet for the user/advisor to paste
post-merge into:

- `C:/Users/barri/Developer/brehon-fork/.claude/settings.local.json`
  (canonical checkout — durable beyond phase-v1-rls-r1 worktree
  removal).
- `C:/Users/barri/Developer/brehon-fork-rls-r1/.claude/settings.
  local.json` (this lane — immediate dogfood).

The two files are **NOT listed in Task 6's `creates:` / `modifies:`
arrays** because gitignored files cannot be the unit of git-tracked
deliverables. Task 6's prose documents the post-task hand-step.

## 12. NOT building in v1-rls-r1

Per the brief §2.2. Each entry pairs a tempting addition with the
deferral pointer.

- **The actual `MCPs/project-memory-mcp/dist/index.js` patch for
  write-time embedding.** Deferred to a focused
  `MCPs/project-memory-mcp` PR after v1-rls-r1 ships the spec
  (PRECON-2). Reason: cross-repo Junior orchestration + MCP-server
  release cadence dilutes scope.
- **`ENABLE_PROMPT_CACHING_1H=1` worker-env rollout.** Deferred to a
  separate token-economy lane (PRECON-3).
- **`cargo-nextest` adoption.** Deferred to follow-up plan after
  first conformance-audit calibration cycle (PRECON-3).
- **CodeRabbit CLI generate→review→fix loop.** Deferred (PRECON-3).
- **rust-analyzer-MCP scope beyond what conformance-audit installs.**
  Deferred (PRECON-3).
- **RLS-PMD review items 4.3 / 4.4 / 4.5 / 4.9 / 4.10.** Deferred to
  `v1-rls-r2` (PRECON-5). One-line each:
  - 4.3 `.mcp.json` documentary-guard re-read hook — adjacent to
    4.1; lower leverage; bundle into v1-rls-r2.
  - 4.4 Junior post-task-retro inline-backfill PostToolUse hook —
    collapses if 4.2 patch ships first; preferred path is 4.2 first.
  - 4.5 `sync-lessons-to-pmd.sh` auto-invoke — bundle into v1-rls-r2
    weekly-review extensions.
  - 4.9 auto-phase 10-category mandatory section — orthogonal to
    the autonomy-substrate framing; bundle into a session-retro-
    skill lane.
  - 4.10 observation-capture.sh consumer — bundle into v1-rls-r2
    weekly-review Step 4.
- **No new ADR.** v1-rls-r1 enforces existing invariants; no new
  design decision. Per PRECON-5.
- **No auto-promote of harvest-surface proposals to
  `.claude/lessons/`.** Per RLS-PMD review §4.6 contract — manual
  review thereafter; the skill SURFACES, the human PROMOTES.
- **No edit to `retro-check.sh`'s fail-open behaviour itself.** The
  3-attempt cap, the fail-open exit code (0), the `rm -f
  "$SESSION_FILE"` cleanup remain. Instrumentation is additive only.
  Per RLS-PMD review §4.7 + R6.
- **No auto-wiring of `pmd-canonical-guard.sh` in lanes other than
  the canonical brehon-fork + this lane** (PRECON-9 / DQ #301). Per-
  lane bootstrap-checklist propagation is the mechanism for other
  active lanes (e.g. `brehon-fork-conformance-audit`,
  `brehon-fork-tooling`).
- **No edit to MCP server source from this repo.** PRECON-2 hard
  refusal.
- **No subagent variant of the canonical-guard.** The hook is
  inline-invoked at SessionStart; subagent variant is anti-pattern
  per cost model + PRECON-3.
- **No `webauthn-rs` step-up gate.** Out of v1-rls scope.
- **No `cargo` invocation from any v1-rls-r1 ship surface.** §15
  cargo runs are sanity-only against the existing wrapper scripts;
  no script/hook/skill in v1-rls-r1 invokes cargo at runtime (R3).
- **No auto-trigger keyword-stuffing in any rule/skill/lesson
  `description:` field.** Per R5 + PRECON-3.
- **Cross-links to `feedback_one_system_memory_in_repo.md`,
  `feedback_lesson_mirror_check.md`,
  `feedback_advisor_watchpoint_specificity.md`.** The planner
  verified these files are ABSENT at HEAD `6e16ee94f` on this
  worktree (`junior/role-planning-v1-rls-r1...`-367). The brief
  authored against an aspirational corpus; the cited lessons may
  live on `governance-v0` (planner did not cross-check the trunk
  state) OR the references are forward-only. Track D Task 12's
  paired lesson files reference
  `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`
  (verified present) instead; absent cross-links are SKIPPED.
  Filed as `kind: "log"` (see §19.1 pre-seed #1).

**Hard out-of-scope (per the broader v1 Brehon platform):** auto-
apply (v3 / ADR-006), reputation portability (v2/v3), cross-instance
jury (v3), OPA federation policy (v2). v1-rls-r1 hardens the
*harness*; it does NOT change Brehon governance behaviour.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md` code-only-via-PR rule + `feedback_
explicit_file_arrays_on_tasks.md` FILES YAML discipline).

> **Cohort dispatch:** the advisor groups consecutive `[P]`-marked
> tasks into a cohort and queues them simultaneously. v1-rls-r1's
> **Cohort A** = Tasks 1, 2, 3, 4, 7 (all `requires: [0]`, disjoint
> FILES YAML). Tasks 5, 6, 8, 9 serialize behind their `requires:`
> entries. Tasks 10, 11 are barriers near phase end. Task 12 (two
> internally `[P]` lesson files) runs after Task 10. Task 13 (retro)
> is the terminal barrier.

> **Shape G note:** PRECON-7 — Shape G is SUSPENDED until
> 2026-06-01. Every §13 task body composes as edit + commit + push;
> per-task validation runs via the `validate-pending-laptop` handler
> (advisor-orchestrator §5.2) — the advisor laptop session runs §15
> commands and mutates the DQ entry.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-rls-r1`; confirm
branch is a `junior/*` worktree off `phase-v1-rls-r1`; confirm
prior phase deliverables (the brief) present; confirm baseline
workspace state intact.

**FILES (machine-parseable):**

```yaml
creates: []
modifies: []
```

**No commit at Task 0** — verification only.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — branch verification (impl-task workers fork from phase-v1-rls-r1)
git branch --show-current
# EXPECT: junior/<impl-task-slug> (Junior worktree branch off phase-v1-rls-r1)

# Probe 1 — phase-branch tip accessible
git log --oneline phase-v1-rls-r1 -n 1
# EXPECT: non-empty (advisor brief commit present)

# Probe 2 — brief file present
test -f .claude/PRPs/briefs/v1-rls-r1-planning-1.md
echo "exit: $?"
# EXPECT: exit 0

# Probe 3 — clean working tree
git status --short
# EXPECT: empty

# Probe 4 — .claude/PRPs/specs/ does NOT yet exist (Task 1 creates)
test ! -d .claude/PRPs/specs
echo "exit: $?"
# EXPECT: exit 0

# Probe 5 — .claude/rules/pmd-invariants.md does NOT yet exist (Task 2 creates)
test ! -f .claude/rules/pmd-invariants.md
echo "exit: $?"
# EXPECT: exit 0

# Probe 6 — .claude/hooks/pmd-canonical-guard.sh does NOT yet exist (Task 3 creates)
test ! -f .claude/hooks/pmd-canonical-guard.sh
echo "exit: $?"
# EXPECT: exit 0

# Probe 7 — .claude/hooks/retro-check.sh exists (Task 7 modifies)
test -f .claude/hooks/retro-check.sh
echo "exit: $?"
# EXPECT: exit 0

# Probe 8 — .claude/skills/weekly-review/SKILL.md exists (Task 4 modifies)
test -f .claude/skills/weekly-review/SKILL.md
echo "exit: $?"
# EXPECT: exit 0

# Probe 9 — .claude/skills/retro-harvest/SKILL.md exists (Task 4 references)
test -f .claude/skills/retro-harvest/SKILL.md
echo "exit: $?"
# EXPECT: exit 0

# Probe 10 — docs/brehon-law-inspired-network/04-data-model-and-api.md exists (Task 8 modifies)
test -f docs/brehon-law-inspired-network/04-data-model-and-api.md
echo "exit: $?"
# EXPECT: exit 0

# Probe 11 — wrapper sanity: cargo-check.sh honors --workspace --features full
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/probe11-check.log 2>&1
echo "exit: $?"
tail -5 /tmp/probe11-check.log
# EXPECT: exit 0 (baseline workspace check clean at phase-v1-rls-r1 HEAD)

# Probe 12 — wrapper sanity: cargo-clippy.sh honors --no-deps
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > /tmp/probe12-clippy.log 2>&1
echo "exit: $?"
tail -5 /tmp/probe12-clippy.log
# EXPECT: exit 0 (baseline clippy clean at phase-v1-rls-r1 HEAD)

# Probe 13 — concurrent-PR check (no other PR touches v1-rls-r1 file set)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("\\.claude/(hooks|rules|skills/weekly-review|PRPs/specs)|docs/brehon-law-inspired-network/governance-log")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 14 — negative test: confirm exit-code propagation
test -d /this/does/not/exist
echo "exit: $?"
# EXPECT: exit 1
```

**EXPECT block:**
- Probes 0–13 exit 0
- Probe 14 exits NON-ZERO (negative test confirms exit-code propagation)

If any probe fails, file `kind: "blocker"` per `.claude/rules/
advisor-orchestrator.md` §5.4 (DQ triage decision tree) and DO NOT
proceed.

### Task 1 [P]: CREATE `.claude/PRPs/specs/` directory + spec README + MCP write-time embedding spec

**ACTION:** establish the new `.claude/PRPs/specs/` directory (per
DQ #298 advisor-resolved — directory does not exist at HEAD); write
a 5-line `README.md` describing the directory's purpose; author the
item 4.2-spec doc per Track A scope.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/PRPs/specs/README.md
  - .claude/PRPs/specs/mcp-write-time-embedding.md
modifies: []
requires:
  - task: 0
    reason: "Pre-flight verifies .claude/PRPs/specs/ is absent (Probe 4)."
```

**IMPLEMENT (file 1 of 2):** in `.claude/PRPs/specs/README.md`, 5
lines (one-paragraph + the canonical purpose statement from the
brief §2.1 Track A Task 1):

```markdown
# .claude/PRPs/specs/

Tracked specifications for contracts that live outside brehon-fork
(e.g. MCP server patches, external-repo coordination docs). Each
spec names the canonical implementation target, the contract
verification recipe, and the downstream-collapse claim. Authored at
brehon-fork-spec time; implemented in the target repo's own PR
cadence.
```

**IMPLEMENT (file 2 of 2):** in
`.claude/PRPs/specs/mcp-write-time-embedding.md`, author the spec
doc per §10.7 mirror + brief PRECON-2 §0.1.2. Required H2 sections:

1. `# MCP write-time embedding — patch contract` (H1 title).
2. `## 1. Authority` — cites this plan + RLS-PMD review §4.2 +
   `feedback_pmd_backfill_after_write.md` (source inspection).
3. `## 2. Patch insertion point` — names
   `MCPs/project-memory-mcp/dist/index.js` `memory_write_eval`
   handler; insertion is post-FTS5 insert, pre-commit; includes a
   short JS snippet showing the expected shape (call out to
   `nomic-embed-text` via configured Ollama URL; insert resulting
   vector into `memory_vectors`).
4. `## 3. Graceful-fallback contract` — on Ollama unreachable:
   write the row, log a single stderr WARN with the exact format
   `MCP write-time-embed miss: memory_id=<id> reason=ollama-unreachable`,
   return success to the caller. Weekly-review Step 1b safety net
   catches.
5. `## 4. Verification recipe` — exact 3-write probe sequence
   + `SELECT COUNT(*) FROM memory_vectors` ≡ row count immediately
   after write.
6. `## 5. Downstream-collapse claim` — post-patch deletes /
   demotes (verbatim from brief §0.1.2(d)):
   - Delete `.claude/skills/session-retro/SKILL.md` Step 5.5
     inline-backfill instruction.
   - Demote `.claude/skills/weekly-review/SKILL.md` Step 1b to
     aspirational ("safety net for legacy unembedded rows").
   - Junior 7-day window vanishes (RLS-PMD review §5.3 autonomy
     criterion becomes near-real-time).
7. `## 6. Verification deferred` — explicit "actual `dist/index.js`
   patch ships in a separate `MCPs/project-memory-mcp` PR; this doc
   is the contract that PR implements; verification of §5 collapse
   claim deferred to the MCPs PR's own DoD".

**MIRROR:** read `.claude/lessons/feedback_pmd_backfill_after_write.md`
in full first (per R8 + `feedback_read_canonical_before_writing_
spec.md`). Per §10.7 the brief §0.1.2 is the verbatim source for
sections 2-6.

**GOTCHA:** Section 6 is load-bearing — without the "verification
deferred" disclaimer, a future advisor reads §4 as if v1-rls-r1
ships the patch (it doesn't). Per PRECON-2 hard refusal.

**GOTCHA:** `description:` field — N/A (specs do not carry
frontmatter; they are markdown-only).

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
# Directory + files exist
test -d .claude/PRPs/specs
test -f .claude/PRPs/specs/README.md
test -f .claude/PRPs/specs/mcp-write-time-embedding.md
echo "exit: $?"
# EXPECT: exit 0

# README is short (≤ 10 lines)
wc -l .claude/PRPs/specs/README.md
# EXPECT: 5-10 lines

# Spec has all required H2 sections
grep -c "^## " .claude/PRPs/specs/mcp-write-time-embedding.md
# EXPECT: 6 (sections 1-6)

# Verification-deferred disclaimer present
grep -q "Verification deferred" .claude/PRPs/specs/mcp-write-time-embedding.md
echo "exit: $?"
# EXPECT: exit 0

# Sanity: workspace check still passes
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task1-check.log 2>&1
echo "exit: $?"
tail -5 .claude/PRPs/debug/v1-rls-r1-task1-check.log
# EXPECT: exit 0
```

### Task 2 [P]: CREATE `.claude/rules/pmd-invariants.md` consolidating five PMD-meta invariants

**ACTION:** author the new rule file consolidating five PMD-meta
invariants per Track A scope. Lesson files stay as evidence trail;
the rule is the authoritative reference.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/rules/pmd-invariants.md
modifies: []
requires:
  - task: 0
    reason: "Pre-flight verifies .claude/rules/pmd-invariants.md is absent (Probe 5)."
```

**IMPLEMENT (file 1 of 1):** in `.claude/rules/pmd-invariants.md`,
follow §10.3 (no frontmatter) + §10.8 (verbatim invariant text):

1. `# PMD invariants` H1.
2. One-paragraph **preamble**: "These five statements are non-
   negotiable system invariants, not heuristics. They protect the
   PMD (Project Memory Database) substrate the entire Recursive
   Learning System depends on. Scattering them as lesson files
   conflates invariants with recurrence-based heuristics; this
   rule consolidates them. The lesson files stay as evidence
   trail. Per `docs/research/brehon-rls-pmd-review.md` §4.8."
3. Five numbered H2 sub-sections — one per invariant — each
   carrying:
   - The verbatim invariant statement (per §10.8).
   - **Why this is non-negotiable** — one-paragraph rationale
     citing the originating incident (v1-ship-1 for #1+#5; PMD
     write-path inspection 2026-05-16 for #3; LESSON-trailer
     convention for #4).
   - **How to apply** — one-line naming the SHIPPED enforcement
     (Track B Task 3 `pmd-canonical-guard.sh` for #1+#5; nothing
     yet for #3 → `enforcement: PENDING — see .claude/PRPs/specs/
     mcp-write-time-embedding.md + future MCPs/project-memory-mcp
     PR`; LESSON-trailer convention per
     `feedback_junior_pmd_write_convention.md` for #4).
   - **See also** — bulleted cross-links to the originating
     lesson(s).
4. **Final section** `## See also`:
   - `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane
     shared"
   - `.claude/rules/pmd-search-strategy.md` (hybrid-search status)
   - `.claude/hooks/pmd-canonical-guard.sh` (v1-rls-r1 ships)
   - `docs/research/brehon-rls-pmd-review.md` §3 + §4 + §5.3

**MIRROR:** `.claude/rules/decision-queue.md` lines 1-30 for tone +
no-frontmatter convention; `.claude/rules/multi-lane-worktree.md`
§"PMD is cross-lane shared" for invariant prose shape.

**GOTCHA:** five invariants, EXACTLY. Watchpoint #2 — no sixth, no
fourth.

**GOTCHA:** each invariant's "How to apply" must EITHER name a
SHIPPED enforcement OR carry explicit `enforcement: PENDING + see
<future-sub-phase>` text (per R7).

**GOTCHA:** rule files in `.claude/rules/` auto-load at SessionStart
by file presence (no frontmatter needed). Per §10.3 — do NOT add a
YAML frontmatter block.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
# File exists
test -f .claude/rules/pmd-invariants.md
echo "exit: $?"
# EXPECT: exit 0

# Exactly 5 numbered H2 invariants
grep -cE "^## [1-5]\. " .claude/rules/pmd-invariants.md
# EXPECT: 5

# No frontmatter (per §10.3)
head -1 .claude/rules/pmd-invariants.md
# EXPECT: starts with "# PMD invariants" (NOT "---")

# Each invariant cites either a shipped enforcement or PENDING marker
grep -cE "(enforcement: PENDING|How to apply)" .claude/rules/pmd-invariants.md
# EXPECT: ≥ 5

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task2-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 3 [P]: CREATE `.claude/hooks/pmd-canonical-guard.sh` SessionStart guard script

**ACTION:** author the WARN-not-FAIL SessionStart hook script per
Track B scope + the spec in
`feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
§"Mechanism" verbatim.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/hooks/pmd-canonical-guard.sh
modifies: []
requires:
  - task: 0
    reason: "Pre-flight verifies .claude/hooks/pmd-canonical-guard.sh is absent (Probe 6)."
```

**IMPLEMENT (file 1 of 1):** in
`.claude/hooks/pmd-canonical-guard.sh`, author per §10.1 mirror
(retro-check.sh shape) + §10.2 (Python one-liner):

1. Shebang `#!/usr/bin/env bash` + `set -euo pipefail`.
2. **Leading comment block** (mirror `retro-check.sh` lines 1-13):
   - Purpose: "SessionStart hook: WARN if this lane's
     `.mcp.json` PROJECT_MEMORY_DB diverges from the canonical
     cross-lane path. Per
     `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`."
   - Event: SessionStart.
   - Timeout: 5000.
   - "Exit 0 = continue (always; this hook is WARN-not-FAIL per the
     lesson's WARN-vs-FAIL rationale)."
3. **`.mcp.json` absence handling** — if `.mcp.json` not present at
   CWD root, exit 0 silently (some lanes resolve canonical by
   default; absence is safe per the lesson).
4. **Compute canonical target** — mirror `retro-check.sh` lines
   32-41 (the `git rev-parse --git-common-dir` resolution).
5. **Extract running value** — use the §10.2 Python one-liner; if
   the JSON path key is missing OR python unavailable, exit 0
   silently with a single-line stderr note (graceful tool absence,
   per the lesson; do not block).
6. **Normalise paths** — resolve both to absolute paths; case-fold
   drive letter on Windows (per the lesson §"Mechanism" step 1
   "normalised — resolve to an absolute path, case-fold the drive
   letter on Windows").
7. **Compare** — strings equal → exit 0 silent. Strings differ →
   emit a multi-line stderr WARN banner naming BOTH paths + the
   incident citation + the 1-line fix (verbatim from the lesson
   §"Mechanism" step 1: `"edit '.mcp.json' PROJECT_MEMORY_DB to
   '<canonical>' and restart the MCP — the running MCP cached its
   handle at startup, see the sequencing constraint in
   feedback_pmd_cross_lane_canonical_db.md"`).
8. **Exit 0 always** — every code path. WARN-not-FAIL per R1 +
   Watchpoint #1.

**MIRROR:** `.claude/hooks/retro-check.sh:1-50` (canonical hook
shape); `.claude/lessons/feedback_pmd_cross_lane_canonical_db.md`
§"Diagnosis recipe" step 1 (the Python one-liner verbatim). Per
R8 — Read BOTH before authoring.

**GOTCHA:** script exits 0 on EVERY path including config errors.
A script that exits non-zero on a missing `.mcp.json` would block
sessions in lanes where absence is correct (federation-inbound-a
lane is the historical precedent). Per Watchpoint #1.

**GOTCHA:** path normalisation must handle Windows drive-letter
case-folding (`C:` vs `c:`). The lesson §"Mechanism" cites this
explicitly.

**GOTCHA:** the running MCP cached its handle at startup — even on a
mismatch, the script CANNOT fix the running session's MCP. The WARN
banner explicitly tells the human "restart the MCP". No PreToolUse-
block behaviour.

**GOTCHA:** **dogfood handoff** — Track D Task 10's 4.1 dogfood runs
this script against (a) this lane's `.mcp.json` (silent exit 0)
+ (b) a `/tmp/sentinel.mcp.json` with a wrong `PROJECT_MEMORY_DB`
(stderr WARN, exit 0). The script reads `.mcp.json` from CWD (no
positional arg); Task 10 sets up a sentinel CWD with a wrong-path
`.mcp.json` and invokes the script from there. Document the CWD-
reliance in the script's leading comment block.

**Frontmatter `description:` field:** N/A — shell scripts do not
carry frontmatter. Per Watchpoint #5: the LEADING COMMENT BLOCK
uses plain-language prose; no "ALWAYS RUN" / "MUST USE" verbiage.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
# File exists + executable
test -x .claude/hooks/pmd-canonical-guard.sh
echo "exit: $?"
# EXPECT: exit 0

# Shebang + set discipline
head -2 .claude/hooks/pmd-canonical-guard.sh
# EXPECT: line 1 = #!/usr/bin/env bash
# EXPECT: line 2 (or among lines 2-20) contains "set -euo pipefail"

# Probe-run against current lane (canonical match — silent exit 0)
bash .claude/hooks/pmd-canonical-guard.sh 2>/tmp/probe-canonical.stderr
echo "exit: $?"
cat /tmp/probe-canonical.stderr
# EXPECT: exit 0, stderr empty (canonical lane match)

# Probe-run against deliberate-mispoint sentinel
SENTINEL="/tmp/v1-rls-r1-task3-sentinel-$$"
mkdir -p "$SENTINEL" && cd "$SENTINEL"
cat > .mcp.json <<'EOF'
{"mcpServers":{"project-memory":{"env":{"PROJECT_MEMORY_DB":"/tmp/wrong/path.db"}}}}
EOF
bash <WORKTREE>/.claude/hooks/pmd-canonical-guard.sh 2>/tmp/probe-sentinel.stderr
echo "exit: $?"
cat /tmp/probe-sentinel.stderr
# EXPECT: exit 0, stderr contains "WARN" + both paths + the 1-line fix
cd <WORKTREE>
rm -rf "$SENTINEL"

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task3-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 4 [P]: INSERT Step 2c retro-harvest sweep into `.claude/skills/weekly-review/SKILL.md`

**ACTION:** insert a new Step 2c retro-harvest sweep between current
Step 2b (line 77 at HEAD) and current Step 3 (line 95 at HEAD) per
DQ #296 + Track B scope.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - .claude/skills/weekly-review/SKILL.md   # insert Step 2c retro-harvest sweep between Step 2b and Step 3
requires:
  - task: 0
    reason: "Pre-flight verifies weekly-review/SKILL.md (Probe 8) and retro-harvest/SKILL.md (Probe 9) present."
```

**IMPLEMENT (file 1 of 1):** in
`.claude/skills/weekly-review/SKILL.md`, insert Step 2c per §10.5
mirror. New step body:

1. `### 2c. Retro-harvest sweep` H3 heading.
2. One-paragraph **Why:** "The harvest tier
   (`.claude/skills/retro-harvest/SKILL.md`) surfaces unchecked
   proposals from session retros and sub-phase retros. Before this
   sub-phase, the skill ran ad-hoc — leaving ~48 unchecked proposals
   accumulated across 52 retro files (per RLS-PMD review §4.6
   evidence) and 7-day-before-loss anti-patterns. Folding it into
   weekly cadence guarantees the harvest tier runs at the same
   rhythm as backfill and sync."
3. **Operation sequence** (numbered list):
   1. `Glob .claude/PRPs/reports/*.md` filtered to mtime within
      last 7 days.
   2. For each retro: Read the §"What to change" + §"Decisions to
      revisit" sections.
   3. Extract proposals NOT yet promoted to `.claude/lessons/` OR
      `CLAUDE.md` (check by greppin lesson filenames + canonical
      pattern text against the proposal verbatim quote).
   4. Write a single weekly artifact at `.claude/harvest/<iso-
      week>.md` with proposals enumerated, each as a
      `(retro-source: <path>, proposal-text: <verbatim quote>,
      ground-truth-evidence: <if any>)` triple.
   5. Final paragraph: "**SURFACING, not auto-promoting** — manual
      review thereafter per the RLS-PMD review §4.6 contract.
      Promotion to `.claude/lessons/` or `CLAUDE.md` is a human
      decision, not an automated step."
4. **Output path note:** "Output `.claude/harvest/<iso-week>.md` is
   gitignored (per `.gitignore`; see Task 5). Runtime-journal
   semantics — summary lands in the weekly-review report; harvest
   files prune after N weeks (planner-time choice; see §19.1 pre-
   seed #2)."
5. **Optional invoke-by-reference:** if
   `.claude/skills/retro-harvest/SKILL.md` is invocable from
   another skill (verify by reading retro-harvest's first 50 lines
   for any "do NOT invoke from another skill" hard refusal): the
   Step 2c body MAY say "Invoke the retro-harvest skill
   (`.claude/skills/retro-harvest/SKILL.md`) for the sweep logic;
   this Step is the cadence call point". Otherwise the Step
   duplicates the core sweep logic inline. Worker decides at impl
   time after reading retro-harvest/SKILL.md.

**MIRROR:** `.claude/skills/weekly-review/SKILL.md` lines 23-65
(existing Step 1b — the canonical step-style) + lines 67-93
(existing Step 2 + Step 2b — adjacent step shapes). Read in full
first.

**GOTCHA:** insert BETWEEN Step 2b (line 77) and Step 3 (line 95).
Existing Step 3-6 numbering is **preserved verbatim** (zero
downstream renumber per DQ #296). The new step is `2c`, not `3`.

**GOTCHA:** **SURFACING, not auto-promoting.** Per RLS-PMD review
§4.6 + brief §2.2 "No auto-promote of harvest-surface proposals to
`.claude/lessons/`." The step body emphasises the boundary
explicitly.

**GOTCHA:** the harvest output naming convention `.claude/harvest/
<iso-week>.md` matches Task 5's `.gitignore` line. Worker
synchronises with Task 5's branch tip (Task 5 `requires: [4]` —
Task 5 reads Task 4's output path before adding the gitignore entry).

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
# Step 2c heading present
grep -c "^### 2c\. " .claude/skills/weekly-review/SKILL.md
# EXPECT: 1

# Total step count post-edit (was 7 ## headings — 1, 1b, 2, 2b, 3, 4, 5, 6; now 9 with 2c)
grep -cE "^### " .claude/skills/weekly-review/SKILL.md
# EXPECT: 9

# Existing Step 3 heading unchanged
grep -c "^### 3\. Aggregate eval metrics" .claude/skills/weekly-review/SKILL.md
# EXPECT: 1

# Surfacing-not-auto-promoting disclaimer present
grep -i "SURFACING, not auto-promoting\|surfacing.*not.*auto-promoting" .claude/skills/weekly-review/SKILL.md
# EXPECT: at least one match

# harvest path mentioned
grep -c "\.claude/harvest/" .claude/skills/weekly-review/SKILL.md
# EXPECT: ≥ 1

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task4-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 5: ADD `.claude/harvest/` + `.claude/governance-log/` to `.gitignore`

**ACTION:** add two gitignore entries: `.claude/harvest/` (Task 4's
output path) + `.claude/governance-log/` (Task 7's JSONL sidecar
output path). Both are runtime-journal output per PRECON-9 lean.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - .gitignore   # add .claude/harvest/ and .claude/governance-log/ lines
requires:
  - task: 4
    reason: "Task 4 establishes the .claude/harvest/<iso-week>.md output convention; this gitignore entry references it."
```

**IMPLEMENT (file 1 of 1):** in `.gitignore`, append two lines under
the existing `.claude/`-related entries (read current `.gitignore` to
find the right section; lean is the same block that gitignores
`.claude/auto-state/` if present, OR a new `# RLS runtime journals`
section if no adjacent block):

```
# RLS runtime journals — see .claude/PRPs/plans/v1-rls-r1.plan.md §11
.claude/harvest/
.claude/governance-log/
```

**MIRROR:** existing `.gitignore` style (one block per category,
short comment headers). Read `.gitignore` in full first.

**GOTCHA:** `.gitignore` patterns are line-based and order-
independent. Adding the two lines at the end of the file is safe.
But: putting them adjacent to other `.claude/`-runtime entries
(e.g. `.claude/auto-state/` if it exists) is cleaner. Worker
decides after reading the file.

**GOTCHA:** **Task 5 `requires: [4]`** because the harvest path is
established in Task 4's prose. If Task 5 lands first with a path
Task 4 then disagrees with, the two entries drift. Serial
dependency enforces consistency.

**Frontmatter `description:` field:** N/A (`.gitignore` is not a
markdown file).

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
# Both lines present
grep -E "^\.claude/(harvest|governance-log)/$" .gitignore | wc -l
# EXPECT: 2

# .gitignore syntactically valid (no parse errors)
git check-ignore -v .claude/harvest/foo.md 2>&1
# EXPECT: matches .gitignore line for .claude/harvest/
git check-ignore -v .claude/governance-log/retro-bypass.jsonl 2>&1
# EXPECT: matches .gitignore line for .claude/governance-log/

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task5-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 6: AUTHOR `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` + document manual dual-wire

**ACTION:** author the new lesson file (verified absent at HEAD per
Probe verification — see §12) capturing the lane-bootstrap-checklist
discipline with the new (N+1) step wiring
`pmd-canonical-guard.sh` SessionStart entry; document the manual
dual-wire in both canonical + this-lane settings.local.json files
per DQ #301.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md
modifies: []
requires:
  - task: 3
    reason: "The lesson cites pmd-canonical-guard.sh (Task 3) as the script the wiring activates."
```

**IMPLEMENT (file 1 of 1):** in
`.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`,
follow §10.4 lesson frontmatter shape + the §10.4 anti-keyword-stuffing
discipline. Body sections:

1. **Frontmatter** (per §10.4):
   ```yaml
   ---
   name: Lane-bootstrap checklist — durable record of per-worktree setup steps
   description: Each new lane-dedicated worktree under .claude/rules/multi-lane-worktree.md requires a small set of per-worktree setup steps (copy .mcp.json, copy / create .claude/settings.local.json, wire any SessionStart hooks the lane depends on). This checklist is the durable record so a new lane can be brought up without re-deriving each step from scattered lesson files. Per the v1-rls-r1 sub-phase + DQ #301.
   type: feedback
   ---
   ```
2. `# Lane-bootstrap checklist` H1.
3. **Why this lesson exists** — one-paragraph context: per
   `feedback_settings_local_json_worktree_bootstrap.md`,
   `.claude/settings.local.json` is per-worktree and gitignored;
   per `feedback_pmd_cross_lane_canonical_db.md` + `.claude/rules/
   multi-lane-worktree.md`, `.mcp.json` is per-worktree and
   gitignored. Cumulative state means each new lane has 3-5 manual
   bootstrap steps that don't propagate via git. Without a checklist,
   each new-lane operator re-derives the steps from scattered
   lessons and rules.
4. **Checklist** (numbered list — each step explicit):
   1. `cd C:/Users/barri/Developer/brehon-fork` (canonical).
   2. `git fetch origin <phase-branch>`.
   3. `git worktree add ../brehon-fork-<lane> <phase-branch>`.
   4. `cd ../brehon-fork-<lane>`.
   5. `cp .mcp.json.example .mcp.json` (the example carries the
      canonical absolute `PROJECT_MEMORY_DB` per `feedback_pmd_
      cross_lane_canonical_db.md`; verify line: `grep
      PROJECT_MEMORY_DB .mcp.json` shows the canonical absolute).
   6. **(NEW — v1-rls-r1)** Wire `pmd-canonical-guard.sh`
      SessionStart entry in `.claude/settings.local.json`. Create
      the file if absent. Snippet to paste:
      ```json
      {
        "hooks": {
          "SessionStart": [
            {
              "matcher": ".*",
              "hooks": [
                {"type": "command", "command": "bash .claude/hooks/pmd-canonical-guard.sh"}
              ]
            }
          ]
        }
      }
      ```
      (Sequence this entry BEFORE any existing SessionStart entry
      that calls `pre-phase-audit.sh` — the canonical guard must
      surface drift before phase audit assumes the canonical PMD
      is reachable. Merge into an EXISTING `settings.local.json`;
      do not overwrite.)
   7. Open Claude Code in the new worktree CWD; verify SessionStart
      banner shows no `pmd-canonical-guard.sh` WARN.
5. **DQ #301 dual-wire (v1-rls-r1 ships)** — the lesson documents
   that step 6's wiring MUST be applied in BOTH:
   - The **canonical** `C:/Users/barri/Developer/brehon-fork/.claude/
     settings.local.json` (durable across phase-v1-rls-r1 worktree
     removal).
   - The **lane-dedicated** `C:/Users/barri/Developer/brehon-fork-
     rls-r1/.claude/settings.local.json` (immediate r1-session
     dogfood).
   Cost ≈ 30 seconds extra; belt-and-braces. Future lanes apply
   the wiring at step 6 per the checklist; the canonical wiring
   is the durable record.
6. **See also**:
   - `feedback_settings_local_json_worktree_bootstrap.md`
   - `feedback_pmd_cross_lane_canonical_db.md`
   - `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
   - `.claude/rules/multi-lane-worktree.md`
   - `.claude/rules/pmd-invariants.md` (Task 2)
   - `.claude/hooks/pmd-canonical-guard.sh` (Task 3)

**MIRROR:** `.claude/lessons/feedback_settings_local_json_worktree_
bootstrap.md` (read in full first — it establishes the per-worktree-
gitignored discipline this checklist builds on) +
`.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_at_session_
start.md` §"Mechanism" (the spec citation).

**GOTCHA:** the brief assumed this file existed at HEAD; it does
not. Track D Task 12's earlier draft cross-linked to it. Planner
recommends keeping that cross-link valid by SHIPPING this lesson in
Task 6 (rather than filing a `kind: "blocker"`). The lesson is
durable infra — it has value beyond just propagating the new step.

**GOTCHA:** the snippet's JSON is a fragment, not a complete
`settings.local.json` (existing files may already have other
sections — `permissions`, `env`, etc). The lesson body explicitly
notes this: "merge into your existing `settings.local.json`; do not
overwrite". The N+1 step phrasing in the brief presupposes a
checklist with prior steps — the lesson body provides the FULL
checklist (steps 1-7), not just the new step.

**GOTCHA:** the **dual-wire is a manual hand-off** (per §11
"Manual hand-off"). The lesson documents the JSON snippet but the
two `settings.local.json` files are NOT in any task's `creates:` /
`modifies:` array — they cannot be (gitignored). The advisor / user
applies the wiring post-merge.

**Frontmatter `description:` field:** per Watchpoint #5 — plain
natural-language description, NO "ALWAYS RUN" / "MUST INVOKE"
verbiage.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
# File exists
test -f .claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md
echo "exit: $?"
# EXPECT: exit 0

# Frontmatter valid
python3 -c "
import re, yaml
content = open('.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md').read()
m = re.match(r'^---\n(.*?)\n---', content, re.S)
assert m, 'no frontmatter'
fm = yaml.safe_load(m.group(1))
assert fm['type'] == 'feedback', fm
assert 'name' in fm and 'description' in fm, fm
forbidden = ['DO use when', 'MUST USE', 'ALWAYS INVOKE', 'CRITICAL']
for kw in forbidden:
    assert kw not in fm['description'], f'forbidden keyword: {kw}'
print('OK')
"
# EXPECT: OK

# Checklist mentions the JSON snippet
grep -c "pmd-canonical-guard.sh" .claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md
# EXPECT: ≥ 2

# DQ #301 dual-wire documented
grep -c "brehon-fork-rls-r1\|canonical.*brehon-fork" .claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md
# EXPECT: ≥ 2

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task6-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 7 [P]: APPEND `emit_retro_bypass_log` function to `.claude/hooks/retro-check.sh`

**ACTION:** APPEND one shell function `emit_retro_bypass_log` to
`.claude/hooks/retro-check.sh` AND insert one function-invoke line
in the existing fail-open branch (lines 141-144 at HEAD `6e16ee94f`).
The existing fail-open behaviour (3-attempt cap, `rm -f`, `exit 0`)
is PRESERVED — additive only per R6 + Watchpoint #7.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - .claude/hooks/retro-check.sh   # append emit_retro_bypass_log function + invoke line in fail-open branch
requires:
  - task: 0
    reason: "Pre-flight verifies retro-check.sh present (Probe 7)."
```

**IMPLEMENT (file 1 of 1):** in `.claude/hooks/retro-check.sh`,
TWO edits:

1. **APPEND function** at the end of the file (after line 156 at
   HEAD):

   ```bash
   # --- emit_retro_bypass_log ---
   #
   # Writes one JSONL record to .claude/governance-log/retro-bypass.jsonl
   # on every fail-open path. Per RLS-PMD review §4.7 +
   # .claude/PRPs/plans/v1-rls-r1.plan.md §13 Task 7.
   #
   # Fields (per DQ #297): timestamp, session_id, attempt_count,
   # prompt_hash, branch_at_fail_open, kind.
   #
   # Non-fatal — any error (missing dir, write race, jq absent)
   # silently exits the function. Bypass instrumentation must not
   # itself become a Stop hook failure mode.
   emit_retro_bypass_log() {
     local attempts="$1"
     local branch="$2"
     local logdir=".claude/governance-log"
     local logfile="${logdir}/retro-bypass.jsonl"
     mkdir -p "$logdir" 2>/dev/null || return 0
     command -v jq >/dev/null 2>&1 || return 0
     local ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
     local prompt_hash
     prompt_hash="$(printf '%s' "${CLAUDE_PROMPT:-}" | sha256sum 2>/dev/null | head -c 16)"
     [ -z "$prompt_hash" ] && prompt_hash="unknown"
     local session_id="${CLAUDE_SESSION_ID:-${PPID:-unknown}}"
     jq -nc \
       --arg ts "$ts" \
       --arg sid "$session_id" \
       --argjson att "$attempts" \
       --arg ph "$prompt_hash" \
       --arg br "$branch" \
       '{timestamp:$ts, session_id:$sid, attempt_count:$att, prompt_hash:$ph, branch_at_fail_open:$br, kind:"retro_bypass"}' \
       >> "$logfile" 2>/dev/null || true
   }
   ```

2. **INSERT invoke line** in the existing fail-open branch at HEAD
   lines 141-144 (the `rm -f "$SESSION_FILE"; exit 0` block).
   Edit shape — read the file, find the lines:
   ```bash
   if [ "$ATTEMPTS" -ge 3 ]; then
     rm -f "$SESSION_FILE"
     exit 0
   fi
   ```
   to:
   ```bash
   if [ "$ATTEMPTS" -ge 3 ]; then
     rm -f "$SESSION_FILE"
     emit_retro_bypass_log "$ATTEMPTS" "$CURRENT_BRANCH"
     exit 0
   fi
   ```
   The new line invokes the function with the current attempts
   count + branch (captured at line 63 of retro-check.sh from
   `git rev-parse --abbrev-ref HEAD`).

**MIRROR:** `.claude/hooks/observation-capture.sh:73-82` (canonical
`jq -nc` JSONL emit pattern; the brief explicitly confirmed
observation-capture's `~/.cache/tw-observations/` path is unsuitable
for the trail BUT its `jq -nc --arg` JSONL build pattern is reusable).
Per R8 — Read both files in full before authoring.

**GOTCHA:** **The 3-attempt cap, the `rm -f`, and the `exit 0` are
UNCHANGED.** The only behaviour delta is the JSONL line written
before exit. Watchpoint #7 — additive only.

**GOTCHA:** **Non-fatal contract.** Every error path in
`emit_retro_bypass_log` returns silently (`return 0` or `|| true`).
A failed JSONL write must NOT cause `retro-check.sh` to exit
non-zero — it would convert a fail-open into a fail-block.

**GOTCHA:** `CLAUDE_PROMPT` is the existing env var read at line 20
(verified at HEAD `6e16ee94f`). The hash uses SHA-256 truncated to
16 hex chars (per DQ #297). Loop detection: same prompt firing N
consecutive Stop attempts shows the same hash across N JSONL
records.

**GOTCHA:** `CLAUDE_SESSION_ID` is preferred for `session_id` when
available (Claude Code provides it in hook contexts); falls back
to `PPID` for older harnesses. If neither — `unknown`.

**GOTCHA:** `.claude/governance-log/` is gitignored (Task 5).
Worker confirms by reading `.gitignore` post-Task-5 OR runs the
dogfood (Task 10) which verifies the JSONL file lands and is
ignored by git.

**Frontmatter `description:` field:** N/A (shell script). The
APPENDED function carries a plain-language leading comment block
per Watchpoint #5.

**VALIDATE (story-checkpoint feeds §16a Story 4):**

```bash
# Function present
grep -c "^emit_retro_bypass_log()" .claude/hooks/retro-check.sh
# EXPECT: 1

# Invoke line present in fail-open branch
grep -A1 "rm -f \"\$SESSION_FILE\"" .claude/hooks/retro-check.sh | grep -c "emit_retro_bypass_log"
# EXPECT: 1

# Existing fail-open structure preserved
grep -c '\$ATTEMPTS" -ge 3' .claude/hooks/retro-check.sh
# EXPECT: 1
grep -c '"$SESSION_FILE"' .claude/hooks/retro-check.sh
# EXPECT: ≥ 2 (the rm -f line + the echo-attempts write — preserved)

# Script still executes cleanly (syntactic check)
bash -n .claude/hooks/retro-check.sh
echo "exit: $?"
# EXPECT: exit 0

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task7-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 8: AUTHOR `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md` + cross-link from 04-data-model-and-api.md

**ACTION:** create a NEW dedicated doc at
`docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`
registering hook-emitted JSONL observability kinds (first entry:
`retro_bypass` per Task 7); add a one-sentence cross-link from
`docs/brehon-law-inspired-network/04-data-model-and-api.md` near the
redaction-service-contract block (line 405 at HEAD) pointing at the
new doc.

**FILES (machine-parseable):**

```yaml
creates:
  - docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
modifies:
  - docs/brehon-law-inspired-network/04-data-model-and-api.md   # one-sentence cross-link near line 405
requires:
  - task: 7
    reason: "The retro_bypass kind registry entry cites Task 7's emit_retro_bypass_log function output format (sidecar path, JSONL field schema)."
```

**IMPLEMENT (file 1 of 2):** in
`docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`,
follow §10.6 mirror — three sections:

1. **`# Governance-log JSONL kinds (hook-emitted observability)`**
   H1.
2. **`## 1. Scope`** — one paragraph (verbatim from brief §0.1.10
   /§2.1 Track C Task 8):
   > Harness-internal observability events emitted by
   > `.claude/hooks/*.sh` as JSONL records to sidecar files under
   > `.claude/governance-log/`. **NOT the v1 product governance-
   > log** — that lives in the PG `governance_log` table per
   > `docs/brehon-law-inspired-network/03-architecture.md` §6
   > "The append-only governance log (interface)" and is
   > referenced (in the redaction-service-contract block) by
   > `docs/brehon-law-inspired-network/04-data-model-and-api.md`.
   > These two systems are DIFFERENT THINGS by design: the PG
   > table is hash-chained product surface; this sidecar is
   > observability tooling. Different consumers, different write
   > paths, different durability guarantees.
3. **`## 2. Kind registry`** — 5-column markdown table:

   | kind | originating hook | sidecar path | JSONL field schema | consumer |
   |---|---|---|---|---|
   | `retro_bypass` | `.claude/hooks/retro-check.sh` (fail-open path, lines 141-145 post-v1-rls-r1) | `.claude/governance-log/retro-bypass.jsonl` | `{timestamp: ISO 8601 UTC, session_id: string, attempt_count: integer, prompt_hash: 16-hex SHA-256 of CLAUDE_PROMPT, branch_at_fail_open: string, kind: "retro_bypass"}` | `.claude/skills/weekly-review/SKILL.md` Step 2c retro-harvest sweep (per DQ #296) + future audit reads |

4. **`## 3. Adding a new kind`** — short procedure: (a) define
   sidecar path under `.claude/governance-log/<kind>.jsonl`; (b)
   confirm `.gitignore` includes `.claude/governance-log/`; (c)
   author the emit shell function in the originating hook; (d) add
   a row to the §2 table; (e) document the consumer (weekly-review
   Step 2c / a future skill / etc); (f) add a paired lesson at
   `.claude/lessons/feedback_<kind>_governance_log.md` cross-linking
   the instrumentation. Per `feedback_lesson_must_pair_with_
   structural_fix_when_fixable.md`.

**IMPLEMENT (file 2 of 2):** in
`docs/brehon-law-inspired-network/04-data-model-and-api.md`, INSERT
one sentence near the redaction-service-contract block at line 405
at HEAD (or the nearest governance-log reference paragraph — worker
reads the file in full first to confirm best placement). The exact
insertion:

> See also `governance-log-kinds-jsonl.md` for **hook-emitted JSONL
> observability** — a distinct sidecar trail (e.g. `retro_bypass`)
> NOT routed through the hash-chained PG `governance_log` table this
> document describes.

The sentence sits adjacent to the redaction-service-contract block
because that block is the closest in-document mention of
`governance_log`-related contract surface. The distinction
("DIFFERENT THINGS by design") is load-bearing per brief §0.1.10.

**MIRROR:** the brief §0.1.10 / §2.1 Track C Task 8 prose verbatim
(§10.6 mirror). Per R8, the worker reads the brief's prose first
before authoring.

**GOTCHA:** **§3 architecture.md §6 is the v1 product governance-
log spec.** The new JSONL kinds doc must NOT cite §6 as a sibling-
kind registry — they are different systems. The Scope paragraph
explicitly distinguishes.

**GOTCHA:** **No new ADR.** Per PRECON-5 + brief §0.1.10 binding
consequence: this new doc is observability infrastructure, NOT a
design decision the ADR corpus tracks. Confirm by NOT cross-linking
to `docs/brehon-law-inspired-network/99-decisions-and-open-
questions.md` from the new doc.

**GOTCHA:** the cross-link sentence in 04 must be neutral about the
new doc — don't characterise `retro_bypass` as "more important than"
the PG table or vice versa. Both are part of the harness; both
matter; they are simply different. Per Watchpoint #5 anti-keyword-
stuffing.

**VALIDATE (story-checkpoint feeds §16a Story 4):**

```bash
# New doc exists
test -f docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
echo "exit: $?"
# EXPECT: exit 0

# Three H2 sections (Scope, Kind registry, Adding a new kind)
grep -cE "^## " docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
# EXPECT: 3

# retro_bypass entry in registry table
grep -c "retro_bypass" docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
# EXPECT: ≥ 2 (Scope mention + table row)

# Scope paragraph distinguishes from PG table
grep -iE "NOT the v1 product|DIFFERENT THINGS" docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md
# EXPECT: at least one match

# 04 cross-link present
grep -c "governance-log-kinds-jsonl" docs/brehon-law-inspired-network/04-data-model-and-api.md
# EXPECT: ≥ 1

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task8-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 9: AUTHOR `.claude/lessons/feedback_retro_bypass_governance_log.md`

**ACTION:** author new lesson cross-linking the Track C
instrumentation per Track D scope.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/lessons/feedback_retro_bypass_governance_log.md
modifies: []
requires:
  - task: 7
    reason: "The lesson cites Task 7's emit_retro_bypass_log + the JSONL fields."
  - task: 8
    reason: "The lesson cites Task 8's governance-log-kinds-jsonl.md doc + the retro_bypass kind registry entry."
```

**IMPLEMENT (file 1 of 1):** in
`.claude/lessons/feedback_retro_bypass_governance_log.md`, follow
§10.4 frontmatter shape. Body sections:

1. **Frontmatter:**
   ```yaml
   ---
   name: retro-check.sh fail-open emits a JSONL governance-log entry per bypass
   description: After 3 attempted Stop hooks without a written retro, retro-check.sh fails open by design (loops are real). v1-rls-r1 adds an additive JSONL trail to .claude/governance-log/retro-bypass.jsonl so the bypass surface is countable and the autonomy-readiness criterion 5.2 (bypass rate monotonically decreasing) is measurable. Bypass behaviour unchanged; the trail is the structural fix.
   type: feedback
   ---
   ```
2. `# retro-check.sh fail-open → retro_bypass JSONL trail` H1.
3. **The bypass class** — one paragraph explaining the 3-attempt
   fail-open is by design for true loops; the previous gap was
   invisibility, not the fail-open itself.
4. **The structural fix** — one paragraph naming the JSONL trail
   + the kind registry doc + the weekly-review consumption.
5. **The audit signal** — one paragraph: rate of `retro_bypass`
   entries per week should be monotonically decreasing per RLS-PMD
   review §5.2.
6. **How to apply** — bulleted:
   - **At session start**: nothing required (the trail is passive).
   - **Weekly cadence**: weekly-review Step 2c retro-harvest sweep
     scans `.claude/governance-log/retro-bypass.jsonl` for the
     prior 7 days; surfaces in the weekly summary.
   - **Quarterly**: trend the count week-over-week; if rising,
     surface as a calibration-honesty signal in the next phase
     retro.
7. **See also**:
   - `.claude/hooks/retro-check.sh` (Task 7)
   - `docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`
     (Task 8)
   - `.claude/skills/weekly-review/SKILL.md` Step 2c (Task 4)
   - `.claude/rules/pmd-invariants.md` (Task 2)
   - `feedback_lesson_must_pair_with_structural_fix_when_fixable.md`
   - `docs/research/brehon-rls-pmd-review.md` §4.7 + §5.2

**MIRROR:** `.claude/lessons/feedback_mcp_canonical_pmd_path_enforce_
at_session_start.md` (the prior lesson-pairs-with-spec pattern; this
lesson is structurally similar).

**GOTCHA:** the lesson does NOT propose changing the 3-attempt cap.
The fix is the TRAIL, not the cap. Per R6 + Watchpoint #7.

**GOTCHA:** `description:` per Watchpoint #5 — plain language; no
auto-trigger keyword stuffing.

**GOTCHA:** if any brief-referenced cross-link target (e.g.
`feedback_one_system_memory_in_repo.md`) is absent at impl time,
SKIP that cross-link; cite only verified-present lessons. Do NOT
file `kind: "blocker"` over missing cross-link targets — the
lesson stands without them.

**VALIDATE (story-checkpoint feeds §16a Story 4):**

```bash
# File exists
test -f .claude/lessons/feedback_retro_bypass_governance_log.md
echo "exit: $?"
# EXPECT: exit 0

# Frontmatter valid
python3 -c "
import re, yaml
content = open('.claude/lessons/feedback_retro_bypass_governance_log.md').read()
m = re.match(r'^---\n(.*?)\n---', content, re.S)
assert m, 'no frontmatter'
fm = yaml.safe_load(m.group(1))
assert fm['type'] == 'feedback'
forbidden = ['DO use when', 'MUST USE', 'ALWAYS INVOKE', 'CRITICAL']
for kw in forbidden:
    assert kw not in fm['description'], f'forbidden keyword: {kw}'
print('OK')
"
# EXPECT: OK

# Cross-links the instrumentation + the schema doc
grep -c "retro-check.sh\|governance-log-kinds-jsonl" .claude/lessons/feedback_retro_bypass_governance_log.md
# EXPECT: ≥ 2

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task9-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 10: DOGFOOD — three integration probes + report

**ACTION:** run three dogfood probes per PRECON-8 + R9 + Watchpoint
#6; write the report at `.claude/PRPs/reports/v1-rls-r1-dogfood-
<YYYY-MM-DD>.md`. This task is the integration test for Tracks B+C.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/PRPs/reports/v1-rls-r1-dogfood-<YYYY-MM-DD>.md
modifies: []
requires:
  - task: 3   # pmd-canonical-guard.sh script for 4.1 dogfood
  - task: 4   # weekly-review Step 2c for 4.6 dogfood
  - task: 5   # .gitignore harvest dir + governance-log dir for runtime journals
  - task: 6   # bootstrap-checklist lesson — verifies the wiring is documented
  - task: 7   # retro-check.sh fail-open instrumentation for 4.7 dogfood
  - task: 8   # governance-log-kinds-jsonl.md — verifies the schema doc is in place
  - task: 9   # retro_bypass lesson — verifies cross-links resolve
```

**IMPLEMENT (file 1 of 1):** in
`.claude/PRPs/reports/v1-rls-r1-dogfood-<YYYY-MM-DD>.md`, run three
probes and write the report. Substitute `<YYYY-MM-DD>` with the
date the dogfood runs.

**Dogfood 4.1 — `pmd-canonical-guard.sh`:**

```bash
# Sub-run (a): against this lane's .mcp.json (canonical match expected)
bash .claude/hooks/pmd-canonical-guard.sh 2>/tmp/dogfood-41a.stderr
echo "exit: $?" > /tmp/dogfood-41a.exit
# EXPECT: exit 0, stderr empty

# Sub-run (b): against a deliberate-mispoint sentinel
SENTINEL_DIR="/tmp/v1-rls-r1-sentinel-$$"
mkdir -p "$SENTINEL_DIR" && cd "$SENTINEL_DIR"
cat > .mcp.json <<'EOF'
{
  "mcpServers": {
    "project-memory": {
      "env": {
        "PROJECT_MEMORY_DB": "/tmp/wrong/path.db"
      }
    }
  }
}
EOF
bash <worktree-path>/.claude/hooks/pmd-canonical-guard.sh 2>/tmp/dogfood-41b.stderr
echo "exit: $?" > /tmp/dogfood-41b.exit
cd <worktree-path>
rm -rf "$SENTINEL_DIR"
# EXPECT (a): exit 0, stderr empty (canonical match)
# EXPECT (b): exit 0, stderr contains "WARN" + both paths + 1-line fix
```

**Dogfood 4.6 — weekly-review Step 2c retro-harvest sweep:**

Walk the new Step 2c against `.claude/PRPs/reports/*.md` for the
last 30 days. The dogfood does NOT execute weekly-review end-to-end
(that's a Sunday cadence + Junior task); it executes ONLY the new
Step 2c sweep logic against the existing retro corpus.

```bash
# List retro files mtime ≤ 30 days
find .claude/PRPs/reports -maxdepth 1 -name '*.md' -mtime -30 -printf '%f\n'

# For each retro, identify proposals NOT yet promoted to .claude/lessons/
# (planner's Step 2c body specifies the exact grep + cross-check;
# the dogfood walker follows that body literally)

# Write the dogfood harvest output
mkdir -p /tmp/v1-rls-r1-harvest-dogfood
# ... (run the Step 2c logic; output to /tmp/v1-rls-r1-harvest-dogfood/<iso-week>.md)
```

**EXPECT:** the harvest output surfaces ≥ 1 known proposal — the
cycle-count-≥3 catch-fire proposal from
`feedback_plan_stub_uniformity_with_canonical_sibling.md`
(per RLS-PMD review §4.6 evidence; this is the canonical
surface-target the dogfood validates against). If the proposal is
not surfaced, the dogfood FAILS and the impl-task worker files
`kind: "blocker"` against the Step 2c body.

**Dogfood 4.7 — synthetic 3-attempt fail-open trigger:**

```bash
# Force a 3-attempt fail-open by manipulating SESSION_FILE
SESSION_DIR="/tmp/cc-retro-sessions"
mkdir -p "$SESSION_DIR"
SESSION_FILE="${SESSION_DIR}/${PPID:-dogfood-test}"
echo "3" > "$SESSION_FILE"

# Source the function directly to test JSONL emit deterministically:
CLAUDE_PROMPT="dogfood synthetic trigger" \
  bash -c 'source .claude/hooks/retro-check.sh; emit_retro_bypass_log 3 "phase-v1-rls-r1"'

# Verify JSONL record exists
test -f .claude/governance-log/retro-bypass.jsonl
tail -1 .claude/governance-log/retro-bypass.jsonl | jq .
# EXPECT (a): file exists
# EXPECT (b): jq parses valid JSON
# EXPECT (c): JSON has all required fields: timestamp, session_id,
#             attempt_count, prompt_hash, branch_at_fail_open, kind
```

**EXPECT:** the JSONL record contains all six fields from DQ #297
(`timestamp`, `session_id`, `attempt_count`, `prompt_hash`,
`branch_at_fail_open`, `kind: "retro_bypass"`). If any field is
missing or wrong-typed, the dogfood FAILS.

**Report body** (`.claude/PRPs/reports/v1-rls-r1-dogfood-
<YYYY-MM-DD>.md` shape):

1. `# v1-rls-r1 dogfood report` H1.
2. `## 1. Dogfood 4.1 — pmd-canonical-guard.sh` — sub-run (a) +
   (b) outputs verbatim.
3. `## 2. Dogfood 4.6 — weekly-review Step 2c retro-harvest sweep`
   — harvest output + verification against the known surface-
   target.
4. `## 3. Dogfood 4.7 — synthetic 3-attempt fail-open` — JSONL
   record verbatim + field verification.
5. `## 4. Outcome` — one paragraph: each sub-run's pass/fail; if
   any fail, what's filed as `kind: "blocker"`.

**MIRROR:** `.claude/PRPs/plans/brehon-conformance-audit.plan.md`
Task 7 "DOGFOOD" body for shape + verbatim-output discipline.

**GOTCHA:** Task 10 `requires:` lists Tasks 3, 4, 5, 6, 7, 8, 9 —
ALL must be on `phase-v1-rls-r1` for the dogfood to run. The
advisor enforces via cohort dispatch.

**GOTCHA:** the dogfood runs on the laptop (per PRECON-7 +
validate-pending-laptop discipline). It is NOT a Junior task —
this task is dispatched as `impl-task` but its `VALIDATE` step
walks the runtime dogfood; the impl-task worker captures stdout
+ stderr per sub-run.

**GOTCHA:** if any sub-run FAILS, the worker files `kind: "blocker"`
DQ and does NOT commit the report. The advisor catch-fires per
§5.5 of advisor-orchestrator.md.

**VALIDATE (story-checkpoint feeds §16a Story 5):**

```bash
# Report file exists
ls .claude/PRPs/reports/v1-rls-r1-dogfood-*.md
echo "exit: $?"
# EXPECT: exit 0 (one matching file)

# Report has four H2 sections
grep -cE "^## " .claude/PRPs/reports/v1-rls-r1-dogfood-*.md
# EXPECT: ≥ 4

# All three dogfood sub-runs covered
grep -cE "Dogfood 4\.[167]" .claude/PRPs/reports/v1-rls-r1-dogfood-*.md
# EXPECT: 3

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task10-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 11: WIRE v1-rls-r1 deliverables into `.claude/rules/advisor-orchestrator.md`

**ACTION:** edit `.claude/rules/advisor-orchestrator.md` with two
surgical insertions: §1 polling-loop addendum (one bullet) +
§5 new sub-section "Retro-bypass observability". Per Track D
scope.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - .claude/rules/advisor-orchestrator.md   # §1 one-bullet note + §5 new sub-section
requires:
  - task: 3   # the script the §1 note references
  - task: 7   # the instrumentation §5 sub-section describes
  - task: 8   # the schema doc §5 sub-section cross-links
  - task: 9   # the lesson §5 sub-section cross-links
```

**IMPLEMENT (file 1 of 1):** in
`.claude/rules/advisor-orchestrator.md`, TWO edits:

1. **§1 "Polling loop" addendum** — append ONE bullet at the end
   of the existing §1 bulleted list (read the file in full first
   to identify the right insertion point):

   > - **SessionStart canonical-PMD guard (post-v1-rls-r1):** the
   >   tracked `.claude/hooks/pmd-canonical-guard.sh` runs at every
   >   session start in lanes wired per the bootstrap checklist
   >   (`feedback_phase_lane_worktree_bootstrap_checklist.md` step
   >   6). Surfaces as a stderr WARN on lane drift, exit 0 always
   >   — does NOT block. Per `.claude/rules/pmd-invariants.md`
   >   invariant #5.

2. **§5 new sub-section** — add a new H3 sub-section under
   §5 "Validation, classification, recovery". Worker reads §5 in
   full FIRST and picks the right insertion point:
   - If the existing §5 has sub-sections numbered 5.1-5.5 (or
     similar), insert the new sub-section as **§5.X** where X is
     the next available number (could be 5.5 or 5.6 depending on
     current numbering). If a renumber is needed (e.g. existing
     §5.5 "Catch-fire procedures" must become §5.6), do that
     renumber.
   - Planner lean (NOT binding): insert AS NEW §5.5 BETWEEN the
     existing §5.4 (DQ triage) and existing §5.5 (Catch-fire) —
     because "retro-bypass observability" is the *evidence*
     feeding the existing catch-fire decision. Worker confirms
     after reading.

   The new sub-section body:

   > ### 5.X Retro-bypass observability
   >
   > Per RLS-PMD review §4.7 + autonomy-readiness criterion 5.2 +
   > `.claude/PRPs/plans/v1-rls-r1.plan.md` Task 7. The Stop hook
   > `.claude/hooks/retro-check.sh` fail-open path (3-attempt cap,
   > load-bearing for true loops) emits a JSONL `retro_bypass`
   > record to `.claude/governance-log/retro-bypass.jsonl` on every
   > fail-open. Fields per the kind registry at
   > `docs/brehon-law-inspired-network/governance-log-kinds-
   > jsonl.md`.
   >
   > **Consumer:** weekly-review Step 2c retro-harvest sweep
   > scans the JSONL for the prior 7 days; surfaces in the weekly
   > summary. **Autonomy signal:** the rate of `retro_bypass`
   > entries per week should be monotonically decreasing. Rising
   > rate → calibration-honesty regression; surface in the next
   > phase retro.
   >
   > **Advisor-side action:** none required at session-start (the
   > trail is passive). At retro time, the rate trend is part of
   > the four-role retro signals (Advisor role) per
   > `feedback_four_role_retro_signals.md`. See
   > `feedback_retro_bypass_governance_log.md`.

**MIRROR:** existing §5.1-§5.4 sub-sections — same H3 + bullet
prose style.

**GOTCHA:** the §1 addendum is one bullet, NOT a paragraph. Per
the existing §1 bulleted-list style.

**GOTCHA:** the rule file changes commit subject MUST match the
attribution pattern `^(chore|docs)\((advisor|decision-queue)\)`
per `.claude/rules/decision-queue.md` Attribution-integrity §2.
Recommended: `docs(rules): wire v1-rls-r1 SessionStart guard +
§5 retro-bypass observability` (the impl-task subagent writes
this commit).

**VALIDATE (story-checkpoint feeds §16a Story 5):**

```bash
# §1 addendum present
grep -c "SessionStart canonical-PMD guard" .claude/rules/advisor-orchestrator.md
# EXPECT: ≥ 1

# §5.X sub-section present (any of 5.5/5.6)
grep -cE "^### 5\.[0-9]+ Retro-bypass" .claude/rules/advisor-orchestrator.md
# EXPECT: 1

# pmd-canonical-guard.sh referenced
grep -c "pmd-canonical-guard.sh" .claude/rules/advisor-orchestrator.md
# EXPECT: ≥ 1

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task11-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 12 [P]: AUTHOR paired lesson files closing prior PENDING statuses

**ACTION:** author two NEW lesson files per Track D scope:
- `feedback_pmd_canonical_guard_enforces_invariant.md` — closes the
  `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
  PENDING status.
- `feedback_retro_harvest_weekly_cadence.md` — closes the
  retro-harvest ad-hoc gap.

The two files share no IMPLEMENT lines and have no `requires:`
between them — `[P]` between themselves; both `requires: [3, 7, 10]`.

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/lessons/feedback_pmd_canonical_guard_enforces_invariant.md
  - .claude/lessons/feedback_retro_harvest_weekly_cadence.md
modifies: []
requires:
  - task: 3   # cites pmd-canonical-guard.sh
  - task: 7   # cites retro-check.sh fail-open instrumentation
  - task: 10  # dogfood report is one of the See-also references
```

**IMPLEMENT (file 1 of 2):** in
`.claude/lessons/feedback_pmd_canonical_guard_enforces_invariant.md`,
follow §10.4 frontmatter. Body:

1. **Frontmatter:**
   ```yaml
   ---
   name: SessionStart canonical-PMD guard is now SHIPPED (closes PENDING status)
   description: Pre-v1-rls-r1 the canonical-PMD cross-lane invariant was protected by documentation only. v1-rls-r1 shipped .claude/hooks/pmd-canonical-guard.sh + the lane-bootstrap-checklist update; the SessionStart wiring is per-lane in .claude/settings.local.json per the updated checklist. The prior lesson's fix status: PENDING is now closed.
   type: feedback
   ---
   ```
2. `# pmd-canonical-guard.sh — canonical-PMD invariant enforced at SessionStart` H1.
3. **Pre-v1-rls-r1 state** — one paragraph: lesson `feedback_
   mcp_canonical_pmd_path_enforce_at_session_start.md` carried
   `fix status: PENDING`; documentary `.mcp.json.example` marker
   alone failed to prevent the v1-ship-1 stranding incident.
4. **Post-v1-rls-r1 state** — one paragraph: tracked script ships,
   bootstrap-checklist documents the wiring, two settings.local.json
   files (canonical + this-lane) wired per DQ #301; SessionStart
   WARN-not-FAIL surfaces lane drift loudly.
5. **How to apply** — bulleted:
   - **New lanes**: apply step 6 of the bootstrap checklist.
   - **Existing lanes**: wire on next bootstrap cycle.
   - **Drift detection**: if a SessionStart shows the WARN banner,
     edit `.mcp.json` per the banner's 1-line fix + restart MCP.
6. **See also**:
   - `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`
     (the prior PENDING this closes)
   - `feedback_pmd_cross_lane_canonical_db.md`
   - `feedback_phase_lane_worktree_bootstrap_checklist.md` (Task 6)
   - `.claude/rules/pmd-invariants.md` invariant #1 + #5 (Task 2)
   - `.claude/hooks/pmd-canonical-guard.sh` (Task 3)
   - `.claude/PRPs/plans/v1-rls-r1.plan.md` (this plan)

**IMPLEMENT (file 2 of 2):** in
`.claude/lessons/feedback_retro_harvest_weekly_cadence.md`, follow
§10.4 frontmatter. Body:

1. **Frontmatter:**
   ```yaml
   ---
   name: retro-harvest folded into weekly-review Step 2c (Sunday cadence)
   description: Pre-v1-rls-r1 the .claude/skills/retro-harvest/SKILL.md skill ran ad-hoc and ~48 proposals accumulated unread across 52 retros. v1-rls-r1 added Step 2c to weekly-review/SKILL.md so retro-harvest fires on the same Sunday 02:00 UTC cadence as backfill and lesson promotion. One weekly artifact at .claude/harvest/<iso-week>.md (gitignored). SURFACING not auto-promoting.
   type: feedback
   ---
   ```
2. `# retro-harvest weekly cadence` H1.
3. **Pre-v1-rls-r1 state** — one paragraph: 48 unchecked proposals
   across 52 retros (RLS-PMD review §4.6 evidence); cycle_count ≥ 3
   catch-fire proposal sat 7 days before it would have prevented
   ~123-min loss.
4. **Post-v1-rls-r1 state** — one paragraph: Step 2c folds the
   existing retro-harvest skill into the weekly cadence; output
   `.claude/harvest/<iso-week>.md`.
5. **How to apply** — bulleted:
   - **Weekly cadence**: weekly-review Step 2c fires automatically
     on Sunday 02:00 UTC.
   - **Manual review thereafter**: the human reads the weekly
     harvest artifact and decides promotions per RLS-PMD review
     §4.6 contract.
   - **Quarterly trend**: count of surfaced-but-unpromoted
     proposals week-over-week — rising is a calibration signal.
6. **See also**:
   - `.claude/skills/retro-harvest/SKILL.md` (the consumed skill)
   - `.claude/skills/weekly-review/SKILL.md` Step 2c (Task 4)
   - `docs/research/brehon-rls-pmd-review.md` §4.6
   - `.claude/PRPs/plans/v1-rls-r1.plan.md` (this plan)

**MIRROR:** §10.4 lesson frontmatter shape + Task 9's lesson body
as a peer canonical-shape sample.

**GOTCHA:** **`[P]` between the two lesson files** — `union(creates,
modifies)` for file 1 = `[feedback_pmd_canonical_guard_enforces_
invariant.md]`; for file 2 = `[feedback_retro_harvest_weekly_
cadence.md]`; intersect = ∅. Disjoint. Worker may author in either
order.

**GOTCHA:** `description:` per Watchpoint #5 — both descriptions are
plain language.

**GOTCHA:** **Task 12 is not the closure of every prior PENDING.**
The MCP write-time embedding gap (item 4.2) is still PENDING
post-v1-rls-r1 — the spec ships in Task 1 but the patch ships in a
separate `MCPs/project-memory-mcp` PR per PRECON-2. The
`feedback_pmd_backfill_after_write.md` lesson stays in place and
its mitigations (Step 5.5 + Step 1b) remain load-bearing until the
MCP patch lands. Track D Task 12 does NOT close that PENDING.

**VALIDATE (story-checkpoint feeds §16a Story 5):**

```bash
# Both files exist
test -f .claude/lessons/feedback_pmd_canonical_guard_enforces_invariant.md
test -f .claude/lessons/feedback_retro_harvest_weekly_cadence.md
echo "exit: $?"
# EXPECT: exit 0

# Both have valid frontmatter (no auto-trigger keyword stuffing)
python3 -c "
import re, yaml
files = [
  '.claude/lessons/feedback_pmd_canonical_guard_enforces_invariant.md',
  '.claude/lessons/feedback_retro_harvest_weekly_cadence.md',
]
for f in files:
    content = open(f).read()
    m = re.match(r'^---\n(.*?)\n---', content, re.S)
    assert m, f'no frontmatter in {f}'
    fm = yaml.safe_load(m.group(1))
    assert fm['type'] == 'feedback'
    forbidden = ['DO use when', 'MUST USE', 'ALWAYS INVOKE', 'CRITICAL']
    for kw in forbidden:
        assert kw not in fm['description'], f'forbidden keyword in {f}: {kw}'
print('OK')
"
# EXPECT: OK

# Sanity workspace check
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task12-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### Task 13: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. One H2 per role
(Advisor / Planning / Impl / BM) with signals + lessons. Promote
any new lessons to `.claude/lessons/feedback_*.md` in the same
retro commit (per the retro-not-report discipline).

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/PRPs/reports/v1-rls-r1-retro.md
modifies: []
requires:
  - task: 10
  - task: 11
  - task: 12
```

**IMPLEMENT (file 1 of 1):** in
`.claude/PRPs/reports/v1-rls-r1-retro.md`, follow the canonical
retro shape:

1. **`# v1-rls-r1 retrospective`** H1.
2. **Three canonical H2 headers** (per
   `feedback_retro_not_report.md`):
   - `## What surprised us`
   - `## What to change`
   - `## What to carry forward`
3. **Per-role signals** (per `feedback_four_role_retro_signals.md`)
   — one H3 sub-section per role under each H2 if there's role-
   specific signal; otherwise narrative.
4. **Per-task complexity score** (per
   `feedback_retro_task_complexity_score.md`) — for each of Tasks
   1-12, the
   `<files-touched>/<commits>/<runtime-min>/<max-log-silence-min>`
   quad; aggregated rollup in a closing § if useful.
5. **Decisions to revisit** — bulleted list of judgment calls the
   planner / impl-task workers made that future v1-rls-* sub-phases
   might re-litigate (e.g. the `.claude/harvest/` gitignored-vs-
   tracked choice; the `pmd-invariants.md` auto-loading vs `paths:`
   frontmatter scoping; the Track C kind-registry placement in
   docs/ vs `.claude/refs/`).
6. **Lessons promoted in this retro commit** — any new lesson the
   retro surfaces (NOT a lesson file Task 12 already shipped; this
   is for new patterns the retro-authoring uncovers). If none,
   note "no new lessons promoted".
7. **Harness gap to flag**: the `.claude/**` write-protection /
   Junior-worker harness gap recurred during v1-rls-r1 planning
   (same as v1-fed-in-a). Adding `.claude/PRPs/plans/**` +
   `.claude/decision-queue.json` to the Junior worker permission
   allowlist is a candidate v1-rls-r2 item.

**MIRROR:** the most recent shipped retro at
`.claude/PRPs/reports/v1-*-retro.md`. Worker `Glob`s
`.claude/PRPs/reports/v1-*-retro.md` and Reads the most-recent
file first.

**GOTCHA:** retros write the past tense; the H2 headers are
exactly the three canonical strings (no synonyms, no expansions).

**GOTCHA:** the retro file is committed in the SAME commit as any
new lesson the retro surfaces (per the retro-not-report skill's
auto-promote step). Worker checks this discipline at commit time.

**GOTCHA:** Task 13 is the TERMINAL barrier — no `[P]`. Runs
after Tasks 10-12 complete.

**VALIDATE (story-checkpoint feeds §16a Story 5):**

```bash
# File exists
test -f .claude/PRPs/reports/v1-rls-r1-retro.md
echo "exit: $?"
# EXPECT: exit 0

# Three canonical H2 headers
grep -cE "^## (What surprised us|What to change|What to carry forward)$" .claude/PRPs/reports/v1-rls-r1-retro.md
# EXPECT: 3

# Sanity workspace check (last one)
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task13-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

---

## 14. Testing strategy

v1-rls-r1 ships zero Rust changes; the §15 cargo runs are sanity-
only against an unchanged workspace. Layer-by-layer:

- **Workspace compile (sanity):** `bash scripts/brehon/cargo-
  check.sh --workspace --features full` — exit 0 after every task
  (confirms no incidental Rust breakage from `.claude/` / `docs/`
  edits).
- **Workspace clippy (sanity):** `bash scripts/brehon/cargo-
  clippy.sh --workspace --no-deps --features full -- -D warnings`
  — exit 0 (confirms no clippy regression from any incidental
  cross-cutting change).
- **e2e link-target:** `cargo test --test e2e --no-run -p
  lemmy_server` — exit 0 (confirms phase-branch test-link still
  works post-v1-rls-r1).
- **Hook script probes (per Task 3 + Task 7):** runtime exit-code
  + stderr-content assertions per the VALIDATE blocks of each
  task.
- **Frontmatter probes (per Tasks 2 + 6 + 9 + 12):** Python YAML
  parse + Watchpoint #5 forbidden-keyword absence checks.
- **Cross-link probes (per Task 8):** grep + presence checks for
  the cross-link sentence in 04.
- **Dogfood (Task 10):** the integration test for Tracks B+C.
  Three sub-runs (4.1 / 4.6 / 4.7) per PRECON-8.

**No e2e test (cargo test --test e2e):** v1-rls-r1 ships no Rust;
the existing e2e suite is sanity-checked via `--no-run` only.

**No migration round-trip:** v1-rls-r1 ships no migrations.

---

## 15. Validation commands (DoD)

> **PRECON-7 laptop-shape (NOT Shape G).** Per DQ #229 + advisor-
> orchestrator §5.2. Each impl-task raises `kind: "validate-pending-
> laptop"` post-push naming §15 DoD commands verbatim with
> `--workspace --features full`.

### 15.1 Per-task workspace check (Tasks 1-12)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-rls-r1-task<N>-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-rls-r1-task<N>-check.log
```

**EXPECT:** exit 0. v1-rls-r1 ships zero Rust changes, so every
per-task check is a sanity confirmation that the working tree
remains workspace-compilable.

### 15.2 Per-task clippy (Tasks 1-12 — R6)

```bash
bash scripts/brehon/cargo-clippy.sh --workspace --no-deps --features full -- -D warnings > .claude/PRPs/debug/v1-rls-r1-task<N>-clippy.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-rls-r1-task<N>-clippy.log
```

**EXPECT:** exit 0.

### 15.3 e2e test-target compile (R7 — every task)

```bash
cargo test --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-rls-r1-task<N>-test-norun.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-rls-r1-task<N>-test-norun.log
```

**EXPECT:** exit 0. Confirms phase-branch test-link still works.

### 15.4 Hook script probe (Task 3 — `pmd-canonical-guard.sh`)

```bash
# Sub-probe (a): current lane (canonical match)
bash .claude/hooks/pmd-canonical-guard.sh 2>/tmp/v1-rls-r1-guard-a.stderr
echo "exit: $?"
# EXPECT: exit 0, stderr empty

# Sub-probe (b): sentinel mispoint
SENTINEL="/tmp/v1-rls-r1-guard-sentinel-$$"
mkdir -p "$SENTINEL" && cd "$SENTINEL"
cat > .mcp.json <<'EOF'
{"mcpServers":{"project-memory":{"env":{"PROJECT_MEMORY_DB":"/tmp/wrong/path.db"}}}}
EOF
bash <worktree-path>/.claude/hooks/pmd-canonical-guard.sh 2>/tmp/v1-rls-r1-guard-b.stderr
echo "exit: $?"
cd <worktree-path>
rm -rf "$SENTINEL"
# EXPECT: exit 0, stderr contains "WARN" + both paths
```

### 15.5 Phase 2 e2e — N/A

v1-rls-r1 ships **zero Rust changes** — no Phase 2 e2e dispatch.
User Gate 4 (Phase 2 e2e — local vs dispatch) is **skipped**
per PRECON-7. The advisor's stage-shape (advisor-orchestrator
§3.1) skips the Phase 2 e2e step for this sub-phase. The §15.3
`--no-run` check is the e2e-coverage proxy.

### 15.6 Shape G section — DORMANT

**NOT applicable.** Per PRECON-7 + DQ #229 (Shape G suspended
until 2026-06-01).

### 15.7 Cross-cutting verification

- [ ] **R1:** Every hook script in v1-rls-r1 exits 0 on every
  code path (Watchpoint #1).
- [ ] **R2:** `.claude/rules/pmd-invariants.md` enumerates exactly
  5 numbered invariants (Watchpoint #2).
  `grep -cE "^## [1-5]\. " .claude/rules/pmd-invariants.md`
  returns 5.
- [ ] **R3:** No script/hook/skill in v1-rls-r1 invokes `cargo` at
  runtime (Watchpoint #3).
  `grep -rE "cargo\s+(check|clippy|test|build)" .claude/hooks/
  pmd-canonical-guard.sh .claude/skills/weekly-review/SKILL.md
  | grep -v "^#"` returns 0 hits in NEW/EDITED scope.
- [ ] **R4:** Every DQ entry filed during v1-rls-r1 (planner +
  impl-task workers) commits + pushes mid-task (Watchpoint #4).
- [ ] **R5:** No auto-trigger keyword stuffing in
  `description:` fields of any new lesson, rule, or skill section
  (Watchpoint #5).
- [ ] **R6:** `retro-check.sh` fail-open path is ADDITIVE only
  (Watchpoint #7). `git diff governance-v0..phase-v1-rls-r1 --
  .claude/hooks/retro-check.sh` shows: function appended at end +
  one invoke line; no other lines changed.
- [ ] **R7:** Every invariant in `pmd-invariants.md` either names
  a SHIPPED enforcement OR carries explicit `enforcement: PENDING`
  text (Watchpoint #8).
- [ ] **R8:** Every new artifact's worker Read 1-2 canonical
  siblings before authoring (verified via per-task `MIRROR:`
  blocks).
- [ ] **R9:** Task 10 dogfood runs ALL three sub-probes (4.1 /
  4.6 / 4.7); report contains verbatim outputs.
- [ ] **No new ADR:** `git diff governance-v0..phase-v1-rls-r1 --
  docs/brehon-law-inspired-network/99-decisions-and-open-
  questions.md` shows 0 changes.
- [ ] **No `crates/**` edits:** `git diff governance-v0..phase-v1-
  rls-r1 -- crates/` shows 0 changes.
- [ ] **No migrations:** `git diff governance-v0..phase-v1-rls-r1
  -- migrations/` shows 0 changes.
- [ ] **`.gitignore` updated:** both `.claude/harvest/` and
  `.claude/governance-log/` lines present.

### 15.8 ADR / OQ compliance

- [ ] **No ADR change.** Per PRECON-5. The ADR corpus (1-15) is
  untouched.
- [ ] **ADR-008 honoured** — the v1 product governance_log PG
  table is NOT modified; Task 8's new doc EXPLICITLY distinguishes
  hook-emitted JSONL observability from the product surface.
- [ ] **PRECON-2 honoured** — the actual MCP `dist/index.js` patch
  is NOT shipped; Track A Task 1 ships the spec only.
- [ ] **PRECON-5 honoured** — exactly five PMD invariants in
  `pmd-invariants.md`; no sixth, no fourth.
- [ ] **PRECON-7 honoured** — Shape G NOT used; `validate-pending-
  laptop` DoD shape per advisor-orchestrator §5.2.
- [ ] **PRECON-9 honoured** — bootstrap-checklist lesson updated;
  dual-wire documented; settings.local.json files NOT in any
  `creates:` / `modifies:` array.

---

## 16. Acceptance criteria

- [ ] All 13 tasks (Task 0 pre-flight + Tasks 1-12 + Task 13 retro)
  committed in dependency order.
- [ ] §15.1 (cargo check) exit 0 after every task.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after
  every task.
- [ ] §15.3 (cargo test --no-run) exit 0 after every task.
- [ ] §15.4 (hook probe) — both sub-probes pass per Task 3 VALIDATE
  block.
- [ ] §15.7 (cross-cutting verification) — all 12 boxes ticked.
- [ ] §15.8 (ADR / OQ compliance) — all 6 boxes ticked.
- [ ] §16a stories — all 5 stories `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per Task 13.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] Dogfood report at
  `.claude/PRPs/reports/v1-rls-r1-dogfood-<YYYY-MM-DD>.md` shows
  all three sub-runs pass.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Track A artifacts shipped — spec + invariants rule

- **Composing tasks:** Task 1, Task 2 (Cohort A `[P]` — disjoint
  FILES YAML).
- **Checkpoint command:**
  ```bash
  test -f .claude/PRPs/specs/mcp-write-time-embedding.md \
    && test -f .claude/PRPs/specs/README.md \
    && test -f .claude/rules/pmd-invariants.md \
    && grep -cE "^## [1-5]\. " .claude/rules/pmd-invariants.md
  ```
- **Expected output:** all three `test -f` exit 0; `grep -cE`
  returns `5`.
- **Brief-Scope outputs to verify** (consumed by `/brehon-verify`):
  - `.claude/PRPs/specs/mcp-write-time-embedding.md` contains six
    H2 sections (Authority / Patch insertion point / Graceful-
    fallback contract / Verification recipe / Downstream-collapse
    claim / Verification deferred).
  - `.claude/PRPs/specs/README.md` is ≤ 10 lines and describes the
    directory's purpose.
  - `.claude/rules/pmd-invariants.md` has H1 `# PMD invariants`
    and exactly 5 numbered H2 sub-sections — invariants 1-5 per
    §10.8 verbatim text.
  - Each invariant in `pmd-invariants.md` carries a "How to apply"
    line (either shipped-enforcement or `enforcement: PENDING`).

### Story 2: SessionStart guard wired + checklist updated

- **Composing tasks:** Task 3 (`[P]`), Task 6 (depends on Task 3).
- **Checkpoint command:**
  ```bash
  test -x .claude/hooks/pmd-canonical-guard.sh \
    && bash .claude/hooks/pmd-canonical-guard.sh \
    && test -f .claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md
  ```
- **Expected output:** all three exits = 0; canonical-match probe
  produces no stderr WARN.
- **Brief-Scope outputs to verify:**
  - `.claude/hooks/pmd-canonical-guard.sh` exists + is executable
    + starts with `#!/usr/bin/env bash` + has `set -euo pipefail`.
  - The script's body uses the §10.1 mirror pattern (`git rev-
    parse --git-common-dir` resolution).
  - `feedback_phase_lane_worktree_bootstrap_checklist.md` contains
    a numbered step referencing `pmd-canonical-guard.sh` and the
    `.claude/settings.local.json` JSON snippet.
  - The lesson documents the manual dual-wire in both canonical +
    this-lane settings.local.json files.

### Story 3: weekly-review Step 2c retro-harvest + harvest dir gitignored

- **Composing tasks:** Task 4 (`[P]`), Task 5 (depends on Task 4).
- **Checkpoint command:**
  ```bash
  grep -c "^### 2c\. " .claude/skills/weekly-review/SKILL.md \
    && grep -cE "^\.claude/(harvest|governance-log)/$" .gitignore
  ```
- **Expected output:** Step 2c present (count = 1);
  `.gitignore` lines (count = 2).
- **Brief-Scope outputs to verify:**
  - `.claude/skills/weekly-review/SKILL.md` has the new Step 2c
    heading between current Steps 2b and 3 (existing Step 3-6
    numbering UNCHANGED).
  - Step 2c body mentions `.claude/harvest/<iso-week>.md`,
    invokes / duplicates retro-harvest skill logic, and includes
    the "SURFACING, not auto-promoting" disclaimer.
  - `.gitignore` has both `.claude/harvest/` and
    `.claude/governance-log/` lines.

### Story 4: retro_bypass instrumentation — function + schema doc + lesson

- **Composing tasks:** Task 7 (`[P]`), Task 8 (depends on Task 7),
  Task 9 (depends on Tasks 7+8).
- **Checkpoint command:**
  ```bash
  grep -c "^emit_retro_bypass_log()" .claude/hooks/retro-check.sh \
    && grep -A1 "rm -f \"\$SESSION_FILE\"" .claude/hooks/retro-check.sh | grep -c emit_retro_bypass_log \
    && test -f docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md \
    && test -f .claude/lessons/feedback_retro_bypass_governance_log.md \
    && bash -n .claude/hooks/retro-check.sh
  ```
- **Expected output:** all returns positive + final `bash -n`
  exits 0.
- **Brief-Scope outputs to verify:**
  - `.claude/hooks/retro-check.sh` fail-open branch invokes
    `emit_retro_bypass_log` BEFORE `exit 0`.
  - `emit_retro_bypass_log` function writes JSONL to
    `.claude/governance-log/retro-bypass.jsonl` with 6 fields
    (timestamp, session_id, attempt_count, prompt_hash,
    branch_at_fail_open, kind).
  - `governance-log-kinds-jsonl.md` has three H2 sections (Scope
    / Kind registry / Adding a new kind) + `retro_bypass` registry
    row.
  - `04-data-model-and-api.md` has the cross-link sentence
    pointing at the new doc.
  - `feedback_retro_bypass_governance_log.md` cross-links the
    instrumentation, the schema doc, and the autonomy criterion
    5.2.

### Story 5: wiring + lessons + dogfood + retro complete

- **Composing tasks:** Task 10 (dogfood), Task 11 (orchestrator
  wiring), Task 12 (paired lessons, `[P]` internally), Task 13
  (retro — terminal barrier).
- **Checkpoint command:**
  ```bash
  ls .claude/PRPs/reports/v1-rls-r1-dogfood-*.md \
    && grep -c "pmd-canonical-guard.sh" .claude/rules/advisor-orchestrator.md \
    && grep -cE "^### 5\.[0-9]+ Retro-bypass" .claude/rules/advisor-orchestrator.md \
    && test -f .claude/lessons/feedback_pmd_canonical_guard_enforces_invariant.md \
    && test -f .claude/lessons/feedback_retro_harvest_weekly_cadence.md \
    && test -f .claude/PRPs/reports/v1-rls-r1-retro.md
  ```
- **Expected output:** all returns positive.
- **Brief-Scope outputs to verify:**
  - Dogfood report contains four H2 sections + all three sub-
    runs covered with verbatim outputs.
  - `advisor-orchestrator.md` §1 has the SessionStart-guard one-
    bullet addendum.
  - `advisor-orchestrator.md` §5 has the "Retro-bypass
    observability" sub-section with consumer + autonomy-signal
    language.
  - Two new paired lesson files exist with valid frontmatter +
    no auto-trigger keyword stuffing.
  - Retro file has three canonical H2 headers (What surprised
    us / What to change / What to carry forward).

> **Verification mapping:** `/brehon-verify v1-rls-r1` iterates
> these five stories, runs each checkpoint against the worktree
> branch, and confirms each Brief-Scope output. Phantoms (task
> complete but output absent or empty) trigger catch-fire per
> `.claude/rules/advisor-orchestrator.md` §5.5.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (14 probes confirmed)
- [ ] Tasks 1-13 committed in dependency order
- [ ] §15 validation green at every gate
- [ ] §15.7 + §15.8 cross-cutting / ADR-OQ boxes ticked
- [ ] §16a stories all `[done]`
- [ ] Dogfood report shows all three sub-runs pass (Task 10)
- [ ] Retro committed per `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md` (Task 13)
- [ ] PR opened by BM session against `governance-v0` with
  `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-rls-r1-verify.md` shows all 5 stories ✓
- [ ] Manual hand-off applied post-merge: settings.local.json
  wiring in canonical brehon-fork + this lane per DQ #301
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `pmd-canonical-guard.sh` false-positives on a legitimate non-canonical path (e.g. user's machine has a different canonical) | LOW | LOW | WARN-not-FAIL contract per R1 — user reads stderr, ignores or fixes. No session blocked. |
| `retro-check.sh` fail-open instrumentation regresses the existing fail-open behaviour | LOW | HIGH | R6 + Watchpoint #7 — ADDITIVE only; `git diff` review confirms no other lines changed; Task 7 VALIDATE block runs `bash -n` syntactic check. |
| JSONL writes race on concurrent fail-opens, corrupting the sidecar | LOW | LOW | `jq -nc` builds the JSONL atomically per the observation-capture.sh canonical pattern; `>>` append is line-buffered. Worst case: one corrupt line out of thousands. Non-fatal contract — bypass instrumentation must not itself fail. |
| Step 2c retro-harvest sweep misses the cycle-count proposal (dogfood 4.6 fails) | MED | MED | Task 10 dogfood explicitly verifies; failure files `kind: "blocker"` and the impl-task is re-dispatched after the Step 2c body is revised. |
| `.claude/harvest/` gitignored choice (PRECON-9 lean) turns out wrong (e.g. the user wants harvest tracked for cross-session continuity) | LOW | LOW | Decision is reversible — flip the `.gitignore` line + commit the existing harvest files in a follow-up `chore(rls):` commit. Filed as `kind: "log"` for retro harvest if it happens. |
| Track B Task 6's bootstrap-checklist lesson conflicts with a future v1-rls-r2 update to the same file | LOW | LOW | Lesson files are append-friendly; v1-rls-r2 adds new checklist steps without re-writing existing ones. |
| Track C Task 8's cross-link in 04 lands in a wrong paragraph (worker reads wrong location) | LOW | LOW | Task 8's IMPLEMENT block says "near the redaction-service-contract block (line 405)"; worker re-reads on conflict. CR review catches any wrong placement. |
| Task 6 ships a lesson the brief assumed already existed — surfaces as "scope drift" in retro | LOW | LOW | `kind: "log"` filed by planner pre-seed; documented in §12 NOT-building rationale + §3 problem statement; retro records the discrepancy. |
| Manual dual-wire in settings.local.json forgotten post-merge | MED | LOW | Task 6 lesson is the durable record; PR description + retro both name the post-merge hand-step; advisor-orchestrator §1 addendum (Task 11) re-surfaces the wiring on next session start (the WARN banner fires if wiring is absent). |
| Claude Code sensitive-file protection blocks Junior worker writes under `.claude/**` (recurred during this planning task — see DELIVERY NOTE) | HIGH | LOW | Per the delivery note at top of this file. Pattern is well-known (v1-fed-in-a precedent); advisor handles via `git mv PLAN_DELIVERABLE.md .claude/PRPs/plans/v1-rls-r1.plan.md`. Filed as v1-rls-r2 candidate. |

---

## 19. Notes

### 19.1 Planner-time `kind: "log"` pre-seeds

The planning Junior task SHOULD file the following `kind: "log"` DQ
entries directly into `resolved[]` (per `.claude/rules/decision-
queue.md` Recipe 2). HOWEVER — same blockage as the plan file
itself (Junior worker cannot write to `.claude/decision-queue.json`
— per the delivery note). The pre-seeds are described here so the
advisor can transcribe them to `.claude/decision-queue.json`
directly when moving this file. Next available id at HEAD = **303**
(max id across live + archives = 302).

**Pre-seed #1 — missing lesson files referenced by the brief.**

```json
{
  "id": 303,
  "from": "planner",
  "kind": "log",
  "timestamp": "2026-05-21T<TBD>Z",
  "question": "The v1-rls-r1 brief cross-references four lesson files that are absent at HEAD 6e16ee94f on this worktree: feedback_one_system_memory_in_repo.md, feedback_lesson_mirror_check.md, feedback_advisor_watchpoint_specificity.md, feedback_phase_lane_worktree_bootstrap_checklist.md. Task 6 ships the fourth; the first three are SKIPPED as cross-link targets (per §12 NOT-building rationale).",
  "context": "Planner Read the .claude/lessons/ directory on the junior/role-planning-v1-rls-r1-... worktree and confirmed absence. The brief may have been authored against governance-v0 (which carries them) OR they are forward-only references. Track D Task 9 + Task 12 lessons reference feedback_lesson_must_pair_with_structural_fix_when_fixable.md (verified present) instead. No blocker — proceeding.",
  "options": ["proceed", "blocker"],
  "answer": "proceed — Track D Task 9 + Task 12 cite the verified-present lesson; future v1-rls-r1 retro can re-litigate if the absent lessons become load-bearing.",
  "answered_by": "planner-self-resolved",
  "resolved_at": "2026-05-21T<TBD>Z"
}
```

**Pre-seed #2 — `.claude/harvest/` gitignored choice.**

```json
{
  "id": 304,
  "from": "planner",
  "kind": "log",
  "timestamp": "2026-05-21T<TBD>Z",
  "question": ".claude/harvest/ is gitignored (per PRECON-9 lean) vs tracked-with-prune-after-N-weeks?",
  "context": "Brief §2.3 ambiguity #2 — planner picks gitignored per the .claude/PRPs/audit-metrics/ analogue from brehon-conformance-audit.plan.md. Runtime journal output; summary lands in weekly-review report; harvest files are reproducible. If retro surfaces a need for tracked-with-prune (e.g. cross-session continuity), easy follow-up: flip the .gitignore line.",
  "options": ["gitignored", "tracked"],
  "answer": "gitignored — per .claude/PRPs/audit-metrics/ analogue. Reversible if needed.",
  "answered_by": "planner-self-resolved",
  "resolved_at": "2026-05-21T<TBD>Z"
}
```

**Pre-seed #3 — `pmd-invariants.md` auto-load scope.**

```json
{
  "id": 305,
  "from": "planner",
  "kind": "log",
  "timestamp": "2026-05-21T<TBD>Z",
  "question": ".claude/rules/pmd-invariants.md auto-loads at SessionStart (no `paths:` frontmatter scoping) vs scoped via paths frontmatter?",
  "context": "Brief §2.3 ambiguity #3 — planner picks auto-loading. Five invariants on a single page (~150 lines max) fits the MEMORY.md startup-budget calculus. Sibling rule files (decision-queue.md, advisor-orchestrator.md, multi-lane-worktree.md) all auto-load without paths scoping; pmd-invariants.md is in the same load category.",
  "options": ["auto-load", "paths-scoped"],
  "answer": "auto-load — sibling-pattern convention.",
  "answered_by": "planner-self-resolved",
  "resolved_at": "2026-05-21T<TBD>Z"
}
```

**Pre-seed #4 — Claude Code sensitive-file write-protection recurrence.**

```json
{
  "id": 306,
  "from": "planner",
  "kind": "log",
  "timestamp": "2026-05-21T<TBD>Z",
  "question": "Claude Code's built-in sensitive-file protection blocks Junior worker writes to .claude/**. Same pattern observed in v1-fed-in-a planning task; recurs in v1-rls-r1. Adding .claude/PRPs/plans/** + .claude/decision-queue.json to a Junior worker permission allowlist (or a harness-side configuration option) would close the gap. Candidate v1-rls-r2 item.",
  "context": "Planner attempted to Write .claude/PRPs/plans/v1-rls-r1.plan.md and was prompted by the harness; the same Write to PLAN_DELIVERABLE.md succeeded. The advisor handles via git mv post-finalize. The recurrence + the cost (the deliverable lands at a non-canonical path until advisor intervention) is itself a v1-rls-r2 candidate.",
  "options": ["v1-rls-r2-candidate", "no-action"],
  "answer": "v1-rls-r2-candidate — file as a deferred item in §12 of v1-rls-r2 brief when authored.",
  "answered_by": "planner-self-resolved",
  "resolved_at": "2026-05-21T<TBD>Z"
}
```

### 19.2 Open questions deferred to advisor / user

None. The brief's PRECON-1..PRECON-10 + DQ #296-#302 resolutions
closed every binding question. Planner-time choices are recorded
as `kind: "log"` per §19.1.

### 19.3 Complexity-score gate (per `.claude/agents/planning.md`)

- **Score: 6/10** (per §5.1).
- **Target model:** `sonnet-4-6` (default).
- **Threshold:** `> 8` (Sonnet target).
- **Gate fires:** NO. Score below threshold.
- **No split-DQ filed.**

### 19.4 Schema-changing-spec retrofit gate (§3.8)

This plan adds NEW artifact CLASSES (`.claude/PRPs/specs/`,
`.claude/hooks/pmd-canonical-guard.sh`, `.claude/harvest/`,
`.claude/governance-log/`, `.claude/rules/pmd-invariants.md`,
`docs/brehon-law-inspired-network/governance-log-kinds-jsonl.md`).
It also adds a new SECTION to `.claude/rules/advisor-orchestrator.md`
(§5.X) and a new STEP to `.claude/skills/weekly-review/SKILL.md`
(Step 2c).

Per `.claude/rules/advisor-orchestrator.md` §3.8 "purely additive
functionality is a skip condition" — this plan adds new commands /
new artifact classes; it does NOT change the SHAPE of an existing
artifact class (no new required section in an existing skill file;
no new required field in an existing JSON/YAML schema; no new
marker in an existing section). The Step 2c insertion in
weekly-review/SKILL.md is additive (new step between existing
steps; no existing-step shape changed). The §5.X insertion in
advisor-orchestrator.md is additive (new sub-section under §5).
The rule-file `pmd-invariants.md` is a new file, not a change to
an existing rule.

**Retrofit gate result: SKIP.** No `AskUserQuestion` needed.
Forward-only.

### 19.5 Mirror to canonical sibling plans

- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` — §15
  laptop-DoD shape mirror (per PRECON-7). v1-rls-r1 omits §15.4
  migration round-trip (no migrations) and §15.5 Phase 2 e2e
  (no Rust changes).
- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — §13
  task body shape mirror (SKILL.md authoring task, paired-lesson
  shape, dogfood task shape).

---

## 20. Confidence score

A 1-10 score per dimension:

- **Plan correctness:** 8/10 — the brief's PRECON-1..PRECON-10 are
  exhaustive; the planner-time choices are well-bounded.
- **Cargo budget:** 10/10 — zero new cargo budget; sanity-only.
- **Test coverage:** 7/10 — no Rust tests; dogfood (Task 10) is the
  integration test; story-grain checkpoints in §16a are explicit;
  the dogfood report is the verifiable artifact.
- **Risk:** 8/10 — risk table §18 covers known surfaces; the WARN-
  not-FAIL contract on hooks keeps every script's blast radius
  low; the additive-only discipline on `retro-check.sh` preserves
  the existing fail-open semantics.

Average: 8.25/10. Plan is shippable.

LESSON: the `.claude/**` Junior-worker-write-protection recurred during v1-rls-r1 planning (same as v1-fed-in-a). The deliverable lands at PLAN_DELIVERABLE.md and requires advisor-side `git mv` to reach `.claude/PRPs/plans/v1-rls-r1.plan.md`. Worth a focused harness-side fix in v1-rls-r2: extend the Junior worker permission allowlist to cover `.claude/PRPs/plans/**` + `.claude/decision-queue.json` so future planning tasks can land at the canonical path directly. See pre-seed #4 in §19.1.
