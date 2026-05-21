[role:impl-task] brehon-conformance-audit fix-impl-4 — apply 7 CR fix-in-pr majors on PR #141

## 1. Role + dispatch

`[role:impl-task] brehon-conformance-audit fix-impl-4 — apply 7 CR fix-in-pr majors on PR #141`

## 2. Scope

Apply the **7 fix-in-pr majors** from CR triage on PR #141 (per
`.claude/PRPs/reviews/pr-141-findings.yaml` @ `phase-brehon-conformance-audit`
tip `293523e64`). Single commit `chore(skill): apply 7 CR fix-in-pr majors
on PR #141 (fix-impl-4)` covering all 7. NO new functionality; NO scope
expansion; CR's suggested diffs are the contract.

Numbered fix-impl-4 because fix-impl-1/2/3 were pre-PR cycle-3 catch-fire
remediations (DQ #311 mechanism revision) per retro section 2.

This is a **non-allowlist fix-impl** (not section G4 mechanical) — each
fix has a specific CR-provided diff but they touch 7 different files.
Treat as a **bundled mechanical fix** at the file level. Per finding,
apply CR's diff verbatim or tightly equivalent.

### 2.1 The 7 fixes (verbatim from CR triage rationales)

| ID | File | Fix |
|---|---|---|
| **cr-6** | `.claude/PRPs/briefs/brehon-conformance-audit-impl-10.md:64-65` (and duplicate at 103-110) | Replace constraint #8: old text "**NO cargo-check** required - cargo install outputs are captured separately at .claude/PRPs/debug/brehon-conformance-audit-task10-install.log." → new text "**Workspace cargo-check is required** - run bash scripts/brehon/cargo-check.sh --workspace --features full per section 5.3 before push." (CR "Also applies to: 103-110") |
| **cr-13** | `.claude/PRPs/plans/brehon-conformance-audit.plan.md:1308-1309` (verify line range with `grep -n "GOTCHA" .claude/PRPs/plans/brehon-conformance-audit.plan.md`) | Replace the Task-8a GOTCHA block per CR diff. Old: "Workspace clippy will STILL fail (exit 101) after Task 8a lands alone - because governance-v0 baseline at 4480a1bdb (before any Task 8 dispatch) had no clippy.toml, and removing the broken Task 8 dispatch's clippy.toml returns the tree to that baseline state." New: "Workspace clippy is expected to return to baseline behaviour after Task 8a lands alone, because removing the broken Task 8 dispatch's clippy.toml restores the pre-Task-8 state." |
| **cr-16** | `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json:14-34` | Apply CR's full schema patch: add `required` + `properties` + `additionalProperties:false` to `predictions.items`, `ground_truth_compile_caught.items`, `ground_truth_runtime.items`. Fetch full diff via `gh api repos/barrie-cork/lemmy/pulls/comments/3281002035 --jq .body`. |
| **cr-18** | `.claude/skills/brehon-conformance-audit/METRICS.md:23-26 + 37-39` | Replace `compile_caught[]` with `ground_truth_compile_caught[]` and `runtime[]` with `ground_truth_runtime[]` everywhere they appear in METRICS.md prose. Verify all occurrences via `grep -nE "(compile_caught|runtime)\[\]" .claude/skills/brehon-conformance-audit/METRICS.md`. |
| **cr-19** | `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh:110-111 + 120-131` | Fix FP attribution per CR diff. Old: `axis_fp[axis] += len(pred_targets - gt_targets)`. New: read `false_positives = data.get("false_positives", [])`, then for each axis: `fp_targets = {e.get("target") for e in false_positives if e.get("axis") == axis}`, then `axis_fp[axis] += len(pred_targets & fp_targets)`. Apply same correction at 120-131 per CR "Also applies to". Fetch full diff via `gh api repos/barrie-cork/lemmy/pulls/comments/3281002062 --jq .body`. |
| **cr-20** | `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh:143-149` | Lead-time loop. Old: `for gt_entry in gt_runtime:`. New: `for gt_entry in (gt_compile + gt_runtime):`. |
| **cr-21** | `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh:75-77` | Broaden the Rust fn regex per CR diff. Old: `r'^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)'`. New: `r'^(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+(?:"[^"]*"\s+)?)?fn\s+([A-Za-z_]\w*)'`. |

## 3. Required reading

- `.claude/PRPs/reviews/pr-141-findings.yaml` — full finding text + `cr_url` per ID
- `.claude/PRPs/reviews/pr-141-comment.md` — triage digest + bucket rationales
- `.claude/skills/brehon-conformance-audit/METRICS.md` — Attribution Rule 4 (cr-19) + lead-time spec (cr-20)
- `.claude/skills/brehon-conformance-audit/audit-metrics.schema.json` — schema cr-16 strengthens
- `.claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh` — script cr-19 + cr-20 fix
- `.claude/skills/brehon-conformance-audit/scripts/find-sibling.sh` — script cr-21 fixes
- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — Task 8a GOTCHA section near line 1308 (cr-13)
- `.claude/PRPs/briefs/brehon-conformance-audit-impl-10.md` — cr-6 contradiction sites at lines 64-65 + 103-110

**MIRROR refs:** none — surgical edits per CR's verbatim diffs.

## 4. Constraints

1. **NEVER touch `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`.** This task ONLY modifies the 7 files listed in section 2.1.
2. **One commit** for all 7 fixes — subject `chore(skill): apply 7 CR fix-in-pr majors on PR #141 (fix-impl-4)`. Cite each CR ID in the commit body (one bullet per cr-N).
3. **CR's suggested diff is the contract.** For each finding, read its `notes:` field from `pr-141-findings.yaml`. If `notes` is truncated, fetch the full CR text via `gh api repos/barrie-cork/lemmy/pulls/comments/<id> --jq '{path, line, body}'`. Apply the diff verbatim or tightly equivalent. Where CR provides a Suggested edit code block, copy that diff exactly.
4. **After each Edit, verify the file still parses:**
   - `audit-metrics.schema.json`: `python -c "import json; json.load(open('.claude/skills/brehon-conformance-audit/audit-metrics.schema.json'))"` → must exit 0.
   - `compute-metrics.sh`: `bash -n .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh` → must exit 0.
   - `find-sibling.sh`: `bash -n .claude/skills/brehon-conformance-audit/scripts/find-sibling.sh` → must exit 0.
   - Markdown files: visual-check the diff via `git diff <file>`.
5. **Validate dogfood metrics didn't regress** after compute-metrics.sh fixes:
   ```
   bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh .claude/PRPs/audit-metrics/v1-federation-inbound-b.json
   ```
   EXPECT stdout containing `axis-4 precision: 1.000` AND `axis-4 recall: 1.000` AND `Lead time (median): ` (numeric, may differ from prior 27.5h if cr-20 broadens source) AND `Latent-footgun catch rate (axis-4): 1`. The FP attribution fix (cr-19) MUST NOT regress precision because v1-federation-inbound-b's metrics file has zero `false_positives[]` entries (FP=0 either way).
6. **Sanity-check the broadened regex (cr-21)** by running:
   ```
   python -c "
   import re
   fn_pat = re.compile(r'^(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?(?:extern\s+(?:\"[^\"]*\"\s+)?)?fn\s+([A-Za-z_]\w*)')
   tests = [('pub fn foo() {}', 'foo'), ('async fn bar() {}', 'bar'), ('pub async fn baz() {}', 'baz'), ('pub(crate) fn qux() {}', 'qux'), ('pub(super) fn quux() {}', 'quux'), ('const fn corge() {}', 'corge'), ('unsafe fn grault() {}', 'grault'), ('pub unsafe fn garply() {}', 'garply'), ('extern \"C\" fn waldo() {}', 'waldo'), ('pub extern \"C\" fn fred() {}', 'fred')]
   for src, expected in tests:
       m = fn_pat.match(src)
       actual = m.group(1) if m else None
       assert actual == expected, 'regex fail: ' + repr(src) + ' -> ' + repr(actual) + ' (want ' + repr(expected) + ')'
   print('all 10 regex variants OK')
   "
   ```
   EXPECT: `all 10 regex variants OK`.
7. **Mid-task push discipline:** after the 7-fix commit + the findings-YAML update commit, raise `kind: "validate-pending-laptop"` DQ. Single validation command:
   ```
   bash .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh .claude/PRPs/audit-metrics/v1-federation-inbound-b.json && python -c "import json; json.load(open('.claude/skills/brehon-conformance-audit/audit-metrics.schema.json'))" && bash -n .claude/skills/brehon-conformance-audit/scripts/find-sibling.sh && bash -n .claude/skills/brehon-conformance-audit/scripts/compute-metrics.sh && grep -c "Workspace cargo-check is required" .claude/PRPs/briefs/brehon-conformance-audit-impl-10.md && grep -c "return to baseline behaviour after Task 8a" .claude/PRPs/plans/brehon-conformance-audit.plan.md && grep -c "ground_truth_compile_caught" .claude/skills/brehon-conformance-audit/METRICS.md
   ```
   EXPECT: exit 0 + metrics line printed + all greps return >=1.
8. **After 7-fix commit, in a SECOND commit, update each fix-in-pr finding's `addressed_in` field** in `.claude/PRPs/reviews/pr-141-findings.yaml` to the new 7-fix commit's short SHA. THEN recompute counters + flip `recommendation` from `block` to `approve`. Commit subject: `chore(advisor): update pr-141 findings - 7 majors addressed in <short-sha> (fix-impl-4)`.

   Worker authors this YAML-update commit on advisor's behalf (worker has the file in worktree state + knows the new SHA). The commit subject `chore(advisor):` matches the attribution-integrity detection regex `^(chore|docs)\((advisor|decision-queue)\)`.
9. **DQ raise atomic order:** commit + push 7-fix FIRST; commit + push findings-YAML update SECOND; commit + push DQ raise THIRD. Three commits total.
10. **Branch:** fork from `phase-brehon-conformance-audit` tip (currently `293523e64`). Worker branch will be auto-generated by Junior.
11. **Read the brief FIRST.** Per the failure mode of bm-triage Junior #394 (which never read its brief), the FIRST `Read` tool call in this task MUST be on `.claude/PRPs/briefs/brehon-conformance-audit-fix-impl-4.md`. The section 2.1 7-row table is the verbatim contract for this brief.

## 5. Validation gate (worker-side)

All checks below must pass before raising the DQ:

- Section 4 constraint 4 (file parses): all 3 parse-checks exit 0
- Section 4 constraint 5 (dogfood non-regression): axis-4 precision/recall both 1.000
- Section 4 constraint 6 (regex test): 10 variants pass
- Section 4 constraint 7 (combined validation command): exit 0 + greps return >=1

## 6. Commit subject

Three commits, in order:

1. `chore(skill): apply 7 CR fix-in-pr majors on PR #141 (fix-impl-4)` — body lists each cr-N: file:line + one-line summary
2. `chore(advisor): update pr-141 findings - 7 majors addressed in <short-sha> (fix-impl-4)`
3. `chore(decision-queue): impl raised DQ #<next-id> - fix-impl-4 validate-pending-laptop`

LESSON trailer: per `feedback_junior_pmd_write_convention.md`, append `LESSON: ...` if any durable observation surfaces (e.g., a CR diff that didn't apply cleanly; an unexpected callsite).
