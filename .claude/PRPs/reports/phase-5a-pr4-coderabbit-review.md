### .claude/decision-queue.json:56

_⚠️ Potential issue_ | _🟡 Minor_

**Decision-queue entries `#13` and `#14` appear in both `pending` and `resolved`.**

The Phase 5a completion report (§4) and handover (§4) both state that entries 13 and 14 were moved to `resolved`, yet the same ids are still present in the `pending` array (lines 31–56). Per `.claude/rules/decision-queue.md` discipline, a resolved entry should not remain in `pending`. This will cause downstream tooling/LLM sessions reading the queue to treat them as still-open.

<details>
<summary>🔧 Proposed fix</summary>

```diff
     },
-    {
-      "id": 13,
-      "from": "planner",
-      ...
-    },
-    {
-      "id": 14,
-      "from": "planner",
-      ...
-    },
     {
       "id": 4,
```
Remove the duplicated pending copies of ids 13 and 14; keep only the entries in the `resolved` array.
</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In @.claude/decision-queue.json around lines 31 - 56, The file contains
duplicate decision entries for id 13 and id 14 appearing in both the pending and
resolved arrays; remove the duplicate objects from the pending array so each id
only exists in resolved. Locate the JSON arrays named "pending" and "resolved",
find the objects with "id": 13 and "id": 14 in pending, delete those pending
entries (keeping the corresponding resolved entries intact), and validate the
JSON structure after removal.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### .claude/PRPs/reports/phase-5a-complete-report.md:25

_⚠️ Potential issue_ | _🟡 Minor_

**Unescaped `|` characters break the §1 Delivered table.**

Row 24 (task 55) contains `(age|open|closed)` and row 30 mentions `minor=-10|moderate=-50|severe=-200`; inside a GFM table, `|` is the column separator unless escaped. Static analysis flags this on line 24 (MD056: "Too many cells"). GitHub will render the row with mis-aligned cells and truncate content.

<details>
<summary>🔧 Proposed fix — escape pipes inside the cell</summary>

```diff
-| 55 | `POST /api/v4/governance/endorsement`: `create_endorsement.rs` handler with config-driven `SponsorGateStrategy` dispatch (`age|open|closed` + `Unknown → age` fallback) + ... | §12.6 |
+| 55 | `POST /api/v4/governance/endorsement`: `create_endorsement.rs` handler with config-driven `SponsorGateStrategy` dispatch (`age\|open\|closed` + `Unknown → age` fallback) + ... | §12.6 |
```

Apply the same escape to the severity-units line (§5 carry-forward bullet `minor=-10|moderate=-50|severe=-200`) if it also sits inside a table.
</details>

<!-- suggestion_start -->

<details>
<summary>📝 Committable suggestion</summary>

> ‼️ **IMPORTANT**
> Carefully review the code before committing. Ensure that it accurately replaces the highlighted code, contains no missing lines, and has no issues with indentation. Thoroughly test & benchmark the code to ensure it meets the requirements.

```suggestion
| Task | Deliverable | Plan §ref |
|------|-------------|-----------|
| 0 | Pre-phase audit (3 wrapper probes + 3 DoD dry-runs); plan-drift fixes; pagination lint carry-patch | §12.0 |
| 50 | `governance_config` table + Rust reader (`config.rs`) + 34 seed rows + structural parity test + DB round-trip (`config_parity_round_trip`) + `reputation_snapshot.can_sponsor` column + `threshold_score` micros rescale | §12.1 |
| 51 | `person.membership_state` column + `MembershipState` enum (`DbValueStyle = "snake_case"`) + `parse_membership_state` helper + `register()` handler patch + federated-upsert default + two grep-guard scripts | §12.2 |
| 52 | `crates/db_views/reputation` view crate with `ReputationSummaryView` + `EndorsementSummaryView` + three tuple-load queries, no `Selectable` derive | §12.3 |
| 53 | `reputation_snapshot.rs` (~700 lines incl. 4 unit tests) with `recompute_snapshot`, `run_snapshot_batch` (chunked per config), `detect_capability_changes`; `ENTRY_KIND_*` const block (15 entries) in `governance_log.rs`; `mod.rs` wiring | §12.4 |
| 54 | `run_snapshot_batch` registered via clokwerk in `scheduled_tasks.rs` (15-min tick) + `BREHON_DISABLE_BACKGROUND_JOBS=1` override + `lemmy_api` dep added to `routes/Cargo.toml` + `governance.rs` stub-message update | §12.5 |
| 55 | `POST /api/v4/governance/endorsement`: `create_endorsement.rs` handler with config-driven `SponsorGateStrategy` dispatch (`age\|open\|closed` + `Unknown → age` fallback) + federated-upsert default + two grep-guard scripts | §12.6 |
| 56 | Level 0–5 validation + `report_to_modlog_golden_path` + lint guards + this report + PR | §12.7 |
```

</details>

<!-- suggestion_end -->

<details>
<summary>🧰 Tools</summary>

<details>
<summary>🪛 markdownlint-cli2 (0.22.0)</summary>

[warning] 24-24: Table column count
Expected: 3; Actual: 5; Too many cells, extra data will be missing

(MD056, table-column-count)

</details>

</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In @.claude/PRPs/reports/phase-5a-complete-report.md around lines 16 - 25, The
table row containing "(age|open|closed)" and the cell with
"minor=-10|moderate=-50|severe=-200" use raw pipe characters that break the GFM
table; edit the markdown so those pipe characters are escaped (replace | with
\|) or wrap the entire cell content in inline code backticks, e.g., update the
cell text for the SponsorGateStrategy entry "(age\|open\|closed)" and the
severity units "minor=-10\|moderate=-50\|severe=-200" wherever they appear in
the Delivered table to restore correct column alignment.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### .claude/PRPs/reports/phase-5a-complete-report.md:44

_🧹 Nitpick_ | _🔵 Trivial_

**Placeholder code fence will not be filled automatically.**

The `<to be filled at PR-open time via \`git log ...\`>` placeholder inside the fenced block is a manual TODO that is easy to miss at PR-open time. Either fill it in now, remove the fence if the commit list below is authoritative, or add a BUG/TODO marker with a checklist item so the phase-close checklist catches it.


Want me to open a tracked checklist item (or generate the filled commit list from the PR head) to close this out before merge?

<details>
<summary>🧰 Tools</summary>

<details>
<summary>🪛 markdownlint-cli2 (0.22.0)</summary>

[warning] 42-42: Fenced code blocks should have a language specified

(MD040, fenced-code-language)

</details>

</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In @.claude/PRPs/reports/phase-5a-complete-report.md around lines 42 - 44, The
fenced placeholder in phase-5a-complete-report.md containing "<to be filled at
PR-open time via `git log --oneline origin/governance-v0..HEAD`>" must be
resolved: either replace the fenced block with the actual git log output from
the PR head, remove the fence if the surrounding text is authoritative, or add a
visible BUG/TODO checklist entry so the phase-close checklist will catch it;
locate the placeholder by searching for that exact string in
phase-5a-complete-report.md and update the file accordingly before merging.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### .claude/PRPs/reports/phase-5a-handover-task-54-onward.md:300

_⚠️ Potential issue_ | _🟡 Minor_

**Absolute Windows paths leak author-local filesystem layout.**

`C:\Users\barri\Developer\homeserver\.claude\...` appears twice in §9. These are not reachable from other contributors' machines or from CI. Replace with repo-relative references (or clearly flag as "advisor-local, not in repo") so the handover is actionable for a fresh session or a different contributor.

<details>
<summary>🔧 Proposed fix</summary>

```diff
-3. **`C:\Users\barri\Developer\homeserver\.claude\advisor-context-phase-5.md` §4 + §5 only** — watchpoints and operational rules; ...
+3. **`<advisor-local>/.claude/advisor-context-phase-5.md` §4 + §5 only** (advisor-managed; not in this repo) — watchpoints and operational rules; ...
...
-- `C:\Users\barri\Developer\homeserver\.claude\memory\` (advisor-managed; out of scope for impl).
+- `<advisor-local>/.claude/memory/` (advisor-managed; out of scope for impl).
```
</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In @.claude/PRPs/reports/phase-5a-handover-task-54-onward.md around lines 291 -
300, The handover contains absolute Windows path strings in section §9 (the
advisor-local .claude references) which leak a user-local layout; edit the
report to replace those absolute paths with repo-relative references or
explicitly tag them as "advisor-local, not in repo" so other contributors/CI can
follow the handover, and update any repeated occurrences to the same normalized
form (search for the .claude advisor references in §9 and replace consistently).
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api_crud/src/governance/create_endorsement.rs:133

_⚠️ Potential issue_ | _🟡 Minor_

**`actor_pseudonym_helper::get_or_create` for the sponsor runs outside the endorsement transaction.**

`sponsor_pseudonym` is resolved on the outer pool (Line 127) before `conn` is checked out for the tx. If the sponsor has no `actor_pseudonym` row yet, a row is created on a different connection and committed immediately, but all subsequent work runs in a new tx. If that tx aborts (e.g. gate closed at Line 173, cap reached, etc.) the pseudonym insert remains, which is a subtle side-effect for a rejected endorsement attempt.

