# Plan: v1-federation-inbound-b — HTTP-path wrapper + enforcement + Phase-6 fixture update + handler e2e

> **DELIVERY NOTE (planner — 2026-05-19):** this file is the plan
> deliverable. The canonical target path
> `.claude/PRPs/plans/v1-federation-inbound-b.plan.md` is blocked for
> Junior worker writes by Claude Code's built-in sensitive-file
> protection (every `.claude/**` path triggers the prompt regardless
> of `settings.json` `permissions.allow`). The advisor laptop session
> (which has full Write authority) MUST move this file to its canonical
> path BEFORE plan approval:
>
> ```
> git mv PLAN_DELIVERABLE.md .claude/PRPs/plans/v1-federation-inbound-b.plan.md
> ```
>
> The plan body is unchanged — only the path changes on `mv`. Same
> harness gap that fed-in-a's planner hit (see fed-in-a plan §
> "DELIVERY NOTE" + the v1-federation-inbound-a.plan.md commit
> history). The advisor's existing DQ #235 (harness gap on Junior
> `.claude/**` writes) tracks the fix; no new DQ filed here.
>
> **Planner DQs (#276 split-or-proceed; #277 rate-limit storage; #278
> trait-impl placement) are ALSO blocked** — Junior workers cannot
> write to `.claude/decision-queue.json` either, despite the path
> being explicitly whitelisted in `settings.json`. The DQ pending +
> resolved entries are described in §5.2 + §19 of THIS plan; the
> advisor must transcribe them to `.claude/decision-queue.json`
> directly when moving this file. Suggested wire shape provided in
> §19.1 / §19.2 below.

## 1. Summary

v1-federation-inbound-b ships the **HTTP-path enforcement layer** of the
federation-inbound v1 track: a `wrap_governance_inbound` wrapper + 5
pre-`inner` reject-gate check helpers + a `log_inbox_drop` audit
helper + a `GovernanceInboundActivity` trait + 3 trait impls (one per
governance activity type) + the new `receive_remote_moderation_label`
handler body (fills the Phase-6 `PublishLabel::receive` stub) + a
storage-cap-eviction helper called from each `receive_remote_*` at
insert time + 6 new `LemmyErrorType` variants with HTTP-status-code
mappings (403/413/400/429/429/409) + `#[serde(deny_unknown_fields)]`
on the 3 governance protocol structs (schema-fuzzing defence) + 1 new
`ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED` const dual-file
declaration + emit (best-effort) on each `receive_remote_*` DB-error
path + the Phase-6 `sanction_notice_round_trip` test fixture update
(Allowlist the `instance-a.test` peer before `receive`, per PRD §5.4
designed breakage) + a new behavioural handler-e2e module asserting
the 5 reject-gates end-to-end (specific HTTP status codes 403/429/409
per PRD §5.3) + the replay-cleanup-cron scheduler wiring that calls
fed-in-a's shipped `federation_inbox_nonce::delete_older_than` every
`federation.inbound.replay_cleanup_cron_interval_minutes` (default 60
min). **Headline acceptance:** `git diff
governance-v0..phase-v1-federation-inbound-b` at plan-approval shows
only `crates/utils/src/error.rs` (6 new variants + extended
`status_code()` match), `crates/apub/objects/src/protocol/governance/{sanction_notice,trust_attestation,moderation_label}.rs`
(`#[serde(deny_unknown_fields)]`), `crates/apub/activities/src/governance/inbox.rs`
(wrapper + 5 check helpers + log_inbox_drop + GovernanceInboundActivity
trait + storage-cap eviction helper + new
`receive_remote_moderation_label` body + best-effort persist_failed
emits on Phase-6 receive_remote_*), `crates/apub/activities/src/governance/publish_{sanction_notice,trust_attestation,label}.rs`
(impl GovernanceInboundActivity + one-line `receive` body),
`crates/db_schema/src/source/governance/governance_log.rs` +
`crates/api/api/src/governance/governance_log.rs` (1 new const +
re-export), `.claude/rules/governance-log-entry-kind-registry.md`
(append v1-federation-inbound-b section + remove
`federation_inbound_persist_failed` from fed-in-a "Deferred"),
`crates/routes/src/utils/scheduled_tasks.rs` (replay-cleanup cron
block), `crates/server/tests/e2e.rs` (Phase-6 fixture in-place Edit at
`sanction_notice_round_trip` + NEW handler-e2e module appended at file
end), the plan file, the retro. Acceptance invariants count goes
**54 → 55** (one new entry-kind const).

## 2. Source

- `.claude/PRPs/briefs/v1-federation-inbound-b-planning-1.md` @
  `49a0d6b1d` — the advisor brief (post-`/brehon-clarify`; DQ #273 +
  #274 + #275 resolved 2026-05-19; DQ #250 user-resolved 2026-05-18,
  all four binding).
- `.claude/PRPs/prds/v1-federation-inbound.prd.md` @ `governance-v0`
  — parent PRD; §3.1, §3.2, §3.3 (per-handler invariants), §5.1
  (Lemmy plumbing unchanged), **§5.2 (wrapper layering — `-b` core)**,
  **§5.3 (failure mode → HTTP table — 6 error variants + codes)**,
  **§5.4 (designed breakage + no `_unchecked` rule)**, **§7.1/§7.2
  (rate limit)**, **§7.3 (storage cap — INSERT-time eviction)**,
  **§7.4 (schema-fuzzing — `deny_unknown_fields`)**, **§7.5 (replay
  + cleanup cron)**, §8.4 (Diesel models — fed-in-a-shipped), §9.1
  (module layout — **path corrected per PRECON-2**), **§9.2 (wrapper
  signature verbatim + 6 LemmyErrorType variants)**, **§9.3 (per-handler
  patches verbatim shape, 3 files)**, **§9.4 (label-handler body
  verbatim, peer_trust_level_at_receipt fill-in resolved per DQ #273
  option-2)**, §10 (defaults — 11 keys from fed-in-a-seeded
  `governance_config`), §11.1 (outbound unchanged), §11.4 (Phase-6
  fixture Allowlist), §12.1 (DQ-FED-IN-1 carry-forward —
  `signature: String::new()`), §12.2 (`deny_unknown_fields`), §12.3
  (`local_case_id` NULL invariant), §12.4 (every inbound produces a
  log), §12.5 (step-up — out of `-b`).
- `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` @
  `7af873731` — **structural-skeleton mirror** + **call-surface
  reference** (PRECON-1 + PRECON-4). §6 carving table, §10.5/§10.6/§10.7
  (the 11 config keys + 9 ENTRY_KIND consts + trust-helper signatures
  + `get_conn(pool)` access pattern), §11 (the exact file paths
  fed-in-a created), §12 (fed-in-a's "NOT building" list — its `-b`
  items are `-b`'s scope), §15 laptop-DoD shape (already validated
  under PRECON-3).
- `.claude/PRPs/plans/phase-6-federation.plan.md` @ `governance-v0`
  — **§13-decomposition precedent** for the "`receive_remote_*` + `Activity::receive` + `governance_log::append` + handler-e2e" task shape.
- `.claude/rules/governance-log-entry-kind-registry.md` —
  pre-landed-const exemption + count-invariant (54 → 55 post-`-b`)
  + Phase-6 path-qualification of `crates/apub/activities/src/governance/inbox.rs::receive_remote_sanction_notice` (the authoritative
  correction of PRD §5.2/§9.1's `crates/apub/apub/...` descriptor
  error per PRECON-2).
- `.claude/rules/decision-queue.md` — schema-v2 attribution + Recipe
  1/2 + `kind: "validate-pending-laptop"` routing (PRECON-3).
- `.claude/rules/advisor-orchestrator.md` — Stage-shape, §5.2
  `validate-pending-laptop` handler, §G4 classifier (allowlist for
  auto-fix categories the impl-task may hit), cohort dispatch (Task
  5/6/7 disjoint files → `[P]` with `requires: task 4`).
- `.claude/PRPs/templates/plan.template.md` — 20-section schema.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — **ADR-006** (advisory-only inbound — `-b`'s wrapper persists with
  `local_case_id = NULL`, NEVER auto-applies — load-bearing security
  invariant), **ADR-014** (vanilla peers never trigger
  governance-inbound code), **ADR-015** (pseudonymisation — `actor_url`
  / `target_url` TEXT verbatim, no raw-id leakage into logs),
  **ADR-013** (`-b` adds no enums).

### Lessons that bind §13 decisions

- `feedback_multi_write_handlers_need_transactions.md` — the wrapper
  + receive_remote_* paths are multi-write; the plan §10 specifies
  `run_transaction` boundaries explicitly. Phase-6 already uses this
  pattern at `crates/apub/activities/src/governance/inbox.rs:160-176`
  + `:232-247`; `-b` preserves it.
- `feedback_lemmy_error_no_std_error.md` — **Case A** (uniform
  `LemmyResult<()>` outer; bare `?`; no `.map_err` bridges) — Task 9
  e2e mirrors the canonical
  `mod v1_federation_inbound_a_fixtures` sibling at
  `crates/server/tests/e2e.rs:15093-15151` verbatim.
- `feedback_async_pool_test_pattern.md` — e2e test helpers take
  `&mut AsyncPgConnection` or `governance_fixtures::bootstrap()` →
  `(container, context, db_url)` tuple.
- `feedback_junior_worker_e2e_edit_hang.md` — Task 9 uses TWO
  Edits on a 15,482-line file: one IN-PLACE Edit at the
  `sanction_notice_round_trip` block (~ line 5243) inserting the
  `federation_peer` Allowlist fixture insert, and ONE anchor-Edit
  appending the new handler-e2e module at file end. Strict
  anchor-append discipline; no mid-file scanning.
- `feedback_fix_impl_enumerate_all_callsites.md` — Task 4 enumerates
  all 3 governance `receive` impls via
  `rg "fn receive" crates/apub/activities/src/governance/publish_*.rs`
  before designing per-handler patches (Tasks 5/6/7). Watchpoint #2.
- `feedback_fix_impl_pre_push_cargo_check.md` — every impl-task ends
  with `cargo-check.bat --workspace --features full` pre-push (DoD
  §15.1).
- `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint
  cites a concrete file:line / table / function / lesson.
- `feedback_complexity_score_pre_split.md` — `-b` scores **14**
  (factor breakdown in §5.1); DQ #276 split-or-proceed filed.
- `feedback_explicit_file_arrays_on_tasks.md` — every §13 task
  carries FILES YAML.
- `feedback_parallel_cohort_dispatch.md` — Tasks 1+2+3 = Cohort A
  (3-way `[P]`, disjoint files); Tasks 5+6+7 = Cohort B (3-way
  `[P]`, disjoint `publish_*.rs` files, all `requires: task 4`).
- `feedback_cohort_validation_dependency_check.md` — Tasks 5/6/7
  declare `requires: task 4`; Task 9 declares `requires: 4, 5, 6, 7,
  8`.
- `feedback_pre_phase_dod_smoke_test.md` +
  `feedback_plan_dod_dry_run_at_write.md` — advisor smoke-test runs
  every §15 command literally before plan-approval.
- `feedback_features_full_p_crate_incompatible.md` — `-b` uses only
  `--workspace --features full`.
- `feedback_features_full_workspace_only.md` — `--features full`
  activates `cfg(feature = "full")` paths for the new wrapper.
- `feedback_wrapper_script_flag_silence.md` — all cargo via
  `scripts/brehon/cargo-{check,clippy,test}.{bat,sh}`.
- `feedback_windows_e2e_requires_bat_wrapper.md` — `.bat` wrapper on
  Windows for e2e.
- `feedback_clippy_test_style.md` — R1 `i64::from(...)`.
- `feedback_read_canonical_before_writing_spec.md` — §10 cites
  fed-in-a's `v1_federation_inbound_a_fixtures` sibling + Phase-6
  `receive_remote_*` shapes + PRD §9.2/§9.3/§9.4 verbatim.
- `feedback_principles_not_rules.md` — score 14 is a signal; lean
  proceed-as-one with PRD §0-atomic-carving precedent.
- `feedback_laptop_default_for_validate_pending.md` — PRECON-3
  laptop-shape DoD.
- `feedback_clarify_before_plan.md` — DQ #273/#274/#275 resolved
  before plan-write (brief §3a).

## 3. Problem statement

fed-in-a (PR #138, merged `7af873731`) shipped the schema/model
substrate of the federation-inbound v1 track: 4 new tables (`federation_peer`,
`federation_inbox_dropped_log`, `federation_inbox_nonce`,
`remote_moderation_label`), 2 new enums, 2 ALTERs on Phase-6's
`remote_sanction_notice` + `federation_attestation` tables, 11
`federation.inbound.*` `governance_config` seeds, 9
`ENTRY_KIND_FEDERATION_INBOUND_*` consts, 2 newtypes, 4 new Diesel
models + 2 extended Phase-6 models, and the
`federation_inbox_check_peer_trust` + `federation_peer_upsert_trust`
helpers in `crates/db_schema/src/source/governance/federation_peer.rs`.
But **none of the HTTP-path enforcement is wired**: every governance
inbound activity from any peer still lands in
`remote_sanction_notice` / `federation_attestation` with only HTTP
signature verification (Phase 6's baseline) — no peer-trust gate, no
size cap, no rate limit, no replay window, no storage-cap eviction,
no drop-log audit, no `ModerationLabel` advisory handler (Phase-6
shipped a `PublishLabel::receive` stub at
`crates/apub/activities/src/governance/publish_label.rs:27`).

PRD §3.2 + §3.3 + §5.2 + §5.3 + §7 specify the v1 enforcement layer:
a `wrap_governance_inbound` wrapper invoked from each governance
`Activity::receive` impl that checks (a) peer trust → 403 on
Blocklisted, (b) payload size → 413 on oversize, (c) schema strictness
→ 400 on unknown fields (serde-layer), (d) per-peer rate → 429 on
exceeded, (e) per-actor rate (attestations) → 429 on exceeded, (f)
replay nonce → 409 on duplicate-within-window, then (g) delegates to
the underlying `receive_remote_*` for the row insert + governance_log
emit, plus oldest-unreviewed-row storage-cap eviction at insert time.
PRD §5.4 names a **designed breakage**: the Phase-6
`sanction_notice_round_trip` direct-call test must be updated to
Allowlist its test peer before `receive` (no `_unchecked` variant
allowed). PRD §9.4 names the new `receive_remote_moderation_label`
body verbatim. PRD §7.5 names the replay-cleanup cron interval; fed-in-a
shipped the cleanup-query fn with a `// TODO(v1-federation-inbound-b)`
marker (DQ #250 user-resolved 2026-05-18: `-b` wires the scheduler).

Without `-b`: a hostile or compromised peer can flood the inbox, fill
the DB unboundedly, replay activities, or send oversize payloads —
the v1 production-grade-governance milestone (05 §7.1) cannot ship.

## 4. Solution statement

**Single chokepoint** — every governance inbound flows through
`wrap_governance_inbound`. The 3 per-handler patches in
`publish_sanction_notice.rs` / `publish_trust_attestation.rs` /
`publish_label.rs` reduce each `Activity::receive` body to a one-line
`wrap_governance_inbound(self, context, |a, c| { … }).await` call;
all enforcement runs **before** the prior Phase-6 `receive_remote_*`
fires.

**5 reject-gates + 1 success-side eviction.** The wrapper runs 5
pre-`inner` checks (trust → size → per-peer-rate → per-actor-rate →
replay; schema is enforced at the serde layer by
`#[serde(deny_unknown_fields)]` on the 3 protocol structs, NOT by a
wrapper helper — PRD §9.2 step 3 comment); each rejection writes a
`federation_inbox_dropped_log` row + a `governance_log::append` entry
keyed to the matching fed-in-a-shipped `ENTRY_KIND_FEDERATION_INBOUND_*`
const, all inside `run_transaction` for atomic audit. The 6th check
(storage-cap eviction, PRD §7.3) runs INSIDE each `receive_remote_*`
at the head of the success path per DQ #275 option-a — eviction is
NON-rejecting (drops the oldest unreviewed row for the peer, not the
new inbound), so it doesn't belong in the pre-`inner` reject chain.

**New label handler.** `receive_remote_moderation_label` fills the
Phase-6 stub. Body matches PRD §9.4 verbatim except the
`peer_trust_level_at_receipt` fill-in — per DQ #273 option-a (BINDING),
the handler **re-queries** `federation_inbox_check_peer_trust` inside
the wrapper's `inner` closure (one extra indexed
`federation_peer` lookup; zero PRD §9.2 signature change; rationale
in §10.4 + planner DQ #277 log).

**Config source.** Per DQ #274 option-a (BINDING), all wrapper config
reads route through `lemmy_api::governance::config::get_int` /
`get_text` against `Scope::Instance` on the fed-in-a-seeded 11
`federation.inbound.*` rows in `governance_config` — NOT through
Lemmy's `Settings` (PRD §9.2 `context.settings()` is illustrative
pseudocode). PRD §10 mandates `governance_config` for dashboard
tunability.

