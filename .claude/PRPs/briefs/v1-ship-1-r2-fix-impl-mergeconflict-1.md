# Brief — v1-ship-1-r2 conflict-resolution impl-task (merge governance-v0 into phase-v1-ship-1)

## 1. Role + dispatch line

`[role:impl-task] resolve PR#137 governance-v0 merge conflict — e2e.rs additive + DQ union`

This is a **merge-conflict resolution task**, NOT fresh feature authoring. Branch from `phase-v1-ship-1`. Merge `origin/governance-v0` in, resolve the 2 conflicts mechanically per the recipes below, re-validate, commit the merge, push.

## 2. Scope

PR #137 (`phase-v1-ship-1` → `governance-v0`) is `CONFLICTING`. `governance-v0` advanced 77 commits (v1-AD-e shipped on another lane) since merge-base `8c271285e`. Resolve the conflict so the PR becomes mergeable, preserving BOTH lanes' work.

**Exactly 2 conflict files. NO other files. NO scope beyond conflict resolution + re-validation.**

### 2.1 `crates/server/tests/e2e.rs` — ONE conflict region, PURELY ADDITIVE

After `git merge origin/governance-v0`, the conflict region is a single block (~line 14864-15187 in the conflicted file):

```
<<<<<<< HEAD
#[tokio::test(flavor = "multi_thread")]
async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()> {
  ... [the COMPLETE agpl test — ship-1's Task 6 rebuild, ~129 lines] ...
}
=======
// ============================================================================
// v1-AD-e — server-rendered HTML admin pages (Dashboard + Audit)
  ... [v1-AD-e admin-HTML test block — comment banner + 5 admin_*_html tests] ...
>>>>>>> origin/governance-v0
  );

  Ok(())
}
```

**Resolution recipe (CONCATENATE — zero logic merge, zero overlap):**

1. Both sides are NET-NEW test code appended at end-of-file by two independent lanes. There is ZERO semantic overlap (`agpl_source_disclosure_surface_returns_notice` does NOT exist on the governance-v0 side; the `admin_*_html` tests do NOT exist on the HEAD side — verified by advisor: `git show origin/governance-v0:crates/server/tests/e2e.rs | grep -c agpl_source_disclosure_surface_returns_notice` == 0).
2. The resolved content is: **the ENTIRE HEAD block (the agpl test, complete, including its closing `}`)** immediately followed by **the ENTIRE governance-v0 block (the `// ==== v1-AD-e ====` comment banner + all 5 admin-HTML tests)**.
3. Remove ALL THREE conflict markers: `<<<<<<< HEAD`, `=======`, `>>>>>>> origin/governance-v0`.
4. **CRITICAL — the trailing `);` + `Ok(())` + `}` that appears AFTER `>>>>>>> origin/governance-v0` is the close of governance-v0's LAST admin-HTML test function** (the merge cut governance-v0's block mid-function). It MUST remain, immediately after the governance-v0 block content, as that last test's closing lines. Do NOT delete it; do NOT duplicate it onto the agpl test (the agpl test already has its own complete `}`).
5. Net effect: the resolved file = `<prior content up to line 14862 unchanged>` + `<blank line>` + `<complete agpl test fn>` + `<blank line>` + `<v1-AD-e comment banner + 5 admin-HTML test fns, complete>` + `<rest of file unchanged>`. Both lanes' tests coexist; nothing is overwritten.

After resolving, `grep -c "fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs` MUST == 1 AND `grep -c "fn admin_dashboard_html_returns_html_for_admin" crates/server/tests/e2e.rs` MUST == 1 AND there must be ZERO remaining `<<<<<<<`/`=======`/`>>>>>>>` markers in the file.

### 2.2 `.claude/decision-queue.json` — JSON-aware UNION (7 conflict regions, 14 markers)

Do NOT hand-merge the conflict markers in this file. It is a structured JSON document; naive text-merge will corrupt it. Use the canonical resolver:

```bash
git checkout --theirs .claude/decision-queue.json   # take governance-v0 (trunk-target) base
# THEN re-apply the ship-1-only resolved entries via the canonical resolver:
bash scripts/brehon/resolve-dq-canonical.sh v1-ship-1
```

If `scripts/brehon/resolve-dq-canonical.sh` does NOT produce a clean unioned `.claude/decision-queue.json` (it spans phase-branch + worker branches; verify its output is valid JSON with `python -c "import io,json; json.load(io.open('.claude/decision-queue.json',encoding='utf-8'))"`), FALL BACK to the structured Python union:

