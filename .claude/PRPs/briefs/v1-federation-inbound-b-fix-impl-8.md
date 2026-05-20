---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 8
authored: 2026-05-20
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: null  # CR findings on PR #139, not a DQ entry
triggering_task: "PR #139 CR triage (gate-3 APPROVED 2026-05-20)"
classification: "Three CR fix-in-pr findings on PR #139 (CodeRabbit). All three are in-PR quality-bar improvements with concrete recipes posted in the CR comments; gate-3 approved 2026-05-20 with the four-bucket draft `3 fix-in-pr / 3 carry-forward / 1 done / 12 wont-fix`. (1) `scheduled_tasks.rs:447` — clamp `replay_window_days` to `>= 1` before passing to `federation_inbox_nonce::delete_older_than`; config <= 0 would nullify replay protection (security/correctness). (2) `e2e.rs:15620` — assert pre-limit `ActivityTrait::receive` calls in the rate-limit seed loop return `Ok(())` so the 429 test specifically validates the configured cap, not a coincidental unrelated failure. (3) `e2e.rs:15704` — change `assert!(log_count >= 1)` to `assert_eq!(log_count, 1)` to catch duplicate-emission regressions (fresh DB processes exactly one label). All three are mechanical recipe applications — §G4-allowlist-equivalent canonical-recipe-mirror class (CR posted committable suggestions verbatim)."
base: "phase-v1-federation-inbound-b @ de594992f (post-bm-pr Junior #349; PR #139 OPEN). The 3 sites all exist on this tip: scheduled_tasks.rs:447 (Task 7 ship), e2e.rs:15620 (Task 9 fixtures), e2e.rs:15704 (Task 9 fixtures)."
cap: "EXACTLY 3 hunks (≤25 lines net), 2 files ONLY: crates/routes/src/utils/scheduled_tasks.rs (Hunk-1) + crates/server/tests/e2e.rs (Hunk-2 + Hunk-3). NEVER touch any other file. NEVER touch the Phase-6 outer fixture or the 4 other v1_federation_inbound_b_fixtures tests outside the 2 named sites. NEVER touch the production rate-gate code in inbox.rs (out of scope — copilot-1/2/3 are carry-forward, NOT bundled here). NEVER add `#[allow(...)]` or `#[expect(...)]`. A 4th hunk or any other-file edit → STOP + kind:blocker."
serial: "Single-task fix bundle for CR fix-in-pr findings. Strictly serial cap=1 — only in-flight Junior for this lane. PR #139 is OPEN and the fix-impl-8 commits push onto phase-v1-federation-inbound-b which auto-updates PR #139 (rebasing CR review). No re-validation of Phase-2 e2e required (Phase-2 e2e already PASSED CLEAN on the pre-fix-impl-8 tip d2e002cb0/4efce35c8; fix-impl-8's 3 hunks are local quality-bar improvements without semantic change to the wrap_governance_inbound enforcement path)."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-8 — CR fix-in-pr (cr-1 replay_window_days clamp + cr-2 seed-loop assert + cr-3 label log assert_eq)

> **Provenance:** CodeRabbit review on PR #139 posted 2026-05-20T12:00:05Z; 3 fix-in-pr findings classified by advisor + ratified by user-gate-3 approval 2026-05-20. Findings YAML at `.claude/PRPs/reviews/pr-139-findings.yaml` (gitignored runtime artifact; bucket counters: critical=0/major=2/medium=0/low=2/nit=0 open; the 3 fix-in-pr ones are cr-1 major (scheduled_tasks.rs) + cr-2 major (e2e.rs seed loop) + cr-3 low (e2e.rs assert_eq label count)). All three are mechanical recipe applications with CR-supplied committable suggestions (verbatim in §2.2). No DQ entry was raised because CR review is the gate, not a separate `kind:"validate-pending"` cycle. **§G4-allowlist-equivalent** (three byte-identical canonical-recipe mirrors, zero design ambiguity).

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `de594992f` or newer — accept any tip on `phase-v1-federation-inbound-b` that contains commit `1153d4b62` which finalize-merged Task 9). `git merge-base --is-ancestor 1153d4b62 HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ into `.claude/decision-queue.json` (or the harness-gap escalation file per §4 if the gate blocks).
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>` unless dispatch carries `(user-authorised forbidden-window override per DQ #<id>)`. Shape G SUSPENDED (cargo on laptop) — keep the check.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root. `crates/email/translations` is NOT auto-initialized in a fresh worktree; without it cargo fails pre-existing. Infra, not the fix.
- Confirm Hunk-1 anchor present (`scheduled_tasks.rs` replay_window_days assignment):

  ```bash
  grep -n "window_days_i64 = lemmy_api::governance::config::get_int" crates/routes/src/utils/scheduled_tasks.rs
  grep -n "federation.inbound.replay_window_days" crates/routes/src/utils/scheduled_tasks.rs
  ```

  Expected: ~1 line around 432-439 with the `let window_days_i64 = lemmy_api::governance::config::get_int(...)` block, and ~1 line with the `"federation.inbound.replay_window_days"` key. If 0 → STOP + `kind:"blocker"` (already fixed, or upstream rebase changed the shape). If multiple → STOP + `kind:"blocker"` (cap-1-occurrence assumption invalid).
