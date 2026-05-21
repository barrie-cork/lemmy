[role:impl-task] brehon-conformance-audit Task 12 — author two paired lesson files (Cohort 5)

## 1. Role + dispatch

`[role:impl-task]` — author two new `.claude/lessons/feedback_*.md` files. No code; no Edit of existing files.

## 2. Scope

Per plan §13 Task 12 + plan §10.12. Create two paired lesson files that codify the conformance-audit skill's defect class + the planning-side prevention rule:

1. **File 1 of 2:** `.claude/lessons/feedback_mirror_phase6_convention_in_same_file.md` — defect-class lesson naming Phase-6 convention-divergence as the catch.
2. **File 2 of 2:** `.claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md` — planning-side prevention (compile-at-Task-N not Task-N+1).

Both files must follow existing `feedback_*.md` shape: YAML frontmatter (`name`, `description`, `type: feedback`, `originSessionId`), then body sections.

### 2.1 File 1 spec — `feedback_mirror_phase6_convention_in_same_file.md`

**Frontmatter** (standard feedback shape; read an existing lesson for reference — see §3):

```yaml
---
name: Mirror Phase-6 conventions in the same file
description: Phase-6 federation handlers + DB-source modules have established conventions per-file/per-module. New code added to the same file must mirror the canonical sibling (error idiom, conn type, append-reborrow, trait bounds, conn acquisition, ADR-015 pseudonym handling). The brehon-conformance-audit skill is the structural detection; Clippy disallowed_methods is the structural prevention. Surfaced at v1-federation-inbound-b fix-impl-1+2+3 (3 compile-caught + 1 latent footgun Finding 6.1).
type: feedback
originSessionId: <new uuid — generate or use task uuid>
---
```

**Required cross-links (verbatim, in body):**

- `[[feedback_plan_stub_uniformity_with_canonical_sibling]]`
- `[[feedback_lemmy_error_no_std_error]]`
- `[[feedback_multi_write_handlers_need_transactions]]`
- `[[project_phase6_convention_divergence_class]]`
- `[[feedback_read_canonical_before_writing_spec]]`

**Body sections (in order):**

1. `## What this catches` — define Phase-6 convention-divergence defect class. The class is: new federation governance code adds a function to one of the three federation governance modules (`crates/apub/activities/src/governance/**.rs`, `crates/api/api/src/governance/**.rs`, `crates/db_schema/src/source/governance/**.rs`) and the new function diverges from an established canonical-sibling pattern in the same file along one or more of six axes.

2. `## Evidence` — the four fed-in-b incidents (3 compile-caught at fix-impl-1+2 + 1 latent Finding 6.1 caught at fix-impl-3); cite the Task 7 dogfood report at `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md` (MUST cite — the validate-checkpoint greps for the substring `conformance-audit-v1-federation-inbound-b-dogfood`).

3. `## The six axes` — lift the table from plan §10.1 VERBATIM. The plan §10.1 axis table lists: (1) conn type, (2) append-reborrow, (3) trait bound, (4) error idiom, (5) conn acquisition, (6) ADR-015 pseudonym handling. Read plan §10.1 for the exact wording before lifting.

4. `## The structural fix` — name the skill (`.claude/skills/brehon-conformance-audit/`) + the Clippy gate (`clippy.toml` with workspace-allow + per-module `#![deny(clippy::disallowed_methods)]` on the three federation `mod.rs` files). Cite the rustc lint-precedence rule 4 mechanism (workspace-allow + lower-scope deny = federation-only enforcement).

5. `## Brief-author checklist` — "If your task creates a fn under one of the three federation governance module roots: (a) read the same-file sibling first; (b) cite the sibling in brief §3 Required reading with line range; (c) rerun the conformance-audit skill at brief-time with `target_scope = file <brief-named-file>`."

### 2.2 File 2 spec — `feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md`

**Frontmatter** (standard feedback shape):

```yaml
---
name: Plan mirror stubs must compile + annotate pre-landed callers
description: Planner authoring a multi-task plan whose Task N adds infra and Task N+1 first-calls it MUST include a temporary unit test (or assert call site) that exercises Task N's signature so the compiler proves call-site discipline at Task N's §15, NOT Task N+1's. Strip the assert in the same task that strips #[expect(dead_code)]. Prevents the "stub passes Task N validate, breaks at Task N+1 because no compile-call exists" failure mode.
type: feedback
originSessionId: <new uuid>
---
```

**Required cross-link:**

- `[[feedback_dead_code_shields_latent_type_errors]]` — if this lesson does not exist at write time, **fall back** to `[[feedback_plan_stub_uniformity_with_canonical_sibling]]` AND document the substitution in a one-line body note ("Cross-link `feedback_dead_code_shields_latent_type_errors` not yet promoted per session-retro-2026-05-20 change-#3; using `feedback_plan_stub_uniformity_with_canonical_sibling` as closest sibling.").

   **Verify**: `test -f .claude/lessons/feedback_dead_code_shields_latent_type_errors.md && echo PROMOTED || echo FALLBACK`.

**Body sections (in order):**

