# v1-RT-r4 — Sponsor-gate strategies + admin allowlist endpoints

## 1. Summary

Complete PRD §11 row 4 ("Sponsor-gate strategy expansion") by adding the three
remaining `SponsorGateStrategy` variants (`age_or_surety`, `reputation`,
`allowlist`) to the endorsement flow, plus the admin endpoints that maintain the
allowlist that the `allowlist` strategy reads. The work spans five crates but is
one cohesive feature: a strategy enum extension in `create_endorsement.rs`, three
inline allowlist db-helpers in `sponsor_allowlist.rs`, four admin DTOs, two admin
HTTP handlers in a new `admin_sponsor_allowlist.rs`, route registration, a
governance-log registry-row flip from `(pending)` to live, and e2e coverage.

No migration. No new ENTRY_KIND consts (both `sponsor_allowlist_added` /
`sponsor_allowlist_removed` consts pre-landed in v1-RT-r1 — this sub-phase is
their first emission site). OQ-020 contract holds: a single strategy string,
no composition. Grandfathering preserved: existing endorsements are NOT
re-validated against the new strategies.

## 2. Source

- **PRD:** `.claude/PRPs/prds/v1-rt.prd.md` §11 row 4.
- **Brief:** `.claude/PRPs/briefs/rt-r4-planning-1.md` (authoritative spec; §0
  dependency-reality table, §2.1 deliverables a–g, §2.2 handler step ordering,
  §4 constraints, clarify resolutions `de57d6ce31bc-001..003`).
- **Clarify resolutions (resolved DQs):**
  - `de57d6ce31bc-001` — `surety` table exists; `age_or_surety` reads it.
  - `de57d6ce31bc-002` — `reputation_snapshot.can_sponsor` column exists;
    `reputation` reads it.
  - `de57d6ce31bc-003` — allowlist db query helpers are ABSENT → deliverable
    (b) is a real §13 task.
- **Complexity split DQ:** `9f9026a1d618-001` (resolved, proceed-as-one).

## 3. Problem statement

The `SponsorGateStrategy` enum (`crates/api/api_crud/src/governance/create_endorsement.rs:84`)
ships in v0 with exactly three variants — `Age`, `Open`, `Closed`, plus the
`Unknown(String)` catch-all final arm (GOTCHA-55a at `:78-82`). PRD §11 row 4
specifies four additional named strategies. RT-r1/RT-r2/RT-r3 landed supporting
infrastructure (the `sponsor_allowlist` table, `SponsorAllowlistId`, both
governance-log ENTRY_KIND consts, the `surety` table, the
`reputation_snapshot.can_sponsor` column) but never wired the strategies into
the endorsement-gate match, and never built the admin endpoints that populate
the allowlist. The registry rows at
`.claude/rules/governance-log-entry-kind-registry.md:190-191` still carry
`(pending)` against `admin_sponsor_allowlist.rs` because that file does not exist.

So today an instance admin can set `onboarding.sponsor_gate_strategy` to
`allowlist` and the endorsement flow silently falls through to `age` semantics
(the `Unknown(s)` arm), with no way to add anyone to the allowlist. Three named
strategies are unreachable; the allowlist is unmaintainable.

## 4. Solution statement

Four surfaces, one feature.

### Surface 1 — strategy enum + dispatch (deliverable a)

Extend `SponsorGateStrategy` with `AgeOrSurety`, `Reputation`, `Allowlist`
variants; extend `parse()` (`:92`) and `label()` (`:102`); add three match arms
in `process_endorsement`'s `match &strategy` (`:161`). Semantics:

- **`age_or_surety`** — age gate (existing `enforce_age_gate`, `:331`) OR an
  active surety: `EXISTS (surety WHERE sponsored_id = caller AND revoked_at IS
  NULL)`. Reads `surety` (`crates/db_schema/src/source/governance/surety.rs:17`;
  `sponsor_id:19`, `sponsored_id:20`, `revoked_at:23`).
- **`reputation`** — reads `reputation_snapshot.can_sponsor`
  (`reputation_snapshot.rs:31`); community-scoped when `data.community_id.is_some()`
  (row with `community_id = Some(c)`), else instance-wide (`community_id IS NULL`,
  `:20`).
- **`allowlist`** — caller present in `sponsor_allowlist` (community row
  `community_id = Some(c)` OR instance-wide `community_id IS NULL`) via the
  deliverable-(b) `sponsor_allowlist_exists` helper.

**Watchpoint:** preserve GOTCHA-55a. `Unknown(String)` stays the exhaustive-but-
final arm; there is NO `_ =>` catchall (workspace clippy denies it via
`feedback_clippy_test_style`). The three new arms are explicit named arms ABOVE
the `Unknown(s)` arm.

### Surface 2 — allowlist db-helpers (deliverable b)

Three inline `#[cfg(feature = "full")] pub async fn` helpers in
`crates/db_schema/src/source/governance/sponsor_allowlist.rs`, mirroring the
inline-helper convention in `federation_peer.rs:55`/`:70`
(signature `(... , conn: &mut AsyncPgConnection) -> LemmyResult<T>`):

- `sponsor_allowlist_insert(form: &SponsorAllowlistInsertForm, conn) -> LemmyResult<SponsorAllowlist>`
- `sponsor_allowlist_delete(allowlist_id: SponsorAllowlistId, conn) -> LemmyResult<usize>`
- `sponsor_allowlist_exists(person_id: PersonId, community_id: Option<CommunityId>, conn) -> LemmyResult<bool>`