**6 new LemmyErrorType variants + status-code mapping.** Add to the
`LemmyErrorType` enum at `crates/utils/src/error.rs:114-122` (adjacent
to existing `CannotCombineFederationBlocklistAndAllowlist` /
`FederationDisabled` / `FederationDisabledByStrictAllowList` Federation
cluster): `FederationPeerBlocklisted` (→403), `FederationPayloadTooLarge`
(→413), `FederationSchemaInvalid` (→400), `FederationPeerRateLimitExceeded`
(→429), `FederationActorRateLimitExceeded` (→429),
`FederationActivityReplayed` (→409). Extend `impl
actix_web::error::ResponseError for LemmyError::status_code` at
`crates/utils/src/error.rs:223-230` (default-branch match) with 6 new
arms returning the PRD §5.3 codes.

**Schema-fuzzing defence.** Add `#[serde(deny_unknown_fields)]` at the
container-struct level on the 3 governance protocol structs in
`crates/apub/objects/src/protocol/governance/{sanction_notice,trust_attestation,moderation_label}.rs`
(adjacent to the existing `#[serde(rename_all = "camelCase")]` attrs
at lines 30/40/33). Per PRD §7.4 + §9.2 step 3 — unknown variant or
unknown field at deserialise time → 400, before `receive` runs.

**One new entry-kind const + best-effort emit.** Per fed-in-a §10.7
"Deferred" + brief §2.1 item 2:
`ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED` is the only new const
`-b` adds (the other 9 are fed-in-a-shipped). Declared in
`crates/db_schema/src/source/governance/governance_log.rs` (alphabetical
within the `federation_inbound_*` cluster), re-exported in the api
shim `crates/api/api/src/governance/governance_log.rs`. Emitted as
**best-effort** at the head of each `receive_remote_*`'s error branch:
on tx failure, attempt `governance_log::append(...,
ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED, ...)` outside the
rollback; if that append also fails, swallow + surface the original
error. Per PRD §5.3 row 7.

**Replay-cleanup cron.** New scheduler block in
`crates/routes/src/utils/scheduled_tasks.rs` mirroring the existing
`appeal_window_expiry` / `sponsor_liability_grace` pattern: an
hourly-default tick (interval read from
`federation.inbound.replay_cleanup_cron_interval_minutes` =60 default)
calls fed-in-a's `lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than(window_days, conn)`. Concurrency
guarded by an atomic-bool sentinel (mirrors
`SPONSOR_LIABILITY_GRACE_RUNNING`); disabled in tests via
`BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1`. Per DQ #250 (user-resolved
2026-05-18, BINDING).

**Phase-6 fixture surgical update.** ONE in-place Edit at the
`sanction_notice_round_trip` block (~line 5243 — the
`ActivityTrait::receive` call) inserts a 5-line `federation_peer`
fixture seeding `instance-a.test` with `trust_level =
FederationPeerTrust::Allowlisted` against instance B's DB BEFORE
the `receive` call. Mirror fed-in-a's `mod v1_federation_inbound_a_fixtures::seed_federation_peer`
shape at `crates/server/tests/e2e.rs:15106-15130`. No `_unchecked`
variant (PRD §5.4). `grep -n "PublishSanctionNotice::receive\|PublishTrustAttestation::receive\|PublishLabel::receive\|ActivityTrait::receive"
crates/server/tests/e2e.rs` enumerated at plan-time confirms
`sanction_notice_round_trip` is the **only** existing test that
drives a governance `receive` directly (line 5244; watchpoint #3 +
PMD #8 closed_at-rename guard satisfied — no sibling tests need the
Allowlist treatment).

**Handler-e2e module.** NEW `mod v1_federation_inbound_b_fixtures` at
the end of `crates/server/tests/e2e.rs` (Case A error-shape;
`LemmyResult<()>` uniform), single anchor-Edit append. 5 test
functions asserting end-to-end: (a) Blocklisted peer → 403
(`FederationPeerBlocklisted`); (b) per-peer rate exceeded → 429
(`FederationPeerRateLimitExceeded`); (c) replayed activity-id within
window → 409 (`FederationActivityReplayed`); (d) happy path
(Allowlisted peer, in-limits) — Phase-6 `receive_remote_*` row
persists + governance_log entry emits; (e) new
`receive_remote_moderation_label` happy path — `remote_moderation_label`
row persists + `ENTRY_KIND_FEDERATION_LABEL_RECEIVED` log entry.

**DoD per PRECON-3 (`validate-pending-laptop`, NOT Shape G).** Shape
G suspended until 2026-06-01 per DQ #229. Each impl-task raises
`kind: "validate-pending-laptop"` post-push naming §15 DoD commands
verbatim with `--workspace --features full`; advisor laptop runs each
sequentially via `.bat` wrapper. Phase-2 e2e selects local-vs-dispatch
at user-gate-4. NO `.github/workflows/*.yml` §15.6 section.

## 5. Metadata

- **Phase:** `v1-federation-inbound-b`
- **Branch:** `phase-v1-federation-inbound-b` (cut by BM-task before
  Task 1)
- **Target impl-task model:** `sonnet-4-6` (default per planning.md
  §5). Split-DQ threshold is `> 8`.
- **Estimated tasks:** 11 (Task 0 pre-flight + Tasks 1-9 impl + Task
  10 retro)
- **Estimated cargo budget:** 0 GB peak laptop-concurrent. Per
  `advisor-orchestrator.md` §5.2: laptop handler serially processes
  `validate-pending-laptop` entries. Peak ~6 GB cargo at threshold
  (single-cargo serialised).
- **Forbidden-window applicability:** non-binding for EliteDesk worker
  daemon (cargo runs on laptop, not on the daemon). Binding for
  advisor-side §3.4 DoD smoke test (pre-plan-approval).
- **Complexity score:** **14/10** — threshold-tripping; planner DQ
  #276 filed.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. The score is mechanical:

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | **4** | 9 impl tasks (Tasks 1-9; Task 0 + retro excluded). `max(0, 9-5) = 4` |
| Migrations touched | +2 each | **0** | NO migration in `-b` (handler-only; PRECON-1 — schema substrate is fed-in-a-shipped) |
| Crates touched | +1 each | **7** | `lemmy_utils` (error variants + mapper), `lemmy_apub_objects` (deny_unknown_fields on 3 protocol structs), `lemmy_apub_activities` (wrapper + 5 helpers + label handler + 3 per-handler patches), `lemmy_db_schema` (1 new const), `lemmy_api` (1 new shim re-export), `lemmy_routes` (replay-cleanup cron), `lemmy_server` (e2e fixture + handler-e2e module) |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | **3** | Task 9 modifies `crates/server/tests/e2e.rs` (one task; fed-in-a precedent (score 13) counts this fork's `crates/server/tests/e2e.rs` as +3) |
| New ADR-affecting decisions | +2 each | **0** | PRD §16 settled all decisions; brief §3a clarify-DQs all resolved before plan-write |
| Cargo budget peak above 6 GB | +1 per GB | **0** | Cargo runs on laptop serially (PRECON-3); peak ~6 GB at threshold but not above (laptop handler enforces serial). |
| **Total** | — | **14** | Threshold (Sonnet): `> 8`. **Tripped.** |

### 5.2 Split-or-proceed DQ

**DQ #276** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`,
filed pre-commit) — split-or-proceed per §5.1. Question: "Complexity
score 14 exceeds Sonnet threshold 8 — split v1-federation-inbound-b
into `-b-1` (Tasks 1-4: error variants + mapper + protocol attrs +
persist_failed const + wrapper/helpers/trait/label-handler/storage-cap
in inbox.rs) + `-b-2` (Tasks 5-9: per-handler patches + replay-cleanup
cron + fixture update + handler-e2e) + retro, or proceed as one
plan?" Options: `split` / `proceed`. Context: "First HTTP-path
sub-phase in the federation-inbound v1 track. Brief §0 names `-b` as
one atomic deliverable (HTTP-path enforcement layer; the wrapper +
its 5 check helpers are one logical chokepoint and must not be
fragmented from each other per brief §4.1). Dominant factors: 9 impl
tasks (+4), 7 crates touched (+7), e2e edit (+3). fed-in-a precedent
at score 13 shipped proceed-as-one with no operational regret. Per
brief §4.1 the wrapper + 5 check helpers are one atomic enforcement
layer — the split seam above respects this (both halves in `-b-1`).
Splitting still races shared files: Task 4 modifies `inbox.rs`; Task
9 modifies `e2e.rs`. If split, `-b-1` ships Tasks 1-4 and `-b-2`
resumes from Task 5 (no shared-file race because Tasks 5/6/7 are
disjoint `publish_*.rs` files, Task 8 is `scheduled_tasks.rs`, Task
9 is `e2e.rs`)."

**Planner lean: proceed.** fed-in-a precedent at 14-class score
shipped proceed-as-one. Plan ships under proceed-as-one assumption
pending DQ #276 resolution. If the advisor answers `split`, re-plan
per `feedback_complexity_score_pre_split.md`.

---

## 6. Relationship to other v1-federation-inbound sub-phases

Per brief §0 carving (advisor-decided 2026-05-16, BINDING — restated
verbatim from `v1-federation-inbound-a.plan.md` §6 + brief §0):

| Sub-phase | Status | What it ships | `-b` relationship |
|---|---|---|---|
| v1-federation-inbound-a | **SHIPPED** (PR #138 merged 2026-05-19 at `7af873731`) | Schema + 2 enums + 4 tables + 2 ALTERs + 11 seeds + 9 ENTRY_KIND consts + 2 newtypes + 4 new Diesel models + 2 extended Phase-6 models + 2 trust helpers + e2e migration round-trip + trust foundation test | `-b` **CONSUMES** `-a`'s substrate verbatim. NO new migration, NO `schema.rs` edit, NO new newtypes, NO new model files. |
| **v1-federation-inbound-b (THIS PLAN)** | NOT YET CUT | `wrap_governance_inbound` + 5 check helpers + `log_inbox_drop` + GovernanceInboundActivity trait + 3 trait impls + storage-cap eviction + `receive_remote_moderation_label` body + 6 new `LemmyErrorType` variants + status-code mapper extension + `deny_unknown_fields` on 3 protocol structs + 1 new `_persist_failed` const + replay-cleanup cron + Phase-6 fixture Allowlist update + handler-e2e module | — |
| v1-federation-inbound-c | pending (after `-b`) | 3 admin REST endpoints (`GET .../inbox` + `POST .../cross-link` + `POST .../dismiss`) + `crates/db_views/federation_inbox/` view crate + OQ-FED-IN-1 pseudonym rendering + `webauthn-rs` step-up on trust-change + admin-path e2e. Emits 2 entry-kind consts NOT shipped here (`_cross_linked`, `_dismissed`). | Reads columns + tables `-a` adds; consumes `-b`'s drop-log + advisory rows; reads `peer_trust_level_at_receipt` + `admin_action`. |

**Cross-PRD sequencing:** Phase 6 (merged at PR #46, strict ancestor;
provides the 3 governance `Activity::receive` impls `-b` patches +
the `receive_remote_sanction_notice` / `receive_remote_trust_attestation`
fns `-b` wraps); fed-in-a (most-recent shared-file sibling, merged
2026-05-19 — provides the 4 tables + 11 config seeds + 9 consts +
trust helpers + the canonical `LemmyResult<()>` Case-A test sibling
`-b`'s handler-e2e mirrors).

**Re-carving refusal.** Per brief §4.1: the §0 carving is BINDING.
If the planner finds the wrapper genuinely cannot ship without a
`-c` admin endpoint (it can — admin endpoints read advisory rows but
do not produce them), STOP and file `kind: "blocker"`. Not a re-plan
the planner does unilaterally.

---

## 7. Preflight guardrails inherited from prior phases

These are non-negotiable for this plan; §15 must respect them.

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never
  `as` cast (per `feedback_clippy_test_style.md`).
- **R5:** Task 0 enumerates ALL probes explicitly (per JM-b retro
  Event 4; `pre-phase-harness-audit.md`).
- **R6:** all clippy invocations use `--no-deps -- -D warnings`
  uniformly (per JM-b retro Event 3).
- **R7:** every Task that touches a struct or re-export runs
  `cargo test --no-run --workspace --features full --test e2e` after
  cargo check + clippy.
- **R9:** when adding a field to a public struct (or extending an
  enum), enumerate **all** callsites with `rg`. Per
  `feedback_fix_impl_enumerate_all_callsites.md`. Applies to `-b`
  Task 1's 6 `LemmyErrorType` variant adds (no caller change required
  — `LemmyErrorType` is `#[non_exhaustive]`, variant adds are
  forward-compatible); Task 2's `deny_unknown_fields` attr add (no
  caller change — purely deserialise behaviour).
- **DQ #232 carry-forward:** all shared-file edits APPEND-ONLY where
  possible.

## 8. Flow design

```text
                       HTTP inbox (Lemmy: shared_inbox)
                                  │
                                  ▼
          activitypub_federation::actix_web::inbox::receive_activity_with_hook
                                  │
                  ┌───────────────┴───────────────┐
                  │   HTTP signature verify        │  ← Lemmy code, unchanged in -b
                  │   serde deserialise            │  ← deny_unknown_fields (Task 2)
                  │   ReceivedActivity::create     │  ← Lemmy dedup, unchanged
                  └───────────────┬───────────────┘
                                  │
                                  ▼
                  untagged dispatch → SharedInboxActivities::{PublishSanctionNotice,PublishTrustAttestation,PublishLabel}
                                  │
                                  ▼
                          Activity::verify (Phase-6, unchanged)
                                  │
                                  ▼
                          Activity::receive  ← Task 5 / 6 / 7 (one-line wrap call)
                                  │
                                  ▼
                  wrap_governance_inbound<A: GovernanceInboundActivity>(activity, ctx, inner)
                                  │
                                  ├── 1. federation_inbox_check_peer_trust(peer_domain, conn)
                                  │       ├── Blocklisted → log_inbox_drop(..) + Err(FederationPeerBlocklisted) → 403
                                  │       └── else continue (Unknown / Allowlisted / UntrustedReceive)
                                  │
                                  ├── 2. federation_inbox_check_size(activity, ctx)
                                  │       └── exceeded → log_inbox_drop(..) + Err(FederationPayloadTooLarge) → 413
                                  │
                                  │   [3. schema = serde deny_unknown_fields, already enforced upstream — no wrapper helper]
                                  │
                                  ├── 4. federation_inbox_check_peer_rate_limit(peer_domain, ctx)
                                  │       └── exceeded → log_inbox_drop(..) + Err(FederationPeerRateLimitExceeded) → 429
                                  │
                                  ├── 5. activity.check_per_actor_rate_limit(ctx)  // trait method
                                  │       ├── attestation + exceeded → log_inbox_drop(..) + Err(FederationActorRateLimitExceeded) → 429
                                  │       └── sanction_notice / moderation_label → no-op (returns Ok)
                                  │
                                  ├── 6. federation_inbox_check_replay(peer_domain, activity_id, ctx)
                                  │       └── seen-in-window → log_inbox_drop(..) + Err(FederationActivityReplayed) → 409
                                  │
                                  └── 7. inner(activity, context).await    ← Phase-6 / new label handler
                                              │
                                              ▼
                                  receive_remote_<TYPE>:
                                      ├── evict_oldest_unreviewed_if_needed(peer_domain, table, cap, conn)  // PRD §7.3, DQ #275
                                      │       └── oldest dropped (non-rejecting) → log_inbox_drop("storage_cap_evicted") + governance_log(_DROPPED_STORAGE_CAP_EVICTED)
                                      ├── run_transaction:
                                      │       ├── insert into remote_sanction_notice / federation_attestation / remote_moderation_label
                                      │       │   (with local_case_id: None — ADR-006 invariant)
                                      │       └── governance_log::append(ENTRY_KIND_FEDERATION_<TYPE>_RECEIVED, payload, None)
                                      └── on tx failure: best-effort governance_log::append(_PERSIST_FAILED, …) outside tx, then surface error
```