Consider resolving/creating the sponsor pseudonym inside the same tx (mirroring the target pseudonym lookup at Line 315), so a rejected request leaves no trace.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api_crud/src/governance/create_endorsement.rs` around lines 125 -
133, The sponsor pseudonym is created outside the endorsement transaction
(actor_pseudonym_helper::get_or_create is called before checking out conn),
causing a committed side-effect even if the tx later aborts; move the sponsor
pseudonym resolution/creation into the same transaction that uses conn (mirror
the target lookup at Line 315) so that actor_pseudonym_helper::get_or_create is
invoked using the transaction/connection obtained from get_conn (instead of
context.pool()) and assign the result to sponsor_pseudonym/pseudonym_for_tx
inside the tx scope.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api_crud/src/governance/create_endorsement.rs:214

_🛠️ Refactor suggestion_ | _🟠 Major_

**Every failure path collapses to `LemmyErrorType::NotFound`, destroying observability.**

Closed gate (Line 173), self-endorsement (Line 187), cap reached (Line 199), cooldown active (Line 213), and age-gate failure (Line 358) all return `NotFound`. Enumeration-resistance for "target exists" is a fine reason to use 404 at Line 187/Line 189, but the cap, cooldown, and gate cases are strictly about the caller's own state and leak nothing — mapping them to 404 makes client UX (no useful error message) and ops debugging (no distinguishing log signal) significantly worse.

Introduce distinct `LemmyErrorType` variants (e.g. `SponsorGateClosed`, `EndorsementCapReached`, `EndorsementCooldownActive`, `SponsorAgeBelowThreshold`) so the response body and the scrubbed log carry the real reason. Keep `NotFound` only for the "target person does not exist / self-endorse" cases where enumeration resistance matters.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api_crud/src/governance/create_endorsement.rs` around lines 170 -
214, Change the generic NotFound error returns in create_endorsement to distinct
LemmyErrorType variants so callers and logs can distinguish
gate/cap/cooldown/age failures: replace the return in the
SponsorGateStrategy::Closed branch with a new SponsorGateClosed error; have
enforce_age_gate propagate or map its failures to SponsorAgeBelowThreshold;
replace the active_count >= MAX_ACTIVE_ENDORSEMENTS return with
EndorsementCapReached; replace the recent_count > 0 return (using
ENDORSEMENT_COOLDOWN_HOURS) with EndorsementCooldownActive; keep NotFound for
the Person::read/self-endorsement path to preserve enumeration resistance.
Ensure new LemmyErrorType variants are defined and used consistently where
create_endorsement, enforce_age_gate, and related checks return errors.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api_crud/src/governance/create_endorsement.rs:214

_⚠️ Potential issue_ | _🟠 Major_

**TOCTOU: the caller-side cap and cooldown checks are raceable under default isolation.**

The active-endorsement cap (Lines 192–200) and the 48 h cooldown (Lines 205–214) are plain `SELECT count(*)` reads with no row-level lock on the caller and no advisory lock on `sponsor_id`. Under Postgres' default `READ COMMITTED`, two concurrent endorsement requests from the same caller can both observe `active_count = 4` and `recent_count = 0`, then both proceed to `insert_into(endorsement::table)` at Line 222, producing 6 active endorsements and collapsing the cooldown to zero. `run_transaction` does not upgrade isolation.

Take a per-caller advisory lock at the top of `process_endorsement` (e.g. `pg_advisory_xact_lock(hash(sponsor_id))`), mirroring the pattern used in `reputation_snapshot::acquire_advisory_xact_lock`, so the cap/cooldown checks serialise per sponsor.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api_crud/src/governance/create_endorsement.rs` around lines 191 -
214, The active-count and cooldown SELECTs in process_endorsement are raceable;
take a per-caller Postgres advisory transaction lock at the start of
process_endorsement (before the active_count and recent_count queries) to
serialize endorsement attempts for a given sponsor_id (e.g. call the same
pattern as reputation_snapshot::acquire_advisory_xact_lock using
pg_advisory_xact_lock(hash(sponsor_id))). Ensure the lock acquisition occurs
inside the same DB transaction scope so it is released automatically at
transaction end and keep the existing count checks and insert logic unchanged.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api_crud/src/governance/create_endorsement.rs:225

_⚠️ Potential issue_ | _🟠 Major_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
# Confirm the current state of the endorsement unique constraint.
rg -nP -C2 'CREATE TABLE endorsement\b' --type=sql
rg -nP -C5 'UNIQUE\s*\([^)]*from_person_id' --type=sql
```

Repository: barrie-cork/lemmy

Length of output: 1810

---

🏁 Script executed:

```shell
cat -n crates/api/api_crud/src/governance/create_endorsement.rs
```

Repository: barrie-cork/lemmy

Length of output: 14544

---



</details>

**Address re-endorsement blocking on revoked endorsements: either allow re-endorsement or surface explicit error.**

The schema has a full `UNIQUE (from_person_id, to_person_id, community_id)` constraint. When an endorsement is revoked (soft delete via `revoked_at`), the row persists, causing subsequent re-endorsement attempts to fail with `UniqueViolation`. The code (Step 6, lines 222–225) lacks a pre-check and will propagate the database constraint violation as an opaque `LemmyError` instead of a user-friendly response.

Choose one:
- Change the unique constraint to partial: `UNIQUE (from_person_id, to_person_id, community_id) WHERE revoked_at IS NULL` to permit re-endorsement.
- Pre-check for revoked endorsements (query `revoked_at IS NOT NULL`) and return an explicit error before the INSERT.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api_crud/src/governance/create_endorsement.rs` around lines 216 -
225, The INSERT into endorsement::table using EndorsementInsertForm can hit the
UNIQUE constraint because revoked endorsements remain (revoked_at IS NOT NULL)
and cause an opaque UniqueViolation; add a pre-check query before calling
insert_into(endorsement::table) that searches for an existing row with
from_person_id == sponsor_id, to_person_id == data.person_id, community_id ==
data.community_id and revoked_at IS NOT NULL, and if found return a clear
LemmyError (e.g., "cannot re-endorse revoked endorsement" or similar) instead of
letting the INSERT fail; alternatively, if you prefer schema change, update the
DB migration to make the UNIQUE constraint partial (UNIQUE (...) WHERE
revoked_at IS NULL) so re-endorsement is allowed and document that choice.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api_crud/src/governance/create_endorsement.rs:248

_⚠️ Potential issue_ | _🟠 Major_

**Surety cap check is TOCTOU on the sponsee.**

Same category of bug as the caller-side cap: two concurrent endorsers of the same sponsee can both observe `active_sureties = 1` and both insert, yielding 3 active sureties. Either gate this on the same advisory lock keyed on `data.person_id`, or rely on a partial unique index / exclusion constraint on `surety(sponsored_id) WHERE revoked_at IS NULL` with an appropriate cardinality enforcement at the DB layer.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api_crud/src/governance/create_endorsement.rs` around lines 227 -
248, The TOCTOU bug is in the cap check around active_sureties /
SuretyInsertForm insertion in create_endorsement: either acquire a per-sponsee
advisory lock (keyed by data.person_id) before reading active_sureties and
holding it until after insert_into(surety::table) completes, or enforce the
limit at the DB (add a partial unique/index or exclusion constraint on
surety(sponsored_id) WHERE revoked_at IS NULL for the allowed cardinality) and
change the insert path to handle unique/constraint violation errors from
insert_into(surety::table) by treating them as "cap reached" instead of
crashing; update the code around active_sureties, SuretyInsertForm, and the
insert_into call to use one of these two approaches so concurrent endorsers
cannot exceed MAX_ACTIVE_SURETIES_PER_SPONSEE.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api_crud/src/user/create.rs:157

_🛠️ Refactor suggestion_ | _🟠 Major_

**Deduplicate the `default_membership_state` read block.**

The exact same 11-line block is repeated verbatim in `register` (lines 141-157) and in the OAuth-new-user branch (lines 397-409). If the key name, scope, or fallback semantics ever change, the two registration paths will drift silently — and both flows are user-facing. Extract a small helper and call it from both sites.

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
duplicated 11-line block that reads and parses
"onboarding.default_membership_state" into a single async helper (e.g.
read_default_membership_state) that constructs ConfigCache, calls
lemmy_api::governance::config::get_text with Scope::Instance, and returns
lemmy_api::governance::config::parse_membership_state(&raw) (propagate errors).
Replace the inline blocks in the register flow and the OAuth-new-user branch
with let default_membership_state = read_default_membership_state(pool).await?;
so both paths reuse the same logic and ConfigCache usage.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/config.rs:127

_🧹 Nitpick_ | _🔵 Trivial_

**Cache probe allocates `(String, String)` on every read.**

