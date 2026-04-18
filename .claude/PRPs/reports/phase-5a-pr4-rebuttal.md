# Response to CodeRabbit review (40 comments)

Pushed as `5910c3a04` on `phase-5a`. Four categories:

### Patched in `5910c3a04` — 13 findings

**Critical + major**
- **C2** `scheduled_tasks.rs` — module-scope `AtomicBool REPUTATION_SNAPSHOT_RUNNING` + RAII `RunningGuard` prevents overlapping `run_snapshot_batch` if a tick overruns. Items at module scope so clippy's `items_after_statements` stays quiet.
- **C3** `migrations/.../{up,down}.sql` — reverted the `UPDATE moderation_case SET threshold_score * 1_000_000` (and its divide-back). Phase 5b task 58 owns the micros formula. Leaves Phase 4's `report_to_modlog_golden_path` semantics intact (handler still in integer units).
- **M2a** `db_schema/source/person.rs` — `#[serde(skip)]` + `#[ts(skip)]` on `membership_state` so `Person` does not leak the deferred-enforcement column over the API.
- **M2b** `db_schema/impls/person.rs` — `Person::upsert` chains `.set((form, person::membership_state.eq(person::membership_state)))` so federation refresh preserves the locally-assigned value instead of resetting to the SQL default.
- **M3** `governance_config.rs` — drop `AsChangeset` derive; the table is append-only by design.
- **M4** `migrations/.../up.sql` — `updated_by` FK `ON DELETE SET NULL` → `ON DELETE RESTRICT` to preserve admin attribution on the append-only history.
- **M5** `db_views/reputation` — `EndorsementSummaryView.active_sureties` split into `active_sureties_inbound` + `active_sureties_outbound` so consumers can tell which direction the count represents.
- **M6** `db_views/reputation/src/lib.rs` — `#[serde(skip)]` + `#[ts(skip)]` on the four raw dimension scores (ADR-005: users see capabilities, not numbers).

**Judgment calls**
- `reputation_snapshot.rs:657` — defensive advisory-lock key shift so `None` and `Some(CommunityId(0))` do not collide (no live impact today since SERIAL starts at 1).
- `reputation_snapshot.rs:335` — added TODO explicitly pinning v0 single-step halving; compound-decay + regression test are a v1 concern alongside the `can_sponsor` gate flip. Watermark issue + missed cascades are carry-forward per below.
- `routes/src/lib.rs:498` — `scope("/governance").wrap(rate_limit.post())` so governance writes inherit per-route rate limiting on top of the outer `rate_limit.message()`.

**Mechanicals / hygiene**
- Env var `BREHON_DISABLE_BACKGROUND_JOBS` → `BREHON_DISABLE_SNAPSHOT_JOB` (old name misleadingly implied all bg jobs).
- `.claude/decision-queue.json` — removed duplicated `id: 13`/`14` from `pending` (they already exist in `resolved`).
- Phase-5a complete report — escaped `|` inside table cells (`(age\|open\|closed)`), dropped the reverted micros rescale from the task-50 deliverable, synced the env-var name.
- Phase-5a handover — scrubbed two absolute Windows paths to `<advisor-local>/...`.
- `lint-no-membership-read.sh` — added the 3 paths the `grep -v` chain already excluded (`db_schema/impls/person.rs`, `db_views/registration_applications/src/impls.rs`, `server/tests/e2e.rs`) to the header authorised-sites list.
- `migrations/.../down.sql` — dropped redundant `DROP INDEX` before `DROP TABLE` (cascades).
- `server/src/governance.rs` — clarified the function docstring: declarative stub, real scheduling in `scheduled_tasks::setup`.

### Plan-accepted tradeoffs — not patched (4 rebuttals)