NOT in `impls/` — governance convention is inline in `source/governance/<file>.rs`.

### Surface 3 — admin DTOs + endpoints (deliverables c, d)

Four DTOs in `crates/api/api_common/src/governance.rs` (mirror the
`AdminSetConfig`/`AdminSetConfigResponse` derive stack at `:444`/`:465`):
`AddSponsorAllowlist`, `AddSponsorAllowlistResponse`, `RemoveSponsorAllowlist`,
`RemoveSponsorAllowlistResponse`.

Two handlers in NEW `crates/api/api/src/governance/admin_sponsor_allowlist.rs`,
mirroring `admin_config.rs::admin_set_config` (`:383-558`) step ordering verbatim:
validation → admin capability (`is_admin`, `admin_config.rs:62`/`:753`) → denial-
log OUTSIDE `run_transaction` → `actor_pseudonym_helper::get_or_create(pool,
admin_id)` (`:485`) → success path: db write inside `run_transaction` (`:497`),
`governance_log::append` opens its OWN internal tx.

- **add** — emits `sponsor_allowlist_added` (payload `{allowlist_id, community_id?,
  person_pseudonym, added_by_admin_pseudonym, note?, added_at}`).
- **remove** — resolves the row (person + community default) to obtain
  `allowlist_id` for the log, deletes, emits `sponsor_allowlist_removed` (payload
  `{allowlist_id, community_id?, person_pseudonym, removed_by_admin_pseudonym,
  removed_at}`).

**ADR-015:** log `person_pseudonym` (the allowlisted person) + admin pseudonym,
NEVER raw `person_id`. **ADR-008:** governance_log append-only.

### Surface 4 — wiring (deliverables e, f)

- Route registration in `crates/api/routes/src/lib.rs` admin scope (`:493-507`)
  + `use` import: `scope("/sponsor-allowlist").route("/add", post().to(add))
  .route("/remove", post().to(remove))`.
- Module declaration `pub mod admin_sponsor_allowlist;` (alphabetical) in
  `crates/api/api/src/governance/mod.rs`.
- Registry-row flip at `governance-log-entry-kind-registry.md:190-191` — drop
  `(pending)`, fill real handler fn names. Named as a §13 task (Task 6),
  advisor/impl-executed — the Junior planner is blocked from `.claude/**`.

## 5. Metadata

- **Phase:** v1-RT-r4
- **Phase branch:** `phase-v1-RT-r4`
- **Trunk:** `governance-v0`
- **Target impl-task model:** sonnet-4-6
- **Complexity score:** 10/10 (split-or-proceed DQ `9f9026a1d618-001` resolved
  → PROCEED-AS-ONE; cohesive single-feature, tight serial dependency chain,
  per-task ceilings all hold, RT precedent ships at this size).
- **Shape:** pre-Shape-G (Shape G SUSPENDED until 2026-06-01) → cargo validation
  via `validate-pending-laptop` / `validate-pending-laptop-e2e`.
- **Migration:** none.
- **ADR-affecting:** no (operates within ADR-008 / ADR-015 / OQ-020 as already
  decided).

### 5.1 Complexity factor breakdown

| Factor | Rule | Count | Score |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each (excludes Task 0 + retro) | 7 tasks (1–7); 2 above 5 | +2 |
| Migrations | +2 each | 0 | 0 |
| Crates touched | +1 each (crates/server e2e EXCLUDED) | db_schema, api_common, api_crud, api/api, routes = 5 | +5 |
| e2e edits | +3 each per-file | 1 file (e2e.rs) | +3 |
| ADR-affecting | +2 | no | 0 |
| Cargo budget >6 GB | +1/GB (Shape-G = 0; pre-Shape-G here) | within 6 GB | 0 |
| **Total** | | | **10** |

10 > 8 → Sonnet split threshold tripped. Split DQ `9f9026a1d618-001` filed and
resolved PROCEED-AS-ONE: the allowlist deliverable chain (b → c/d → e → f) is a
tight serial dependency that cannot be split without creating cross-task
half-features; the two independent strategy arms (`age_or_surety`, `reputation`)
live entirely within Task 3's single file; every per-task ceiling in §5.2 holds.
RT-r3 shipped at 7 with a comparable surface count; RT-r4's +3 is purely the
fifth crate and is not decomposable.

### 5.2 Per-task ceiling (mechanical)

≤4 files / ≤2 crates per task; `e2e.rs` is never bundled with non-test logic.
Every §13 task below respects this — verified per-task in §13.

## 6. Relationship to prior sub-phases

- **RT-r1** landed the `sponsor_allowlist` table, `SponsorAllowlistId`, both
  ENTRY_KIND consts (`governance_log.rs:217`/`:218` + shim re-exports `:60`/`:61`),
  and the registry rows (`:190-191`, marked `(pending)`).
- **RT-r2/RT-r3** landed `surety`, `reputation_snapshot.can_sponsor`, and the v0
  three-strategy enum.
- **RT-r4 (this)** is the first emission site for both allowlist consts and the
  first wiring of the four named strategies. It adds NO new consts → the registry
  acceptance invariant (total = 55) is UNCHANGED.

## 7. Preflight guardrails (R1–R11, inherited)

