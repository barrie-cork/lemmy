# Handover: v1-quality-r3c advisor session — 2026-05-31

**Author:** advisor (canonical brehon-fork, governance-v0 session)
**Date:** 2026-05-31 ~17:35 UTC

---

## RESUME

**Phase:** v1-quality-r3c  
**Stage:** T1 complete (advisor-direct), ready to dispatch T2  
**Phase branch:** `phase-v1-quality-r3c` @ `35008c74f`  
**governance-v0:** `c62230362`  
**Lane mode:** Mode B (no dedicated worktree)

---

## What shipped this session

| Commit | Branch | Description |
|---|---|---|
| `baaf62fbc` | governance-v0 | chore(advisor): clarify v1-quality-r3c-planning-1 — DQ a3d0e9941441-043 |
| `07cada651` | governance-v0 | docs(plan): v1-quality-r3c plan (advisor-authored; planning #548 cancelled after 80+ min) |
| `7c4fc33ed` | governance-v0 | chore(advisor): v1-quality-r3c bm-cut brief |
| `c62230362` | governance-v0 | chore(advisor): v1-quality-r3c impl-1 brief |
| `ec5d803c4` | phase-v1-quality-r3c | bm-cut (phase branch created by BM #554) — note: DQ entry d669291aa4f2-001 raised by BM re runlog commit target |
| `1ff19e5e6` | phase-v1-quality-r3c | Mode B sync: impl-1 brief pulled from governance-v0 |
| `35008c74f` | phase-v1-quality-r3c | fix(coderabbit): scope endpoint-count rule to v0 era — fixes Issue #165 (**T1 DONE, advisor-direct**) |

---

## Current state

**T1 (Issue #165 — .coderabbit.yaml):** DONE. `grep -c 'EXACTLY 11' .coderabbit.yaml` = 0 on phase branch. Committed directly by advisor after impl-task #556 reported success but failed post-condition (worker hallucinated "DTO impels" anchor, got stuck in retry loop, exited without making the edit).

**T2 (Issue #166 — sponsor-allowlist sweep):** NOT STARTED. Brief not yet authored.

**T3 (BREHON_DISABLE_* coverage):** NOT STARTED. Brief not yet authored.

**Open DQ:** BM task #554 raised DQ `d669291aa4f2-001` — "runlog commit target conflict". Check `.claude/decision-queue.json` on phase branch for details before dispatching T2. May be self-resolved (BM wrote runlog commit to governance-v0 instead of phase branch — check runlog exists on governance-v0).

---

## Next concrete actions (in order)

1. **Check DQ d669291aa4f2-001** — read it from the phase branch DQ. Likely advisory; resolve if self-answerable.

2. **Author T2 impl brief** at `.claude/PRPs/briefs/v1-quality-r3c-impl-2.md`:
   - Base: `phase-v1-quality-r3c`
   - Task: extend Phase A array in `crates/server/tests/e2e.rs` with 2 sponsor-allowlist entries
   - Mandatory lessons: `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`, `feedback_rate_limit_debug_config_post_bucket.md`
   - Unique anchor (confirmed count=1): the block ending `"/api/v4/governance/admin/reputation-stats",` + `];`
   - DO NOT add second set_config — already present at e2e.rs:4135-4146
   - Commit subject: `test(e2e): add sponsor-allowlist routes to HTTP-path sweep (Issue #166)`

3. **Mode B sync** — pull brief onto phase branch via surgical `git checkout FETCH_HEAD -- <file>` (same pattern as T1 brief sync above).

4. **Dispatch T2** via `mcp__junior-brehon__create_task` with `base_branch=phase-v1-quality-r3c`. Check running worker count first — cap is 2 total.

5. **After T2 done + post-condition pass**: author T3 brief, sync, dispatch.

6. **After T3 done**: validate-pending-laptop e2e run (local `cargo-test.bat --workspace --test e2e --features full`), then bm-pr, CR triage, bm-merge.

---

## Key facts for T2 brief authoring

From plan §13 Task 2 + pre-populated research:

- Function: `all_mvp_endpoints_return_non_404` at e2e.rs ~line 4081
- Rate-limit bump: already at lines 4135-4146 — DO NOT add second set_config
- Unique anchor for edit (occurrence count confirmed = 1):
  ```rust
      (
        "GET",
        "/api/v4/governance/admin/reputation-stats",
        "",
        &[200, 400, 401],
      ),
    ];
  ```
- Insert BEFORE the `];` closing line:
  ```rust
      (
        "POST",
        "/api/v4/governance/admin/sponsor-allowlist/add",
        r#"{"actor_id":1}"#,
        &[200, 400, 401, 403, 422],
      ),
      (
        "POST",
        "/api/v4/governance/admin/sponsor-allowlist/remove",
        r#"{"actor_id":1}"#,
        &[200, 400, 401, 403, 422],
      ),
  ```
- Routes registered at `crates/api/routes/src/lib.rs:526-528`

## Key facts for T3 brief authoring

- Mirror: `crates/server/tests/e2e.rs:17592-17731` (PARTICIPATION_JOB pattern)
- Insertion anchor (confirmed count=1): `sponsor_allowlist row must be absent after remove`
- SNAPSHOT_JOB: calls `reputation_snapshot::run_snapshot_batch(&context)` — assert no new reputation_snapshot rows
- FED_REPLAY_CLEANUP: guard in scheduler CLOSURE not in `delete_older_than`; two-probe shape per DQ a3d0e9941441-043
- `FederationInboxNonceInsertForm`: fields `peer_instance: String, activity_id: String` (db_schema/src/source/governance/federation_inbox_nonce.rs:30)
- `delete_older_than(window_days: i64, conn: &mut AsyncPgConnection)` at :37

---

## Incidents to flag at T4 retro

1. Planning #548 ran 80+ min without writing plan (anchor verification loop anti-pattern)
2. impl-task #556 hallucinated "DTO impels" anchor, reported success without making the edit
3. 4 concurrent RT-r5 workers violated new 2-worker cap (advisor deferred rather than blocking)

---

## ASSUMES / VERIFY on resume

- **VERIFY:** `git show origin/phase-v1-quality-r3c:.coderabbit.yaml | grep -c 'EXACTLY 11'` = 0
- **VERIFY:** DQ d669291aa4f2-001 status (resolved or still pending)
- **VERIFY:** running worker count < 2 before dispatching T2
- **VERIFY:** daemon-local governance-v0 ref is synced before dispatching
