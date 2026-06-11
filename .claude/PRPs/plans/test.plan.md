# Plan: test — dogfood sandbox: pure helper through the full `/auto-phase` pipeline

## 1. Summary

This sub-phase produces a single trivial, pure, side-effect-free helper
function — `sandbox_clamp(value: u32, max: u32) -> u32` — in a new
`crates/utils/src/sandbox.rs` module, plus a `#[cfg(test)] mod tests` unit
test. It exists **only to dogfood the `/auto-phase` orchestration
end-to-end** (planning → bm-cut → impl-task → Shape-G workspace validation
→ bm-pr → CodeRabbit → bm-merge). The headline acceptance condition is that
`cargo-validate-workspace.yml` reports `conclusion: "success"` on the impl
task's pushed SHA and the resulting PR merges cleanly into `governance-v0`.
There is no feature value in the code itself; the value is exercising the
machinery on the smallest possible real Rust change.

## 2. Source

- **Intent source (brief absent):** `.claude/PRPs/briefs/test-bm-cut-1.md`
  @ `3a3a9fa5f`. The dispatch named `.claude/PRPs/briefs/test-planning-1.md`,
  which **does not exist on this branch**. Intent was reconstructed from the
  committed sibling `test-bm-cut-1.md`, whose §2 states: *"this is a sandbox
  dogfood phase… The plan will be authored by the planning Junior AFTER the
  phase branch is cut."* Recorded transparently as a `kind: "log"` DQ entry
  (`from: "planner"`, `answered_by: "planner"`) so the advisor sees the brief
  was absent and the plan reconstructed scope from the sibling.
- **Structural mirror (Shape-G plan shape):**
  `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — §13 task shape (push-and-
  exit, no inline cargo), §15 per-workflow DoD, §16a stories. Cited per
  `.claude/lessons/feedback_read_canonical_before_writing_spec.md` (read 1-2
  siblings + cite before authoring shape-prescribing sections).
- **Template:** `.claude/PRPs/templates/plan.template.md` (20-section schema,
  `[P]` markers, FILES YAML, §5 complexity, §16a stories).
- **Lessons that bind decisions here:**
  - `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — source-code extension: read sibling files in the same module dir before authoring.
  - Workspace clippy config `Cargo.toml:85-130` — `tests_outside_test_module = "deny"`, `as_conversions = "deny"`, `unwrap_used = "deny"`, `indexing_slicing = "deny"` shape the §10 helper to be clippy-clean.

## 3. Problem statement

The four-role `/auto-phase` orchestration (advisor → planning/impl-task/
bm-task/ci-watcher Junior subagents) has many moving parts: brief authoring,
clarify gate, bm-cut, cohort dispatch, Shape-G `validate-pending` DQ flow,
ci-watcher mutation, bm-pr, CodeRabbit triage, bm-merge, retro. Exercising
the **whole** chain against a real Rust change — without risking governance
logic, ADR-pinned paths, migrations, or e2e edits — has no dedicated
low-risk fixture. This sub-phase is that fixture: it drives one impl task
through the complete pipeline so the operators can observe each stage on a
change that cannot break anything load-bearing.

## 4. Solution statement

Add one new module file under the existing `lemmy_utils` crate containing a
pure clamp helper, and declare it unconditionally in `lib.rs` (alongside
`pub mod error;`, NOT inside the `cfg_select! { feature = "full" => … }`
block — `lemmy_utils` is consumed with `default-features = false` per
`Cargo.toml:142`, so the module must compile without the `full` feature).
The helper has zero dependencies, zero imports, and a single expression body
(`value.min(max)`), so it compiles identically on every target and trips no
clippy lint. A `#[cfg(test)] mod tests` block holds two `assert_eq!` unit
tests (required by `tests_outside_test_module = "deny"`). From §4 the reader
can predict §11 exactly: one new file + one one-line edit to `lib.rs`, both
in `crates/utils/`.

## 5. Metadata