- Load HEAD-side (`git show HEAD:.claude/decision-queue.json`) and theirs-side (`git show MERGE_HEAD:.claude/decision-queue.json`) as JSON.
- Union `pending[]` + `resolved[]` by entry `id`. On `id` collision: **governance-v0 (MERGE_HEAD / theirs) wins** — it is the trunk-target and has the most-recent cross-lane state for shared ids. EXCEPTION: ship-1-specific ids that exist ONLY on HEAD (the r2 entries: #262, #263 in resolved; #248 in pending) are preserved from HEAD.
- Keep `schema_version: 2`. An entry must never appear in BOTH `pending[]` and `resolved[]` — if a collision id is resolved on either side, it goes to `resolved[]` only.
- Write back with `json.dump(..., ensure_ascii=False, indent=2)` + trailing newline. **DO NOT rewrite historical mojibake/idiosyncratic fields** — forward-only per `.claude/rules/decision-queue.md` v2 rule.
- Validate: `python -c "import io,json; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); assert d['schema_version']==2; ids=[e['id'] for e in d['pending']+d['resolved']]; assert len(ids)==len(set(ids)), 'dup id'; print('OK', sorted(e[\"id\"] for e in d[\"pending\"]))"`

The HEAD-side r2 entries that MUST survive the union: `#263` (resolved, result:pass, the Task-6 e2e PASS), `#262` (resolved), `#248` (pending), `#229` (pending log). The governance-v0 side brings v1-AD-e's resolved entries.

### 2.3 Re-validation (MANDATORY before commit — the merged e2e.rs must still compile + pass)

After BOTH conflicts resolved + `git add` both files, BEFORE `git commit`, run §5.2-laptop Phase-1 equivalents from the worktree:

```
bash scripts/brehon/cargo-check.sh --workspace --features full      # exit 0 required
bash scripts/brehon/cargo-clippy.sh --workspace --features full     # exit 0 required (Lemmy denies warnings)
```

If EITHER exits non-zero: the concatenation introduced a compile/lint error (e.g. a duplicate `use` if both blocks imported the same symbol at module scope, or a name collision). FIX it in the same merge (if mechanical + in-scope: e.g. dedupe a duplicated import), OR raise `kind: "blocker"` DQ (if the conflict is non-mechanical) and STOP. Do NOT `#[allow]`-spam. Do NOT push a broken merge.

Phase-2 e2e (full suite, ~26-46 min) is the ADVISOR's step AFTER this task finalize-merges (the advisor re-runs §5.2 Phase-2 on the merged tip + mutates a fresh validate-pending-laptop-e2e DQ). Do NOT run the full e2e suite in this task — Phase-1 check+clippy is the worker's gate; Phase-2 is advisor-driven post-merge.

## 3. Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A/B/C error-shape (both conflict blocks already use Case A `LemmyResult<()>`; this is context for verifying the concatenation didn't break error discipline).
- `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e test harness pattern (context: confirm neither block's harness wiring is disturbed by concatenation).
- `.claude/lessons/feedback_verify_squash_merge_content_byte_level.md` — after merge, BYTE-verify (grep) both test families are present; do not trust line-count.
- `.claude/lessons/feedback_multi_lane_worktree_discipline.md` — the DQ conflict is the documented cross-lane pattern; governance-v0 (trunk-target) wins on id collision.
- `.claude/rules/decision-queue.md` — DQ schema v2; forward-only (do NOT rewrite historical fields); an id is never in both pending+resolved.
- `scripts/brehon/resolve-dq-canonical.sh` — read it before running it; understand what it unions.

## 4. Constraints

- **File scope HARD LIMIT:** the ONLY files this task may modify are `crates/server/tests/e2e.rs` and `.claude/decision-queue.json` (the 2 conflict files) — plus whatever `git merge origin/governance-v0` auto-merges cleanly (those are NOT conflicts; leave git's auto-merge result as-is). Do NOT touch any other file. Do NOT "improve" adjacent code. If the merge auto-merges files you didn't expect, that is normal (governance-v0's 77 commits) — only the 2 CONFLICT files need manual resolution.
- **e2e.rs is PURELY ADDITIVE concatenation** — there is NO logic to merge, NO test to rewrite, NO behavior to reconcile. If you find yourself editing the *body* of either test, STOP — you've misread the conflict. Both test families are pre-existing, already-reviewed, already-green code from two lanes. The ONLY operation is: remove 3 markers, keep both blocks in order (HEAD-block then gov-v0-block), preserve the trailing `);Ok(())}` as gov-v0's last test close.
- **DQ is JSON-aware union, NEVER text-merge** — use `resolve-dq-canonical.sh` or the structured Python fallback. governance-v0 wins on id collision; HEAD's r2 entries (#262/#263 resolved, #248/#229 pending) survive. Validate JSON + no-dup-id + schema_version==2 before commit.
- **Re-validation gate is MANDATORY** — `cargo-check.sh --workspace --features full` AND `cargo-clippy.sh --workspace --features full` BOTH exit 0 before `git commit`. A broken merge pushed = a full advisor recovery cycle. Local check is ~30s-8min warm; cheap insurance. Per `feedback_fix_impl_pre_push_cargo_check.md`.
- **Commit message (verbatim contract):** `Merge governance-v0 into phase-v1-ship-1 — resolve e2e.rs (additive: agpl + AD-e admin-HTML coexist) + decision-queue.json (cross-lane union)`. This is a MERGE COMMIT (`git merge` produces it); do NOT squash, do NOT rebase, do NOT `git merge --ff-only` (governance-v0 is not an ancestor — it WILL be a real merge commit with 2 parents).
- **Mid-task DQ push discipline:** if you raise a `kind:"blocker"` DQ (non-mechanical conflict), commit+push it to the worker branch immediately per `.claude/rules/decision-queue.md` "Mid-task visibility" so the advisor sees it.
- **NO force, NO --no-verify, NO #[allow]-spam, NO scope creep.** If anything about the conflict is NOT the clean additive/union shape described here, raise a blocker DQ and STOP — do not improvise.
- **Do NOT push to governance-v0.** Push only the worker branch (Junior finalize-merges into phase-v1-ship-1). Per phase-branch.md.

Brief commit body — mandatory file-class lessons fired: `crates/server/tests/e2e.rs` (any edit) → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` injected §3 (advisory: this is a concatenation, not fresh authoring, but table is mechanical). §2.3 hybrid search fired: PMD #214 (byte-verify squash/merge), #254 (multi-lane DQ union, gov-v0 wins collision), #129 (conflict-resolution is impl-task code work). Authored on phase-v1-ship-1 lane worktree.
