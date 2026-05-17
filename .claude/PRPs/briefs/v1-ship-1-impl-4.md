# v1-ship-1 — Task 4 impl brief (e2e test)

## 1. Role + dispatch line

`[role:impl-task]` v1-ship-1 task 4 — append e2e test `agpl_source_disclosure_surface_returns_notice` to `crates/server/tests/e2e.rs` (single anchor-Edit at file end), asserting both `/api/v4/site` source_disclosure block + `/api/v4/source` notice body.

Dispatch string (verbatim):

```
[role:impl-task] v1-ship-1 task 4 — see .claude/PRPs/briefs/v1-ship-1-impl-4.md
```

## 2. Scope

Execute **plan §13 Task 4** exactly (`.claude/PRPs/plans/v1-ship-1-r1.plan.md` lines 993–1106). It is fully self-contained: the IMPLEMENT block (plan lines 1012–1075) is the **verbatim** test function to append — copy it as-is, do not paraphrase or restructure.

**Produce:** one new `#[tokio::test(flavor = "multi_thread")]` fn `agpl_source_disclosure_surface_returns_notice` appended at the END of `crates/server/tests/e2e.rs`, after the last existing test's closing `Ok(())` + `}`.

**Boundaries:**

- **Commit ONLY** `crates/server/tests/e2e.rs`. No other file. (`creates: []`, `modifies: [crates/server/tests/e2e.rs]` per plan §13 FILES.)
- **Do NOT author** any other production code, any other test, any DTO, any route. Tasks 1–3 are already on the phase branch — do not re-touch them.
- **One single Edit call** appending at file end. NOT two Edits. NOT a Write-replace of the file. e2e.rs is ~14,775 lines; multi-Edit / large-range Edit into e2e.rs hangs Junior workers (this is the binding discipline — see §4 + Required reading).
- The test fn outer return is **`lemmy_utils::error::LemmyResult<()>`** (Case A). Bare `?` propagation throughout. NO `Result<(), Box<dyn Error>>` outer (Case B). NO `.map_err(|e| anyhow::anyhow!(...))?` bridges. Mixing shapes = §G4 row 4c hard refusal.
- `requires:` Task 2 (`source_disclosure` field) + Task 3 (`/api/v4/source` route) — BOTH already merged on `phase-v1-ship-1` (commits on branch; verified by advisor before dispatch). Do not file a `requires`-unmet blocker.

## 3. Required reading (read these FIRST, in order)

1. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 Task 4 (lines 993–1106) — the verbatim IMPLEMENT block + all 5 GOTCHAs. **This is the contract.**
2. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §10.6 (lines 527–542) — e2e outer-shape Case A discipline; canonical sibling `crates/server/tests/e2e.rs:11001-11924` (v1-SL-b fixtures mod). Read the sibling at that range to mirror the error-shape verbatim.
3. `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §10.7 (lines 544–568) — in-process HTTP via `actix_web::test`; mirror `all_mvp_endpoints_return_non_404` at `crates/server/tests/e2e.rs:3693-3815`.
4. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **§2.4 MANDATORY (e2e.rs edit).** Case A is the post-LemmyResult-unification canonical shape. The plan pre-resolved this is Case A (do NOT re-derive; do NOT pick Case B/C). `serde_json::from_slice` → bare `?` works (LemmyError has `From<serde_json::Error>`; sibling `e2e.rs:4413`).
5. `.claude/lessons/feedback_async_pool_test_pattern.md` — **§2.4 MANDATORY (e2e.rs edit).** Async pool / DbPool fixture pattern context for e2e tests.
6. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **bound by plan §13 Task 4 GOTCHA 3** (DQ #117 + multiple retros). Single anchor-Edit append discipline — the canonical reason the edit MUST be one Edit at file end.

## 4. Constraints (enforce — hard refusals)

1. **Single anchor-Edit append, file end only.** Per plan GOTCHA 3 + `feedback_junior_worker_e2e_edit_hang.md`. The Edit's `old_string` anchors on the LAST existing test's tail (its final `Ok(())\n}` — read the file tail to find it); `new_string` = that same tail + `\n\n` + the verbatim §13 test fn. ONE Edit call. If you find yourself wanting a second Edit into e2e.rs, STOP and re-read GOTCHA 3.
2. **Pre-edit uniqueness check (plan GOTCHA 4):** before editing, run `grep -cE "async fn agpl_source_disclosure_surface_returns_notice" crates/server/tests/e2e.rs` — EXPECT `0`. If non-zero, the refactor tier added a colliding name: raise `kind: "blocker"` DQ (`from: "impl"`), do NOT proceed.
3. **Bootstrap choice is pre-resolved (plan GOTCHA 2 / DQ #226):** use `governance_fixtures::bootstrap()` (defined at `crates/server/tests/e2e.rs:801`). Do NOT use `admin_config_fixtures::bootstrap()` (line 5578). Do NOT file a blocker about which to pick — the advisor already resolved it (DQ #226 RESOLVED, binding).
4. **Case A only.** Outer `lemmy_utils::error::LemmyResult<()>`, bare `?`. Verbatim from plan §13 IMPLEMENT. No error-bridge closures.
5. **Inline imports inside the test fn** (matches e2e.rs sibling convention — the plan IMPLEMENT block already has them: `use actix_web::{App, test, web::Data};` etc inside the fn). Do NOT add top-of-file `use` statements.
6. **Validation = Shape G SUSPENDED → validate-pending-laptop.** Per `.claude/rules/advisor-orchestrator.md` §5.2 + DQ #229 (Shape G suspended repo-wide until 2026-06-01). After commit + push, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `answered_by: null`), `phase_task: 4`, `branch: <your worker branch>`, `commands:` the §15.1–15.3 workspace commands verbatim:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e`
   (the e2e *run* — §15.4, needs Docker/testcontainers — is a SEPARATE advisor-driven Phase-2 step under user gate 4; do NOT attempt the e2e run yourself). Compute `next_id` across `.claude/decision-queue.json` pending+resolved + any `.claude/decision-queue-archive-*.json` (max+1). Commit the DQ entry + push to your worker branch immediately (mid-task visibility — `.claude/rules/decision-queue.md`).
7. **Commit message (verbatim from plan §13 VALIDATE):** `test(e2e): assert AGPL §13 disclosure surface via /api/v4/site + /api/v4/source (task 4)`.
8. **DQ attribution:** `from: "impl"` only. NEVER `answered_by: "advisor"` / `"user"`. NEVER `kind: "clarify"` / `"validate-result"` / `"validate-failed"`. Per `.claude/rules/decision-queue.md` hard refusals.
9. **Mandatory post-task retro** before exit (`.claude/rules/post-task-retro.md`): `memory_write_eval`, `source_ref` = your exact branch name (the Stop hook on `junior/*` branches requires `source_ref` = branch + a fresh `Task retro:` row).

## 5. §2.4 mandatory-lesson firing record (advisor audit)

Authored under `.claude/rules/advisor-orchestrator.md` §2.4. File list = `crates/server/tests/e2e.rs` (1 edit). Table matches fired:

- `crates/server/tests/e2e.rs` (any edit) → `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` → §3 items 4, 5.
- Plan §13 Task 4 GOTCHA 3 explicitly binds the e2e-edit-hang discipline (single edit, but the discipline is plan-bound regardless of the ≥2 threshold) → `feedback_junior_worker_e2e_edit_hang.md` → §3 item 6.
- §2.3 PMD hybrid presearch: ran (`e2e` / multi-word, lesson tag) — returned nothing; lane PMD DB is the per-worktree DB (lesson corpus indexed in canonical DB only; known lane-DB-isolation issue, same root as the Stop-hook gap). Non-blocking: §2.4 mechanical injection is the load-bearing path; lessons read from disk at `.claude/lessons/` regardless of index. Canonical-schema-first satisfied by plan §10.6 having pre-resolved Case A + the sibling line range cited.