- **Phase:** `test`
- **Branch:** `phase-test` (cut by `test-bm-cut-1` BM task @ `75e38ded1`)
- **Target impl-task model:** `sonnet-4-6` (default; no model trial)
- **Estimated tasks:** 3 (Task 0 pre-flight + Task 1 impl + Task 2 retro)
- **Estimated cargo budget:** 0 GB peak (Shape G — cargo runs on GH-hosted runners)
- **Forbidden-window applicability:** non-binding (Shape G — cargo off-box)
- **Complexity score:** `1/10` — see breakdown below

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target model `sonnet-4-6` →
split-DQ threshold is `score > 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | Only 1 impl task (Task 1) |
| Migrations touched | +2 each | 0 | None |
| Crates touched | +1 each | 1 | `crates/utils/` only |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | None — deliberately avoided |
| New ADR-affecting decisions | +2 each | 0 | None — no governance path touched |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Shape G — off-box |
| **Total** | — | **1** | Threshold for split-DQ: `>8` (Sonnet). **No split-DQ.** |

Score `1 ≤ 8` → no split-DQ filed.

### 5.2 Per-task complexity ceiling

N/A for Sonnet target (Sonnet ceiling `≤ 4` files / `≤ 2` crates). Task 1
touches 2 files in 1 crate — well within ceiling.

## 6. Relationship to other sub-phases

Standalone dogfood sandbox. Depends on nothing; nothing depends on it. The
`phase-test` branch is a throwaway sandbox — after merge (or after the
dogfood run completes), the branch and the `sandbox.rs` module may be
reverted/removed without consequence. Not part of any v1/M1/M2 roadmap lane.

## 7. Preflight guardrails inherited from prior phases

- **R-clippy-1:** no `as` casts anywhere (`as_conversions = "deny"`,
  `Cargo.toml:109`). The helper uses no casts — `value.min(max)` returns
  `u32` directly.
- **R-clippy-2:** unit tests MUST live inside a `#[cfg(test)] mod tests`
  block (`tests_outside_test_module = "deny"`, `Cargo.toml:112`). Free
  `#[test]` fns at module scope are a hard clippy failure.
- **R-clippy-3:** no `.unwrap()` / `.expect()` (`unwrap_used`/`expect_used`
  = "deny", `Cargo.toml:104`+`:108`). The tests use `assert_eq!`, not unwrap.
- **R-mod-decl:** the new module is declared **unconditionally** in `lib.rs`
  (outside the `cfg_select!` `feature = "full"` arm) — `lemmy_utils` builds
  with `default-features = false` (`Cargo.toml:142`), so a `full`-gated
  module would be absent in dependents and fail `cargo check --workspace`
  without `--features full`.
- **R-sibling-read:** per `feedback_read_canonical_before_writing_spec.md`
  source-code extension, the impl agent reads `crates/utils/src/error.rs`
  (the existing unconditional sibling module) before authoring `sandbox.rs`,
  to match crate-local import/doc/module conventions.

## 8. Flow design

```
BEFORE:
  crates/utils/src/lib.rs
    cfg_select! { feature="full" => { cache_header, rate_limit, response, settings, utils } }
    pub mod error;                 ← unconditional modules live here
    (no sandbox module)

AFTER:
  crates/utils/src/lib.rs
    cfg_select! { … }              ← unchanged
    pub mod error;
    pub mod sandbox;               ← Task 1 adds this one line (unconditional)

  crates/utils/src/sandbox.rs      ← Task 1 creates this file
    pub fn sandbox_clamp(value, max) -> u32 { value.min(max) }   (§10.1)
    #[cfg(test)] mod tests { clamps_above_max(); passes_through_below_max(); }
```

Single function, single call site (the unit tests). No cross-crate data
flow. `cargo check --workspace` reaches `sandbox.rs` via the `lib.rs` module
declaration (Task 1, file 2 of 2).

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Module-declaration pattern** — `crates/utils/src/lib.rs:1-15` (the
  `cfg_select!` block + the unconditional `pub mod error;` line that
  `pub mod sandbox;` mirrors).
- **Sibling unconditional module (convention reference)** —
  `crates/utils/src/error.rs:1-30` (import style, doc-comment style, module
  layout the new file should match).
- **Workspace clippy lints** — `Cargo.toml:85-130` (the deny list that the
  helper + tests must satisfy; specifically `tests_outside_test_module`,
  `as_conversions`, `unwrap_used`, `indexing_slicing`).