Cron side (independent):

```text
AsyncScheduler tick (every `federation.inbound.replay_cleanup_cron_interval_minutes`, default 60)
        │
        ▼
guard FED_REPLAY_CLEANUP_RUNNING (atomic CAS; warn-skip on contention)
        │
        ▼
federation_inbox_nonce::delete_older_than(window_days, conn)
        │
        └── deletes rows where seen_at < now - window_days
```

## 9. Mandatory reading

The impl-task subagent MUST Read before its first edit:

### Schema / type definitions (call surface)

- `crates/utils/src/error.rs:1-128` — `LemmyErrorType` enum (6 new
  variants inserted adjacent to `CannotCombineFederationBlocklistAndAllowlist`
  cluster at `:114-122`); `:223-235` — `impl
  actix_web::error::ResponseError for LemmyError::status_code` (the
  3-arm match `-b` extends with 6 new arms before the `_ =>
  BAD_REQUEST` default).
- `crates/db_schema/src/source/governance/federation_peer.rs:54-95` —
  `federation_inbox_check_peer_trust` (sig: `pub async fn ...(peer_domain: &str, conn: &mut AsyncPgConnection) -> LemmyResult<FederationPeerTrust>`),
  `federation_peer_upsert_trust`, `FederationPeerInsertForm`. The
  wrapper calls `federation_inbox_check_peer_trust`; the Phase-6
  fixture + handler-e2e use `FederationPeerInsertForm`.
- `crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs:1-32`
  — `FederationInboxDroppedLogInsertForm` (sig: `source_instance:
  String, activity_id: Option<String>, drop_reason: String,
  payload_excerpt: Option<String>`). The `log_inbox_drop` helper
  writes this row.
- `crates/db_schema/src/source/governance/federation_inbox_nonce.rs:1-51`
  — `FederationInboxNonceInsertForm` (sig: `peer_instance: String,
  activity_id: String`) + UNIQUE constraint at `(peer_instance,
  activity_id)`. `federation_inbox_check_replay` inserts here;
  duplicate-key error → `FederationActivityReplayed`. Also exposes
  `pub async fn delete_older_than(window_days: i64, conn) ->
  LemmyResult<usize>` (line 37-51, the cron consumes this).
- `crates/db_schema/src/source/governance/remote_moderation_label.rs:1-56`
  — `RemoteModerationLabel` model + `RemoteModerationLabelInsertForm`
  (sig: `source_instance, actor_url, target_url, label, summary,
  published_at, signature, local_case_id, peer_trust_level_at_receipt`).
  The new `receive_remote_moderation_label` writes this row.
- `crates/db_schema/src/source/governance/governance_log.rs:140-143` —
  Phase-6 federation consts; `:221-229` — fed-in-a's 9 new
  `ENTRY_KIND_FEDERATION_INBOUND_*` consts (the wrapper + handlers
  emit these). Task 3 appends one new const adjacent to this cluster.
