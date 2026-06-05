# Plan: <phase-slug> — <one-line title>

> Template for Brehon sub-phase plans. Authored by the `planning` subagent (see `.claude/agents/planning.md`). The template carries the canonical 20-section schema observed in shipped plans (`phase-v1-JM-a.plan.md`, `v1-jury-mechanics-c.plan.md`, etc.) plus four spec-kit-derived additions: **`[P]` parallel-task markers in §13**, **§16a Stories block** for story-grain checkpoints, **§5 complexity score + factor breakdown**, and **per-task `creates:`/`modifies:` YAML block** in §13 (machine-parseable file-set declarations consumed by cohort dispatch + `/brehon-verify`).
>
> Delete this leading note and the inline `<...>` placeholders before committing.

## Table of contents

(Optional. Many shipped plans use a TOC; v1-JM-a does not. Keep iff plan exceeds ~1500 lines.)

---

## 1. Summary

One paragraph: what this sub-phase produces, why now, and the headline acceptance condition. **Do not** describe the implementation — describe the deliverable.

## 2. Source

Cite each authoritative document the plan derives from, with sha/path:

- `.claude/PRPs/prds/<phase-family>.prd.md` §<N> @ `<sha>`
- `.claude/lessons/feedback_*.md` (list any lessons that materially shaped the plan; cite only those that bind decisions, not "for awareness")
- ADRs: ADR-NNN @ `<sha>` for each ADR cited
- Prior retros that flagged preflight guardrails (§7)

## 3. Problem statement

What is broken / missing / blocked before this sub-phase ships? Tie each problem to a §10 pattern or §13 task.

## 4. Solution statement

The architectural shape of the change. Diagram-grain — leave file:line detail to §10/§13. The reader should be able to predict §11 (files to change) from §4 alone.

## 5. Metadata

- **Phase:** `<phase-slug>` (e.g. `v1-JM-d`)
- **Branch:** `phase-<phase-slug>` (cut by BM-task before Task 1)
- **Target impl-task model:** `<sonnet-4-6 | minimax-m2.7 | haiku-4-5 | ...>` — names the specific model the impl-task subagent runs under for this sub-phase. Default `sonnet-4-6`. **Used by the complexity-gate threshold below** (non-Sonnet targets get tighter thresholds).
- **Estimated tasks:** N (pre-flight + impl + retro)
- **Estimated cargo budget:** `<X> GB peak` (sum across cohorts; check against `feedback_resource_budget_pre_queue.md`)
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md table)
- **Complexity score:** `<N>/10` — see breakdown below

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. The planner computes the score mechanically and writes it before commit. The threshold for filing a split-DQ depends on the **Target impl-task model** field above:

- **Sonnet target** (`sonnet-4-6`, `sonnet-4-6-1m`, `opus-4-7`): split-DQ if `score > 8`.
- **Non-Sonnet target** (anything not in the Sonnet set above — e.g. `minimax-m2.7`, `haiku-4-5`, `sonnet-3.x`): split-DQ if `score > 6`. **The "proceed-as-one with prior-Sonnet-phase precedent" override is forbidden** — Sonnet precedent does not transfer to a smaller or untested model. The advisor must answer split, or user-relay (no advisor-mode override-with-precedent).

If a split-DQ fires, the planner files a `pending` entry to the advisor with `from: "planner"`, `kind: "blocker"`, asking "complexity N exceeds threshold (target model: M) — split into `<slug>-1` + `<slug>-2`, or proceed?"

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | <count> | Excludes Task 0 (pre-flight) and the retro task |
| Migrations touched | +2 each | <count> | Round-trip risk + production rollback cost |
| Crates touched | +1 each | <count> | Read from §11 grouping |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | <count> | Per `feedback_junior_worker_e2e_edit_hang.md` (worker-hang risk on >8000 line files) |
| New ADR-affecting decisions | +2 each | <count> | Any decision that supersedes an entry in `docs/research/brehon-law-inspired-network/99-decisions-and-open-questions.md` |
| Cargo budget peak above 6 GB | +1 per GB | <count> | Pre-Shape-G plans only; Shape-G plans set this to 0 (cargo runs off-box) |
| **Total** | — | **<N>** | Threshold for split-DQ: `>8` (Sonnet) / `>6` (non-Sonnet) |

