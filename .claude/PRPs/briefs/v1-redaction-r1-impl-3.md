# Brief — v1-redaction-r1 impl-task 3

`[role:impl-task] v1-redaction-r1 task 3 — regex commentary + Maintenance invariants — see .claude/PRPs/briefs/v1-redaction-r1-impl-3.md`

## 1. Role + dispatch line

Junior `impl-task` subagent (Sonnet 4.6) executes Plan §13 Task 3 (commentary-only) on `crates/db_schema/src/source/governance/redaction.rs`. Sub-phase `v1-redaction-r1`; phase branch `phase-v1-redaction-r1`.

## 2. Scope

**Comment-only edit. Zero behaviour change. Zero new tests. Zero file moves.** All edits are within the single file `crates/db_schema/src/source/governance/redaction.rs`. Tasks 1+2 have already shipped on this branch (commits `e42cb4d70` + `471ba7565`); Task 3 lands strictly after them.

Three insertions only:

- **(A)** Add `## Maintenance invariants` H2 subsection to the existing module-level doc-comment (between current line 25 `unchanged.` and line 27 `use regex::Regex;`).
- **(B)** Insert one-line policy comment above `email_regex` (between line 44 blank and line 45 `fn email_regex()`).
- **(C)** Append one new line to the existing `mention_regex` comment block (between line 37 `boundary char and re-emit it in the replacement.` and line 38 `#[expect(clippy::expect_used, ...)]`).

### 2.1 Verbatim §10.4 Maintenance-invariants block (production code)

This is the EXACT text to insert at Edit (A). Copy character-for-character; do NOT paraphrase or reformat.

```rust
//! ## Maintenance invariants
//!
//! 1. **Order of operations in [`scrub`]** is load-bearing —
//!    profile URLs first, then fediverse mentions, then emails. See
//!    the `scrub` fn doc-comment below for the footgun this prevents.
//!    Locked by `scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`
//!    in the test module.
//!
//! 2. **[`scrub_json`] recursion is bounded at
//!    [`MAX_RECURSION_DEPTH`] (= 64)** — defence-in-depth against
//!    adversarial / buggy upstream JSON trees. At the cap, the
//!    substitution is [`serde_json::Value::Null`] (NOT a truncated
//!    subtree). Over-scrub bias per GDPR §17 + ADR-015 — a leak
//!    defeats right-to-delete permanently; an over-scrubbed leaf is
//!    cosmetic loss.
//!
//! 3. **[`email_regex`] is permissive on purpose** — it accepts a
//!    superset of RFC-5322. False positives (e.g.
//!    `version-1.0@build-2026` scrubbed as email) are acceptable;
//!    false negatives are GDPR violations. Do NOT tighten without a
//!    new ADR amending ADR-015's over-scrub bias.
//!
//! 4. **Unicode confusables** (zero-width joiner, right-to-left
//!    override, Cyrillic lookalike in username) are partially covered
//!    by the ASCII-boundary-class behaviour but two known-deferred
//!    cases are tracked as `#[ignore]` tests in this module. Their
//!    resolution is owned by v1-redaction-r2 (not yet on roadmap).
```

### 2.2 Three exact Edits

#### Edit A — Module-doc Maintenance invariants

**Anchor:** current lines 25-27 in `crates/db_schema/src/source/governance/redaction.rs` (verified at brief-author time @ phase tip `bb9fe84a7`).

`old_string`:
```
//! call sites in `submit_jury_vote.rs` continue to compile unchanged.

