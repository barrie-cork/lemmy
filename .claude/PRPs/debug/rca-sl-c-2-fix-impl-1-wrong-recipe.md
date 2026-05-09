# Root Cause Analysis — v1-SL-c-2 fix-impl-1 wrong recipe (cycle 2 escalation)

**Issue**: Workflow 25595869651 (cargo-validate-workspace on Junior #158's
fix-impl-1 commit `4be7b7f95`) failed with 11 × E0277 errors at helper-call
`?`-propagation sites within the test fn `grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries`.

**Root Cause**: The advisor-authored brief at
`.claude/PRPs/briefs/sl-c-2-fix-impl-1.md` (lines 42-49, 64-72, 144-149)
prescribed the **inverse** of the §G4 canonical recipe — flipping the test
fn signature from `Result<(), Box<dyn Error>>` to `LemmyResult<()>` AND
explicitly forbidding `.map_err` bridges at the helper call sites — instead
of the documented recipe at `.claude/rules/advisor-orchestrator.md:215-216`
(test fn stays `Box<dyn Error>`, add `.map_err(|e| format!("{e}").into())`
at each Lemmy-native call). Junior #158 followed the brief faithfully; the
fix is wrong because the brief was wrong.

**Severity**: Medium (cost: 1 fix-impl + 1 ci-watcher + ~12 min GH-Actions wall-clock; not load-bearing on shipped governance code).

**Confidence**: High — chain is fully evidenced; symptom matches exactly; canonical recipe is unambiguous in source.

---

## Evidence Chain

WHY: Workflow 25595869651 fails with 11 × E0277 (Send/Sync/Sized bounds).
↓ BECAUSE: Test fn now returns `LemmyResult<()>` (= `Result<(), LemmyError>`,
  `LemmyError: Send + Sync + 'static`), but the 4 helpers in the same mod
  (`count_log_entries`, `read_log_payload`, `seed_pending_case`,
  `seed_active_surety`) still return `Result<X, Box<dyn Error>>`. `Box<dyn
  Error>` (without `+ Send + Sync`) cannot be `?`-converted into
  `LemmyResult` because rustc cannot prove `dyn Error: Send + Sync` from
  the bare trait object.
  Evidence: `gh run view 25595869651 --repo barrie-cork/lemmy --log-failed`
  → 11 × E0277 at `crates/server/tests/e2e.rs:12069, 12070, 12071, 12106,
  12111, 12116, 12121, 12122` — all `.await?` on helper-fn calls returning
  `Result<_, Box<dyn Error>>`.

