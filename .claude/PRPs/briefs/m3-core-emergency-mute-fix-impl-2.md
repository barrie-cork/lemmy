# Brief: m3-core-emergency-mute fix-impl-2 (CR #204 cr-2/cr-3/cr-7/cr-8 — governance hook error propagation)

## §1 Role + dispatch line

`[role:impl-task] m3-core-emergency-mute-fix-impl-2-governance-hook-errors — see .claude/PRPs/briefs/m3-core-emergency-mute-fix-impl-2.md`

## §2 Scope

**Fix CodeRabbit findings cr-2, cr-3, cr-7, cr-8 on PR #204** — four governance handler files silently discard errors on security/hook boundaries. User-approved fix-in-PR (gate-3, 2026-06-19). These were outside-diff findings exposed by the clippy-debt context CR analysed.

**Produces (exactly 4 files, ONE commit):**
1. `crates/api/api/src/governance/actor_app_link.rs` — cr-2: line ~91
2. `crates/api/api_crud/src/governance/revoke_endorsement.rs` — cr-3: line ~165
3. `crates/api/api/src/governance/sanction_publisher.rs` — cr-7: line ~141
4. `crates/api/api/src/governance/submit_jury_vote.rs` — cr-8: line ~178

**The four defects — all share the same pattern (`.ok()` / `unwrap_or_default()` on a security or governance hook boundary):**

### cr-2 — `actor_app_link.rs:91` — silent failure on bridge callback POST (CRITICAL)

`link_actor` calls `post_bridge_callback(...)` or similar and discards the error with `.ok()`. Bridge callback failures must propagate — a silently-failed link means actor identity is broken without the caller knowing.

Fix: replace `.ok()` with `?` (or `map_err(|e| LemmyError::from(e))?` if the error type differs). Read the actual call site first to confirm the exact pattern.

### cr-3 — `revoke_endorsement.rs:165` — silent failure on post-transaction governance hook (CRITICAL)

Post-transaction hook call discards the error with `.ok()`. Governance hook failures must surface — a revocation that silently fails to notify downstream is a governance chain integrity issue.

Fix: replace `.ok()` with `?` propagation. If the hook failure is intentionally best-effort (e.g. a notification that should not roll back the transaction), replace `.ok()` with `if let Err(e) = ... { tracing::warn!("governance hook failed: {e:#}"); }` — but only if there is an existing pattern in sibling handlers showing best-effort hooks. Read sibling handlers first.

### cr-7 — `sanction_publisher.rs:141` — `unwrap_or_default()` on `BRIDGE_CALLBACK_SECRET` (CRITICAL)

`unwrap_or_default()` on the `BRIDGE_CALLBACK_SECRET` env var produces an empty string secret on misconfiguration, silently weakening the auth guarantee. `bridge_auth.rs` already uses `?` propagation on the same env var (that fix shipped in the clippy-debt commit this phase).

Fix: mirror `bridge_auth.rs` — use `.ok_or_else(|| anyhow::anyhow!("BRIDGE_CALLBACK_SECRET not set"))?` or the project-canonical env-var error pattern. Read `bridge_auth.rs` for the exact form.

### cr-8 — `submit_jury_vote.rs:178` — `.ok()` on post-transaction governance hook (CRITICAL)

Same pattern as cr-3. Post-transaction hook call discards error. Fix with the same approach as cr-3 (read `submit_jury_vote.rs:178` to confirm whether the hook is mandatory or best-effort, then apply `?` or `tracing::warn!` accordingly).

**ADR-pinned gate (§2.4a — load-bearing):**
- **ADR-015** pin: none of these files write `actor_pseudonym` in this fix — the fix is error propagation only. No pseudonymity changes.
- **ADR-013 EmergencyRemove gate** is not touched.
- The `bridge_auth.rs` fix (already shipped) is the precedent for cr-7 — this fix completes the pattern across `sanction_publisher.rs`.

**Do NOT:**
- Touch `services/bridge/**` (that's fix-impl-1).
- Address cr-1/cr-4/cr-5/cr-6 (wont-fix), cr-9/cr-10/cr-11 (fix-impl-1), cr-12 (rebutted).
- Add new capabilities, routes, or DB writes.
- Touch `Cargo.toml`, `Cargo.lock`, migrations.

**Branch:** forks from `phase-m3-core-emergency-mute` (current tip `78a7b2655`).

## §3 Required reading

- `crates/api/api/src/governance/actor_app_link.rs` — read the `link_actor` fn and the `.ok()` call site (~line 91).
- `crates/api/api_crud/src/governance/revoke_endorsement.rs` — read the post-txn hook call (~line 165); check if sibling handlers show best-effort vs mandatory pattern.
- `crates/api/api/src/governance/sanction_publisher.rs` — read the `BRIDGE_CALLBACK_SECRET` usage (~line 141); mirror `bridge_auth.rs`.
- `crates/api/api/src/governance/submit_jury_vote.rs` — read the hook call (~line 178).
- `crates/api/api/src/governance/bridge_auth.rs` (or wherever `BRIDGE_CALLBACK_SECRET` is read) — the canonical env-var error-propagation pattern for cr-7.
- **Lessons (mandatory, §2.4 — crates governance handlers + ADR-pinned paths):**
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — ADR gates must remain load-bearing even in fix tasks.
  - `feedback_governance_type_state_handlers.md` — governance handler conventions.
  - `feedback_clippy_test_style.md` — `LemmyResult<()>` + `?`; no `unwrap`/`expect`.
  - `feedback_multi_write_handlers_need_transactions.md` — if any fix adds a second DB write, it needs a transaction.

## §4 Constraints

- **ONE commit, 4 files** (`actor_app_link.rs`, `revoke_endorsement.rs`, `sanction_publisher.rs`, `submit_jury_vote.rs`) — `fix(governance): propagate hook errors instead of silently discarding (CR cr-2/cr-3/cr-7/cr-8, PR #204)`.
- **`?` propagation is the default fix.** Use `tracing::warn!` only if the call site is clearly best-effort AND a sibling handler in the same file shows the warn pattern for the same hook type. Do not invent the warn pattern — read the sibling first.
- **cr-7 MUST mirror `bridge_auth.rs`** — `BRIDGE_CALLBACK_SECRET` missing must be a hard error, not an empty-string default.
- **No new `unwrap`/`expect`/`unwrap_or_default`** (clippy `-D warnings`).
- **Windows cargo validation applies** (these are `crates/` files, not bridge). Write a `validate-pending-laptop` DQ with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full", "./scripts/brehon/cargo-clippy.sh --workspace --features full -- -D warnings"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` + `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending`. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo yourself. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** deliver HANDOVER inline in task output (do NOT write to `.claude/PRPs/handovers/`).

## §3a Handover from prior tasks

- Tasks 1-4 shipped `crates/api/api/src/governance/governance_log.rs` (Task 1) and bridge files (Tasks 2-4).
- The clippy-debt fix (Cohort A) shipped `bridge_auth.rs` with the canonical `?`-on-BRIDGE_CALLBACK_SECRET pattern — cr-7 extends the same pattern to `sanction_publisher.rs`.
- These 4 files were not touched in Tasks 1-4; they are pre-existing governance handlers that CR flagged because the debt-fix context made their `.ok()` patterns visible.