use regex::Regex;
```

`new_string`:
```
//! call sites in `submit_jury_vote.rs` continue to compile unchanged.
//!
//! ## Maintenance invariants
//!
//! 1. **Order of operations in [`scrub`]** is load-bearing —
//!    profile URLs first, then fediverse mentions, then emails. See
//!    the `scrub` fn doc-comment below for the footgun this prevents.
//!    Locked by `scrub_order_dependence_mention_with_remote_host_not_eaten_by_email`
//!    in the test module.
//!
//! 2. **[`scrub_json`] recursion is bounded at
//!    [`MAX_RECURSION_DEPTH`] (= 64)** — defence-in-depth against
//!    adversarial / buggy upstream JSON trees. At the cap, the
//!    substitution is [`serde_json::Value::Null`] (NOT a truncated
//!    subtree). Over-scrub bias per GDPR §17 + ADR-015 — a leak
//!    defeats right-to-delete permanently; an over-scrubbed leaf is
//!    cosmetic loss.
//!
//! 3. **[`email_regex`] is permissive on purpose** — it accepts a
//!    superset of RFC-5322. False positives (e.g.
//!    `version-1.0@build-2026` scrubbed as email) are acceptable;
//!    false negatives are GDPR violations. Do NOT tighten without a
//!    new ADR amending ADR-015's over-scrub bias.
//!
//! 4. **Unicode confusables** (zero-width joiner, right-to-left
//!    override, Cyrillic lookalike in username) are partially covered
//!    by the ASCII-boundary-class behaviour but two known-deferred
//!    cases are tracked as `#[ignore]` tests in this module. Their
//!    resolution is owned by v1-redaction-r2 (not yet on roadmap).

use regex::Regex;
```

#### Edit B — email_regex policy comment

**Anchor:** current lines 43-45 (blank line 44 followed by `fn email_regex()` on line 45).

`old_string`:
```
}

fn email_regex() -> &'static Regex {
```

`new_string`:
```
}

// Permissive on purpose — see Maintenance invariants #3 in the
// module doc above. Over-scrub bias per GDPR §17 + ADR-015.
fn email_regex() -> &'static Regex {
```

#### Edit C — mention_regex policy citation

**Anchor:** end of existing `mention_regex` comment block (line 37 trailing `re-emit it in the replacement.`), before the `#[expect(...)]` attribute on line 38. The new line goes INSIDE the comment block, immediately before the attribute.

`old_string`:
```
  // boundary char and re-emit it in the replacement.
  #[expect(clippy::expect_used, reason = "static regex — infallible at startup")]
```

`new_string`:
```
  // boundary char and re-emit it in the replacement.
  // ASCII boundary class — Unicode-permissive by side-effect; see
  // Maintenance invariants #4 in the module doc above.
  #[expect(clippy::expect_used, reason = "static regex — infallible at startup")]
```

### 2.3 Public-API non-regression (verify with rg after edits)

After all three Edits, run these locally to verify:

```bash
rg -n 'pub fn scrub_json' crates/db_schema/src/source/governance/redaction.rs  # EXPECT 1 hit
rg -n 'pub fn scrub\b' crates/db_schema/src/source/governance/redaction.rs     # EXPECT 1 hit
rg -n 'const MAX_RECURSION_DEPTH' crates/db_schema/src/source/governance/redaction.rs  # EXPECT 1 hit (line 69)
rg -n 'fn scrub_json_inner' crates/db_schema/src/source/governance/redaction.rs        # EXPECT 1 hit (line 105)
rg -n '^## Maintenance invariants' crates/db_schema/src/source/governance/redaction.rs # EXPECT 1 hit (new H2 inside //! doc)
rg -n '^//! ## Maintenance invariants' crates/db_schema/src/source/governance/redaction.rs # EXPECT 1 hit
```

## 3. Required reading (read BEFORE making any edit)

### 3.a Handover from prior cohort (Task 2, commit `471ba7565` + `d396fb7a9`)

```yaml
prior_cohort_tasks:
  - task: 2
    commit: 471ba7565
    filesCreated: []
    filesModified:
      - crates/db_schema/src/source/governance/redaction.rs
    keyDecisions:
      - "MAX_RECURSION_DEPTH = 64 const inserted at line 69 (between profile_url_regex and scrub fn doc)"
      - "scrub_json body now delegates: scrub_json_inner(value, 0)"
      - "scrub_json_inner is file-private; takes &Value + depth: usize; returns Value::Null at depth >= MAX_RECURSION_DEPTH (NOT truncated subtree)"
      - "Added test scrub_json_at_depth_cap_returns_null_not_truncated_tree (line 269) — passing"
      - "Public API byte-identical: pub fn scrub_json(&Value) -> Value (line 101)"
    notes: "Maintenance-invariant #2 in Task 3 must reference both [`scrub_json`] and [`MAX_RECURSION_DEPTH`] symbols already in this file."
  - task: 2-validate
    commit: bb9fe84a7
    filesCreated: []
    filesModified:
      - .claude/decision-queue.json
    keyDecisions:
      - "DQ 7fea5e9cab18-001 (kind: validate-pending-laptop) mutated to result=pass at phase tip add01bd89"
      - "cargo check 1m 08s + clippy 1m 55s + test 11p/0f/2i — all exit 0"
    notes: "Phase tip at brief-author time is bb9fe84a7. Worker forks from this ref."
```