`cache.entries.get(&(scope_repr.clone(), key.to_string()))` allocates two `String`s per lookup just to borrow them as the probe key, on every `get_int`/`get_float`/`get_bool`/`get_text` call — repeated ≥30 times during a config-parity test and in snapshot recomputes that read 5+ keys each. Key the cache by `(&str, &str)` via a `HashMap<(Cow<'static, str>, Cow<'static, str>), _>` or use `hashbrown`'s `raw_entry`/`get_key_value` with a borrowed probe tuple. Pure refactor.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/config.rs` around lines 99 - 127, The cache
probe currently allocates two Strings per lookup in functions like get_int (and
analogous get_float/get_bool/get_text) by calling
cache.entries.get(&(scope_repr.clone(), key.to_string())); change ConfigCache to
avoid those allocations by either (A) changing the map key to (Cow<'static,
str>, Cow<'static, str>) and store owned Cows on insert while probing with
borrowed &str/&str, or (B) switching to hashbrown::HashMap and use
raw_entry/get_key_value with a borrowed probe tuple to lookup without
allocating; update get_int (and the other getters) to construct a borrowed probe
(&str, &str) for the get, and ensure insertions create owned Cows for storage so
existing behavior remains correct.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/config.rs:217

_🧹 Nitpick_ | _🔵 Trivial_

**Four near-identical typed accessors — worth collapsing.**

`get_int`, `get_float`, `get_bool`, and `get_text` are 29-line copy-paste, each differing only in the `CachedValue` variant, the `const_default_*` helper, and the "requested as X but stored as Y" wording. A single generic helper parameterised by a small trait (or a closure taking `CachedValue -> Option<T>` plus the `const_default_*` function pointer) removes ~90 lines and eliminates the drift risk when a fifth value type lands in v1.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/config.rs` around lines 99 - 217, The four
functions get_int, get_float, get_bool and get_text duplicate logic; collapse
them into one generic helper to avoid copy/paste drift by factoring the common
flow (lookup in ConfigCache.entries, fetch_value(pool, scope, key).await?, error
on wrong variant, fallback to const_default_*, insert into cache, return value).
Implement a single helper (e.g. get_typed<T> or get_with) that accepts: the
scope/key and cache/pool, a matcher closure Fn(&CachedValue) -> Option<T> to
extract the variant (matching CachedValue::Int/Float/Bool/Text), and a
const_default function pointer (const_default_int/float/bool/text) to provide
the fallback; update get_int/get_float/get_bool/get_text to call this helper
with the appropriate matcher and const_default_* functions so CachedValue,
fetch_value, const_default_* and ConfigCache.entries are reused and behavior
preserved.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/config.rs:236

_⚠️ Potential issue_ | _🟠 Major_

**Silent fallback to `Member` on unknown membership state masks config corruption.**

If an admin typo'd `"memeber"` (or any future migration introduced a fourth variant the reader hasn't learned about), every new registration would default to `Member` — the most privileged v0 state — with only a log line to notice. Given the seed list is closed and the CHECK constraint on `governance_config.value_text` is advisory, a stricter failure mode is warranted: return `LemmyResult<MembershipState>` with an error on unknown text. Callers can then reject the request and surface it in ops.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/config.rs` around lines 226 - 236, The function
parse_membership_state should stop silently falling back to Member on unknown
input; change its signature from parse_membership_state(s: &str) ->
MembershipState to return a Result/LemmyResult (e.g.,
LemmyResult<MembershipState>) and return an Err when the input doesn't match
"member", "provisional", or "suspended". Update the match arm for the unknown
case to construct a clear error (including the unknown string) instead of
logging+returning Member, and propagate this new error type to callers of
parse_membership_state so they can reject invalid configs or surface the error
to ops.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/config.rs:298

_🧹 Nitpick_ | _🔵 Trivial_

**Community-scoped reads always do two DB round-trips on a cache miss.**

`fetch_value` on a `Community` scope first runs a `SELECT` at community scope, and if that returns `None` runs a second `SELECT` at instance scope — each one acquiring its own connection via `fetch_value_at_scope → get_conn(pool)`. Since v0 writes no community-scoped rows, every community-scoped read pays the cost of a guaranteed miss + fallback. A single query `WHERE scope IN (community_str, 'instance') ORDER BY CASE scope WHEN community_str THEN 0 ELSE 1 END LIMIT 1` (or a `UNION ALL` with priority) collapses it to one round-trip. Also, negative caching (insert `None` for community scope when the fallback hit instance) would avoid paying the miss on subsequent reads of the same key.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/config.rs` around lines 256 - 298, fetch_value
currently does two DB round-trips for Community scope because it calls
fetch_value_at_scope twice (each calls get_conn); change it to perform a single
query that looks up both the community scope and the instance scope in one
round-trip and returns the higher-priority row: obtain a single connection via
get_conn once in fetch_value (or in an updated fetch_value_at_scope), query
governance_config_current with scope IN (community_str, Instance) and ORDER BY
CASE WHEN scope = community_str THEN 0 ELSE 1 END LIMIT 1 (or equivalent UNION
ALL with priority), map the returned row into CachedValue as before, and then
(optionally) implement negative caching by inserting a cached "None" for the
community scope when only the instance row exists so subsequent community reads
avoid the miss; update fetch_value, fetch_value_at_scope and any callsites to
use the new single-query logic and still return
LemmyResult<Option<CachedValue>>.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/reputation_snapshot.rs:142

_🧹 Nitpick_ | _🔵 Trivial_

**Closure unused in the `None` branch.**

The `check` closure defined on Line 131 is only used in the `Some(old)` arm; the `None` arm at Lines 143-166 inlines three near-identical `if new.x { push(...) }` blocks. You can express the `None` branch as `check(false, new.jury_eligible, ...)` etc., collapsing both arms to a single `let old_ref = old.unwrap_or(&DEFAULTS);` style. Purely stylistic.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/reputation_snapshot.rs` around lines 131 - 142,
The closure check is defined but only used in the Some(old) arm while the None
arm duplicates its logic; replace the match arms by computing a reference to the
previous values (e.g. let old_ref = old.unwrap_or(&DEFAULTS)) and then call the
check closure for each capability (e.g. check(old_ref.jury_eligible,
new.jury_eligible, CapabilityDimension::JuryEligible, &mut changes), etc.) so
both branches share the same comparison logic and eliminate the duplicated
if/push blocks.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/reputation_snapshot.rs:335

_⚠️ Potential issue_ | _🟡 Minor_

**Single-step halving means decay stops after one half-life.**

`compute_applied_delta` halves the delta exactly once when `age > half_life` and thereafter never decays further. An event 5×half-life old contributes the same as one at 1.01×half_life. The doc-comment calls this out ("more aggressive schedules … are v1"), but combined with the watermark issue flagged above it means a positive organic event from 400 days ago will pin `endorsement_strength` at `delta/2` essentially forever — gating `can_sponsor` on stale data.

If this is intentional for v0, add a ticketed TODO + a regression test so the decision is revisited before v1 turns on the sponsor gate.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/reputation_snapshot.rs` around lines 321 - 335,
compute_applied_delta currently halves positive deltas only once (age >
half_life) and never decays further; add an explicit ticketed TODO comment in
compute_applied_delta stating that single-step halving is intentional for v0
(include a ticket/issue number placeholder like TICKET-XXXX) and that v1 should
implement chained halving per half-life, and then add a regression test (e.g.,
test_compute_applied_delta_single_step_halving) that verifies a very-old event
(e.g., age >> half_life) still yields only original/2 (not further decay) so
this behavior is revisited before enabling v1; reference symbols: function
compute_applied_delta, variables age, half_life, original, and event.expires_at.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/reputation_snapshot.rs:609

_⚠️ Potential issue_ | _🔴 Critical_

**Dirty-pair detection misses time-based decay and config-cascade flips.**

`load_dirty_pairs` only marks a pair dirty when a new `reputation_event` row appears, or when an event's `expires_at` falls inside `(watermark, now()]`. Two legitimate cascades are silently dropped:

1. **Half-life decay.** A positive organic event crosses the 90-day half-life without any new event or expiry firing. Snapshots of quiet users grow stale and their capability booleans can be wrong for arbitrarily long.
2. **Admin config edits** — the module docs explicitly claim (Watch 11): *"An admin raising `config.thresholds.jury_reliability` will cascade into many `capability_changed` entries on the next tick."* With this query, it does not — no event rows change, so no pairs are dirty, and the cascade never runs.

Either include a "stale-by-age" predicate (`max(calculated_at) < now() - interval '<decay_half_life_days>'`) or a periodic full-refresh branch, and trigger a one-shot full re-scan when a `thresholds.*` / `decay.*` config row is written. Otherwise the advertised semantics do not hold.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/reputation_snapshot.rs` around lines 573 - 609,
load_dirty_pairs only marks pairs dirty when reputation_event rows are new or
expire, so it misses pairs that become stale due to time-based decay (half-life)
or when admin config (thresholds/decay) changes; modify load_dirty_pairs to also
mark pairs dirty when their latest snapshot is older than the decay window or
when a config-change triggers a full rescan: compute a "stale" cutoff (e.g. now
- decay_half_life_days) and include an OR predicate comparing the
max(calculated_at) per (person_id,community_id) against that cutoff (or, if
easier, add a periodic full-refresh branch gated by a config flag), and ensure a
config write path triggers a one-shot rescan that forces all pairs to be
considered dirty; look for the symbols load_dirty_pairs,
reputation_event::created_at, reputation_event::expires_at,
reputation_snapshot::calculated_at, watermark/wm to add the new predicate or the
full-rescan trigger.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/reputation_snapshot.rs:590

_⚠️ Potential issue_ | _🟠 Major_

**Global-max watermark undercounts dirty pairs when any single pair is recent.**

`watermark = MAX(calculated_at)` is a single scalar across the whole table. If *one* pair was just recomputed synchronously by `create_endorsement`, the watermark advances to "now", and `load_dirty_pairs` will find nothing for every other pair whose events predate that recent computation, even though those other pairs may not have been recomputed in ages. Consider storing a per-pair watermark (`max(calculated_at) per (person_id, community_id)`) via a correlated subquery, or a dedicated `job_watermark` row keyed by job name.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/reputation_snapshot.rs` around lines 583 - 590,
Current code uses a single global watermark computed from
reputation_snapshot::table.select(max(reputation_snapshot::calculated_at)) and
stores it in wm, which causes undercounting when any single (person_id,
community_id) pair was recently recomputed; change the approach to compute and
use a per-pair watermark instead of the global max: update the query that
populates wm (and any callers like load_dirty_pairs) to compute
max(calculated_at) grouped or correlated by (person_id, community_id) (or
introduce a job_watermark keyed by job name) so that load_dirty_pairs correctly
detects stale pairs even when one pair was recently updated (refer to
reputation_snapshot::calculated_at, reputation_snapshot::table,
load_dirty_pairs, and create_endorsement to locate places to change).
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/api/src/governance/reputation_snapshot.rs:657

_⚠️ Potential issue_ | _🟡 Minor_

**Advisory-lock key can collide across `(person_id, community_id)` pairs.**

`(i64::from(person_id.0) << 32) | i64::from(community_id.map(|c| c.0).unwrap_or(0))` uses 0 as the sentinel for "no community", so a person with `person_id = P` and `community_id = None` collides with `person_id = P` and `community_id = Some(CommunityId(0))`. `SERIAL`/`SERIAL PRIMARY KEY` in Postgres starts at 1 so there is no live id 0 today, but relying on that is fragile. Also, if `community_id.0` is ever negative (e.g. sentinel), the OR will clobber the upper 32 bits of `person_id`.

Use a safe packing: reserve a bit for the None case (e.g. shift in the low 31 bits and set the high bit of the low half to 1 when `community_id` is `Some`), or hash the `(PersonId, Option<CommunityId>)` tuple with a stable hasher.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/api/src/governance/reputation_snapshot.rs` around lines 651 - 657,
The advisory-lock key packing using bit-shifts on person_id and community_id can
collide (None vs Some(0)) and corrupt bits if ids are negative; replace the
inline pack that defines key with a deterministic 64-bit hash of the tuple
(person_id.0, community_id.map(|c| c.0)) instead, e.g. compute a stable u64 via
a stable hash function (blake3/xxhash64) over the two integers and then cast to
i64 for binding; update the key variable (the binding to pg_advisory_xact_lock
and the sql_query call that uses key) to use that hashed i64 so locks are unique
and stable across processes.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/api/routes/src/lib.rs:501

_⚠️ Potential issue_ | _🟡 Minor_

**Consider applying a rate limiter to the endorsement endpoint.**

`create_endorsement` performs transactional writes (endorsement/surety/events) plus snapshot recomputes and governance-log emits, which is expensive. Neither `/governance/report` nor the new `/governance/endorsement` route is wrapped by any `rate_limit.*` middleware, unlike the other user-facing write endpoints (e.g. `/comment`, `/post`, `/account/auth/*`). Worth confirming this is intentional for a write endpoint that triggers reputation side-effects — abuse here could both spam the governance log and DoS the snapshot job.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/api/routes/src/lib.rs` around lines 498 - 501, Add rate limiting to
the governance endorsement endpoint: wrap the /governance endorsement route (the
scope("/governance") or specifically the route("/endorsement",
post().to(create_endorsement))) with the same rate_limit middleware used for
other write endpoints (e.g., /comment, /post, /account/auth/*), or apply a
per-route limiter on create_endorsement; configure the limiter to match the
existing write-endpoint policy (requests per window, burst, etc.) so the
transactional writes, snapshot recomputes, and governance-log emits are
protected from abuse and DoS.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:2e3ecca3-9a9f-46bd-8039-5d92a6ed2fd1 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/apub/objects/src/objects/person.rs:181

_⚠️ Potential issue_ | _🟠 Major_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
# Inspect PersonInsertForm definition and the upsert impl to confirm on-conflict behaviour for membership_state.
fd -t f 'person.rs' crates/db_schema/src | xargs -I{} sh -c 'echo "=== {} ==="; cat {}'
rg -nP -C4 '\bfn\s+upsert\b' crates/db_schema/src
rg -nP -C3 'membership_state' crates/db_schema crates/db_schema_file
```

Repository: barrie-cork/lemmy

Length of output: 37716

---

🏁 Script executed:

```shell
# Search for tests or documentation about membership_state in upsert context
rg -nP 'membership_state' crates/apub/objects/src/objects/person.rs -A 3 -B 3

# Check for any tests that verify upsert preserves values across refreshes
rg -nP 'test.*upsert|upsert.*test' crates/db_schema/src/impls/person.rs -A 10

# Check Diesel patterns: are there other fields in InsertForm but NOT in UpdateForm that get set in upsert?
rg -nP 'pub struct PersonUpdateForm' crates/db_schema/src -A 20

# Look for similar upsert patterns in codebase to understand the intended behaviour
rg -nP 'do_update\(\)\.set\(' crates/db_schema/src/impls/ -B 3 -A 1 | head -50
```

Repository: barrie-cork/lemmy

Length of output: 1956

---

🏁 Script executed:

```shell
# Check if there's Diesel documentation or test code showing how Option<T> with #[new(default)] behaves in do_update().set()
rg -nP 'AsChangeset|do_update' crates/db_schema/src/source/person.rs -B 2 -A 2

# Look for tests that exercise upsert with membership_state
rg -nP 'upsert.*person|person.*upsert' crates/ -l | head -10 | xargs -I {} sh -c 'echo "=== {} ===" && grep -n "membership_state" {} || echo "no membership_state found"'

# Check Diesel version and any related configuration
fd -t f 'Cargo.toml' crates/db_schema | head -3 | xargs rg -A 5 'diesel'

# Look at the actual from_json call that leads to upsert to understand the full context
rg -nP 'from_json' crates/apub/objects/src/objects/person.rs -B 5 -A 15
```

Repository: barrie-cork/lemmy

Length of output: 4122

---

🏁 Script executed:

```shell
# Search for Diesel documentation or patterns showing how Option<T> is handled in do_update().set()
# Look at how other upserts in the codebase handle Optional fields

rg -nP 'do_update.*\.set\(form\)' crates/db_schema/src/impls/ -B 5 -A 2

# Check if there are any database migrations or schema documentation about membership_state column
fd -t f 'migrations' crates/ | head -5
rg -r 'membership_state' crates/db_schema/ --type sql 2>/dev/null || echo "No SQL files found"

# Look for comment patterns that explain when/why fields are excluded from UpdateForm
rg -nP 'membership_state|UpdateForm' crates/db_schema/src/source/person.rs -B 2 -A 1 | grep -E 'pub struct|pub [a-z_]+:|///'

# Check the actual from_apub conversion to see what fields are being set in the PersonInsertForm
rg -nP 'PersonInsertForm|membership_state' crates/apub/objects/src/objects/person.rs -B 2 -A 2
```

Repository: barrie-cork/lemmy

Length of output: 1302

---

🏁 Script executed:

```shell
# Check the Diesel version to understand Option<T> behaviour in AsChangeset/do_update
grep -A 5 'workspace' crates/db_schema/Cargo.toml | head -20

# Look at workspace dependencies to find Diesel version
rg 'diesel.*=' Cargo.toml --type toml | head -10

# Search for any explicit handling of membership_state in the upsert or from_json context
rg -nP 'membership_state|upsert' crates/apub/objects/src/objects/person.rs -C 5

# Look for Diesel documentation comments or patterns about Option<T> in AsChangeset in the codebase
rg -nP 'AsChangeset|Option.*None|do_update' crates/db_schema/src/ -C 2 | grep -A 5 -B 5 "membership\|Option" | head -40
```

Repository: barrie-cork/lemmy

Length of output: 3455

---

🏁 Script executed:

```shell
# Search for Diesel documentation or usage patterns in the codebase that show how Option<T> is handled in do_update
# Look for explicit tests that verify upsert preserves values

rg -nP 'on_conflict.*do_update' crates/ -A 10 | grep -A 10 'do_update' | head -50

# Check if there are any comments or documentation about the expected behavior of do_update with Option fields
rg -nP 'Option.*None|skip|default' crates/db_schema/src/impls/ | grep -i update

# Look at how the person form is actually constructed in the from_json call to see all fields set
rg -nP 'PersonInsertForm \{' crates/apub/objects/src/objects/person.rs -A 30
```

Repository: barrie-cork/lemmy

Length of output: 1273

---

🏁 Script executed:

```shell
# Check Diesel 2.3.7 documentation or look for any skip_update attributes used in PersonInsertForm or similar forms
rg -nP '#\[diesel\(' crates/db_schema/src/source/person.rs

# Search for any tests that verify upsert preserves values across refresh cycles
rg -nP 'test.*upsert|fn.*test' crates/apub/objects/src/objects/person.rs -A 15 | grep -A 15 'upsert\|membership'

# Look for Diesel AsChangeset patterns with Option<T> to understand expected behaviour
rg -nP 'AsChangeset.*Option|#\[diesel.*skip' crates/db_schema/src/ | head -20

# Check if there's any explicit documentation in Lemmy's code about how None in Option fields is handled in do_update
rg -nP 'do_update|on_conflict' crates/db_schema/src/impls/person.rs -B 2 -A 5
```

Repository: barrie-cork/lemmy

Length of output: 1766

---



</details>

**Fix upsert to preserve locally-assigned `membership_state` on federation refresh.**

The `Person::upsert()` implementation passes `PersonInsertForm` directly to `.set()`, including `membership_state: None`. This resets the column to its SQL default on every refresh, overwriting any locally-assigned membership state. The fact that `PersonUpdateForm` explicitly omits this field indicates it should not be updated in normal update paths; the upsert should honour the same constraint.

Either exclude `membership_state` from the conflict update, use a separate upsert form without this field, or conditionally set only remote-sourced fields. Locally-enforced governance state must persist across federation refreshes.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/apub/objects/src/objects/person.rs` around lines 177 - 181,
Person::upsert currently calls .set(PersonInsertForm { ... , membership_state:
None, ... }) which overwrites any locally-assigned membership_state on conflict;
change the upsert so that the conflict UPDATE does not touch membership_state
(mirror the omission in PersonUpdateForm) — either build a separate "upsert
without membership_state" struct or call .on_conflict(...).do_update().set(...)
with only the remote-sourced fields (exclude membership_state), or conditionally
apply membership_state only when coming from remote authority; ensure
Person::upsert, PersonInsertForm and the conflict/update path no longer reset
membership_state.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:2e3ecca3-9a9f-46bd-8039-5d92a6ed2fd1 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_schema_file/src/schema.rs:442

_🧹 Nitpick_ | _🔵 Trivial_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
# Dump the CREATE VIEW statement and confirm column order aligns with the
# table! block above (id, scope, key, value_type, value_int, value_float,
# value_bool, value_text, valid_from, updated_by).
fd -t f -e sql governance_config
rg -nP -A20 'CREATE\s+(OR\s+REPLACE\s+)?VIEW\s+governance_config_current' --type=sql
```

Repository: barrie-cork/lemmy

Length of output: 2735

---



</details>

**Consider adding a comment to pin the `governance_config_current` view column order.**

The column order in the current `CREATE VIEW` definition matches the `table!` macro exactly (`id, scope, key, value_type, value_int, value_float, value_bool, value_text, valid_from, updated_by`). However, since Diesel deserialises view columns by position rather than by name, any future refactoring of the view's `CREATE VIEW` statement could silently mismap values to fields. Adding a comment in the `table!` block (e.g. referencing the migration file or pinning the invariant) would help prevent accidental reordering during future schema maintenance.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_schema_file/src/schema.rs` around lines 411 - 442, Add a short
explanatory comment above the diesel::table! block for governance_config_current
that documents the column order is pinned to the corresponding CREATE VIEW in
the migration (so Diesel deserialises by position), reference the migration file
or its migration name/identifier, and state that columns must not be reordered
in the view without updating this table! declaration; update the comment near
governance_config_current to make the invariant explicit.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_schema/src/source/governance/governance_config.rs:46

_🛠️ Refactor suggestion_ | _🟠 Major_

**`AsChangeset` derive contradicts append-only config history.**

The migration-level design (and the `governance_config_current` DISTINCT-ON view) treats `governance_config` as append-only: admin edits insert a new `(scope, key, valid_from)` row and the view surfaces the latest. Deriving `AsChangeset` here makes it trivially easy for future code to do `diesel::update(governance_config::table).set(&form)` and mutate historical rows in place, breaking the audit trail and the parity between `valid_from` and action time.

Drop `AsChangeset` and keep only `Insertable`. If an update path is ever genuinely needed (e.g. fixing the `updated_by` on the most-recent row), give it a dedicated narrow changeset struct.


<details>
<summary>Proposed diff</summary>

```diff
 #[derive(Clone, Default)]
-#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
+#[cfg_attr(feature = "full", derive(Insertable))]
 #[cfg_attr(feature = "full", diesel(table_name = governance_config))]
 pub struct GovernanceConfigInsertForm {
```
</details>

<!-- suggestion_start -->

<details>
<summary>📝 Committable suggestion</summary>

> ‼️ **IMPORTANT**
> Carefully review the code before committing. Ensure that it accurately replaces the highlighted code, contains no missing lines, and has no issues with indentation. Thoroughly test & benchmark the code to ensure it meets the requirements.

```suggestion
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_config))]
pub struct GovernanceConfigInsertForm {
  pub scope: String,
  pub key: String,
  pub value_type: String,
  pub value_int: Option<i64>,
  pub value_float: Option<f64>,
  pub value_bool: Option<bool>,
  pub value_text: Option<String>,
  pub updated_by: Option<PersonId>,
}
```

</details>

<!-- suggestion_end -->

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_schema/src/source/governance/governance_config.rs` around lines 34
- 46, The AsChangeset derive on GovernanceConfigInsertForm allows in-place
updates that break the append-only audit model; remove the #[cfg_attr(feature =
"full", derive(AsChangeset))] so GovernanceConfigInsertForm only derives
Insertable (and Clone/Default as needed), and if any update path is required
later create a separate, narrowly-scoped changeset struct (e.g.,
GovernanceConfigUpdate or similar) tailored to only the allowed mutable fields
(such as updated_by on the latest row) instead of reusing
GovernanceConfigInsertForm.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_schema/src/source/person.rs:68

_⚠️ Potential issue_ | _🟠 Major_

<details>
<summary>🧩 Analysis chain</summary>

🌐 Web query:

`Does serde's derive(Serialize) by default include struct fields without #[serde(skip)] regardless of whether application code reads them?`

💡 Result:

Yes, serde's derive(Serialize) by default includes all struct fields without #[serde(skip)] or similar skip attributes, regardless of whether application code reads them. The generated Serialize impl accesses every non-skipped field via &self.field_name when calling serialize_field, requiring the fields to be read from memory.

Citations:

- 1: https://serde.rs/field-attrs.html
- 2: https://serde.rs/impl-serialize.html
- 3: https://serde.rs/attr-skip-serializing.html
- 4: https://serde.rs/derive.html
- 5: https://doc.servo.org/src/serde_derive/ser.rs.html

---

🏁 Script executed:

```shell
cat -n crates/db_schema/src/source/person.rs | sed -n '55,80p'
```

Repository: barrie-cork/lemmy

Length of output: 1140

---

🏁 Script executed:

```shell
cat -n crates/db_schema/src/source/person.rs | head -70 | tail -30
```

Repository: barrie-cork/lemmy

Length of output: 1327

---

🏁 Script executed:

```shell
cat -n crates/db_schema/src/source/person.rs | head -40
```

Repository: barrie-cork/lemmy

Length of output: 1765

---



</details>

**`membership_state` is auto-serialised on `Person`, contradicting the "NO v0 handler reads this" contract.**

`Person` derives `Serialize` (line 14) and this field has no `#[serde(skip)]`, so every API endpoint that returns a `Person` (user profile, GetPersonDetails, login response, `GetSiteResponse.my_user`, federated objects reflected through JSON, etc.) will emit `membership_state` to clients in v0. That effectively exposes the deferred-enforcement column to the wire even though the doc comment and `lint-no-membership-read.sh` claim v0 silence.

If [99 OQ-016]'s deferred-enforcement story is strictly "no source-level reads in business logic" and API exposure is intentional, please update the doc comment to make that explicit. If it's meant to be wire-silent in v0, add `#[serde(skip)]` (and consider `#[cfg_attr(feature = "ts-rs", ts(skip))]` to match patterns used on similar fields like `inbox_url`) until v1 flips the gate.

<details>
<summary>🛡️ Proposed fix if wire-silence is required</summary>

```diff
   /// Deferred-enforcement membership-state flag per [99 OQ-016]. Populated
   /// at registration time from `config.onboarding.default_membership_state`.
   /// NO v0 handler reads this — grep-guarded by
   /// `scripts/brehon/lint-no-membership-read.sh` (Watch 7).
   /// TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR `#___`
+  #[serde(skip)]
+  #[cfg_attr(feature = "ts-rs", ts(skip))]
   pub membership_state: MembershipState,
```
</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_schema/src/source/person.rs` around lines 63 - 68, The Person
struct currently derives Serialize and exposes the membership_state field
publicly, contradicting the "NO v0 handler reads this" contract; to make it
wire-silent in v0, add #[serde(skip)] and #[cfg_attr(feature = "ts-rs",
ts(skip))] to the membership_state field (referencing the Person type and its
membership_state field) so it is omitted from JSON output and TypeScript
generation, and adjust the doc comment if instead you intend to allow API
exposure.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:500b5faf-decf-4419-9609-a38cceb425a5 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_views/reputation/src/impls.rs:128

