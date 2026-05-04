# Plan: v1-sponsor-liability-b — `revoke_endorsement` handler + DTO + route + 9 integration tests

> **Shape G plan** — SL-b ships under Shape G (Layer G2 push-and-exit). §15
> references workflow YAMLs by path + expected `conclusion`, not inline cargo.
> Cargo runs on GitHub-hosted runners (workspace-check on `junior/*`) and on
> the laptop / `workflow_dispatch` GH runner (e2e on `phase-v1-SL-b`
> post-finalize-merge). See `.claude/PRPs/templates/plan.template.md` §15.6
> + `.claude/PRPs/plans/v1-validate-agent.plan.md`.

## Table of contents

| § | Heading |
|---|---|
| 1 | Summary |
| 2 | Source |
| 3 | Problem statement |
| 4 | Solution statement |
| 5 | Metadata + complexity score |
| 6 | Relationship to other v1-SL sub-phases |
| 7 | Preflight guardrails inherited from prior phases |
| 8 | Flow design |
| 9 | Mandatory reading |
| 10 | Patterns to mirror |
| 11 | Files to change |
| 12 | NOT building in v1-SL-b |
| 13 | Step-by-step tasks |
| 14 | Testing strategy |
| 15 | Validation commands (DoD — Shape G) |
| 16 | Acceptance criteria |
| 16a | Stories (independently-testable behaviour units) |
| 17 | Completion checklist |
| 18 | Risks and mitigations |
| 19 | Notes |
| 20 | Confidence score |

---

## 1. Summary

v1-SL-b ships the v0-deferred `POST /api/v4/governance/endorsement/revoke`
endpoint per PRD §5 + §15 row 2. It extends the existing `RevokeEndorsement`
DTO with a required `reason: String` field, adds a new
`RevokeEndorsementResponse` DTO, creates the `revoke_endorsement.rs` handler
in `crates/api/api_crud/src/governance/`, registers the new route in
`crates/api/routes/src/lib.rs`, and ships **9 integration tests** that
behaviourally cover PRD §5 (self-revoke, admin-revoke, idempotency,
capability rejection, reason validation, rate-limit + admin bypass) and PRD
§5.3 step 4 grace-window severance (single-sponsor escape, multi-sponsor
`any_revocation` rule, non-severance no-pending-case path). The handler
fires the `endorsement_revoked` log entry **always**, the
`sponsor_liability_escaped` log entry on actual severance, and recomputes
both sponsor + sponsee snapshots inside the same `run_transaction` closure.
SL-b does NOT add new schema, migrations, ENTRY_KIND consts, CaseStatus
variants, or governance_config seeds — all five required ENTRY_KIND consts
(`endorsement_revoked` 195, `sponsor_liability_escaped` 197), the three
SponsorLiability* CaseStatus variants, the `grace_expires_at` +
`liability_escape_reason` columns on `moderation_case`, and the 13
`liability.*` + `job.grace_check_*` config seeds shipped in SL-a; SL-b
reads them.

**Why now.** SL-a shipped 2026-05-04 via PR #111 squash-merge at
`governance-v0` HEAD `790f6101d`; the schema foundation is in place. SL-b
is the first lane sub-phase to author a write handler against it. Until
SL-b lands, sponsors of cases that transition to `SponsorLiabilityPending`
(SL-d) have no API surface to escape liability — the carrot-for-de-escalation
half of the Brehon `athgabál` reading is unimplemented.

**Headline acceptance condition.** All three §16a stories are `[done]`
with their checkpoint workflows green: (1) DTO extension + handler module
+ route registration compile clean and the route resolves
`/api/v4/governance/endorsement/revoke` to the new handler; (2) the six
PRD §5/§12 contract guards (capability + reason + rate-limit + admin
bypass + idempotency) are enforced and tested; (3) the three grace-window
severance branches (single-sponsor escape, multi-sponsor `any_revocation`
escape, non-severance no-pending-case) drive
`SponsorLiabilityPending → SponsorLiabilityEscaped` transitions correctly
with `liability_escape_reason` JSONB matching PRD §8.1 schema (`version: 1`,
`actor_pseudonym` not raw `caller_id`).

This sub-phase touches NO schema, NO migration, NO ENTRY_KIND const, NO
CaseStatus variant, NO governance_config seed. It is **DTO + handler +
route + 9 integration tests + retro**.

---

## 2. Source

- `.claude/PRPs/briefs/sl-b-planning-1.md` @ `governance-v0` `080fcd7b7` —
  the advisor brief (post-`/brehon-clarify`; DQ #138-#142 resolved
  2026-05-04).
- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` @ `governance-v0` —
  parent PRD; §1 (vision; Brehon athgabál framing), §2 (IN/OUT — confirms
  SL-b is handler/DTO/route/tests only), §3.1 + §3.4 (CaseStatus extensions
  — confirms SL-a shipped the 3 variants), **§5 (`POST /api/v4/governance/endorsement/revoke`
  — IS the SL-b spec)** §5.1-§5.6, §7.3 (restoration interaction —
  out-of-scope for SL-b), **§8.1 (`liability_escape_reason` JSONB schema —
  load-bearing)**, §10 (Defaults Matrix — confirms `liability.revoke_rate_limit_per_day`
  default 5, `liability.multi_sponsor_escape_rule` default `"any_revocation"`),
  §11 (backwards-compat — §11.2 mid-flight cases), **§12 (security —
  §12.1 rate-limit, §12.2 reason-required + scrub, §12.3 step-up
  reservation = NO ENFORCEMENT in SL-b)**, §15 (implementation phases —
  confirms SL-b is row 2), §17 (cross-cutting impact — names ADR-013
  enum-exhaustiveness; SL-b adds zero variants), §18 (B4 key-rename
  table for `liability.*` is authoritative).
- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — predecessor; §10
  patterns + §13 task ordering + §15 Shape-G DoD shape (SL-b mirrors).
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — most recent
  post-Shape-G plan + post-spec-kit-adoption. SL-b adopts §15.6 Shape-G
  shape, §16a Stories grain, §13 FILES YAML blocks, §5.1 complexity
  breakdown table.
- `.claude/PRPs/templates/plan.template.md` — canonical 20-section schema;
  §15.6 Shape G DoD; §16a Stories mandatory.
- `.claude/agents/planning.md` — subagent contract; §13 per-task FILES
  YAML block discipline; §5 complexity score rule.
- `.claude/rules/decision-queue.md` — schema-v2 attribution (planner →
  `from: "planner"`, never `"advisor"`); Recipe 1 (raise blocker) +
  Recipe 2 (planner-resolved pre-seed); `kind: "validate-pending"` Shape
  G routing.
- `.claude/rules/advisor-orchestrator.md` — Stage shape "Each impl-task
  complete (under Shape G)" + Phase 2 e2e user gate (local-vs-dispatch
  per PR #105).
- `.claude/rules/governance-log-entry-kind-registry.md` — confirms
  `ENTRY_KIND_ENDORSEMENT_REVOKED` (line 195) and
  `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (line 197) shipped in SL-a;
  SL-b is a fire-site for both. Registry count `38` stays unchanged.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` —
  ADR-010 (won't-disadvantage rule — informs SL-b's idempotency +
  re-revoke semantics), ADR-013 (CaseStatus enum-exhaustiveness — SL-b's
  match on `case.status` MUST enumerate all 12 variants; no `_ =>`
  arms), ADR-014 (federation deferral — SL-b emits log events on local
  instance only; no outbox), **ADR-015 (pseudonymisation — load-bearing
  for SL-b; every `liability_escape_reason` JSONB field that names a
  person uses pseudonym, not raw id)**, OQ-V1-SL-05
  (`liability_escape_reason` schema versioning — `version: 1` from day
  one), OQ-V1-SL-01 (multi-sponsor escape rule — `any_revocation`
  default; community-configurable).

### Lessons that bind §13 decisions

- `feedback_multi_write_handlers_need_transactions.md` —
  **load-bearing for SL-b**. The handler wraps endorsement UPDATE +
  surety UPDATE + N case UPDATEs + 2 snapshot recomputes + 2 log
  entries inside one `run_transaction`. Rate-limit count + capability
  checks happen BEFORE the transaction (read-only). Idempotency check
  inside the transaction (TOCTOU avoidance per PRD §5.4).
- `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint
  cites a concrete file/handler/`schema.rs` line. Binds §4 entries.
- `feedback_complexity_score_pre_split.md` — score 38 → planner files
  split-or-proceed DQ #143 before commit. Binds §5.2.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task carries
  a FILES YAML block; cohort dispatch reads `union(creates, modifies)`.
- `feedback_parallel_cohort_dispatch.md` — Tasks 4-12 all
  `modifies: crates/server/tests/e2e.rs`; YAML overlap rule refuses
  cohort dispatch; tasks ship serially (no `[P]` markers in §13).
- `feedback_pre_phase_dod_smoke_test.md` +
  `feedback_plan_dod_dry_run_at_write.md` — advisor-side DoD smoke-test
  runs every §15 command literally before plan approval. Under Shape G,
  the §15 entries name workflow YAML paths + `gh run list` queries —
  both verifiable mid-plan-approval.
- `feedback_features_full_p_crate_incompatible.md` — never `-p <crate>`
  + `--features full`. Workflow YAMLs already comply (`cargo check
  --workspace --features full` per `cargo-validate-workspace.yml:88`);
  §15.7 manual snippets respect the rule.
- `feedback_features_full_workspace_only.md` — `--features full`
  required to activate `DbEnum` + `ts-rs` derives. Encoded in workflow
  YAML.
- `feedback_insertform_default_propagation.md` — citation-only: SL-b
  does not add new InsertForm fields. Existing
  `ModerationCaseInsertForm` already carries `grace_expires_at` +
  `liability_escape_reason` from SL-a Task 4 (verified at brief-write
  time, lines 135-136 of `moderation_case.rs`).
- `feedback_clippy_test_style.md` — R1 every `i32 ↔ i64` uses
  `i64::from(...)`. Bound in §4 watchpoint #14 (rate-limit count).
- `feedback_junior_worker_e2e_edit_hang.md` — **load-bearing for SL-b**.
  9 tests = 9 individual §13 tasks, each one anchor-pattern Edit at
  file end. e2e.rs is now 10976 lines; bundle Edits hang Junior workers.
- `feedback_e2e_filter_assumes_naming.md` — §15.7 manual validation
  snippets, if included, run full e2e suite without
  `--test e2e <filter>`, OR confirm via grep first.
- `feedback_brehon_verify_pre_merge.md` — §16a Stories grain enables
  `/brehon-verify` phantom check before `bm-merge`.
- `feedback_principles_not_rules.md` — score 38 is a signal; planner
  observation in §5.2 leans proceed-as-one with rationale (each test is
  anchor-Edit-friendly; per-task wall-clock ~6-8 min under Sonnet 4.6;
  SL-a + JM-e proceed-as-one precedents).
- `feedback_read_canonical_before_writing_spec.md` — citation-only
  (SL-b adds no new spec/template/rule); plan §10 cites
  `create_endorsement.rs:1-361` as canonical mirror per the spirit.
- `feedback_schema_changing_spec_retrofit_question.md` — citation-only
  (SL-b changes no spec/template shape).
- `feedback_pr_per_phase.md` — one commit per task; §13 ordering matters.
- `feedback_handover_trailer_cohort_propagation.md` — non-binding under
  serial dispatch (no `[P]` cohorts in §13); per-task `HANDOVER:` commit
  trailer still recommended as mechanical discipline.
- `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`
  + `feedback_retro_task_complexity_score.md` — retro shape (Task 13).

### Related prior plans (canonical-shape mirror)

- `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` — immediate
  predecessor; ships the schema foundation SL-b reads. SL-b §11 file
  inventory does not duplicate SL-a entries; SL-b mirrors SL-a's §13
  ordering + §15 DoD shape.
- `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` — most recent
  Shape-G plan with anchor-Edit-per-test discipline. SL-b §13 e2e tasks
  mirror JM-e Tasks 2-5 shape (each test is one anchor-Edit at file
  end; serial dispatch; YAML overlap rule binds).
- `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md`
  — context-only; the v0 sponsor-liability foundation. SL-b does NOT
  touch this code.
- `.claude/PRPs/briefs/jm-e-planning-1.md` + `sl-a-planning-1.md` —
  exemplar planning briefs for shape and constraint language.

---

## 3. Problem statement

Post-SL-a-merge on `governance-v0` @ `790f6101d`:

- **The `/api/v4/governance/endorsement/revoke` endpoint does not exist.**
  Sponsors who created an endorsement via `POST /api/v4/governance/endorsement`
  (Phase 5b) can never revoke it via API. The `endorsement.revoked_at`
  column exists (Phase 5a) and `surety.revoked_at` exists (Phase 5a),
  but no handler writes either field.
- **The carrot-for-de-escalation half of `athgabál` is unimplemented.**
  Per PRD §1.1, sponsor revocation during a grace window is the
  procedural mechanism by which sponsors escape liability. SL-d will
  ship the `Decided → SponsorLiabilityPending` transition; without
  SL-b, sponsors of mid-grace cases have no API to revoke and trigger
  the `SponsorLiabilityEscaped` transition.