- Confirm Hunk-2 anchor present (`e2e.rs` rate-limit seed loop):

  ```bash
  grep -n "let _ = ActivityTrait::receive(activity, &context).await" crates/server/tests/e2e.rs
  grep -n "build_unique_sanction_notice_activity(\"rate-test.test\", i)" crates/server/tests/e2e.rs
  ```

  Expected: ~1 line around 15617-15620 with `let _ = ActivityTrait::receive(activity, &context).await;` inside a `for i in 0..2` loop. If 0 → STOP + `kind:"blocker"` (already fixed by an interleaving change). If multiple `let _ = ActivityTrait::receive` occurrences (the pattern may be reused elsewhere) → STOP + `kind:"blocker"` (the seed-loop site needs a more specific anchor; surface to advisor).
- Confirm Hunk-3 anchor present (`e2e.rs` `assert!(log_count >= 1)`):

  ```bash
  grep -n "assert!(log_count >= 1)" crates/server/tests/e2e.rs
  grep -n "federation_label_received" crates/server/tests/e2e.rs
  ```

  Expected: exactly 1 line around 15704 with `assert!(log_count >= 1);`, and ~1 line near it with `governance_log::entry_kind.eq("federation_label_received")`. If 0 → STOP + `kind:"blocker"`. If multiple `assert!(log_count >= 1)` (pattern reused) → STOP + `kind:"blocker"` (need a more specific anchor).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-8 — CR fix-in-pr 3 findings (cr-1 + cr-2 + cr-3)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-8 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-8.md