### 3.b Mandatory lessons (per advisor-orchestrator.md §2.4 file-class table)

1. `.claude/lessons/feedback_clippy_doc_lazy_continuation_in_doc_comments.md` — **CRITICAL.** The new `//!` Maintenance-invariants block contains a 4-item numbered list with multi-line items. Each list item is separated by a blank `//!` line. Within each item, continuation lines start with whitespace-aligned bullet text (NOT the word "and"). The §10.4 verbatim text is already compliant — DO NOT reformat.
2. `.claude/lessons/feedback_clippy_test_style.md` — workspace-level clippy denies `-D warnings` including `doc_lazy_continuation`.
3. `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate the three Edit anchors verbatim BEFORE making any edit. The brief includes them verbatim in §2.2.

### 3.c Plan references

- `.claude/PRPs/plans/v1-redaction-r1.plan.md` §13 Task 3 (lines 1159-1235).
- `.claude/PRPs/plans/v1-redaction-r1.plan.md` §10.4 (lines 626-675) — source of the verbatim Maintenance-invariants block.

## 4. Constraints

### 4.1 Hard refusals

1. **NEVER edit any file other than `crates/db_schema/src/source/governance/redaction.rs`.** Task 3 is comment-only on this single file. Per Plan §11 NOT-modified list:
   - `crates/api/api/src/governance/redaction.rs` (re-export shim) — DO NOT EDIT
   - `crates/api/api/src/governance/admin_dashboard_html.rs` (caller) — DO NOT EDIT
   - `crates/api/api/src/governance/submit_jury_vote.rs` (caller) — DO NOT EDIT
   - `crates/api/api/src/governance/admin_rule_sets.rs` (caller) — DO NOT EDIT
   - `crates/db_schema/src/source/governance/governance_log.rs` (caller) — DO NOT EDIT
   - `crates/server/tests/e2e.rs` (existing tests) — DO NOT EDIT
2. **NEVER add or modify any test.** Task 3 is doc-only; existing 11 active + 2 #[ignore] tests stay byte-identical.
3. **NEVER reformat the §10.4 verbatim block.** Copy character-for-character including the leading 4-space indents on continuation lines. Per `feedback_clippy_doc_lazy_continuation_in_doc_comments.md`, the block is pre-validated against `-D warnings`.
4. **NEVER use `///` for the module-level Maintenance-invariants block.** Use `//!` (module-doc prefix). The block goes INSIDE the existing module-doc, before the first `use` statement.
5. **NEVER write `"answered_by": "advisor"` or `"approved_by": <value>` in any DQ entry.** Both are advisor-exclusive per DQ Hard refusals #1 + #8.
6. **NEVER use the abolished `next_id = max(all_ids)+1` recipe.** Use `bash scripts/brehon/dq-v3-new-entry.sh` per DQ Hard refusal #9.

### 4.2 Worker self-test gates (run BEFORE `git push`)

Per `feedback_fix_impl_pre_push_cargo_check.md` and Plan §13 Task 3 §VALIDATE block:

```bash
bash scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-redaction-r1-task3-check.log 2>&1
echo "exit: $?"
tail -5 .claude/PRPs/debug/v1-redaction-r1-task3-check.log
# EXPECT exit 0

bash scripts/brehon/cargo-clippy.sh -p lemmy_db_schema --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task3-clippy.log 2>&1
echo "exit: $?"
tail -15 .claude/PRPs/debug/v1-redaction-r1-task3-clippy.log
# EXPECT exit 0 — doc_lazy_continuation must not fire
```

