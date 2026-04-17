**Actionable comments posted: 3**

<details>
<summary>♻️ Duplicate comments (1)</summary><blockquote>

<details>
<summary>crates/api/api_crud/src/user/create.rs (1)</summary><blockquote>

`141-157`: _🛠️ Refactor suggestion_ | _🟠 Major_

**Duplicate `default_membership_state` read block still present in both registration paths.**

The 11-line block is still copy-pasted verbatim between the `register` flow (141-157) and the OAuth new-user branch (397-409). If the key name, scope, fallback, or cache semantics ever change, the two registration paths will drift silently. Extract once:

<details>
<summary>♻️ Proposed refactor</summary>

```diff
+async fn read_default_membership_state(
+  pool: &mut lemmy_db_schema::utils::DbPool<'_>,
+) -> LemmyResult<lemmy_db_schema_file::enums::MembershipState> {
+  let mut cfg_cache = lemmy_api::governance::config::ConfigCache::new();
+  let raw = lemmy_api::governance::config::get_text(
+    &mut cfg_cache,
+    pool,
+    lemmy_api::governance::config::Scope::Instance,
+    "onboarding.default_membership_state",
+  )
+  .await?;
+  Ok(lemmy_api::governance::config::parse_membership_state(&raw))
+}
```

Then replace both inline blocks with `let default_membership_state = read_default_membership_state(pool).await?;`.
</details>





Also applies to: 397-409

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api_crud/src/user/create.rs` around lines 141 - 157, Extract the
duplicated 11-line block that computes default_membership_state into a single
async helper (e.g., async fn read_default_membership_state(pool: &PgPool) ->
Result<MembershipState, Error>) that performs ConfigCache::new(), calls
get_text(..., Scope::Instance, "onboarding.default_membership_state"), awaits
it, and returns parse_membership_state(&raw); then replace both inline blocks
(the one that assigns let default_membership_state in the register flow and the
OAuth new-user branch) with let default_membership_state =
read_default_membership_state(pool).await?; ensuring you import any needed types
(ConfigCache, Scope, parse_membership_state) and reuse pool and error
propagation consistently.
```

</details>

</blockquote></details>

</blockquote></details>

<details>
<summary>🤖 Prompt for all review comments with AI agents</summary>

```
Verify each finding against the current code and only fix it if needed.

Inline comments:
In `@crates/api/api/src/governance/config.rs`:
- Around line 291-298: The current row-to-CachedValue mapping in the closure
(the row.and_then(|(vtype, vi, vf, vb, vt)| ...)) silently returns None when the
value_type matches but the corresponding value column (e.g., value_int for
"int") is NULL; update that logic in the function that performs this mapping
(the closure around row.and_then / the fetch_value path in config.rs) to treat a
matching type with a NULL column as an explicit error instead of None: change
each arm (e.g., "int" => vi.map(CachedValue::Int)) to check presence and return
a distinct Err variant (e.g., ConfigError::CorruptRow or a dedicated CorruptRow
sentinel) when vi/vf/vb/vt is None but vtype matches, so callers get a hard
failure rather than falling back to DEFAULT_*; ensure the function signature and
upstream callers handle and propagate this Err (or the sentinel) accordingly.
- Around line 63-70: The current Scope::as_str consumes self and allocates a new
String on every call; rename the method (e.g., to to_scope_string or into_key)
if you want an allocating API, or change the signature to avoid allocation by
returning either &str for the "instance" case and allocating only for Community
via Cow<'static, str> (i.e., implement as fn scope_key(&self) -> Cow<'static,
str> or fn as_str(&self) -> &str with a separate to_string() for the Community
case); update all call sites that expect Scope::as_str to use the new name or
new return type and ensure you don’t repeatedly allocate on cache misses.

In `@crates/api/api/src/governance/reputation_snapshot.rs`:
- Line 221: The current load_person_context call returns active_sanctions
without scoping by the snapshot's community_id, so community-scoped snapshots
get disqualified by sanctions from other communities; change load_person_context
to accept an Option<CommunityId> parameter (e.g., community_filter:
Option<Uuid>) and update its sanction-count query to only include sanctions that
are either instance-wide (community_id IS NULL) or match the provided
community_filter when Some; when None preserve existing behavior (count all).
Update the call sites that use load_person_context (the calls that pass
person_id for snapshot processing) to pass the snapshot.community_id, and add a
short inline comment in load_person_context explaining the filtering rule.

---

Duplicate comments:
In `@crates/api/api_crud/src/user/create.rs`:
- Around line 141-157: Extract the duplicated 11-line block that computes
default_membership_state into a single async helper (e.g., async fn
read_default_membership_state(pool: &PgPool) -> Result<MembershipState, Error>)
that performs ConfigCache::new(), calls get_text(..., Scope::Instance,
"onboarding.default_membership_state"), awaits it, and returns
parse_membership_state(&raw); then replace both inline blocks (the one that
assigns let default_membership_state in the register flow and the OAuth new-user
branch) with let default_membership_state =
read_default_membership_state(pool).await?; ensuring you import any needed types
(ConfigCache, Scope, parse_membership_state) and reuse pool and error
propagation consistently.
```