- `crates/api/api/src/governance/config.rs:227-258` — `pub async fn
  get_int(cache, pool, scope, key)` — the wrapper's int-config read
  surface (per DQ #274 option-a); `:326-357` — `get_text` for the
  `federation.inbound.default_trust_for_new_peers` key (Task 4 may
  consume); `:1150-1180` — `const_default_int` for the 10 int
  federation.inbound.* keys.

### Existing patterns (MIRROR refs §13 tasks point at)

- `crates/apub/activities/src/governance/inbox.rs:1-12` — the
  module doc-comment naming the **DQ-6.6-inbound / resolved id 37**
  precedent that pins this file's location as `lemmy_apub_activities`
  (NOT `lemmy_apub`). Task 4 cites this verbatim in its commit
  body (PRECON-2). Wrapper goes IN this file, alongside Phase-6's
  `receive_remote_sanction_notice` (`:105`) and
  `receive_remote_trust_attestation` (`:195`).
- `crates/apub/activities/src/governance/inbox.rs:105-179` —
  Phase-6 `receive_remote_sanction_notice` — `-b` Task 4 modifies
  this fn to call `evict_oldest_unreviewed_if_needed(...)` at the
  head and to wrap the tx call in best-effort `_persist_failed`
  emission on error.
- `crates/apub/activities/src/governance/inbox.rs:195-250` —
  Phase-6 `receive_remote_trust_attestation` — same modification
  pattern as `receive_remote_sanction_notice`.
- `crates/apub/activities/src/governance/inbox.rs:157-176` — the
  canonical `pool = &mut context.pool(); conn = &mut get_conn(pool).await?;
  conn.run_transaction(|conn| async move { ... .scope_boxed() }).await?;`
  pattern for multi-write atomic insert + governance_log append.
  `feedback_multi_write_handlers_need_transactions.md` binding.
- `crates/apub/activities/src/governance/publish_sanction_notice.rs:103-110`
  — Phase-6 `Activity::receive` impl shape. Task 5 rewrites the
  body to one `wrap_governance_inbound` call.
- `crates/apub/activities/src/governance/publish_trust_attestation.rs:86-93`
  — same for Task 6.
- `crates/apub/activities/src/governance/publish_label.rs:27-30` —
  Phase-6 STUB receive (the `_context`-ignored stub). Task 7
  rewrites it to wrap-call the new `receive_remote_moderation_label`.
- `crates/apub/objects/src/protocol/governance/sanction_notice.rs:30,40-42`
  — existing `#[serde(rename_all = "camelCase")]` attr above the
  protocol struct. Task 2 adds `#[serde(deny_unknown_fields)]`
  adjacent (same attr block). Same for `trust_attestation.rs:33-35`
  and `moderation_label.rs:30-32`.
- `crates/routes/src/utils/scheduled_tasks.rs:260-288` — `appeal_window_expiry`
  hourly tick (the cleanest mirror for `-b`'s replay-cleanup cron —
  same hour interval, same atomic-bool guard pattern, same
  `BREHON_DISABLE_*` env-var disable).
- `crates/routes/src/utils/scheduled_tasks.rs:308-360` —
  `sponsor_liability_grace` with config-driven interval. The
  replay-cleanup cron uses an analogous config-driven interval read
  (`federation.inbound.replay_cleanup_cron_interval_minutes` via
  `get_int(... Scope::Instance, ...)`).
- `crates/db_schema/src/source/governance/governance_log.rs:251-312`
  — `governance_log::append` signature + the
  `pool: &mut DbPool<'_>` reborrow-friendly form. Used in best-effort
  `_persist_failed` emit and the existing `inner` happy path.
- `crates/server/tests/e2e.rs:801-836` — `governance_fixtures::bootstrap`
  signature (sig: `async fn bootstrap() -> LemmyResult<(ContainerAsync,
  Data<LemmyContext>, String)>`). The handler-e2e fixtures use this.
- `crates/server/tests/e2e.rs:4809-5340` — Phase-6
  `sanction_notice_round_trip` test. The fixture insert lands AT
  the `ActivityTrait::receive` call (line 5244) — insert
  `federation_peer` row for `instance-a.test` immediately BEFORE
  the receive call. Mirror the same fixture pattern fed-in-a's
  `seed_federation_peer` uses (next bullet).
- `crates/server/tests/e2e.rs:15093-15151` — fed-in-a's
  `mod v1_federation_inbound_a_fixtures` — the **canonical
  sibling** for the new `mod v1_federation_inbound_b_fixtures`
  Task 9 appends. Same imports, same `LemmyResult<()>` Case-A
  outer, same `seed_federation_peer` helper (Task 9 mirrors this
  helper verbatim — DO NOT re-invent), same
  `governance_fixtures::bootstrap()` tuple unpack.

### Adjacent test fixtures

- `crates/server/tests/e2e.rs:15106-15130` — `seed_federation_peer`
  helper (mirrors verbatim into Task 9's new module).
- `crates/server/tests/e2e.rs:5239-5245` — the
  `ActivityTrait::verify` + `ActivityTrait::receive` direct-call
  shape in `sanction_notice_round_trip`.

### Lessons (binding — per `advisor-orchestrator.md` §2.4)

- `feedback_lemmy_error_no_std_error.md` — Case A (mandatory for the
  Task 9 e2e module).
- `feedback_async_pool_test_pattern.md` — `&mut AsyncPgConnection` /
  `governance_fixtures::bootstrap()`.
- `feedback_junior_worker_e2e_edit_hang.md` — 2 Edits on a 15,482-line
  e2e.rs; one in-place (small), one append (file end).
- `feedback_multi_write_handlers_need_transactions.md` — `run_transaction`
  pattern at every multi-write site.
- `feedback_fix_impl_enumerate_all_callsites.md` — `rg "fn receive"`
  enumeration at the head of Task 4's IMPLEMENT block.
- `feedback_advisor_watchpoint_specificity.md` — every §4 watchpoint
  cites a concrete file:line.
- `feedback_read_canonical_before_writing_spec.md` — §10 mirrors PRD
  §9 + Phase-6/fed-in-a source verbatim.
- `feedback_features_full_p_crate_incompatible.md` —
  `--workspace --features full` only.
- `feedback_clippy_test_style.md` — R1 `i64::from(...)`.
- `feedback_principles_not_rules.md` — score 14 is a signal, not a
  hard refusal.

## 10. Patterns to mirror

Each entry cites a specific file / line / table / lesson per
`feedback_advisor_watchpoint_specificity.md`. Tasks `IMPLEMENT` against
these blocks verbatim.

### 10.1 Six new `LemmyErrorType` variants + status-code mapper extension

**Mirror:** `crates/utils/src/error.rs:114-122` (existing Federation
cluster) + `:223-235` (existing `status_code()` match).

```rust
// crates/utils/src/error.rs — INSERT adjacent to the existing
// Federation cluster (line ~:114-122). All 6 variants unit; serde
// `rename_all = "snake_case"` already applied at enum level
// (line :9).

  CannotCombineFederationBlocklistAndAllowlist,   // existing
  // v1-federation-inbound-b additions (PRD §5.3 + §9.2):
  FederationPeerBlocklisted,
  FederationPayloadTooLarge,
  FederationSchemaInvalid,
  FederationPeerRateLimitExceeded,
  FederationActorRateLimitExceeded,
  FederationActivityReplayed,
  CouldntParsePaginationToken,                    // existing (line :115)
  ...
```

**Status-code mapper extension** at `crates/utils/src/error.rs:223-230`:

```rust
impl actix_web::error::ResponseError for LemmyError {
  fn status_code(&self) -> actix_web::http::StatusCode {
    match self.error_type {
      LemmyErrorType::IncorrectLogin => actix_web::http::StatusCode::UNAUTHORIZED,
      LemmyErrorType::NotFound => actix_web::http::StatusCode::NOT_FOUND,
      // v1-federation-inbound-b additions (PRD §5.3):
      LemmyErrorType::FederationPeerBlocklisted        => actix_web::http::StatusCode::FORBIDDEN,         // 403
      LemmyErrorType::FederationPayloadTooLarge        => actix_web::http::StatusCode::PAYLOAD_TOO_LARGE, // 413
      LemmyErrorType::FederationSchemaInvalid          => actix_web::http::StatusCode::BAD_REQUEST,       // 400
      LemmyErrorType::FederationPeerRateLimitExceeded  => actix_web::http::StatusCode::TOO_MANY_REQUESTS, // 429
      LemmyErrorType::FederationActorRateLimitExceeded => actix_web::http::StatusCode::TOO_MANY_REQUESTS, // 429
      LemmyErrorType::FederationActivityReplayed       => actix_web::http::StatusCode::CONFLICT,          // 409
      _ => actix_web::http::StatusCode::BAD_REQUEST,
    }
  }
  ...
}
```

**Notes:** `LemmyErrorType` enum is `#[non_exhaustive]` (line `:10`),
so variant adds do NOT require caller updates. The enum's
`#[serde(rename_all = "snake_case")]` (line `:9`) means the wire
form is `federation_peer_blocklisted` / `federation_payload_too_large`
/ etc — consumed by lemmy-ui translation tables (out of `-b` scope
to add UI strings).

### 10.2 `#[serde(deny_unknown_fields)]` on 3 governance protocol structs

**Mirror:** `crates/apub/objects/src/protocol/governance/sanction_notice.rs:30,40-42`
(existing `#[serde(rename_all = "camelCase")]` + `#[serde(rename = "type")]`
attrs).

```rust
// crates/apub/objects/src/protocol/governance/sanction_notice.rs
// INSERT adjacent to the existing #[serde(rename_all = "camelCase")]
// attribute (line :40). Same for trust_attestation.rs (:33) and
// moderation_label.rs (:30).

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]              // v1-federation-inbound-b
pub struct SanctionNoticeProtocol {
  #[serde(rename = "type")]
  pub kind: SanctionNoticeKind,
  ...
}
```

**Per PRD §7.4:** unknown enum variants of `SanctionAction`,
`SanctionScope`, `AttestationType` already cause `serde_json::from_value`
to fail (existing strict-deserialise). The new attr adds field-level
strictness: a peer sending `{"foo": "bar", ...}` with an unrecognised
field is rejected at deserialise time → `serde_json::Error` →
`activitypub_federation` returns HTTP 400. The wrapper's "schema"
gate (PRD §9.2 step 3) is therefore enforced at the serde layer, not
via a wrapper helper.

**Field-coverage gotcha:** if any of the 3 protocols carry a freeform
`rest: Map<String, Value>` field, `deny_unknown_fields` may interact
with `#[serde(flatten)]`. **Verify at task start:** read each struct's
existing field list before applying the attr; if a struct uses
`#[serde(flatten)]` with a catch-all map at the outer level, the
attr must be skipped for that struct + planner files `kind: "blocker"`.
Plan-time check via `grep -n "flatten" crates/apub/objects/src/protocol/governance/*.rs`
returned empty — the typed `*Protocol` outer structs do NOT use
`#[serde(flatten)]`; the stub structs at `lemmy_apub_activities` (NOT
modified by Task 2) carry the `rest` map.

### 10.3 New entry-kind const + registry append

**Mirror:** fed-in-a §10.7 dual-file discipline. Both files are
APPEND-ONLY shared-file edits.

```rust
// crates/db_schema/src/source/governance/governance_log.rs
// INSERT alphabetically within the federation_inbound_* cluster
// (currently lines :221-229). v1-federation-inbound-b adds 1 const:

pub const ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED: &str = "federation_inbound_persist_failed";
```

```rust
// crates/api/api/src/governance/governance_log.rs (shim)
// ADD one `pub use` line alphabetically with the existing
// _FEDERATION_INBOUND_* re-exports.

pub use lemmy_db_schema::source::governance::governance_log::ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED;
```

```markdown
<!-- .claude/rules/governance-log-entry-kind-registry.md APPEND a
new section AFTER the existing v1-federation-inbound-a section; also
EDIT the Deferred bullet (remove `federation_inbound_persist_failed`
— now shipped). -->

## v1-federation-inbound-b entry kinds (1, this sub-phase)

Landed alongside task 3's dual-file edit. v1-federation-inbound-b
defines the const AND ships the live emitting call site in the same
sub-phase (best-effort emit in each `receive_remote_*`'s DB-error
branch, per PRD §5.3 row 7 + §9.2).

| Rust const | `&str` value | Source | Emitting handler | Semantic |
|---|---|---|---|---|
| `ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED` | `federation_inbound_persist_failed` | -b shipped | -b `inbox.rs::receive_remote_sanction_notice` + `receive_remote_trust_attestation` + `receive_remote_moderation_label` (best-effort outside tx on rollback) | Phase-6/-b `receive_remote_*` insert tx failed; HTTP 500. Per PRD §5.3 row 7. |

<!-- Update the "Deferred" bullet in v1-federation-inbound-a's
section: remove the `federation_inbound_persist_failed` line. -->
```

**Acceptance invariants count goes 54 → 55** (one new const).

### 10.4 `wrap_governance_inbound` + 5 check helpers + `log_inbox_drop` + storage-cap eviction + `receive_remote_moderation_label`

**Mirror:** PRD §9.2 verbatim (wrapper signature + 7-step body) +
PRD §9.4 verbatim (label handler body, except the resolved
`peer_trust_level_at_receipt` fill-in per DQ #273 option-a — re-query).

Goes IN: `crates/apub/activities/src/governance/inbox.rs` (the
`lemmy_apub_activities` crate — **PRECON-2 BINDING**: NOT the PRD
§5.2/§9.1 literal `crates/apub/apub/src/governance/inbox.rs` path,
which is a documented descriptor error / non-existent file path
rejected for the same reason DQ-6.6-inbound id 37 rejected it for
Phase-6 — see the file's `:1-12` module doc-comment verbatim).
Alongside Phase-6's `receive_remote_sanction_notice` (`:105`) +
`receive_remote_trust_attestation` (`:195`).

```rust
// crates/apub/activities/src/governance/inbox.rs (Task 4 APPENDS
// below Phase-6's existing fns; also MODIFIES the 2 existing Phase-6
// fns to insert the eviction call + best-effort persist_failed emit).

use std::collections::HashMap;
use std::sync::OnceLock;
use parking_lot::Mutex;

use lemmy_db_schema::source::governance::{
  federation_inbox_dropped_log::FederationInboxDroppedLogInsertForm,
  federation_inbox_nonce::FederationInboxNonceInsertForm,
  federation_peer::federation_inbox_check_peer_trust,
  governance_log::{
    self,
    ENTRY_KIND_FEDERATION_INBOUND_BLOCKED,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY,
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED,
    ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED,
    ENTRY_KIND_FEDERATION_LABEL_RECEIVED,
  },
  remote_moderation_label::{RemoteModerationLabel, RemoteModerationLabelInsertForm},
};
use lemmy_db_schema_file::enums::FederationPeerTrust;

// -----------------------------------------------------------------
// GovernanceInboundActivity trait — exposes per-activity discriminators
// the wrapper needs. Impls land in publish_{sanction_notice,
// trust_attestation,label}.rs (Tasks 5/6/7) alongside the existing
// `impl Activity for ...` blocks.
// -----------------------------------------------------------------
#[async_trait::async_trait]
pub(crate) trait GovernanceInboundActivity: Sized {
  /// AP activity id (used for replay nonce + drop-log).
  fn activity_id(&self) -> &Url;
  /// Domain of the activity actor (used for peer-trust + per-peer rate).
  fn actor_domain(&self) -> LemmyResult<String>;
  /// Serialized payload size in bytes (the wrapper compares against the
  /// per-type cap read from governance_config).
  fn payload_size_bytes(&self) -> LemmyResult<usize>;
  /// Governance-config key for this activity's size cap (per PRD §10
  /// + fed-in-a §10.6 — one of the 3 `federation.inbound.max_payload_bytes_*`
  /// rows).
  fn payload_size_cap_key(&self) -> &'static str;
  /// Per-actor rate-limit check (only attestations are meaningful — the
  /// default impl returns Ok). Returns the resolved error on exceed.
  async fn check_per_actor_rate_limit(
    &self,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let _ = context;
    Ok(())
  }
}

// -----------------------------------------------------------------
// In-memory rate-limit counters (PRD §7.1 + §7.2; planner DQ #277 log).
// Co-located here for v1 simplicity — extracted to a dedicated rate-
// limit module in v2 if state grows. Keys: (peer_domain, hour_bucket)
// and (subject_url_hash, hour_bucket).
// -----------------------------------------------------------------
pub(crate) fn rate_per_peer_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}
pub(crate) fn rate_per_actor_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}
pub(crate) fn current_hour_bucket() -> i64 {
  chrono::Utc::now().timestamp() / 3600
}

// -----------------------------------------------------------------
// wrap_governance_inbound — PRD §9.2 verbatim signature.
// -----------------------------------------------------------------
pub(crate) async fn wrap_governance_inbound<F, Fut, A>(
  activity: A,
  context: &Data<LemmyContext>,
  inner: F,
) -> LemmyResult<()>
where
  F: FnOnce(A, &Data<LemmyContext>) -> Fut,
  Fut: std::future::Future<Output = LemmyResult<()>>,
  A: GovernanceInboundActivity,
{
  let peer_domain = activity.actor_domain()?;
  let activity_id = activity.activity_id().to_string();

  // Step 1 — peer trust gate.
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let trust = federation_inbox_check_peer_trust(&peer_domain, conn).await?;
  if trust == FederationPeerTrust::Blocklisted {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "blocklisted",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_BLOCKED,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationPeerBlocklisted.into());
  }

  // Step 2 — per-type size cap (from governance_config per DQ #274).
  let cap: i64 = lemmy_api::governance::config::get_int(
    &mut lemmy_api::governance::config::ConfigCache::new(),
    pool,
    lemmy_api::governance::config::Scope::Instance,
    activity.payload_size_cap_key(),
  )
  .await?;
  let size = i64::try_from(activity.payload_size_bytes()?).unwrap_or(i64::MAX);
  if size > cap {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "oversize",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationPayloadTooLarge.into());
  }

  // Step 3 — schema strictness is enforced at the serde layer
  // (deny_unknown_fields on protocol structs; Task 2). No wrapper helper.

  // Step 4 — per-peer rate limit (PRD §7.1).
  let peer_cap: i64 = lemmy_api::governance::config::get_int(
    &mut lemmy_api::governance::config::ConfigCache::new(),
    pool,
    lemmy_api::governance::config::Scope::Instance,
    "federation.inbound.per_peer_rate_per_hour",
  )
  .await?;
  let bucket = current_hour_bucket();
  let exceeded_peer = {
    let mut counts = rate_per_peer_counts().lock();
    // Opportunistic prune of stale buckets (older than current - 1).
    counts.retain(|(_, b), _| *b >= bucket - 1);
    let entry = counts.entry((peer_domain.clone(), bucket)).or_insert(0);
    *entry = entry.saturating_add(1);
    i64::from(*entry) > peer_cap
  };
  if exceeded_peer {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "rate_limit_peer",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationPeerRateLimitExceeded.into());
  }

  // Step 5 — per-actor rate limit (PRD §7.2; attestations only via trait
  // default no-op for other activity types).
  activity.check_per_actor_rate_limit(context).await?;

  // Step 6 — replay nonce (PRD §7.5).
  let nonce_form = FederationInboxNonceInsertForm {
    peer_instance: peer_domain.clone(),
    activity_id: activity_id.clone(),
  };
  let nonce_result = diesel::insert_into(federation_inbox_nonce::table)
    .values(&nonce_form)
    .execute(conn)
    .await;
  if let Err(diesel::result::Error::DatabaseError(
    diesel::result::DatabaseErrorKind::UniqueViolation,
    _,
  )) = nonce_result
  {
    log_inbox_drop(
      &peer_domain,
      Some(&activity_id),
      "replay",
      None,
      ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY,
      conn,
    )
    .await?;
    return Err(LemmyErrorType::FederationActivityReplayed.into());
  }
  nonce_result?;  // surface other DB errors

  // Step 7 — Phase-6 (or new label) domain handler.
  inner(activity, context).await
}

// -----------------------------------------------------------------
// log_inbox_drop — write federation_inbox_dropped_log row + matching
// governance_log entry, inside one tx (multi-write atomic per
// feedback_multi_write_handlers_need_transactions.md).
// -----------------------------------------------------------------
pub(crate) async fn log_inbox_drop(
  peer_domain: &str,
  activity_id: Option<&str>,
  reason: &str,
  excerpt: Option<&str>,
  entry_kind: &'static str,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  let form = FederationInboxDroppedLogInsertForm {
    source_instance: peer_domain.to_string(),
    activity_id: activity_id.map(str::to_string),
    drop_reason: reason.to_string(),
    payload_excerpt: excerpt.map(str::to_string),
  };
  let payload = json!({
    "peer_domain": peer_domain,
    "activity_id": activity_id,
    "reason": reason,
  });
  conn
    .run_transaction(|conn| {
      async move {
        diesel::insert_into(federation_inbox_dropped_log::table)
          .values(&form)
          .execute(conn)
          .await?;
        governance_log::append(
          &mut (&mut *conn).into(),
          entry_kind,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await
}

// -----------------------------------------------------------------
// evict_oldest_unreviewed_if_needed — PRD §7.3 storage-cap eviction.
// Called at the HEAD of each receive_remote_* SUCCESS path before
// the insert (per DQ #275 option-a). Non-rejecting: drops the oldest
// unreviewed row for the peer if the cap is reached, emits a
// federation_inbox_dropped_log entry + governance_log
// `_DROPPED_STORAGE_CAP_EVICTED`. table_name is the literal SQL
// table name; the helper uses sql_query for the COUNT + DELETE
// because Diesel's DSL can't accept a table_name parameter.
// -----------------------------------------------------------------
async fn evict_oldest_unreviewed_if_needed(
  peer_domain: &str,
  table_name: &str,
  cap: i64,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  // SELECT COUNT(*) WHERE source_instance = $1 AND admin_reviewed_at IS NULL
  // If count >= cap: DELETE oldest WHERE source_instance = $1 AND admin_reviewed_at IS NULL ORDER BY received_at LIMIT 1
  // Then log_inbox_drop with entry_kind _DROPPED_STORAGE_CAP_EVICTED.
  // Implementation: diesel::sql_query for both because table is parameterised.
  // ... (full body in task 4 IMPLEMENT) ...
  Ok(())
}

// -----------------------------------------------------------------
// receive_remote_moderation_label — fills the Phase-6 stub.
// PRD §9.4 verbatim except the peer_trust_level_at_receipt fill-in
// (DQ #273 option-a: re-query inside inner).
// -----------------------------------------------------------------
pub async fn receive_remote_moderation_label(
  activity: PublishLabel,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  let peer_domain = activity
    .actor
    .inner()
    .domain()
    .map(str::to_string)
    .unwrap_or_default();
  let actor_url = activity.actor.inner().to_string();
  let target_url = activity.object.target.to_string();
  let label = activity.object.label.clone();
  let summary = activity.object.summary.clone();
  let published_at = activity.object.published;
  let activity_id = activity.id.to_string();

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  // PRD §7.3 storage-cap eviction at insert time.
  let cap: i64 = lemmy_api::governance::config::get_int(
    &mut lemmy_api::governance::config::ConfigCache::new(),
    pool,
    lemmy_api::governance::config::Scope::Instance,
    "federation.inbound.per_peer_storage_cap",
  )
  .await?;
  evict_oldest_unreviewed_if_needed(&peer_domain, "remote_moderation_label", cap, conn).await?;

  // DQ #273 option-a: re-query trust inside inner.
  let trust = federation_inbox_check_peer_trust(&peer_domain, conn).await?;

  let form = RemoteModerationLabelInsertForm {
    source_instance: peer_domain.clone(),
    actor_url: actor_url.clone(),
    target_url: target_url.clone(),
    label: label.clone(),
    summary: summary.clone(),
    published_at,
    signature: String::new(),  // DQ-FED-IN-1 carry-forward (PRD §12.1)
    local_case_id: None,       // ADR-006
    peer_trust_level_at_receipt: Some(trust),
  };
  let payload = json!({
    "source_instance": peer_domain,
    "actor_url": actor_url,
    "target_url": target_url,
    "label": label,
    "activity_id": activity_id,
  });

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        diesel::insert_into(remote_moderation_label::table)
          .values(&form)
          .execute(conn)
          .await?;
        governance_log::append(
          &mut (&mut *conn).into(),
          ENTRY_KIND_FEDERATION_LABEL_RECEIVED,
          payload,
          None,
        )
        .await?;
        Ok(())
      }
      .scope_boxed()
    })
    .await;
  if let Err(e) = &outcome {
    // Best-effort persist_failed emit OUTSIDE the rollback. Swallow
    // a secondary failure — the original error is what we surface.
    let _ = governance_log::append(
      &mut context.pool(),
      ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED,
      json!({
        "peer_domain": peer_domain,
        "activity_id": activity_id,
        "table": "remote_moderation_label",
        "error": format!("{e}"),
      }),
      None,
    )
    .await;
  }
  outcome
}
```

**Phase-6 receive_remote_* modifications (Task 4):** at the head of
each existing fn (`receive_remote_sanction_notice` at `:105` and
`receive_remote_trust_attestation` at `:195`), insert the
`evict_oldest_unreviewed_if_needed` call BEFORE the tx; wrap the
existing `conn.run_transaction(...)` call so a tx failure triggers a
best-effort `_persist_failed` emit OUTSIDE the rollback. Pattern
mirrors the `receive_remote_moderation_label` body above.

### 10.5 Per-handler patches (Tasks 5/6/7)

**Mirror:** PRD §9.3 verbatim. Each Phase-6 `Activity::receive` body
becomes a one-line `wrap_governance_inbound` call. Each file ALSO adds
the `impl GovernanceInboundActivity for <type>` block.

```rust
// crates/apub/activities/src/governance/publish_sanction_notice.rs (Task 5)
// REPLACE the body of `async fn receive` at line :103-110 with:

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    crate::governance::inbox::wrap_governance_inbound(self, context, |a, c| async move {
      crate::governance::inbox::receive_remote_sanction_notice(a, c).await
    }).await
  }

// ADD the trait impl (anywhere in the file; idiomatic placement is
// directly after the `impl Activity` block at :47-111):

#[async_trait::async_trait]
impl crate::governance::inbox::GovernanceInboundActivity for PublishSanctionNotice {
  fn activity_id(&self) -> &Url { &self.id }
  fn actor_domain(&self) -> LemmyResult<String> {
    self.actor.inner().domain()
      .map(str::to_string)
      .ok_or_else(|| LemmyErrorType::Unknown(
        format!("PublishSanctionNotice actor {} has no domain", self.actor.inner())
      ).into())
  }
  fn payload_size_bytes(&self) -> LemmyResult<usize> {
    Ok(serde_json::to_vec(self)?.len())
  }
  fn payload_size_cap_key(&self) -> &'static str {
    "federation.inbound.max_payload_bytes_sanction_notice"
  }
  // check_per_actor_rate_limit uses the trait default no-op.
}
```

**Same shape for Tasks 6 and 7:**

- Task 6 — `publish_trust_attestation.rs` — `payload_size_cap_key`
  returns `"federation.inbound.max_payload_bytes_trust_attestation"`;
  `check_per_actor_rate_limit` is OVERRIDDEN to actually rate-limit
  (PRD §7.2 — only attestations carry this gate). The override reads
  `federation.inbound.per_actor_attestation_rate_per_hour`, hashes the
  `subject_url` from the typed protocol (the existing object stub
  carries the `subject` URL in `rest`), increments the
  `rate_per_actor_counts` static, and returns `Err(LemmyErrorType::FederationActorRateLimitExceeded.into())`
  on exceed AFTER calling `log_inbox_drop` with `_DROPPED_RATE_LIMIT_ACTOR`.
- Task 7 — `publish_label.rs` — REPLACES the Phase-6 stub
  receive body at `:27-30`; `payload_size_cap_key` returns
  `"federation.inbound.max_payload_bytes_moderation_label"`;
  `check_per_actor_rate_limit` uses the default no-op. The `inner`
  closure calls `crate::governance::inbox::receive_remote_moderation_label`
  (Task 4).

### 10.6 Phase-6 `sanction_notice_round_trip` fixture update (Task 9)

**Mirror:** fed-in-a `seed_federation_peer` at
`crates/server/tests/e2e.rs:15106-15130` (verbatim shape). The
Allowlist insert lands IMMEDIATELY before the
`ActivityTrait::receive` call at `:5244`.

```rust
// crates/server/tests/e2e.rs — INSERT before line :5243 (the
// `ActivityTrait::verify(...)` call) inside the
// `sanction_notice_round_trip` test, against the instance-B DB.

  // v1-federation-inbound-b: Allowlist the test peer so the wrapper's
  // peer-trust gate (Task 4) admits the activity. PRD §5.4 designed
  // breakage — fixture-only, no `_unchecked` variant per PRD §5.4 +
  // §11.4.
  {
    use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
    use lemmy_db_schema_file::enums::FederationPeerTrust;
    use lemmy_db_schema_file::schema::{federation_peer, instance};
    let mut async_conn_b_fixture = AsyncPgConnection::establish(&url_b).await?;
    // Look up instance-a.test by domain (created by Phase 6 fixture).
    let peer_instance_id: i32 = instance::table
      .filter(instance::domain.eq("instance-a.test"))
      .select(instance::id)
      .first::<i32>(&mut async_conn_b_fixture)
      .await?;
    let form = FederationPeerInsertForm {
      instance_id: lemmy_db_schema::newtypes::InstanceId(peer_instance_id),
      trust_level: Some(FederationPeerTrust::Allowlisted),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(&mut async_conn_b_fixture)
      .await?;
  }
```

### 10.7 Handler-e2e module (Task 9)

**Mirror:** `crates/server/tests/e2e.rs:15093-15151` (fed-in-a's
`mod v1_federation_inbound_a_fixtures` — Case A; uniform
`LemmyResult<()>` outer; bare `?`; one `governance_fixtures::bootstrap()`
per test) + `feedback_lemmy_error_no_std_error.md` Case A binding.

Goes at file end (`wc -l crates/server/tests/e2e.rs` = **15,482** at
HEAD `49a0d6b1d` — re-verify at task start; per
`feedback_junior_worker_e2e_edit_hang.md` strict anchor-append).

```rust
mod v1_federation_inbound_b_fixtures {
  use super::*;
  use activitypub_federation::traits::Activity as ActivityTrait;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_apub_activities::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
  use lemmy_db_schema::source::governance::{
    federation_peer::FederationPeerInsertForm,
    federation_inbox_nonce::FederationInboxNonceInsertForm,
    remote_sanction_notice::RemoteSanctionNotice,
    remote_moderation_label::RemoteModerationLabel,
  };
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{
    federation_inbox_nonce,
    federation_peer,
    governance_log,
    instance,
    remote_moderation_label,
    remote_sanction_notice,
  };
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};

  // 5 test fns: blocklisted_peer_returns_403, per_peer_rate_limit_returns_429,
  // replayed_activity_returns_409, allowlisted_happy_path_persists_advisory_row,
  // moderation_label_handler_persists_and_logs.
  // Full bodies in §13 Task 9 IMPLEMENT.
}
```

**The 5 test bodies live in §13 Task 9's IMPLEMENT block** (kept out
of §10 because they require ~200-400 lines of fixture scaffolding per
test). Each test asserts (i) a specific `LemmyErrorType` variant, (ii)
the resolved HTTP `StatusCode` per the §10.1 mapper (`403` / `429` /
`409`), (iii) the matching `federation_inbox_dropped_log` row presence
(or absence for happy path), and (iv) the matching `governance_log`
entry kind.

### 10.8 Replay-cleanup cron block (Task 8)

**Mirror:** `crates/routes/src/utils/scheduled_tasks.rs:260-288`
(`appeal_window_expiry` hourly tick — closest pattern: hourly default,
atomic-bool guard, env-var test-disable).

```rust
// crates/routes/src/utils/scheduled_tasks.rs — APPEND a new scheduler
// block AFTER the existing sponsor_liability_grace block (~line :360).

// Concurrency sentinel — mirrors APPEAL_WINDOW_EXPIRY_RUNNING /
// SPONSOR_LIABILITY_GRACE_RUNNING.
static FED_REPLAY_CLEANUP_RUNNING: AtomicBool = AtomicBool::new(false);
struct FedReplayCleanupRunningGuard;
impl Drop for FedReplayCleanupRunningGuard {
  fn drop(&mut self) {
    FED_REPLAY_CLEANUP_RUNNING.store(false, Ordering::Release);
  }
}

// v1-federation-inbound-b: replay-cleanup cron tick. Delete
// federation_inbox_nonce rows older than
// `federation.inbound.replay_window_days` (default 7); interval read
// from `federation.inbound.replay_cleanup_cron_interval_minutes`
// (default 60).
//
// Disabled in tests via BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1.
let context_fed_replay = context.reset_request_count();
let fed_replay_pool = &mut context.pool();
let fed_replay_interval_minutes_i64: i64 = lemmy_api::governance::config::get_int(
  &mut lemmy_api::governance::config::ConfigCache::new(),
  fed_replay_pool,
  lemmy_api::governance::config::Scope::Instance,
  "federation.inbound.replay_cleanup_cron_interval_minutes",
)
.await
.unwrap_or(60);
let fed_replay_interval_minutes: u32 =
  u32::try_from(fed_replay_interval_minutes_i64).unwrap_or(60);
scheduler
  .every(CTimeUnits::minutes(fed_replay_interval_minutes))
  .run(move || {
    let context = context_fed_replay.reset_request_count();
    async move {
      if std::env::var("BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB").as_deref() == Ok("1") {
        return;
      }
      if FED_REPLAY_CLEANUP_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
      {
        warn!("federation_inbox_nonce_cleanup: previous batch still running, skipping this tick");
        return;
      }
      let _guard = FedReplayCleanupRunningGuard;
      let pool = &mut context.pool();
      let mut cache = lemmy_api::governance::config::ConfigCache::new();
      let window_days_i64 = lemmy_api::governance::config::get_int(
        &mut cache,
        pool,
        lemmy_api::governance::config::Scope::Instance,
        "federation.inbound.replay_window_days",
      )
      .await
      .unwrap_or(7);
      let conn_pool = &mut context.pool();
      let conn_result = lemmy_diesel_utils::connection::get_conn(conn_pool).await;
      match conn_result {
        Ok(mut conn) => {
          let deleted = lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than(
            window_days_i64,
            &mut conn,
          )
          .await
          .inspect_err(|e| warn!("Failed federation_inbox_nonce cleanup: {e}"))
          .unwrap_or(0);
          if deleted > 0 {
            info!("federation_inbox_nonce cleanup: deleted {deleted} rows older than {window_days_i64} days");
          }
        }
        Err(e) => warn!("federation_inbox_nonce cleanup: get_conn failed: {e}"),
      }
    }
  });
```

**Notes:** uses the same `clokwerk::AsyncScheduler` pattern Lemmy
already wires; no new dependency. The `inspect_err(...).unwrap_or(0)`
shape mirrors `appeal_window_expiry`'s error-handling discipline
(`.inspect_err(|e| warn!(...)).ok();`).

---

## 11. Files to change

### `lemmy_utils` crate

- `crates/utils/src/error.rs` — INSERT 6 new `LemmyErrorType` variants
  + 6 new `status_code()` match-arms. **Task 1**.

### `lemmy_apub_objects` crate

- `crates/apub/objects/src/protocol/governance/sanction_notice.rs` —
  ADD `#[serde(deny_unknown_fields)]` on `SanctionNoticeProtocol`.
  **Task 2**.
- `crates/apub/objects/src/protocol/governance/trust_attestation.rs`
  — ADD `#[serde(deny_unknown_fields)]` on `TrustAttestationProtocol`.
  **Task 2**.
- `crates/apub/objects/src/protocol/governance/moderation_label.rs`
  — ADD `#[serde(deny_unknown_fields)]` on the label protocol
  struct. **Task 2**.

### `lemmy_db_schema` crate

- `crates/db_schema/src/source/governance/governance_log.rs` — ADD
  1 new const `ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED`. **Task 3**.

### `lemmy_api` crate

- `crates/api/api/src/governance/governance_log.rs` — ADD 1 new
  `pub use` re-export. **Task 3**.

### `lemmy_apub_activities` crate

- `crates/apub/activities/src/governance/inbox.rs` — APPEND
  `GovernanceInboundActivity` trait + `wrap_governance_inbound` +
  5 check helpers (peer-trust, size, per-peer-rate, per-actor-rate
  via trait default, replay) + `log_inbox_drop` +
  `evict_oldest_unreviewed_if_needed` + `receive_remote_moderation_label`
  + 2 static `OnceLock<Mutex<HashMap<...>>>` rate-limit cells; ALSO
  MODIFY `receive_remote_sanction_notice` (`:105`) and
  `receive_remote_trust_attestation` (`:195`) to insert
  `evict_oldest_unreviewed_if_needed` at fn head + wrap tx in
  best-effort `_persist_failed` emit on rollback. **Task 4**.
- `crates/apub/activities/src/governance/publish_sanction_notice.rs`
  — REPLACE `Activity::receive` body (`:103-110`) with one-line
  `wrap_governance_inbound` call + ADD `impl GovernanceInboundActivity`
  block. **Task 5**.
- `crates/apub/activities/src/governance/publish_trust_attestation.rs`
  — REPLACE `Activity::receive` body (`:86-93`) with one-line
  `wrap_governance_inbound` call + ADD `impl GovernanceInboundActivity`
  block (overrides `check_per_actor_rate_limit` for the attestation
  per-actor gate). **Task 6**.
- `crates/apub/activities/src/governance/publish_label.rs` — REPLACE
  the stub `receive` body (`:27-30`) with one-line
  `wrap_governance_inbound` call calling
  `receive_remote_moderation_label` + ADD `impl GovernanceInboundActivity`
  block. **Task 7**.

### `lemmy_routes` crate

- `crates/routes/src/utils/scheduled_tasks.rs` — APPEND new scheduler
  block for `federation_inbox_nonce` cleanup cron + 1 new static
  `FED_REPLAY_CLEANUP_RUNNING` guard. **Task 8**.

### `lemmy_server` crate

- `crates/server/tests/e2e.rs` — IN-PLACE Edit at the
  `sanction_notice_round_trip` block (~line :5243) inserting the
  `federation_peer` Allowlist fixture (~10 lines); APPEND new
  `mod v1_federation_inbound_b_fixtures` at file end. **Task 9**.

### Meta files

- `.claude/rules/governance-log-entry-kind-registry.md` — APPEND
  new `## v1-federation-inbound-b entry kinds (1, this sub-phase)`
  section; EDIT the v1-federation-inbound-a Deferred bullet (remove
  `federation_inbound_persist_failed`); BUMP acceptance-invariant
  count `54 → 55`. **Task 3**.
- `.claude/PRPs/reports/v1-federation-inbound-b-retro.md` — CREATE.
  **Task 10**.

### Files explicitly NOT touched

- `crates/db_schema/src/source/governance/federation_peer.rs` —
  fed-in-a-shipped; `-b` CALLS `federation_inbox_check_peer_trust`
  but does not modify the helper.
- `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`
  — fed-in-a-shipped; `-b` CALLS `delete_older_than` from the cron
  block but does not modify the model.
- `crates/db_schema/src/source/governance/federation_inbox_dropped_log.rs`
  — fed-in-a-shipped; `-b` CALLS the InsertForm but does not modify.
- `crates/db_schema/src/source/governance/remote_moderation_label.rs`
  — fed-in-a-shipped; `-b` consumes the model.
- `crates/db_schema/src/source/governance/{remote_sanction_notice,federation_attestation}.rs`
  — fed-in-a-extended; `-b` does not touch the model files.
- `crates/db_schema_file/src/schema.rs` — fed-in-a-shipped; `-b`
  reads but does not modify.
- `crates/api/api/src/governance/config.rs` — fed-in-a-shipped;
  `-b` CALLS `get_int` / `get_text` but does not add keys (the 11
  `federation.inbound.*` keys are fed-in-a-shipped).
- `crates/apub/apub/src/governance/` — the `lemmy_apub` crate's
  governance dir contains only `mod.rs` / `outbox.rs` / `verify.rs`
  (verified at plan-time via `ls`). NO `inbox.rs` in `lemmy_apub`;
  `-b` does NOT create one (PRECON-2; PRD §5.2/§9.1 descriptor error
  superseded).
- `crates/api/api/src/governance/federation_inbox/` — `-c` (admin
  endpoints).
- `crates/db_views/federation_inbox/` — `-c`.
- `crates/api/routes/src/lib.rs` / `governance.rs` — no new routes
  in `-b`.
- `crates/utils/src/rate_limit/` — the existing Lemmy rate-limit
  module is NOT extended in `-b`. Planner DQ #277 records the
  decision (in-module statics, not extension of the shared module).
- `migrations/**` — NO new migration in `-b` (PRECON-1).
- `Cargo.toml` / `Cargo.lock` / `rust-toolchain.toml` /
  `.coderabbit.yaml` — no toolchain or dep change.
- `.github/workflows/**` — PRECON-3; Shape G dormant until 2026-06-01.

## 12. NOT building in v1-federation-inbound-b

Per brief §2.2 + §0 carving (BINDING):

- **No admin REST endpoints** (PRD §6 `GET .../inbox`, `POST .../cross-link`,
  `POST .../dismiss`) — `-c`. `-b` ships zero new routes, zero new
  DTOs, zero new handlers under `crates/api/api/src/governance/federation_inbox/`
  or `crates/api/routes/`.
- **No `crates/db_views/federation_inbox/` view crate** (PRD §6.1) —
  `-c`.
- **No OQ-FED-IN-1 pseudonym rendering** — `-c` (admin-surface
  concern; `-b` stores `actor_url` / `target_url` TEXT as fed-in-a's
  schema already defines).
- **No `webauthn-rs` step-up on trust-change** (PRD §12.5) — `-c`
  (gates the `-c` admin `POST .../peers/{id}/trust` endpoint, which
  `-b` does not ship; fed-in-a's `federation_peer_upsert_trust`
  helper exists but `-b` does not wire it to any HTTP endpoint).
- **No `_cross_linked` / `_dismissed` entry-kind consts** — `-c`
  (fed-in-a §10.7 "Deferred"). `-b` defines ONLY
  `federation_inbound_persist_failed`.
- **No `receive_remote_*_unchecked` variant** — PRD §5.4 names this
  a footgun; the only sanctioned path is the test-fixture update in
  Task 9 (Allowlist the test peer in `federation_peer` before
  `receive`).
- **No outbound change** — PRD §11.1 unchanged: outbound stays
  `ActivitySendTargets::to_all_instances()`; `federation_peer.trust_level`
  does NOT narrow outbound in v1 (v2).
- **No fork-local lint guards** (PRD §12.8) — meta-edit, advisor-side,
  out of `-b`.
- **No new migration / schema change.** fed-in-a substrate complete.
  **If ANY §13 task implies a migration under `crates/db_schema/migrations/**`
  or `migrations/**`, or a `crates/db_schema_file/src/schema.rs` edit,
  STOP and file `kind: "blocker"`.**
- **No extension of Lemmy's `crates/utils/src/rate_limit/` token-bucket
  module.** `-b`'s in-memory rate counters live as module-local statics
  inside `crates/apub/activities/src/governance/inbox.rs` (Task 4).
  Planner DQ #277 records the design call.

**Hard out-of-scope (per PRD §2.2):** auto-apply (v3 / ADR-006 —
`-b`'s wrapper persists advisory rows with `local_case_id = NULL`,
NEVER auto-applies); reputation portability (v2/v3); cross-instance
jury (v3); per-community-per-peer trust (v2); OPA federation policy
(v2); SSRF-isolated fetch worker (v2 — `-b` stores remote URLs as
TEXT only, fetches nothing); federation discovery (v2).

---

## 13. Step-by-step tasks

> **Cohort dispatch:** Cohort A = Tasks 1 + 2 + 3 (3-way `[P]` —
> disjoint files; `crates/utils/src/error.rs`, 3 protocol structs in
> `crates/apub/objects/src/protocol/governance/`, and the dual-file
> `governance_log.rs` const + registry). Cohort B = Tasks 5 + 6 + 7
> (3-way `[P]` — 3 disjoint `crates/apub/activities/src/governance/publish_*.rs`
> files; all `requires: task 4`). Tasks 0, 4, 8, 9, 10 are barriers.
>
> **PRECON-3 laptop-shape (NOT Shape G):** each impl-task raises
> `kind: "validate-pending-laptop"` post-push naming §15 DoD commands
> verbatim. Advisor laptop session runs the commands sequentially per
> `.claude/rules/advisor-orchestrator.md` §5.2 and mutates the entry.

### Task 0: Pre-flight harness audit + branch verification

**FILES:**

```yaml
creates: []
modifies: []
```

**Probes (per `pre-phase-harness-audit.md` — R5):**

```bash
# Probe 0 — Docker daemon (e2e harness uses testcontainers)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe -1 — submodule init
git submodule status > /tmp/fed-in-b-task0-submodule.log 2>&1
if grep -q '^-' /tmp/fed-in-b-task0-submodule.log; then
  git submodule update --init --recursive > /tmp/fed-in-b-task0-submodule-init.log 2>&1
fi

# Probe 1 — branch
git branch --show-current
# EXPECT: phase-v1-federation-inbound-b

# Probe 2 — fed-in-a substrate present (PRECON-1 re-verify)
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 54
grep -c "federation.inbound." crates/api/api/src/governance/config.rs
# EXPECT: > 11

# Probe 3 — inbox.rs at the lemmy_apub_activities path (PRECON-2)
test -f crates/apub/activities/src/governance/inbox.rs && echo "inbox.rs OK" || {
  echo "ERROR: lemmy_apub_activities/governance/inbox.rs missing — PRECON-2 violated"
  exit 1
}
test ! -f crates/apub/apub/src/governance/inbox.rs && echo "apub/apub/governance/inbox.rs absent OK" || {
  echo "ERROR: PRD §5.2/§9.1 descriptor-error path exists — investigate"
  exit 1
}

# Probe 4 — Phase-6 receive_remote_* + 3 publish_*.rs receive impls enumerated
grep -nE 'pub async fn receive_remote_(sanction_notice|trust_attestation)' \
  crates/apub/activities/src/governance/inbox.rs
# EXPECT: 2 matches at lines ~:105 and ~:195
grep -nE 'async fn receive' \
  crates/apub/activities/src/governance/publish_*.rs
# EXPECT: 3 matches (publish_sanction_notice.rs, publish_trust_attestation.rs, publish_label.rs)

# Probe 5 — designed-breakage sweep (PRD §5.4 + watchpoint #3)
grep -nE 'PublishSanctionNotice::receive|PublishTrustAttestation::receive|PublishLabel::receive|ActivityTrait::receive' \
  crates/server/tests/e2e.rs
# EXPECT: 1 active direct-call site at ~line :5244 inside sanction_notice_round_trip;
# other matches are comments only.

# Probe 6 — fed-in-a fixtures module present
grep -n "mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs
# EXPECT: 1 match at ~line :15093

# Probe 7 — e2e.rs line count
wc -l crates/server/tests/e2e.rs
# EXPECT: 15,482 (re-verify; baseline for the anchor-Edit append in Task 9)

# Probe 8 — Phase-6 receive bodies are NOT already wrapped (idempotency)
grep -n "wrap_governance_inbound" \
  crates/apub/activities/src/governance/publish_*.rs
# EXPECT: no matches (else trunk contamination — file kind: "blocker")

# Probe 9 — DQ pending state
python3 -c "import json; d=json.load(open('.claude/decision-queue.json')); print('pending:', [(e['id'], e.get('kind')) for e in d.get('pending',[])])"

# Probe 10 — federation.inbound.* config keys land (PRECON-1)
grep -nE "federation\\.inbound\\." migrations/2026-05-17-000000-0000_add_federation_inbound_v1/up.sql 2>&1 | wc -l
# EXPECT: 11 INSERTs

# Probe 11 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/utils/src/error\\.rs|crates/apub/objects/src/protocol/governance/|crates/apub/activities/src/governance/|crates/db_schema/src/source/governance/governance_log\\.rs|crates/api/api/src/governance/governance_log\\.rs|crates/routes/src/utils/scheduled_tasks\\.rs|crates/server/tests/e2e\\.rs|\\.claude/rules/governance-log-entry-kind-registry\\.md")) | {number, title, headRefName}'

# Probe 12 — wrapper availability
ls scripts/brehon/cargo-check.sh scripts/brehon/cargo-check.bat 2>&1 | head -3
ls scripts/brehon/cargo-clippy.sh scripts/brehon/cargo-clippy.bat 2>&1 | head -3
ls scripts/brehon/cargo-test.sh scripts/brehon/cargo-test.bat 2>&1 | head -3
```

**EXPECT:** Probes 0-8, 10-12 exit 0; Probe 9 informational. On
any unexpected baseline, file `kind: "blocker"` DQ.

**No commit at Task 0** — verification only.

### Task 1 [P]: Add 6 new `LemmyErrorType` variants + extend `status_code()` mapper

**FILES:**

```yaml
creates: []
modifies:
  - crates/utils/src/error.rs   # add 6 variants + 6 status_code arms
```

**ACTION:** insert 6 new `LemmyErrorType` variants adjacent to the
existing Federation cluster (line ~:114-122); extend the
`status_code()` match in `impl ResponseError for LemmyError`
(line :223-230) with 6 new arms returning the PRD §5.3 codes.

**IMPLEMENT (file 1 of 1):** in `crates/utils/src/error.rs`,
apply §10.1's two blocks verbatim. The enum is `#[non_exhaustive]`
(line :10) so callers do not need to be updated.

**MIRROR:** §10.1; existing Federation-prefixed variants
(`FederationDisabled` at `:143`) for naming convention.

**GOTCHA (cfg-select):** the `ResponseError` impl is inside
`cfg_select! { feature = "full" => { ... } }` (line :162-350). Adding
the status-code arms inside that block keeps the build conditional.
`cargo check --workspace --features full` exercises this.

**GOTCHA (R9):** variants are forward-compatible (`#[non_exhaustive]`).
No caller changes required.

**Push and exit (PRECON-3):**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task1-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task1-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): LemmyErrorType — 6 new federation inbound variants + HTTP status mapper (task 1)`

### Task 2 [P]: Add `#[serde(deny_unknown_fields)]` to 3 governance protocol structs

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/objects/src/protocol/governance/sanction_notice.rs   # +1 attr
  - crates/apub/objects/src/protocol/governance/trust_attestation.rs # +1 attr
  - crates/apub/objects/src/protocol/governance/moderation_label.rs  # +1 attr
```

**ACTION:** add `#[serde(deny_unknown_fields)]` adjacent to the
existing `#[serde(rename_all = "camelCase")]` attribute on each
of the 3 governance protocol structs (PRD §7.4 + §9.2 step 3).

**IMPLEMENT (file 1 of 3):** in `crates/apub/objects/src/protocol/governance/sanction_notice.rs`,
insert per §10.2. **Pre-edit:** read the existing struct's field
list (line :40-...); if `#[serde(flatten)]` is present at the outer
struct level, STOP and file `kind: "blocker"`.

**IMPLEMENT (file 2 of 3):** same in `trust_attestation.rs:33`.

**IMPLEMENT (file 3 of 3):** same in `moderation_label.rs:30`.

**MIRROR:** §10.2; existing `#[serde(rename_all = "camelCase")]` +
`#[serde(rename = "type")]` attrs.

**GOTCHA (test serialise compat):** audit any test fixture using
`serde_json::from_str(...)` on these 3 protocol structs for extra
fields; plan-time check returned clean.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task2-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task2-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): protocol structs — deny_unknown_fields on 3 governance protocols (task 2)`

### Task 3 [P]: Declare `ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED` const + shim re-export + registry append

**FILES:**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/governance_log.rs       # 1 new const
  - crates/api/api/src/governance/governance_log.rs                 # 1 new pub use
  - .claude/rules/governance-log-entry-kind-registry.md             # +1 row + Deferred bullet edit + count 54→55
```

**ACTION:** add the new const declaration (db_schema), the shim
re-export (api), and the registry section per §10.3.

**IMPLEMENT (file 1 of 3):** in `crates/db_schema/src/source/governance/governance_log.rs`,
append `pub const ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED: &str =
"federation_inbound_persist_failed";` adjacent to the fed-in-a
cluster at lines :221-229.

**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/governance/governance_log.rs`,
add the `pub use ...::ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED;`
re-export alphabetically with the existing fed-in-a re-exports.

**IMPLEMENT (file 3 of 3):** in
`.claude/rules/governance-log-entry-kind-registry.md`, APPEND a new
`## v1-federation-inbound-b entry kinds (1, this sub-phase)` section
per §10.3 verbatim AND edit the v1-federation-inbound-a "Deferred"
bullet to remove the `federation_inbound_persist_failed` line AND
bump the acceptance-invariants count from **54 → 55**.

**MIRROR:** §10.3; fed-in-a's dual-file precedent at
`v1-federation-inbound-a.plan.md` §10.7.

**GOTCHA:** `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs`
MUST equal the count cited in the registry. After Task 3: **55**.

**GOTCHA (Junior write-permission to `.claude/rules/`):** the
registry file is at `.claude/rules/`. If the harness refuses the
write, file `kind: "blocker"` for advisor-side authorship per
`feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md` (fed-in-a's
Task 4 ran into this on the registry edit and was advisor-authored).

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task3-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task3-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): governance_log — 1 new federation_inbound_persist_failed const + registry append 54→55 (task 3)`

### Task 4: Add wrapper + 5 check helpers + GovernanceInboundActivity trait + storage-cap helper + label handler + Phase-6 receive_remote_* modifications

**Barrier — Cohort B (Tasks 5/6/7) `requires: task 4`.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/inbox.rs   # APPEND trait+wrapper+helpers+label handler; MODIFY 2 existing receive_remote_*
requires:
  - task: 1
    reason: "wrapper returns LemmyErrorType::FederationPeerBlocklisted/FederationPayloadTooLarge/FederationPeerRateLimitExceeded/FederationActivityReplayed — variants land in Task 1"
  - task: 3
    reason: "wrapper modifications to existing receive_remote_* emit ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED — const lands in Task 3"
```