WHY: Test fn signature is `LemmyResult<()>` while helpers stay `Box<dyn Error>`.
↓ BECAUSE: Junior #158 commit `4be7b7f95` is a 1-line diff at `e2e.rs:12042`
  changing only the test fn return type — exactly what the brief prescribed.
  The brief at lines 144-145 explicitly forbade touching helpers
  ("Do NOT touch the helpers (seed_pending_case, seed_active_surety). They
  compile.").
  Evidence: `git show 4be7b7f95` → 1 file changed, 1 insertion, 1 deletion
  on `crates/server/tests/e2e.rs:12042`.

WHY: Brief prescribed inverting the test fn signature instead of `.map_err`.
↓ BECAUSE: Brief author misdiagnosed the original DQ #164 fail at
  `e2e.rs:12049` as "test fn signature is wrong" instead of "Lemmy-native
  call sites need `.map_err` bridges". Brief cited the canonical lesson
  (`feedback_lemmy_error_no_std_error.md`) at line 95 but did NOT apply
  its recipe; instead, brief lines 42-49 + 64-72 prescribed the signature
  flip and brief lines 144-149 explicitly hard-refused `.map_err`.
  Evidence: brief lines 42-43 ("Change the test fn return type from
  `Result<(), Box<dyn Error>>` to `lemmy_utils::error::LemmyResult<()>`");
  brief lines 95-96 ("the underlying lesson that was missed in plan §13
  Task 1"); brief lines 144-145 ("Do NOT touch the helpers ... They
  compile.") — NOTE: this contradicts the canonical recipe.

WHY: Brief author misread the cited "sibling pattern" at `e2e.rs:8009-8011`.
↓ BECAUSE: The sibling fn `admin_assign_jury_severity_tier_regular_minor_panel_5_jurors`
  does return `LemmyResult<()>`, but every helper call in its body returns
  `LemmyResult<...>` directly — except `v1_jm_b_fixtures::seed_case` (line
  8030, returns `Result<X, Box<dyn Error>>`) which is bridged with
  `.map_err(|e| anyhow::anyhow!("seed_case: {e}"))?`. The sibling actually
  DEMONSTRATES the canonical bridge pattern — the brief author looked only
  at the signature line and missed the call-site bridges in the body.
  Evidence: `e2e.rs:8030` — `v1_jm_b_fixtures::seed_case(...).await
  .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;`.

WHY: Brief author wrote a brief that contradicted its own cited lesson.
↓ ROOT CAUSE: The advisor session, when authoring fix-impl-1 brief, did
  not verify the prescribed recipe against the canonical §G4 classifier
  table row at `.claude/rules/advisor-orchestrator.md:215-216`. The
  canonical recipe is unambiguous:

    "wrap the call with `.map_err(|e| format!("{e}").into())` per the
     lesson; **verify the test fn signature is `Result<(), Box<dyn Error>>`**"

  The brief inverted both halves of this recipe. This is a brief-authorship
  verification lapse; the recipe text exists in `.claude/rules/` already
  and was loaded into the advisor's session context at session start.

  Evidence:
  - `.claude/PRPs/briefs/sl-c-2-fix-impl-1.md:42-49, 64-72, 144-149` (the
    wrong recipe).
  - `.claude/rules/advisor-orchestrator.md:215-216` (canonical recipe).
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md:8-12` (lesson
    confirming the canonical recipe).
  - Junior #158 commit `4be7b7f95` (literal brief execution).

---

## Git History

- **Original Task 1 commit (c2761b284)**: 2026-05-08 22:26 UTC, solo-dev — introduced 4 helpers and test fn, all returning `Result<X, Box<dyn Error>>`. Plan §13 Task 1 brief lacked the canonical lesson injection (planner-side miss; brief admits this at line 95-96).
- **DQ #164** (`5baa9a54f`): 2026-05-08 22:33 UTC — first failure, original E0277 at `e2e.rs:12049` (`bootstrap()` site).
- **Brief authored (sl-c-2-fix-impl-1.md)**: 2026-05-09 today — advisor-session brief.
- **Junior #158 fix (4be7b7f95)**: 2026-05-09 07:54 UTC — 1-line change per brief.
- **DQ #165** (`3fdb666a4`): 2026-05-09 — second failure, 11 × E0277 at helper-call sites.
- **ci-watcher #159 mutation (0dc170d5d)**: 2026-05-09 — DQ #165 marked `result: "fail"` on its own worker branch (NOT yet finalize-merged into impl-1 branch).
- **Type**: advisor brief-authorship lapse on a documented mechanical recipe. Not a regression. Not an upstream Lemmy issue. Not a Junior worker lapse.

---

## Fix Specification

### What Needs to Change

Reverse Junior #158's commit `4be7b7f95` (test fn signature back to
`Result<(), Box<dyn Error>>`), then add `.map_err(|e| format!("{e}").into())`
bridges at the 4 Lemmy-native call sites inside the test fn body.

The 4 call sites (per worker branch tip
`origin/junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154`):

| Line | Call | Current return | Bridge needed |
|---|---|---|---|
| ~12049 | `governance_fixtures::bootstrap().await?` | `LemmyResult<...>` | `.map_err(\|e\| format!("{e}").into())` before `?` |
| ~12051 | `Instance::read_or_create(&mut context.pool(), "test.invalid").await?` | `LemmyResult<Instance>` | `.map_err(\|e\| format!("{e}").into())` before `?` |
| ~12055-12057 | 3 × `governance_fixtures::seed_user(...).await?` | `LemmyResult<(PersonId, _)>` | `.map_err(\|e\| format!("{e}").into())` on each |
| ~12075 | `run_grace_check_batch(&context).await?` | `LemmyResult<...>` | `.map_err(\|e\| format!("{e}").into())` before `?` |

The 4 helper calls (`seed_pending_case`, `seed_active_surety`,
`count_log_entries`, `read_log_payload`) need NO change — they already
return `Result<_, Box<dyn Error>>` which `?`-propagates cleanly into
`Result<(), Box<dyn Error>>`.

The 3 internal `AsyncPgConnection::establish` and Diesel calls also need
no change — Diesel errors implement `std::error::Error` natively.

### Implementation Guidance

```rust
// Current (Junior #158's commit 4be7b7f95 — WRONG):
async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
) -> lemmy_utils::error::LemmyResult<()> {
    // ... helpers fail to ?-propagate
}

// Required (canonical §G4 recipe):
async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
) -> Result<(), Box<dyn Error>> {
    // ... bridge Lemmy-native calls only
    let (_container, context, db_url) = governance_fixtures::bootstrap()
        .await
        .map_err(|e| format!("{e}").into())?;
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid")
        .await
        .map_err(|e| format!("{e}").into())?;
    let (sponsee, _) = governance_fixtures::seed_user(&context, instance.id, "slc1_sponsee", false)
        .await
        .map_err(|e| format!("{e}").into())?;
    // ... etc for the 5 Lemmy-native sites
}
```

The `?` on each `seed_pending_case().await?`, `seed_active_surety().await?`,
`count_log_entries().await?`, `read_log_payload().await?`, and the Diesel
chains all keep working unchanged.

### Files to Modify

- `crates/server/tests/e2e.rs` (single file) on the existing worker branch
  `origin/junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154`.
  - **Revert** line 12042: `LemmyResult<()>` → `Result<(), Box<dyn Error>>`.
  - **Add** `.map_err(|e| format!("{e}").into())` at the 5 Lemmy-native call
    sites (the helpers stay untouched — that part of the original brief
    was right, just for the wrong reason).

Estimated diff: ~7 line additions (1 reverted signature + 5 `.map_err`
bridges, lined up). Single mechanical edit. Mechanical match for the §G4
allowlist, not judgment-heavy.

### Verification

1. Push the corrected fix-impl-2 commit to the existing worker branch
   (so daemon finalize-merge picks up impl-1 + fix-impl-1 + fix-impl-2 as
   one unit).
2. Capture new `cargo-validate-workspace` workflow run id; raise a fresh
   `kind: "validate-pending"` DQ entry (next id = 166 or beyond) with the
   captured run id.
3. ci-watcher polls; expects `conclusion: success` (no E0277 errors at
   any site within `mod v1_sl_c_fixtures`).
4. To reproduce the original symptom locally, on a worker-branch checkout:
   ```
   cargo build -p lemmy_server --tests --features full 2>&1 | grep -c E0277
   # Pre-fix: 11; post-fix: 0
   ```

---

## Notes for retro

These belong in the v1-SL-c-2 retro (when authored), not this RCA:

1. **Brief authorship lapse on mechanical recipe.** The §G4 classifier
   table row 4 (E0277 LemmyError) is canonical and unambiguous. The advisor
   session loaded `advisor-orchestrator.md` at session start and still
   prescribed the inverse recipe. Possible mitigations:
   - When authoring a fix-impl brief whose triggering DQ failure cites a
     §G4 allowlist class, COPY-PASTE the canonical recipe text from the
     classifier table row VERBATIM into the brief Scope, before adding
     specific file:line context. This eliminates the "I read the row but
     prescribed something different" failure mode.
   - When citing a "sibling pattern", walk the function body to verify
     the call sites actually demonstrate the recipe — don't stop at the
     signature line.

2. **§G4 cycle-2 escalation policy worked correctly.** The advisor surfaced
   to user instead of auto-queueing fix-impl-2; this is the correct
   per-rule behavior (`auto-phase.md` Phase 5 "do NOT auto-retry"). Cost
   was 1 wasted fix-impl + 1 wasted ci-watcher + ~12 min GH-Actions
   wall-clock; the alternative (auto-fix-impl-3 with another wrong recipe)
   would have been worse.

3. **Daemon finalize-merge skip on intra-cohort base-branch.** ci-watcher
   #159's mutation landed on its own worker branch (0dc170d5d) but was
   NOT finalize-merged into the impl-1 worker branch (still 3fdb666a4).
   Consistent with `feedback_junior_daemon_finalize_skips_when_worker_pre_pushes.md`
   pattern. Note for retro; not blocking this RCA's fix path.

4. **The §G4 file-class table addition (2026-05-09 to advisor-orchestrator.md
   line 100+) would have caught the planner-side miss on Task 1**, but
   it was authored AFTER Task 1's brief was queued — so it didn't fire on
   the original c2761b284. Tasks 2-5 of v1-SL-c-2 will benefit from it
   forward-going. This is the lesson that documented its own emergence.