_🧹 Nitpick_ | _🔵 Trivial_

**Query-branch duplication for `community_id = Some | None`.**

The only difference between the two arms is `.eq(cid)` vs `.is_null()`. Diesel's `.into_boxed()` collapses this to a single query builder like in `list_endorsements_for_person` below — the 9-column select list is otherwise pasted twice. Worth refactoring to reduce the chance of the two branches drifting (you will add `can_sponsor` here in v1 per the OQ-014 note).



<details>
<summary>Sketch</summary>

```rust
let base = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(person_id))
    .into_boxed();
let q = match community_id {
    Some(cid) => base.filter(reputation_snapshot::community_id.eq(cid)),
    None      => base.filter(reputation_snapshot::community_id.is_null()),
};
let row: Option<SnapshotRow> = q.select((/* 9 columns */)).first::<SnapshotRow>(conn).await.optional()?;
```
</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_views/reputation/src/impls.rs` around lines 93 - 128, The two match
arms for community_id duplicate the same select and filters; refactor by
creating a boxed query from reputation_snapshot::table (call .into_boxed()) with
the common filter reputation_snapshot::person_id.eq(person_id), then
conditionally add either .filter(reputation_snapshot::community_id.eq(cid)) or
.filter(reputation_snapshot::community_id.is_null()) based on community_id, and
finally call .select((...the 9
columns...)).first::<SnapshotRow>(conn).await.optional()? on that single builder
so SnapshotRow and the select tuple are not duplicated.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_views/reputation/src/impls.rs:272

_⚠️ Potential issue_ | _🟠 Major_

**`EndorsementSummaryView.active_sureties` is overloaded with two different meanings.**

`list_endorsements_for_person` populates `active_sureties` from `surety::sponsored_id.eq(person_id)` (i.e. sureties where the person is the sponsee, Line 185), while `list_sureties_for_person` populates the *same* field from `surety::sponsor_id.eq(person_id)` (Line 253). A consumer holding an `EndorsementSummaryView` cannot tell which direction the count represents — same struct shape, different semantics depending on which function produced it.

Split into two fields (`active_sureties_inbound`, `active_sureties_outbound`) on the view, or introduce two separate view structs. Otherwise a future caller is very likely to misinterpret the count.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_views/reputation/src/impls.rs` around lines 203 - 272, The
EndorsementSummaryView.active_sureties field is ambiguous because
list_endorsements_for_person sets it from surety::sponsored_id (sponsee/inbound)
while list_sureties_for_person sets it from surety::sponsor_id
(sponsor/outbound); update the view to disambiguate by adding two fields (e.g.,
active_sureties_inbound and active_sureties_outbound) or create two distinct
view structs, then update both constructors in list_endorsements_for_person and
list_sureties_for_person to populate the correct field(s), adjust the
struct/type definition for EndorsementSummaryView (or new structs), and update
any call sites that consume EndorsementSummaryView to use the new fields/types.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_views/reputation/src/lib.rs:49