**ACTION:** add `GovernanceInboundActivity` trait + `wrap_governance_inbound`
+ 5 check helpers + `log_inbox_drop` + `evict_oldest_unreviewed_if_needed`
+ `receive_remote_moderation_label` + 2 rate-limit static cells. Then
MODIFY existing `receive_remote_sanction_notice` (`:105-179`) and
`receive_remote_trust_attestation` (`:195-250`) to insert
`evict_oldest_unreviewed_if_needed` at fn head and wrap the existing
`conn.run_transaction(...)` call in best-effort `_persist_failed`
emit on rollback. All per §10.4 verbatim.

**Pre-task enumeration (R9 + watchpoint #2):**

```bash
rg "fn receive" crates/apub/activities/src/governance/publish_*.rs
# EXPECT 3 matches
```

**IMPLEMENT (file 1 of 1):** in `crates/apub/activities/src/governance/inbox.rs`,
apply §10.4 verbatim. New code lands at file tail (after
`insert_federation_attestation` at line :325). Modifications to
existing receive fns are surgical (eviction call + best-effort emit).

**MIRROR:** §10.4; PRD §9.2 + §9.4 verbatim; Phase-6 `receive_remote_sanction_notice`
`run_transaction` shape at `:157-176`; PRECON-2 path correction in
`inbox.rs:1-12` module doc.

**GOTCHA (PRECON-2 — load-bearing):** wrapper goes IN
`crates/apub/activities/src/governance/inbox.rs` (the
`lemmy_apub_activities` crate). NOT `crates/apub/apub/src/governance/inbox.rs`.
Watchpoint #1.

**GOTCHA (multi-write atomicity):** `log_inbox_drop` writes 2 rows
(drop-log + governance_log) — `run_transaction` required.
`receive_remote_moderation_label` writes 2 rows + optional best-effort
persist_failed OUTSIDE the tx.

**GOTCHA (DQ #273 option-a):** `receive_remote_moderation_label`
RE-QUERIES `federation_inbox_check_peer_trust(peer_domain, conn)`.

**GOTCHA (DQ #274 option-a):** all `federation.inbound.*` config
reads route through `lemmy_api::governance::config::get_int` against
`Scope::Instance`.

**GOTCHA (DQ #275 option-a):** storage-cap eviction at INSERT time
inside each `receive_remote_*`, NOT as a 6th pre-`inner` reject-gate.

**GOTCHA (rate-limit statics):** 2 `OnceLock<Mutex<HashMap<_, u32>>>`
cells. Tests use distinct peer domains to avoid cross-test pollution;
test (b) deliberately exercises cell exhaustion.

**GOTCHA (replay via UNIQUE):** `DatabaseError(UniqueViolation, _)`
detects duplicate `(peer_instance, activity_id)`. Other DB errors
propagate.

**GOTCHA (mid-tx pool reborrow):** `governance_log::append(&mut (&mut *conn).into(), ...)`
preserves Phase-6's `:166` reborrow pattern.

**VALIDATE:** R7 — `cargo test --no-run`.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task4-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task4-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-b-task4-test-norun.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): wrap_governance_inbound + 5 check helpers + log_inbox_drop + GovernanceInboundActivity trait + storage-cap + receive_remote_moderation_label (task 4)`

### Task 5 [P]: Patch `publish_sanction_notice.rs` — `Activity::receive` body + impl `GovernanceInboundActivity`

**Cohort B.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_sanction_notice.rs
requires:
  - task: 4
    reason: "this task calls wrap_governance_inbound + impl's GovernanceInboundActivity trait — both land in Task 4"
```

**ACTION:** replace the existing `Activity::receive` body (`:103-110`)
with a one-line `wrap_governance_inbound` call per §10.5; add
`impl GovernanceInboundActivity for PublishSanctionNotice` block.

**IMPLEMENT (file 1 of 1):** apply §10.5's first block + the trait-impl
block (placed after `impl Activity` at ~line :112).

**MIRROR:** §10.5; existing Phase-6 receive body at `:103-110`.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task5-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task5-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): publish_sanction_notice — wrap receive + impl GovernanceInboundActivity (task 5)`

### Task 6 [P]: Patch `publish_trust_attestation.rs` — `Activity::receive` body + impl `GovernanceInboundActivity` (with per-actor rate-limit override)

**Cohort B.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_trust_attestation.rs
requires:
  - task: 4
    reason: "uses wrap_governance_inbound + GovernanceInboundActivity + rate_per_actor_counts()/current_hour_bucket()/log_inbox_drop pub(crate) helpers from Task 4"
```

**ACTION:** replace the existing `Activity::receive` body (`:86-93`)
with a one-line `wrap_governance_inbound` call; add `impl
GovernanceInboundActivity for PublishTrustAttestation` block with
`check_per_actor_rate_limit` override that enforces PRD §7.2
(per-`subject_url` rate limit, keyed by hash, summed across peers).

**IMPLEMENT (file 1 of 1):** per §10.5 (adjusted for trust-attestation
per-actor rate-limit override):

```rust
#[async_trait::async_trait]
impl crate::governance::inbox::GovernanceInboundActivity for PublishTrustAttestation {
  fn activity_id(&self) -> &Url { &self.id }
  fn actor_domain(&self) -> LemmyResult<String> { /* ... */ }
  fn payload_size_bytes(&self) -> LemmyResult<usize> {
    Ok(serde_json::to_vec(self)?.len())
  }
  fn payload_size_cap_key(&self) -> &'static str {
    "federation.inbound.max_payload_bytes_trust_attestation"
  }
  async fn check_per_actor_rate_limit(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    // Read subject URL from self.object.rest.get("subject").
    // Hash it. Read federation.inbound.per_actor_attestation_rate_per_hour from governance_config.
    // Increment crate::governance::inbox::rate_per_actor_counts() entry at (subject_hash, current_hour_bucket()).
    // If > cap, call log_inbox_drop with ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR,
    // then return Err(LemmyErrorType::FederationActorRateLimitExceeded.into()).
    // Full body in IMPLEMENT (~50 lines mirroring §10.4 wrapper step 4 structure).
    Ok(())
  }
}
```

The `rate_per_actor_counts()` + `current_hour_bucket()` + `log_inbox_drop`
helpers are exposed as `pub(crate)` from Task 4's `inbox.rs`.

**MIRROR:** §10.4 (rate-limit cell pattern) + §10.5.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task6-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task6-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): publish_trust_attestation — wrap receive + impl GovernanceInboundActivity with per-actor rate gate (task 6)`

### Task 7 [P]: Patch `publish_label.rs` — fill stub `receive` + impl `GovernanceInboundActivity`

**Cohort B.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/publish_label.rs
requires:
  - task: 4
    reason: "calls wrap_governance_inbound + receive_remote_moderation_label (both Task 4); impl's GovernanceInboundActivity (Task 4)"
```

**ACTION:** REPLACE the existing stub `receive` body (`:27-30`) with
a one-line `wrap_governance_inbound` call delegating to
`crate::governance::inbox::receive_remote_moderation_label`; ADD
`impl GovernanceInboundActivity for PublishLabel` block (uses
trait-default `check_per_actor_rate_limit` no-op).

**IMPLEMENT (file 1 of 1):** per §10.5 (using
`"federation.inbound.max_payload_bytes_moderation_label"` for
`payload_size_cap_key`).

**MIRROR:** §10.5; existing Phase-6 stub at `:27-30`.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task7-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task7-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): publish_label — fill stub receive + impl GovernanceInboundActivity (task 7)`

### Task 8: Wire replay-cleanup cron in `scheduled_tasks.rs`

**Barrier.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/routes/src/utils/scheduled_tasks.rs   # APPEND scheduler block + 1 static guard
```