R1–R11 from RT-r3 §7 carry forward unchanged. Highlights load-bearing here:

- **R3 (no migration):** this sub-phase touches no `migrations/**`. Any task that
  reaches for a migration is mis-scoped → stop, file `kind: "blocker"` DQ.
- **R5 (governance-log via `governance_log::append`):** never INSERT into the log
  table directly; `append` owns its internal tx and hash-chain.
- **R7 (pseudonym discipline, ADR-015):** raw `person_id` never enters a log
  payload; use `actor_pseudonym_helper::get_or_create`.
- **R9 (clippy `--workspace --features full --no-deps -- -D warnings`):** no
  `_ =>` catchall on the strategy match (GOTCHA-55a).
- **R11 (Task 0 pre-flight harness audit mandatory):** 14 probes before Task 1.

## 8. Flow design (before → after)

**Before** (`process_endorsement`, `create_endorsement.rs:161`):
```
match &strategy {
  Age            => enforce_age_gate(...)
  Open           => (no gate)
  Closed         => deny
  Unknown(s)     => log unrecognized + enforce_age_gate(...)   // fallthrough
}
```

**After:**
```
match &strategy {
  Age            => enforce_age_gate(...)
  AgeOrSurety    => enforce_age_gate OR surety EXISTS
  Reputation     => reputation_snapshot.can_sponsor (scoped)
  Allowlist      => sponsor_allowlist_exists(caller, community_id)
  Open           => (no gate)
  Closed         => deny
  Unknown(s)     => log unrecognized + enforce_age_gate(...)   // fallthrough
}
```

**Box → task map:**

| Surface box | §13 task |
|---|---|
| allowlist db-helpers | Task 1 `[P]` |
| admin DTOs | Task 2 `[P]` |
| strategy enum/parse/label/3 arms | Task 3 (requires 1) |
| admin add/remove handlers + mod.rs | Task 4 (requires 1,2) |
| route registration | Task 5 (requires 4) |
| registry-row flip | Task 6 (requires 4; advisor-executed) |
| e2e coverage | Task 7 (requires 3,4,5) |

## 9. Mandatory reading (impl-task)

Inject into every matching impl-task brief §3 (file-class table, advisor §2.4):

- `feedback_multi_write_handlers_need_transactions.md` — Task 4 (handlers do db
  write + governance-log append).
- `feedback_lemmy_error_no_std_error.md` — Task 7 (e2e returns `LemmyResult<()>`).
- `feedback_async_pool_test_pattern.md` — Task 7.
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — Task 7 (≥2 e2e edits; dominant
  complexity factor per brief §5).
- `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md`
  — every cargo task (`#[cfg(feature = "full")]` gates throughout).
- `feedback_clippy_test_style.md` — Task 3 (GOTCHA-55a: no `_ =>` catchall).
- `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — Task 1
  (`SponsorAllowlistId`, `PersonId`, `CommunityId` imports).

MIRROR refs to read before writing:

- `create_endorsement.rs:78-192` (enum/parse/label/dispatch).
- `admin_config.rs:383-558` (handler step ordering — the conformance-audit axis
  most likely to drift; §3.1.1).
- `api_common/governance.rs:444`/`:465` (DTO derive stack).
- `federation_peer.rs:55-94` (inline db-helper convention).
- `surety.rs:17-31`, `reputation_snapshot.rs:17-46`, `sponsor_allowlist.rs:11-40`.

## 10. Patterns to mirror

- **Handler step ordering:** mirror `admin_set_config` (`:383-558`) verbatim —
  validation → `is_admin` → denial-log OUTSIDE `run_transaction` →
  `get_or_create(pool, admin_id)` (`:485`) → write inside `run_transaction`
  (`:497`) → `governance_log::append` (own internal tx). Do NOT invent a
  single-tx-wrapping-append or SAVEPOINT shape (`run_transaction` exposes no
  SAVEPOINT primitive — `admin_config.rs:10`).
- **DTO derive stack:** `#[skip_serializing_none] #[derive(Debug, Serialize,
  Deserialize, Clone, Default, PartialEq)] #[cfg_attr(feature = "ts-rs",
  derive(ts_rs::TS))] #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]`.
- **Inline db-helper:** `#[cfg(feature = "full")] pub async fn name(..., conn:
  &mut AsyncPgConnection) -> LemmyResult<T>` with `.optional()?` /
  `.get_result(conn).await?` per `federation_peer.rs`.
- **Strategy match:** named arms above the `Unknown(s)` final arm; no `_ =>`.
- **e2e fixtures:** mirror the most recent RT/SL/JM fixtures sibling — Case A
  (outer fn `LemmyResult<()>`, helpers `LemmyResult<T>`, NO `.map_err` bridges)
  per `feedback_lemmy_error_no_std_error.md`.

## 11. Files to change

| File | Crate | Task | Action |
|---|---|---|---|
| `crates/db_schema/src/source/governance/sponsor_allowlist.rs` | db_schema | 1 | modify (add 3 inline helpers) |
| `crates/api/api_common/src/governance.rs` | api_common | 2 | modify (add 4 DTOs) |
| `crates/api/api_crud/src/governance/create_endorsement.rs` | api_crud | 3 | modify (enum/parse/label/3 arms) |
| `crates/api/api/src/governance/admin_sponsor_allowlist.rs` | api/api | 4 | create (2 handlers) |
| `crates/api/api/src/governance/mod.rs` | api/api | 4 | modify (`pub mod` decl) |
| `crates/api/routes/src/lib.rs` | routes | 5 | modify (route + use) |
| `.claude/rules/governance-log-entry-kind-registry.md` | meta | 6 | modify (registry-row flip; advisor-executed) |
| `crates/server/tests/e2e.rs` | server | 7 | modify (e2e coverage) |