- **M7 / M9** (`create_endorsement.rs:214/248`) — TOCTOU on cap/cooldown/surety. Plan-accepted per GOTCHA-55e. v0 serialises via `run_transaction` + `FOR UPDATE` on the snapshot row (`acquire_advisory_xact_lock` later extends this to pair-level). Sponsor-level advisory-lock hardening is a v1 item (open OQ).
- **M8** (`create_endorsement.rs:214`) — `LemmyErrorType::NotFound` collapse. Plan-accepted per GOTCHA-55f. Introducing a dedicated `EndorsementRejected`/`SponsorGateClosed`/`EndorsementCapReached`/`EndorsementCooldownActive` variant is a `LemmyErrorType` carry-patch tracked for v1 (the enum is upstream-held so it needs a separate commit when we land other upstream-PR patches in bulk).
- **M10** (`user/create.rs:141-157 + 397-409`) — `default_membership_state` read duplication. Plan-intended per GOTCHA-51b: the federated and non-federated register paths diverge in what they do *around* the config read, so the extract-to-helper pattern shown in the suggestion would entangle two intentionally-separate flows. Reconfirmed both call sites against the plan — both are load-bearing.
- **M11** (`config.rs:236`) — silent `Member` fallback on unknown `membership_state` text. Plan/task-51 spec: warn-and-fall-back is intentional. The CHECK constraint is advisory and the lint guard (`lint-no-membership-read.sh`) catches any real read path. Flipping the return type to `LemmyResult<MembershipState>` would force every on-read-path caller to handle a failure that in practice cannot happen given the seed surface and the grep-guard — a larger refactor than the risk warrants in v0.

### Carry-forward to Phase 5b / 5c

- **C1** (`reputation_snapshot.rs:609`) — dirty-pair detection missing time-based decay and config-cascade flips. Real hole. Fix requires a stale-by-age predicate and/or a one-shot full re-scan triggered on `thresholds.*` / `decay.*` config writes; the cleanest cascade-trigger path depends on the `admin-config-write.sh` wrapper which is deferred to 5c sibling docs per DQ#13. Tracked in `phase-5a-complete-report.md §5`.
- **Watermark bug** (`reputation_snapshot.rs:590`) — global-max watermark undercounts dirty pairs when one pair is recent. Part of the same C1 work; per-pair watermark (or `job_watermark` keyed by job name) lands alongside the stale-by-age predicate.
- **M1** (`create_endorsement.rs:225`) — `UNIQUE (from_person_id, to_person_id, community_id)` blocks re-endorsement after revocation. Real bug. The revoke endpoint is a later MVP slot; once it ships, the migration can convert the constraint to partial-unique (`WHERE revoked_at IS NULL`) or the handler can pre-check for revoked rows and respond explicitly.
- **`governance_config_current` view security_invoker + column-order comment** — deferred; small docs-only tweaks, no behaviour change, will bundle with the 5c admin-config wrapper commit.
- **`list_endorsements_for_person` / `read_reputation_summary` Diesel `.into_boxed()` dedup** — pure-style refactor opportunity flagged by CodeRabbit; not urgent; will bundle into the 5b view-crate tidy pass.
- **`config.rs` generic typed-accessor helper + `Cow`-keyed cache + community-scope single-query** — three separate 🔵 Trivials on the config reader; all quality-of-life, none changes behaviour. Bundle into a `refactor(config):` commit in 5b.
- **`config_parity_round_trip` existence check** — stronger assertion that each seeded key has a row in `governance_config_current` before calling the accessor (so a missing seed row cannot hide behind the `const_default_*` fallback). Small test-tightening; 5b task-0-style audit will add it.

### Two comment-only findings — not fixes

- **`pagination.rs` / `vote/impls.rs` `PR #___` TODO placeholder** — keep as-is. Per `feedback_carry_patch_todos.md` this is the canonical Brehon fork carry-patch format: `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` with `#___` left literal as the placeholder for a real upstream PR number. The rule explicitly says "Do NOT invent a number, do NOT use TBD, do NOT drop the `#___` placeholder entirely." Consistency across all carry-patches is load-bearing for the weekly upstream rebase.
- **`governance_config_current` table!-macro comment** — deferred to the same 5c admin-config-wrapper commit as the view security_invoker tweak (both touch the same file region).

### Validation

- `cargo check --features full --workspace` exit 0.
- `cargo clippy --features full --workspace --no-deps -- -D warnings` exit 0.
- `cargo test -p lemmy_server --test e2e` 9 passed / 0 failed.
- `lint-no-membership-read.sh` + `lint-no-can-sponsor-read.sh` exit 0.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
