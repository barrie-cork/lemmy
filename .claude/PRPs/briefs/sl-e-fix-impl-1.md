---
phase: v1-SL-e
role: impl-task
task: fix-impl-1
brief_n: 8
authored: 2026-05-13
---

# [role:impl-task] v1-SL-e fix-impl-1 — RAII env-var guard + sponsor naming + grace-window timing + revoked_at bounds (5 findings on e2e.rs) — see .claude/PRPs/briefs/sl-e-fix-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-e fix-impl-1 — RAII env-var guard + sponsor naming + grace-window timing + revoked_at bounds (5 CR/Copilot findings on e2e.rs)`

## §2 Scope

Address **5 CR triage-approved fix-in-pr findings** from PR #127 review, all on `crates/server/tests/e2e.rs` inside `mod v1_sl_e_fixtures`. Single file, multiple anchor-Edits.

**The 5 findings (per `.claude/PRPs/reviews/pr-127-findings.yaml` on phase-v1-SL-e tip `e967c8959`):**

### Finding 1 — cr-5 + copilot-1 (paired): RAII env-var restore guard

- **Severity:** major (cr-5) + medium (copilot-1)
- **CR text (cr-5, e2e.rs:14017):** *"Restore BREHON_DISABLE_GRACE_CHECK_JOB on failure paths"*
- **Copilot text (copilot-1, e2e.rs:14014):** *"Env var BREHON_DISABLE_GRACE_CHECK_JOB restored only at end of test; assertion panic mid-test leaves it set, affecting subsequent tests. Suggest RAII guard."*
- **Pattern:** all 3 SL-e test fns use the same shape:
  ```rust
  let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
  unsafe { std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1"); }
  // ... test body with .await? and assert!() that can panic ...
  unsafe {
    match prev_disable {
      Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
      None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
    }
  }
  ```
  If any `?` returns Err mid-test or `assert!` panics, the trailing restore block is skipped. Env var stays set; subsequent tests in same process see `BREHON_DISABLE_GRACE_CHECK_JOB=1` and don't exercise grace-check job paths.
- **Callsites (enumerated per `feedback_fix_impl_enumerate_all_callsites.md`):**
  - `revocation_during_window_escapes_full_lane` — `prev_disable` at line 14014; restore at lines 14347-14350
  - `window_expiry_fires_full_lane` — `prev_disable` at line 14358; restore at lines 14586-14589
  - `backfill_of_mid_flight_v0_to_v1_deploy` — `prev_disable` at line 14604; restore at lines 14861-14864
- **Fix:** introduce a single `GraceCheckDisableGuard` struct INSIDE `mod v1_sl_e_fixtures` (at top of mod, after the existing helpers, before the first test fn). Implement `Drop` to restore `prev_disable` unconditionally. Replace all 3 `prev_disable` setup+restore blocks with `let _guard = GraceCheckDisableGuard::set("1");`. The guard's `Drop` impl restores on **all** exit paths (Ok, Err, panic, early-return).

  ```rust
  // RAII guard for BREHON_DISABLE_GRACE_CHECK_JOB. Restores prior value on Drop,
  // covering Ok / Err / panic exit paths. Per CR cr-5 + Copilot copilot-1 on PR #127.
  struct GraceCheckDisableGuard {
    prev: Option<std::ffi::OsString>,
  }

  impl GraceCheckDisableGuard {
    fn set(value: &str) -> Self {
      let prev = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
      unsafe { std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", value); }
      Self { prev }
    }
  }

  impl Drop for GraceCheckDisableGuard {
    fn drop(&mut self) {
      unsafe {
        match self.prev.take() {
          Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
          None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
        }
      }
    }
  }
  ```

  Test fn body becomes:
  ```rust
  async fn revocation_during_window_escapes_full_lane() -> LemmyResult<()> {
    let _guard = GraceCheckDisableGuard::set("1");
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    // ... rest of test body, NO restore block at end ...
    Ok(())
  }
  ```

### Finding 2 — copilot-2: sponsor1/sponsors[1] naming

- **Severity:** medium
- **Copilot text (e2e.rs:14045):** *"`sponsor1` is taken from `sponsors[1]` (second sponsor); misleading. Rename to `sponsor2` or use `sponsors[0]`."*
- **Site:** `revocation_during_window_escapes_full_lane` at line 14045:
  ```rust
  let (sponsor1, endo1) = sponsors[1];
  ```