_⚠️ Potential issue_ | _🟠 Major_

**Raw reputation numbers exposed on a public view struct.**

`ReputationSummaryView` exposes the four raw dimension scores (`reporting_accuracy`, `jury_reliability`, `participation_consistency`, `endorsement_strength`) as plain `i32` fields with `Serialize` derived. If this view is ever returned through the public API (which is the stated purpose of the `db_views` crate), it will leak raw reputation numbers to end users. Per ADR-005, users should only see capabilities (`jury_eligible`, `trusted_reporter`, and — once v1 flips — `can_sponsor`), never the underlying scores.

Two options:

1. If this struct is only ever consumed by admin/internal paths, gate serialisation (e.g. separate admin DTO) and add a doc comment + lint guard preventing it from being returned from public handlers.
2. Otherwise, drop the four score fields from the serialised shape and only expose booleans + `active_sanctions`.



As per coding guidelines: *"Flag views that collapse the four dimensions into a single score or expose raw reputation numbers to the public API (users see capabilities like `jury_eligible` and `trusted_reporter`, not numbers)."*

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_views/reputation/src/lib.rs` around lines 34 - 49,
ReputationSummaryView currently exposes raw score fields (reporting_accuracy,
jury_reliability, participation_consistency, endorsement_strength) which must
not be serialized to public APIs; either (A) create a separate admin/internal
DTO (e.g., ReputationSummaryAdmin) that includes those four i32 fields and
restrict its use to internal paths, add a doc comment and lint to prevent public
handler returns, and keep ReputationSummaryView for public use with only
jury_eligible, trusted_reporter, active_sanctions, person_id, community_id,
calculated_at; or (B) keep the same struct but mark the four score fields to not
be serialized (e.g., serde skip attributes) so only booleans + active_sanctions
are exposed, and add a doc comment/lint guard on ReputationSummaryView to
prohibit returning the raw scores from public handlers. Ensure references to
ReputationSummaryView and the four field names are used when making the change.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/db_views/vote/src/impls.rs:140

_⚠️ Potential issue_ | _🟡 Minor_

**Same unfilled `PR `#___`` placeholder as `pagination.rs`.**

Mirror whatever resolution is applied to the sibling comment in `crates/diesel_utils/src/pagination.rs` (Line 220) so both carry-patch notes stay in sync — either both reference a real tracking PR/issue, or both drop the placeholder.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/db_views/vote/src/impls.rs` around lines 135 - 140, Update the TODO
comment block that contains the "PR `#___`" placeholder in
crates/db_views/vote/src/impls.rs so it mirrors the resolution applied to the
sibling comment in pagination.rs (crates/diesel_utils/src/pagination.rs); locate
the same multi-line TODO (mentioning `clippy::multiple_bound_locations`,
`#[expect(...)]`, and workspace `-D clippy::allow-attributes`) and either
replace the placeholder with the real PR/issue reference used in pagination.rs
or remove the placeholder entirely so both files carry the same final
carry-patch note.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:893e943c-68a2-4541-82b4-45f02fcba63f -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/diesel_utils/src/pagination.rs:228