```

## §2 Scope

### 2.1 The findings being addressed (the contract — CR PR #139 review, gate-3 approved 2026-05-20)

**Finding cr-1 (Hunk-1):** `crates/routes/src/utils/scheduled_tasks.rs` around line 432-447 (inside the federation_inbox_nonce cleanup tick). Currently reads `window_days_i64` from config and passes it directly to `federation_inbox_nonce::delete_older_than`. If admin misconfigures `federation.inbound.replay_window_days` to `<= 0`, the cleanup deletes nearly all nonce rows, effectively nullifying replay protection. **Fix:** clamp the raw value to `>= 1` before use; emit a warn log if the raw value was non-positive.

CR-posted recipe (verbatim — copy-paste source of truth):

```diff
-        let window_days_i64 = lemmy_api::governance::config::get_int(
+        let raw_window_days_i64 = lemmy_api::governance::config::get_int(
           &mut cache,
           pool,
           lemmy_api::governance::config::Scope::Instance,
           "federation.inbound.replay_window_days",
         )
         .await
         .unwrap_or(7);
+        let window_days_i64 = raw_window_days_i64.max(1);
+        if raw_window_days_i64 < 1 {
+          warn!(
+            "federation_inbox_nonce cleanup: invalid replay_window_days={raw_window_days_i64}; clamped to 1"
+          );
+        }
```

CR URL: <https://github.com/barrie-cork/lemmy/pull/139#discussion_r3273714638>

**Finding cr-2 (Hunk-2):** `crates/server/tests/e2e.rs` around line 15617-15620 (the seed loop in `per_peer_rate_limit_returns_429`). Currently discards the result of the first two `ActivityTrait::receive` calls (`let _ = ActivityTrait::receive(...)`). If those baseline calls fail for an unrelated reason, the test still passes as long as the third call errors — masking real regressions. **Fix:** assert each seed call returns `Ok(())` with a clear failure message.

CR-posted recipe (verbatim — copy-paste source of truth):

```diff
-    for i in 0..2 {
-      let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
-      let _ = ActivityTrait::receive(activity, &context).await;
-    }
+    for i in 0..2 {
+      let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
+      ActivityTrait::receive(activity, &context)
+        .await
+        .map_err(|e| anyhow::anyhow!("seed request {i} should succeed before the cap is hit: {e}"))?;
+    }
```

**IMPORTANT:** the test fn signature in the v1_federation_inbound_b_fixtures module is `LemmyResult<()>`, NOT `Result<(), Box<dyn Error>>` or `anyhow::Result`. Confirm by reading the fn signature at the head of `per_peer_rate_limit_returns_429`. **If the outer is `LemmyResult<()>`**, the CR recipe's `anyhow::anyhow!(...)` will not compile (anyhow::Error doesn't convert to LemmyError). Use this corrected recipe instead (Case-A-discipline per `feedback_lemmy_error_no_std_error.md`):

```rust
for i in 0..2 {
  let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
  ActivityTrait::receive(activity, &context)
    .await
    .map_err(|e| LemmyError::from_error_message(
      e,
      format!("seed request {i} should succeed before the cap is hit"),
    ))?;
}
```

OR even simpler — let the `LemmyError` propagate directly (most idiomatic; matches sibling Case A discipline in the v1_federation_inbound_b_fixtures module):

```rust
for i in 0..2 {
  let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
  ActivityTrait::receive(activity, &context).await?;
}
```

This last form is preferred — it surfaces the real `LemmyError` from `receive()` directly, which fails the test with the actual error message (still actionable). NO `anyhow` dependency required.

CR URL: <https://github.com/barrie-cork/lemmy/pull/139#discussion_r3273714643>

**Finding cr-3 (Hunk-3):** `crates/server/tests/e2e.rs` line ~15704 (inside `moderation_label_handler_persists_and_logs` test, the log-count assertion). Currently `assert!(log_count >= 1)` — allows the test to pass if the handler emits the label-received governance-log entry MORE than once (duplicate emission would be a regression). The test runs against a fresh DB and processes exactly one label, so it should assert exactly one log entry. **Fix:** `assert_eq!(log_count, 1)`.

CR-posted recipe (verbatim — copy-paste source of truth):

```diff
-    assert!(log_count >= 1);
+    assert_eq!(log_count, 1);
```

CR URL: <https://github.com/barrie-cork/lemmy/pull/139#discussion_r3273714657>

### 2.2 The fix (the contract — THREE verbatim recipe applications; copy EXACTLY, do not paraphrase)

**§G4 CANONICAL RECIPE (verbatim mirrors — there is no single allowlist row for "CR fix-in-pr quality-bar finding", so the §G4-allowlist-equivalent here is the CR-posted committable-suggestion mirror in `.claude/rules/advisor-orchestrator.md` §G4 classifier "Canonical-recipe-mirror discipline"):**

> | Failure signature | Auto-fix | Source lesson |
> | Admin-misconfig `<= 0` for a config-controlled retention/window/cap value passed directly to a destructive op (DELETE/TRUNCATE/eviction) | **Clamp the raw value to a positive minimum** (`raw.max(N)` for N = the safe-default floor; typically 1 day / 1 hour / 1 count) before passing; emit a `warn!` log when the raw was below the floor. Rename the variable to `raw_*` and introduce the clamped `*` form. | CR PR #139 cr-1 + canonical-recipe-mirror |
> | `let _ = <Result-returning-call>` inside a test seed loop that establishes preconditions for a later `assert!(<other-Result>.is_err())` | **Assert each seed call returns `Ok(())`** so the later assertion specifically validates the configured failure-mode, not a coincidental unrelated failure. In a LemmyResult<()> outer fn, propagate with bare `?` (mirror Case A); in a Result<(), Box<dyn Error>> outer, use `.map_err` annotated closure (Case B). Match the existing module's outer-Result discipline. | CR PR #139 cr-2 + `feedback_lemmy_error_no_std_error.md` |
> | `assert!(<count> >= 1)` when the test setup deterministically processes exactly N items | **Use `assert_eq!(<count>, N)`** to catch duplicate-emission/over-counting regressions. Pin equality, not lower-bound. | CR PR #139 cr-3 + canonical-recipe-mirror |

**Hunk-1 (replace 1 line near `scheduled_tasks.rs:432` + insert 6 lines after the `.unwrap_or(7);` line):**

CURRENT (verbatim from base tip de594992f, lines ~432-439):

```rust
        let window_days_i64 = lemmy_api::governance::config::get_int(
          &mut cache,
          pool,
          lemmy_api::governance::config::Scope::Instance,
          "federation.inbound.replay_window_days",
        )
        .await
        .unwrap_or(7);
```

REPLACEMENT (the §2.2 contract — apply EXACTLY this; the CR-posted committable suggestion):

```rust
        let raw_window_days_i64 = lemmy_api::governance::config::get_int(
          &mut cache,
          pool,
          lemmy_api::governance::config::Scope::Instance,
          "federation.inbound.replay_window_days",
        )
        .await
        .unwrap_or(7);
        let window_days_i64 = raw_window_days_i64.max(1);
        if raw_window_days_i64 < 1 {
          warn!(
            "federation_inbox_nonce cleanup: invalid replay_window_days={raw_window_days_i64}; clamped to 1"
          );
        }
```

Net diff: -1 line (rename `let window_days_i64` to `let raw_window_days_i64`), +6 lines (clamp + warn block). Net hunk: ~7 lines changed. The downstream `window_days_i64` reference at line ~446 (in `delete_older_than(window_days_i64, ...)`) is UNCHANGED — the clamp binds the new value to the same name.

**Verify `warn!` is in scope:** read the top of `scheduled_tasks.rs` for an existing `use tracing::warn;` or `use log::warn;`. The file's existing code uses `warn!("federation_inbox_nonce_cleanup: previous batch still running, skipping this tick");` and `warn!("Failed federation_inbox_nonce cleanup: {e}")` — `warn!` IS already in scope. Use the existing macro; do NOT add a new import.

**Hunk-2 (replace 1 line in `e2e.rs` rate-limit seed loop, lines ~15617-15620):**

CURRENT (verbatim from base tip de594992f, lines ~15617-15620):

```rust
    for i in 0..2 {
      let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
      let _ = ActivityTrait::receive(activity, &context).await;
    }
```

REPLACEMENT (the §2.2 contract — apply EXACTLY this; bare-`?` propagation per Case A discipline):

```rust
    for i in 0..2 {
      let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
      ActivityTrait::receive(activity, &context).await?;
    }
```

Net diff: 1 line changed (`let _ = ActivityTrait::receive(...);` → `ActivityTrait::receive(...).await?;`). Net hunk: 1 line.

**Why bare-`?` and not CR's `anyhow::anyhow!` form:** the v1_federation_inbound_b_fixtures module's `per_peer_rate_limit_returns_429` test fn returns `LemmyResult<()>` (confirm by reading the fn signature at the top of the test body). `LemmyError` already propagates via `?` directly; `anyhow::anyhow!` would either fail to compile (no From impl) or require adding `anyhow` to dev-deps (out of scope; clippy will warn). Bare `?` is the canonical Case-A pattern per `feedback_lemmy_error_no_std_error.md` and matches every other `?` in the module's test fns. If the seed call DOES fail, the `LemmyError`'s message surfaces directly — that IS the actionable error for a developer reading the test failure.

**Hunk-3 (replace 1 line in `e2e.rs` label-handler test assertion, line ~15704):**

CURRENT (verbatim from base tip de594992f, line ~15704):

```rust
    assert!(log_count >= 1);
```

REPLACEMENT (the §2.2 contract — apply EXACTLY this; the CR-posted committable suggestion):

```rust
    assert_eq!(log_count, 1);
```

Net diff: 1 line changed. Net hunk: 1 line.

Total fix-impl-8 diff: ~9 lines net across 3 hunks in 2 files (scheduled_tasks.rs +6 / e2e.rs +0).

### 2.3 What is NOT in scope (the fence)

- **NEVER touch any file other than `crates/routes/src/utils/scheduled_tasks.rs` (Hunk-1) and `crates/server/tests/e2e.rs` (Hunk-2 + Hunk-3).** No other production code, no migrations, no Cargo.*, no schema.rs, no plan, no lesson, no template.
- **NEVER touch the production rate-gate code path** in `crates/apub/activities/src/governance/inbox.rs` (the in-memory rate-limit maps, the `check_per_actor_rate_limit` override, the `evict_oldest_unreviewed_if_needed` TOCTOU concern). copilot-1/2/3 are CARRY-FORWARD findings (gate-3 ratified); they are NOT bundled into fix-impl-8. File a separate GH issue for the DoS-hardening family per the retro carry-forward.
- **NEVER touch `crates/apub/activities/src/governance/publish_trust_attestation.rs`'s analogous rate-key handling.** Same reasoning (copilot-3 carry-forward).
- **NEVER touch the 12 brief/runlog files** flagged by CR's markdownlint findings (cr-4 through cr-15). Those are gate-3-ratified `wont-fix`: archived advisor artifacts, lint-out-of-scope per `branch-manager.md` file ownership.
- **NEVER touch the 4 OTHER v1_federation_inbound_b_fixtures tests** (allowlisted_happy_path / blocklisted_peer_returns_403 / replayed_activity_returns_409 — and the `moderation_label_handler_persists_and_logs` body OUTSIDE the Hunk-3 single-line assertion change). They all PASS on tip de594992f; leave byte-for-byte.
- **NEVER touch the Phase-6 outer fixture body** (lines ~4960-5240) or the sanction_notice_round_trip post-fixture assertions.
- **NEVER add `#[allow(...)]` or `#[expect(...)]`** anywhere.
- **NEVER add `anyhow` as a dev-dep** to address the cr-2 recipe verbatim — use the bare `?` form per §2.2 Hunk-2 reasoning. The CR recipe's `anyhow::anyhow!` is contextual to the reviewer's Rust idiom; the LemmyResult<()> outer makes bare `?` more idiomatic and dependency-free.

### 2.4 Verification (run BEFORE committing — pre-push cargo-check + cargo-test --no-run discipline per `feedback_fix_impl_pre_push_cargo_check.md` 2026-05-13)

In the worker's worktree, after applying the §2.2 edits, run a LOCAL pre-push validation BEFORE pushing. **Three cmds in sequence; bat wrapper required on Windows per `feedback_windows_e2e_requires_bat_wrapper`; SEPARATE `cmd //c` invocations per `feedback_batch_goto_eof_clobbers_errorlevel` (NEVER bat-&&-bat chain):**

```bash
# 1. cargo check (~30s warm)
bash scripts/brehon/cargo-check.sh --workspace --features full 2>&1 | tail -30

# 2. cargo clippy -D warnings (~1m warm)
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings 2>&1 | tail -30

# 3. cargo test --test e2e --no-run (~2m warm — verify e2e binary LINKS after edit)
bash scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run 2>&1 | tail -30
```

Expected: all three exit 0 with `Finished` line. If any cmd fails:

- **Compile error in Hunk-1** (`raw_window_days_i64` not found because Hunk-1's rename didn't catch the downstream `window_days_i64` use at line ~446; or `warn!` not in scope) → patch in-commit (the §2.2 contract says the downstream name STAYS `window_days_i64` — the new local `let window_days_i64 = raw_window_days_i64.max(1);` rebinds; ensure both lines compile).
- **Compile error in Hunk-2** (`LemmyError` doesn't impl `From` for the inner error type returned by `ActivityTrait::receive`) → this is the Case-C symptom; STOP + `kind:"blocker"`. The §2.2 Hunk-2 reasoning was wrong about the outer Result type; surface to advisor.
- **Clippy warning treated as error** (e.g. unused-import on something Hunk-1 added/removed, or `clippy::manual_range_clamp` on `raw_window_days_i64.max(1)` — note: `.max(1)` is canonical Rust, NOT a clamp; clippy should NOT warn. If clippy DOES warn here, surface the exact warning text and STOP + `kind:"blocker"`) → patch or stop per the routing.
- **Clippy fired `clippy::format_in_format_args` or similar on the new `warn!` line** → adjust the format string (split into separate args), one-shot in-commit if mechanical. The CR-posted recipe uses interpolation `{raw_window_days_i64}` which is allowed in `tracing::warn!`; no warning expected.
- **`cargo test --no-run` fails to LINK e2e binary** (unrelated regression: missing trait, missing import) → STOP + `kind:"blocker"` with the link-error text.

Then grep-verify the §2.2 edits landed cleanly (Note: `grep -c` exits 1 if count is 0, which kills `&&` chains — wrap each with `|| echo "0"` if chaining):

```bash
# Hunk-1: expect exactly 1 occurrence of the new raw_ name
grep -c "let raw_window_days_i64 = lemmy_api::governance::config::get_int" crates/routes/src/utils/scheduled_tasks.rs || echo "0"
# (expected: 1 — Hunk-1's rename)

# Hunk-1: expect exactly 1 occurrence of the clamp expression
grep -c "raw_window_days_i64.max(1)" crates/routes/src/utils/scheduled_tasks.rs || echo "0"
# (expected: 1 — Hunk-1's clamp)

# Hunk-1: expect 0 occurrences of the OLD direct `let window_days_i64 = lemmy_api::governance::config::get_int` form
grep -c "let window_days_i64 = lemmy_api::governance::config::get_int" crates/routes/src/utils/scheduled_tasks.rs || echo "0"
# (expected: 0 — Hunk-1's rename removed the old form)

# Hunk-1: expect exactly 1 occurrence of the warn message
grep -c "invalid replay_window_days" crates/routes/src/utils/scheduled_tasks.rs || echo "0"
# (expected: 1 — Hunk-1's warn body)

# Hunk-2: expect 0 occurrences of the OLD `let _ = ActivityTrait::receive` in the rate-test seed loop area
# (use surrounding context to anchor — the pattern may appear elsewhere; if it does, the cap-1 assumption fails)
grep -c "let _ = ActivityTrait::receive" crates/server/tests/e2e.rs || echo "0"
# (expected: 0 — Hunk-2 removed the only known occurrence; if non-zero, check that other occurrences are intentional)

# Hunk-2: expect at least 1 occurrence of the bare `?` form near the rate-test
grep -c "ActivityTrait::receive(activity, &context).await?;" crates/server/tests/e2e.rs || echo "0"
# (expected: 1 — Hunk-2's bare-? form added)

# Hunk-3: expect 0 occurrences of the OLD assert!(log_count >= 1)
grep -c "assert!(log_count >= 1)" crates/server/tests/e2e.rs || echo "0"
# (expected: 0 — Hunk-3 removed it)

# Hunk-3: expect exactly 1 occurrence of the NEW assert_eq!(log_count, 1)
grep -c "assert_eq!(log_count, 1)" crates/server/tests/e2e.rs || echo "0"
# (expected: 1 — Hunk-3 added it)
```

If ANY grep returns the wrong count → patch + re-grep before committing.

## §3 Required reading

In this order:

1. **`crates/routes/src/utils/scheduled_tasks.rs` Hunk-1 area (~lines 420-460)** — read the full federation_inbox_nonce cleanup tick. Note especially:
   - The existing `warn!("federation_inbox_nonce_cleanup: previous batch still running, ...")` call shows `warn!` is in scope (do NOT add a `use` for it).
   - The downstream `window_days_i64` reference at line ~446 inside `federation_inbox_nonce::delete_older_than(window_days_i64, ...)` — Hunk-1 rebinds `window_days_i64` to the clamped form, so this downstream line stays UNCHANGED.

2. **`crates/server/tests/e2e.rs` Hunk-2 area (~lines 15580-15640)** — read the entire `per_peer_rate_limit_returns_429` fn. CONFIRM the outer Result type: search for the `fn per_peer_rate_limit_returns_429` line and read its return type. If it returns `LemmyResult<()>`, use the bare-`?` form per §2.2 Hunk-2 reasoning. If it returns `Result<(), Box<dyn Error>>` (unlikely — fed-in-b fixtures are all LemmyResult), STOP + `kind:"blocker"` and surface the discrepancy (Case-A vs Case-B per `feedback_lemmy_error_no_std_error.md`).

3. **`crates/server/tests/e2e.rs` Hunk-3 area (~lines 15680-15710)** — read the entire `moderation_label_handler_persists_and_logs` fn end. Confirm the test does ONE label processing (one publish_label call) before the log-count query — yes, the test sends one `PublishLabel` activity and expects exactly one `federation_label_received` log row. `assert_eq!(log_count, 1)` is the deterministic equality.

4. **`.claude/PRPs/reviews/pr-139-findings.yaml`** (the gate-3-approved triage) — read the cr-1, cr-2, cr-3 entries verbatim (the brief's §2.2 is the canonical recipe; the YAML is the durable triage record). The yaml is gitignored but lives in the worktree.

5. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4):
   - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Why:** Case A discipline holds on the new Hunk-2 edit; bare `?` is the LemmyResult<()> pattern; CR's `anyhow::anyhow!` recipe is Case B which would fail to compile against LemmyResult. Mirror Case A.
   - `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — **Why:** mechanical fix-impl briefs MUST include the §2.4 pre-push triple-cargo-check; local cargo ~3min vs PR CI cycle on regression.
   - `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** clippy denies `#[allow]`; the §2.2 fix must NOT use a lint-suppression workaround. `assert_eq!` does NOT have a `.expect`-class clippy concern.
   - `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **Why:** TWO Edits ONLY on the 15k-line e2e.rs (Hunk-2 in-place at ~15617 + Hunk-3 in-place at ~15704 — locate each hunk via grep, edit, save, move on; NEVER read the entire file).
   - `.claude/lessons/feedback_batch_goto_eof_clobbers_errorlevel.md` — **Why:** §2.4 cargo cmds are 3 SEPARATE `cmd //c` invocations (NEVER bat-&&-bat chain).
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always) — capture-then-tail for §2.4 pre-push validation.

## §3a Handover from prior cohort

> **Handover state: PR-CR-FIX-IN-PR-DRIVEN** (the trigger is CR review on PR #139, NOT a Phase-2 e2e fail). Phase-2 e2e PASSED CLEAN on the pre-fix-impl-8 tip (4efce35c8: 102 passed / 0 failed / 5 ignored / 34m32s); fix-impl-8 commits will rebase PR #139's CR review on the new tip. No DQ entry is the gate (the YAML at `.claude/PRPs/reviews/pr-139-findings.yaml` is the durable triage record).

```yaml
prior_state:
  - phase_branch_tip: de594992f
    head_pr: 139
    cr_review_posted_at: 2026-05-20T12:00:05Z
    cr_findings: 15 (3 code-level fix-in-pr + 12 brief/runlog wont-fix)
    copilot_review_posted_at: 2026-05-20T11:51:44Z
    copilot_findings: 4 (3 carry-forward + 1 done-duplicate of cr-2)
    triage_approved_at: 2026-05-20 (user-gate-3)
    bundled_into_fix_impl_8: [cr-1, cr-2, cr-3]
    deferred_to_retro_or_follow_on: [copilot-1, copilot-2, copilot-3]
    done_by_duplicate: [copilot-4]
    wont_fix_archived_briefs_lint: [cr-4 through cr-15]
  notes: "fix-impl-8 CONSUMES the pre-fix-impl-8 phase tip unchanged except for the 3 hunks. The §0 grep self-verifies the defect symbol presence as the authoritative existence check. The §2.4 §15-precondition verification (check + clippy + test --no-run) is the worker's contract: §2.4 pre-push MUST pass before exit. Advisor does NOT re-run Phase-2 e2e after fix-impl-8 finalize-merge (the 3 hunks are local quality-bar improvements without semantic change to wrap_governance_inbound's enforcement path; the prior round-2 e2e on 4efce35c8 already cleared the impl phase). On finalize-merge: PR #139 auto-updates with the new tip; advisor surfaces user-gate-5b (merge confirm) directly to user."
```

## §4 Constraints (hard rules)

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-federation-inbound-b` (tip `de594992f` or newer — accept any tip post-`1153d4b62`). Finalize merges your worktree branch back; do NOT push to `phase-v1-federation-inbound-b` directly.
- ONE commit (2 files `crates/routes/src/utils/scheduled_tasks.rs` + `crates/server/tests/e2e.rs`, 3 hunks, ~9 lines net). Commit subject:

  ```
  fix(v1-federation-inbound-b): CR PR #139 fix-in-pr — clamp replay_window_days + assert seed-loop receives + pin label log count (fix-impl-8 cr-1+cr-2+cr-3)
  ```

- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately** to your worktree branch per `.claude/rules/decision-queue.md` "Mid-task visibility".
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235)

If you need to write a DQ entry and the Claude Code sensitive-file gate blocks it: (a) write the intended DQ-entry JSON to `FIXIMPL8_BLOCKER_DQ.json` at worktree root, (b) write `FIXIMPL8_ESCALATION.md` naming the issue, (c) commit both + push, (d) STOP. The advisor transcribes per `.claude/rules/escalation.md`. fix-impl-8 has NO `.claude/` deliverable on the happy path — its only deliverable is the 3-hunk 2-file edit; no DQ write expected (CR review is the gate, not a validate-pending cycle).

### fix-impl-8 GOTCHAs (load-bearing)

- **TWO Edits ONLY on the 15k-line e2e.rs** per `feedback_junior_worker_e2e_edit_hang.md`. Hunk-2 in-place at ~lines 15617-15620; Hunk-3 in-place at ~line 15704. NEVER read the entire file (15k lines blows context budget); locate each hunk via grep (the §0 anchors give you exact line numbers), Edit, save, move on. Do NOT re-read e2e.rs after editing.
- **Hunk-1 has TWO references to `window_days_i64`** in `scheduled_tasks.rs` — the assignment (Hunk-1 renames the SOURCE to `raw_*` and rebinds the SAME `window_days_i64` to the clamped value) and the downstream `delete_older_than(window_days_i64, ...)` call (Hunk-1 leaves this UNCHANGED — the rebind keeps the name). Verify cargo-check compiles by confirming the rebind is visible at the use-site.
- **Hunk-2 uses bare `?`, NOT CR's `anyhow::anyhow!`.** The v1_federation_inbound_b_fixtures module's outer Result type is `LemmyResult<()>`; `anyhow::Error` doesn't have a `From` impl for LemmyError. CR's recipe was contextual to a generic Rust idiom; this codebase's Case A discipline is bare `?`. See `feedback_lemmy_error_no_std_error.md` for the canonical Case A pattern.
- **Hunk-3 changes `assert!(log_count >= 1)` to `assert_eq!(log_count, 1)`.** This is the CR-posted committable suggestion verbatim. No alternative form.
- **One commit, two files.** If §2.4 pre-push validation surfaces a regression in a different file (unexpected), STOP + `kind:"blocker"` (do NOT add a 4th hunk or touch another file).

### Plan-cited line numbers may have drifted

§2.2 cites lines ~432-439 (scheduled_tasks.rs), ~15617-15620 (e2e.rs), ~15704 (e2e.rs). Use the §0 grep anchors for real positions on the actual base tip. The edits are identified by the STRING patterns (`let window_days_i64 = lemmy_api::governance::config::get_int`, `let _ = ActivityTrait::receive(activity, &context).await`, `assert!(log_count >= 1)`), not by line numbers.

## §5 Validation gates (advisor verifies PR auto-update — worker runs §2.4 pre-push validation)

**Worker-side (this fix-impl):** §2.4 pre-push triple-validation (cargo-check + cargo-clippy -D warnings + cargo-test --test e2e --no-run) is the only worker-side validation. Worker does NOT raise a NEW `validate-pending-laptop` DQ — fix-impl-8 is CR-fix-in-pr (no validate-pending cycle); PR #139 picks up the new tip automatically.

**Advisor-side (after this fix's finalize-merge):**

1. Verify daemon finalize-pushed (per the recurring finalize-push-skip class — if the worker pre-pushed, the daemon may not push; advisor ssh-push from daemon as recovery).
2. Verify PR #139 mergeStateStatus is still CLEAN/MERGEABLE after the new tip lands.
3. Surface user-gate-5b (final merge confirm) with the 3-hunk diff summary + reminder that all 3 CR fix-in-pr items are addressed.

NO re-run of Phase-2 e2e is needed for fix-impl-8 (the 3 hunks are quality-bar improvements without semantic change to the enforcement path; the prior round-2 e2e on 4efce35c8 cleared the impl phase). CR re-review on the new tip may post follow-up findings (e.g. "approved" or a minor nit); advisor surfaces any new findings via a fresh /bm-poll-cr cycle.

## §6 Expected output (return to advisor)

```
## fix-impl-8 complete — CR PR #139 fix-in-pr (cr-1 + cr-2 + cr-3)

**Commit:** <sha> on <worktree-branch>
**Files changed:** crates/routes/src/utils/scheduled_tasks.rs (Hunk-1) + crates/server/tests/e2e.rs (Hunk-2 + Hunk-3)
**Hunk-1 (cr-1):** scheduled_tasks.rs ~lines 432-440 — clamped replay_window_days to >= 1; warn on raw<1 (rename to raw_*; rebind window_days_i64; +6 lines)
**Hunk-2 (cr-2):** e2e.rs ~line 15619 — bare `?` propagation for seed-loop receives (LemmyResult<()> Case A; -1 +1 = 0 net)
**Hunk-3 (cr-3):** e2e.rs ~line 15704 — assert_eq!(log_count, 1) (CR verbatim committable suggestion; -1 +1 = 0 net)
**PRECON self-check:** §0 anchors all present; outer Result of per_peer_rate_limit_returns_429 confirmed LemmyResult<()>.
**No other-file edit:** confirmed (only scheduled_tasks.rs + e2e.rs touched).
**§2.4 pre-push validation:** ALL 3 GREEN — cargo-check exit 0, cargo-clippy -D warnings exit 0, cargo-test --test e2e --no-run exit 0 (e2e binary links cleanly).
**Grep-verify:** the 8 grep counts at expected values (Hunk-1: raw_=1, .max(1)=1, OLD direct=0, warn body=1; Hunk-2: OLD let_=0, NEW bare-?=1; Hunk-3: OLD ≥=0, NEW eq=1).
**No new DQ raised** (CR fix-in-pr is gated by PR review, NOT validate-pending).
**Next:** advisor verifies PR #139 picks up the new tip cleanly; user-gate-5b (final merge confirm) → bm-merge.
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for §2.4 pre-push failure — that you patch in-commit per the §2.4 routing).

## §7 Why this brief differs from the plan

Plan §13 closed at Task 9; fix-impl-8 is a post-plan, post-impl-phase, in-PR quality-bar fix bundle triggered by CR review on PR #139. Plan §13 does NOT need to be retrofit — the 3 hunks are CR-suggested improvements, not new functional scope. The CR finding cr-1's recipe (the clamp on `replay_window_days`) DID surface a latent defect-class (admin-misconfig nullifies replay protection); a future sub-phase OR an admin-validation pass over the full `governance_config` schema may want to systematise the "clamp config to safe minimums on read" pattern (TODO: file as `kind:"log"` DQ for retro harvest). cr-2 and cr-3 are pure test-quality improvements.

This brief mirrors the canonical sibling `.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-7.md` schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate (same §0-§7 structure; same multi-file CAP discipline; same §2.4 pre-push triple-cargo-check). Includes the VERBATIM §G4 blockquote (extended for the three failure signatures) in §2.2 per `.claude/rules/advisor-orchestrator.md` §G4 mandatory-verbatim-recipe rule.

**Retro carry-forward (NOT bundled here):**

- **Copilot DoS-hardening family (copilot-1/2/3, gate-3 carry-forward):** unbounded in-memory rate-limit maps + raw-string HashMap keys in `crates/apub/activities/src/governance/inbox.rs:473` and `publish_trust_attestation.rs:165` + TOCTOU eviction in `evict_oldest_unreviewed_if_needed:698`. Design work for fed-in-c OR a dedicated DoS-hardening follow-on sub-phase. File as one GH issue.
- **Append-history-unaware reader defect (from fix-impl-7 carry-forward):** `get_inbound_config_int` in inbox.rs:421-438 + publish_trust_attestation.rs:142-152. Same retro-scope as the Copilot family above.
- **Admin-config clamp-on-read pattern (from cr-1):** systematise the "clamp config value to safe minimums on read" pattern across all `governance_config` consumers; not just `replay_window_days`. Likely a config-helper sub-phase or a config-schema retro item.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per fix-impl-7 precedent the canonical session authors the brief, then cherry-picks onto `phase-v1-federation-inbound-b` so the worker — which branches from the PHASE branch tip — sees it). `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC to the post-cherry-pick tip before queue. §G4-allowlist-equivalent mechanical fix (canonical-recipe-mirror; zero design ambiguity); does NOT require AskUserQuestion — auto-queued per §G4 allowlist routing + user gate-3 pre-approval 2026-05-20. fix-impl-8 is the FIRST fix-impl-8 for PR #139 CR review; numbering: 1-7 = impl-phase fix-impls, 8 = post-PR CR fix-in-pr bundle._