- **Lessons** — `feedback_read_canonical_before_writing_spec.md` (read
  sibling first); `feedback_clippy_test_style.md` (test-style lint shape).

## 10. Patterns to mirror

### 10.1 Pure clamp helper (the deliverable)

**Mirror:** `crates/utils/src/lib.rs:38-44` (`pub const` + simple-body
declarations like `DB_BATCH_SIZE`; the new fn mirrors that file's
plain-declaration, no-import style) and `crates/utils/src/error.rs:1-30`
(unconditional-module file layout).

```rust
//! Sandbox helpers used to dogfood the `/auto-phase` orchestration pipeline.
//! Not part of any governance feature — safe to revert.

/// Clamp `value` to at most `max`, returning the smaller of the two.
///
/// Pure, dependency-free, side-effect-free. Exists only to exercise the
/// full plan → impl → validate → PR → merge pipeline on a minimal change.
pub fn sandbox_clamp(value: u32, max: u32) -> u32 {
  value.min(max)
}

#[cfg(test)]
mod tests {
  use super::sandbox_clamp;

  #[test]
  fn clamps_above_max() {
    assert_eq!(sandbox_clamp(10, 5), 5);
  }

  #[test]
  fn passes_through_below_max() {
    assert_eq!(sandbox_clamp(3, 5), 3);
  }
}
```

GOTCHA: `value.min(max)` (not `if value > max { max } else { value }`) — the
method form is idiomatic and avoids any `style`/`complexity` clippy lint.
No `as` cast, no `.unwrap()`, no indexing — all clean against `Cargo.toml`
deny list.

### 10.2 Unconditional module declaration

**Mirror:** `crates/utils/src/lib.rs:15` (`pub mod error;`).

```rust
pub mod error;
pub mod sandbox;   // ← add immediately after `pub mod error;`
```

GOTCHA: place OUTSIDE the `cfg_select! { feature = "full" => { … } }` block.
`lemmy_utils` is a `default-features = false` workspace dependency
(`Cargo.toml:142`); a `full`-gated module would not compile under a plain
`cargo check --workspace`.

## 11. Files to change

`crates/utils/` (crate `lemmy_utils`):

- `crates/utils/src/sandbox.rs` — NEW. The `sandbox_clamp` pure helper +
  `#[cfg(test)] mod tests` unit tests (Task 1).
- `crates/utils/src/lib.rs` — MODIFIED. Add one line `pub mod sandbox;`
  immediately after `pub mod error;` (Task 1).

No struct-field adds → the "enumerate all callsites" sub-section is N/A.

## 12. NOT building in test

- **Any governance/ADR-touching code** — deferred indefinitely; reason: the
  entire point is a zero-risk dogfood, so no `crates/api/api/src/governance/`
  or `crates/apub/activities/src/governance/` path is touched.
- **Migrations** — none; reason: schema changes carry round-trip + rollback
  cost incompatible with "throwaway sandbox."
- **e2e tests** (`crates/server/tests/e2e.rs`) — none; reason: e2e edits are
  the highest-risk impl class (`feedback_fix_impl_pre_locate_e2e_anchors.md`)
  and would defeat the "smallest possible change" goal. A `lemmy_utils` unit
  test is sufficient to demonstrate the test-compile path.
- **Wiring the helper into any real call site** — none; reason: the helper
  is intentionally dead code (exercised only by its unit tests). Wiring it in
  would create a real dependency that complicates revert.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Only 1 impl task → no
`[P]` cohort (single-task phases omit `[P]` markers; serial dispatch).

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline cargo
> invocations. Task 1 ends with a push to the worker branch; the impl-task
> subagent writes a `kind: "validate-pending"` DQ entry referencing
> `cargo-validate-workspace.yml` per `.claude/rules/decision-queue.md`.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment + branch (`phase-test`) + base state intact;
confirm `crates/utils/src/sandbox.rs` does not already exist.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate all probes):**