If the plan ships under Shape G (v1-JM-e onward, validation off-box), the cargo-budget factor is 0 and the e2e factor's threshold is unchanged (Edit-hang risk is a function of file size, not where validation runs).

### 5.2 Per-task complexity ceiling (non-Sonnet target only)

For non-Sonnet targets, **each individual §13 task** must satisfy:

- `count(union(creates, modifies)) ≤ 3` files, AND
- `count(distinct crates/<X>/ prefixes in union(creates, modifies)) ≤ 1`, AND
- `crates/lemmy_server/tests/e2e/*.rs` does not appear in `modifies:` (e2e edits go in their own dedicated task — never bundled with non-test logic).

A task that violates any of these is a "split-candidate" — the planner splits it into N sub-tasks where N = `ceil(file-count / 3)`, using `[P]` markers aggressively where YAML overlap is empty. The Sonnet ceiling is `≤ 4` files / `≤ 2` crates (the prior implicit norm) and is unchanged for Sonnet targets.

## 6. Relationship to other v<N>-<family> sub-phases

What does this sub-phase depend on? What follows it? Cite the v1 planning queue entry id if applicable.

## 7. Preflight guardrails inherited from prior phases

Bulleted list. Each item is a one-line rule + the retro/lesson that produced it. These are non-negotiable for this plan; the planner-side DoD smoke (§15) must respect them.