**ACTION:** append the replay-cleanup scheduler block per §10.8 after
the existing `sponsor_liability_grace` block (~line :360).

**IMPLEMENT (file 1 of 1):** apply §10.8 verbatim.

**MIRROR:** §10.8; `appeal_window_expiry` at `:260-288`;
`sponsor_liability_grace` at `:308-360`.

**GOTCHA (DQ #250):** fed-in-a's `delete_older_than` is `pub async fn`
in `lemmy_db_schema` (lib crate); Rust does NOT emit `dead_code`
lint for `pub` items in lib crates — NO suppression attribute
needed. Task 8 does NOT modify `federation_inbox_nonce.rs`.

**GOTCHA (test isolation):** `BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1`
disables the cron in tests (mirror of `BREHON_DISABLE_GRACE_CHECK_JOB`).

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task8-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task8-clippy.log 2>&1"'
```

**COMMIT:** `feat(v1-federation-inbound-b): scheduled_tasks — replay-cleanup cron wiring (task 8)`

### Task 9: Update Phase-6 fixture + add handler-e2e module

**Barrier.**

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # IN-PLACE Edit at sanction_notice_round_trip ~line 5243; APPEND new mod at file end
requires:
  - task: 1
    reason: "handler-e2e asserts FederationPeerBlocklisted / FederationPeerRateLimitExceeded / FederationActivityReplayed — variants land in Task 1"
  - task: 4
    reason: "handler-e2e exercises wrap_governance_inbound + receive_remote_moderation_label — Task 4"
  - task: 5
    reason: "happy-path test exercises Activity::receive via PublishSanctionNotice — patch lands in Task 5"
  - task: 6
    reason: "per-actor-rate test exercises PublishTrustAttestation patch — Task 6"
  - task: 7
    reason: "label-handler test exercises PublishLabel patch — Task 7"
  - task: 8
    reason: "test must run with BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1 set — env-var read added in Task 8"
```

**ACTION:** TWO Edits per `feedback_junior_worker_e2e_edit_hang.md`:

1. IN-PLACE Edit at the `sanction_notice_round_trip` test (~line
   :5243): insert a 10-line `federation_peer` Allowlist fixture
   immediately before the `ActivityTrait::verify(&activity, …)` call.
   Mirror §10.6 verbatim.
2. APPEND at file end (after `v1_federation_inbound_a_fixtures`'s
   closing `}` at ~line :15151): a new `mod v1_federation_inbound_b_fixtures`
   block per §10.7 with the 5 test fns fully implemented (asserting
   specific HTTP status codes via `ResponseError::status_code()` +
   `LemmyErrorType` variant equality + DB row presence + governance_log
   entry kind).

**IMPLEMENT (file 1 of 1):**

1. First Edit — anchor on the v0 Step 9 comment block (line :5237-5242)
   + the `ActivityTrait::verify(...)` line :5243. Insert the new
   10-line Allowlist fixture between the comment block and the verify
   call.
2. Second Edit — anchor on the closing `}` of
   `mod v1_federation_inbound_a_fixtures` (line :15151). Insert
   `\nmod v1_federation_inbound_b_fixtures {\n  ...\n}\n` AFTER it.

**Per-test bodies (full):**

```rust
mod v1_federation_inbound_b_fixtures {
  use super::*;
  use activitypub_federation::traits::Activity as ActivityTrait;
  use actix_web::error::ResponseError;
  use actix_web::http::StatusCode;
  use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_apub_activities::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
  use lemmy_db_schema::source::governance::{
    federation_peer::FederationPeerInsertForm,
    federation_inbox_nonce::FederationInboxNonceInsertForm,
    remote_moderation_label::RemoteModerationLabel,
    remote_sanction_notice::RemoteSanctionNotice,
  };
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{
    federation_inbox_dropped_log,
    federation_inbox_nonce,
    federation_peer,
    governance_log,
    instance,
    remote_moderation_label,
    remote_sanction_notice,
  };
  use lemmy_db_schema_file::InstanceId;
  use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};

  // Helper: bootstrap + seed a peer at a given trust (or no row at all).
  async fn bootstrap_with_peer(domain: &str, trust: Option<FederationPeerTrust>)
    -> LemmyResult<(/* container */, /* context */, /* db_url */ String, /* peer_instance_id */ InstanceId)> {
    let (container, context, db_url) = governance_fixtures::bootstrap().await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let peer_instance_id: i32 = diesel::insert_into(instance::table)
      .values((
        instance::domain.eq(domain),
        instance::published_at.eq(diesel::dsl::now),
      ))
      .returning(instance::id)
      .get_result(&mut conn).await?;
    if let Some(t) = trust {
      let form = FederationPeerInsertForm {
        instance_id: InstanceId(peer_instance_id),
        trust_level: Some(t),
        added_by_actor: None,
        notes: None,
      };
      diesel::insert_into(federation_peer::table).values(&form).execute(&mut conn).await?;
    }
    Ok((container, context, db_url, InstanceId(peer_instance_id)))
  }

  // (a) Blocklisted peer → 403.
  #[tokio::test(flavor = "multi_thread")]
  async fn blocklisted_peer_returns_403() -> LemmyResult<()> {
    let (_container, context, db_url, _peer_id) =
      bootstrap_with_peer("blocked.test", Some(FederationPeerTrust::Blocklisted)).await?;
    let activity = build_minimal_sanction_notice_activity("blocked.test")?;
    let result = ActivityTrait::receive(activity, &context).await;
    assert!(result.is_err(), "wrapper must reject Blocklisted peer");
    let err: LemmyError = result.err().unwrap();
    assert!(matches!(err.error_type, LemmyErrorType::FederationPeerBlocklisted));
    assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let drop_rows: i64 = federation_inbox_dropped_log::table
      .filter(federation_inbox_dropped_log::source_instance.eq("blocked.test"))
      .filter(federation_inbox_dropped_log::drop_reason.eq("blocklisted"))
      .count().get_result(&mut conn).await?;
    assert_eq!(drop_rows, 1);
    Ok(())
  }

  // (b) Per-peer rate exceeded → 429.
  #[tokio::test(flavor = "multi_thread")]
  async fn per_peer_rate_limit_returns_429() -> LemmyResult<()> {
    let (_container, context, db_url, _peer_id) =
      bootstrap_with_peer("rate-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    diesel::sql_query("INSERT INTO governance_config (scope, key, value_type, value_int) VALUES ('instance', 'federation.inbound.per_peer_rate_per_hour', 'int', 2)")
      .execute(&mut conn).await?;
    for i in 0..2 {
      let activity = build_unique_sanction_notice_activity("rate-test.test", i)?;
      let _ = ActivityTrait::receive(activity, &context).await;
    }
    let activity3 = build_unique_sanction_notice_activity("rate-test.test", 2)?;
    let result = ActivityTrait::receive(activity3, &context).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err.error_type, LemmyErrorType::FederationPeerRateLimitExceeded));
    assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
    Ok(())
  }

  // (c) Replayed activity id → 409.
  #[tokio::test(flavor = "multi_thread")]
  async fn replayed_activity_returns_409() -> LemmyResult<()> {
    let (_container, context, db_url, _peer_id) =
      bootstrap_with_peer("replay-test.test", Some(FederationPeerTrust::Allowlisted)).await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let nonce_form = FederationInboxNonceInsertForm {
      peer_instance: "replay-test.test".to_string(),
      activity_id: "https://replay-test.test/activities/create/1".to_string(),
    };
    diesel::insert_into(federation_inbox_nonce::table)
      .values(&nonce_form).execute(&mut conn).await?;
    let activity = build_sanction_notice_with_id(
      "replay-test.test",
      "https://replay-test.test/activities/create/1",
    )?;
    let result = ActivityTrait::receive(activity, &context).await;
    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err.error_type, LemmyErrorType::FederationActivityReplayed));
    assert_eq!(err.status_code(), StatusCode::CONFLICT);
    Ok(())
  }

  // (d) Happy path — Allowlisted peer.
  #[tokio::test(flavor = "multi_thread")]
  async fn allowlisted_happy_path_persists_advisory_row() -> LemmyResult<()> {
    let (_container, context, db_url, _peer_id) =
      bootstrap_with_peer("happy.test", Some(FederationPeerTrust::Allowlisted)).await?;
    let activity = build_minimal_sanction_notice_activity("happy.test")?;
    ActivityTrait::receive(activity, &context).await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let rows: i64 = remote_sanction_notice::table
      .filter(remote_sanction_notice::source_instance.eq("happy.test"))
      .count().get_result(&mut conn).await?;
    assert_eq!(rows, 1);
    let advisory: RemoteSanctionNotice = remote_sanction_notice::table
      .filter(remote_sanction_notice::source_instance.eq("happy.test"))
      .select(RemoteSanctionNotice::as_select())
      .first(&mut conn).await?;
    assert!(advisory.local_case_id.is_none(), "ADR-006: local_case_id MUST be NULL");
    Ok(())
  }

  // (e) New receive_remote_moderation_label happy path.
  #[tokio::test(flavor = "multi_thread")]
  async fn moderation_label_handler_persists_and_logs() -> LemmyResult<()> {
    let (_container, context, db_url, _peer_id) =
      bootstrap_with_peer("label.test", Some(FederationPeerTrust::Allowlisted)).await?;
    let activity = build_minimal_publish_label_activity("label.test")?;
    ActivityTrait::receive(activity, &context).await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let rows: i64 = remote_moderation_label::table
      .filter(remote_moderation_label::source_instance.eq("label.test"))
      .count().get_result(&mut conn).await?;
    assert_eq!(rows, 1);
    let label_row: RemoteModerationLabel = remote_moderation_label::table
      .filter(remote_moderation_label::source_instance.eq("label.test"))
      .select(RemoteModerationLabel::as_select())
      .first(&mut conn).await?;
    assert!(label_row.local_case_id.is_none(), "ADR-006: local_case_id MUST be NULL");
    let log_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("federation_label_received"))
      .count().get_result(&mut conn).await?;
    assert!(log_count >= 1);
    Ok(())
  }

  // -------- activity-construction helpers (Case A: LemmyResult<T>) --------
  // Each helper builds a minimal valid AP body mirroring Phase-6's
  // sanction_notice_round_trip activity-construction (e2e.rs :5050-5230).
  // Full bodies ~50 lines each — kept terse to fit Task 9 anchor-append.

  fn build_minimal_sanction_notice_activity(peer_domain: &str) -> LemmyResult<PublishSanctionNotice> {
    build_unique_sanction_notice_activity(peer_domain, 0)
  }
  fn build_unique_sanction_notice_activity(peer_domain: &str, seq: u32) -> LemmyResult<PublishSanctionNotice> {
    build_sanction_notice_with_id(peer_domain, &format!("https://{peer_domain}/activities/create/{seq}"))
  }
  fn build_sanction_notice_with_id(peer_domain: &str, activity_id: &str) -> LemmyResult<PublishSanctionNotice> {
    // Mirror Phase-6 sanction_notice_round_trip's activity-construction.
    // ... ~50 lines ...
    todo!("implement during impl-task — mirror Phase-6 e2e.rs :5050-5230")
  }
  fn build_minimal_publish_label_activity(peer_domain: &str) -> LemmyResult<lemmy_apub_activities::protocol::governance::publish_label::PublishLabel> {
    todo!("implement during impl-task — minimal PublishLabel with actor on peer_domain")
  }
}
```

**MIRROR:** §10.6 + §10.7;
`crates/server/tests/e2e.rs:15093-15151` (fed-in-a Case A); Phase-6
`sanction_notice_round_trip` for activity construction.

**GOTCHA (Case A):** every helper + every test fn returns
`LemmyResult<T>`. Bare `?` only. No `.map_err`. No `Box<dyn Error>`.

**GOTCHA (anchor-Edit on 15k-line file):** ONE in-place Edit + ONE
append. Re-verify `wc -l` at task start.

**VALIDATE:** R7.

**Push and exit:**

```yaml
commands:
  - 'cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task9-check.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task9-clippy.log 2>&1"'
  - 'cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-b-task9-test-norun.log 2>&1"'
```

(Phase-2 e2e execution: user-gate-4 post-finalize-merge per
`advisor-orchestrator.md` §3.2 + §5.2.)

**COMMIT:** `feat(v1-federation-inbound-b): e2e.rs — Phase-6 fixture Allowlist + handler-e2e module (5 tests: blocked/rate/replay/happy/label) (task 9)`

### Task 10: Retro

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-federation-inbound-b-retro.md
modifies: []
```

Author per `feedback_retro_not_report.md` +
`feedback_four_role_retro_signals.md`. One H2 per role with signals
+ lessons. Per-task complexity recorded per
`feedback_retro_task_complexity_score.md`. Promote any new lessons to
`.claude/lessons/feedback_*.md` in the same retro commit.

**Retro carry-forwards to harvest:** rate-limit static-cell design
(DQ #277); storage-cap eviction call-site (DQ #275);
`peer_trust_level_at_receipt` re-query (DQ #273);
`deny_unknown_fields` peer-compat impact.

**COMMIT:** `docs(retro): v1-federation-inbound-b session retro`

---

## 14. Testing strategy

- **Unit (compile-time):** `cargo check --workspace --features full`.
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`.
- **Test target compile (R7):** `cargo test --no-run --workspace --features full --test e2e` on Tasks 4, 9.
- **e2e execution (Phase-2 — user-gate-4):** 5 handler-e2e tests pass; Phase-6 `sanction_notice_round_trip` stays green.
- **Migration round-trip:** N/A (no migration).
- **Replay-cleanup cron:** compile + static guard binding; runtime out of `-b` e2e scope.

---

## 15. Validation commands (DoD)

> **PRECON-3 laptop-shape (NOT Shape G).** Per DQ #229. Each impl-task
> raises `kind: "validate-pending-laptop"` post-push naming §15 DoD
> commands verbatim with `--workspace --features full`. Mirrors
> fed-in-a §15.

### 15.1 Per-task workspace check (Tasks 1-9)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task<N>-check.log 2>&1"
```

**EXPECT:** exit 0.

### 15.2 Per-task clippy (Tasks 1-9 — R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task<N>-clippy.log 2>&1"
```

**EXPECT:** exit 0.

### 15.3 Test target compile (R7 — Tasks 4, 9)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/fed-in-b-task<N>-test-norun.log 2>&1"
```

**EXPECT:** exit 0.

### 15.4 Phase-2 e2e (post-finalize-merge — user-gate-4)

**(a) Local laptop bg** (~26 min, zero billed):

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-b-<sha>.log 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"
```

**(b) GH dispatch** (~26 min billed — escape hatch only):

```bash
gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-federation-inbound-b
```

**EXPECT:** all tests pass — 5 new `v1_federation_inbound_b_fixtures`
tests AND repaired `sanction_notice_round_trip`.

### 15.5 (Shape G section — DORMANT until 2026-06-01)

**NOT applicable.** Per PRECON-3 + DQ #229.

### 15.6 Cross-cutting verification

- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **55**.
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns **55**.
- [ ] `rg -n 'federation_inbound_persist_failed' .claude/rules/governance-log-entry-kind-registry.md` shows the v1-federation-inbound-b section.
- [ ] fed-in-a "Deferred" no longer contains `federation_inbound_persist_failed`.
- [ ] `grep -c "deny_unknown_fields" crates/apub/objects/src/protocol/governance/*.rs` returns 3.
- [ ] `grep -c "FederationPeerBlocklisted\|FederationPayloadTooLarge\|FederationSchemaInvalid\|FederationPeerRateLimitExceeded\|FederationActorRateLimitExceeded\|FederationActivityReplayed" crates/utils/src/error.rs` returns 12 (6 in enum + 6 in status_code).
- [ ] `grep -n "wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs` shows 1 match (def).
- [ ] `grep -c "wrap_governance_inbound" crates/apub/activities/src/governance/publish_*.rs` returns 3.
- [ ] `grep -c "impl crate::governance::inbox::GovernanceInboundActivity" crates/apub/activities/src/governance/publish_*.rs` returns 3.
- [ ] `grep -n "FED_REPLAY_CLEANUP_RUNNING\|federation_inbox_nonce_cleanup" crates/routes/src/utils/scheduled_tasks.rs` shows the cron block.
- [ ] `grep -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` shows 1 match at file end.
- [ ] `sanction_notice_round_trip` contains an `instance::table` lookup for `"instance-a.test"` + a `federation_peer` insert with `trust_level = Allowlisted` BEFORE `ActivityTrait::receive`.
- [ ] R1: no `i32 as i64` casts in `-b` new code.
- [ ] R6: every clippy uses `--no-deps -- -D warnings`.
- [ ] R7: Task 4 + Task 9 ran `cargo test --no-run`.
- [ ] R9: Task 4 enumerated `rg "fn receive" crates/apub/activities/src/governance/publish_*.rs`.
- [ ] DQ #232 honoured.
- [ ] All `-b` stories `[done]`.

### 15.7 ADR / OQ compliance

- [ ] **ADR-006**: every persisted row has `local_case_id = NULL`.
- [ ] **ADR-014**: vanilla peers never trigger the wrapper.
- [ ] **ADR-015**: `actor_url` / `target_url` TEXT verbatim; no raw-id leakage.
- [ ] **PRD §2 OUT** honoured.
- [ ] **PRD §5.4** designed-breakage repaired without `_unchecked` variant.
- [ ] **DQ #273 option-a** applied.
- [ ] **DQ #274 option-a** applied.
- [ ] **DQ #275 option-a** applied.
- [ ] **DQ #250** applied (cron wired without `#[allow(dead_code)]`).

---

## 16. Acceptance criteria

- [ ] All 11 tasks committed.
- [ ] §15.1-§15.3 exit 0 after each task.
- [ ] §15.4 Phase-2 e2e: 5 new tests pass + `sanction_notice_round_trip` repaired-green.
- [ ] §15.6 + §15.7 all boxes ticked.
- [ ] §16a stories all `[done]`.
- [ ] No edits outside §11 list.
- [ ] Retro committed.
- [ ] PR opens against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] `/brehon-verify` report shows all stories ✓.
- [ ] DQ #276 resolved.
- [ ] DQ #277 + DQ #278 recorded as planner-self-resolved.

