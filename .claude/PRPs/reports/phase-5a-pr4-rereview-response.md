# Response to CodeRabbit re-review (3 findings → down from 40)

Pushed as `9dbbbe14a` on `phase-5a`.

**Patched (1):**

- **`reputation_snapshot.rs:221`** — genuine correctness bug. `load_person_context` counted active sanctions without filtering by the snapshot's `community_id`, so a community-scoped snapshot was silently disqualified by any unrelated sanction from another community. Extended the helper to accept `Option<CommunityId>` and gate the count: community-scoped snapshots count only instance-wide (`target_community_id IS NULL`) or same-community sanctions; instance-scoped snapshots (`None`) count every active sanction. Passes `community_id` from `recompute_snapshot` to the helper.

**Not patched (rebuttals):**

- **`user/create.rs:141-157 / 397-409`** (CodeRabbit re-posted M10 as a duplicate) — plan-intended per **GOTCHA-51b** ([previous rebuttal](https://github.com/barrie-cork/lemmy/pull/4#issuecomment-4269497456)). The federated and non-federated register paths diverge in what they do *around* the config read, so extracting to a shared helper would entangle two intentionally-separate flows. The bot doesn't read rebuttals across re-reviews, so this is the same answer as before. No code change.

- **`config.rs:291-298`** — NULL-with-matching-value_type is DB-constraint-impossible. The CHECK constraint `governance_config_typed` (migration `up.sql:31-34`) enforces that `value_type = 'int'` requires `value_int IS NOT NULL AND value_float IS NULL AND value_bool IS NULL AND value_text IS NULL` (and so on for each type). The row-to-`CachedValue` mapping can therefore only hit its NULL arm if the DB has been corrupted outside the CHECK — at which point the `const_default_*` fallback is a desirable degradation rather than a silent-success bug. Escalating to a hard error adds complexity for a case the schema already prevents.

- **`config.rs:63-70`** — 🔵 trivial style. `Scope::as_str` allocates a `String` on every cache-probe. Real but tiny; bundling into the `refactor(config):` commit planned for Phase 5b (alongside the `Cow`-keyed cache and generic typed-accessor helper flagged in the earlier review — all same file, same cluster of style-level wins). Keeping phase-5a as close to the "governance_config infrastructure" scope as possible without scope creep into config-reader refactors.

**Validation:**

- `cargo check --features full -p lemmy_api` exit 0.
- `cargo clippy --features full -p lemmy_api --no-deps -- -D warnings` exit 0.
- `cargo test -p lemmy_server --test e2e` 9 passed / 0 failed.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