## 12. NOT building (out of scope)

- **Composition strategies** (OQ-020 REJECTED — single strategy string only, no
  `age_or_surety_and_reputation` AND/OR combinators beyond the explicit
  `age_or_surety` named variant).
- **Retroactive re-validation** of existing endorsements against new strategies
  (grandfathering preserved — non-goal).
- **Weighted allowlist** (boolean membership only).
- **New migration** (table + column + consts all pre-landed).
- **New ENTRY_KIND consts** (both pre-landed RT-r1; registry count stays 55).
- **Single-file MIRROR-maximal handler restructure for the MiniMax A/B trial** —
  that trial does NOT gate or appear in this plan.

## 13. Step-by-step tasks

> **Cohort dispatch:** `[P]`-marked tasks with disjoint FILES YAML may be queued
> in parallel. Task 0 is always non-`[P]`. `requires:` names tasks whose impl
> commit must already be on `phase-v1-RT-r4`. A `[P]` task may not `requires:` a
> same-cohort peer. Cohort {1,2} is file-disjoint (db_schema vs api_common) and
> both have `requires: []` → parallel-safe. Tasks 3–7 are serial.

---

### Task 0 — Pre-flight harness audit (non-`[P]`, NO commit)

**ACTION:** Run the 14-probe pre-phase harness audit per
`.claude/rules/pre-phase-harness-audit.md`. On the Linux daemon use the
`./scripts/brehon/cargo-<verb>.sh` form (probe semantics identical to the Windows
`cmd //c "scripts\\brehon\\cargo-<verb>.bat"` form).

**Probes:**
0. Docker daemon: `docker ps > /dev/null 2>&1 && echo DOCKER_OK || exit 1`.
1. Wrapper honors `-p`: `./scripts/brehon/cargo-check.sh -p lemmy_utils` → only
   `lemmy_utils` compiles.
2. Feature-flag activation: `./scripts/brehon/cargo-check.sh -p lemmy_db_schema
   --features full` → compiles with features.
3. cargo-test honors target: `./scripts/brehon/cargo-test.sh --test e2e --no-run
   -p lemmy_server` → only e2e target.
4. Negative-exit propagation: bogus `--features nonexistent_xyz` on check AND test
   → BOTH non-zero exit.
5. Clippy baseline: `./scripts/brehon/cargo-check.sh --workspace --features full
   --no-deps` clippy → capture exit; non-zero = pre-existing debt, document.
6. Branch verify: `git branch --show-current` == `phase-v1-RT-r4`.
7. Prior-deliverable: `sponsor_allowlist` table migration present;
   `SponsorAllowlistId` newtype present.
8. Schema-cols: `surety.sponsored_id` + `surety.revoked_at` +
   `reputation_snapshot.can_sponsor` present in `schema.rs`.
9. Consts present: `ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED` +
   `ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED` in `governance_log.rs`.
10. Config-key: `onboarding.sponsor_gate_strategy` read path intact in
    `create_endorsement.rs`.
11. Index: `admin_sponsor_allowlist.rs` does NOT yet exist (deliverable c builds it).
12. Helper-absence: `sponsor_allowlist.rs` has NO query helpers yet
    (deliverable b adds them).
13. Concurrent-PR: `gh pr list --repo barrie-cork/lemmy --state open` — no open PR
    on `phase-v1-RT-r4`.
14. Registry-invariant: registry total ENTRY_KIND count == 55 BEFORE this phase.

**DoD:** all 14 probes pass. Any failure → file `kind: "blocker"` DQ via
`scripts/brehon/dq-v3-append-fragment.sh ... --pending`, commit+push, STOP.
NO code commit for Task 0.

---

### Task 1 `[P]` — Allowlist db-helpers (deliverable b)

```yaml
FILES:
  creates: []
  modifies:
    - crates/db_schema/src/source/governance/sponsor_allowlist.rs
  requires: []
```

**ACTION:** Add three inline `#[cfg(feature = "full")] pub async fn` helpers to
`sponsor_allowlist.rs`, mirroring `federation_peer.rs:55-94`.

**IMPLEMENT (file 1 of 1):**
- `sponsor_allowlist_insert(form: &SponsorAllowlistInsertForm, conn: &mut
  AsyncPgConnection) -> LemmyResult<SponsorAllowlist>` —
  `diesel::insert_into(sponsor_allowlist::table).values(form)
  .returning(SponsorAllowlist::as_returning()).get_result(conn).await?`.
- `sponsor_allowlist_delete(allowlist_id: SponsorAllowlistId, conn: &mut
  AsyncPgConnection) -> LemmyResult<usize>` —
  `diesel::delete(sponsor_allowlist::table.find(allowlist_id)).execute(conn).await?`.