_⚠️ Potential issue_ | _🟡 Minor_

**Placeholder `PR `#___`` left in upstream TODO.**

The comment advertises an upstream PR number that was never filled in. Either drop the "PR `#___`" fragment or replace it with the actual tracking issue/PR once filed, so this doesn't linger as an untracked TODO.

<details>
<summary>✏️ Suggested tweak</summary>

```diff
-// TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR `#___`
+// TODO(brehon-fork): upstream this to LemmyNet/lemmy (tracking issue TBD).
```
</details>

<!-- suggestion_start -->

<details>
<summary>📝 Committable suggestion</summary>

> ‼️ **IMPORTANT**
> Carefully review the code before committing. Ensure that it accurately replaces the highlighted code, contains no missing lines, and has no issues with indentation. Thoroughly test & benchmark the code to ensure it meets the requirements.

```suggestion
// TODO(brehon-fork): upstream this to LemmyNet/lemmy (tracking issue TBD).
// On the cargo/clippy version pinned by rust-toolchain.toml the
// `clippy::multiple_bound_locations` lint no longer fires on this
// signature, so the previous `#[expect(...)]` form failed
// `unfulfilled-lint-expectations` under `-D warnings`. The workspace
// also bans `#[allow]` via `-D clippy::allow-attributes`. Resolution:
// remove the attribute entirely — if the bound pattern ever resurfaces
// on a future clippy upgrade, swap back to `#[expect(...)]` at that
// point.
```

</details>

<!-- suggestion_end -->

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/diesel_utils/src/pagination.rs` around lines 220 - 228, Update the
TODO comment that currently reads "TODO(brehon-fork): upstream this to
LemmyNet/lemmy — PR `#___`" by either removing the "PR `#___`" placeholder or
replacing it with the actual upstream PR/issue identifier; locate the comment
block in pagination.rs (the TODO starting with "TODO(brehon-fork): upstream this
to LemmyNet/lemmy") and make the change so the comment no longer contains an
empty placeholder.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:893e943c-68a2-4541-82b4-45f02fcba63f -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/routes/Cargo.toml:38

_🧹 Nitpick_ | _🔵 Trivial_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
# Confirm lemmy_api does not (transitively) depend on lemmy_routes.
fd -t f 'Cargo.toml' crates/api | xargs rg -nP '^\s*lemmy_routes\s*='
# And confirm lemmy_routes is only used as a binary/route wiring crate elsewhere.
rg -nP '^\s*lemmy_routes\s*=' --type=toml
```

Repository: barrie-cork/lemmy

Length of output: 355

---

🏁 Script executed:

```shell
# Check lemmy_api's Cargo.toml to see what it depends on
cat crates/api/Cargo.toml | grep -A 100 '^\[dependencies\]' | head -50
```

Repository: barrie-cork/lemmy

Length of output: 114

---

🏁 Script executed:

```shell
# Check if any of lemmy_api's dependencies might depend on lemmy_routes
# First, list all crates in the workspace to understand structure
fd -t d 'crates' -maxdepth 1 | sort
```

Repository: barrie-cork/lemmy

Length of output: 291

---

🏁 Script executed:

```shell
# Check if lemmy_routes is referenced anywhere as a dependency that lemmy_api might pull in
rg 'lemmy_routes' --type=toml crates/api/
```

Repository: barrie-cork/lemmy

Length of output: 125

---

🏁 Script executed:

```shell
# List all crates
ls -la crates/ | grep -E '^d'
```

Repository: barrie-cork/lemmy

Length of output: 735

---

🏁 Script executed:

```shell
# Find where lemmy_api is defined in Cargo.tomls
rg 'name\s*=\s*"lemmy_api"' --type=toml
```

Repository: barrie-cork/lemmy

Length of output: 180

---

🏁 Script executed:

```shell
# Check the main Cargo.toml workspace members
cat Cargo.toml | grep -A 30 'members'
```

Repository: barrie-cork/lemmy

Length of output: 1072

---

🏁 Script executed:

```shell
# Check lemmy_api's dependencies
cat crates/api/api/Cargo.toml
```

Repository: barrie-cork/lemmy

Length of output: 3402

---

🏁 Script executed:

```shell
# Verify lemmy_routes is not in lemmy_api's dependencies
rg 'lemmy_routes' crates/api/api/Cargo.toml
```

Repository: barrie-cork/lemmy

Length of output: 43

---

🏁 Script executed:

```shell
# Check what lemmy_routes depends on
cat crates/routes/Cargo.toml
```

Repository: barrie-cork/lemmy

Length of output: 2337

---

🏁 Script executed:

```shell
# Check what from lemmy_api is being used in lemmy_routes
rg 'run_snapshot_batch|use.*lemmy_api' crates/routes/
```

Repository: barrie-cork/lemmy

Length of output: 1462

---

🏁 Script executed:

```shell
# Check scheduled_tasks.rs specifically
find crates/routes -name 'scheduled_tasks.rs' -exec cat {} \; | head -100
```

Repository: barrie-cork/lemmy

Length of output: 2835

---

🏁 Script executed:

```shell
# Check if any of lemmy_api's dependencies depend on lemmy_routes
# Extract lemmy_api's dependencies and check each for lemmy_routes
for dep in lemmy_db_views_comment lemmy_db_views_community lemmy_apub_objects lemmy_api_common lemmy_api_utils lemmy_db_schema; do
  echo "=== Checking $dep ==="
  find crates -name "Cargo.toml" -path "*$dep*" -exec grep -l "lemmy_routes" {} \;
done
```

Repository: barrie-cork/lemmy

Length of output: 278

---

🏁 Script executed:

```shell
# More comprehensive check: list all crates that lemmy_api depends on and verify none depend on lemmy_routes
rg 'lemmy_routes' crates --type=toml -l | grep -v 'crates/routes' | grep -v 'crates/api/routes'
```

Repository: barrie-cork/lemmy

Length of output: 85

---

🏁 Script executed:

```shell
# Final verification: ensure run_snapshot_batch is the ONLY lemmy_api usage in routes
rg 'lemmy_api::' crates/routes/src/
```

Repository: barrie-cork/lemmy

Length of output: 182

---



</details>

**No dependency cycle exists; optional refactoring for compile-time efficiency.**

Verified: `lemmy_api` does not depend on `lemmy_routes` (no direct cycle), and no transitive cycle is introduced via `lemmy_api`'s dependencies. However, pulling all of `lemmy_api` solely to call `run_snapshot_batch` from `scheduled_tasks.rs` adds ~31 transitive crate dependencies to `lemmy_routes`. Whilst pragmatic for now, extracting this single scheduled-job function into a thin governance crate would keep `lemmy_routes` leaner for incremental rebuilds and better separation of concerns.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/routes/Cargo.toml` at line 38, The PR adds a direct dependency on
lemmy_api in crates/routes/Cargo.toml solely to call run_snapshot_batch from
scheduled_tasks.rs, pulling ~31 transitive crates into lemmy_routes; extract the
scheduled job into a new lightweight crate (e.g., lemmy_governance or
lemmy_scheduled_utils) containing the run_snapshot_batch function and any
minimal types it needs, update scheduled_tasks.rs to depend on that new crate
instead of lemmy_api, and remove the lemmy_api entry from
crates/routes/Cargo.toml so lemmy_routes no longer inherits lemmy_api's
transitive dependencies.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:199c90d9-e8d4-4a18-ae9d-17582b1e210d -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/routes/src/utils/scheduled_tasks.rs:167

_⚠️ Potential issue_ | _🟡 Minor_

**Env-var name suggests it disables all background jobs, but only the snapshot tick honours it.**

`BREHON_DISABLE_BACKGROUND_JOBS=1` only short-circuits this new 15‑minute snapshot job; the 10‑minute, hourly, and daily ticks registered above still run. If e2e tests set this expecting a quiescent scheduler, they can still race against `update_hot_ranks`, `active_counts`, etc. Either rename to something snapshot-specific (e.g. `BREHON_DISABLE_SNAPSHOT_JOB`) or apply the same guard to the other ticks.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/routes/src/utils/scheduled_tasks.rs` around lines 147 - 167, The env
var BREHON_DISABLE_BACKGROUND_JOBS only affects the snapshot task but its name
implies global behavior; change the code around the snapshot scheduler (the
closure created from context_gov_snapshot and the scheduler.every(...).run(...)
that calls lemmy_api::governance::reputation_snapshot::run_snapshot_batch) to
check a snapshot-specific variable instead (e.g. BREHON_DISABLE_SNAPSHOT_JOB)
and update any tests or docs that set the old env var to use the new name;
alternatively, if you prefer the original name, add the same
std::env::var("BREHON_DISABLE_BACKGROUND_JOBS") check to the other scheduled
ticks like update_hot_ranks and active_counts so the env var truly disables all
background jobs.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:2e3ecca3-9a9f-46bd-8039-5d92a6ed2fd1 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/routes/src/utils/scheduled_tasks.rs:165

_⚠️ Potential issue_ | _🔴 Critical_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
rg -nP -C5 'run_snapshot_batch|FOR UPDATE|process_chunk' crates/api/api/src/governance/reputation_snapshot.rs
```

Repository: barrie-cork/lemmy

Length of output: 4690

---

🏁 Script executed:

```shell
sed -n '347,390p' crates/api/api/src/governance/reputation_snapshot.rs
```

Repository: barrie-cork/lemmy

Length of output: 1681

---

🏁 Script executed:

```shell
rg -nP -B3 -A3 'run_snapshot_batch|re.enter|concurrent.*snapshot' crates/routes/src/utils/scheduled_tasks.rs
```

Repository: barrie-cork/lemmy

Length of output: 681

---

🏁 Script executed:

```shell
rg -nP 'clokwerk|AsyncScheduler' crates/routes/src/utils/scheduled_tasks.rs | head -20
```

Repository: barrie-cork/lemmy

Length of output: 305

---

🏁 Script executed:

```shell
rg -nP 'static.*Mutex|SNAPSHOT.*LOCK|BATCH.*GUARD' crates/routes/src/
```

Repository: barrie-cork/lemmy

Length of output: 43

---

🏁 Script executed:

```shell
rg -nP 'task.*54|background.*job|concurrent' crates/api/api/src/governance/reputation_snapshot.rs | grep -i 'concurrent\|task 54' | head -10
```

Repository: barrie-cork/lemmy

Length of output: 584

---

🌐 Web query:

`clokwerk AsyncScheduler concurrent task invocation behaviour re-entrance overlap`

💡 Result:

The clokwerk AsyncScheduler (Rust crate version 0.4.0) exhibits concurrent task invocation by spawning asynchronous futures for all pending jobs when run_pending is called. These futures run concurrently on the async runtime (e.g., Tokio), enabling overlap between different tasks and between multiple invocations of the same task if a long-running job is not awaited fully. Key behaviors: - run_pending returns a future that polls all pending jobs' futures until completion. Awaiting it sequentially executes batches but allows internal concurrency. - Long-running tasks do not block the scheduler loop if the run_pending future is spawned (e.g., tokio::spawn(scheduler.run_pending)), explicitly enabling concurrent invocations of the same job across calls, which can lead to re-entrance and overlap. - No built-in mutex or deduplication per-job; re-entrance is possible if a job's future is long-running and the next scheduler tick triggers it again before completion. - Official docs recommend spawn or timeouts to handle long tasks, confirming overlap risk: "This might result in multiple invocations of the same task running concurrently." To prevent overlap: Await run_pending fully in the loop, use timeouts, or spawn jobs internally with blocking.

Citations:

- 1: https://docs.rs/clokwerk/latest/clokwerk/struct.AsyncScheduler.html
- 2: https://docs.rs/clokwerk/latest/clokwerk/struct.AsyncJob.html
- 3: https://linkedin.com/company/concurrent-design-limited
- 4: https://www.itsoverlap.com

---

🏁 Script executed:

```shell
rg -nP 'snapshot.*duration|minutes\(15\)|task.*duration|GOTCHA.*54|Watch.*6' crates/api/api/src/governance/reputation_snapshot.rs
```

Repository: barrie-cork/lemmy

Length of output: 238

---



</details>

**Add a batch-level concurrency guard to prevent overlapping `run_snapshot_batch` invocations.**

clokwerk's AsyncScheduler allows concurrent task invocations if the previous invocation hasn't completed by the next tick. If `run_snapshot_batch` exceeds the 15-minute interval, overlapping invocations will pile up connections and contend on row locks, even though pair-level guards (`FOR UPDATE`, `pg_advisory_xact_lock`) prevent data corruption. Add a static `Mutex<bool>` or `AtomicBool` to skip invocation if a batch is already running.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/routes/src/utils/scheduled_tasks.rs` around lines 162 - 165, Add a
batch-level concurrency guard around the call to
lemmy_api::governance::reputation_snapshot::run_snapshot_batch(&context).await:
introduce a static AtomicBool (or once_cell::sync::Lazy<AtomicBool>) named e.g.
REPUTATION_SNAPSHOT_RUNNING and at invocation do an atomic compare_exchange to
skip the run if already true, returning early; if the exchange succeeds set it
to true, run the async work, then ensure you set it back to false in a
finally/drop path (use scopeguard or a small RAII guard) so the flag is cleared
even on panic or errors; keep the existing inspect_err(...) logging and .ok()
behavior but only call run_snapshot_batch when the guard succeeded.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:2e3ecca3-9a9f-46bd-8039-5d92a6ed2fd1 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/server/src/governance.rs:28

_🧹 Nitpick_ | _🔵 Trivial_

**Minor: `schedule_governance_jobs` no longer schedules anything — just logs.**

The function name implies registration, but the actual registration happens in `scheduled_tasks::setup`. The log message is now accurate, but consider renaming this to `log_governance_jobs_registered` (or removing it entirely) to avoid the impression that this is where the scheduling lives. Pure doc/ergonomics.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/server/src/governance.rs` around lines 23 - 28, The function
schedule_governance_jobs only logs and does not register jobs; rename it to
log_governance_jobs_registered (or remove it) to avoid misleading callers.
Update the function signature pub fn schedule_governance_jobs(_context:
&LemmyContext) to pub fn log_governance_jobs_registered(_context: &LemmyContext)
and leave the info! call intact, then find and update all callers/imports of
schedule_governance_jobs to use the new name (or delete the function and remove
its references if you choose removal).
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:2e3ecca3-9a9f-46bd-8039-5d92a6ed2fd1 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### crates/server/tests/e2e.rs:1339

_⚠️ Potential issue_ | _🟡 Minor_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
# Confirm accessors fall back to const_default when the DB row is absent.
rg -nP -C8 'pub\s+async\s+fn\s+get_(int|float|bool|text)\b' crates/api/api/src/governance/config.rs
rg -nP -C3 'const_default_(int|float|bool|text)' crates/api/api/src/governance/config.rs
```

Repository: barrie-cork/lemmy

Length of output: 4647

---



</details>

**The `config_parity_round_trip` test cannot detect a missing seed row.**

The accessors (`get_int`, `get_float`, `get_bool`, `get_text`) fall back to `const_default_*` when no `governance_config_current` row is found, so this test reports success even if a key is entirely absent from the migration seed — which is precisely one of the drift classes the stated purpose claims to catch. Type mismatches at the DB level are caught via Diesel deserialisation errors, but missing seed rows are not.

Suggest adding an assertion that each seeded key has a row in `governance_config_current` with the expected `value_type` before calling the accessor:

<details>
<summary>🔧 Proposed strengthening</summary>

```diff
   for (key, _const_name, vtype) in SEEDED_KEYS_WITH_CONSTS {
+    // Assert the seed row exists with matching value_type — accessor
+    // success alone would be satisfied by const_default_* fallback.
+    let row_vtype: String = diesel::sql_query(
+      "SELECT value_type AS n FROM governance_config_current WHERE scope = 'instance' AND key = $1",
+    )
+    .bind::<diesel::sql_types::Text, _>(*key)
+    .get_result::<SingleText>(&mut async_conn) // define SingleText { n: String }
+    .await
+    .map_err(|e| -> Box<dyn Error> { format!("seed row missing for `{key}`: {e}").into() })?
+    .n;
+    assert_eq!(&row_vtype, vtype, "seed value_type mismatch for `{key}`");
+
     match *vtype {
```
</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@crates/server/tests/e2e.rs` around lines 1281 - 1339, The test
config_parity_round_trip currently calls the accessors
(get_int/get_float/get_bool/get_text) which silently return const defaults when
a governance_config_current row is missing; add an explicit existence/type-check
before calling the accessor: for each (key, _const_name, vtype) from
SEEDED_KEYS_WITH_CONSTS query governance_config_current (using the existing
pool/DbPool or AsyncPgConnection) to assert a row exists for that key and that
its value_type column equals the expected vtype, and return Err or assert
failure if missing/mismatched; keep the subsequent calls to get_* and
ConfigCache unchanged so the test still exercises round-trip deserialisation
after verifying the seed row is present.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:1fe5fefe-3ad2-4539-8e27-37ce5bd7f9b0 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### migrations/2026-04-18-000000-0000_add_governance_config/down.sql:15

_🧹 Nitpick_ | _🔵 Trivial_

**Redundant `DROP INDEX` before `DROP TABLE governance_config`.**

`DROP TABLE` cascades its own indexes, so lines 13–14 are no-ops once line 15 runs. Harmless but noisy — consider removing to keep the mirror of `up.sql` clean, unless `up.sql` created the indexes on a different object than the table.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@migrations/2026-04-18-000000-0000_add_governance_config/down.sql` around
lines 13 - 15, The DROP INDEX statements for governance_config_scope_key_idx and
governance_config_scope_key_valid_from_idx are redundant because DROP TABLE IF
EXISTS governance_config will remove its indexes; remove those two DROP INDEX
lines from down.sql so it mirrors up.sql cleanly, unless the corresponding
up.sql created those indexes on a different object—in that case leave them as-is
and add a comment explaining why they target a different table/index owner.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:500b5faf-decf-4419-9609-a38cceb425a5 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### migrations/2026-04-18-000000-0000_add_governance_config/up.sql:36

_⚠️ Potential issue_ | _🟠 Major_

**`updated_by` nullable with `ON DELETE SET NULL` breaks admin attribution for the append-only history.**

`governance_config` is meant to be an append-only audit trail of admin edits (Watch 11: attribute config cascades at action time). But `updated_by INTEGER REFERENCES person (id) ON DELETE SET NULL` silently erases the admin identity if that admin is later deleted, which destroys the audit trail for past rows — the table is append-only, yet a FK side-effect mutates historical data.

Consider either:
- `ON DELETE RESTRICT` (or no action) to preserve attribution, paired with a soft-delete convention for admins, or
- Copying the pseudonym/actor string into the row at write time (same pattern as `governance_log.actor_pseudonym`) so FK cascades cannot erase it.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@migrations/2026-04-18-000000-0000_add_governance_config/up.sql` around lines
19 - 36, The FK on governance_config.updated_by currently uses "ON DELETE SET
NULL", which erases admin attribution; change the schema to preserve attribution
by either (A) making updated_by a non-null FK with "REFERENCES person(id) ON
DELETE RESTRICT" (or no action) so deletes are prevented and require soft-delete
semantics, or (B) add a copied actor pseudonym column (e.g.,
updated_by_pseudonym TEXT NOT NULL) and populate it at write time following the
governance_log.actor_pseudonym pattern so identity survives FK cascades; update
the CREATE TABLE DDL for governance_config to implement your chosen approach and
ensure NOT NULL constraints where appropriate.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### migrations/2026-04-18-000000-0000_add_governance_config/up.sql:52

_🧹 Nitpick_ | _🔵 Trivial_

<details>
<summary>🧩 Analysis chain</summary>

🌐 Web query:

`What is the default value of security_invoker for CREATE VIEW in PostgreSQL 15 and 16?`

💡 Result:

The default value of security_invoker for CREATE VIEW in PostgreSQL 15 and 16 is false. This means views behave as security definer views by default, checking permissions based on the view owner's privileges unless explicitly set to WITH (security_invoker=true). Official PostgreSQL documentation for versions 15 and 16 describes security_invoker as an optional boolean parameter in the WITH clause. Examples in the docs create views without specifying it, confirming the default behavior matches pre-15 views (security definer). Secondary sources note that security_invoker=true must be explicitly set for invoker behavior, and pg_class.reloptions is null by default for views without the option (Stack Overflow confirms checking for explicit settings). No changes in 16 release notes.

Citations:

- 1: http://postgresql.org/docs/15/sql-createview.html
- 2: http://www.postgresql.org/docs/16/sql-createview.html
- 3: https://stackoverflow.com/questions/75909367/postgresql-check-if-security-invoker-is-turned-on-on-view
- 4: https://www.postgresql.org/docs/15/sql-createview.html
- 5: https://www.postgresql.org/docs/16/sql-createview.html
- 6: http://www.postgres.com/docs/15/sql-createview.html
- 7: https://medium.com/@mydbopsdatabasemanagement/exploring-security-invoker-views-in-postgresql-15-a-step-towards-safer-data-access-04131a96fa79
- 8: https://www.mydbops.com/blog/security-invoker-views-in-postgresql-15

---



</details>

**Make the view's security behaviour explicit with `WITH (security_invoker = true)` and add appropriate GRANTs.**

This view exposes `updated_by` (admin-edit history), which is sensitive data. PostgreSQL views default to security definer (checking permissions based on the view owner), but making this explicit prevents future migrations from inadvertently widening access. On PostgreSQL 15+, use `CREATE VIEW ... WITH (security_invoker = true)` or confirm the security definer default is intentional. If the app role shouldn't see all scopes, add a matching GRANT to restrict access.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@migrations/2026-04-18-000000-0000_add_governance_config/up.sql` around lines
47 - 52, The view governance_config_current should explicitly set
security_invoker to avoid unintended definer privileges and then restrict/grant
access: update the CREATE VIEW governance_config_current statement to include
WITH (security_invoker = true) and add explicit GRANT SELECT on the view to the
application role(s) (and revoke/avoid PUBLIC access) so only intended roles can
read the sensitive updated_by column; ensure the grants correspond to your app
role names and that any necessary grants on underlying governance_config are
present for the invoker role.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### migrations/2026-04-18-000000-0000_add_governance_config/up.sql:72

_⚠️ Potential issue_ | _🔴 Critical_

<details>
<summary>🧩 Analysis chain</summary>

🏁 Script executed:

```shell
#!/bin/bash
# Confirm the create_report handler still uses integer-unit V0_THRESHOLD / V0_REPORTER_WEIGHT
# and has NOT been updated to micros.
rg -nP -C3 '\bV0_(THRESHOLD|REPORTER_WEIGHT)\b'
rg -nP -C5 'threshold_score\s*\.\s*eq\s*\(' --type=rust
rg -nP -C3 'case_threshold_micros' --type=rust
```

Repository: barrie-cork/lemmy

Length of output: 13233

---

🏁 Script executed:

```shell
cat -n migrations/2026-04-18-000000-0000_add_governance_config/up.sql | sed -n '60,80p'
```

Repository: barrie-cork/lemmy

Length of output: 1633

---

🏁 Script executed:

```shell
cat -n crates/api/api_crud/src/governance/create_report.rs | sed -n '50,160p'
```

Repository: barrie-cork/lemmy

Length of output: 4799

---



</details>

**Rescaling to micros must be paired with handler update or deferred until task 58 lands.**

The migration rescales existing `threshold_score` rows to micros (×1,000,000) at line 72, but `create_report.rs` still uses integer-unit constants (`V0_THRESHOLD = 3`, `V0_REPORTER_WEIGHT = 1`) for threshold comparison and score increment. This creates two unit regimes in the same column:

- Existing rows: rescaled to micros; next report adds 1 (not 1,000,000), triggering `should_flip` on every subsequent report because `3_000_001 > 3`.
- Fresh cases: accumulate integer units stored in a column now treated as micros by the config seed (`report.case_threshold_micros = 3_000_000`) and any Phase 5a code reading micros.

Either pair this migration with an immediate handler patch (multiply `V0_REPORTER_WEIGHT` by 1,000,000 or read `case_threshold_micros` from config), or defer the rescaling to the PR that lands task 58's config-driven formula. As-is, the Phase 4 golden-path test breaks and case state corrupts.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@migrations/2026-04-18-000000-0000_add_governance_config/up.sql` around lines
68 - 72, The migration scales threshold_score to micros but the reporting
handler still uses integer-unit constants, causing mixed units; either
(preferred) update the handler in create_report.rs so scoring/comparison uses
micros—e.g., read the config field case_threshold_micros and multiply
V0_REPORTER_WEIGHT by 1_000_000 (or convert all local constants like
V0_THRESHOLD/V0_REPORTER_WEIGHT to micros) so increments and comparisons use the
same units as threshold_score, or (alternatively) revert/defer the UPDATE in the
migration so unit conversion happens together with the task-58 config-driven
change; locate create_report.rs and the constants V0_THRESHOLD and
V0_REPORTER_WEIGHT and ensure their units match the DB column (threshold_score)
and report.case_threshold_micros before keeping this migration.
```

</details>

<!-- fingerprinting:phantom:medusa:nectarine:2b3878d4-8ef6-40a4-bbb8-0dc316e54296 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### scripts/brehon/lint-no-can-sponsor-read.sh:31

_🧹 Nitpick_ | _🔵 Trivial_

**Consider tightening directory-wide exclusions.**

`^crates/db_views/reputation/src/` and `^crates/db_schema_file/` exclude entire subtrees. A future Rust file added under either path that reads `can_sponsor` will silently bypass this guard. If the intent is to allow only struct/column definitions and doc-comments, consider narrowing to specific files (as you already do for the other four entries) or adding a secondary check that flags non-doc-comment, non-struct-field references within those directories.

Not blocking — the current list matches the documented authorised sites.

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@scripts/brehon/lint-no-can-sponsor-read.sh` around lines 25 - 31, The current
FORBIDDEN grep excludes whole directories (^crates/db_views/reputation/src/ and
^crates/db_schema_file/) which can let future files silently bypass the
can_sponsor check; narrow those exclusions by replacing the directory-wide
patterns with explicit file paths (like reputation_snapshot.rs and any specific
schema files) or add a secondary grep that flags matches in those directories
unless they are only in doc-comments or struct/field definitions; update the
FORBIDDEN logic (the variable and its pipeline of grep -v patterns) to reference
the exact filenames you already allow (e.g.,
crates/db_schema/src/source/governance/reputation_snapshot.rs,
crates/api/api/src/governance/reputation_snapshot.rs,
crates/db_views/reputation/src/<specific_file>.rs) or add an additional filter
step that rejects matches containing non-comment/non-field contexts.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:199c90d9-e8d4-4a18-ae9d-17582b1e210d -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


### scripts/brehon/lint-no-membership-read.sh:32

_⚠️ Potential issue_ | _🟡 Minor_

**Header authorised-sites list is out of sync with the actual exclusion list.**

The docstring (lines 8–16) enumerates authorised sites, but the `grep -v` chain (lines 22–32) also excludes three paths not mentioned in the header:

- `crates/db_schema/src/impls/person.rs` (test fixture)
- `crates/db_views/registration_applications/src/impls.rs`
- `crates/server/tests/e2e.rs`

Either add these to the header comment or drop them from the exclusion list, otherwise future maintainers won't know why a new v1-activation site should or shouldn't be added. Since the header is meant to be the canonical allowlist, prefer updating the header.

<details>
<summary>📝 Proposed header update</summary>

```diff
 #   - crates/db_schema_file/       (schema.rs + enums.rs own the column type)
 #   - crates/db_schema/src/source/person.rs (struct + InsertForm)
 #   - crates/db_schema/src/lib.rs  (Person1/Person2AliasAllColumnsTuple — schema-layer tuple aliases that MUST list the column to match person::all_columns arity)
+#   - crates/db_schema/src/impls/person.rs (test fixture carry-patch for `Person { ... }` literals)
 #   - crates/api/api/src/governance/config.rs (parse_membership_state helper, consumed only by the authorised writer below)
 #   - crates/api/api_crud/src/user/create.rs (register handler writes the value)
 #   - crates/apub/objects/src/objects/person.rs (federated-person upsert writes `None` so the SQL DEFAULT takes effect)
+#   - crates/db_views/registration_applications/src/impls.rs (view-layer test fixture)
+#   - crates/server/tests/e2e.rs (e2e test fixture)
 #   - migrations/                  (up.sql + down.sql for the column)
 #   - .claude/                     (plan, rules, memory, decision queue all discuss it)
```
</details>

<details>
<summary>🤖 Prompt for AI Agents</summary>

```
Verify each finding against the current code and only fix it if needed.

In `@scripts/brehon/lint-no-membership-read.sh` around lines 8 - 32, Update the
header authorised-sites list to match the exclusions used by the FORBIDDEN grep
chain: add entries for crates/db_schema/src/impls/person.rs,
crates/db_views/registration_applications/src/impls.rs, and
crates/server/tests/e2e.rs so the comment and the FORBIDDEN variable remain in
sync and future maintainers can see why those paths are allowed; locate the
header near the top of scripts/brehon/lint-no-membership-read.sh and the
FORBIDDEN assignment to confirm the exact paths referenced.
```

</details>

<!-- fingerprinting:phantom:poseidon:nectarine:500b5faf-decf-4419-9609-a38cceb425a5 -->

<!-- This is an auto-generated comment by CodeRabbit -->

---


