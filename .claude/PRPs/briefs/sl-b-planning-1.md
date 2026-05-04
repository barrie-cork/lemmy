# SL-b planning brief

**Written**: 2026-05-04 by advisor session (laptop, brehon-fork CWD) for Junior dispatch on EliteDesk.
**Subagent target**: `planning` (Opus 4.7, color purple — see `.claude/agents/planning.md`).
**Worktree**: Junior cuts `junior/sl-b-planning-1` from `governance-v0` per concurrency-1 default; the planning subagent commits the plan file and pushes back to `governance-v0` at finalize.
**Authority anchor**: `v1-sponsor-liability.prd.md` §5 (endpoint definition) + §15 row 2 (`v1-SL-b — revoke_endorsement handler + DTO + route + integration tests`). v1-SL-a shipped 2026-05-04 via PR #111 squash-merge at governance-v0 HEAD `790f6101d`; retro signed off at `e00819e2d`. SL-b is the second SL-lane sub-phase to plan from this CWD.

---

## 1. Role + dispatch line

`[role:planning] v1-sponsor-liability-b plan — revoke_endorsement handler + DTO + route + integration tests`

The actual `mcp__junior-brehon__create_task` description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] v1-sponsor-liability-b plan — see .claude/PRPs/briefs/sl-b-planning-1.md
```

Everything else lives in this brief.

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` for sub-phase **v1-SL-b**, the `revoke_endorsement` handler + DTO extension + route + integration tests. The plan covers PRD §5 in full plus the §12 security knobs that bind on the handler. SL-a shipped the schema, enum variants, ENTRY_KIND consts, and config seeds; SL-b reads them and writes to them.

### 2.1 Six concrete deliverables (per PRD §5 + §12)

The plan's §13 task list MUST cover all six:

a. **Extend `RevokeEndorsement` DTO + add `RevokeEndorsementResponse`** in `crates/api/api_common/src/governance.rs`. The existing struct at line 311 (verified at brief-write time) has only `endorsement_id: EndorsementId` — PRD §5.1 specifies a `reason: String` field and §12.2 marks it required (length > 0 after trim). The current shape is `#[derive(... Copy, Default ...)]` — adding `reason: String` will force removing `Copy` (String isn't Copy) and may force removing `Default` (or accepting an empty-string default that the handler rejects). Plan §13 must specify which derives stay and which are dropped, with rationale. **Add new `RevokeEndorsementResponse` struct** (does not currently exist; verified at brief-write time): `endorsement_id: EndorsementId`, `revoked_at: DateTime<Utc>`, `liability_chain_severed_for_cases: Vec<ModerationCaseId>`. Mirror the `CreateEndorsementResponse` derive stack at line 302 (verify exact derive list at planning time — `skip_serializing_none` + ts-rs gating).

b. **Create new handler `revoke_endorsement.rs`** at `crates/api/api_crud/src/governance/revoke_endorsement.rs`. CRUD-shaped sibling of `create_endorsement.rs` (361 lines, verified at brief-write time — read in full as the canonical mirror). Module wiring: add `pub mod revoke_endorsement;` to `crates/api/api_crud/src/governance/mod.rs`. The handler is the body of the work — see §2.2 for the inside-transaction step list.

c. **Register the route** at `crates/api/routes/src/lib.rs:523` (verified — the existing `/endorsement` route is on line 523, NOT 518 as PRD §5.6 cites; PRD was authored against an older HEAD). Add `.route("/endorsement/revoke", post().to(revoke_endorsement))` immediately after the existing `/endorsement` line, inside the same `scope("/governance")` block. Import the handler at the route file's `use` block (see line 149 `create_endorsement::create_endorsement,` for the pattern).

d. **Integration tests** for the handler. The plan §13 must enumerate per-test-function tasks (one Edit per task, anchor-pattern based at file end) per `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is now ~10,300+ lines. Required test branches per PRD §5.3 + §5.4 + §12:
   1. **Self-revoke success path** — caller is sponsor, endorsement exists and is unrevoked, surety is updated, both snapshots recompute, `endorsement_revoked` log entry emitted.
   2. **Admin-revoke success path** — caller is admin (not sponsor), endorsement exists, all writes proceed identically; admin's pseudonym recorded in the log payload.
   3. **Re-revoke idempotency** — same endorsement revoked twice; second call returns existing `revoked_at` with empty `liability_chain_severed_for_cases: vec![]` and emits NO new log entry (verify governance_log row count unchanged).
   4. **Capability rejection — non-sponsor non-admin** — caller is neither sponsor nor admin; handler returns `LemmyErrorType::NotFound` (do NOT leak the existence of the endorsement).
   5. **Reason validation** — empty-string `reason` (or whitespace-only after trim) rejects with `LemmyErrorType::Unknown("revoke-endorsement reason required".to_string()).into()` per **DQ #139 (advisor 2026-05-04 via /brehon-clarify)** mirroring `admin_close_case.rs:28-32` canonical pattern. Do NOT add a new lemmy_utils variant (out of scope per §4.3).
   6. **Rate-limit enforcement** — caller has revoked `liability.revoke_rate_limit_per_day` (default 5) endorsements in the rolling 24h window; 6th revocation within 24h rejects with `LemmyErrorType::RateLimitError`. Admin caller bypasses the limit. **Per DQ #140 (advisor 2026-05-04 via /brehon-clarify):** the bypass is recorded by adding a `rate_limit_bypassed: true` field to the existing `endorsement_revoked` log entry payload, ONLY when (admin AND would-have-been-rate-limited). NO new ENTRY_KIND const (preserves §2.3 scope). Test #6 asserts the field is `true` when admin bypasses an at-threshold rate-limit; test #1 (self-revoke under threshold) asserts the field is absent. **Per DQ #141 (advisor 2026-05-04 via /brehon-clarify):** tests #2 and #6 stay separate; #2 exercises admin capability under-threshold (no rate-limit considered), #6 exercises admin bypass at-threshold (5 prior revocations seeded for admin caller).
   7. **Grace-window severance — single sponsor case** — synthetic `SponsorLiabilityPending` case inserted via direct DB-write (test pre-seeds `moderation_case` with `status = SponsorLiabilityPending`, `grace_expires_at = now() + 24h`, `target_person_id = sponsee`); caller revokes their endorsement; case transitions to `SponsorLiabilityEscaped`, `liability_escape_reason` JSONB matches PRD §8.1 schema with `version: 1`, `actor_pseudonym` not raw `caller_id`; response's `liability_chain_severed_for_cases` contains the case id; `sponsor_liability_escaped` log entry emitted.
   8. **Grace-window severance — multi-sponsor `any_revocation` rule (default)** — case has 3 active sponsors; one revokes; case escapes; the other 2 sponsors' surety rows are untouched (only the revoking sponsor's surety flips `revoked_at`).
   9. **Grace-window non-severance — no matching pending case** — endorsement exists, sponsor revokes, but target has no `SponsorLiabilityPending` cases; revocation succeeds, response's `liability_chain_severed_for_cases: vec![]`, only the `endorsement_revoked` log entry fires (no `sponsor_liability_escaped`).

   Tests #7-9 are the SL-b-specific code path; tests #1-6 are the per-PRD §5/§12 contract. Plan §13 should split per-test (9 tasks just for tests, plus the handler/DTO/route tasks). The §16a Stories should group these into story-level acceptance.

e. **Read-cascade for `liability.multi_sponsor_escape_rule`** — handler must read the community-scoped config key first (case's `community_id`), fall back to instance, fall back to default `"any_revocation"`. Default-string lives at `crates/api/api/src/governance/config.rs:937` (`DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE: &str = "any_revocation"` — verified at brief-write time). Use `ConfigCache` + `Scope::Community(case.community_id)` per Phase 5a `create_endorsement.rs:enforce_age_gate` pattern (line 339-361 in `create_endorsement.rs`). **Per DQ #142 (advisor 2026-05-04 via /brehon-clarify):** read PER CASE inside the grace-window loop, NOT once before the loop. Target persons may have cases across multiple communities (PRD §11.2 backfill is instance-wide), and the escape rule is per-community. The same `ConfigCache` instance threaded through the tx closure deduplicates duplicate-community lookups automatically (verified at `config.rs:209-256`). Concretely: inside step 6 of the inside-transaction body, call `config::get_str_cascade(cache, conn, Scope::Community(case.community_id), "liability.multi_sponsor_escape_rule")` per iteration. Plan §4 watchpoint #3 must explicitly cite this per-case pattern.

f. **Rate-limit query** — `count(*) FROM endorsement WHERE from_person_id = caller_id AND revoked_at > now() - INTERVAL '24 hours'`. Must run at handler entry, BEFORE entering `run_transaction` (per `create_endorsement.rs:198-214` cooldown pattern — the rate-limit-check pattern is read-then-error-or-continue, no transaction needed for the count). Handler entry order: (1) `check_local_user_valid`, (2) `is_admin` check (sets bypass flag), (3) IF NOT admin: rate-limit count + reject if exceeded, (4) reason validation + trim, (5) THEN open `run_transaction` per §2.2.

### 2.2 Inside-transaction step ordering (PRD §5.3 — load-bearing for plan §13 task split)

PRD §5.3 enumerates the 6-step transactional body. Plan §13 must order these correctly; reversing steps 3 and 4 yields stale severance results (see §4 watchpoint #2):

1. **Load endorsement** with `FOR UPDATE` — verify `revoked_at IS NULL`; if already revoked, EARLY-RETURN existing `revoked_at` with empty `liability_chain_severed_for_cases` (idempotency per PRD §5.4 — read-inside-transaction to avoid TOCTOU).
2. **Verify capability** — caller is sponsor (`endorsement.from_person_id == caller_id`) OR admin; else `Err(NotFound)`. (Already done via `is_admin` flag set pre-tx; the in-tx check is the sponsor-id comparison.)
3. **Set `endorsement.revoked_at = now()`** via UPDATE.
4. **Set sibling `surety.revoked_at = now()`** — query for `surety` row matching `(from_person_id = caller_id, to_person_id = endorsement.to_person_id, community_id = endorsement.community_id)`; UPDATE if found (the sibling-row-creation pattern is documented in `create_endorsement.rs` step 3-4 — verify the surety lookup signature at planning time). May not exist (community-scoped vs unscoped endorsement); UPDATE no-op is fine.
5. **Re-query active sponsors** — find all `surety` rows with `to_person_id = endorsement.to_person_id, revoked_at IS NULL` AFTER step 4. (Step 4 already flipped this caller's `revoked_at`; the re-query reads the post-update state.)
6. **Grace-window evaluation** — query for `moderation_case` rows with `status = 'SponsorLiabilityPending'` AND `target_person_id = endorsement.to_person_id` AND `grace_expires_at > now()`. For each:
   a. Apply community-configured escape rule (read once before this loop per §2.1 deliverable e). Default `any_revocation` — any sponsor revoke escapes. `all_revocation` — only escapes if NO active sponsors remain (i.e. every sponsor has revoked). `majority_revocation` — escapes when >50% of pre-revoke sponsors have revoked.
   b. If escape condition met:
      - Transition case to `SponsorLiabilityEscaped` (UPDATE).
      - Set `liability_escape_reason = json!({"version": 1, "reason": "sponsor_revoked", "actor_pseudonym": caller_pseudonym, "endorsement_id": endorsement.id.0})`. **`version: 1` is mandatory per OQ-V1-SL-05 forward-compat; `actor_pseudonym` is mandatory per ADR-015 GDPR — never raw `caller_id`.**
      - Emit `sponsor_liability_escaped` log entry (using existing `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` const — SL-a shipped it at `governance_log.rs:197`).
      - Append case_id to outcome's `liability_chain_severed_for_cases`.
7. **Recompute snapshots** — `reputation_snapshot::recompute_snapshot(conn, sponsor_id, community_id, &mut config)` for both sponsor and sponsee, mirroring `create_endorsement.rs:307-309`. Plan §13 must include both calls; missing the sponsee recompute leaves stale `can_sponsor` for downstream gates (PRD §5.3 step 6).
8. **Emit `endorsement_revoked` log entry** ALWAYS (even if no severance). Use existing `ENTRY_KIND_ENDORSEMENT_REVOKED` const (SL-a shipped at `governance_log.rs:195`). Payload: `{"sponsor_pseudonym": ..., "target_pseudonym": ..., "community_id": ..., "reason_redacted": <scrubbed reason string>}`. The `reason` field flows through `governance_log::append`'s scrub layer per ADR-015.

### 2.3 Scope boundary — what is NOT in SL-b

Per PRD §15 phase table:

- **SL-c (next):** Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring. Reads `moderation_case.grace_expires_at`. SL-b does NOT author the scheduler.
- **SL-d (after SL-c):** `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition. SL-b's tests insert `SponsorLiabilityPending` cases by direct DB-write (synthetic seed); SL-d's mutation is what creates them in production.
- **SL-e (after SL-d):** Lane-wide e2e suite (full revocation-during-window-escapes flow exercising SL-c scheduler + SL-d transition). SL-b's integration tests #7-9 pre-seed pending cases directly; they exercise revocation severance in isolation, not the full lane.

**Hard out-of-scope for SL-b** (per PRD §2 OUT + §15):

- The scheduler tick (SL-c).
- The `apply_sponsor_liability` compute/fire split (SL-d).
- The `submit_jury_vote` mutation that drives `Decided → SponsorLiabilityPending` (SL-d).
- Restoration completion endpoint (out — owned by restorative-mechanics-v1 PRD).
- Notification UX (out — PRD §13 OQ-V1-SL-03).
- Cross-instance federation of revocation events (out — v2 per ADR-014).
- Step-up auth for admin revocation (out — v2 reservation per PRD §12.3; do NOT add `step_up_token` field).
- Any new migration. SL-a shipped the schema; SL-b adds zero migrations. Plan §13 must NOT include a migration task. (Watch §4 #11.)
- Any new ENTRY_KIND_* const. All 5 needed consts shipped in SL-a (`endorsement_revoked` 195, `restoration_completed` 196, `sponsor_liability_escaped` 197, `sponsor_liability_fired` 198, `sponsor_liability_pending` 199 in `governance_log.rs`). Plan §13 must NOT add to the registry. (Watch §4 #10.)
- New CaseStatus variants. SL-a shipped 3 (`SponsorLiabilityPending/Fired/Escaped` at `enums.rs:411-432`). SL-b uses them; does not add.
- Any `governance_config` seed beyond what SL-a shipped. The 13 SL-a seeds (10 `liability.*` + 3 `job.grace_check_*`) cover SL-b's needs (rate-limit + multi-sponsor rule + restoration knobs already seeded). SL-b READS them; does not seed. If the planner discovers a needed knob not in SL-a's seed list, file a `kind: "blocker"` DQ — do not silently add a seed.

### 2.4 Plan file deliverable shape

- **One file:** `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md`.
- **Follow `.claude/PRPs/templates/plan.template.md` 20-section schema literally.** §16a Stories is mandatory (post-spec-kit-adoption).
- **Shape G applies — SL-b is post-Shape-G** (after JM-e + SL-a). §15 DoD MUST use the per-workflow shape (workflow path + phase-branch SHA + `conclusion: "success"`). Inline cargo invocations are forbidden in §15.
- **§5 complexity score with breakdown table** per `feedback_complexity_score_pre_split.md`. **Per DQ #138 (advisor 2026-05-04 via /brehon-clarify):** the planner computes the actual score from the §5.1 breakdown table at write-time and applies the >8 split-or-proceed gate then. Pre-estimate range is roughly 6-10 depending on how the planner counts e2e Edits (each test is a single anchor-Edit at file end, so 9 tests = 9 effective edits — ~9-10 with edit-weight 0.5). If the final tally exceeds 8, planner MUST file split-or-proceed `kind: "blocker"` DQ before finalising §13. Possible split: SL-b1 (handler + DTO + route + tests #1-6) / SL-b2 (severance tests #7-9). Or proceed without split if the planner judges the tasks individually well-scoped (each test is a single anchor-Edit, low compile-fix risk; SL-a ran 9 §13 tasks at score 7 cleanly).
- **§16a Stories** — every story names (a) composing §13 tasks, (b) a checkpoint command in the Shape-G workflow shape, (c) Brief-Scope outputs to verify (file:line + symbol) per `feedback_advisor_watchpoint_specificity.md`. Concept-only watchpoints rejected. Likely 3 stories: (1) "DTO extension + handler module + route registration" (composing handler/DTO/route tasks); (2) "Capability + reason + rate-limit guards enforced" (composing tests #1-6); (3) "Grace-window severance correctly cascades to SponsorLiabilityEscaped + actor_pseudonym + version=1 reason JSON" (composing tests #7-9).
- **§4 watchpoints** — every entry cites a specific file, table, or `schema.rs`/handler line. No abstract concepts. The 9 SL-b watchpoints from `advisor-context-v1-SL-b.md §4` are the seed list — plan §4 MUST cite each with the specific gate location. Plan §4 may add others surfaced at planning time (e.g. async-fn vs async-closure pattern in e2e.rs per `feedback_e2e_filter_assumes_naming.md`).
- **Explicit scope-boundary section in §6** ("Relationship to other v1-SL sub-phases") listing what SL-c/SL-d/SL-e will ship and confirming SL-b does not duplicate their work.

**Commit only the plan file.** Do not author Rust code, do not open PRs, do not touch any file under `crates/`, `migrations/`, or `tests/`. Junior finalize pushes the plan-file commit to `junior/sl-b-planning-1`; advisor merges it into `governance-v0` after DoD smoke + watchpoint-specificity gates pass and the user approves.

---

## 3. Required reading (in order, before drafting any plan section)

1. `.claude/agents/planning.md` — your subagent contract; defines model, tools, "Before you start (always)" sequence, plan-content discipline cross-references.
2. `.claude/PRPs/templates/plan.template.md` — canonical Brehon plan template (20 sections incl. §15.6 Shape-G DoD and §16a Stories). Structure is load-bearing.
3. `.claude/PRPs/prds/v1-sponsor-liability.prd.md` — the parent PRD. **Read in full the first time.** Specifically:
   - §1 (vision/goals — Brehon `athgabál` framing; gives motivation).
   - §2 (scope IN/OUT — confirm SL-b is handler/DTO/route/tests only).
   - §3.1 + §3.4 (CaseStatus extensions — confirm SL-a shipped the 3 variants; SL-b reads `SponsorLiabilityPending`, writes `SponsorLiabilityEscaped`).
   - §5 (`POST /api/v4/governance/endorsement/revoke` — **§5 IS the SL-b spec**; read §5.1 DTO + §5.2 capability check + §5.3 effect + §5.4 idempotency + §5.5 backfill + §5.6 route).
   - §7.3 (restoration interaction — SL-b doesn't write restoration escapes; SL-c does. But the `liability.restoration_*` config keys exist; confirm SL-b's read-cascade is for `liability.multi_sponsor_escape_rule` only, not `restoration_*`).
   - §8.1 (`liability_escape_reason` JSONB schema — **load-bearing for SL-b**; the `version: 1` field is mandatory per OQ-V1-SL-05).
   - §10 (Defaults Matrix — confirm `liability.revoke_rate_limit_per_day` default 5, `liability.multi_sponsor_escape_rule` default `"any_revocation"` — both already seeded by SL-a).
   - §11 (backwards compat — §11.2 mid-flight cases backfilled to `SponsorLiabilityPending` in SL-a are real test surfaces; SL-b's test #7 may exercise this path explicitly).
   - §12 (security — §12.1 rate-limit, §12.2 reason-required + scrub, §12.3 step-up reservation = NO ENFORCEMENT in SL-b).
   - §15 (implementation phases — confirms SL-b is row 2; SL-c+ depend on SL-b).
   - §17 (cross-cutting impact — names ADR-013 enum-exhaustiveness; confirm no new variants in SL-b).
   - §18 (resolutions applied — B4 key-rename table for `liability.*` is authoritative).
4. **`crates/api/api_crud/src/governance/create_endorsement.rs`** — **read the entire file (361 lines).** This is the canonical mirror SL-b's handler is shaped on. Specifically:
   - Lines 1-34 (module doc-comment shape — SL-b's handler doc-comment mirrors this with RevokeEndorsement-specific Watch citations).
   - Lines 35-60 (use block — verify exact import paths for `actix_web::web::{Data, Json}`, `lemmy_api::governance::{actor_pseudonym_helper, config, governance_log, reputation_snapshot}`, `lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid}`).
   - The outer handler (entry, `check_local_user_valid`, `is_admin` check, pseudonym setup, `run_transaction` open).
   - The transaction body (`process_endorsement` analogue — for SL-b name it `process_revocation`).
   - The recompute_snapshot calls at lines 307-309 (sponsor + sponsee pattern).
   - The governance_log::append at lines 311-328 (payload shape, scrub layer entry).
5. `crates/api/api_common/src/governance.rs` — the DTO file. Specifically:
   - Lines 311-313 (existing `RevokeEndorsement` — currently single-field `endorsement_id: EndorsementId`; SL-b extends with `reason: String` and adds `RevokeEndorsementResponse`). **Confirm at planning time that the DTO derive stack drops `Copy` cleanly — String isn't Copy, so `#[derive(... Copy ...)]` will need amendment.**
   - Lines 290-310 (`CreateEndorsement` + `CreateEndorsementResponse` — the response-derive-stack mirror for `RevokeEndorsementResponse`).
6. `crates/api/api_crud/src/governance/mod.rs` — read; SL-b adds `pub mod revoke_endorsement;` per the `create_endorsement.rs` precedent; verify alphabetical position.
7. `crates/api/routes/src/lib.rs` — read lines 145-160 (use block) + lines 510-540 (the `/governance` scope). Specifically:
   - Line 149 `create_endorsement::create_endorsement,` (the import pattern SL-b mirrors).
   - Line 523 `.route("/endorsement", post().to(create_endorsement))` — SL-b inserts the new route immediately after this line (NOT line 518 as PRD §5.6 cites; PRD pre-dates Phase 5c+ route additions).
8. `crates/api/api/src/governance/governance_log.rs` — confirm:
   - Line 195 `ENTRY_KIND_ENDORSEMENT_REVOKED: &str = "endorsement_revoked"`.
   - Line 197 `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED: &str = "sponsor_liability_escaped"`.
   - The shim's `pub use` re-export block (verify SL-a wired re-exports; if missing, file a `kind: "blocker"` DQ — but SL-a Task 7 shipped the dual-file edit).
9. `crates/api/api/src/governance/sponsor_liability.rs` — read entirely. SL-b's handler doesn't call into this file directly, but the file documents the SL-b-relevant state space:
   - The v0 `apply_sponsor_liability` shape (which SL-d will eventually split per PRD §9.1).
   - Watch 10 PII discipline: governance-log payloads carry `*_pseudonym` strings, never raw `*_id`. SL-b inherits this discipline.
   - Watch 3 exhaustive-match: severity_for_action with no `_ =>` wildcard. SL-b doesn't extend this match but inherits the no-wildcard discipline.
10. `crates/api/api/src/governance/config.rs` — find and read:
    - The default-string for `liability.multi_sponsor_escape_rule` (added in SL-a Task 5; verify const name + value).
    - The `ConfigCache` API + `Scope::Community` / `Scope::Instance` / fallback pattern (per `create_endorsement.rs:enforce_age_gate` lines 339-361).
    - The default for `liability.revoke_rate_limit_per_day` (SL-a Task 5 seeded; verify default=5).
11. `crates/db_schema_file/src/enums.rs:411-432` — confirm:
    - `SponsorLiabilityPending` variant at the enum (the test cases pre-seed cases in this status).
    - `SponsorLiabilityEscaped` variant (the SL-b handler writes this status).
12. `crates/db_schema/src/source/moderation_case.rs` (or wherever the Diesel struct lives — confirm path at planning time) — verify:
    - `grace_expires_at: Option<DateTime<Utc>>` (SL-a Task 3 added — must exist).
    - `liability_escape_reason: Option<serde_json::Value>` (SL-a Task 3 added — must exist).
    - The `ModerationCaseUpdateForm` (or equivalent) — SL-b's tests UPDATE these columns; confirm the InsertForm/UpdateForm has them.
13. `crates/db_schema/src/source/endorsement.rs` (confirm path) — verify the `Endorsement` struct + `EndorsementUpdateForm` for `revoked_at` UPDATE shape. Mirror what `create_endorsement.rs` uses for the insert; the UPDATE pattern is symmetric.
14. `crates/db_schema/src/source/surety.rs` (confirm path) — verify the `Surety` struct + `SuretyUpdateForm` for the sibling-row revoke. The lookup key is `(from_person_id, to_person_id, community_id)`; verify Diesel filter shape.
15. `crates/server/tests/e2e.rs` — locate the file (~10,300 lines). Read:
    - The first 100 lines (test-setup helpers — `setup_e2e_pool`, fixture builders).
    - The most-recent SL-a tests (find via `git log -p --follow crates/server/tests/e2e.rs | head -200` to see what test functions SL-a added). SL-b's tests anchor at file-end; verify the pattern.
    - The orphan-case ModerationCaseInsertForm pattern at line 9989+ — **MUST NOT mutate `Decided` cases**; SL-b's tests pre-seed `SponsorLiabilityPending` only.
    - Existing endorsement-test helpers (grep for `create_endorsement` test calls; SL-b tests likely reuse them for setup).
16. **Glob `.claude/lessons/feedback_*.md` and Read every file whose name keyword matches:** `multi_write_handlers`, `transactions`, `idempotency`, `recompute_snapshot`, `cooldown`, `rate_limit`, `pseudonym`, `pii`, `gdpr`, `redaction`, `scrub`, `actor_pseudonym`, `e2e_filter`, `e2e_edit_hang`, `junior_worker_e2e`, `complexity_score`, `pre_queue`, `watchpoint`, `dod`, `validate`, `shape_g`, `validate_pending`, `governance_log`, `entry_kind`, `seed`, `parametric`, `dual_file`, `re_export`, `cargo`, `features-full`, `features_full_p_crate`, `wrapper-script`, `clippy`, `test-style`, `pre_phase_dod`, `dry_run`, `wrapper_silence`, `principles_not_rules`, `read_canonical`, `parallel_cohort`, `cohort_yaml`, `four_role`, `retro_not_report`, `insertform`, `propagation`, `micros`, `micros_scaled`, `branch_switch`, `commit_aggressively`, `daemon_finalize`, `pre_phase_harness_audit`. That is the lessons-corpus discipline per `planning.md` step 3.
17. **Glob `.claude/lessons/reference_*.md` and Read every file whose name keyword matches:** `governance_log_entry_kind_registry`, `phase_branch`, `branch_manager`, `worktree`.
18. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — read **ADR-010** (won't-disadvantage rule), **ADR-013** (CaseStatus enum-exhaustiveness — no `_ =>` arms in SL-b's matches), **ADR-014** (federation deferral — SL-b emits log events on local instance only; do NOT outbox-emit), **ADR-015** (pseudonymisation; **load-bearing for SL-b** — every `liability_escape_reason` JSONB field that names a person uses pseudonym, not raw id). OQ-V1-SL-05 (`liability_escape_reason` schema versioning — `version: 1` from day one). OQ-V1-SL-01 (multi-sponsor escape rule).
19. `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — the immediate predecessor. Read §10 (file inventory; SL-b doesn't add new files except the handler + tests but uses the SL-a-shipped types), §13 task list (for shape consistency), §15 DoD (Shape G workflow shape SL-b copies). SL-b's plan §13 should pattern-match SL-a's tasks for consistency.
20. `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — the most recent post-Shape-G plan. Read §15 per-workflow DoD shape, §16a Stories shape, §5 complexity score breakdown table, the `[P]` cohort markers + FILES YAML blocks in §13. SL-b's plan §13 may use `[P]` cohorts if test tasks are independent (they should be — each test is its own anchor-Edit), but planner judges at write time.
21. `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md` — the v0 sponsor-liability foundation. Read §13 to see the v0 `apply_sponsor_liability` task shape; SL-b's handler doesn't call into this code but the v0 sponsor-liability test branches are precedents for SL-b's test data shape (creating endorsements, sureties, sponsees).
22. `.claude/PRPs/briefs/sl-a-planning-1.md` and `.claude/PRPs/briefs/jm-e-planning-1.md` — exemplar planning briefs for shape and constraint language.
23. `.github/workflows/cargo-validate-workspace.yml` + `.github/workflows/cargo-validate-features-full.yml` + `.github/workflows/cargo-test-e2e.yml` — the existing Shape-G workflows the plan §15 references. Verify they carry `--no-deps -- -D warnings` and `--features full` where applicable. SL-b does NOT add new workflows or edit existing ones; if any flag is missing, file a DQ.
24. **DQ #67 resolved (per user 2026-04-27)** — workflow YAML DoD dry-run discipline. Honoured by SL-b §15: do NOT prescribe `act` invocations; do NOT require pre-merge full dry-run.

---

## 4. Constraints (hard rules — violating any of these is a process breach)

### 4.1 Plan-content discipline

- **Shape G DoD shape (forward-only).** §15 MUST use the per-workflow shape per `.claude/PRPs/templates/plan.template.md` §15.6. Inline cargo invocations are forbidden in §15. Per `feedback_schema_changing_spec_retrofit_question.md`. SL-b's two workflows are `cargo-validate-workspace.yml` (workspace-check on `junior/*`) and `cargo-test-e2e.yml` (e2e on `phase-v1-SL-b` after finalize-merge — `workflow_dispatch`-only per PR #105).
- **§16a Stories mandatory** (NOT optional). Every story names composing §13 tasks, a Shape-G checkpoint command, Brief-Scope outputs to verify (file:line + symbol), and lists the §13 IMPLEMENT entries it covers. Per `.claude/PRPs/templates/plan.template.md` §16a + `.claude/commands/brehon-verify.md`. Likely 3 stories (see §2.4).
- **§4 watchpoints cite specific files / handlers / `enums.rs` lines**, never abstract concepts. Per `feedback_advisor_watchpoint_specificity.md`. Concrete watchpoints SL-b's plan §4 MUST include — these are the 11 watchpoints from `advisor-context-v1-SL-b.md §4`, restated here for the planner's convenience but with citation gates:
   1. **TOCTOU on re-revoke.** Plan §13 task that authors `revoke_endorsement.rs` cites "step 1: load endorsement → check `revoked_at IS NOT NULL` → early-return — all inside `run_transaction`." Separate-read-then-tx-write pattern is a TOCTOU bug; flag at advisor plan-approval gate.
   2. **Step ordering — sibling surety revoke BEFORE re-query active sponsors.** Plan §13 must order: revoke endorsement → revoke matching surety → THEN re-query active sponsors → evaluate escape rule. Reversing yields zero or stale severance results.
   3. **Escape condition is community-configured + fall-back pattern.** Plan §13 must cite `crates/api/api/src/governance/config.rs` (read-cascade pattern from Phase 5a) and the default-string const for `liability.multi_sponsor_escape_rule`. Hard-coding the escape rule is a regression.
   4. **`liability_escape_reason` JSON schema locked.** Schema: `{"version": 1, "reason": "sponsor_revoked", "actor_pseudonym": "...", "endorsement_id": ...}`. `actor_pseudonym` mandatory per ADR-015 (GDPR pseudonymisation); `version: 1` mandatory per OQ-V1-SL-05. Plan §13 must cite `actor_pseudonym_helper::get_or_create` from `create_endorsement.rs:314-315` as the source. Raw `caller_id` in reason JSON is a GDPR-013 violation, catch-fire.
   5. **Response `liability_chain_severed_for_cases` populated only on actual severance.** Re-revocation returns `vec![]`. Test #3 (re-revoke idempotency) MUST assert empty vec + NO new log entries + NO snapshot recompute. Without this assertion the idempotency claim is unverified.
   6. **Two recompute-snapshot calls inside the same transaction.** Plan §13 must cite `create_endorsement.rs:307-309` and call `recompute_snapshot` for BOTH sponsor and sponsee. Missing the sponsee recompute leaves stale `can_sponsor` for downstream gates.
   7. **No new `moderation_case.status` mutations OUTSIDE the grace-severance step.** SL-b touches case status ONLY for `SponsorLiabilityPending → SponsorLiabilityEscaped` transitions. MUST NOT mutate `Decided` cases (PRD §11 backwards-compat); MUST NOT create new `SponsorLiabilityPending` cases (SL-d's job). The orphan-case ModerationCaseInsertForm pattern at `e2e.rs:9989+` (creator_id=NULL + winning_decision=NoAction) is the cautionary anchor — never mutate orphan or decided cases.
   8. **PRD §11.2 mid-flight cases at v1 deploy** — SL-a Task 1 backfill creates `SponsorLiabilityPending` cases at v0→v1 transition. SL-b's test #7 should include a "v0-mid-flight case backfilled to SponsorLiabilityPending, sponsor revokes during 24h grace, case escapes" branch.
   9. **Route registration site is line 523**, NOT 518 (PRD §5.6 cites stale line). Plan §13 task that adds the route must include `grep -n '"/endorsement"' crates/api/routes/src/lib.rs` to find the current line, then anchor-insert immediately after.
   10. **No new ENTRY_KIND_*** — all 5 already shipped in SL-a. Plan §13 must NOT add to the registry. Use existing const names: `ENTRY_KIND_ENDORSEMENT_REVOKED` (governance_log.rs:195), `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (governance_log.rs:197).
   11. **No migration in SL-b.** Plan §13 must produce zero migration files. `git diff governance-v0..phase-v1-SL-b -- migrations/` at plan-approval time must show no output. Watch §4 #11 in advisor-context.
   12. **DTO derive amendment — Copy must drop.** Existing `RevokeEndorsement` derives `Copy`; adding `reason: String` forces removing `Copy` (String isn't Copy). Plan §13 task that extends the DTO must explicitly note the derive-stack change. `Default` may also need to drop or accept `String::new()`-default with handler-side validation. Plan must specify which choice.
   13. **e2e Edit-per-task discipline.** 9 SL-b tests = 9 individual §13 tasks, each one anchor-pattern Edit at file end. Do NOT bundle. Per `feedback_junior_worker_e2e_edit_hang.md` — `e2e.rs` is now ~10,300+ lines; bundle Edits hang Junior workers.
- **§5 complexity score breakdown table mandatory** per `feedback_complexity_score_pre_split.md`. Pre-estimate 9-10. Likely above split-or-proceed threshold (8). Planner MUST file split-or-proceed `kind: "blocker"` DQ before finalising §13 if final tally exceeds 8. Suggested split: SL-b1 (handler/DTO/route + tests #1-6) / SL-b2 (severance tests #7-9). Or proceed without split with rationale (each task individually well-scoped).
- **e2e edit discipline** per `feedback_junior_worker_e2e_edit_hang.md`. ALSO per `feedback_e2e_filter_assumes_naming.md`: test-name-substring filtering against cargo test will silently match zero tests if SL-b's test names don't share a slug with any prior phase. Plan §15.7 (if it includes manual validation snippets) MUST run full e2e suite (no `--test e2e <filter>`), or confirm the filter via grep before recommending it.
- **Cargo feature/flag propagation** per `pattern_cargo_feature_flag_propagation.md`. Never combine `-p <crate>` with `--features full` (per `feedback_features_full_p_crate_incompatible.md`); use `--workspace --features full`. Even though §15 is Shape-G workflow-shape, the planner-side DoD smoke test (advisor side, pre-merge) and any §15.7 manual validation snippets respect this.
- **Build only what tests exercise** per PMD #14 / `feedback_build_what_tests_exercise.md`. SL-b is handler+tests; do NOT pre-implement fields/functions for SL-c/SL-d (the scheduler module, the compute/fire split). Drift-stub if a struct field is technically reachable but no SL-b test sets it.
- **Multi-write handlers transactionality** per `feedback_multi_write_handlers_need_transactions.md` — **load-bearing for SL-b**. The handler's writes (endorsement UPDATE + surety UPDATE + N case UPDATEs + 2 snapshot recomputes + 2 log entries) ALL run inside one `run_transaction` closure. Rate-limit count + capability checks happen BEFORE the transaction (read-only). Idempotency check (step 1: load + check `revoked_at IS NOT NULL`) happens INSIDE the transaction (TOCTOU avoidance per PRD §5.4).
- **R-rule inheritance from JM-a/b/c/d/e + AD-a + SL-a retros** — every R1-R7 from prior retros applies. R6 in particular (clippy `--no-deps -- -D warnings`) — under Shape G this lives in the workflow YAML. R5 (Task 0 enumerates ALL probes explicitly) is load-bearing for SL-b's pre-flight harness audit.
- **Wrapper-script flag silence** per `feedback_wrapper_script_flag_silence.md`. Under Shape G non-binding for §15 DoD; if §15.7 manual-validation snippets are included, they MUST cite Linux `.sh` wrappers and verify wrapper `$@` passthrough.
- **Pseudonym discipline** per ADR-015 + Watch 10 from `sponsor_liability.rs:11-15` doc-comment. Every governance-log payload field that names a person uses `*_pseudonym` (string), never raw `*_id`. The pseudonym is sourced via `actor_pseudonym_helper::get_or_create`. Plan §4 watchpoint cites this discipline; impl-task must follow.
- **Reason-redaction discipline** per ADR-015 + PRD §12.2. The `RevokeEndorsement.reason` field flows through `governance_log::append`'s scrub layer when emitted in the `endorsement_revoked` log payload. Plan §13 must cite the `scrub_json` invocation site (verify in `governance_log.rs` at planning time) and confirm the payload uses the scrubbed string, not the raw input.

### 4.2 Decision-queue discipline (per `.claude/rules/decision-queue.md`)

- **Attribution integrity.** Any DQ entry seeded by the planning subagent uses `answered_by: "planner"` (forward-looking pre-resolved entries) or `answered_by: null` (genuinely needs advisor input). NEVER `"advisor"`, `"user"`, `"impl-self-resolved"`. Subjects on planner DQ commits MUST start with `chore(decision-queue): planner raised DQ #<id> — <slug>`.
- **Mid-task DQ commits push immediately**, not at finalize, per `decision-queue.md` §"Mid-task visibility (Junior worktrees)". Push to `junior/sl-b-planning-1` (this worktree's branch); advisor's polling loop fetches all branches.
- **Boundary-of-judgment — when to STOP and queue rather than guess:**
  - **Complexity score > 8 split-or-proceed.** If the final §5 score exceeds 8 (per `feedback_complexity_score_pre_split.md`), planner MUST file `kind: "blocker"` DQ before finalising §13. Pre-estimate is 9-10. Options: split (SL-b1 + SL-b2) OR proceed-with-rationale (each task individually well-scoped, e2e Edit discipline keeps per-task risk low).
  - If `liability.multi_sponsor_escape_rule` default-const naming or location surprises (e.g. SL-a Task 5 named it differently than expected) → planner DQ surfacing the actual const name.
  - If the existing `RevokeEndorsement` DTO has been renamed OR the derive stack has been changed by an unexpected commit between brief-write and planning-time → planner DQ asking advisor to reconcile.
  - If at planning time the route at line 523 has shifted (e.g. another v1 sub-phase added a route immediately after `/endorsement` between SL-a merge and SL-b planning) → planner DQ surfacing the actual line number.
  - If the canonical mirror `create_endorsement.rs` has been mutated between brief-write and planning time (e.g. SL-a refactored it) → planner DQ surfacing the diff.
  - **DO NOT file a scope-hypothesis DQ.** SL-b's scope is unambiguous from PRD §5 + §15 row 2; this brief is explicit. If the planner finds itself wanting to bundle SL-c (scheduler) or SL-d (mutation) into SL-b, refuse the temptation and file a DQ asking the advisor — but the default is "stay narrow per PRD".
  - **DO NOT file a scope-creep DQ on the `liability_escape_reason` schema.** PRD §8.1 + OQ-V1-SL-05 lock the shape (`{"version": 1, "reason": ..., "actor_pseudonym": ..., "endorsement_id": ...}`). If the planner thinks an extra field is needed (e.g. `community_id`, `at_grace_remaining`), file a DQ — do NOT silently extend.

### 4.3 File ownership

- **Touch only:**
  - `.claude/PRPs/plans/v1-sponsor-liability-b.plan.md` (CREATE).
  - `.claude/decision-queue.json` (APPEND new pending or planner-resolved entries).
- **Do NOT edit anything under** `crates/`, `migrations/`, `tests/`, `docs/`, `.claude/PRPs/prds/`, `.claude/PRPs/reports/`, `.claude/agents/`, `.claude/rules/`, `.claude/commands/`, `.github/workflows/`, or any other plan file in `.claude/PRPs/plans/`.
- **One commit at finalize:** `feat(plan): v1-SL-b sub-phase plan` (matching SL-a/JM-a/JM-c/JM-d/JM-e + AD-a planner-task commit subjects).

### 4.4 Schema-first discipline

- **Read `crates/db_schema_file/src/schema.rs` BEFORE designing §13.** Confirm at planning time that:
  - `moderation_case` table has `grace_expires_at TIMESTAMPTZ NULL` AND `liability_escape_reason JSONB NULL` columns (SL-a Task 3 added).
  - `case_status` enum type lists 12 variants including the 3 SL-a variants (`SponsorLiabilityPending`, `SponsorLiabilityFired`, `SponsorLiabilityEscaped`).
  - `endorsement` table has `revoked_at TIMESTAMPTZ NULL` (Phase 5a baseline).
  - `surety` table has `revoked_at TIMESTAMPTZ NULL` (Phase 5a baseline) + `surety_sponsored_id_active` partial index (SL-a Task 1 — Issue #24).
- **Read `crates/db_schema_file/src/enums.rs:411-432`** to confirm the Rust enum has 12 variants. If any of these baseline assumptions fails, file a `kind: "blocker"` DQ before writing §13.

### 4.5 Cross-cutting from PMD-promoted patterns

- **Pattern: `multi_write_handlers_need_transactions`** — applies. Handler runs in one `run_transaction`; idempotency check inside; rate-limit + capability checks outside.
- **Pattern: `verify_before_trusting_shell_output`** (PMD #pattern_verify_before_trusting_shell_output) — when the planner runs git-grep to enumerate match sites or test setups, verify the count via direct file read; don't trust pipe-counts.
- **Pattern: `cargo_feature_flag_propagation`** (PMD #pattern_cargo_feature_flag_propagation) — applies to any `--features full` invocation in §15.7 manual snippets. Never combine `-p <crate>` with `--features full`.
- **Pattern: dual-file ENTRY_KIND edit** (per v1-AD-a §10.8 + v1-JM-a §15) — does NOT apply to SL-b. All consts shipped in SL-a; SL-b reads them via the existing shim re-export.
- **Pattern: `read_canonical`** — the canonical-schema-first gate. SL-b's plan §13 cites `create_endorsement.rs:1-361` as the canonical mirror for the handler shape; deviation requires DQ.

### 4.6 Attribution integrity reminder

The only valid `answered_by` labels for a Junior planning subagent are `"planner"` or `null`. Never `"advisor"`, `"user"`, `"impl-self-resolved"`. Any commit with `answered_by: "advisor"` from this subagent triggers the catch-fire procedure in `advisor-orchestrator.md`.

---

**Lean / advisor-side tip (not a constraint):** SL-b is a "single new handler + DTO extension + 9 tests" sub-phase — the cleanest possible shape after a schema-foundation phase like SL-a. The canonical mirror `create_endorsement.rs` exists in tree (361 lines, mature, exhaustively reviewed across Phase 5a-5c CR cycles); SL-b's handler is a reflection of it with three deltas: (1) UPDATE-then-cascade instead of INSERT; (2) idempotency check at step 1 (TOCTOU avoidance via FOR UPDATE inside transaction); (3) the grace-window-severance loop at step 6. Nothing in SL-b is novel architecturally; the work is mechanical pattern-following with strict invariant enforcement (pseudonym discipline, version-tagged JSON, transactional ordering).

A second observation: the 9-test surface is the dominant complexity contributor to the score. Each test is anchor-Edit-friendly (sibling async fn at file end, no helper-extraction needed because test setup helpers exist from Phase 5a-5c endorsement work — the plan should grep for `create_endorsement` test calls in `e2e.rs` to find them). Per-task wall-clock under Sonnet 4.6 is likely 3-8 minutes per test (Edit + cargo check on test target). 9 tests × 6 min average = ~54 minutes of pure impl-task time, plus per-task validate-pending workflows. Cohort dispatch via `[P]` is plausible if the planner can prove via FILES YAML overlap check that the 9 tests touch only `e2e.rs` and don't share helper-function changes — but per `feedback_parallel_agents_one_worktree_per_agent` the daemon already enforces one-worktree-per-task; what `[P]` controls is *advisor dispatch order*, not parallel filesystem access.

A third observation: PRD §5.4 idempotency is the most subtle invariant — re-revocation must return vec![] AND emit no log AND not recompute snapshots, even though the handler's normal flow does all three. The clean implementation is: step 1 returns `EarlyReturn(existing_revoked_at)` when `revoked_at.is_some()`; the outer handler checks for this branch and constructs the response without invoking steps 2-8. Test #3 verifies governance_log row count is unchanged across two revoke calls — strong assertion that catches "early-return forgot to skip the log emit" bugs. Plan §13 task that implements step 1 must be paired with test #3; both fail together if either is wrong.

A fourth observation: the rate-limit query (§2.1 deliverable f) is the only handler write that doesn't go inside `run_transaction`. It's a pure read (count) and rejecting on rate-limit doesn't depend on transactional consistency — the worst case is a sponsor revokes 6 endorsements within 24h because the 6th call read a stale count, and that's acceptable (the limit is a courtesy, not a security boundary). Admin caller bypasses entirely. Plan §13 must order: rate-limit check (read) → reason validation (no IO) → open transaction → step 1-8. Reversing this puts the count inside the transaction, which is correct but wastes connection time on the rejecting case.

A fifth observation: SL-b unblocks SL-c and SL-d in sequence (per PRD §15 row 2 → row 3 → row 4). SL-c (scheduler) reads the same `SponsorLiabilityPending` cases SL-b's tests pre-seed; SL-d (`submit_jury_vote` mutation) is what creates them in production. SL-b is the gate for both, but doesn't itself compete with them for shared state — SL-b's tests insert via `ModerationCaseInsertForm` direct-write (synthetic), so they don't depend on SL-d; SL-c's scheduler tick fires on the same column SL-b reads (`grace_expires_at`), but SL-b doesn't write that column (only SL-d writes `grace_expires_at`; SL-c reads it; SL-b reads `liability_escape_reason` and writes a new value). The lane sequence is the cleanest possible after SL-a's foundation.

---

_Brief author: advisor session (laptop CWD `C:\Users\barri\Developer\brehon-fork` on `governance-v0` @ `e00819e2d`, 2026-05-04). Brief committed on `governance-v0` before Junior planning task is queued. Per the advisor-orchestrator clarify gate, advisor runs `/brehon-clarify .claude/PRPs/briefs/sl-b-planning-1.md` after this brief commits and pushes; planning task only queues after every clarify-DQ entry is resolved. Lane progression confirmed 2026-05-04: SL-b scope is `revoke_endorsement` handler + DTO + route + tests per PRD §5 + §15 row 2 (NOT "compute branch primitives" — that's SL-d). The scope-correction is recorded in `homeserver/.claude/advisor-context-v1-SL-b.md` §"Scope-correction note"._