- `sponsor_allowlist_exists(person_id: PersonId, community_id: Option<CommunityId>,
  conn: &mut AsyncPgConnection) -> LemmyResult<bool>` — filter
  `person_id.eq(person_id)` AND (`community_id.eq(c)` when `Some(c)` else
  `community_id.is_null()`); `.first::<SponsorAllowlist>(conn).await.optional()?
  .is_some()`. Build the community predicate with a branch on
  `community_id` (diesel `BoxedQuery` or two-arm `match`), NOT a single
  `.eq(community_id)` (that would match only NULL-vs-NULL incorrectly).

**MIRROR:** `federation_peer.rs:55` (`.optional()?` read), `:70` (insert returning).
**GOTCHA:** import `SponsorAllowlistId`, `PersonId`, `CommunityId` from their
newtype homes (`feedback_newtype_locations_lemmy_db_schema_vs_file.md`); the
`community_id IS NULL` branch must be a distinct query arm.
**VALIDATE (laptop):** `cargo check -p lemmy_db_schema --features full`; clippy
`-p lemmy_db_schema --features full --no-deps -- -D warnings`. (`-p` + `--features
full` valid for db_schema per `feedback_features_full_p_crate_incompatible` — this
crate DOES define `full`.)

---

### Task 2 `[P]` — Admin DTOs (deliverable d)

```yaml
FILES:
  creates: []
  modifies:
    - crates/api/api_common/src/governance.rs
  requires: []
```

**ACTION:** Add four DTOs to `api_common/governance.rs`, mirroring the
`AdminSetConfig`/`AdminSetConfigResponse` derive stack (`:444`/`:465`).

**IMPLEMENT (file 1 of 1):**
- `AddSponsorAllowlist { person_id: PersonId, community_id: Option<CommunityId>,
  note: Option<String> }`.
- `AddSponsorAllowlistResponse { allowlist_id: SponsorAllowlistId }`.
- `RemoveSponsorAllowlist { person_id: PersonId, community_id: Option<CommunityId> }`.
- `RemoveSponsorAllowlistResponse { success: bool }`.

**MIRROR:** `governance.rs:444`/`:465` derive stack verbatim
(`#[skip_serializing_none]` + `#[derive(Debug, Serialize, Deserialize, Clone,
Default, PartialEq)]` + the two `ts-rs` `cfg_attr` lines).
**GOTCHA:** newtype imports per `feedback_newtype_locations`.
**VALIDATE (laptop):** `cargo check -p lemmy_api_common --features full`; clippy
same scope `--no-deps -- -D warnings`.

---

### Task 3 — Strategy enum + dispatch (deliverable a, requires 1)

```yaml
FILES:
  creates: []
  modifies:
    - crates/api/api_crud/src/governance/create_endorsement.rs
  requires:
    - task: 1
```

**ACTION:** Extend `SponsorGateStrategy` + `parse()` + `label()` + add three match
arms in `process_endorsement`.

**IMPLEMENT (file 1 of 1):**
- Enum (`:84`): add `AgeOrSurety`, `Reputation`, `Allowlist` ABOVE `Unknown(String)`.
- `parse()` (`:92`): map `"age_or_surety"`/`"reputation"`/`"allowlist"` →
  the new variants; everything else → `Unknown(other.to_string())` (`:97`).
- `label()` (`:102`): return the canonical string for each new variant.
- `match &strategy` (`:161`): add three named arms ABOVE the `Unknown(s)` arm:
  - `AgeOrSurety` → `if enforce_age_gate(...).is_ok() OR surety EXISTS` (call
    `enforce_age_gate` `:331`; surety EXISTS via an inline filter on `surety`
    table `sponsored_id = sponsor_id AND revoked_at.is_null()`). On neither →
    deny with the existing age-gate error type.
  - `Reputation` → read `reputation_snapshot.can_sponsor` (scoped by
    `data.community_id`); false/absent → deny.
  - `Allowlist` → `sponsor_allowlist_exists(sponsor_id, data.community_id,
    conn).await?`; false → deny.

**MIRROR:** existing `Age`/`Unknown(s)` arms (`:168`/`:170-172`) for the gate-call
and error shape.
**GOTCHA-55a:** NO `_ =>` catchall — `Unknown(String)` stays the final exhaustive
arm (`feedback_clippy_test_style`). New arms are explicit and named.
**VALIDATE (laptop):** `cargo check --workspace --features full`; clippy
`--workspace --features full --no-deps -- -D warnings` (NOT `-p lemmy_api_crud
--features full` — api_crud `full` interaction per
`feedback_features_full_p_crate_incompatible`; use `--workspace`).

---

### Task 4 — Admin handlers + mod decl (deliverables c partial, requires 1+2)

```yaml
FILES:
  creates:
    - crates/api/api/src/governance/admin_sponsor_allowlist.rs
  modifies:
    - crates/api/api/src/governance/mod.rs
  requires:
    - task: 1
    - task: 2
```

**ACTION:** Create `admin_sponsor_allowlist.rs` with `add` + `remove` handlers;
add `pub mod admin_sponsor_allowlist;` (alphabetical) to `mod.rs`.

**IMPLEMENT (file 1 of 2 — admin_sponsor_allowlist.rs):**
- `pub async fn add(data: Json<AddSponsorAllowlist>, context: Data<LemmyContext>,
  local_user_view: LocalUserView) -> LemmyResult<Json<AddSponsorAllowlistResponse>>`.