Examples:

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`)
- **R5:** Task 0 enumerates ALL probes explicitly; do NOT inherit implicitly (per JM-b retro-events Event 4)
- **R6:** all clippy invocations use `--no-deps` uniformly (per JM-b retro-events Event 3)

## 8. Flow design

Before/after diagrams. ASCII sequence or call-graph showing data flow at the granularity of "function → function". Cite each box back to a §13 task that creates/edits it.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit. Group by purpose:

- **Schema/type definitions** — `<file>:<line-range>` (verbatim doc-comments planned in §10)
- **Existing patterns** — `<file>:<line-range>` (the MIRROR refs §13 tasks point at)
- **Adjacent test fixtures** — `<file>:<line-range>` (so the impl agent doesn't re-invent seed helpers)
- **Lessons** — list each `feedback_*.md` that gates a §13 task

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: each entry **must** cite a specific table, file, or `schema.rs` line. Patterns named only by concept ("watch for trait drift") are rejected at advisor-side review.

### 10.1 <Pattern name>

**Mirror:** `<file>:<line-range>`

```rust
// Verbatim doc-comment + signature + key constants.
// Plan-time text is what the impl agent edits; stays in plan to absorb drift.
```

### 10.2 <Pattern name>

…

## 11. Files to change

Bulleted list, grouped by crate. Each entry: path + one-line purpose + which §13 task(s) write it. The planner-side `cargo metadata` check (per planning.md) verifies each path exists in the workspace before committing the plan.

- `crates/db_schema/src/source/governance/<file>.rs` — <purpose> (Task <N>, <M>)
- `crates/api/api/src/governance/<file>.rs` — <purpose> (Task <N>)
- `migrations/<id>__<name>/up.sql` + `down.sql` — <purpose> (Task <N>)
- `crates/lemmy_server/tests/e2e/<file>.rs` — <purpose> (Task <N>, <M>)

### Struct-field add: enumerate all callsites (mandatory)

Per `feedback_planner_enumerate_struct_callsites_for_addfield.md`. When any task in §13 adds a field to a **public** struct (or any struct exported beyond a single file), the planner MUST run `rg "<StructName>" crates/ tests/` at plan-authoring time and enumerate every constructor-site file (every `<Type> { ... }` literal) in §11 under the task that adds the field. Tag the section "Caller crates (compiles-only-after-Task-N)" so the dependency is visible. List each caller crate explicitly so §13 FILES YAML `modifies:` arrays carry the right set.

Skip enumeration ONLY when the type has `Default` impl AND every existing caller uses `<Type> { ..Default::default() }` shape (verify by reading each caller).

Retroactive cost of skipping: 1-N fix-impl cycles. Plan-time cost: ~30s of `rg`. Always do the enumeration upfront.

## 12. NOT building in <phase-slug>

Out-of-scope items, with rationale. Each entry pairs a "tempting addition" with a deferral-to-later-sub-phase pointer. Prevents scope creep mid-impl.

- **<thing>** — deferred to <phase-slug>; reason: <one-line>.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per `feedback_pr_per_phase.md`'s code-only-via-PR rule). Each task header carries a `[P]` marker iff its **IMPLEMENT** files share no path with any other `[P]`-marked task in the same cohort. Task 0 is **always** non-`[P]` (barrier for verification).

> **Cohort dispatch (advisor-side):** the advisor groups consecutive `[P]`-marked tasks into a "cohort" and queues them simultaneously to Junior, each on its own worktree (per `feedback_parallel_agents_one_worktree_per_agent.md`). Cohort dispatch is degraded to serial above the EliteDesk cargo budget (per `feedback_resource_budget_pre_queue.md`). Non-`[P]` tasks are barriers — they queue alone. See `.claude/rules/advisor-orchestrator.md` "Cohort dispatch" for the mechanical rules.
>
> **Shape G (Layer G2 push-and-exit, v1-JM-e onward):** plans authored under Shape G compose tasks as edit + commit + push (validation runs async on GH Actions); cargo invocations belong to the workflow YAML at `.github/workflows/cargo-validate-*.yml`, not to the §13 task body. impl-task subagents capture the workflow_run_id post-push, write a `kind: "validate-pending"` DQ entry, and exit. The cohort budget check (memory cap) is non-binding under Shape G since cargo runs off-box; the forbidden-window check is non-binding for impl-task throughput. Pre-existing plans (v1-JM-d and earlier) keep their inline cargo task body — forward-only-immune per `feedback_schema_changing_spec_retrofit_question.md`.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `<phase>`; confirm branch is `phase-<phase>`; confirm prior phase's deliverables are intact on the base; confirm pre-existing clippy baseline is clean.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — wrapper sanity (cargo-check honors -p)
…

# Probe N — concurrent-PR check (no other PR touches §11 files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | contains("<critical-file>")) | {number, title, headRefName}'
# EXPECT: empty output; if non-empty, STOP and reconcile (file ownership conflict)
```

**EXPECT block:**
- Probes 0..N exit 0
- Probe <neg> exits NON-ZERO (negative test confirms exit-code propagation)

**No commit at Task 0** — this is verification only.

### Task 1 [P]: <title>

**ACTION:** <one-sentence summary>.

**FILES (machine-parseable, used by `/brehon-verify` + cohort dispatch):**

```yaml
creates:
  - <new-file-1>
  - <new-file-2>
modifies:
  - <existing-file-1>   # what changes (one line)
  - <existing-file-2>   # what changes (one line)
requires:               # optional — declare cross-task validation dependencies
  - task: <N>           # this task's validation cannot pass without task <N>'s changes also being present
    reason: <one-line>  # what specifically is needed (e.g. "Task 1 creates source_event_type column referenced in this UPDATE")
```