- **The `SponsorLiabilityEscaped` terminal state has no fire site for
  the revocation branch.** PRD §5.3 step 4 specifies the grace-window
  severance loop runs inside `revoke_endorsement`'s `run_transaction`.
  Until SL-b ships, this code path is uncovered; the `(pending)` marker
  on `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (registry §"v1-SL-a entry
  kinds") is unflipped for the revocation branch.
- **The `endorsement_revoked` audit-log entry has no fire site.** PRD
  §5.3 step 5 specifies the entry fires always (even when no
  severance). The const exists (`governance_log.rs:195`) but no
  emitter.
- **The §12.1 revocation rate-limit is unenforced.** PRD §12.1
  specifies `count(*) FROM endorsement WHERE from_person_id = caller_id
  AND revoked_at > now() - INTERVAL '24 hours'` against
  `liability.revoke_rate_limit_per_day` (default 5; SL-a-seeded). With
  no handler, the limit can never apply.
- **The §12.2 reason-required + scrub discipline is unenforced.** PRD
  §12.2 specifies `RevokeEndorsement.reason` is required and flows
  through `governance_log::append`'s scrub layer. The current DTO at
  `governance.rs:311` has `endorsement_id` only; no `reason` field.
- **Existing `RevokeEndorsement` DTO has shape mismatch with PRD §5.1.**
  The DTO at `governance.rs:308-313` derives `Copy + Default`. PRD §5.1
  specifies `reason: String` — String isn't `Copy`, so Copy must drop;
  Default may need to drop or accept empty-string default that the
  handler rejects.
- **The `liability.multi_sponsor_escape_rule` config knob has no
  reader.** SL-a Task 5 seeded the key (default `"any_revocation"`,
  per-community + instance scoped) and shipped
  `DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE` const at
  `config.rs:937`. No handler reads it; SL-b is the first reader.

The substrate is in place; only the handler edits + DTO extension +
route registration + integration tests are missing.

---

## 4. Solution statement

Twelve surgical changes, organised as 12 impl tasks + Task 0 pre-flight
+ retro.

### 4.1 Architecturally load-bearing decisions locked in this sub-phase

- **The handler is a sibling of `create_endorsement.rs`, not a rewrite
  or a new module.** `crates/api/api_crud/src/governance/revoke_endorsement.rs`
  is created; mirrors `create_endorsement.rs:1-361` shape (module
  doc-comment → use block → outer handler → `run_transaction` →
  `process_revocation` body helper). Module wired via
  `crates/api/api_crud/src/governance/mod.rs` `pub mod
  revoke_endorsement;` (alphabetically after `create_report;`, before
  `request_appeal;` per existing precedent).
- **The DTO extension drops `Copy` and keeps `Default`.** Existing
  `RevokeEndorsement` at `governance.rs:307-313` derives
  `#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default,
  PartialEq, Eq, Hash)]`. Adding `reason: String` forces removing
  `Copy` (String isn't `Copy`). `Default` is preserved — `String`
  defaults to `""` (empty), and the handler rejects empty-after-trim
  per §12.2 + DQ #139 resolution.
- **The new `RevokeEndorsementResponse` mirrors `CreateEndorsementResponse`
  shape.** Added in same `governance.rs` file, same derive stack as
  `CreateEndorsementResponse` at `governance.rs:297-305`. Fields per
  PRD §5.1: `endorsement_id: EndorsementId`, `revoked_at: DateTime<Utc>`,
  `liability_chain_severed_for_cases: Vec<ModerationCaseId>`. The Vec
  field forces removing `Copy` from this DTO too.
- **Handler entry order — rate-limit BEFORE transaction.** Per PRD
  §5.3 + create_endorsement cooldown precedent (`create_endorsement.rs:198-214`):
  (1) `check_local_user_valid`, (2) `is_admin` check (sets
  `is_admin_caller` flag), (3) IF NOT admin: rate-limit count + reject
  if exceeded, (4) reason validation + trim, (5) THEN open
  `run_transaction` → `process_revocation`. Reversing order puts the
  count inside the transaction (correct but wastes connection time on
  the rejecting case).
- **Inside-transaction step ordering (PRD §5.3 — load-bearing for §13
  task split).** `process_revocation` body:
  1. Load endorsement with `FOR UPDATE`; verify
     `endorsement.revoked_at IS NULL`. If already revoked, EARLY-RETURN
     existing `revoked_at` with `liability_chain_severed_for_cases:
     vec![]` (idempotency per PRD §5.4 — read-inside-transaction to
     avoid TOCTOU).
  2. Verify capability — `endorsement.from_person_id == caller_id`
     OR `is_admin_caller` (set pre-tx). Else `Err(NotFound)`.
  3. UPDATE `endorsement SET revoked_at = now()` via diesel `update()`.
  4. UPDATE the matching `surety` row
     (`from_person_id = caller_id, to_person_id = endorsement.to_person_id,
     community_id = endorsement.community_id`); UPDATE no-op if not
     found (community-scoped vs unscoped endorsement; per the conditional
     surety insert in `create_endorsement.rs:227-248`).
  5. Re-query active sponsors for `to_person_id` (those with
     `surety.revoked_at IS NULL` AFTER step 4's UPDATE).
  6. Grace-window evaluation loop — query `moderation_case` rows with
     `status = 'SponsorLiabilityPending' AND target_person_id =
     endorsement.to_person_id AND grace_expires_at > now()`. For each
     matched case, read community-scoped escape rule via
     `config::get_text(cache, conn, Scope::Community(case.community_id),
     "liability.multi_sponsor_escape_rule")` (per DQ #142 — per-case,
     not pre-loop). Apply the rule:
     - `any_revocation` (default): caller's revocation severs the
       chain immediately.
     - `all_revocation`: severance only if step-5 active sponsors == 0.
     - `majority_revocation`: severance if revoked-since-decision >50%
       of pre-revoke sponsors.
     - Unknown / NULL falls through to `any_revocation` (defensive
       default; do NOT error).
     If escape met: UPDATE case `SET status = 'SponsorLiabilityEscaped',
     liability_escape_reason = json!({"version": 1, "reason":
     "sponsor_revoked", "actor_pseudonym": caller_pseudonym,
     "endorsement_id": endorsement.id.0})`; emit
     `sponsor_liability_escaped` log entry; append `case.id` to
     outcome's `liability_chain_severed_for_cases`.
  7. Recompute snapshots — `reputation_snapshot::recompute_snapshot`
     for sponsor (`caller_id`) AND sponsee
     (`endorsement.to_person_id`), mirroring
     `create_endorsement.rs:307-309`. Both calls inside the same
     transaction.
  8. Emit `endorsement_revoked` log entry ALWAYS (even if no
     severance). Payload: `{"sponsor_pseudonym": caller_pseudonym,
     "target_pseudonym": target_pseudonym, "community_id":
     endorsement.community_id.map(|c| c.0), "reason": data.reason,
     "liability_chain_severed_for_cases":
     [<case_id.0> ...]}`. Per DQ #140: when (admin AND
     would-have-been-rate-limited), additionally include
     `"rate_limit_bypassed": true` in the payload. The `reason` field
     flows through `governance_log::append`'s `scrub_json` layer per
     ADR-015.
- **`actor_pseudonym` discipline (ADR-015) is mandatory.** Every
  `liability_escape_reason` JSONB field naming a person uses
  pseudonym, not raw `caller_id`. Pseudonym sourced via
  `actor_pseudonym_helper::get_or_create(&mut context.pool(),
  caller_id)` BEFORE opening the transaction (mirrors
  `create_endorsement.rs:126-127`). Inside the loop, also fetch
  `target_pseudonym` for the `endorsement_revoked` payload (mirrors
  `create_endorsement.rs:314-315`).
- **The route registration sits at line 524, NOT 518 as PRD §5.6
  cites.** The current `/endorsement` route is at `lib.rs:523`
  (verified at brief-write time + plan-write time). SL-b adds
  `.route("/endorsement/revoke", post().to(revoke_endorsement))`
  immediately after, AND adds the import at `lib.rs:149` (after
  `create_endorsement::create_endorsement,` per the
  `governance::{...}` use block precedent). PRD §5.6 cites a stale
  line number from when v1-AD-* phases hadn't yet shipped routes that
  pushed `/endorsement` down. Plan §13 Task 3 includes a `grep -n
  '"/endorsement"' crates/api/routes/src/lib.rs` to find the current
  line at task-start time and anchor-insert immediately after the
  matched line.
- **The 9 e2e tests live in a NEW `mod v1_sl_b_fixtures` block in
  `crates/server/tests/e2e.rs`, after `mod v1_jm_e_fixtures`** (the
  current last fixture mod, ending around line 10975). Each test is
  its OWN §13 task per `feedback_junior_worker_e2e_edit_hang.md` —
  e2e.rs is now 10976 lines; multi-test bulk Edits hang the worker.
  Per-test anchor pattern: each task Edits at file end, appending its
  test inside the same `mod v1_sl_b_fixtures` block.
- **Test #1 (Task 4) creates the `mod v1_sl_b_fixtures` shell + the
  helpers, AS WELL AS the first test.** Tasks 5-12 anchor-insert
  AFTER the prior task's test inside the same mod. This means Tasks
  4-12 all `modifies: crates/server/tests/e2e.rs`; YAML overlap rule
  refuses cohort dispatch — they ship serially. No `[P]` markers in
  §13.
- **Tests pre-seed `SponsorLiabilityPending` cases via direct DB-write
  (`ModerationCaseInsertForm`)**, NOT via SL-d's mutation handler
  (which doesn't exist yet). The pre-seed sets
  `status: SponsorLiabilityPending`, `grace_expires_at: now() + 24h`,
  `target_person_id: sponsee_id`, plus the SL-a-mandatory fields per
  the existing `ModerationCaseInsertForm` shape.
- **`ConfigCache` is threaded through the tx closure**, mirroring
  `create_endorsement.rs:156` and the `enforce_age_gate` pattern at
  `create_endorsement.rs:339-361`. Per DQ #142: per-case lookup
  inside the grace-window loop deduplicates duplicate-community
  reads automatically (verified at `config.rs:209-256`).
- **Shape G — DoD references workflow YAMLs by path + expected
  `conclusion`, not inline cargo.** Per
  `.claude/PRPs/templates/plan.template.md` §15.6: every §13 task's
  DoD references `cargo-validate-workspace.yml` on the worker branch
  + workflow_run_id captured by the impl-task subagent post-push.
- **No migration round-trip workflow fires.** SL-b touches no
  `migrations/**` paths; `cargo-validate-migration.yml`'s path filter
  excludes it. Only `cargo-validate-workspace.yml` (per task) +
  `cargo-test-e2e.yml` (per Phase 2 e2e dispatch on
  `phase-v1-SL-b`).

### 4.2 Watchpoints (specific files / handlers / `enums.rs` lines)

Per `feedback_advisor_watchpoint_specificity.md` — every entry cites a
specific file/line/handler. The 14 watchpoints:

1. **TOCTOU on re-revoke.** `crates/api/api_crud/src/governance/revoke_endorsement.rs`
   step 1: load endorsement + check `revoked_at IS NOT NULL` →
   early-return — all inside `run_transaction`. Separate-read-then-tx-write
   is a TOCTOU bug. Plan §13 Task 2 commits the load+check inside the
   tx body; advisor plan-approval gate verifies via Story 2 Brief-Scope
   structural pattern.
2. **Step ordering — sibling surety revoke BEFORE re-query active
   sponsors.** Plan §13 Task 2 inside-transaction body must order:
   revoke endorsement → revoke matching surety → THEN re-query active
   sponsors → evaluate escape rule. Reversing yields zero or stale
   severance results; the test #8 (Task 11) asserts only the revoking
   sponsor's surety row has `revoked_at` set.
3. **Escape condition is community-configured + fall-back pattern.**
   Plan §13 Task 2 cites
   `crates/api/api/src/governance/config.rs::get_text` (cascade
   `Community → Instance → const`) and
   `DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE` (`config.rs:937`).
   Hard-coding the escape rule is a regression. Per DQ #142: per-case
   read, not once-before-loop.
4. **`liability_escape_reason` JSON schema locked.** Schema:
   `{"version": 1, "reason": "sponsor_revoked", "actor_pseudonym":
   "...", "endorsement_id": ...}`. `actor_pseudonym` mandatory per
   ADR-015; `version: 1` mandatory per OQ-V1-SL-05. Plan §13 Task 2
   cites `actor_pseudonym_helper::get_or_create` from
   `create_endorsement.rs:126-127` as the source. Raw `caller_id` in
   reason JSON is a GDPR-013 violation, catch-fire.
5. **Response `liability_chain_severed_for_cases` populated only on
   actual severance.** Re-revocation returns `vec![]`. Test #3 (Task
   6, re-revoke idempotency) asserts empty vec + NO new log entries +
   NO snapshot recompute. Without this assertion, the idempotency
   claim is unverified.
6. **Two recompute-snapshot calls inside the same transaction.** Plan
   §13 Task 2 cites `create_endorsement.rs:307-309` and calls
   `recompute_snapshot` for BOTH sponsor (`caller_id`) and sponsee
   (`endorsement.to_person_id`). Missing the sponsee recompute leaves
   stale `can_sponsor` for downstream gates.
7. **No new `moderation_case.status` mutations OUTSIDE the
   grace-severance step.** SL-b touches case status ONLY for
   `SponsorLiabilityPending → SponsorLiabilityEscaped` transitions.
   MUST NOT mutate `Decided` cases (PRD §11 backwards-compat); MUST
   NOT create new `SponsorLiabilityPending` cases (SL-d's job). The
   orphan-case ModerationCaseInsertForm pattern at `e2e.rs` line ~9989
   (creator_id=NULL + winning_decision=NoAction) is the cautionary
   anchor — never mutate orphan or decided cases. Plan §13 Tasks 10-12
   pre-seed `SponsorLiabilityPending` cases ONLY (not `Decided`,
   `Closed`, etc.).
8. **PRD §11.2 mid-flight cases at v1 deploy.** SL-a Task 1 backfill
   creates `SponsorLiabilityPending` cases at v0→v1 transition. Plan
   §13 Task 10 (test #7) seeds a `SponsorLiabilityPending` case via
   `ModerationCaseInsertForm` (synthetic equivalent of the backfill
   shape) and exercises revocation severance during the 24h grace.
9. **Route registration site is line 524, NOT 518.** PRD §5.6 cites a
   stale line. Plan §13 Task 3 includes `grep -n '"/endorsement"'
   crates/api/routes/src/lib.rs` to find current line, then
   anchor-insert immediately after. At plan-write time the
   `.route("/endorsement", post().to(create_endorsement))` is at
   `lib.rs:523`; SL-b's route registers at the next line.
10. **No new ENTRY_KIND_*** consts.** All 5 already shipped in SL-a.
    Plan §13 must NOT add to the registry. Use existing const names:
    `ENTRY_KIND_ENDORSEMENT_REVOKED` (`governance_log.rs:195`),
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` (`governance_log.rs:197`).
    Verify at retro time: `rg -c '^pub const ENTRY_KIND_'
    crates/db_schema/src/source/governance/governance_log.rs` returns
    `38` (unchanged).
11. **No migration in SL-b.** Plan §13 produces zero migration files.
    `git diff governance-v0..phase-v1-SL-b -- migrations/` at
    plan-approval time must show no output.
    `cargo-validate-migration.yml`'s path filter excludes SL-b tasks.
12. **DTO derive amendment — Copy must drop.** Existing
    `RevokeEndorsement` derives `Copy`; adding `reason: String` forces
    removing `Copy` (String isn't Copy). Plan §13 Task 1 explicitly
    notes the derive-stack change. `Default` is preserved — `String`
    defaults to `""`; handler rejects empty-after-trim per §12.2 + DQ
    #139.
13. **e2e Edit-per-task discipline.** 9 SL-b tests = 9 individual §13
    tasks (Tasks 4-12), each one anchor-pattern Edit at file end. Do
    NOT bundle. Per `feedback_junior_worker_e2e_edit_hang.md` —
    `e2e.rs` is 10976 lines; bundle Edits hang Junior workers.
14. **Rate-limit count `i64::from(...)` discipline.** Per R1 +
    `feedback_clippy_test_style.md`, the rate-limit count
    (`i64`-typed Diesel `count_star()` result) is compared against the
    `liability.revoke_rate_limit_per_day` config value (`i64` per
    `get_int`). Direct comparison is OK; if the handler accidentally
    introduces an `i32` intermediate, bridge with `i64::from(...)`.

### 4.3 Rejected alternatives

- **Bundle the 9 e2e tests into 1-3 §13 tasks.** Rejected per
  `feedback_junior_worker_e2e_edit_hang.md` — e2e.rs is 10976 lines;
  multi-test bulk Edits hang Junior workers. JM-e shipped 4 tests as
  4 tasks for the same reason.
- **Mark Tasks 4-12 as `[P]`.** Rejected: YAML overlap rule refuses
  cohort because all 9 tasks `modifies: crates/server/tests/e2e.rs`.
  Each task anchor-Edits at the prior task's commit tip; cohort
  dispatch would race.
- **Mark Task 2 + Task 3 as `[P]`.** Rejected: Task 3 (route
  registration in `lib.rs`) imports the handler symbol from Task 2's
  new `revoke_endorsement.rs`. If Tasks 2+3 are dispatched
  simultaneously, Task 3's worker branch is cut from `phase-v1-SL-b`
  before Task 2's commit lands; Task 3's compile fails with
  "unresolved import revoke_endorsement::revoke_endorsement". Serial
  dispatch is correct.
- **Consolidate Task 1 (DTO) + Task 2 (handler) + Task 3 (route) into
  one task.** Rejected per `feedback_pr_per_phase.md` one-commit-per-task
  discipline + per-task FILES YAML block + per-task workspace-check
  workflow run. Three logically distinct concerns get three commits.
- **Add a separate `endorsement_revoked_admin_bypass` ENTRY_KIND
  const for admin-bypass-rate-limit cases.** Rejected per DQ #140:
  reuse the existing `endorsement_revoked` entry with a
  `rate_limit_bypassed: true` field set ONLY when (admin AND
  would-have-been-rate-limited). Preserves §2.3 scope (no new const).
- **Add a `step_up_token: Option<String>` slot on
  `RevokeEndorsement` per PRD §12.3.** Rejected for SL-b — PRD §12.3
  reserves step-up auth for v2 (Keycloak/MFA substrate). v1 ships
  without the slot; v2 grafts onto the DTO without a wire migration
  per the additive-field-with-`#[serde(default,
  skip_serializing_if = "Option::is_none")]` pattern.
- **Read `liability.multi_sponsor_escape_rule` once before the
  grace-window loop.** Rejected per DQ #142: per-case read inside the
  loop. Cases may belong to different communities (PRD §11.2 backfill
  is instance-wide); the same `ConfigCache` instance threaded through
  the tx closure deduplicates duplicate-community lookups
  automatically. The brief §2.1.e prose "read config BEFORE iterating
  cases, not per-case (caching)" is misleading; the correct framing is
  "use ConfigCache (which deduplicates), called per-iteration".
- **Use `LemmyErrorType::NoBodyToSend` or a new variant for empty-reason
  rejection.** Rejected per DQ #139:
  `LemmyErrorType::Unknown("revoke-endorsement reason required".to_string())`
  — mirrors `admin_close_case.rs:30-32` canonical pattern. Adding a
  new lemmy_utils variant is out-of-scope for SL-b (would force a
  cross-crate edit + ts-rs regen).
- **Backfill `governance_log` rate-limit-bypass entries for
  pre-SL-b admin revocations.** Rejected — no such revocations exist
  pre-SL-b (no handler shipped before SL-b).
- **Step-up auth enforcement on admin revocation.** Rejected per PRD
  §12.3 + ADR-010 — v2 reservation; no v1 enforcement.

---

## 5. Metadata

- **Phase:** `v1-SL-b`
- **Branch:** `phase-v1-SL-b` (cut by BM-task before Task 1)
- **Estimated tasks:** 14 (Task 0 pre-flight + Tasks 1-12 impl + Task
  13 retro)
- **Estimated cargo budget:** 0 GB peak local (Shape G — cargo runs on
  GH-hosted runners)
- **Forbidden-window applicability:** non-binding for impl-task
  throughput under Shape G; standard for any local diagnostic cargo
  run (rare under Shape G)
- **Complexity score:** **38/10** — see breakdown below.
  Threshold-tripping; planner DQ #143 filed.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Computed mechanically
from §13 task list:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **7** | 12 impl tasks (Tasks 1-12; Task 0 + retro excluded). `max(0, 12-5) = 7` |
| Migrations touched | +2 each | **0** | SL-b ships zero migrations (per §4.1 + §11) |
| Crates touched | +1 each | **4** | `lemmy_api_common` (governance.rs DTO, Task 1), `lemmy_api_crud` (revoke_endorsement.rs + mod.rs, Task 2), `lemmy_routes` (lib.rs, Task 3), `lemmy_server` (tests/e2e.rs, Tasks 4-12) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **27** | Tasks 4, 5, 6, 7, 8, 9, 10, 11, 12 each modify `crates/server/tests/e2e.rs`. 9 × +3 = 27 |
| New ADR-affecting decisions | +2 each | **0** | PRD §5 + §12 already settled all decisions (DQ #138-#142 resolved at clarify time); SL-b is implementation only |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Shape G — cargo runs off-box |
| **Total** | — | **38** | Threshold for split-DQ: `>8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

Per `feedback_complexity_score_pre_split.md` + `planning.md` §5
"Decision-queue — pre-seed forward-looking OQs": planner files **DQ
#143** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`)
BEFORE committing the plan, asking "Complexity score 38 exceeds 8 —
split `v1-sponsor-liability-b` into `v1-SL-b-1` (Tasks 1-9: DTO +
handler + route + tests #1-6, score ~26) + `v1-SL-b-2` (Tasks 10-12
+ retro: severance tests #7-9, score ~10), or proceed as one plan?"

**Planner observation (non-binding lean):** the dominant factor is
the 9 e2e edits (+27 of the +38). Splitting into b1 (DTO + handler +
route + tests #1-6) and b2 (tests #7-9 + retro) yields scores of
`4 + 0 + 4 + 18 = 26` (b1: 9 impl tasks, 0 migrations, 4 crates, 6
e2e edits) and `0 + 0 + 1 + 9 = 10` (b2: 3 impl tasks, 0 migrations,
1 crate, 3 e2e edits). The b2 score remains above 8; the split does
NOT meaningfully reduce the second sub-phase. Splitting also
fragments a logically atomic deliverable (PRD §15 row 2 names the
deliverable as one phase: "revoke_endorsement handler + DTO + route
+ integration tests").

Per `feedback_principles_not_rules.md`: the score is a signal, not a
hard rule. Mechanical handler+tests work with strong precedents —
`create_endorsement.rs:1-361` is the canonical mirror; each test is
anchor-Edit-friendly (single fn at file end, no helper-extraction
needed because `seed_user`/`seed_community`/`bootstrap` exist from
earlier phases). Per-task wall-clock under Sonnet 4.6 likely 6-8 min
per test (Edit + cargo-check workflow). 9 tests × 7 min average = ~63
min of pure impl-task time, plus per-task validate-pending workflows.

Precedent: SL-a shipped at score 13 with proceed-as-one and no
operational regret per the SL-a retro. JM-e shipped at score 15 with
proceed-as-one. No prior phase has shipped at 38, but the dominant
factor is mechanical e2e-edit multiplicity (low per-task risk), not
ADR/migration/cargo-budget complexity.

This plan ships under the **proceed-as-one** assumption pending DQ
#143 resolution.

---

## 6. Relationship to other v1-SL sub-phases

| Sub-phase | Status | What it ships | SL-b dependency |
|---|---|---|---|
| v1-SL-a | MERGED (PR #111, governance-v0 @ `790f6101d`) | Schema + 13 seeded keys + 5 entry-kind consts (incl. `_ENDORSEMENT_REVOKED`, `_SPONSOR_LIABILITY_ESCAPED`) + 3 CaseStatus variants + Issue #24 partial index + backfill | SL-b reads `endorsement.revoked_at`, `surety.revoked_at`, `moderation_case.{grace_expires_at, liability_escape_reason, status}`, `liability.{multi_sponsor_escape_rule, revoke_rate_limit_per_day}` config keys; fires `_ENDORSEMENT_REVOKED` always + `_SPONSOR_LIABILITY_ESCAPED` on severance |
| **v1-SL-b (THIS PLAN)** | NOT YET CUT | `revoke_endorsement` handler + DTO + route + 9 e2e tests | — |
| v1-SL-c | PENDING (depends on SL-b's `_SPONSOR_LIABILITY_ESCAPED` fire-site verification + SL-a schema) | Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring at 5-min tick + `evaluate_escape_conditions` + `fire_or_escape_case` per-case isolation | SL-c reads `moderation_case.grace_expires_at`; SL-c's escape-branch fires the SAME `_SPONSOR_LIABILITY_ESCAPED` const SL-b's revocation-branch fires (per registry §"v1-SL-a entry kinds"). SL-b does NOT ship the scheduler |
| v1-SL-d | PENDING (depends on SL-c) | `submit_jury_vote` mutation + `apply_sponsor_liability` compute/fire split + `Decided → SponsorLiabilityPending` transition + `_SPONSOR_LIABILITY_PENDING` fire-site | SL-d transitions cases to `SponsorLiabilityPending` (the state SL-b's tests pre-seed via direct DB-write). SL-b does NOT ship SL-d's mutation |
| v1-SL-e | PENDING (depends on SL-c + SL-d) | Lane-wide e2e suite (full revocation-during-window-escapes flow exercising SL-c scheduler + SL-d transition) | SL-b's tests #7-9 exercise revocation severance in isolation (synthetic pre-seed); SL-e's tests exercise the full lane (end-to-end through SL-d's transition) |
| restorative-mechanics-v1 | PENDING (separate PRD) | `restoration_complete` endpoint + restorative escape branch (defendant-initiated; admin-attested) | Independent of SL-b — SL-b handles revocation-branch escape only; restoration-branch escape ships in restorative-mechanics-v1 |

**Cross-PRD sequencing (per PRD §15 + §17.1):**

- SL-b parallel-safe with rep-tuning-r3/r4/r5 (different files +
  different concerns).
- SL-b parallel-safe with admin-dashboard-v1 (admin-config-write
  surface; SL-b READS the config keys, doesn't write).
- SL-b unblocks SL-c (scheduler) and SL-d (mutation). Both depend on
  SL-b's `_SPONSOR_LIABILITY_ESCAPED` fire-site for the revocation
  branch.

---

## 7. Preflight guardrails inherited from prior phases

Per SL-a retro (`feedback_*.md` lessons + R1-R7 inherited via JM-* +
AD-a + SL-a retros) + DQs #138-#142 resolved at clarify time:

- **DQ #44 — Docker daemon preflight.** Enumerated in §13 Task 0
  (R5 — Probe 0).
- **DQ #138 — Score gate at write-time.** Resolved (advisor):
  planner computes from §5.1 breakdown table; gate fires if final
  tally exceeds 8. SL-b ships at 38 with planner DQ #143 filed.
- **DQ #139 — Reason validation error type.** Resolved (advisor):
  `LemmyErrorType::Unknown("revoke-endorsement reason required".to_string())`
  per `admin_close_case.rs:30-32` canonical pattern. Bound in §13
  Task 2 + Task 8 (test #5).
- **DQ #140 — Admin rate-limit bypass log payload.** Resolved
  (advisor): annotate existing `endorsement_revoked` entry with
  `rate_limit_bypassed: true` field; no new ENTRY_KIND const. Bound
  in §13 Task 2 (handler) + Task 9 (test #6).
- **DQ #141 — Tests #2 vs #6 separation.** Resolved (advisor): keep
  separate. Test #2 (Task 5) exercises admin capability under
  threshold; test #6 (Task 9) exercises admin bypass at threshold (5
  prior revocations seeded). Bound in §13 Tasks 5, 9.
- **DQ #142 — Per-case config read.** Resolved (advisor): per-case
  lookup using `ConfigCache` (deduplicates on community_id
  automatically). Bound in §13 Task 2 inside-tx step 6.
- **R1 — `i64::from(...)` on `i32 ↔ i64`.** Bound in §4 watchpoint
  #14 (rate-limit count) + §13 Task 2.
- **R2 — `seed_jury_eligible_snapshots` BEFORE `admin_assign_jury` in
  tests.** Non-binding for SL-b (no jury-assembly tests; tests
  pre-seed `SponsorLiabilityPending` cases directly).
- **R3 — struct-extension grep sweep on `RevokeEndorsement`
  literal-construction sites.** Bound in §13 Task 1. Pre-SL-b sites
  expected: zero (no handler emits `RevokeEndorsement` yet); R3
  fires only if a stale type-construction site is discovered.
- **R4 — lowercase snake_case test names.** Bound in §13 Tasks 4-12.
- **R5 — pre-phase harness audit enumerates ALL probes.** Bound in
  §13 Task 0. Probes 0..12 listed.
- **R6 — uniform `--no-deps -- -D warnings` clippy.** Encoded in
  `cargo-validate-workspace.yml:92`.
- **R7 — `cargo test --no-run -p lemmy_server --test e2e` on tasks
  touching a struct or re-export.** Encoded in
  `cargo-validate-workspace.yml:95`. Bound in Task 1 (DTO struct
  add) + Task 2 (handler module add).
- **JM-d retro §3.5 — `feedback_clippy_rerun_after_fix.md`.** Bound
  in §13 Task 2 GOTCHA — if dispatch refactor unmasks an
  `unused-mut` or `unused-imports` lint, re-run clippy locally before
  push.
- **JM-d retro §5 — `feedback_laptop_default_for_validate_pending.md`.**
  Phase 2 e2e local-default per advisor-orchestrator user gate (PR
  #105, 2026-04-28).
- **SL-a retro §lessons — pseudonym discipline (ADR-015).** Bound in
  §4 watchpoint #4 + §13 Task 2 (handler payload schema).
- **SL-a retro §lessons — registry rule pre-landed-const exemption.**
  SL-b is the fire-site for `_ENDORSEMENT_REVOKED` and the
  revocation-branch fire-site for `_SPONSOR_LIABILITY_ESCAPED`. Per
  registry rule, `(pending)` markers stay until the const has a live
  emitter; SL-b retro flips relevant markers (see §13 Task 13).

---

## 8. Flow design

### 8.1 Before state (post-SL-a-merge)

The endorsement lifecycle today (post-SL-a):

- `POST /api/v4/governance/endorsement` (Phase 5b shipped) →
  `create_endorsement.rs` inserts `endorsement` row + conditional
  `surety` row + 2 `reputation_event` rows + 2 snapshot recomputes +
  emits `endorsement_created` log.
- No revocation handler. `endorsement.revoked_at` and
  `surety.revoked_at` exist as nullable columns but no writer.
- `moderation_case.grace_expires_at` and `liability_escape_reason`
  columns exist (SL-a Task 3) but no writer (SL-d will write
  `grace_expires_at`; SL-b will write `liability_escape_reason` on
  severance).

**PROBLEM:** when a sponsor wants to revoke their endorsement (e.g.
during a grace window for a sponsee whose case has transitioned to
`SponsorLiabilityPending`), there is no API. The escape branch of
PRD §1.2 goal 2 ("sponsor revocation OR defendant restoration during
the grace window severs the liability chain") is uncovered.

### 8.2 After state (v1-SL-b)

New endpoint: `POST /api/v4/governance/endorsement/revoke`.

DTO (`crates/api/api_common/src/governance.rs`):

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RevokeEndorsement {
  pub endorsement_id: EndorsementId,
  pub reason: String,  // required; passes through scrub_json on log emit
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct RevokeEndorsementResponse {
  pub endorsement_id: EndorsementId,
  pub revoked_at: DateTime<Utc>,
  pub liability_chain_severed_for_cases: Vec<ModerationCaseId>,
}
```

Handler (`crates/api/api_crud/src/governance/revoke_endorsement.rs`):

- Outer entry: `check_local_user_valid`; capture
  `caller_id = local_user_view.person.id`; capture
  `caller_pseudonym` via `actor_pseudonym_helper::get_or_create`;
  capture `is_admin_caller = is_admin(&local_user_view).is_ok()`.
- Pre-tx: if NOT admin, run rate-limit count
  (`count(*) FROM endorsement WHERE from_person_id = caller_id AND
  revoked_at > now() - INTERVAL '24 hours'`); reject with
  `LemmyErrorType::RateLimitError` if exceeds
  `liability.revoke_rate_limit_per_day` (default 5). If ADMIN AND
  count would exceed, set `bypass_recorded = true` flag (used in step
  8 log payload).
- Pre-tx: validate reason — `data.reason.trim().is_empty()` →
  `LemmyErrorType::Unknown("revoke-endorsement reason required".to_string())`.
- Open `run_transaction` → `process_revocation(conn, caller_id,
  caller_pseudonym, is_admin_caller, bypass_recorded, data)`.
- Inside-transaction body (steps 1-8 per §4.1).
- Return `Json(RevokeEndorsementResponse { ... })`.

Module wiring (`crates/api/api_crud/src/governance/mod.rs`):

```rust
pub mod create_endorsement;
pub mod create_report;
pub mod revoke_endorsement;   // NEW (alphabetical: between create_report and request_appeal)
pub mod request_appeal;
```

Route registration (`crates/api/routes/src/lib.rs`):

```rust
// In the use block (line ~149):
governance::{
    create_endorsement::create_endorsement,
    create_report::create_report,
    request_appeal::request_appeal,
    revoke_endorsement::revoke_endorsement,  // NEW
},

// In the /governance scope (line ~524, immediately after /endorsement):
.route("/endorsement", post().to(create_endorsement))
.route("/endorsement/revoke", post().to(revoke_endorsement))  // NEW
```

Registry change: at retro time, the registry §"v1-SL-a entry kinds"
table "Emitting handler" column flips
`_ENDORSEMENT_REVOKED` from "(pending)" to "(active)" entirely; for
`_SPONSOR_LIABILITY_ESCAPED`, the SL-b portion flips to "(active)"
while the SL-c portion (scheduler branch) stays "(pending)". The
registry count check `38` stays unchanged.

### 8.3 Endpoint changes

- `POST /api/v4/governance/endorsement/revoke` — NEW. Accepts
  `RevokeEndorsement { endorsement_id, reason }`. Returns
  `RevokeEndorsementResponse { endorsement_id, revoked_at,
  liability_chain_severed_for_cases }`.
- No DTO changes to other endpoints.
- No changes to `POST /api/v4/governance/endorsement` (SL-b is purely
  additive on the route table).

---

## 9. Mandatory reading

### 9.1 Brehon design docs (P0)

- `.claude/PRPs/prds/v1-sponsor-liability.prd.md` §1 (vision), §2
  (IN/OUT scope), §3.1-3.4 (CaseStatus extensions), **§5
  (`POST /api/v4/governance/endorsement/revoke`)** §5.1-§5.6, §7.3
  (restoration interaction — out for SL-b), **§8.1
  (`liability_escape_reason` JSONB schema)**, §10 (Defaults Matrix),
  §11 (backwards-compat), **§12 (security — §12.1 rate-limit, §12.2
  reason-required + scrub)**, §15 (impl phases), §17 (cross-cutting
  impact), §18 (B4 key-rename table).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  ADR-010 (won't-disadvantage rule), **ADR-013 (CaseStatus
  enum-exhaustiveness — no `_ =>` arms)**, ADR-014 (federation
  deferral), **ADR-015 (pseudonymisation)**, OQ-V1-SL-05
  (`liability_escape_reason` schema versioning), OQ-V1-SL-01
  (multi-sponsor escape rule).
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` §17
  (governance table inventory).

### 9.2 Codebase reads (P0 — mirror these patterns)

- **`crates/api/api_crud/src/governance/create_endorsement.rs`
  (entire file, 361 lines)** — primary canonical mirror. Read in full
  the first time. Specifically:
  - Lines 1-34: module doc-comment shape — SL-b handler doc-comment
    mirrors with revoke-specific Watch citations.
  - Lines 35-69: use block — verify exact import paths.
  - Lines 118-145: outer handler (entry, `check_local_user_valid`,
    pseudonym setup, `run_transaction` open).
  - Lines 150-334: `process_endorsement` body (named transaction
    helper). SL-b's `process_revocation` mirrors with UPDATE-then-
    cascade shape and idempotency early-return at step 1.
  - Lines 198-214: cooldown rate-limit pattern (count + reject
    pre-tx) — SL-b's rate-limit count mirrors but applies different
    config key (`liability.revoke_rate_limit_per_day` not cooldown
    constant).
  - Lines 307-309: snapshot recompute calls (sponsor + sponsee) —
    SL-b mirrors verbatim with caller_id + endorsement.to_person_id.
  - Lines 311-328: governance_log::append pattern — SL-b mirrors for
    `endorsement_revoked` entry; reuses scrub-via-append discipline.
- **`crates/api/api_common/src/governance.rs` lines 287-313** — DTO
  block. Confirm:
  - Lines 287-295 `CreateEndorsement` derive stack (mirror for
    DEFINING `RevokeEndorsement` with same `Clone Default
    PartialEq Eq Hash` minus `Copy`).
  - Lines 297-305 `CreateEndorsementResponse` derive stack (mirror
    for `RevokeEndorsementResponse`).
  - Lines 307-313 existing `RevokeEndorsement` (the struct SL-b
    extends; verify derives + field shape).
- **`crates/api/api_crud/src/governance/mod.rs`** (12 lines) —
  add `pub mod revoke_endorsement;` alphabetically (between
  `create_report` and `request_appeal`).
- **`crates/api/routes/src/lib.rs` lines 145-160** (use block) +
  lines 510-540 (the `/governance` scope). Specifically:
  - Line 149: `create_endorsement::create_endorsement,` — the
    import pattern SL-b mirrors.
  - Line 523: `.route("/endorsement", post().to(create_endorsement))`
    — SL-b inserts the new route immediately after this line. PRD
    §5.6 cites stale line 518.
- **`crates/api/api/src/governance/admin_close_case.rs` lines
  23-105**:
  - Lines 28: `is_admin(&local_user_view)?;` — admin gate pattern.
  - Lines 30-32: reason-validation pattern
    (`data.reason.trim().is_empty() → LemmyErrorType::Unknown`) per
    DQ #139.
  - Lines 65-80: `match case.status` exhaustive enumeration of all
    12 CaseStatus variants — ADR-013 pattern.
- **`crates/api/api/src/governance/governance_log.rs`**:
  - Lines 51, 74, 76 (and surrounding `pub use` block): confirms
    `_ENDORSEMENT_REVOKED`, `_SPONSOR_LIABILITY_ESCAPED`,
    `_SPONSOR_LIABILITY_PENDING` re-exports shipped in SL-a Task 7.
- **`crates/db_schema/src/source/governance/governance_log.rs`
  lines 195-199** — confirms 5 SL-a const declarations
  (`_ENDORSEMENT_REVOKED 195`, `_RESTORATION_COMPLETED 196`,
  `_SPONSOR_LIABILITY_ESCAPED 197`, `_SPONSOR_LIABILITY_FIRED 198`,
  `_SPONSOR_LIABILITY_PENDING 199`). Lines 220-282: `pub async fn
  append` signature + scrub_json behaviour.
- **`crates/api/api/src/governance/config.rs`**:
  - Line 937: `DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE: &str =
    "any_revocation"` — default const.
  - Line 940: `DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY: i64 = 5`
    — default const.
  - Lines 200-217: `ConfigCache` API.
  - Lines 213-217: `ConfigCache::new()`.
  - Lines 225-256: `pub async fn get_int` — SL-b's rate-limit reader.
  - Lines 324-388: `pub async fn get_text` — SL-b's escape-rule
    reader (Scope::Community cascade).
  - Lines 718-758: `fetch_value` cascade implementation — confirms
    Community → Instance fallback is built-in, no manual cascade
    needed in SL-b handler.
- **`crates/api/api/src/governance/sponsor_liability.rs`** (entire
  file) — read for context. SL-b's handler doesn't call into this
  file directly, but the file documents the SL-b-relevant state
  space (Watch 10 PII discipline, Watch 3 exhaustive-match no
  wildcard).
- **`crates/db_schema/src/source/governance/endorsement.rs`** (34
  lines) — `Endorsement` struct + `EndorsementInsertForm`. NB: there
  is no `EndorsementUpdateForm` — SL-b handler uses
  `diesel::update(endorsement::table.filter(...)).set(endorsement::revoked_at.eq(now))`
  directly.
- **`crates/db_schema/src/source/governance/surety.rs`** (34 lines)
  — `Surety` struct. Similarly no UpdateForm; direct `diesel::update`
  pattern.
- **`crates/db_schema/src/source/governance/moderation_case.rs`
  lines 21-94 (struct) + 96-137 (InsertForm)** — confirms
  `grace_expires_at: Option<DateTime<Utc>>` (line 88) +
  `liability_escape_reason: Option<Value>` (line 93). InsertForm has
  matching `Option<_>` fields (lines 135-136); v0/earlier-v1 callers
  compile via `..Default::default()`. SL-b's tests pre-seed via
  `ModerationCaseInsertForm` with `status:
  CaseStatus::SponsorLiabilityPending, grace_expires_at: Some(future),
  target_person_id: Some(sponsee_id)`.
- **`crates/db_schema_file/src/enums.rs:393-432`** — `CaseStatus`
  enum (12 variants post-SL-a). Confirm
  `SponsorLiabilityPending/Fired/Escaped` at lines 411-432.
- **`crates/db_schema_file/src/schema.rs`** `moderation_case` block
  — confirm `grace_expires_at -> Nullable<Timestamptz>` and
  `liability_escape_reason -> Nullable<Jsonb>` columns. The
  `endorsement` table block — confirm `revoked_at ->
  Nullable<Timestamptz>` (Phase 5a baseline). The `surety` table —
  confirm `revoked_at -> Nullable<Timestamptz>` + the
  `surety_sponsored_id_active` partial index (SL-a Task 1, Issue
  #24).

### 9.3 Test patterns (P1 — fixture sources)

- **`crates/server/tests/e2e.rs:88-...`** `mod governance_fixtures`
  — outermost test fixture module. SL-b's `mod v1_sl_b_fixtures`
  is a sibling at file-end (NOT nested inside).
- **`crates/server/tests/e2e.rs:2730-2760`** — fixture-mod use
  block. SL-b's mod will import similar shape with additions for
  `endorsement::table`, `EndorsementInsertForm`,
  `RevokeEndorsement`, `RevokeEndorsementResponse`,
  `revoke_endorsement` handler.
- **`crates/server/tests/e2e.rs:2871-2886`** — `seed_surety` helper.
  Mirror for SL-b's `seed_endorsement_with_surety` if needed (or
  inline per-test for clarity).
- **`crates/server/tests/e2e.rs:2917-2937`** —
  `seed_organic_endorsement` helper (note: this seeds a
  `reputation_event`, not an `endorsement` row; SL-b tests need a
  REAL `endorsement` row via `EndorsementInsertForm`).
- **`crates/server/tests/e2e.rs:9786-...`** — `mod
  v1_jm_e_fixtures` block — most recent fixture-mod precedent. SL-b
  ships AFTER this block at file end (anchor: line 10976).
- **`crates/server/tests/e2e.rs:1700-1900`** —
  `ModerationCaseInsertForm` direct-write patterns (existing tests
  insert cases via this; SL-b tests #7-9 follow same shape with
  `status: SponsorLiabilityPending` + `grace_expires_at: Some(now +
  24h)`).

### 9.4 Rules (P0 — auto-loaded)

- `.claude/rules/governance-log-entry-kind-registry.md` — registry
  count `38` + `(pending)`→`(active)` discipline for the SL-b
  fire-sites
- `.claude/rules/decision-queue.md` — schema-v2 attribution + Recipe
  1 + Recipe 2 + `kind: validate-pending` Shape G routing
- `.claude/rules/advisor-orchestrator.md` — Shape G + Phase 2 e2e
  user gate
- `.claude/rules/branch-manager.md`, `.claude/rules/phase-branch.md`
  — branch-cut + PR flow
- `.claude/rules/cargo-output-capture.md` +
  `.claude/rules/no-cargo-output-paste.md` — local-cargo discipline
  if any
- `.claude/rules/pm-plugin-hooks-stable.md` (Task 0 Probe 10)

### 9.5 External documentation

- chrono: `Utc::now()`, `DateTime<Utc>`, `Duration::hours(i64)` —
  used for rate-limit cutoff + `revoked_at` timestamp.
- diesel-async: `RunQueryDsl::execute` + `update().set(...)` tuple
  form + `count_star()`.
- serde_json: `json!(...)` for log payloads + the
  `liability_escape_reason` JSONB shape.
- actix-web: `Json<T>` + `Data<LemmyContext>` + route registration
  via `post().to(...)`.

---

## 10. Patterns to mirror

### 10.1 Outer handler shape

**SOURCE:** `crates/api/api_crud/src/governance/create_endorsement.rs:118-145`.

```rust
pub async fn revoke_endorsement(
  Json(data): Json<RevokeEndorsement>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RevokeEndorsementResponse>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;
  let is_admin_caller = is_admin(&local_user_view).is_ok();

  // PRE-TX: rate-limit count (admin bypasses).
  let rate_limit_per_day: i64 = config::get_int(
    &mut ConfigCache::new(),
    &mut context.pool(),
    Scope::Instance,
    "liability.revoke_rate_limit_per_day",
  )
  .await?;
  let cutoff: DateTime<Utc> = Utc::now() - Duration::hours(24);
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let recent_count: i64 = endorsement::table
    .filter(endorsement::from_person_id.eq(caller_id))
    .filter(endorsement::revoked_at.gt(cutoff))
    .select(count_star())
    .get_result(conn)
    .await?;
  let bypass_recorded = is_admin_caller && recent_count >= rate_limit_per_day;
  if !is_admin_caller && recent_count >= rate_limit_per_day {
    return Err(LemmyErrorType::RateLimitError.into());
  }

  // PRE-TX: reason validation (DQ #139).
  if data.reason.trim().is_empty() {
    return Err(LemmyErrorType::Unknown(
      "revoke-endorsement reason required".to_string()
    ).into());
  }

  let data_for_tx = data.clone();
  let pseudonym_for_tx = caller_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        process_revocation(
          conn,
          caller_id,
          pseudonym_for_tx,
          is_admin_caller,
          bypass_recorded,
          data_for_tx,
        ).await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}
```

**GOTCHA:** `LocalUserView::person.id` is the caller's PersonId. The
`is_admin(...)?` returning `Ok(())` denotes admin; the `.is_ok()`
boolean conversion captures it without erroring on non-admin (which
is valid — non-admins are allowed to self-revoke).

**GOTCHA (R1):** rate-limit count (`recent_count: i64`) compared
against `rate_limit_per_day: i64` — direct comparison, no `as` cast.

### 10.2 Transaction body — `process_revocation`

**SOURCE:** `crates/api/api_crud/src/governance/create_endorsement.rs:150-334`.

```rust
async fn process_revocation(
  conn: &mut AsyncPgConnection,
  caller_id: PersonId,
  caller_pseudonym: String,
  is_admin_caller: bool,
  bypass_recorded: bool,
  data: RevokeEndorsement,
) -> LemmyResult<RevokeEndorsementResponse> {
  let mut config = ConfigCache::new();

  // Step 1: load endorsement FOR UPDATE; idempotency check.
  let row: Endorsement = endorsement::table
    .filter(endorsement::id.eq(data.endorsement_id))
    .for_update()
    .select(Endorsement::as_select())
    .first(conn)
    .await?;

  // Step 1.5: idempotency (PRD §5.4) — re-revocation is a no-op.
  if let Some(existing_revoked_at) = row.revoked_at {
    return Ok(RevokeEndorsementResponse {
      endorsement_id: row.id,
      revoked_at: existing_revoked_at,
      liability_chain_severed_for_cases: vec![],
    });
  }

  // Step 2: capability check.
  if !is_admin_caller && row.from_person_id != caller_id {
    return Err(LemmyErrorType::NotFound.into());
  }

  let now: DateTime<Utc> = Utc::now();

  // Step 3: UPDATE endorsement.revoked_at.
  diesel::update(endorsement::table.filter(endorsement::id.eq(row.id)))
    .set(endorsement::revoked_at.eq(Some(now)))
    .execute(conn)
    .await?;

  // Step 4: UPDATE matching surety row (UPDATE no-op if not found).
  let revoking_sponsor_id = row.from_person_id;
  diesel::update(
    surety::table
      .filter(surety::sponsor_id.eq(revoking_sponsor_id))
      .filter(surety::sponsored_id.eq(row.to_person_id))
      .filter(surety::community_id.eq(row.community_id))
      .filter(surety::revoked_at.is_null())
  )
    .set(surety::revoked_at.eq(Some(now)))
    .execute(conn)
    .await?;

  // Step 5: re-query active sponsors (post-step-4 state).
  // (Used inside the loop at step 6 for all_revocation / majority_revocation.)

  // Step 6: grace-window evaluation loop.
  let pending_cases: Vec<ModerationCase> = moderation_case::table
    .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
    .filter(moderation_case::target_person_id.eq(Some(row.to_person_id)))
    .filter(moderation_case::grace_expires_at.gt(Some(now)))
    .select(ModerationCase::as_select())
    .load(conn)
    .await?;

  let mut severed: Vec<ModerationCaseId> = vec![];
  for case in &pending_cases {
    let scope = match case.community_id {
      Some(cid) => Scope::Community(cid),
      None => Scope::Instance,
    };
    let escape_rule = config::get_text(
      &mut config,
      &mut (&mut *conn).into(),
      scope,
      "liability.multi_sponsor_escape_rule",
    ).await?;

    let active_sponsor_count: i64 = surety::table
      .filter(surety::sponsored_id.eq(row.to_person_id))
      .filter(surety::revoked_at.is_null())
      .select(count_star())
      .get_result(conn)
      .await?;

    let escapes = match escape_rule.as_str() {
      "all_revocation" => active_sponsor_count == 0,
      "majority_revocation" => {
        // pre-revoke sponsor count = active + 1 (this caller just revoked)
        let pre_count = active_sponsor_count + 1;
        let revoked_since_decision = pre_count - active_sponsor_count;
        revoked_since_decision * 2 > pre_count
      }
      _ => true, // "any_revocation" default + unknown fallback
    };

    if escapes {
      let escape_reason = json!({
        "version": 1,
        "reason": "sponsor_revoked",
        "actor_pseudonym": caller_pseudonym,
        "endorsement_id": row.id.0,
      });
      diesel::update(moderation_case::table.filter(moderation_case::id.eq(case.id)))
        .set((
          moderation_case::status.eq(CaseStatus::SponsorLiabilityEscaped),
          moderation_case::liability_escape_reason.eq(Some(escape_reason.clone())),
        ))
        .execute(conn)
        .await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
        json!({
          "case_id": case.id.0,
          "escaped_at": now,
          "reason": "sponsor_revoked",
          "actor_pseudonym": caller_pseudonym,
          "endorsement_id": row.id.0,
        }),
        Some(caller_pseudonym.clone()),
      ).await?;
      severed.push(case.id);
    }
  }

  // Step 7: recompute snapshots (sponsor + sponsee).
  reputation_snapshot::recompute_snapshot(conn, revoking_sponsor_id, row.community_id, &mut config).await?;
  reputation_snapshot::recompute_snapshot(conn, row.to_person_id, row.community_id, &mut config).await?;

  // Step 8: emit endorsement_revoked log entry (always).
  let target_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), row.to_person_id).await?;
  let mut payload = json!({
    "sponsor_pseudonym": caller_pseudonym,
    "target_pseudonym": target_pseudonym,
    "community_id": row.community_id.map(|c| c.0),
    "reason": data.reason,
    "liability_chain_severed_for_cases": severed.iter().map(|c| c.0).collect::<Vec<_>>(),
  });
  if bypass_recorded {
    payload["rate_limit_bypassed"] = json!(true);
  }
  governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_ENDORSEMENT_REVOKED,
    payload,
    Some(caller_pseudonym.clone()),
  ).await?;

  Ok(RevokeEndorsementResponse {
    endorsement_id: row.id,
    revoked_at: now,
    liability_chain_severed_for_cases: severed,
  })
}
```

**GOTCHA (TOCTOU):** the `for_update()` lock + idempotency early-return
inside the transaction is load-bearing.

**GOTCHA (ADR-013):** the filter
`status.eq(SponsorLiabilityPending)` ensures SL-b NEVER mutates
`Decided` cases.

**GOTCHA (ADR-015 — pseudonym discipline):** every JSONB payload
field naming a person is `*_pseudonym` (string). NEVER write
`caller_id.0` or `endorsement.from_person_id.0` directly.

**GOTCHA (per DQ #142 — per-case ConfigCache):** the
`config::get_text` call inside the `for case in &pending_cases` loop
reads via `ConfigCache` which deduplicates `(scope_repr, key)`
tuples.

### 10.3 Test fixture mod shape

**SOURCE:** `crates/server/tests/e2e.rs:9786-...` (mod
`v1_jm_e_fixtures`).

```rust
mod v1_sl_b_fixtures {
  use super::*;
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, insert_into, update};
  use diesel_async::RunQueryDsl as AsyncRunQueryDsl;
  use lemmy_api_common::governance::{
    RevokeEndorsement, RevokeEndorsementResponse,
  };
  use lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement;
  use lemmy_db_schema::{
    newtypes::{EndorsementId, ModerationCaseId, PersonId},
    source::governance::{
      endorsement::{Endorsement, EndorsementInsertForm},
      moderation_case::ModerationCaseInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    enums::{CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType, SeverityTier},
    schema::{endorsement, moderation_case, surety},
  };

  async fn seed_endorsement_active(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsee: PersonId,
  ) -> Result<EndorsementId, Box<dyn Error>> { /* ... */ }

  async fn seed_pending_case(
    conn: &mut AsyncPgConnection,
    sponsee: PersonId,
    community: Option<CommunityId>,
    grace_hours: i64,
  ) -> Result<ModerationCaseId, Box<dyn Error>> { /* ... */ }

  // ... 9 #[tokio::test] async fn ... per Tasks 4-12 ...
}
```

**GOTCHA (R4):** test fn names lowercase snake_case.

**GOTCHA (e2e fixture-mod placement):** the new mod opens AFTER mod
`v1_jm_e_fixtures` closes (line 10975) at file end. Subsequent
sub-phases (SL-c, SL-d, SL-e) extend their own fixture mods after
SL-b's mod.

### 10.4 governance_log payload schema (escape branch)

**SOURCE:** PRD §8.1 + DQ #139/#140/#142 resolutions.

`liability_escape_reason` JSONB (written to `moderation_case` row):

```json
{
  "version": 1,
  "reason": "sponsor_revoked",
  "actor_pseudonym": "<caller_pseudonym>",
  "endorsement_id": <endorsement.id.0>
}
```

`endorsement_revoked` log entry payload:

```json
{
  "sponsor_pseudonym": "<caller_pseudonym>",
  "target_pseudonym": "<target_pseudonym>",
  "community_id": <endorsement.community_id.0 or null>,
  "reason": "<scrubbed via governance_log::append>",
  "liability_chain_severed_for_cases": [<case.id.0> ...],
  "rate_limit_bypassed": true   // only when (admin AND would-have-been-rate-limited); else absent
}
```

`sponsor_liability_escaped` log entry payload (per case in severed
list):

```json
{
  "case_id": <case.id.0>,
  "escaped_at": "<DateTime<Utc> ISO 8601>",
  "reason": "sponsor_revoked",
  "actor_pseudonym": "<caller_pseudonym>",
  "endorsement_id": <endorsement.id.0>
}
```

**GOTCHA (ADR-015):** `actor_pseudonym` is REQUIRED in the
`liability_escape_reason` JSONB AND in the
`sponsor_liability_escaped` log payload.

### 10.5 Reason validation pattern

**SOURCE:** `crates/api/api/src/governance/admin_close_case.rs:30-32`
+ DQ #139 resolution.

```rust
if data.reason.trim().is_empty() {
  return Err(LemmyErrorType::Unknown(
    "revoke-endorsement reason required".to_string()
  ).into());
}
```

**GOTCHA:** the trim happens before .is_empty() to reject
whitespace-only reasons. Per PRD §12.2.

### 10.6 Route registration pattern

**SOURCE:** `crates/api/routes/src/lib.rs:149` (use block) +
`lib.rs:523` (route).

Use block addition:

```rust
governance::{
  create_endorsement::create_endorsement,
  create_report::create_report,
  request_appeal::request_appeal,
  revoke_endorsement::revoke_endorsement,  // NEW (alphabetical)
},
```

Route addition (immediately after `/endorsement` line):

```rust
.route("/endorsement", post().to(create_endorsement))
.route("/endorsement/revoke", post().to(revoke_endorsement))  // NEW
```

**GOTCHA:** PRD §5.6 cites `lib.rs:518`; the actual line at
plan-write time is `lib.rs:523`. Plan §13 Task 3 includes a
`grep -n '"/endorsement"' crates/api/routes/src/lib.rs` to find the
current line; anchor-insert immediately after the matched line.

### 10.7 Rate-limit + bypass-flag pattern

**SOURCE:** PRD §12.1 + DQ #140 resolution +
`create_endorsement.rs:198-214` cooldown precedent.

```rust
// PRE-TX:
let rate_limit_per_day: i64 = config::get_int(...).await?;
let cutoff: DateTime<Utc> = Utc::now() - Duration::hours(24);
let recent_count: i64 = endorsement::table
  .filter(endorsement::from_person_id.eq(caller_id))
  .filter(endorsement::revoked_at.gt(cutoff))
  .select(count_star())
  .get_result(conn)
  .await?;