```bash
# Probe -1 — submodule init (Linux/Junior worktree quirk; lemmy_email build.rs)
git submodule status > /tmp/test-task0-submodule.log 2>&1
if grep -q '^-' /tmp/test-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/test-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-test (or the junior/<task-slug> worktree branch forked from it)

# Probe 2 — target module does NOT already exist (idempotency / no clobber)
test ! -f crates/utils/src/sandbox.rs && echo "SANDBOX ABSENT OK" || { echo "sandbox.rs already exists — STOP"; exit 1; }

# Probe 3 — declaration anchor present (the line Task 1 inserts after)
rg -n '^pub mod error;' crates/utils/src/lib.rs
# EXPECT: exactly one match (the insertion anchor for `pub mod sandbox;`)

# Probe 4 — anchor uniqueness (Edit target must be unique)
test "$(rg -c '^pub mod error;' crates/utils/src/lib.rs)" = "1" && echo "ANCHOR UNIQUE OK" || { echo "anchor not unique — STOP"; exit 1; }

# Probe 5 — workspace clippy deny list present (the lints the helper satisfies)
rg -n 'tests_outside_test_module|as_conversions|unwrap_used' Cargo.toml | head
# EXPECT: three deny lines (informational — confirms §7 guardrails apply)

# Probe 6 — Shape G workflow YAML present + lint-clean
test -f .github/workflows/cargo-validate-workspace.yml && echo "WORKFLOW PRESENT OK" || { echo "workspace-check workflow missing — STOP"; exit 1; }

# Probe 7 — concurrent-PR check (no other open PR touches utils/lib.rs or sandbox.rs)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/utils/src/lib\\.rs|crates/utils/src/sandbox\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output
```

**EXPECT:** Probes 1..7 exit 0 (Probe 2/4/6 exit 1 with explicit STOP on failure).

**No commit at Task 0** — verification only.

### Task 1: Sandbox clamp helper + unconditional module declaration

**ACTION:** create `crates/utils/src/sandbox.rs` with the `sandbox_clamp`
pure helper + `#[cfg(test)] mod tests`; declare `pub mod sandbox;` in
`lib.rs` immediately after `pub mod error;`.

**FILES:**

```yaml
creates:
  - crates/utils/src/sandbox.rs   # pure clamp helper + unit tests
modifies:
  - crates/utils/src/lib.rs       # add `pub mod sandbox;` after `pub mod error;`
```

**IMPLEMENT (file 1 of 2):** create `crates/utils/src/sandbox.rs` with the
**verbatim** content from §10.1 (module doc-comment, `sandbox_clamp` fn,
`#[cfg(test)] mod tests` with both `assert_eq!` tests).

**IMPLEMENT (file 2 of 2):** in `crates/utils/src/lib.rs`, add the single
line `pub mod sandbox;` immediately after the existing `pub mod error;` line
(per §10.2). Do NOT place it inside the `cfg_select!` `feature = "full"` arm.

**MIRROR:** §10.1 (helper + tests), §10.2 (module declaration).

**GOTCHA (R-clippy-1):** no `as` casts — use `value.min(max)`.

**GOTCHA (R-clippy-2):** the two `#[test]` fns MUST be inside the
`#[cfg(test)] mod tests` block, never at module scope
(`tests_outside_test_module = "deny"`).

**GOTCHA (R-mod-decl):** `pub mod sandbox;` is unconditional — outside the
`cfg_select!` block — so the module compiles under `default-features = false`
dependents.

**GOTCHA (R-sibling-read):** read `crates/utils/src/error.rs:1-30` first to
match the crate's import/doc/module conventions before writing `sandbox.rs`.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`; the
impl-task subagent writes a `kind: "validate-pending"` DQ entry capturing the
`workflow_run_id` of `cargo-validate-workspace.yml` (fields per
`.claude/rules/decision-queue.md`: `workflow_run_id`, `branch`, `phase_task`,
`result: null`, `log_slice: null`, `failed_jobs: null`). No local cargo
invocation.

**COMMIT MESSAGE:** `feat(test): sandbox_clamp dogfood helper in lemmy_utils (task 1)`

### Task 2: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and
`feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning /
Impl / BM) capturing what the dogfood run surfaced about the `/auto-phase`
pipeline — specifically the **missing-brief** handling (the planning step
proceeded by reconstructing intent from the committed `test-bm-cut-1.md`
sibling rather than blocking; recorded as the §2 `kind: "log"` DQ entry).
Promote any durable lesson (e.g. "planning brief absent → reconstruct from
committed sibling vs hard-block, when phase is a declared sandbox") to
`.claude/lessons/feedback_*.md` in the same retro commit
(`feedback_one_system_memory_in_repo.md`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/test-retro.md
modifies: []
```