Narrow `-p lemmy_db_schema` scope per Task 1 + Task 2 worker discipline (matches sibling commits).

**If clippy fires `doc_lazy_continuation` on the new module-doc block:** the §10.4 verbatim text already complies — but if the worker introduces any whitespace drift, the lint will fire. Recovery recipe (in scope): re-copy the §10.4 block verbatim from `.claude/PRPs/plans/v1-redaction-r1.plan.md` lines 636-664. If the lint still fires after a verbatim re-copy, raise `kind: "blocker"` DQ — do NOT `#[allow]`-spam.

### 4.3 validate-pending-laptop DQ entry (post-push)

After worker self-test passes AND `git push origin <worker-branch>` succeeds, generate a composite v3 id and append a `kind: "validate-pending-laptop"` entry to `pending[]`. Use the canonical helper recipe (NEVER `next_id = max+1`):

```bash
# Step 1: generate composite id via helper
NEW_ID=$(bash scripts/brehon/dq-v3-new-entry.sh)
echo "$NEW_ID"

# Step 2: write fragment to .claude/PRPs/debug/v1-redaction-r1-task3-vp.json
# (forward-slash literal path; per feedback_windows_backslash_path_dq_via_write_fragment.md)
```

Fragment shape (use the Write tool — do NOT inline-Python on the live DQ):

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 at fragment-write time>",
  "question": "Run cargo check + clippy on canonical brehon-fork-redaction-r1 lane worktree for v1-redaction-r1 Task 3 — regex commentary + Maintenance invariants (comment-only, no new tests).",
  "options": ["all gates pass", "any gate fails"],
  "context": "Task 3 inserts Maintenance-invariants H2 in module-doc + 2 per-regex policy comments. Pure comment-only edit; zero production-code or test changes. Worker self-test (cargo check + clippy on -p lemmy_db_schema) passed; laptop-side runs --workspace gates.",
  "answer": null,
  "answered_by": null,
  "approved_by": null,
  "resolved_at": null,
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "branch": "<worker-branch-name>",
  "phase_task": "v1-redaction-r1-task-3",
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-redaction-r1-task3-check.log 2>&1\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-redaction-r1-task3-clippy.log 2>&1\""
  ]
}
```

Then:

```bash
# Step 3: append via helper (NEVER inline JSON edit)
bash scripts/brehon/dq-v3-append-fragment.sh .claude/PRPs/debug/v1-redaction-r1-task3-vp.json --pending
```

### 4.4 Commit subjects

Two commits expected on the worker branch:

```
feat(redaction): regex commentary + Maintenance invariants (task 3)

chore(decision-queue): impl raised validate-pending-laptop for v1-redaction-r1 task 3
```

Each commit's body MUST end with a `HANDOVER:` YAML trailer per `feedback_handover_trailer_cohort_propagation.md`.

## 5. Definition of done

- [ ] Edits A, B, C applied verbatim per §2.2; all 6 `rg -n` checks in §2.3 pass.
- [ ] Worker self-test gates `bash scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full` + `bash scripts/brehon/cargo-clippy.sh -p lemmy_db_schema --features full --no-deps -- -D warnings` both exit 0.
- [ ] Two commits on worker branch with subjects matching §4.4.
- [ ] Worker branch pushed to `origin/junior/...`.
- [ ] `.claude/decision-queue.json` has a new `kind: validate-pending-laptop` entry in `pending[]` with composite v3 id, `from: impl`, `answered_by: null`, `approved_by: null`.
- [ ] `HANDOVER:` trailer on both commits.

## 6. Out of scope (do NOT do)

- Any edit to a file outside `crates/db_schema/src/source/governance/redaction.rs`.
- Any new test.
- Any production-code change (no regex tightening; no behaviour drift).
- Running e2e tests (the phase-tip e2e gate is advisor-driven, NOT this task).
- Authoring the retro (Task 4 — advisor-driven, separate dispatch).
- Closing GH Issue #58 (BM at merge time).