---

## 16a. Stories

### Story 1: HTTP status mapping + schema strictness + persist-failed const land cleanly

- **Composing tasks:** Tasks 1, 2, 3 (Cohort A, all `[P]`).
- **Checkpoint (laptop):** `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` on Task 3's worker branch → exit 0.
- **Brief-Scope outputs:**
  - `crates/utils/src/error.rs` contains 6 new `LemmyErrorType` variants + 6 new `status_code()` arms returning 403/413/400/429/429/409.
  - `crates/apub/objects/src/protocol/governance/{sanction_notice,trust_attestation,moderation_label}.rs` each contain `#[serde(deny_unknown_fields)]` on the protocol struct.
  - `crates/db_schema/src/source/governance/governance_log.rs` contains `pub const ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED`.
  - `crates/api/api/src/governance/governance_log.rs` re-exports it.
  - `.claude/rules/governance-log-entry-kind-registry.md` has populated `## v1-federation-inbound-b` section + count bumped 54 → 55.

### Story 2: Wrapper + helpers + label handler compile and present

- **Composing tasks:** Task 4 (barrier).
- **Checkpoint (laptop):** `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` on Task 4's worker branch → exit 0.
- **Checkpoint (laptop, R7):** `cargo-test.bat --workspace --features full --test e2e --no-run` → exit 0.
- **Brief-Scope outputs:**
  - `crates/apub/activities/src/governance/inbox.rs` contains `pub(crate) async fn wrap_governance_inbound`.
  - `pub(crate) trait GovernanceInboundActivity` with 4 methods + default `check_per_actor_rate_limit`.
  - `pub async fn receive_remote_moderation_label`.
  - `fn log_inbox_drop`, `fn evict_oldest_unreviewed_if_needed`, `fn rate_per_peer_counts`, `fn rate_per_actor_counts`, `fn current_hour_bucket`.
  - Existing `receive_remote_sanction_notice` + `receive_remote_trust_attestation` contain a call to `evict_oldest_unreviewed_if_needed` + best-effort `_persist_failed` emit on tx-rollback.