**IMPLEMENT (file 1 of 1):** author `.claude/PRPs/reports/test-retro.md`
with the four role H2s + per-task complexity score
(`feedback_retro_task_complexity_score.md`: `<files>/<commits>/<runtime-min>/
<max-log-silence-min>`).

**COMMIT MESSAGE:** `docs(retro): test dogfood phase retro`

---

## 14. Testing strategy

- **Unit (compile-time):** `cargo check --workspace --features full` (runs
  in `cargo-validate-workspace.yml` — reaches `sandbox.rs` via the `lib.rs`
  module declaration).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`
  (same workflow; confirms the helper + tests satisfy the `Cargo.toml` deny
  list).
- **Unit-test compile:** the `#[cfg(test)] mod tests` block compiles under
  the workspace check; full execution of the two `assert_eq!` tests is
  optional (advisor-side `cargo test -p lemmy_utils sandbox` smoke only — the
  dogfood's binding signal is check + clippy green, not test execution).
- **e2e:** N/A — this phase ships no e2e edits (§12).
- **Migration round-trip:** N/A — no migrations.

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6.

### 15.1 Per-task workspace check (Shape G)

For Task 1:

- **DoD entry:** `cargo-validate-workspace.yml` on `junior/<task-slug>` SHA
  `<sha>` → `conclusion: "success"`
- **Validation command:**
  `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`,
`cargo clippy --workspace --features full --no-deps -- -D warnings`, and
`cargo test --no-run -p lemmy_server --test e2e`. The check + clippy steps
are workspace-scoped, so they reach `crates/utils/src/sandbox.rs`.

### 15.2 Phase 2 e2e — N/A

This phase ships no e2e edits and wires the helper into no runtime path. The
optional `cargo test -p lemmy_utils sandbox` is advisor-side smoke only (not
a §16 acceptance gate).

### 15.3 Cross-cutting verification (Task 2 retro time)

- [ ] `test -f crates/utils/src/sandbox.rs` → exit 0 (file present, non-empty)
- [ ] `rg -c 'pub fn sandbox_clamp' crates/utils/src/sandbox.rs` returns `1`
- [ ] `rg -c '^pub mod sandbox;' crates/utils/src/lib.rs` returns `1`
- [ ] `rg -n 'pub mod sandbox;' crates/utils/src/lib.rs` is OUTSIDE the `cfg_select!` block (declared alongside `pub mod error;`)
- [ ] `rg -c '#\[cfg\(test\)\]' crates/utils/src/sandbox.rs` returns `1` (tests inside `mod tests`)
- [ ] No `as` cast in `sandbox.rs` (`rg -c ' as ' crates/utils/src/sandbox.rs` returns `0`)
- [ ] No edits to files outside §11 list

### 15.4 Migration round-trip — N/A

No migrations.

### 15.5 ADR / OQ compliance verification

- [ ] No `crates/api/api/src/governance/` or `crates/apub/.../governance/` path touched (no ADR gate involved) — `git diff --name-only governance-v0..phase-test` shows only `crates/utils/src/{lib,sandbox}.rs` + `.claude/PRPs/{plans,reports}/test*`

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch: Task 1's `junior/<task-slug>` (and `phase-test` after finalize-merge)
- Expected `conclusion`: `"success"`

**Phase 2 (e2e):** N/A (no e2e edits).

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p <crate>` +
> `--features full`; use `--workspace --features full`.

```bash
yamllint .github/workflows/cargo-validate-workspace.yml
echo "yamllint exit: $?"

cargo check --workspace --features full
echo "exit: $?"
```

Advisor-side only; not §16 acceptance criteria.

---

## 16. Acceptance criteria

- [ ] All 3 tasks (Task 0 + Task 1 + Task 2 retro) committed
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"` on Task 1's pushed SHA
- [ ] §15.3 (cross-cutting verification — 7 boxes) all ticked
- [ ] §15.5 (ADR / OQ compliance — 1 box) ticked
- [ ] §16a story `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per Task 2
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/test-verify.md` shows the story ✓

---

## 16a. Stories (independently-testable behaviour units)

Per `feedback_story_grain_checkpoint.md` + `feedback_brehon_verify_pre_merge.md`.
This is a 1-impl-task phase → a **single story** whose checkpoint is the
phase-as-a-whole (back-compatible with current plans).

### Story 1: `sandbox_clamp` compiles workspace-clean and is reachable from `lemmy_utils`

- **Composing tasks:** Task 1
- **Checkpoint workflow:** `cargo-validate-workspace.yml` on Task 1's worker
  branch → `conclusion: "success"` (the workspace `cargo check` +
  `cargo clippy --no-deps -- -D warnings` steps cover `sandbox.rs`)
- **Expected output:** `{"conclusion": "success", "databaseId": <id>}` from
  the §15.1 validation command
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/utils/src/sandbox.rs` contains `pub fn sandbox_clamp(`
  - `crates/utils/src/sandbox.rs` contains a `#[cfg(test)]` `mod tests` block with `clamps_above_max` and `passes_through_below_max`
  - `crates/utils/src/lib.rs` contains `pub mod sandbox;` declared OUTSIDE the `cfg_select!` block (alongside `pub mod error;`)

> **Verification mapping:** the advisor's `/brehon-verify` iterates this
> section, runs the checkpoint against the worktree branch, and confirms each
> Brief-Scope output exists + matches its structural pattern. A phantom
> (task complete but `sandbox.rs` absent or empty) triggers catch-fire.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all probes confirmed)
- [ ] Task 1 committed (`feat(test): sandbox_clamp …`)
- [ ] §15 validation green (workspace-check workflow `success`)
- [ ] §16a story `[done]`
- [ ] Task 2 retro committed
- [ ] PR opened by BM session against `governance-v0`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/test-verify.md` shows the story ✓
- [ ] Post-merge `phase-test` branch retained for retro reads (or reverted per sandbox disposability)

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Module declared inside `cfg_select!` `full` arm → fails `cargo check --workspace` without `--features full` | LOW | MED | §7 R-mod-decl + §10.2 GOTCHA + Task 0 Probe 3/4 anchor on `pub mod error;` (unconditional line) |
| Unit tests at module scope → `tests_outside_test_module` clippy deny | LOW | MED | §7 R-clippy-2 + §10.1 verbatim `#[cfg(test)] mod tests` block |
| Brief absence misread as "no work to do" | LOW | LOW | §2 records intent reconstruction + `kind: "log"` DQ entry; this plan IS the deliverable |
| Dogfood mistaken for a real feature and wired into runtime | LOW | MED | §12 explicitly forbids wiring the helper into any call site (dead code by design) |

---

## 19. Notes

- **Missing brief.** The dispatch named `.claude/PRPs/briefs/test-planning-1.md`,
  which is absent on this branch. Rather than hard-block (the literal
  brief-missing fallback), the planner reconstructed scope from the committed
  sibling `test-bm-cut-1.md` (@ `3a3a9fa5f`), whose §2 explicitly states the
  plan is to be authored by the planning Junior post-cut for a "test dogfood
  sandbox." This judgment is recorded transparently as a `kind: "log"` DQ
  entry (`from: "planner"`, `answered_by: "planner"`) so the advisor can see
  the brief was absent and decide whether to author `test-planning-1.md`
  retroactively for the audit trail. For a non-sandbox phase, a missing brief
  would have been a `kind: "blocker"` instead — the sandbox designation in
  the sibling is what made reconstruction the right call.
- **Disposability.** `phase-test` and `crates/utils/src/sandbox.rs` are
  throwaway. After the dogfood run, the module can be reverted with no
  downstream impact (it is dead code with no call sites — §12).
- **No DQ pre-seeds beyond the missing-brief log entry.** No forward-looking
  OQs; the scope is fully determined.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — single trivial pure fn, mirror verbatim in §10, clippy deny-list checked against `Cargo.toml:85-130`.
- **Cargo budget:** 10/10 — Shape G, off-box, 0 GB local peak.
- **Test coverage:** 7/10 — two unit tests exercise the helper; intentionally no e2e (sandbox scope). Adequate for a dogfood.