- `pub async fn remove(...)  -> LemmyResult<Json<RemoveSponsorAllowlistResponse>>`.
- Both mirror `admin_set_config` (`:383-558`) step ordering:
  1. validation;
  2. `is_admin(&local_user_view)?` (`admin_config.rs:62`/`:753`);
  3. denial path: log denial OUTSIDE `run_transaction`, return error;
  4. `let admin_pseudonym = actor_pseudonym_helper::get_or_create(pool,
     admin_id).await?` (`:485`);
  5. `let person_pseudonym = actor_pseudonym_helper::get_or_create(pool,
     data.person_id).await?`;
  6. success path inside `run_transaction` (`:497`): **add** →
     `sponsor_allowlist_insert(&form, conn)` then `governance_log::append(...,
     ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED, payload, conn)`; **remove** → resolve row
     via `sponsor_allowlist_exists`-style lookup to get `allowlist_id`, then
     `sponsor_allowlist_delete(allowlist_id, conn)` then `append(...,
     ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED, payload, conn)`.

**MIRROR:** `admin_config.rs:383-558` (the conformance-audit axis most likely to
drift — §3.1.1). Pseudonym at `:485`; `run_transaction` write branch `:497-558`.
**GOTCHA:** `run_transaction` has NO SAVEPOINT (`admin_config.rs:10`);
`governance_log::append` opens its OWN internal tx — do NOT wrap append in a
second explicit tx. ADR-015: payload carries `person_pseudonym` +
`added_by_admin_pseudonym`/`removed_by_admin_pseudonym`, NEVER raw `person_id`.
For **remove** the row must be resolved first (person + community default) to fill
`allowlist_id` in the log payload.
**IMPLEMENT (file 2 of 2 — mod.rs):** `pub mod admin_sponsor_allowlist;`
alphabetically.
**VALIDATE (laptop):** `cargo check --workspace --features full`; clippy
`--workspace --features full --no-deps -- -D warnings`.

---

### Task 5 — Route registration (deliverable e, requires 4)

```yaml
FILES:
  creates: []
  modifies:
    - crates/api/routes/src/lib.rs
  requires:
    - task: 4
```

**ACTION:** Register the two routes in the admin scope (`:493-507`) + add the
`use` import.

**IMPLEMENT (file 1 of 1):**
- In the admin scope: `.service(scope("/sponsor-allowlist")
  .route("/add", post().to(admin_sponsor_allowlist::add))
  .route("/remove", post().to(admin_sponsor_allowlist::remove)))`.
- `use` import for `admin_sponsor_allowlist` matching the sibling admin-handler
  import style already in the file.

**MIRROR:** the existing admin-scope service registrations at `:493-507`.
**GOTCHA:** path prefix is `/api/v4/governance/admin/sponsor-allowlist/{add,remove}`
— register relative to the existing admin scope, do NOT re-prefix the full path.
**VALIDATE (laptop):** `cargo check --workspace --features full`; clippy
`--workspace --features full --no-deps -- -D warnings`.

---

### Task 6 — Registry-row flip (deliverable f, requires 4; ADVISOR-EXECUTED)

```yaml
FILES:
  creates: []
  modifies:
    - .claude/rules/governance-log-entry-kind-registry.md
  requires:
    - task: 4
```

**ACTION:** Flip the two registry rows (`:190-191`) from `(pending)` to the live
handler fn names landed in Task 4.

**IMPLEMENT (file 1 of 1):**
- Row `sponsor_allowlist_added`: handler →
  `admin_sponsor_allowlist.rs::add`.
- Row `sponsor_allowlist_removed`: handler →
  `admin_sponsor_allowlist.rs::remove`.
- Drop the `(pending)` marker from both.

**EXECUTION NOTE:** This file is under `.claude/**` — the Junior impl worker is
BLOCKED from writing it (worktree-guard). The advisor (or a laptop-side impl
session) executes this edit directly on `phase-v1-RT-r4`. The §13 task exists so
the dependency (requires 4) and the registry-invariant check (Task 0 probe 14;
count stays 55, no new consts) are tracked.
**GOTCHA:** RT-r4 emits EXISTING consts — it does NOT add new ENTRY_KIND consts.
Registry total stays 55. Do NOT increment the count.
**VALIDATE:** registry count unchanged (55); both rows no longer carry `(pending)`;
both name a real fn that exists in Task 4's committed `admin_sponsor_allowlist.rs`.

---

### Task 7 — e2e coverage (deliverable g, requires 3+4+5)

```yaml
FILES:
  creates: []
  modifies:
    - crates/server/tests/e2e.rs
  requires:
    - task: 3
    - task: 4
    - task: 5
```

**ACTION:** Add an e2e fixtures module covering: (i) each new strategy gates
endorsement correctly (`age_or_surety` passes via surety; `reputation` passes via
`can_sponsor`; `allowlist` passes via membership; each denies when the condition
is absent); (ii) admin add/remove round-trips through the HTTP endpoints and emits
the correct governance-log entries.

**IMPLEMENT (file 1 of 1):**
- New fixtures module mirroring the most recent RT/SL/JM fixtures sibling.
- Outer test fn returns `LemmyResult<()>`; helpers return `LemmyResult<T>`; NO
  `.map_err` bridges (Case A per `feedback_lemmy_error_no_std_error.md`).
- Pre-locate verbatim `old_string`/`new_string` anchors BEFORE editing
  (`feedback_fix_impl_pre_locate_e2e_anchors.md`) — e2e.rs is ~18k lines; anchor
  drift hangs the editor.