> **Discipline:** the planner asserts that `union(creates, modifies)` exactly equals the set of files named in the **IMPLEMENT (file N of M)** lines below. Drift between the YAML and the IMPLEMENT lines is a planner-side miss — file a DQ pending entry asking the planner to fix before any impl runs. The cohort-dispatch logic in `.claude/rules/advisor-orchestrator.md` "Cohort dispatch" intersects `union(creates, modifies)` across `[P]`-cohort peers and refuses to dispatch a cohort with overlap. `/brehon-verify` consumes `creates:` for the phantom-presence check (each `creates:` entry must exist + non-empty on the worktree branch).
>
> **`requires:` discipline (new — per `feedback_cohort_validation_dependency_check.md` 2026-05-11):** `[P]` marks worktree-write disjointness; `requires:` marks validation-time dependence. The two are independent. A task may be `[P]` (no file overlap with cohort peers) AND have `requires:` entries (its workflow validation references symbols/columns/types created by a non-cohort-peer task). Examples: a backfill SQL migration `requires:` the column-add migration that created the column; a Rust enum with `ExistingTypePath = "crate::schema::sql_types::Foo"` `requires:` the schema.rs task that creates `sql_types::Foo`. Advisor cohort-dispatch refuses to start a cohort whose `[P]` member has unsatisfied `requires:`. Tasks with unsatisfied `requires:` either (a) wait for the required task's phase-branch merge, or (b) bundle with the required task into a single Junior dispatch. **Empty / missing `requires:` = no cross-task dependency** (back-compat: pre-2026-05-11 plans assumed all `[P]` tasks were validation-independent).

**IMPLEMENT (file 1 of N):** in `<file>`, `<edit description>`. Use the verbatim doc-comment from §10.X.

**MIRROR:** `<file>:<line-range>` (`<symbol>` declaration) for the shape.

**GOTCHA:** <non-obvious constraint that bit a prior phase>.

**VALIDATE (story-checkpoint feeds §16a):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/<phase>-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/<phase>-task1-check.log
# EXPECT: exit 0
```

### Task 2 [P]: <title>

(same shape — including the **FILES** YAML block; `[P]` because `union(creates, modifies)` from Task 2 shares zero paths with Task 1's `union(creates, modifies)`)

### Task 3: <title>

(non-`[P]` — Task 3's `modifies:` overlaps Task 2's `creates:` or `modifies:`; this task is a barrier. Include the **FILES** YAML block uniformly — non-`[P]` tasks declare file-sets too, so cross-cohort refusals also work mechanically.)

…

### Task N: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full`
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`
- **Test target compile:** `cargo test --no-run -p lemmy_server --test e2e` (R7 — per task that touches a struct or re-export)
- **e2e execution:** `cargo test --test e2e -p lemmy_server [test_name]` (Task <N>)
  - **GOLDEN_INVARIANT check (bm-merge gate):** always add `-- --no-fail-fast` so a timing flake does not short-circuit counting. Expected: `130 passed, 5 skipped`. Without `--no-fail-fast`, a single flake exits early and the reported count is artificially low.
- **Migration round-trip:** `bash scripts/brehon/migrate-roundtrip.sh <id>__<name>` (when migrations are touched)

---

## 15. Validation commands (DoD)

> **Planner-side discipline (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`):** every command in this section MUST be dry-run by the advisor against current HEAD before plan approval. Unexecutable commands (missing `--features full`, missing `--no-deps`, `-p <crate>` + `--features full` per `feedback_features_full_p_crate_incompatible.md`) are advisor-side rejection grounds.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/<phase>-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/<phase>-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (R7 — per task touching a struct or re-export)

…

### 15.4 e2e test execution (Task <N>)

…

### 15.5 Cross-cutting verification

Bulleted checklist of invariants the planner asserts hold at end-of-phase:

- [ ] Every new governance_log emission calls `governance_log::append(...)` (not direct INSERT)
- [ ] R1: every i32 ↔ i64 comparison uses `i64::from(...)`, never `as` cast
- [ ] R5: Task 0 enumerated all probes
- [ ] R6: all clippy invocations use `--no-deps` uniformly
- [ ] …

### 15.6 DoD per workflow (Shape G plans — v1-JM-e onward)