</details>

<details>
<summary>🪄 Autofix (Beta)</summary>

Fix all unresolved CodeRabbit comments on this PR:

- [ ] <!-- {"checkboxId": "4b0d0e0a-96d7-4f10-b296-3a18ea78f0b9"} --> Push a commit to this branch (recommended)
- [ ] <!-- {"checkboxId": "ff5b1114-7d8c-49e6-8ac1-43f82af23a33"} --> Create a new PR with the fixes

</details>

---

<details>
<summary>ℹ️ Review info</summary>

<details>
<summary>⚙️ Run configuration</summary>

**Configuration used**: Path: .coderabbit.yaml

**Review profile**: ASSERTIVE

**Plan**: Pro

**Run ID**: `38500427-b6b8-42ce-bdd6-3ed7e3507d89`

</details>

<details>
<summary>📥 Commits</summary>

Reviewing files that changed from the base of the PR and between ca57b292c84af2d9ec16d237deb6035b39b7944f and 5910c3a040346eb58f3d8282f12947279161bd94.

</details>

<details>
<summary>⛔ Files ignored due to path filters (1)</summary>

* `Cargo.lock` is excluded by `!**/*.lock`, `!Cargo.lock`

</details>

<details>
<summary>📒 Files selected for processing (44)</summary>

* `.claude/PRPs/plans/phase-5a-config-and-reputation-infrastructure.plan.md`
* `.claude/PRPs/reports/phase-5a-complete-report.md`
* `.claude/PRPs/reports/phase-5a-handover-task-54-onward.md`
* `.claude/PRPs/reports/phase-5a-pr4-coderabbit-review.md`
* `.claude/PRPs/reports/phase-5a-pr4-handover-review-response.md`
* `.claude/decision-queue.json`
* `.claude/rules/phase-branch.md`
* `Cargo.toml`
* `crates/api/api/Cargo.toml`
* `crates/api/api/src/governance/config.rs`
* `crates/api/api/src/governance/governance_log.rs`
* `crates/api/api/src/governance/mod.rs`
* `crates/api/api/src/governance/reputation_snapshot.rs`
* `crates/api/api_common/src/governance.rs`
* `crates/api/api_crud/src/governance/create_endorsement.rs`
* `crates/api/api_crud/src/governance/mod.rs`
* `crates/api/api_crud/src/user/create.rs`
* `crates/api/routes/src/lib.rs`
* `crates/apub/objects/src/objects/person.rs`
* `crates/db_schema/src/impls/person.rs`
* `crates/db_schema/src/lib.rs`
* `crates/db_schema/src/newtypes.rs`
* `crates/db_schema/src/source/governance/governance_config.rs`
* `crates/db_schema/src/source/governance/mod.rs`
* `crates/db_schema/src/source/governance/reputation_snapshot.rs`
* `crates/db_schema/src/source/person.rs`
* `crates/db_schema_file/src/enums.rs`
* `crates/db_schema_file/src/schema.rs`
* `crates/db_views/registration_applications/src/impls.rs`
* `crates/db_views/reputation/Cargo.toml`
* `crates/db_views/reputation/src/impls.rs`
* `crates/db_views/reputation/src/lib.rs`
* `crates/db_views/vote/src/impls.rs`
* `crates/diesel_utils/src/pagination.rs`
* `crates/routes/Cargo.toml`
* `crates/routes/src/utils/scheduled_tasks.rs`
* `crates/server/src/governance.rs`
* `crates/server/tests/e2e.rs`
* `migrations/2026-04-18-000000-0000_add_governance_config/down.sql`
* `migrations/2026-04-18-000000-0000_add_governance_config/up.sql`
* `migrations/2026-04-18-000100-0000_add_person_membership_state/down.sql`
* `migrations/2026-04-18-000100-0000_add_person_membership_state/up.sql`
* `scripts/brehon/lint-no-can-sponsor-read.sh`
* `scripts/brehon/lint-no-membership-read.sh`

</details>

</details>

<!-- This is an auto-generated comment by CodeRabbit for review status -->