**MIRROR:** the most recent RT/SL/JM fixtures module Case A shape; read it at its
cited line range BEFORE authoring (canonical-schema-first gate).
**GOTCHA:** `e2e.rs` is never bundled with non-test logic (§5.2). Use
`feedback_async_pool_test_pattern` for pool setup.
**VALIDATE (laptop e2e):** `validate-pending-laptop-e2e` — `./scripts/brehon/
cargo-test.sh --workspace --test e2e --features full` via bat/sh wrapper; on
success append `E2E_EXIT_0` marker; capture exit code per cargo-output-capture.

---

### Task 8 — Retro (non-`[P]`)

Author the sub-phase retro per `feedback_retro_not_report` +
`feedback_four_role_retro_signals` + `feedback_retro_task_complexity_score`
(per-task `<files>/<commits>/<runtime-min>/<max-log-silence-min>`, aggregated in
§5). Commit subject `docs(retro): v1-RT-r4 — <slug>`.

## 14. Testing strategy

- **Unit/check:** per-task `cargo check` (scope per §13 VALIDATE lines).
- **Clippy:** `--workspace --features full --no-deps -- -D warnings` (Task 3
  GOTCHA-55a is the live clippy risk).
- **e2e:** Task 7 fixtures module, run via laptop bat/sh wrapper
  (`validate-pending-laptop-e2e`).
- **Manual structural verification:** `/brehon-verify` against §16a stories.

## 15. Validation / DoD

### 15.1 Workspace check
`./scripts/brehon/cargo-check.sh --workspace --features full` → exit 0.

