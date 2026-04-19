# polish-2 (Bucket C residual) — impl brief

**Authored by:** advisor (homeserver session), 2026-04-19 post-polish-1-merge
**For:** whoever picks up polish-2 next
**Base:** `governance-v0` HEAD `155eb8d18` (PR #64 merge)
**Branch to cut:** `polish/bucket-c`

## Scope

Three residuals from GH #54 (PR #46 Bucket C follow-ups umbrella):

### #54 sub-item #2p-7 — lock-race TOCTOU in `task-hopper.sh`

**Severity:** Critical (per CodeRabbit), but non-production code — advisor tooling only, single-worker v0 runs.

**Background:** polish-1 predecessor `0ff06abc9` closed the guaranteed-race window via compare-and-release. CodeRabbit found a residual TOCTOU between token comparison and `rmdir`. Atomic-rename patch was sketched in issue #54 body — **use that patch shape** if it still applies cleanly.

**Fix shape (atomic rename, from #54 body):**

```bash
release_lock() {
  local staging="$lock_dir.releasing.$$"
  if mv "$lock_dir" "$staging" 2>/dev/null; then
    local current
    current="$(cat "$staging/owner" 2>/dev/null || cat "$lock_owner_file" 2>/dev/null || true)"
    if [ "$current" = "$lock_token" ]; then
      rm -f "$lock_owner_file"
      rmdir "$staging" 2>/dev/null || rm -rf "$staging"
    else
      if ! mv "$staging" "$lock_dir" 2>/dev/null; then
        rm -rf "$staging"
      fi
    fi
  fi
  rm -f "$tmp_file"
}
```

**Scope note:** the owner file can stay sibling (current pattern) with the fallback `cat` shown above — smaller diff than moving the owner file inside `$lock_dir`. Don't change the sibling pattern unless something else forces it.

**Validation:** `bash -n scripts/brehon/task-hopper.sh`. If a stress-test harness already exists, run it. If not, don't write one — belt-and-braces.

**File:** `scripts/brehon/task-hopper.sh`

### #54 sub-item #2p-2 / #2p-3 — `publish_sanction_notice.rs` defensive hardening

**Severity:** Major (per CodeRabbit). Belt-and-braces — no current caller violates.

**Two findings:**
- **#2p-2:** reject remote actor at builder. The activity builder constructs a sanction notice without asserting the actor is local.
- **#2p-3:** enforce federated-scope invariant. Builder doesn't assert `scope = FederatedRecommendation` before constructing.

**Fix shape:** add two guards at the top of the builder function. Return `LemmyErrorType::*` (choose fitting variant or introduce a new one — check #54 body for ADR pointer) when either invariant fails.

**Acceptance:** unit test the builder (not e2e) — pass a remote actor, expect error; pass a non-federated scope, expect error.

**File:** `crates/apub/activities/src/governance/publish_sanction_notice.rs`

### #54 sub-item #2p-6 — brief doc-rot

**Severity:** Major (per CodeRabbit), but doc-only — `task-hopper.json` shows agents invoked correctly despite briefs saying `task-hopper.sh start 70` (bare numeric). Validator regex is `^task-[a-z0-9-]+$`.

**Fix shape:** update all `.claude/PRPs/phase-6-runlog/briefs/agent-*.md` to use `task-<id>` form. Doc update only. Alternative (relax validator) is rejected — doc fix is the smaller surgery and keeps the validator strict.

**Files:** `.claude/PRPs/phase-6-runlog/briefs/agent-a.md` through `agent-g.md` (plus `agent-e2.md` if distinct).

## Shipping rules (from PR #64 / PR #46 retros)

- Base `governance-v0` at `155eb8d18` (cut fresh; do not rebase onto a stale ancestor)
- Merge method: `--merge` (never `--squash`)
- Critical-only CodeRabbit discipline: defer new non-critical findings to a follow-up PR, don't chase CR tail in this PR
- One retro per PR at `.claude/PRPs/reports/polish-2-retro.md` (three H2 sections per `feedback_retro_not_report.md`)
- Single impl session — these three fixes share no code surface but are small enough that one session carries all three serially

## Validation sequence

After all three fixes:

1. `cargo check --workspace --features full` (pass — none of these changes should affect compile)
2. `cargo clippy --workspace --no-deps --features full -- -D warnings`
3. `cargo test --test e2e --no-run -p lemmy_server`
4. `cargo test --test e2e -p lemmy_server` — expect 16 passed / 0 failed / 3 ignored (same as polish-1 baseline)

**Remember `feedback_pipes_mask_exit_codes.md`:** capture cargo output to file, grep `^error` even if wrapper says exit 0. `cargo-test.bat` exit-code masking is documented as plan item 22a.

## PR shape

Suggested commit sequence:
1. `fix(hopper): atomic-rename release_lock (CodeRabbit #54 #2p-7)` — sh
2. `fix(apub): reject remote actor + non-federated scope at publish_sanction_notice builder (CodeRabbit #54 #2p-2 #2p-3)` — rs
3. `test(apub): unit coverage for publish_sanction_notice invariant guards` — rs
4. `docs(briefs): use task-<id> form in phase-6 agent briefs (CodeRabbit #54 #2p-6)` — md

PR title: `v0-polish-2: Bucket C residual — hopper TOCTOU + apub defensive hardening + brief doc-rot`

PR body: reference GH #54 sub-items #2p-7, #2p-2, #2p-3, #2p-6. Include validation matrix.

## Out-of-scope

- #2p-4 governance_log INSERT+UPDATE atomicity — **already landed in polish-1** (`1bb622e8a`). Do not touch.
- Any new CodeRabbit findings from this PR's review cycle — defer to polish-N unless Critical.
- Polish-3 docs sweep — separate PR, separate brief (`11-polish-3-brief.md`).

## Parallel-safety

- Polish-2 touches: `scripts/brehon/task-hopper.sh`, `crates/apub/activities/src/governance/publish_sanction_notice.rs`, `.claude/PRPs/phase-6-runlog/briefs/agent-*.md`
- Polish-3 touches: `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md`, `crates/api/api/src/governance/accept_jury_assignment.rs` (doc-comment only), `crates/db_views/governance_case/src/impls.rs` (doc-comment only), `.claude/PRPs/plans/phase-6-federation.plan.md`, `.claude/decision-queue.json`, `.claude/rules/task-hopper.md`, `.claude/PRPs/reports/phase-5c-complete-report.md`
- **No file overlap.** Polish-2 and polish-3 can land in either order.