Plans authored under Shape G (Layer G2 push-and-exit, per
`.claude/PRPs/plans/v1-validate-agent.plan.md`) reference workflow
files by path + expected `conclusion` field rather than enumerating
cargo commands inline. The advisor's `/brehon-verify` cross-checks
the latest workflow run on the phase-branch SHA against the listed
conclusion before queueing `bm-merge`.

Shape:

- **DoD entry**: `<workflow-name>.yml` on `<phase-branch>` SHA `<sha>` → `conclusion: "success"`
- **Validation command**: `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT**: `{"conclusion": "success", "databaseId": <id>}`

Pre-existing JM-d/JM-c/JM-b/JM-a plans stay under prose §15
(forward-only per
`feedback_schema_changing_spec_retrofit_question.md`). The single
bounded retrofit at `.claude/PRPs/briefs/jm-d-impl-2.md` §5 switches
to push-and-exit for the parked JM-d Task 2 brief; all other
pre-Shape-G plans and briefs keep their inline cargo §15 entries.

---

## 16. Acceptance criteria

Roll-up of §15 + §16a story checkpoints. The planner asserts each box is ticked at end-of-phase. The advisor's `/brehon-verify` cross-checks each box against the worktree branch before queueing `bm-merge`.

- [ ] All N tasks completed in dependency order
- [ ] §15.1 (cargo check) exit 0 after every task
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after every task touching a struct or re-export
- [ ] §15.4 (e2e tests) — N new tests pass; pre-existing tests still pass
- [ ] §15.5 (cross-cutting verification) — all M boxes ticked
- [ ] §16a stories — all stories `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task N
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

---

## 16a. Stories (independently-testable behaviour units)

> **Why this section exists** (spec-kit pattern adoption): a Brehon plan can have 8+ tasks, but the advisor today only validates the phase as a whole. Story-grain decomposition catches "task 5 broke task 2's invariant" earlier and enables `/brehon-verify` to surface phantom completions before `bm-merge`.
>
> **A story is the smallest unit that produces an end-to-end testable behaviour.** Each story names its composing §13 tasks + a checkpoint command (typically the e2e probe nearest the behaviour). When the checkpoint is green and `/brehon-verify` confirms outputs match brief Scope + §11 entries, the story is `[done]`. The phase ships only when every story is `[done]`.
>
> Small phases (1–3 tasks) ship a **single story** whose checkpoint is the phase-as-a-whole — back-compatible with current plans.

### Story 1: <user-facing behaviour, one line>

- **Composing tasks:** Task 1, Task 2 (both `[P]` — share no IMPLEMENT files)
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server <test_fn_name>"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/db_schema/src/source/governance/<file>.rs` contains `<symbol>` declaration
  - `crates/api/api/src/governance/<file>.rs` re-exports `<symbol>`

### Story 2: <user-facing behaviour, one line>

- **Composing tasks:** Task 3 (barrier — not `[P]`)
- **Checkpoint command:** …
- **Expected output:** …
- **Brief-Scope outputs to verify:** …

### Story 3: <user-facing behaviour, one line>

(same shape; story-grain checkpoints scale with phase complexity)

> **Verification mapping:** the advisor's `/brehon-verify` step (per `.claude/commands/brehon-verify.md`) iterates this section, runs each Story's Checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent or empty) trigger the catch-fire procedure in advisor-orchestrator.md.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all probes confirmed)
- [ ] Task 1..N committed
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/<phase>-verify.md` shows all stories ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| <risk> | LOW/MED/HIGH | LOW/MED/HIGH | <mitigation citing §10 or §15> |

---

## 19. Notes

Free-form notes the planner wants to surface to the advisor: open questions filed as DQ pre-seeds, alternative approaches considered, anything that didn't fit the schema.

---

## 20. Confidence score

(Optional. Used by JM-a; later plans drop. Not load-bearing.)

A 1-10 score per dimension:
- **Plan correctness:** N/10 — <one-line>
- **Cargo budget:** N/10 — <one-line>
- **Test coverage:** N/10 — <one-line>