### 15.2 Clippy
`./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
→ exit 0. (`--no-deps` mandatory to avoid upstream lint debt;
`--features full` mandatory to see governance code behind the gate.)

### 15.3 Test compile
`./scripts/brehon/cargo-test.sh --workspace --test e2e --no-run --features full`
→ exit 0.

### 15.4 e2e (laptop)
`./scripts/brehon/cargo-test.sh --workspace --test e2e --features full` (NOT
`-p lemmy_server --features full` — lemmy_server has no `full` feature; use
`--workspace`). On success append `E2E_EXIT_0` to the captured log.

### 15.5 Cross-cutting checklist
- [ ] No new migration added (`git diff --stat migrations/` empty).
- [ ] No new ENTRY_KIND const (registry count stays 55).
- [ ] Registry rows `:190-191` no longer carry `(pending)`.
- [ ] `SponsorGateStrategy` has 7 arms (4 named + Open + Closed + Unknown); NO
      `_ =>` catchall.
- [ ] `parse()` maps all three new strings; unrecognized → `Unknown(s)`.
- [ ] `label()` returns canonical string for each new variant.
- [ ] Three inline allowlist helpers present in `sponsor_allowlist.rs`, NOT in
      `impls/`.
- [ ] Four DTOs present with the `AdminSetConfig` derive stack verbatim.
- [ ] `admin_sponsor_allowlist.rs` mirrors `admin_set_config` step ordering
      (denial-log OUTSIDE tx; pseudonym via `get_or_create`; append owns own tx).
- [ ] ADR-015: no raw `person_id` in any log payload (only pseudonyms).
- [ ] ADR-008: governance_log written via `append`, never direct INSERT.
- [ ] `pub mod admin_sponsor_allowlist;` declared alphabetically in `mod.rs`.
- [ ] Routes registered under existing admin scope (no full-path re-prefix).
- [ ] `age_or_surety` reads `surety` (sponsored_id + revoked_at IS NULL).
- [ ] `reputation` reads `reputation_snapshot.can_sponsor`, community-scoped.
- [ ] `allowlist` uses `sponsor_allowlist_exists` (community + instance-wide rows).
- [ ] Grandfathering preserved (no retroactive re-validation).
- [ ] OQ-020: single strategy string, no composition.
- [ ] e2e outer fn `LemmyResult<()>`, helpers `LemmyResult<T>`, no `.map_err`.
- [ ] e2e covers gate-pass + gate-deny for all three strategies.
- [ ] e2e covers admin add/remove round-trip + governance-log emission.
- [ ] Per-task ceilings hold (≤4 files / ≤2 crates; e2e isolated).
- [ ] Commit subjects match `feat(rt-r4): … (task N)` / `chore`/`docs(retro):`.
- [ ] All §16a stories ✓ under `/brehon-verify`.

### 15.6 Shape G
N/A — Shape G SUSPENDED until 2026-06-01. Validation via
`validate-pending-laptop` / `validate-pending-laptop-e2e`.

## 16. Acceptance criteria

1. All four named strategies reachable and correctly gating endorsement.
2. Admin add/remove endpoints live at
   `/api/v4/governance/admin/sponsor-allowlist/{add,remove}`, admin-gated,
   emitting the two pre-landed ENTRY_KINDs with pseudonymised payloads.
3. Registry rows flipped to live; count unchanged at 55.
4. No migration; OQ-020 + grandfathering honored.
5. Full DoD (§15) green.

## 16a. Stories

### Story A — Strategy arms gate endorsement
- **Composing tasks:** 1, 3, 7.
- **Checkpoint command:** `./scripts/brehon/cargo-test.sh --workspace --test e2e
  --features full -- sponsor_gate_strategies` (fixtures module filter).
- **Expected output:** all strategy-arm tests pass (pass + deny per strategy).
- **Brief-Scope outputs to verify (`/brehon-verify` greps):**
  - `grep -n 'AgeOrSurety\|Reputation\|Allowlist' create_endorsement.rs` → enum +
    parse + label + 3 match arms present.
  - `grep -n 'sponsor_allowlist_exists' create_endorsement.rs` → allowlist arm
    wired to the helper.

### Story B — Admin allowlist maintenance
- **Composing tasks:** 2, 4, 5, 6.
- **Checkpoint command:** `./scripts/brehon/cargo-test.sh --workspace --test e2e
  --features full -- admin_sponsor_allowlist` (fixtures module filter).
- **Expected output:** add/remove round-trip tests pass; governance-log entries
  emitted with pseudonyms.
- **Brief-Scope outputs to verify:**
  - `test -f crates/api/api/src/governance/admin_sponsor_allowlist.rs` → exists.
  - `grep -n 'pub async fn add\|pub async fn remove' admin_sponsor_allowlist.rs`.
  - `grep -n 'sponsor-allowlist' crates/api/routes/src/lib.rs` → routes registered.
  - `grep -n 'admin_sponsor_allowlist.rs::add\|admin_sponsor_allowlist.rs::remove'
    .claude/rules/governance-log-entry-kind-registry.md` → rows flipped, no
    `(pending)`.

**Verification-mapping note:** Story checkpoints confirm behavior (tests pass);
Brief-Scope greps confirm the structural deliverables exist. `/brehon-verify`
catches phantom completions; CR triage catches regressions. Both required.

## 17. Completion checklist

- [ ] Task 0 harness audit: 14/14 probes pass (no commit).
- [ ] Tasks 1–7 committed on `phase-v1-RT-r4`, subjects `feat(rt-r4): <title>
      (task N)`.
- [ ] Task 6 registry flip executed by advisor/laptop-impl (Junior blocked from
      `.claude/**`).
- [ ] DoD §15 green (check / clippy / test-compile / e2e).
- [ ] `/brehon-verify` ✓ on both stories.
- [ ] Retro authored, subject `docs(retro): v1-RT-r4 — <slug>`.

## 18. Risks

| ID | Risk | Mitigation |
|---|---|---|
| W1 | `_ =>` catchall sneaks into the strategy match | GOTCHA-55a in §10/Task 3; clippy `-D warnings` catches it (`feedback_clippy_test_style`) |
| W2 | Handler invents single-tx-wrapping-append or SAVEPOINT | §10 mirror `admin_set_config` verbatim; `run_transaction` has no SAVEPOINT (`admin_config.rs:10`); append owns own tx |
| W3 | Raw `person_id` leaks into log payload (ADR-015 breach) | §10/Task 4 GOTCHA: payload carries pseudonyms only; `/brehon-verify` greps; conformance-audit §3.9.1 |
| W4 | `allowlist`/`reputation` community-scope query matches NULL-vs-NULL wrong | Task 1 GOTCHA: distinct `is_null()` query arm, not `.eq(community_id)` |
| W5 | e2e anchor drift hangs editor (18k-line file) | `feedback_fix_impl_pre_locate_e2e_anchors`; pre-locate verbatim anchors |
| W6 | e2e error-type mismatch (`Box<dyn Error>` vs `LemmyResult`) | Case A mirror; `feedback_lemmy_error_no_std_error` |
| W7 | `-p <crate> --features full` on api_crud/server fails | §13 VALIDATE uses `--workspace` for those; `-p` only for db_schema/api_common (`feedback_features_full_p_crate_incompatible`) |
| W8 | Registry count accidentally incremented | Task 0 probe 14 + Task 6 GOTCHA: RT-r4 emits existing consts, adds none; count stays 55 |
| W9 | Task 6 attempted by Junior worker (blocked from `.claude/**`) | §13 Task 6 EXECUTION NOTE: advisor/laptop-impl executes |
| W10 | Cohort {1,2} false-parallel (file overlap) | FILES YAML disjoint (db_schema vs api_common); both `requires: []`; advisor §4.1 step 4 overlap check |
| W11 | `requires:` chain mis-dispatched (Task 3 before Task 1 lands) | FILES YAML `requires:` threaded; advisor §4.1 step 4a dependency check |
| W12 | remove-handler can't fill `allowlist_id` for log | Task 4 GOTCHA: resolve row (person + community default) first |
| W13 | Composition/weighted-allowlist scope creep | §12 explicit out-of-scope; OQ-020 REJECTED |

## 19. Notes

- Lane mode A: worker commits directly to `phase-v1-RT-r4`.
- The single-file MIRROR-maximal handler shape for the separate MiniMax A/B trial
  is intentionally NOT part of this plan and does not gate it.
- Both ENTRY_KIND consts pre-landed RT-r1; this is their first emission site.

## 20. Confidence

High. Every MIRROR ref verified against live code this session
(`create_endorsement.rs:78-192`, `admin_config.rs:383-558`,
`governance.rs:444`/`:465`, `federation_peer.rs:55-94`, `surety.rs:17-31`,
`reputation_snapshot.rs:17-46`, `sponsor_allowlist.rs:11-40`). The dependency
chain is tight and serial; the two independent strategy arms sit inside one file.
The only residual risks are mechanical (clippy catchall, e2e anchor drift, tx
shape) and all carry a cited lesson + §13 GOTCHA.
