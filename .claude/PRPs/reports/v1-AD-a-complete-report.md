# v1-AD-a completion report

**Branch:** `phase-v1-AD-a` at `319b861fb` (10 commits ahead of `governance-v0` @ `563d7904c`)
**Plan:** `.claude/PRPs/plans/v1-admin-dashboard-a.plan.md` (1332 lines, 10 tasks, 9 producing commits)
**Landed:** 2026-04-20
**Validation status:** all plan §15 DoD gates green

## Commits (in order)

| SHA | Task | Summary |
|---|---|---|
| `6b16bc7cc` | 1 | `migrations/2026-04-22-000000-0000_add_rule_set_versions/{up,down}.sql` |
| `363d91187` | 2 | `migrations/2026-04-22-000100-0000_add_sponsor_allowlist/{up,down}.sql` |
| `e66853f5f` | 3 | `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/{up,down}.sql` |
| `dbaf58e4b` | 4 | `crates/db_schema_file/src/schema.rs` — `rule_set_version` + `sponsor_allowlist` table blocks + 2 cols on `moderation_case` |
| `50482e55f` | 5 | `crates/db_schema/src/source/governance/{rule_set_version,sponsor_allowlist}.rs` + newtypes + mod.rs |
| `026656ae7` | 6 | `config.rs` — `ConfigKeyMetadata` registry + 27 new `DEFAULT_*` consts + extended `SEEDED_KEYS_WITH_CONSTS` to 61 + `every_seeded_key_has_metadata` parity test |
| `fc6f3d733` | 7 | `migrations/.../seed_v1_config_keys` + 2 new `ENTRY_KIND_ADMIN_CONFIG_*` consts (DEFINE + RE-EXPORT) |
| `ac8cf70d6` | 8 | `.claude/rules/governance-log-entry-kind-registry.md` (closes GH #41) |
| `a27b86d63` | 9 | Append OQ-V1-AD-01/02/03 to `99-decisions-and-open-questions.md` + changelog entry |
| `319b861fb` | 5-fix | `ModerationCase` Queryable + `ModerationCaseInsertForm` Insertable — add 2 columns (`applied_config_snapshot`, `rule_set_version_id`) missing from task 5's initial Rust-side work; patch 11 struct-literal sites with `..Default::default()` |

## Validation (plan §15)

| Level | Command | Result |
|---|---|---|
| 1 | `cargo-check.bat --workspace` (per-task) | ✅ exit 0 after every task |
| 2 | `cargo-check.bat --workspace --features full` | ✅ exit 0 (19.54s warm) — log `phase-v1-AD-a-level2-postfix2.log` |
| 3 | `cargo-test.bat --workspace --lib parity` | ✅ exit 0; 3/3 tests: `seeded_keys_count_matches_const_count`, `every_seeded_key_has_const_fallback`, `every_seeded_key_has_metadata` — log `phase-v1-AD-a-level3-postfix.log` |
| 4 | `cargo-test.bat --test e2e --no-run -p lemmy_server` | ✅ exit 0 (13m24s cold) — produced e2e executable; `config_parity_round_trip` test-run deferred (requires Postgres via Docker, not available in session) |
| 5 | `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` | ✅ exit 0 (2m58s warm) — zero warnings, zero errors — log `phase-v1-AD-a-level5-postfix.log` |
| 6 | PM-hook literal presence loop | ✅ all 6 hooks present |

## Stop-hook supplementary

| Command | Result | Notes |
|---|---|---|
| `cargo fmt --all -- --check` | pre-existing noise | Nightly-only gate per `.woodpecker.yml`; stable-fmt emits 600+ diffs all in upstream / prior-phase files (none in v1-AD-a commits). Not a plan DoD gate. |
| `cargo clippy --workspace --all-targets -- -D warnings` | 100 errors | All errors in pre-existing `tests/e2e.rs` (phases 1-5c) and `reputation_snapshot.rs` (Phase 5a). Zero v1-AD-a commits modified the flagged code. Baseline at `governance-v0` @ `563d7904c` fails the same gate. This fork's GH Actions CI does not run `--all-targets` (only Woodpecker does, and Woodpecker is not integrated with GitHub here). Plan's Level 5 (without `--all-targets`) is the authoritative equivalent per stop-hook language "or equivalent from your plan". |

## Acceptance criteria (plan §16)

- [x] 9 commit tasks + 1 correction commit (task 5 gap) on `phase-v1-AD-a` (10 total ahead of `governance-v0`)
- [x] Levels 1/2/3/4-compile/5/6 exit 0
- [x] `SEEDED_KEYS_WITH_CONSTS.len() == EXPECTED_SEED_COUNT (34) + EXPECTED_SEED_COUNT_V1_AD (27) == 61` (parity test)
- [x] `CONFIG_KEY_METADATA.len() == SEEDED_KEYS_WITH_CONSTS.len()` (parity test)
- [x] `EXPECTED_SEED_COUNT == 34` (v0 invariant unchanged)
- [x] `rule_set.active_version_id` NOT seeded, NOT in SEEDED/METADATA/const_default_int (advisor edit #2)
- [x] `ENTRY_KIND_ADMIN_CONFIG_CHANGED == "admin_config_changed"` — byte-identical to `scripts/brehon/admin-config-write.sh:148`
- [x] `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED == "admin_config_change_denied"`
- [x] Registry file exists with 25 populated-kind rows (19 v0 + 4 Phase 6 + 2 v1-AD-a) + 4 reserved v1 PRD sections (6 level-2 Markdown headings — v0, Phase 6, v1-AD-a, jury-mechanics-v1-reserved, sponsor-liability-v1-reserved, reputation-tuning-v1-reserved, federation-inbound-v1-reserved, Acceptance invariants, Closes)
- [x] OQ-V1-AD-01/02/03 opened in `99-decisions-and-open-questions.md`
- [x] Changelog entry dated 2026-04-21 appended

## Drifts discovered + resolved

1. **Migration timestamps (plan §11 drift):** plan §11 said next-free slot was `2026-04-21-000000-0000`, but Phase 6's merge had already occupied that slot (`add_federation_attestations`). All 4 v1-AD-a migrations shifted forward to `2026-04-22-000000` through `-000300-0000`. Recorded in state file codebase-patterns.

2. **`schema.rs` line shift (plan §10.5 reference):** plan pointed at `schema.rs:724-742` for `moderation_case` block; post-Phase-6 the block is at `:743`. Re-read before task 4 edit; no impact on edit correctness.

3. **27-key reconciliation (plan §13 task 6 GOTCHA drift):** plan §13 enumerated 27 keys but omitted the 2 `governance.dashboard.*` keys and over-counted `rule_set.*`. Authoritative list reconciled from PRD §5.2 sub-PRD-contribution table (advisor edit #1 source-of-truth directive). Ships as committed.

4. **Probe 4 exit-code capture class (iteration 1 finding):** `cmd //c "...>log 2>&1"` + follow-up `echo "$?"` in a SEPARATE bash line loses the exit code under `run_in_background=true`. Rule landed at `.claude/rules/cmd-c-redirect-exit-capture.md` in primary worktree commit `4a2a60b17` on branch `plan/v1-admin-dashboard`; memory entry `feedback_cmd_c_redirect_exit_code_capture.md` added.

5. **Task 5 Rust-side gap (iteration 5 finding, resolved via fix commit `319b861fb`):** task 4 added 2 columns to `moderation_case` schema, but task 5 did not update the `ModerationCase` Queryable struct. Library `cargo check --workspace --features full` passed because no library caller exercised the 18-col Queryable bound; test-target compile (`cargo test --test e2e --no-run`) and `cargo clippy --all-targets` both surfaced the 5 `ModerationCase: FromSqlRow` errors. Fix added the 2 fields to both Queryable and Insertable, plus `..Default::default()` on 11 struct-literal call sites. **Learning carried forward:** schema.rs column additions MUST be reflected in the Queryable struct within the same task. Plan addendum recommended: the `--no-run` test-target compile is a compile-time gate (not a DB-runtime gate) and should run per-task even without Postgres.

6. **TOCTOU with background parity run (iteration 5 transient):** an earlier iteration's parity-test background cargo started mid-way through my commit chain. It compiled against a stale snapshot where the shim file had the new imports but db_schema had not yet landed the definitions. Fresh re-runs against committed tip pass cleanly. Not a code issue — build-cache race. Fix: wait for bg to exit before re-running validation.

## Files touched (grouped)

**Migrations (new):**
- `migrations/2026-04-22-000000-0000_add_rule_set_versions/{up,down}.sql`
- `migrations/2026-04-22-000100-0000_add_sponsor_allowlist/{up,down}.sql`
- `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/{up,down}.sql`
- `migrations/2026-04-22-000300-0000_seed_v1_config_keys/{up,down}.sql`

**Rust (new):**
- `crates/db_schema/src/source/governance/rule_set_version.rs`
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs`

**Rust (modified):**
- `crates/db_schema_file/src/schema.rs` — 2 new `table!` blocks + 2 new cols on `moderation_case`
- `crates/db_schema/src/source/governance/mod.rs` — exports
- `crates/db_schema/src/newtypes.rs` — `RuleSetVersionId` + `SponsorAllowlistId`
- `crates/db_schema/src/source/governance/moderation_case.rs` — 2 new fields on Queryable + InsertForm
- `crates/db_schema/src/source/governance/governance_log.rs` — 2 new `ENTRY_KIND_*` consts
- `crates/api/api/src/governance/config.rs` — `ConfigKeyMetadata` + `ValueType`/`ConfigScope`/`ApplyAt`/`NumericRange` types + 27 new `DEFAULT_*` consts + 27 new `const_default_*` match arms + 27 new `SEEDED_KEYS_WITH_CONSTS` entries + 61-entry `CONFIG_KEY_METADATA` + `EXPECTED_SEED_COUNT_V1_AD` + new `every_seeded_key_has_metadata` parity test
- `crates/api/api/src/governance/governance_log.rs` — 2 new `pub use` re-exports
- `crates/api/api/src/governance/admin_emergency_remove.rs` — `..Default::default()`
- `crates/api/api_crud/src/governance/create_report.rs` — `..Default::default()`
- `crates/server/tests/e2e.rs` — 9 struct-literal sites patched with `..Default::default()`

**Docs / rules (new/modified):**
- `.claude/rules/governance-log-entry-kind-registry.md` (new — closes GH #41)
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — OQ-V1-AD-01/02/03 + changelog

## Not built in v1-AD-a (per plan §12)

- No HTTP route handlers — handlers ship in v1-AD-b/c/d
- No dashboard frontend — v1-AD-e (blocked on OQ-V1-AD-01)
- No SSE endpoint — v1-AD-d (blocked on OQ-V1-AD-02)
- No step-up auth infrastructure — v2
- No `rule_set.active_version_id` seed — deliberately absent (v1-AD-c activates)

## Next step

PR open from `phase-v1-AD-a` into `governance-v0`. Plan §20 sub-phase stubs (v1-AD-b/c/d/e) are the next `/prp-plan` inputs.

---

*Generated 2026-04-20 by /prp-ralph iteration 5 (final).*