let bypass_recorded = is_admin_caller && recent_count >= rate_limit_per_day;
if !is_admin_caller && recent_count >= rate_limit_per_day {
  return Err(LemmyErrorType::RateLimitError.into());
}
```

**GOTCHA (DQ #140):** `bypass_recorded` is set ONLY when (admin AND
would-have-been-rate-limited). Standard admin revocations under
threshold OR non-admin revocations under threshold see
`bypass_recorded = false` and the `endorsement_revoked` payload
omits the `rate_limit_bypassed` field entirely.

---

## 11. Files to change

### `lemmy_api_common` crate

- `crates/api/api_common/src/governance.rs` — extend
  `RevokeEndorsement` with `reason: String`; add new
  `RevokeEndorsementResponse` struct. Drop `Copy` derive on both
  (String + Vec are not Copy). Keep `Default`. **Task 1**.

### `lemmy_api_crud` crate

- `crates/api/api_crud/src/governance/revoke_endorsement.rs` — NEW
  file. Outer handler + `process_revocation` body. **Task 2**.
- `crates/api/api_crud/src/governance/mod.rs` — add `pub mod
  revoke_endorsement;` alphabetically between `create_report;` and
  `request_appeal;`. **Task 2**.

### `lemmy_routes` crate

- `crates/api/routes/src/lib.rs` — extend governance use block with
  `revoke_endorsement::revoke_endorsement,` (line ~149) + add route
  `.route("/endorsement/revoke", post().to(revoke_endorsement))`
  immediately after `/endorsement` (line ~524). **Task 3**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — append new `mod v1_sl_b_fixtures`
  AFTER mod `v1_jm_e_fixtures` (line ~10976) at file end. Within the
  mod, ship 9 tests via 9 anchor-Edit tasks (Tasks 4-12). **Tasks
  4-12**. (Task 4 ships the mod shell + helpers + test #1; Tasks 5-12
  anchor-insert subsequent tests inside the same mod.)

### Meta files (rules + reports)

- `.claude/rules/governance-log-entry-kind-registry.md` — flip
  `(pending)` → `(active)` markers on the SL-b fire-sites:
  `_ENDORSEMENT_REVOKED` row entirely; `_SPONSOR_LIABILITY_ESCAPED`
  row's SL-b portion (the SL-c portion stays `(pending)`). **Task
  13 (retro)**.
- `.claude/PRPs/reports/v1-SL-b-retro.md` — CREATE retro per
  `feedback_retro_not_report.md` +
  `feedback_four_role_retro_signals.md` +
  `feedback_retro_task_complexity_score.md`. **Task 13**.

### Files explicitly NOT touched

- `crates/api/api/src/governance/sponsor_liability.rs` — v0 helper
  stays intact through SL-b; SL-d is the rewrite.
- `crates/api/api/src/governance/submit_jury_vote.rs` — SL-d adds
  the `Decided → SponsorLiabilityPending` transition; SL-b does NOT
  touch.
- `crates/api/api/src/governance/sponsor_liability_grace.rs` — does
  not exist yet. SL-c creates.
- `crates/api/api/src/governance/governance_log.rs` (api shim) — no
  re-export changes; SL-a Task 7 already exported the 5 SL-a consts.
- `crates/db_schema/src/source/governance/governance_log.rs` — no
  const additions; SL-a Task 7 declared the 5 SL-a consts.
- `crates/db_schema/src/source/governance/{endorsement,surety,moderation_case}.rs`
  — no struct changes; SL-a Task 4 added the SL-a fields.
- `crates/db_schema_file/src/{enums,schema}.rs` — no schema changes.
- `migrations/**` — zero migrations; SL-a is the schema foundation.
- `crates/api/api/src/governance/config.rs` — no const additions; the
  2 SL-b-required keys shipped in SL-a Task 6.
- `crates/db_views/governance_case/src/impls.rs` — view-crate stays
  unchanged; new SponsorLiability* variants were already handled in
  SL-a Task 5 ADR-013 sweep.
- `crates/api/api/src/governance/{request_appeal,admin_close_case,admin_assign_jury,...}.rs`
  — already ADR-013-extended in SL-a Task 5.
- `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
  `.coderabbit.yaml` — no dep / build-config / review-config changes.
- `.github/workflows/**` — no workflow YAML changes.

---

## 12. NOT building in v1-SL-b

- **Scheduler module `sponsor_liability_grace.rs` + clokwerk wiring**
  — SL-c's. SL-b's tests pre-seed `SponsorLiabilityPending` cases via
  direct DB-write; SL-c writes the runtime production path.
- **`submit_jury_vote` mutation: compute/fire split + `Decided →
  SponsorLiabilityPending` transition** — SL-d's. The v0
  `apply_sponsor_liability` at `sponsor_liability.rs:142` keeps its
  v0 shape through SL-b.
- **`restoration/complete` endpoint** — restorative-mechanics-v1
  PRD's. SL-a declared the `RESTORATION_COMPLETED` const for
  `governance_log.rs` const-discipline; the endpoint ships separately.
- **e2e behavioural tests for the full lane** (revocation-during-
  window-escapes through SL-c scheduler + SL-d transition) — SL-e's.
  SL-b's tests #7-9 exercise revocation severance in isolation
  (synthetic pre-seed of `SponsorLiabilityPending`).
- **Sponsor notification UX** — out per PRD §13 OQ-V1-SL-03.
- **Step-up auth enforcement on `RevokeEndorsement.step_up_token`** —
  out per PRD §12.3 (v2 reservation). The DTO does NOT carry a
  `step_up_token` field in v1.
- **Dashboard write surface for `liability.*` keys** —
  admin-dashboard-v1 PRD's. v0 admins edit via direct `psql` until
  that ships.
- **Cross-instance sponsor-liability federation** — out per PRD §2
  OUT + ADR-014.
- **Sponsor-of-sponsor liability chain depth** — kept 1-deep per PRD
  §13 OQ-V1-SL-02.
- **New ENTRY_KIND_*** consts** — all 5 already shipped in SL-a. SL-b
  fires existing consts; does NOT add to the registry.
- **New CaseStatus variants** — SL-a shipped 3
  (`SponsorLiabilityPending/Fired/Escaped`); SL-b uses; does not add.
- **New `governance_config` seeds** — SL-a shipped 13 (10
  `liability.*` + 3 `job.grace_check_*`); SL-b reads
  `liability.{multi_sponsor_escape_rule, revoke_rate_limit_per_day}`;
  does not seed.
- **Schema migrations** — none; SL-a is the schema foundation.
- **Backfill of v0 endorsements** — out; existing endorsements with
  `revoked_at = NULL` continue working unchanged (PRD §5.5).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per
`feedback_pr_per_phase.md`). Each task header carries a `[P]` marker
iff its **FILES** YAML `union(creates, modifies)` shares no path with
any other `[P]`-marked task in the same cohort. Per the YAML overlap
rule (Tasks 4-12 all `modifies: crates/server/tests/e2e.rs`), the
e2e tasks are NOT cohort-compatible; they ship serially. Tasks 2 and
3 are also serial because Task 3 imports the symbol Task 2 creates.
Task 0 (pre-flight harness audit) is **always** non-`[P]`.

> **Cohort dispatch (advisor-side):** No `[P]` cohorts in SL-b. All
> tasks dispatch serially per `advisor-orchestrator.md`. Single-task
> cohort each; cohort handover aggregation per
> `feedback_handover_trailer_cohort_propagation.md` still applies as
> a per-task `HANDOVER:` commit trailer when the next task benefits.

> **Shape G (Layer G2 push-and-exit):** §13 task bodies do NOT inline
> cargo invocations. Each task ends with a push to the worker branch;
> the impl-task subagent writes a `kind: "validate-pending"` DQ entry
> referencing `cargo-validate-workspace.yml` per
> `.claude/rules/decision-queue.md` schema-v2.

### Task 0: Pre-flight harness audit + branch verification + SL-a state confirmation

**Goal:** verify environment + branch (`phase-v1-SL-b`) + SL-a
schema/handlers/seeds intact.

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes):**

```bash
# Probe 0 — Docker daemon (e2e harness uses testcontainers-rs)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init (Linux/Junior worktree quirk)
git submodule status > /tmp/sl-b-task0-submodule.log 2>&1
if grep -q '^-' /tmp/sl-b-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/sl-b-task0-submodule-init.log 2>&1
  echo "submodule init exit: $?"
fi

# Probe 1 — branch verification
git branch --show-current
# EXPECT: phase-v1-SL-b (BM-task cuts before Task 1)

# Probe 2 — SL-a state confirmation: 3 CaseStatus variants present
rg -n 'SponsorLiabilityPending|SponsorLiabilityFired|SponsorLiabilityEscaped' crates/db_schema_file/src/enums.rs | head
# EXPECT: 3+ matches (variant declarations)

# Probe 3 — SL-a state confirmation: schema columns present
rg -n 'grace_expires_at -> Nullable<Timestamptz>|liability_escape_reason -> Nullable<Jsonb>' crates/db_schema_file/src/schema.rs | head
# EXPECT: both lines

# Probe 4 — SL-a state confirmation: ENTRY_KIND consts declared
rg -n 'ENTRY_KIND_ENDORSEMENT_REVOKED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED' crates/db_schema/src/source/governance/governance_log.rs | head
# EXPECT: both consts at lines 195 + 197

# Probe 5 — SL-a state confirmation: shim re-exports
rg -n 'ENTRY_KIND_ENDORSEMENT_REVOKED|ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED' crates/api/api/src/governance/governance_log.rs | head
# EXPECT: both re-exported in pub use block

# Probe 6 — SL-a state confirmation: config keys seeded
rg -n 'DEFAULT_LIABILITY_MULTI_SPONSOR_ESCAPE_RULE|DEFAULT_LIABILITY_REVOKE_RATE_LIMIT_PER_DAY' crates/api/api/src/governance/config.rs | head
# EXPECT: 2 const declarations + 2 match arms + 2 SEEDED_KEYS_WITH_CONSTS tuples + 2 CONFIG_KEY_METADATA entries

# Probe 7 — existing RevokeEndorsement DTO shape confirmation
rg -n 'pub struct RevokeEndorsement' crates/api/api_common/src/governance.rs | head
# EXPECT: at line ~311 (Task 1 extends; verify no concurrent rename)

# Probe 8 — current endorsement route line
grep -n '"/endorsement"' crates/api/routes/src/lib.rs | head
# EXPECT: line containing `.route("/endorsement", post().to(create_endorsement))` at line ~523. If line shifted, Task 3 anchors at the actual current line.

# Probe 9 — DQ pending status (advisory)
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 10 — PM-plugin-hooks-stable check
for h in \
  local_private_message_before_create \
  local_private_message_after_create \
  local_private_message_before_update \
  local_private_message_after_update \
  federated_private_message_before_receive \
  federated_private_message_after_receive; do
  rg -q "\"$h\"" crates/ || { echo "missing hook literal: $h"; exit 1; }
done

# Probe 11 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("revoke_endorsement\\.rs|api_common/src/governance\\.rs|routes/src/lib\\.rs|tests/e2e\\.rs")) | {number, title, headRefName}'
# EXPECT: empty output

# Probe 12 — Shape G workflow YAMLs accessible (yamllint may not be installed; soft-fail)
yamllint .github/workflows/cargo-validate-workspace.yml \
         .github/workflows/cargo-test-e2e.yml > /tmp/sl-b-task0-yamllint.log 2>&1 || \
         echo "yamllint not installed or warnings — non-blocking; advisor verifies workflow shape pre-merge"
```

**EXPECT:** Probes 0..11 exit 0 (or, for Probe 0/1, exit 1 with
explicit STOP). Probe 12 is informational.

**No commit at Task 0** — verification only.

### Task 1: Extend RevokeEndorsement DTO + add RevokeEndorsementResponse

**ACTION:** in `crates/api/api_common/src/governance.rs`, extend the
existing `RevokeEndorsement` struct with `reason: String` field;
drop `Copy` from its derive stack (String isn't Copy); keep
`Default` (String defaults to ""; handler rejects empty-after-trim).
Add new `RevokeEndorsementResponse` struct with `endorsement_id`,
`revoked_at`, `liability_chain_severed_for_cases` fields.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api_common/src/governance.rs   # extend RevokeEndorsement; add RevokeEndorsementResponse
```

**IMPLEMENT (file 1 of 1):** in
`crates/api/api_common/src/governance.rs`, locate the existing
`RevokeEndorsement` struct (line ~307-313 at plan-write time).
Replace its derive line and struct body:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Revoke an existing endorsement. PRD §5.1 — required reason flows
/// through governance_log::append's scrub layer per ADR-015.
pub struct RevokeEndorsement {
  pub endorsement_id: EndorsementId,
  pub reason: String,
}
```

Note: the derive list dropped `Copy` (String isn't Copy); kept
`Default`, `Clone`, `PartialEq`, `Eq`, `Hash`.

Add `RevokeEndorsementResponse` immediately after:

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from revoking an endorsement. PRD §5.1.
/// `liability_chain_severed_for_cases` is non-empty when the
/// revocation severed one or more `SponsorLiabilityPending` cases'
/// grace windows (§5.3 step 4).
pub struct RevokeEndorsementResponse {
  pub endorsement_id: EndorsementId,
  pub revoked_at: DateTime<Utc>,
  pub liability_chain_severed_for_cases: Vec<ModerationCaseId>,
}
```

The `DateTime<Utc>` and `Vec<ModerationCaseId>` are non-Copy → the
response struct does NOT derive `Copy`.

**IMPORTS:** verify `DateTime`, `Utc`, `Vec` are already in scope
(or add via `use chrono::{DateTime, Utc};` if missing). The file
already imports `EndorsementId` + `ModerationCaseId` (line 18).

**MIRROR:** The `CreateEndorsement` + `CreateEndorsementResponse`
pair at lines 287-305 is the structural canonical mirror.

**GOTCHA (R3 — struct-extension grep sweep):** after the derive +
field change, `git grep -l 'RevokeEndorsement {'` to find any
literal-construction sites. Pre-SL-b expected: zero matches. If
matches appear (e.g. a test fixture), propagate
`..Default::default()` to compile.

**GOTCHA (ts-rs regen):** the ts-rs derive on the new struct
generates a TypeScript binding; the workspace-check workflow's
`cargo test --no-run -p lemmy_server --test e2e` step picks up
ts-rs validation failures.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry
capturing `workflow_run_id` of `cargo-validate-workspace.yml`.

**COMMIT MESSAGE:** `feat(v1-SL-b): extend RevokeEndorsement DTO with reason + add RevokeEndorsementResponse (task 1)`

### Task 2: Create revoke_endorsement.rs handler + module wiring

**ACTION:** in `crates/api/api_crud/src/governance/`, create new
file `revoke_endorsement.rs` with the outer handler +
`process_revocation` body per §10.1 + §10.2 patterns. In
`crates/api/api_crud/src/governance/mod.rs`, add `pub mod
revoke_endorsement;` alphabetically.

**FILES:**

```yaml
creates:
  - crates/api/api_crud/src/governance/revoke_endorsement.rs
modifies:
  - crates/api/api_crud/src/governance/mod.rs
```

**IMPLEMENT (file 1 of 2):** in
`crates/api/api_crud/src/governance/revoke_endorsement.rs`, write
the full handler body per §10.1 (outer handler) + §10.2
(`process_revocation`) + §10.4 (payload schemas) + §10.5 (reason
validation) + §10.7 (rate-limit + bypass).

Module doc-comment shape (mirror
`create_endorsement.rs:1-34`):

```rust
//! `POST /api/v4/governance/endorsement/revoke` — revoke a previously-
//! created endorsement; cascade revoke the matching surety; sever any
//! `SponsorLiabilityPending` grace-window chains for the sponsee per
//! the community-configured `liability.multi_sponsor_escape_rule`.
//!
//! Per PRD §5 + §12. Self-revocation by the sponsor OR admin
//! revocation; both flow through the same handler. Reason is
//! required (§12.2) and scrubbed via `governance_log::append`'s
//! redaction layer (ADR-015). Revocation rate-limit per §12.1
//! (default 5 / 24h, instance-scoped); admin caller bypasses with
//! `rate_limit_bypassed: true` annotation on the
//! `endorsement_revoked` log entry (per DQ #140).
//!
//! Every write (endorsement UPDATE, surety UPDATE, N case UPDATEs,
//! 2 snapshot recomputes, 2 log entries) runs inside one
//! `run_transaction` closure. Idempotency check is INSIDE the
//! transaction (TOCTOU avoidance per PRD §5.4 +
//! `feedback_multi_write_handlers_need_transactions.md`).
```

Use block (mirror `create_endorsement.rs:35-69`):

```rust
use actix_web::web::{Data, Json};
use chrono::{DateTime, Duration, Utc};
use diesel::{
  ExpressionMethods, NullableExpressionMethods, QueryDsl, SelectableHelper,
  dsl::count_star, update,
};
use diesel_async::{AsyncPgConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api::governance::{
  actor_pseudonym_helper,
  config::{self, ConfigCache, Scope},
  governance_log::{self, ENTRY_KIND_ENDORSEMENT_REVOKED, ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED},
  reputation_snapshot,
};
use lemmy_api_common::governance::{RevokeEndorsement, RevokeEndorsementResponse};
use lemmy_api_utils::{context::LemmyContext, utils::{check_local_user_valid, is_admin}};
use lemmy_db_schema::{
  newtypes::{ModerationCaseId, PersonId},
  source::governance::{endorsement::Endorsement, moderation_case::ModerationCase},
};
use lemmy_db_schema_file::{
  enums::CaseStatus,
  schema::{endorsement, moderation_case, surety},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
```

Outer handler body — verbatim per §10.1 (about 50 lines).
`process_revocation` body — verbatim per §10.2 (about 100 lines).

**IMPLEMENT (file 2 of 2):** in
`crates/api/api_crud/src/governance/mod.rs`:

```rust
pub mod create_endorsement;
pub mod create_report;
pub mod revoke_endorsement;   // NEW (alphabetical)
pub mod request_appeal;
```

**MIRROR:** §10.1, §10.2, §10.4, §10.5, §10.7.
`crates/api/api_crud/src/governance/create_endorsement.rs:1-361`
(canonical full-file mirror).
`crates/api/api/src/governance/admin_close_case.rs:23-105` (admin
gate + reason validation + governance_log append pattern).

**GOTCHA (TOCTOU on re-revoke):** the FOR UPDATE lock at step 1 is
load-bearing.

**GOTCHA (R1 — i64 typing):** `recent_count: i64` (Diesel
`count_star()` is `i64`); `rate_limit_per_day: i64` (config
`get_int` returns `i64`). Direct `>=` comparison; no `as` cast.

**GOTCHA (config Scope::Community on Option<CommunityId>):**
`case.community_id` is `Option<CommunityId>`. Match-pick the right
top-level scope.

**GOTCHA (ADR-013 exhaustive match):** the handler uses Diesel
`.filter(...)` not `match case.status` — exhaustive-match isn't
required at the row level. SL-b adds NO match site to the ADR-013
enum.

**GOTCHA (clippy rerun per `feedback_clippy_rerun_after_fix.md`):**
re-run clippy locally before push if dispatch refactor unmasks lint.

**GOTCHA (`run_transaction` async-closure scoping):** `data_for_tx`
+ `pseudonym_for_tx` are `move`'d into the async closure;
`is_admin_caller` + `bypass_recorded` + `caller_id` are `Copy` so
they cross the closure boundary as values.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry.

**COMMIT MESSAGE:** `feat(v1-SL-b): create revoke_endorsement handler + module wiring (task 2)`

### Task 3: Register `/endorsement/revoke` route + import handler

**ACTION:** in `crates/api/routes/src/lib.rs`, extend the
`governance::{...}` use block with `revoke_endorsement::revoke_endorsement,`
(line ~149) AND add the route registration
`.route("/endorsement/revoke", post().to(revoke_endorsement))`
immediately after the existing `/endorsement` line.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/routes/src/lib.rs   # use-block import + route registration
```

**IMPLEMENT (file 1 of 1):** in `crates/api/routes/src/lib.rs`:

1. **Use-block addition** (line ~149): insert
   `revoke_endorsement::revoke_endorsement,` after the existing
   `request_appeal::request_appeal,` line so the resulting block
   stays alphabetical.

2. **Route registration** (anchor: `grep -n '"/endorsement"' lib.rs`
   to find current line; expect ~523). Insert immediately after the
   matched line:
   ```rust
   .route("/endorsement/revoke", post().to(revoke_endorsement))
   ```
   Indentation matches the surrounding `.route(...)` lines.

**MIRROR:** §10.6.

**GOTCHA (route line shift):** `grep -n` finds the current line
dynamically. Document the actual line in the commit body if it
differs from 523.

**GOTCHA (handler symbol scope):** Task 3's worker branch is cut
from `phase-v1-SL-b` AFTER Task 2's finalize-merge. The
`revoke_endorsement::revoke_endorsement` symbol is in scope.

**GOTCHA (rate-limit middleware inheritance):** the route is inside
the `/governance` scope which wraps `rate_limit.post()`. The
`revoke_endorsement` handler inherits the post-rate-limit
middleware automatically.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry.

**COMMIT MESSAGE:** `feat(v1-SL-b): register POST /api/v4/governance/endorsement/revoke route (task 3)`

### Task 4: e2e test #1 — Self-revoke success path + mod v1_sl_b_fixtures shell

**ACTION:** in `crates/server/tests/e2e.rs`, append a NEW
`mod v1_sl_b_fixtures` block AFTER the existing `mod v1_jm_e_fixtures`
(closing `}` at line ~10975). The new mod opens with shared use
imports + helper fns + the first test
`revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # new mod v1_sl_b_fixtures + helpers + test #1
```

**IMPLEMENT (file 1 of 1):** anchor-Edit at end of file (line
10976+). Append:

```rust
mod v1_sl_b_fixtures {
  use super::*;
  // [imports per §10.3 fixture mod shape]

  async fn seed_endorsement_active(...) // helper
  async fn seed_pending_case(...)       // helper

  #[tokio::test]
  async fn revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots() -> Result<(), Box<dyn Error>> {
    // - Bootstrap context + 2 users (sponsor + sponsee)
    // - seed_endorsement_active(sponsor → sponsee)
    // - Direct INSERT a surety row matching (sponsor, sponsee, None) via SuretyInsertForm
    // - Pre-call: assert endorsement.revoked_at IS NULL + surety.revoked_at IS NULL
    // - Call revoke_endorsement(Json(RevokeEndorsement { endorsement_id, reason: "self-revoke test" }), context, sponsor_view).await?
    // - Assert response.endorsement_id matches; response.revoked_at recent (within last 5s); response.liability_chain_severed_for_cases.is_empty()
    // - Re-read endorsement: revoked_at IS Some(_)
    // - Re-read surety: revoked_at IS Some(_)
    // - Re-read reputation_snapshot for sponsor: assert non-stale (recompute fired)
    // - Re-read reputation_snapshot for sponsee: assert non-stale (recompute fired)
    // - Read governance_log: exactly one new entry of kind "endorsement_revoked"; payload contains sponsor_pseudonym, target_pseudonym, reason="self-revoke test"; rate_limit_bypassed field absent
    // - Assert NO entry of kind "sponsor_liability_escaped" (no pending case to sever)
    Ok(())
  }
}
```

**MIRROR:** §10.3 fixture mod shape;
`crates/server/tests/e2e.rs:9786+` `mod v1_jm_e_fixtures` (most
recent fixture-mod precedent);
`crates/server/tests/e2e.rs:2871-2886` `seed_surety` helper
pattern.

**GOTCHA (R4):** test fn name lowercase snake_case.

**GOTCHA (anchor placement):** the new mod opens AFTER the closing
`}` of `mod v1_jm_e_fixtures` at line 10975.

**GOTCHA (handler invocation):** the test calls
`revoke_endorsement(Json(RevokeEndorsement { ... }), context.clone(),
sponsor_view).await?`.

**GOTCHA (governance_log read):** post-call, query `governance_log`
table directly with diesel; assert count = 1 for
`"endorsement_revoked"`, count = 0 for
`"sponsor_liability_escaped"`.

**Push and exit (Shape G):** push to origin/`junior/<task-slug>`;
impl-task subagent writes `kind: "validate-pending"` DQ entry.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #1 — revoke_endorsement self-succeeds (task 4)`

### Task 5: e2e test #2 — Admin-revoke success path

**ACTION:** anchor-insert
`revoke_endorsement_admin_succeeds_under_threshold` test fn inside
`mod v1_sl_b_fixtures` AFTER Task 4's test fn closing `}`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # add test #2 inside mod v1_sl_b_fixtures
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 4's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_admin_succeeds_under_threshold() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 3 users: sponsor, sponsee, admin (admin: true)
  // - seed_endorsement_active(sponsor → sponsee); INSERT surety
  // - Call revoke_endorsement with admin_view (NOT sponsor_view); reason="admin policy intervention"
  // - Assert response.endorsement_id matches; revoked_at recent; severed empty
  // - Re-read endorsement + surety: revoked_at populated
  // - Read governance_log: 1 entry "endorsement_revoked" with sponsor_pseudonym = admin_pseudonym; target_pseudonym = sponsee_pseudonym; rate_limit_bypassed field ABSENT (under threshold per DQ #141)
  Ok(())
}
```

**MIRROR:** Task 4 + DQ #141 (separation rationale).

**GOTCHA (DQ #141):** test #2 exercises admin-as-caller WITHOUT
triggering rate-limit. Pre-seed: zero prior revocations by admin.
The `rate_limit_bypassed` field must be ABSENT.

**GOTCHA (admin flag):** the admin local_user is created via
`LocalUserInsertForm { admin: true, ... }` (per existing test
patterns at `crates/server/tests/e2e.rs:1700+`).

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #2 — admin revoke under threshold succeeds (task 5)`

### Task 6: e2e test #3 — Re-revoke idempotency

**ACTION:** anchor-insert
`revoke_endorsement_re_revoke_returns_existing_revoked_at_no_log_no_severance`
test fn inside `mod v1_sl_b_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 5's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_re_revoke_returns_existing_revoked_at_no_log_no_severance() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 2 users (sponsor + sponsee)
  // - seed_endorsement_active; INSERT surety
  // - First call: revoke_endorsement → succeeds; capture response.revoked_at as t1
  // - Read governance_log row count; capture as count_after_first = N
  // - Second call: revoke_endorsement (same endorsement_id; reason="second attempt") → succeeds
  // - Assert second response.endorsement_id matches; second response.revoked_at == t1 (idempotency)
  // - Assert second response.liability_chain_severed_for_cases.is_empty()
  // - Re-read governance_log row count; assert count == N (NO new log entry on re-revoke)
  // - Re-read endorsement: revoked_at == t1 (unchanged)
  Ok(())
}
```

**MIRROR:** §4 watchpoint #5 (idempotency guarantee).

**GOTCHA (timestamp comparison):** the second response's
`revoked_at` must equal `t1` exactly. The handler's step-1
idempotency early-return returns the existing value, not a fresh
`Utc::now()`.

**GOTCHA (no-log assertion):** the governance_log row count
unchanged check is the strong assertion that the early-return
skipped step 8's emission.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #3 — re-revoke idempotency (task 6)`

### Task 7: e2e test #4 — Capability rejection (non-sponsor non-admin)

**ACTION:** anchor-insert
`revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found`
test fn inside `mod v1_sl_b_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 6's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 3 users: sponsor, sponsee, third_party (non-sponsor non-admin)
  // - seed_endorsement_active(sponsor → sponsee); INSERT surety
  // - Call revoke_endorsement with third_party_view; reason="impersonation attempt"
  // - Assert resp.is_err()
  // - Assert matches!(err.error_type, LemmyErrorType::NotFound) — do NOT leak existence
  // - Re-read endorsement: revoked_at IS STILL NULL
  // - Re-read surety: revoked_at IS STILL NULL
  // - governance_log row count unchanged
  Ok(())
}
```

**GOTCHA (NotFound vs Unauthorized):** PRD §5.2 specifies
`NotFound` not `Unauthorized`. The handler's check is
`!is_admin_caller && row.from_person_id != caller_id ⇒ NotFound`.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #4 — capability rejection (task 7)`

### Task 8: e2e test #5 — Reason validation (empty / whitespace-only)

**ACTION:** anchor-insert
`revoke_endorsement_empty_reason_rejects` test fn inside
`mod v1_sl_b_fixtures`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 7's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_empty_reason_rejects() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 2 users (sponsor + sponsee)
  // - seed_endorsement_active; INSERT surety
  // - Three sub-cases (single test fn):
  //   (a) reason: "" (empty string)
  //   (b) reason: "   " (whitespace-only)
  //   (c) reason: "\t\n  " (mixed whitespace)
  //   For each: call revoke_endorsement; assert resp.is_err(); assert matches!(err.error_type, LemmyErrorType::Unknown(msg) if msg == "revoke-endorsement reason required")
  // - Re-read endorsement: revoked_at IS STILL NULL after all three attempts
  Ok(())
}
```

**MIRROR:** §10.5 reason validation pattern; DQ #139 resolution.

**GOTCHA (DQ #139):** the error message string is
`"revoke-endorsement reason required"` exactly. Match exactly.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #5 — empty reason rejection (task 8)`

### Task 9: e2e test #6 — Rate-limit enforcement + admin bypass

**ACTION:** anchor-insert
`revoke_endorsement_rate_limit_enforces_unless_admin_bypasses` test
fn. Combined positive (rate-limit enforces on non-admin
at-threshold) + negative (admin bypasses with
`rate_limit_bypassed: true` annotation).

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 8's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_rate_limit_enforces_unless_admin_bypasses() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 2 users: regular_caller, admin_caller (admin: true)
  // - For regular_caller: seed 5 endorsements with revoked_at = Some(now - 1h) (5 prior revocations)
  //                      seed 1 active endorsement (target = some_sponsee) — the 6th attempt
  // - Assert config: liability.revoke_rate_limit_per_day = 5 (default)
  // - Call revoke_endorsement (regular_caller_view, reason="6th attempt")
  // - Assert resp.is_err(); matches!(err.error_type, LemmyErrorType::RateLimitError)
  // - Re-read the active endorsement: revoked_at IS STILL NULL (rejected pre-tx)
  // - For admin_caller: same setup
  // - Call revoke_endorsement (admin_caller_view, reason="admin override")
  // - Assert resp.is_ok(); response.endorsement_id matches
  // - Read governance_log: 1 NEW entry kind "endorsement_revoked"; payload contains "rate_limit_bypassed": true (per DQ #140)
  Ok(())
}
```

**MIRROR:** §10.7 rate-limit + bypass; DQ #140 + DQ #141 resolutions.

**GOTCHA (DQ #141):** test #6 distinct from test #2 — test #2 was
admin under threshold; test #6 is admin bypass at threshold.

**GOTCHA (DQ #140):** `rate_limit_bypassed: true` field present in
admin's bypass log entry; absent in non-admin under-threshold and
regular_caller's rejection (no entry at all on rejection).

**GOTCHA (seed prior revocations):** the 5 prior revocations seed
via direct INSERT with `revoked_at = Some(now() -
Duration::hours(1))`.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #6 — rate-limit + admin bypass (task 9)`

### Task 10: e2e test #7 — Grace-window severance, single sponsor

**ACTION:** anchor-insert
`revoke_endorsement_severs_grace_window_single_sponsor_case` test
fn.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 9's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_severs_grace_window_single_sponsor_case() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 2 users (sponsor + sponsee)
  // - seed_endorsement_active(sponsor → sponsee); INSERT surety (sponsor, sponsee, None)
  // - seed_pending_case(sponsee, community=None, grace_hours=24)
  //   → status: SponsorLiabilityPending, target_person_id: Some(sponsee), grace_expires_at: Some(now + 24h), liability_escape_reason: None
  // - Pre-call: assert case.status == SponsorLiabilityPending; case.liability_escape_reason IS NULL
  // - Call revoke_endorsement(sponsor_view, reason="prudent withdrawal")
  // - Assert response.liability_chain_severed_for_cases == vec![case_id]
  // - Re-read case: status == SponsorLiabilityEscaped; liability_escape_reason IS Some(json) where:
  //   * json["version"] == 1
  //   * json["reason"] == "sponsor_revoked"
  //   * json["actor_pseudonym"] is a String (NOT raw caller_id)
  //   * json["endorsement_id"] == endorsement_id.0
  //   * json["actor_pseudonym"] does NOT equal format!("{}", sponsor_id.0) (defensive ADR-015)
  // - Read governance_log: 1 entry "endorsement_revoked" + 1 entry "sponsor_liability_escaped"
  Ok(())
}
```

**MIRROR:** §4 watchpoint #4 (`liability_escape_reason` JSON
schema); §10.2 step 6 (escape branch).

**GOTCHA (ADR-015 actor_pseudonym):** the `actor_pseudonym` field
in the JSONB MUST be a string (the pseudonym), NOT a number (raw
person_id). Defensive: assert string does not equal
`format!("{}", sponsor_id.0)`.

**GOTCHA (severed vec contents):** `liability_chain_severed_for_cases`
contains `ModerationCaseId` values; the response Vec serialises as
`[<case_id_int>]`.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #7 — single-sponsor grace-window severance (task 10)`

### Task 11: e2e test #8 — Multi-sponsor `any_revocation` rule (default)

**ACTION:** anchor-insert
`revoke_endorsement_multi_sponsor_any_revocation_severs_chain` test
fn.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 10's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_multi_sponsor_any_revocation_severs_chain() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 4 users: sponsee, sponsor_a, sponsor_b, sponsor_c
  // - For each sponsor: seed_endorsement_active(sponsor_X → sponsee); INSERT surety (sponsor_X, sponsee, None)
  // - All 3 sureties present + active for sponsee
  // - seed_pending_case(sponsee, community=None, grace_hours=24)
  // - Confirm config: liability.multi_sponsor_escape_rule defaults to "any_revocation"
  // - Call revoke_endorsement(sponsor_a_view, reason="any-rev test")
  // - Assert response.liability_chain_severed_for_cases == vec![case_id]
  // - Re-read case: status == SponsorLiabilityEscaped
  // - Re-read sureties:
  //   * sponsor_a's surety: revoked_at IS Some(_)
  //   * sponsor_b's surety: revoked_at IS STILL NULL (only the revoking sponsor's surety flipped)
  //   * sponsor_c's surety: revoked_at IS STILL NULL
  // - governance_log: 1 entry "endorsement_revoked" + 1 entry "sponsor_liability_escaped"
  Ok(())
}
```

**MIRROR:** §4 watchpoint #2 (step ordering); PRD §13 OQ-V1-SL-01.

**GOTCHA (only revoking sponsor's surety):** the `surety` UPDATE at
step 4 matches `(sponsor_id = caller_id, sponsored_id =
endorsement.to_person_id, community_id = endorsement.community_id)`.

**GOTCHA (escape rule resolution):** with no community on the case
(community_id=None), handler falls through to `Scope::Instance`.
Default is `"any_revocation"` per SL-a Task 6 seed. No config row
needed.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #8 — multi-sponsor any_revocation severance (task 11)`

### Task 12: e2e test #9 — Grace-window non-severance (no matching pending case)

**ACTION:** anchor-insert
`revoke_endorsement_no_pending_case_no_severance_only_revoked_log`
test fn.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
```

**IMPLEMENT (file 1 of 1):** anchor-Edit AFTER Task 11's test fn:

```rust
#[tokio::test]
async fn revoke_endorsement_no_pending_case_no_severance_only_revoked_log() -> Result<(), Box<dyn Error>> {
  // - Bootstrap; 2 users (sponsor + sponsee)
  // - seed_endorsement_active; INSERT surety
  // - DO NOT seed any moderation_case row
  // - Call revoke_endorsement(sponsor_view, reason="standard withdrawal")
  // - Assert response.endorsement_id matches; revoked_at recent
  // - Assert response.liability_chain_severed_for_cases.is_empty()
  // - Re-read endorsement + surety: revoked_at populated
  // - Read governance_log: 1 entry "endorsement_revoked"; payload's "liability_chain_severed_for_cases" is []; NO entry of kind "sponsor_liability_escaped"
  // - Assert no moderation_case rows for this sponsee (defensive)
  Ok(())
}
```

**MIRROR:** §4 watchpoint #5 (severed-cases populated only on
actual severance).

**GOTCHA (empty-vec serialisation):** the JSON payload's
`liability_chain_severed_for_cases` field serialises as `[]` when
empty (Vec<T> always serialises even when empty).

**GOTCHA (no-pending-case path):** handler's step-6 query loads
zero rows; the for-loop is a no-op; severed stays `vec![]`. Step 8
fires `endorsement_revoked` with empty severed array. Step 7 still
fires both snapshot recomputes.

**Push and exit (Shape G):** push; write `kind: "validate-pending"`.

**COMMIT MESSAGE:** `test(v1-SL-b): e2e test #9 — non-severance no-pending-case (task 12)`

### Task 13: Retro

**Goal:** author retro per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md` +
`feedback_retro_task_complexity_score.md`. Flip
`(pending) → (active)` markers on registry §"v1-SL-a entry kinds"
for SL-b's fire-sites: `_ENDORSEMENT_REVOKED` row entirely;
`_SPONSOR_LIABILITY_ESCAPED` row's SL-b portion (the SL-c portion
stays `(pending)`).

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-SL-b-retro.md
modifies:
  - .claude/rules/governance-log-entry-kind-registry.md
```

**IMPLEMENT (file 1 of 2):** in
`.claude/PRPs/reports/v1-SL-b-retro.md`, write the retro per the
canonical 4-role format (Advisor / Planning / Impl / BM). Include:

- §1 Summary: SL-b shipped (DTO + handler + route + 9 e2e tests +
  retro). Story status — Story 1 ✓ / Story 2 ✓ / Story 3 ✓.
- §2 Per-role signals (4 H2 sections; each lists "what worked" +
  "what surprised us" + "what should change next").
- §3 Carry-forward — list of items SL-c/SL-d/SL-e should know.
- §4 Per-task complexity-score table (mandatory per
  `feedback_retro_task_complexity_score.md`):
  `| task | files-changed | commits | runtime-min | max-log-silence-min |`
  for each of Tasks 0..13.
- §5 Lessons promotion — any new `feedback_*.md` candidates.
- §6 Acceptance — confirm all checkboxes from §17.

**IMPLEMENT (file 2 of 2):** in
`.claude/rules/governance-log-entry-kind-registry.md`, locate the
"v1-SL-a entry kinds (5, this sub-phase)" section. Edit the
"Emitting handler" column for two rows:

- `ENTRY_KIND_ENDORSEMENT_REVOKED`: change "v1-SL-b
  `crates/api/api_crud/src/governance/revoke_endorsement.rs`
  (pending)" to "(active)" — flipped 2026-MM-DD post-SL-b.
- `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`: change the v1-SL-b portion
  to "(active)". The v1-SL-c portion stays `(pending)`.

Total count `38` stays unchanged.

**Cross-cutting verification (Task 13 retro time — invariants):**

- `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs` returns
  `38` (unchanged).
- `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs | wc -l` returns
  `38`.
- `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  ENDORSEMENT_REVOKED` returns empty.
- `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_ESCAPED` returns 1 line (SL-c portion only).
- `cargo test -p lemmy_server --test e2e -- v1_sl_b_fixtures`
  passes.
- `cargo build` workspace exit 0 via cargo-validate-workspace.yml on
  the phase-branch tip.
- `/brehon-verify` reports all three §16a stories `[done]`.

**MIRROR:** `.claude/PRPs/reports/v1-SL-a-retro.md` (predecessor);
`.claude/PRPs/reports/v1-JM-e-retro.md` for section structure.

**GOTCHA (retro before PR):** retro is written BEFORE `gh pr
create` per `feedback_retro_not_report.md`.

**GOTCHA:** §4 per-task complexity-score table is mandatory.

**Push and exit (Shape G — retro is meta-work; no `crates/**`
change → `cargo-validate-workspace` does not trigger).**

**COMMIT MESSAGE:** `docs(v1-SL-b): phase retrospective + flip ENTRY_KIND_* registry markers (task 13)`

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full`
  via `cargo-validate-workspace.yml:88`.
- **Lint:** `cargo clippy --workspace --features full --no-deps --
  -D warnings` via `cargo-validate-workspace.yml:92` (R6).
- **Test target compile (R7):** `cargo test --no-run -p lemmy_server
  --test e2e` via `cargo-validate-workspace.yml:95`. Triggers on
  Tasks 1, 2 + Tasks 4-12.
- **Migration round-trip:** N/A — SL-b ships zero migrations.
- **e2e execution (Phase 2):** the 9 new tests run via Phase 2 e2e
  user gate: (a) local on laptop or (b) GH dispatch via
  `cargo-test-e2e.yml`.

Pre-merge advisor-side verification: Story 1 checkpoint is
workspace-check workflow `conclusion: "success"` on Task 3's push;
Stories 2-3 checkpoints are Phase 2 e2e exit 0; `/brehon-verify`
Brief-Scope outputs check.

### 14.1 Pre-existing tests preserved

- All SL-a-shipped tests — preserved verbatim
- All JM-e-shipped tests (`mod v1_jm_e_fixtures::*`) — preserved
- All JM-c/JM-b-shipped tests — preserved
- `governance_log_hash_chain_holds` at e2e.rs:277 — preserved
- All endorsement-creation tests (Phase 5b) — preserved

---

## 15. Validation commands (DoD)

> **Shape G plan — DoD is per-workflow, not inline cargo.** Per
> `.claude/PRPs/templates/plan.template.md` §15.6. SL-b ships zero
> migrations, so `cargo-validate-migration.yml` does not fire.

### 15.1 Per-task workspace check (Shape G)

For every §13 impl task that modifies `crates/**` (Tasks 1, 2, 3, 4,
5, 6, 7, 8, 9, 10, 11, 12):

- **DoD entry:** `cargo-validate-workspace.yml` on `junior/<task-slug>`
  SHA `<sha>` → `conclusion: "success"`
- **Validation command:** `gh run list --repo barrie-cork/lemmy
  --branch <branch> --workflow cargo-validate-workspace --limit 1
  --json conclusion,databaseId --jq '.[0]'`
- **EXPECT:** `{"conclusion": "success", "databaseId": <id>}`

The workflow runs `cargo check --workspace --features full`,
`cargo clippy --workspace --features full --no-deps -- -D warnings`,
and `cargo test --no-run -p lemmy_server --test e2e` per
`.github/workflows/cargo-validate-workspace.yml:88-95`. R6 + R7 are
encoded.

### 15.2 Migration round-trip — N/A

SL-b ships zero migrations; `cargo-validate-migration.yml` path
filter `migrations/**` excludes SL-b commits. No DoD entry.

### 15.3 Phase 2 e2e (post-finalize-merge of last impl task)

After Task 12's worker branch finalize-merges into `phase-v1-SL-b`,
the advisor surfaces the **Phase 2 e2e local-vs-dispatch user gate**
per `advisor-orchestrator.md`:

- **(a) local:** `cargo test -p lemmy_server --test e2e --features
  full -- --test-threads=1` on laptop in `run_in_background`; ~26 min
  wall-clock; zero billed.
- **(b) dispatch:** `gh workflow run cargo-test-e2e.yml --repo
  barrie-cork/lemmy --ref phase-v1-SL-b`; ci-watcher polls; ~26 min
  billed.

Plan-side DoD: e2e exit code 0 with all 9 SL-b tests passing;
failure path → §G4 classifier on log slice.

### 15.4 Cross-cutting verification (Task 13 retro time)

- [ ] `rg -c '^pub const ENTRY_KIND_'
  crates/db_schema/src/source/governance/governance_log.rs` returns
  **38** (unchanged from SL-a end-state).
- [ ] `rg '^\s+ENTRY_KIND_'
  crates/api/api/src/governance/governance_log.rs | wc -l` returns
  **38**.
- [ ] `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  ENDORSEMENT_REVOKED` returns empty.
- [ ] `rg -n '\(pending\)'
  .claude/rules/governance-log-entry-kind-registry.md | rg
  SPONSOR_LIABILITY_ESCAPED` returns exactly 1 line (SL-c portion).
- [ ] `rg -n 'pub mod revoke_endorsement;'
  crates/api/api_crud/src/governance/mod.rs` returns 1 line.
- [ ] `rg -n '"/endorsement/revoke"' crates/api/routes/src/lib.rs`
  returns 1 line.
- [ ] `rg -n 'pub struct RevokeEndorsementResponse'
  crates/api/api_common/src/governance.rs` returns 1 line.
- [ ] `rg -c 'async fn revoke_endorsement_'
  crates/server/tests/e2e.rs` returns 9.
- [ ] `rg -n 'mod v1_sl_b_fixtures'
  crates/server/tests/e2e.rs` returns 1 line.
- [ ] R1: every `i32 ↔ i64` comparison in Tasks 2 + 4-12 uses
  `i64::from(...)` if any cross-type comparison arises.
- [ ] R6: clippy invocations in `cargo-validate-workspace.yml` use
  `--no-deps -- -D warnings`.
- [ ] No edits to files outside §11 list.
- [ ] Every SL-b §16a story is `[done]`.

### 15.5 ADR / OQ compliance verification

- [ ] **ADR-010** honoured — re-revocation idempotency returns
  existing `revoked_at` (no retroactive invalidation).
- [ ] **ADR-013** honoured — exhaustive `CaseStatus` match preserved
  (SL-b adds NO new match site).
- [ ] **ADR-014** honoured — no federation outbound on
  `endorsement_revoked` or `sponsor_liability_escaped`.
- [ ] **ADR-015** honoured — `liability_escape_reason` JSONB carries
  `actor_pseudonym` (string), NOT raw `caller_id`.
- [ ] **OQ-V1-SL-05** honoured — `liability_escape_reason` JSON
  carries `version: 1` from day one.
- [ ] **OQ-V1-SL-01** honoured — multi-sponsor escape rule defaults
  to `any_revocation`; community/instance configurable.
- [ ] **PRD §2 OUT** honoured — no scheduler module / no
  `submit_jury_vote` mutation / no `restoration_complete` endpoint
  / no `step_up_token` field.
- [ ] **PRD §12.3 v2 reservation** honoured — no step-up auth
  enforcement; DTO has no `step_up_token` field.

### 15.6 DoD per workflow (canonical Shape G shape)

**Phase 1 (workspace check):**

- Workflow: `.github/workflows/cargo-validate-workspace.yml`
- Branch (per task): `junior/<task-slug>`
- Expected `conclusion`: `"success"`

**Phase 1b (migration round-trip):** N/A — no migrations in SL-b.

**Phase 2 (e2e):**

- Workflow: `.github/workflows/cargo-test-e2e.yml` (only on user
  dispatch per PR #105)
- OR local: `cargo test -p lemmy_server --test e2e --features full
  -- --test-threads=1` on laptop
- Branch: `phase-v1-SL-b`
- Expected: all tests pass

### 15.7 Manual validation snippets (advisor-side smoke only — non-binding)

> Per `feedback_features_full_p_crate_incompatible.md`: never `-p
> <crate>` + `--features full`; use `--workspace --features full`.

> Per `feedback_e2e_filter_assumes_naming.md`: SL-b tests all start
> with `revoke_endorsement_` — confirm via grep before filter.

```bash
ls .github/workflows/cargo-validate-workspace.yml
ls .github/workflows/cargo-test-e2e.yml

gh run list --repo barrie-cork/lemmy \
  --branch phase-v1-SL-b \
  --workflow cargo-validate-workspace \
  --limit 1 --json conclusion,databaseId

cargo check --workspace --features full
echo "exit: $?"

rg 'async fn revoke_endorsement_' crates/server/tests/e2e.rs | wc -l
# EXPECT: 9
```

These are advisor-side only; not §16 acceptance criteria. Per DQ
#67 resolution.

---

## 16. Acceptance criteria

- [ ] All 14 tasks (Task 0..12 + Task 13 retro) committed.
- [ ] §15.1 (`cargo-validate-workspace.yml`) `conclusion: "success"`
  after every impl task push (Tasks 1-12).
- [ ] §15.2 (migration round-trip) — N/A (zero migrations).
- [ ] §15.3 (Phase 2 e2e — local or dispatch) all tests pass; the 9
  new `v1_sl_b_fixtures::*` tests green.
- [ ] §15.4 (cross-cutting verification — 12 boxes) all ticked.
- [ ] §15.5 (ADR / OQ compliance — 8 boxes) all ticked.
- [ ] §16a stories — all three `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per Task 13.
- [ ] PR opens against `governance-v0` (NOT `main`) with
  `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-b-verify.md` shows all stories ✓.

---

## 16a. Stories (independently-testable behaviour units)

Per `.claude/PRPs/templates/plan.template.md` §16a +
`feedback_story_grain_checkpoint.md` +
`feedback_brehon_verify_pre_merge.md`. Three stories — one per
behaviourally distinct unit.

### Story 1: DTO extension + handler module + route registration compile clean and the new route resolves

- **Composing tasks:** Tasks 1, 2, 3
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 3's worker branch SHA → `conclusion: "success"`
- **Expected output (workspace-check):** all three jobs pass
- **Brief-Scope outputs to verify (used by `/brehon-verify`):**
  - `crates/api/api_common/src/governance.rs` contains
    `pub struct RevokeEndorsement` declaration with `pub reason:
    String` field AND derive line that does NOT contain `Copy`.
  - `crates/api/api_common/src/governance.rs` contains
    `pub struct RevokeEndorsementResponse` declaration with
    `pub endorsement_id: EndorsementId`,
    `pub revoked_at: DateTime<Utc>`,
    `pub liability_chain_severed_for_cases: Vec<ModerationCaseId>`
    fields.
  - `crates/api/api_crud/src/governance/revoke_endorsement.rs`
    exists; contains `pub async fn revoke_endorsement(` declaration;
    contains `async fn process_revocation(` private fn.
  - `crates/api/api_crud/src/governance/revoke_endorsement.rs`
    references `ENTRY_KIND_ENDORSEMENT_REVOKED` AND
    `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED` AND
    `actor_pseudonym_helper::get_or_create` AND
    `reputation_snapshot::recompute_snapshot` AND
    `governance_log::append`.
  - `crates/api/api_crud/src/governance/mod.rs` contains
    `pub mod revoke_endorsement;` line.
  - `crates/api/routes/src/lib.rs` contains
    `revoke_endorsement::revoke_endorsement,` in the `governance::{
    ... }` use block.
  - `crates/api/routes/src/lib.rs` contains
    `.route("/endorsement/revoke", post().to(revoke_endorsement))`
    inside the `scope("/governance")` block.

### Story 2: Six PRD §5/§12 contract guards (capability + reason + rate-limit + admin bypass + idempotency + e2e behaviour proof)

- **Composing tasks:** Tasks 4, 5, 6, 7, 8, 9
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 9's worker branch SHA → `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for the 6 tests →
  all pass.
- **Expected output (local Phase 2):** `6 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `mod v1_sl_b_fixtures`
    block.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_admin_succeeds_under_threshold(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_re_revoke_returns_existing_revoked_at_no_log_no_severance(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_empty_reason_rejects(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_rate_limit_enforces_unless_admin_bypasses(`.
  - Test bodies cite `LemmyErrorType::NotFound` (Task 7) AND
    `LemmyErrorType::Unknown` (Task 8) AND
    `LemmyErrorType::RateLimitError` (Task 9).
  - Task 9 test asserts `payload["rate_limit_bypassed"] == true` for
    admin bypass case AND absent for non-admin under-threshold.

### Story 3: Grace-window severance branches drive `SponsorLiabilityPending → SponsorLiabilityEscaped` correctly with PRD §8.1 schema

- **Composing tasks:** Tasks 10, 11, 12
- **Checkpoint workflow (Phase 1):** `cargo-validate-workspace.yml`
  on Task 12's worker branch SHA → `conclusion: "success"`.
- **Checkpoint workflow (Phase 2):** Phase 2 e2e for the 3 severance
  tests → all pass.
- **Expected output (local Phase 2):** `3 passed; 0 failed`.
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_severs_grace_window_single_sponsor_case(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_multi_sponsor_any_revocation_severs_chain(`.
  - `crates/server/tests/e2e.rs` contains
    `async fn revoke_endorsement_no_pending_case_no_severance_only_revoked_log(`.
  - Task 10 test body asserts `case.status ==
    CaseStatus::SponsorLiabilityEscaped` AND
    `case.liability_escape_reason["version"] == 1` AND
    `case.liability_escape_reason["reason"] == "sponsor_revoked"`
    AND `case.liability_escape_reason["actor_pseudonym"].is_string()`.
  - Task 11 test body asserts only the revoking sponsor's surety
    flips `revoked_at`; sponsor_b's and sponsor_c's sureties stay
    NULL.
  - Task 12 test body asserts
    `response.liability_chain_severed_for_cases.is_empty()` AND
    governance_log row count delta = +1 (only `endorsement_revoked`).

> **Verification mapping:** `/brehon-verify` iterates this section,
> runs each Story's checkpoint workflow on the worktree branch, and
> confirms each Brief-Scope output exists + matches its structural
> pattern.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0..12 confirmed).
- [ ] Tasks 1..12 committed.
- [ ] Task 13 retro committed.
- [ ] §15 validation green at every gate (Phase 1 workspace check
  ×12, Phase 2 e2e ×1).
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete with findings triaged per
  `feedback_pr_review_triage_pattern.md`.
- [ ] `/brehon-verify` report at
  `.claude/PRPs/reports/v1-SL-b-verify.md` shows all stories ✓.
- [ ] Post-merge phase branch retained for retro reads.
- [ ] DQ #143 (split-or-proceed) resolved by advisor with
  proceed-as-one rationale.
- [ ] Registry markers flipped (`_ENDORSEMENT_REVOKED` row entirely;
  `_SPONSOR_LIABILITY_ESCAPED` row's SL-b portion).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Complexity score 38 rejected by advisor (split-mandated) | HIGH | LOW | §5.2 — DQ #143 filed; if split, this plan becomes `v1-SL-b-1.plan.md` (Tasks 1-9, score ~26) and `v1-SL-b-2.plan.md` (Tasks 10-12 + retro, score ~10). JM-e + SL-a both shipped proceed-as-one above threshold without operational regret |
| Junior worker-hang on e2e.rs Edits (file is 10976 lines) | MED | HIGH | Per `feedback_junior_worker_e2e_edit_hang.md`: each test is its own §13 task; per-task anchor-Edit appends inside `mod v1_sl_b_fixtures`; no bulk Edit. JM-e shipped 4 e2e tests as 4 tasks cleanly at 9786-line file size |
| Task 3 (route) dispatches before Task 2 (handler) finalize-merges; compile fails on `unresolved import revoke_endorsement::revoke_endorsement` | LOW | MED | Tasks 1-12 dispatch serially (no `[P]` cohort); per `advisor-orchestrator.md` cohort-dispatch rule, advisor waits for prior task finalize-merge before next dispatch |
| `RevokeEndorsement` DTO derive change unmasks a test fixture that constructs a literal | LOW | LOW | Task 1 GOTCHA — R3 sweep `git grep -l 'RevokeEndorsement {'` runs at task-start; expected zero matches |
| Route line shift (e.g. another v1 sub-phase added a route between SL-a merge and SL-b plan-write) | LOW | LOW | Task 3 anchor-grep finds current line dynamically |
| Per-case ConfigCache thrash on cases across distinct communities | LOW | LOW | Per DQ #142 + `config.rs:209-256` — ConfigCache keys on `(scope_repr, key)`; thrash bounded by N-distinct-communities reads |
| `liability_escape_reason` JSONB schema regression (raw `caller_id` instead of `actor_pseudonym`) | LOW | HIGH | §4 watchpoint #4 + Task 10 test's defensive ADR-015 assertion catches |
| TOCTOU on re-revoke (concurrent calls bypass idempotency) | LOW | HIGH | §4 watchpoint #1 — FOR UPDATE lock + idempotency early-return inside same tx; Task 6 test asserts second response's `revoked_at == t1` |
| Step ordering bug — surety revoke AFTER active-sponsor re-query yields stale severance results | LOW | HIGH | §4 watchpoint #2 + Task 11 test asserts only the revoking sponsor's surety flipped |
| Rate-limit query against revoked_at NULL semantics | LOW | MED | Task 9 seeds 5 prior revocations with explicit `revoked_at = Some(now - 1h)`; the `gt(cutoff)` filter only matches non-NULL revoked_at within window |
| `rate_limit_bypassed: true` field missing on admin bypass log entry | LOW | MED | DQ #140 + Task 9 test asserts payload field present on admin bypass case + absent on non-admin under-threshold case |
| ts-rs derive regression (export breaks downstream client codegen) | LOW | LOW | `cargo test --no-run -p lemmy_server --test e2e` step in workspace-check workflow catches ts-rs validation; Task 1 + 2 push triggers it |
| GH-Actions minutes budget exceeded by 12 Phase-1 + 1 Phase-2 e2e | LOW | LOW | Phase 2 e2e local-default per JM-d retro §5; user picks dispatch only on audit-trail need. Total ~62 min monthly burn well within Free 3000 min |
| PMD #126 — DQ ID collision from concurrent SL-b / rep-tuning-r3 work | LOW | LOW | next-id calc spans archives + live; advisor monitors at finalize-merge. SL-b plan-write computed next_id = 143 against live `decision-queue.json` at plan-write time |
| Junior worker pre-pushes break finalize-merge | LOW | LOW | Junior daemon's finalize step is post-Shape-G correct (per JM-d retro §1.1) |
| Test pre-seed `SponsorLiabilityPending` cases via direct DB-write inadvertently break v0 cases | LOW | LOW | Tests insert NEW rows via `ModerationCaseInsertForm`; no UPDATE on existing rows. Each test bootstraps a fresh Postgres container per testcontainers-rs |
| ADR-013 enum-exhaustiveness lint trips on a new match site SL-b accidentally introduces | LOW | LOW | §4 watchpoint #7 — SL-b uses Diesel `.filter(status.eq(SponsorLiabilityPending))`, never `match case.status` |

---

## 19. Notes

### 19.1 Planner DQs filed

- **DQ #143** (`from: "planner"`, `kind: "blocker"`, `answered_by:
  null`) — complexity-score-split decision per §5.2. Question:
  "Complexity score 38 exceeds 8 — split `v1-sponsor-liability-b`
  into `v1-SL-b-1` (Tasks 1-9: DTO + handler + route + tests #1-6,
  score ~26) + `v1-SL-b-2` (Tasks 10-12 + retro: severance tests
  #7-9, score ~10), or proceed as one plan?". Options: split /
  proceed. Planner observation favours proceed (mechanical handler
  + tests work; each e2e test anchor-Edit-friendly; SL-a + JM-e
  proceed-as-one precedents; PRD §15 row 2 names the deliverable as
  one atomic phase; both halves still score above 8, so split's
  mitigation value is limited).

### 19.2 Self-resolved planner findings (LESSON candidates)

- **The brief's pre-estimate (6-10) was significantly mathematically
  optimistic.** The actual mechanical score per
  `feedback_complexity_score_pre_split.md` factor table is 38. The
  factor weights (e2e edits +3 each, applied to 9 tests) far
  outweigh the brief's "edit-weight 0.5" framing. Recommendation:
  brief-template pre-estimate convention should be dropped in
  favour of "planner computes from §5.1 breakdown table" per DQ
  #138 resolution. Aligns with SL-a §19.2 self-finding.
- **The 9-test-per-task discipline (per
  `feedback_junior_worker_e2e_edit_hang.md`) inflates §5 e2e factor
  count proportionally to test count.** Future SL-c/SL-d/SL-e plans
  with similar test-heavy lanes will likely score in the 30-50
  range. The complexity-score gate becomes a near-certain "trip"
  for any test-heavy phase. Recommendation: consider re-calibrating
  e2e-edit weight (e.g. +1 per cluster of 3 tests) OR accept that
  the gate's purpose is to surface "split or proceed" and that
  test-heavy phases routinely answer proceed.
- **No new ENTRY_KIND consts or schema = clean SL-b shape.** Unlike
  SL-a, SL-b is purely handler+tests. Score composition is
  dominated by impl-task multiplicity (12 tasks + e2e factor)
  rather than cross-cutting structural changes.

### 19.3 Pre-existing pending DQ entries

- **None at planning time.** `decision-queue.json` shows 0 pending
  entries on governance-v0 @ `080fcd7b7`. Recently resolved: DQ
  #138-#142 (SL-b clarify pass).

### 19.4 Out-of-scope follow-ups (Task 13 retro candidates)

- **Scheduler module `sponsor_liability_grace.rs`** — SL-c.
- **`apply_sponsor_liability` compute/fire split** — SL-d.
- **`submit_jury_vote` mutation: `Decided →
  SponsorLiabilityPending` transition** — SL-d.
- **`restoration_complete` endpoint** — restorative-mechanics-v1
  PRD.
- **e2e behavioural lane tests** (full revocation-during-window-
  escapes through SL-c + SL-d) — SL-e.
- **Sponsor notification UX** — PRD §13 OQ-V1-SL-03; gated on
  notification surface generalised by jury-mechanics-v1 OQ-005.
- **Step-up auth enforcement on `RevokeEndorsement.step_up_token`**
  — v2.
- **Cross-instance sponsor-liability federation** — v2 per ADR-014.

### 19.5 Confidence bands

- **High (9/10):** handler shape — exact mirror of
  `create_endorsement.rs:1-361` with 3 deltas.
- **High (9/10):** DTO extension — straightforward derive-stack
  amendment + new sibling response struct.
- **High (9/10):** route registration — single-line addition.
- **High (8/10):** 9 e2e tests — each is anchor-Edit-friendly.
- **Moderate (7/10):** §5 complexity score 38 will trip split-DQ;
  plan ships under proceed-as-one assumption pending DQ #143
  resolution.
- **Moderate (6/10):** rate-limit + admin bypass log payload
  semantics (DQ #140) — straightforward but novel.
- **High (8/10):** ADR-015 pseudonym discipline — defensive Task 10
  assertion catches regression.

### 19.6 Why no clarify DQ at impl time

Brief §4.2 enumerates the boundary-of-judgment cases that should
trigger an impl-side DQ. None apply to this plan as written:

- Existing `RevokeEndorsement` DTO is at `governance.rs:308-313`
  with derive `Copy + Default` (verified at plan-write time).
- `liability.multi_sponsor_escape_rule` default at
  `config.rs:937` is `"any_revocation"`.
- `liability.revoke_rate_limit_per_day` default at `config.rs:940`
  is `5`.
- `_ENDORSEMENT_REVOKED` (line 195) and
  `_SPONSOR_LIABILITY_ESCAPED` (line 197) ENTRY_KIND consts
  declared in SL-a.
- Shim re-exports at `crates/api/api/src/governance/governance_log.rs`
  lines 51 + 74 are present.
- Route at `lib.rs:523` is the current `/endorsement` registration.
- Canonical mirror `create_endorsement.rs:1-361` is intact.

If any of these baseline assumptions changes between plan-write and
impl-time, the impl-task subagent files a DQ pending entry.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — patterns mirror
  `create_endorsement.rs` directly + §10 §13 §16a all match the JM-e
  Shape-G shape; the DTO derive amendment + handler + route + 9
  tests are mechanical with strong precedents.
- **Cargo budget:** 10/10 — Shape G (cargo runs off-box; budget
  non-binding; forbidden-window non-binding).
- **Test coverage:** 8/10 — 9 behaviourally distinct tests covering
  the 6 PRD §5/§12 contract guards + 3 grace-window severance
  branches. Multi-sponsor `all_revocation` / `majority_revocation`
  rules are not exercised (default `any_revocation` is; the others
  are config-overridable per community and are tested at SL-c when
  the scheduler exercises them across more case shapes).
- **Story-grain decomposition:** 9/10 — every task maps to exactly
  one story; all three stories have concrete checkpoint workflows
  + grep-verifiable Brief-Scope outputs.
- **Plan-shape conformance:** 9/10 — 20-section schema followed
  literally; §15.6 Shape G shape adopted; §16a mandatory present;
  §13 per-task FILES YAML block on every task; §5.1 complexity
  breakdown table mechanical.

---

_Plan author: planning subagent (Junior `sl-b-planning-1`,
2026-05-04). Plan committed on
`junior/role-planning-v1-sponsor-liability-b-plan-see-claude-prps-briefs-sl-b-planning-1-md-113`;
finalize step pushes to that branch; advisor session merges via the
standard sub-phase flow into `governance-v0` where SL-b's BM-task
then cuts `phase-v1-SL-b`. One planner DQ raised at commit time:
DQ #143 (split-or-proceed; advisor blocking). Confidence 8/10. Plan
ships under proceed-as-one assumption pending DQ #143 resolution._

LESSON: handler+test sub-phases (the "single new handler with 9
behavioural tests" shape) score 30-40 on the
`feedback_complexity_score_pre_split.md` factor table because the
e2e-edit weight (+3 each) compounds with the
`feedback_junior_worker_e2e_edit_hang.md` mandate (one anchor-Edit
per test, so test-count = §13-task-count for the test cluster).
The score gate as currently calibrated will trip on essentially
every test-heavy v1 sub-phase. Two paths: (a) re-calibrate e2e-edit
weight (e.g. +1 per cluster of 3 tests, capped at +6 for the
cluster); or (b) accept that the gate's purpose is to surface
"split-or-proceed" as an explicit decision per
`feedback_principles_not_rules.md`, and that test-heavy phases
routinely answer proceed citing JM-e/SL-a precedents. Future
sub-phase planners (SL-c, SL-d, SL-e, restorative-mechanics-v1's
SL-counterpart) will face the same trip; advisor should consider
recalibration before SL-c.