1. `## What this catches` — the failure-mode where a Task N "stub" infra (a struct, fn signature, or migration) passes its own §15 `cargo check` because nothing in the workspace calls it yet (or it's annotated `#[expect(dead_code)]`); Task N+1 first-calls the stub and the call-site error pops at Task N+1's §15 instead of Task N's.

2. `## Recipe (planner-side)` — the planner authoring such a plan MUST add a temporary unit test or a `let _ = <stub>(...)` assert call site in Task N's IMPLEMENT block that exercises the stub's signature against its eventual caller. The assert lives in the SAME crate as the stub, so Task N's `cargo check --workspace --features full` exercises it. Strip the assert IN THE SAME COMMIT that strips `#[expect(dead_code)]` (typically Task N+1).

3. `## Worked example` — name a concrete pattern (e.g., "Task N adds `pub fn process_inbound(payload: &Foo) -> Result<...>` with `#[expect(dead_code)]`; Task N+1 first-calls it from a handler. Without an assert, Task N+1 sees the first compile error when the handler signature mismatches. With an assert `let _: Result<_, _> = process_inbound(&Foo::test_value());` in Task N, the compiler catches the signature drift one task earlier.").

4. `## When this applies` — multi-task plans where Task N adds infra (data structures, fn signatures, migrations) that no existing call-site exercises, and Task N+1+ adds the first call sites.

## 3. Required reading

Read in order BEFORE writing either file:

1. `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — §10.1 (six axes table — File 1 lifts verbatim); §10.12 (lesson cross-link discipline); §13 Task 12 (lines ~1608-1672) for the IMPLEMENT spec.
2. `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-dogfood-2026-05-20.md` — the dogfood evidence File 1 must cite.
3. Read **2 existing lessons** for frontmatter + structure (per `feedback_read_canonical_before_writing_spec.md`):
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` (canonical lesson shape)
   - `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` (planner-side lesson — closest sibling to File 2's pattern)
4. `.claude/lessons/feedback_one_system_memory_in_repo.md` + `.claude/lessons/feedback_lesson_mirror_check.md` + `.claude/lessons/feedback_lesson_must_pair_with_structural_fix_when_fixable.md` — cross-link discipline guidance.

**MIRROR refs:** existing `.claude/lessons/feedback_*.md` for frontmatter shape (4 fields min: `name`, `description`, `type: feedback`, `originSessionId`); existing body shape (`## section` headers, `[[link]]` cross-references, evidence-then-recipe structure).

## 4. Constraints

1. **NEVER touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, or any file outside `.claude/lessons/`.**
2. **Two file creates only.** No modifications to existing files. No `sync-lessons-to-pmd.sh` invocation (out of scope per plan §13 Task 12 GOTCHA — advisor handles in retro).
3. **Cross-link `[[...]]` syntax is mandatory** — the validate-checkpoint greps for `[[feedback_lemmy_error_no_std_error]]` + `[[project_phase6_convention_divergence_class]]` substrings in File 1.
4. **File 1 MUST cite the dogfood report** — the validate-checkpoint greps for `conformance-audit-v1-federation-inbound-b-dogfood` substring.
5. **File 2 fallback discipline** — if `feedback_dead_code_shields_latent_type_errors.md` is not yet promoted (likely the case per plan GOTCHA), use `feedback_plan_stub_uniformity_with_canonical_sibling.md` as the closest-sibling cross-link AND document the substitution in body prose.
6. **Frontmatter parses as valid YAML** — the validate-checkpoint loads it with `yaml.safe_load`.
7. **Mid-task push discipline:** after both Writes, raise `kind: "validate-pending-laptop"` DQ per the standard pattern. Single command:
   ```
   test -f .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md && test -f .claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md && grep -F "[[feedback_lemmy_error_no_std_error]]" .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md && grep -F "[[project_phase6_convention_divergence_class]]" .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md && grep -F "conformance-audit-v1-federation-inbound-b-dogfood" .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md
   ```
   EXPECT: exit 0 (all four greps + both file-exists checks pass).
8. **DQ raise atomic order:** commit + push the two lesson files FIRST, THEN commit + push the DQ raise (atomic raise-before-dispatch).
9. **Branch:** fork from `phase-brehon-conformance-audit` tip (currently `4daccb4de`). Worker branch will be auto-generated by Junior.
10. **One commit** for the two lesson files (single commit, both files together); **one commit** for the DQ raise. Two commits total.

## 5. Validation gate (worker-side)

Per plan §13 Task 12 VALIDATE block:

```bash
# Both lesson files exist
test -f .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md
test -f .claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md

# Cross-links present (File 1)
grep -F "[[feedback_lemmy_error_no_std_error]]" .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md
grep -F "[[project_phase6_convention_divergence_class]]" .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md

# Lesson cites the dogfood report
grep -F "conformance-audit-v1-federation-inbound-b-dogfood" .claude/lessons/feedback_mirror_phase6_convention_in_same_file.md

# Frontmatter parses (both files)
python3 -c "
import yaml, re
for f in ['.claude/lessons/feedback_mirror_phase6_convention_in_same_file.md',
          '.claude/lessons/feedback_plan_mirror_stub_must_compile_and_annotate_prelanded.md']:
    content = open(f).read()
    m = re.match(r'^---\n(.*?)\n---', content, re.S)
    assert m, f'no frontmatter in {f}'
    fm = yaml.safe_load(m.group(1))
    print(f, 'frontmatter OK:', list(fm.keys()))
"
```

All checks must pass before raising the DQ.

## 6. Commit subject

`docs(lessons): add Phase-6 convention-divergence + plan-stub-compile-at-task-N lessons (task 12)`

LESSON trailer: include if you discover a durable observation (per `feedback_junior_pmd_write_convention.md`).