### Story 3: Per-handler patches wire the wrapper across all 3 governance activity types

- **Composing tasks:** Tasks 5, 6, 7 (Cohort B, all `[P]`; all `requires: task 4`).
- **Checkpoint (laptop):** `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings` on Task 7's worker branch → exit 0.
- **Brief-Scope outputs:**
  - 3 governance `publish_*.rs::receive` bodies are single `wrap_governance_inbound(...).await` calls.
  - 3 `impl crate::governance::inbox::GovernanceInboundActivity for ...` blocks.

### Story 4: Replay-cleanup cron + Phase-6 fixture repair + 5 handler-e2e tests pass

- **Composing tasks:** Tasks 8, 9.
- **Checkpoint (laptop):** `cargo-test.bat --workspace --features full --test e2e --no-run` on Task 9's worker branch → exit 0.
- **Checkpoint (Phase-2):** filter `v1_federation_inbound_b_fixtures` → 5 passed; `sanction_notice_round_trip` → 1 passed.
- **Expected output (Phase-2):** `6 passed; 0 failed`.
- **Brief-Scope outputs:**
  - `crates/routes/src/utils/scheduled_tasks.rs` contains `static FED_REPLAY_CLEANUP_RUNNING` + a scheduler block calling `federation_inbox_nonce::delete_older_than`.
  - `sanction_notice_round_trip` inserts a `federation_peer` row Allowlisting `instance-a.test` before `ActivityTrait::receive`.
  - `mod v1_federation_inbound_b_fixtures` appended at e2e.rs file end with 5 `#[tokio::test(flavor = "multi_thread")]` test fns.
  - Each test asserts (a) `LemmyErrorType` variant via `matches!`, (b) HTTP `StatusCode` via `ResponseError::status_code()`, (c) DB row presence via Diesel query, (d) governance_log entry kind presence.

---

## 17. Completion checklist

- [ ] Task 0 audit complete.
- [ ] Tasks 1-9 committed.
- [ ] Task 10 retro committed.
- [ ] §15 validation green.
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit triaged.
- [ ] `/brehon-verify` report shows all stories ✓.
- [ ] Post-merge phase branch retained.
- [ ] DQ #276 / #277 / #278 resolved.
- [ ] DQ #273 / #274 / #275 / #250 honoured.
- [ ] PRECON-1 / PRECON-2 / PRECON-3 / PRECON-4 honoured.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Complexity 14 rejected (split-mandated) | HIGH | LOW | DQ #276 filed; fed-in-a precedent at 13 shipped proceed; clean split seam (Tasks 1-4 / Tasks 5-9) documented. |
| Wrapper module home contamination (PRD §5.2/§9.1 descriptor error) | LOW | HIGH | Task 0 Probe 3 enforces inbox.rs path; §10.4 cites the `:1-12` doc-comment + DQ id 37; watchpoint #1. |
| Sibling test driving `receive` directly without Allowlist | LOW | MED | Task 0 Probe 5 + Task 9 sweep; plan-time `rg` confirms only `sanction_notice_round_trip` (line :5244). |
| Phase-6 receive body shape drift | LOW | MED | Task 4 reads `inbox.rs :105-179` + `:195-250` verbatim before modifying; `run_transaction` preserved. |
| `deny_unknown_fields` breaks a peer's wire format | LOW | MED | 3 governance protocols are fork-only AP types (ADR-014); only Brehon peers send them. |
| Rate-limit cell pollution across tests | MED | LOW | Distinct peer domains per test; test (b) deliberately exercises exhaustion. |
| Replay-nonce UNIQUE-violation matching | LOW | MED | Match `DatabaseError(UniqueViolation, _)` specifically; other DB errors propagate. |
| Storage-cap eviction races with concurrent inserts | LOW | LOW | v1 advisory-only / not hot-path; single-admin per ADR-010. |
| Best-effort `_persist_failed` emit fails silently | VERY LOW | LOW | Acceptable: audit-trail loss bounded; original error surfaces. |
| Per-actor rate test fixture race | LOW | LOW | `parking_lot::Mutex` is fast; counts i32-bounded. |
| Phase-2 e2e laptop saturation | LOW | LOW | user-gate-4 dispatch option (PRECON-3). |
| Junior pre-pushes break finalize-merge | LOW | LOW | Post-Shape-G correct per `feedback_junior_finalize_skips_when_worker_pre_pushes.md`. |
| Cross-lane DQ id collision | LOW | LOW | Max id at plan-time = 275; planner uses 276 + 277 + 278. |
| BM/Junior write-permission to `.claude/rules/` | LOW | LOW | Task 3 fallback per `feedback_cc_v2_1_119_claude_gate_blocks_bm_writes.md`. |
| `cargo check` peak >6 GB | LOW | LOW | Serialised `validate-pending-laptop`. |
| Per-handler patches break Phase-6 invariants | LOW | HIGH | Task 4 preserves Phase-6 `run_transaction` shape; ADR-006 invariant unchanged. |
| `governance_config` read at wrapper hot path adds latency | LOW | LOW | Acceptable for v1; v2 may extract a per-context cache. |
| Wrapper `OnceLock<Mutex<...>>` static survives across test processes | VERY LOW | LOW | Per-process static; tests isolated via testcontainers. |

---

## 19. Notes

### 19.1 Planner DQs filed (advisor to transcribe to decision-queue.json)

- **DQ #276** (`from: "planner"`, `kind: "blocker"`, `answered_by: null`,
  filed pre-commit) — split-or-proceed per §5.2. Question:
  "Complexity score 14 exceeds Sonnet threshold 8 — split
  v1-federation-inbound-b into v1-federation-inbound-b-1 (Tasks 1-4:
  error variants + mapper + protocol attrs + persist_failed const +
  wrapper/helpers/trait/label-handler/storage-cap in inbox.rs) +
  v1-federation-inbound-b-2 (Tasks 5-9: per-handler patches +
  replay-cleanup cron + fixture update + handler-e2e) + retro, or
  proceed as one plan?" Options: `split` / `proceed`. Lean: proceed.

Suggested wire shape:

```json
{
  "id": 276,
  "from": "planner",
  "kind": "blocker",
  "timestamp": "2026-05-19T00:00:00Z",
  "question": "Complexity score 14 exceeds Sonnet threshold 8 — split v1-federation-inbound-b into -b-1 (Tasks 1-4) + -b-2 (Tasks 5-9) + retro, or proceed as one plan?",
  "options": ["split", "proceed"],
  "context": "First HTTP-path sub-phase in the federation-inbound v1 track. Brief §0 names -b as one atomic deliverable. Dominant factors: 9 impl tasks (+4), 7 crates touched (+7), e2e (+3). fed-in-a precedent at score 13 shipped proceed-as-one with no operational regret. Clean split seam: Tasks 1-4 (error/protocol/persist_failed/wrapper foundation) / Tasks 5-9 (per-handler patches + cron + fixture + e2e).",
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 19.2 Self-resolved planner findings (kind: log)

- **DQ #277** (`from: "planner"`, `kind: "log"`, `answered_by:
  "planner-self-resolved"`) — Rate-limit storage design. Resolved:
  in-module `OnceLock<Mutex<HashMap<(String, i64), u32>>>` statics
  (one per-peer, one per-actor) co-located with the wrapper in
  `crates/apub/activities/src/governance/inbox.rs`. Rationale: state
  is small; wrapper is the only consumer; no cross-crate dep. PRD
  §7.1 "extend rather than fork" honoured in spirit (no new dep;
  in-process counter mirroring Lemmy's pattern), though not by
  literal extension of `crates/utils/src/rate_limit/` (the
  token-bucket shape of `BucketConfig` doesn't fit a
  sliding-window-by-peer model). v2 may extract.

Suggested wire shape:

```json
{
  "id": 277,
  "from": "planner",
  "kind": "log",
  "timestamp": "2026-05-19T00:00:00Z",
  "question": "Rate-limit storage shape for v1-federation-inbound-b — extend crates/utils/src/rate_limit/ or co-locate as inbox.rs statics?",
  "answer": "Co-located in crates/apub/activities/src/governance/inbox.rs as 2 OnceLock<Mutex<HashMap<(String, i64), u32>>> statics. Rationale: state is small (per-peer + per-actor keyed by (key, hour_bucket); opportunistic prune); wrapper is the only consumer; no cross-crate dep. PRD §7.1 'extend rather than fork' honoured in spirit (in-process counter, no new dep), though token-bucket shape of crates/utils BucketConfig doesn't fit a sliding-window-by-peer model. v2 may extract to lemmy_utils::rate_limit::federation_inbox if state grows.",
  "answered_by": "planner-self-resolved",
  "resolved_at": "2026-05-19T00:00:00Z"
}
```

- **DQ #278** (`from: "planner"`, `kind: "log"`, `answered_by:
  "planner-self-resolved"`) — `GovernanceInboundActivity` trait
  impl placement. Resolved: impls live alongside `impl Activity`
  in each `crates/apub/activities/src/governance/publish_*.rs`
  (Tasks 5/6/7), NOT co-located in `inbox.rs` next to the trait
  definition. Rationale: per-type code stays with the per-type
  file; per-actor-rate-limit override on `PublishTrustAttestation`
  reads typed protocol fields naturally co-located; Rust orphan rule
  satisfied. The 3 patches stay disjoint (`[P]` cohort).

Suggested wire shape:

```json
{
  "id": 278,
  "from": "planner",
  "kind": "log",
  "timestamp": "2026-05-19T00:00:00Z",
  "question": "GovernanceInboundActivity trait impl placement — alongside the trait def in inbox.rs, or per-type in each publish_*.rs?",
  "answer": "Per-type, alongside impl Activity in each publish_{sanction_notice,trust_attestation,label}.rs. Rationale: per-type code stays with per-type file; PublishTrustAttestation's per-actor rate-limit override reads typed protocol fields (subject URL) naturally co-located; Rust orphan rule satisfied (trait + types both in lemmy_apub_activities); 3 disjoint files preserve [P]-cohort dispatch.",
  "answered_by": "planner-self-resolved",
  "resolved_at": "2026-05-19T00:00:00Z"
}
```

### 19.3 Pre-existing pending DQ entries

- **DQ #229** (advisor 2026-05-16, ADVISORY-LOG — Shape G re-enable
  2026-06-01). Re-check at 2026-06-01; today's plan ships under
  laptop-shape per PRECON-3.

### 19.4 Out-of-scope follow-ups

- 3 admin REST endpoints (`-c`).
- `crates/db_views/federation_inbox/` view crate (`-c`).
- OQ-FED-IN-1 pseudonym rendering (`-c`).
- `webauthn-rs` step-up on trust-change (`-c`).
- `_cross_linked` / `_dismissed` entry-kind consts (`-c`).
- Outbound per-peer trust narrowing (v2).
- SSRF-isolated fetch worker (v2).
- Federation discovery (v2).
- Per-community-per-peer trust (v2).
- Extracting rate-limit state to a dedicated module (v2 if needed).

### 19.5 Confidence bands

- **High (9/10):** PRD §9.2/§9.3/§9.4 verbatim mirror.
- **High (9/10):** PRECON-2 wrapper-home correction.
- **High (9/10):** 6 new `LemmyErrorType` variants + status-code mapping.
- **High (8/10):** Phase-6 fixture Allowlist surgery.
- **High (8/10):** Per-handler patch shape (3 disjoint files, `[P]`-eligible after Task 4).
- **Moderate (7/10):** Rate-limit static-cell design (DQ #277).
- **Moderate (7/10):** Storage-cap eviction call-site (DQ #275 option-a).
- **Moderate (7/10):** Handler-e2e activity-construction helpers (`build_minimal_*` stubs).
- **High (8/10):** §5 complexity 14 trips DQ #276 mechanically.
- **High (8/10):** Replay-cleanup cron pattern mirror.

### 19.6 Why no clarify DQ at impl time

Brief §4.2 boundary-of-judgment cases — none apply:

- §0 carve is BINDING; wrapper doesn't need `-c` endpoints.
- PRECON-2 wrapper-home pre-resolved.
- §0.2 `peer_trust_level_at_receipt` resolved (DQ #273 option-a).
- Config-source seam resolved (DQ #274 option-a).
- Storage-cap call-site resolved (DQ #275 option-a).
- Replay-cleanup cron boundary resolved (DQ #250 user-binding).
- Phase-6 fixture sweep confirms only one direct-call site
  (`sanction_notice_round_trip:5244`).

If baseline drifts between plan-write and impl-time, impl-task files
`kind: "blocker"`.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — PRD §9 verbatim mirror; PRECON-2
  correction explicit; fed-in-a substrate consumption verified at
  plan-time.
- **Cargo budget:** 8/10 — laptop-shape serial; peak ~6 GB at
  threshold per fed-in-a precedent.
- **Test coverage:** 7/10 — 5 handler-e2e tests cover 4 reject-gates
  + happy path + label handler; size-cap implicitly tested via
  `deny_unknown_fields` + Phase-6 serialise path.
- **Complexity-handling:** 9/10 — DQ #276 mechanically filed; clean
  split seam documented; fed-in-a-precedent-13 supports proceed-lean.
- **Designed-breakage handling:** 9/10 — Phase-6 fixture sweep
  + Task 9 fixture insert; no `_unchecked` variant.