- **Fix:** rename binding to `sponsor2` + `endo2`. Renaming the index to `[0]` would change test semantics (Test #1 specifically revokes sponsor[1]'s endorsement, not sponsor[0]'s — verify by reading the surrounding test body before changing; if `sponsors[1]` is the second-of-two and the test depends on this being the "second sponsor" for some assertion shape, keep the index and just rename the binding).
- Update all downstream references in the same fn body (`sponsor1_view`, `sponsor1.0`, etc).

### Finding 3 — copilot-3: 5s grace tolerance flake risk

- **Severity:** medium
- **Copilot text (e2e.rs:14106):** *"5s tolerance for `expected_grace` may be flaky under slow CI; `before_decisive` captured before full drive_jury_to_quorum flow."*
- **Site:** `revocation_during_window_escapes_full_lane`. The `before_decisive = Utc::now()` at line 14079 is captured BEFORE `drive_jury_to_quorum` (a multi-second async flow with 5 juror votes). The grace_expires_at assertion at lines 14103-14107:
  ```rust
  let expected_grace = before_decisive + Duration::hours(168);
  assert!(
    (grace - expected_grace).num_seconds().abs() < 5,
    "grace_expires_at ≈ now + 168h (within 5s), got {grace:?}"
  );
  ```
  Under slow CI, `drive_jury_to_quorum` could easily take >5s, and grace_expires_at is computed against the post-decisive time. The 5s tolerance is then too tight.
- **Fix:** capture `before_decisive` AND `after_decisive`. Use the range as bounds for grace_expires_at:
  ```rust
  let before_decisive = Utc::now();
  drive_jury_to_quorum(&context, &federation_context, admin_view, case_id, JuryDecision::SuspendCommunityMember).await?;
  let after_decisive = Utc::now();
  // ... later:
  let grace_lower = before_decisive + Duration::hours(168);
  let grace_upper = after_decisive + Duration::hours(168) + Duration::seconds(1);
  assert!(
    grace >= grace_lower && grace <= grace_upper,
    "grace_expires_at in [before_decisive + 168h, after_decisive + 168h + 1s], got {grace:?}"
  );
  ```
  This is tight-but-non-flaky regardless of CI speed.

### Finding 4 — copilot-4: revoked_at bounded check

- **Severity:** medium
- **Copilot text (e2e.rs:14204):** *"`recent` check on revoked_at only asserts not-in-future; would pass for very old timestamp. Use bounded window (t_start..Utc::now())."*
- **Site:** `revocation_during_window_escapes_full_lane` at lines 14201-14204:
  ```rust
  assert!(
    revoke_resp.revoked_at < Utc::now() + Duration::seconds(1),
    "revoked_at is recent"
  );
  ```
- **Fix:** introduce a `t_start = Utc::now()` capture **immediately before** the `revoke_endorsement` call (line 14185 area), then assert `t_start <= revoked_at <= Utc::now()`:
  ```rust
  let t_revoke_start = Utc::now();
  let revoke_resp = revoke_endorsement(/* ... */).await?.into_inner();
  let t_revoke_end = Utc::now();
  // ...
  assert!(
    revoke_resp.revoked_at >= t_revoke_start && revoke_resp.revoked_at <= t_revoke_end + Duration::seconds(1),
    "revoked_at in [t_revoke_start, now+1s], got {:?}",
    revoke_resp.revoked_at
  );
  ```

## §3 Required reading

**Mandatory file-class lessons (per `.claude/rules/advisor-orchestrator.md` §2.4 file-class table):**

1. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A canonical** throughout. `LemmyResult<()>` test fn. NO `Box<dyn Error>`, NO `.map_err`. Existing SL-e tests are Case A — preserve.
2. `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixture context pattern.
3. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **MANDATORY anchor-based Edit, never full-file Read.** e2e.rs is **14,869 lines** on phase-v1-SL-e tip. Each Edit uses a unique 5-10 line anchor.
4. `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — callsites for the RAII guard fix are enumerated above (3 sites in §2 Finding 1).
5. `.claude/lessons/feedback_clippy_test_style.md` — Lemmy workspace test-style for clippy denies.

**Plan + review sections:**

6. `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md` §13 Task 1 (lines ~1730-1872) — Test #1 IMPLEMENT spec, for context on what semantic invariants the existing assertions must preserve.
7. `.claude/PRPs/reviews/pr-127-findings.yaml` — the source-of-truth findings file on phase-v1-SL-e tip. Find ids cr-5, copilot-1, copilot-2, copilot-3, copilot-4.
8. `.claude/rules/decision-queue.md` — DQ schema for validate-pending raise + Recipe 1 if blocker found.

**Cargo wrapper rules:**

9. `.claude/rules/cargo-output-capture.md` — capture to file, don't pipe to tail.
10. `.claude/rules/no-cargo-output-paste.md` — last 20 lines only into conversation if reading a captured log.

## §3a Handover from prior task

- **Task 4 (job-253):** v1-SL-e retro authored at `.claude/PRPs/reports/v1-SL-e-retro.md` (commit `259f5db5a`). Phase tip then advanced through bm-pr (#254), bm-poll-cr (#255), bm-triage (#256) verbs.
- **phase-v1-SL-e tip:** `e967c8959` (BM merge of triage task #256).
- **PR #127 CR triage approved by user 2026-05-13:** 7 fix-in-pr + 2 rebut. This brief addresses 5 of the 7 fix-in-pr (all e2e.rs). The remaining 2 fix-in-pr (cr-2, cr-3 on `.claude/decision-queue.json`) ship under `sl-e-fix-impl-2`.
- **e2e.rs structure on phase-v1-SL-e tip (confirmed via grep):**
  - Line 12205-12303: SL-c-2 test `prev_disable` patterns (DO NOT TOUCH — these are out-of-scope SL-c-2 tests).
  - Line 12334-12973: 4 more SL-c-2 pattern sites (DO NOT TOUCH).
  - Lines 14013-14350: `revocation_during_window_escapes_full_lane` (Task 1 — fix here).
  - Lines 14357-14589: `window_expiry_fires_full_lane` (Task 2 — fix `prev_disable` only).
  - Lines 14596-14864: `backfill_of_mid_flight_v0_to_v1_deploy` (Task 3 — fix `prev_disable` only).
- **Mod v1_sl_e_fixtures is closed** at line ~14869. Insert `GraceCheckDisableGuard` struct + impl AFTER the existing helpers and BEFORE the first SL-e test fn (around line ~14005-14010 area; verify via local grep).
- **Migrations on phase-v1-SL-e:** 12 post-JM-a migrations (SL-b + JM-d + JM-a + RT-r1). e2e.rs:1585 `limit(12)` is correct.

## §4 Constraints

- **Touch only:** `crates/server/tests/e2e.rs` + `.claude/decision-queue.json` (validate-pending raise) + `.claude/PRPs/reviews/pr-127-findings.yaml` (mark `addressed_in: <commit-sha>`, `bucket: done`).
- **Anchor-Edit only on e2e.rs.** e2e.rs is 14,869 lines on phase-v1-SL-e tip. NEVER full-file Read. Use unique 5-10 line anchors around each edit site. **No `replace_all`** — each `prev_disable` site has distinct surrounding context.
- **DO NOT touch SL-c-2 test sites** (e2e.rs lines 12200-13000 region). Those use the same `prev_disable` pattern but are out-of-scope SL-c-2 tests. cr-5/copilot-1 cite lines 14014/14017 only.
- **GraceCheckDisableGuard scope:** define INSIDE `mod v1_sl_e_fixtures`, NOT at file scope or in a sibling mod. Other phases (SL-c-2 etc) keep their own `prev_disable` blocks unchanged.
- **Case A canonical**: `LemmyResult<()>` test fn outer preserved. No `Box<dyn Error>`, no `.map_err`.
- **Semantic preservation**: rename `sponsor1` → `sponsor2` (NOT change `sponsors[1]` to `sponsors[0]` — test #1 depends on indexing the second seeded sponsor; preserve the data flow, fix only the binding name).
- **Watchpoint #7**: at task-end, run `git diff governance-v0..HEAD --stat`. EXPECT: one (or two) files: `crates/server/tests/e2e.rs | +N -M`, optionally `.claude/PRPs/reviews/pr-127-findings.yaml | +5 -5` (5 findings flipped to `done`). NO `crates/`/`migrations/`/`docs/` changes elsewhere.
- **Update findings YAML**: after fix commit, set `bucket: done` + `addressed_in: <commit-sha>` for cr-5, copilot-1, copilot-2, copilot-3, copilot-4. Regenerate `counters` block (major.open: 3→2; medium.open: 4→1; major.done: 0→1; medium.done: 0→4).
- **Commit message:** `test(v1-SL-e): RAII env-var guard + sponsor naming + grace timing + revoked_at bounds (5 CR/Copilot findings on PR #127)`. Footer: `Addresses CR cr-5; Copilot copilot-1, copilot-2, copilot-3, copilot-4 on PR #127.`
- **Shape G:** after committing, push worker branch; capture `gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId`. Write `kind: "validate-pending"` DQ entry with that `workflow_run_id`, `branch`, `phase_task: "fix-impl-1"` (string, not number — this is a fix task, not a §13 task). `from: "impl"`, `answered_by: null`, `result`/`log_slice`/`failed_jobs` all null. **Atomic raise**: `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json` AND `origin/phase-v1-RT-r1` (RT-r1 lane has higher ids — RT-r1's max as of brief authorship is 212). Next id for this brief's validate-pending raise: **213**.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
- **DO NOT post a PR comment.** This task only commits fix code + raises validate-pending. Posting CR replies is a separate advisor-side step (user gate 3 already cleared; post happens after fix-impl-1 + fix-impl-2 both land).
