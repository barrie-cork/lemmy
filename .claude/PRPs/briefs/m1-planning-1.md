# M1 planning brief

**Written**: 2026-06-03 by advisor session (canonical checkout `C:\Users\barri\Developer\brehon-fork` on `governance-v0`). M1 has no phase branch yet (bm-cut runs after plan approval), so this planning brief commits on `governance-v0` and the resulting plan file commits on `governance-v0` too (planner finalize-merge), per `.claude/refs/auto-phase.md` §"Brief location per role" (planning briefs → governance-v0).

**Subagent target**: `planning` (Opus 4.8, color purple — see `.claude/agents/planning.md`).

**Worktree**: Junior cuts `junior/m1-planning-1` from `governance-v0` per concurrency-1 default. The plan file commits + pushes to `governance-v0` at finalize.

**Authority anchor**: `.claude/PRPs/prds/m1-chat-infrastructure.prd.md` (the M1 sub-PRD, authored this session, commit `5c96f30c5`) — read in full. It is the narrowed, M1-only spec derived from the umbrella `v2-messaging-rtc.prd.md`. Where the umbrella and the M1 sub-PRD disagree (e.g. the umbrella's "hash-chain touched: YES" vs M1's "NO"), **the M1 sub-PRD wins** — it is the schedule-time-decided scope.

**Lane mode**: N/A at planning time (no phase branch). After plan approval, bm-cut creates `phase-<M1-slug>` and the impl phase runs Mode A or B per a later decision.

---

## 0. Dependency-reality check (advisor-verified 2026-06-03, against `5c96f30c5`)

Per `feedback_handover_assumptions_need_empirical_verification.md` + `feedback_runbook_audit_drift_post_event_check.md` — every claim grep-verified against the live `governance-v0` tree this session:

| Dependency | State | Evidence |
|---|---|---|
| PM plugin hook dispatcher | ✅ EXISTS | `crates/api/api_utils/src/plugins.rs:33` (`plugin_hook_after`), `:48` (`plugin_hook_notification`), `:103` (`run_plugin_hook_after`) |
| PM hooks fire at PM create site | ✅ EXISTS | `crates/api/api_crud/src/private_message/create.rs:72` (`local_private_message_before_create`), `:75` (`local_private_message_after_create`) |
| Legacy `matrix_user_id` AP field (to be left UNTOUCHED in M1) | ✅ EXISTS | `crates/apub/objects/src/protocol/person.rs:49` (`pub(crate) matrix_user_id: Option<String>`) |
| Greenfield — zero matrix-sdk / livekit deps in workspace | ✅ CONFIRMED | grep across all `Cargo.toml` = empty |
| `governance_messaging_config` table | ❌ ABSENT | grep across `crates/db_schema*/src/` = empty — **M1 BUILDS this** |
| `services/bridge/` directory | ❌ ABSENT | does not exist — **M1 BUILDS this** (workspace-excluded) |
| Tuwunel homeserver decision | ✅ RESOLVED | OQ-V2-10 → Tuwunel (Synapse fallback), per `99-decisions-and-open-questions.md` 2026-06-01 |

> **Note on line-number drift:** the umbrella PRD cited `create.rs:75-80` for the PM hooks; the live tree shows `:72,75` (upstream rebase shifted them). This is expected drift — the **planner MUST re-verify every file:line in this brief against the live tree at plan time** before writing it into the plan §10 MIRROR refs or §13 IMPLEMENT lines. Cite live line numbers, not this brief's.

**Load-bearing consequence:** M1 needs exactly **one new migration** (`governance_messaging_config` table) and **zero** governance-log / hash-chain changes (those are M2). The bridge daemon is entirely new code in a new, workspace-excluded directory.

---

## 1. Role + dispatch line

`[role:planning] M1 plan — chat infrastructure (bridge daemon + Tuwunel + 1:1 DM + admin config)`

The actual create-task description is a single line under 100 chars per `.claude/rules/advisor-orchestrator.md` Junior task description template:

```
[role:planning] M1 plan — see .claude/PRPs/briefs/m1-planning-1.md
```

---

## 2. Scope — what to produce

Produce **one plan file** at `.claude/PRPs/plans/m1.plan.md` for sub-phase **M1** (chat infrastructure). The plan covers the M1 sub-PRD §2 IN-scope items in full, following `.claude/PRPs/templates/plan.template.md` verbatim.

### 2.1 The deliverable's two distinct file-ownership trees

M1 has an unusual shape for a Brehon phase: it spans **two distinct trees** that the plan §13 task list MUST keep as separate task groups (different ownership → cleaner cohorts, and different validation paths — see §2.3):

**Tree A — `services/bridge/` (greenfield Rust daemon, workspace-EXCLUDED):**
- Crate skeleton with its own `Cargo.toml`, excluded from the Brehon workspace `members` (so `cargo build --workspace` never pulls Matrix deps). Read sibling external-service patterns first.
- `matrix-sdk-appservice` integration. **Plan-time task:** pin a known-good version; verify the Tuwunel issue-#219 (`whoami` response code) interaction against the chosen `matrix-sdk-appservice` version (per M1 sub-PRD §13 + matrix research §Recommendation item 2).
- Puppet-on-first-contact (Brehon user → Matrix puppet account).
- 1:1 DM relay (Brehon↔Matrix), text + image + voice note (rich media via Matrix media repo).
- Manual community-room provisioning command surface.
- Bridge-local `messaging_user_id` puppet map (NOT the ADR-016 B-actor portable ID — that's M2+).
- Soft-pause flag poll/notify + graceful drain-to-idle (per M1 sub-PRD §6 — reversible relay-pause).

**Tree B — Brehon-side (in-workspace, additive only):**
- `governance_messaging_config` table — **one new migration** + Diesel model. Holds `messaging_enabled` (default false), identity-policy (per-room-type default), lifecycle policy (`hard_delete_after_days`).
- Admin write path for the config (mirror an existing admin-config endpoint — planner finds the closest sibling, likely the `governance_config` admin write surface).
- **Identity-policy validator** that rejects jury/appeals identity-policy overrides (ADR-015 pin — enforced structurally from M1 even though jury/appeals rooms are M2; the validator exists now, satisfied vacuously until M2 adds the rooms).
- Wire the existing 7 PM plugin hooks to notify the bridge (notification/read-only — NO new hooks, NO change to the PM path).
- `messaging_enabled = false` clean-posture guard (default false; when false, zero messaging surface).

**Tree C — deployment / ops + design docs (the planner decides whether this is its own task group or folded into A/B):**
- docker-compose: Tuwunel + bridge side-by-side; the four "verify before committing" Tuwunel items (M1 sub-PRD §13); `ip_source` NOT set on loopback; containerised non-host-network handling; federation-disabled launch posture.
- Restart fixture for the admin-panel-persistence acceptance test.
- AGPL notice extension (bridge + Tuwunel).
- Design-doc updates (chat-plane only, NOT app-plane): `06 §2.2` plane-boundary, `06 §7` threat rows (bridge compromise, hostile homeserver payload, E2EE key mishandling, media artefact leak), `07 §5` soft-pause posture row.

### 2.2 Acceptance criteria the plan §16a stories must cover

From M1 sub-PRD §8 (M1-only success criteria) — each becomes a §16a story with a checkpoint:

1. 1:1 DM text+image+voice round-trip < 3s (integration test `services/bridge/tests/dm_round_trip.rs`).
2. Admin panel identity-policy change persists across restart (restart fixture).
3. `messaging_enabled = false` preserves clean v0 governance-only posture (`cargo test --test e2e` governance flow unchanged).
4. Soft pause reversible (enable→disable→enable) without process restart or data loss.
5. Identity-policy validator rejects jury/appeals override (unit test — ADR-015 pin).
6. `services/bridge/` workspace-excluded; `cargo build --workspace` pulls zero Matrix deps (CI / `cargo tree` assertion).

### 2.3 The validation-strategy question (LOAD-BEARING — see §4 + clarify)

The standard Brehon DoD (`cargo check --workspace --features full` + e2e) **does not cover Tree A** because the bridge is workspace-excluded. The plan §15 DoD and §13 per-task validation gates MUST address:
- How does `services/bridge/` get compiled/checked? (Its own `cargo check` inside `services/bridge/`, separate from the workspace check.)
- How do the bridge integration tests (`dm_round_trip.rs`) run, given they need a live Tuwunel + the bridge running? (Likely docker-compose-gated integration tests, NOT part of the standard `cargo test --test e2e` workspace suite.)
- What is the laptop-runnable subset vs the docker-compose-gated subset?

This is explicitly flagged for the planner to design and is a clarify question below. Do NOT assume the standard validate-pending-laptop cargo flow covers the bridge.

### 2.4 Scope boundary — what is NOT in M1

Per M1 sub-PRD §1.3 + §2 OUT-of-scope:
- **Governance-triggered rooms** (jury/appeal/emergency) → M2. M1 rooms are manual only.
- **`Room::*` governance-log entries** → M2. M1 writes NOTHING to the hash chain.
- **Town halls / MatrixRTC / LiveKit / Element Call** → M3.
- **ADR-016 backplane** (B-fetch adapter, B-actor link flow, B-publish) → M2+. M1 users are bridge-local puppets, NOT B-actor-linked.
- **Vanilla-Lemmy interop** → resolved OQ-V2-08 (a): Brehon↔Brehon only. No degraded-mode `Announce` mirror.
- **New AP-wire actor-extension** → M2+. M1 leaves `matrix_user_id` (`person.rs:49`) exactly as upstream; the puppet map is bridge-local.
- **Hard-decommission / media cleanup on disable** → soft-pause does NOT trigger cleanup (M1 sub-PRD §6 sub-question D).

**Hard out-of-scope (do NOT let the plan drift into these):**
- No `Room::*` consts, no governance-log emit, no hash-chain touch.
- No second migration beyond `governance_messaging_config`.
- No change to `crates/apub/objects/src/protocol/person.rs` or any AP serialization.
- No new PM plugin hooks (use the existing 7).

### 2.5 Plan file deliverable shape

Plan file at `.claude/PRPs/plans/m1.plan.md` follows `.claude/PRPs/templates/plan.template.md` verbatim. Required sections: §1 goal, §4 watchpoints (each citing a specific file/table per `feedback_advisor_watchpoint_specificity.md`), §5 complexity score (expect high — greenfield daemon + new external service + docker; a split-or-proceed DQ is likely and acceptable), §10 MIRROR refs (live-verified file:line), §13 task list with `[P]` markers + FILES YAML (`creates:`/`modifies:`/`requires:`), §15 DoD (must address the Tree-A-vs-Tree-B validation split per §2.3), §16a stories (the 6 acceptance criteria above).

---

## 3. Required reading (in order, before drafting any plan section)

### 3.1 PRD + scope anchors (load first)

1. `.claude/PRPs/prds/m1-chat-infrastructure.prd.md` — **the M1 sub-PRD, read in full.** Most load-bearing: §2 (IN/OUT scope + the bridge-location + Person-field decisions), §6 (OQ-V2-09 soft-pause spec — the four sub-answers), §7 (phase details / work breakdown), §8 (success criteria), §9 (cross-cutting impact — note "hash-chain: NO", a deliberate narrowing from the umbrella).
2. `.claude/PRPs/prds/v2-messaging-rtc.prd.md` — the umbrella, for context ONLY. **Where it disagrees with the M1 sub-PRD, the sub-PRD wins.** Read §Research-Summary (Q1 hooks, Q5 greenfield) + §Technical-Approach.

### 3.2 ADR + open-question anchors

3. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **ADR-016** (cross-app backplane) — read to understand WHY the backplane is deferred from M1. M1 builds none of B-fetch/B-publish/B-actor.
4. `99-decisions-and-open-questions.md` **ADR-015** (pseudonymised actor IDs) — the jury/appeals identity-policy pin the M1 admin validator must enforce structurally.
5. `99-decisions-and-open-questions.md` **ADR-014** (vanilla-Lemmy interop) — OQ-V2-08 resolution (a) Brehon↔Brehon only.
6. `99-decisions-and-open-questions.md` **ADR-012** (Extism plugin host) — the PM plugin hooks are M1's only seam into the Brehon binary.
7. `99-decisions-and-open-questions.md` **OQ-V2-09** + the 2026-06-03 entry — soft-pause resolution.

### 3.3 Technical-grounding reads

8. `docs/research/matrix-homeserver-selection-2026.md` — **§Recommendation** (the four "verify before committing" Tuwunel items + the architecture diagram + RAM profile). Load-bearing for the deployment task + the `matrix-sdk-appservice` version-pin task.
9. `docs/brehon-law-inspired-network/V2/messaging.md` — predecessor research §5.1 (bridge daemon Option A vs in-process Option B — Option A chosen) + §8 (the v0 hooks V2 depends on). Read §5.1 + §8 at minimum.
10. `.claude/rules/pm-plugin-hooks-stable.md` — the stability rule for the 7 hooks M1 wires to.

### 3.4 Canonical-sibling code reads (live tree — re-verify line numbers)

11. `crates/api/api_utils/src/plugins.rs` — the PM hook dispatcher (`plugin_hook_before/after/notification`). The bridge-wiring task hooks into these.
12. `crates/api/api_crud/src/private_message/create.rs` (~`:72,:75`) — a PM hook call site, the pattern for where the bridge gets notified.
13. **Closest admin-config-write sibling** — planner greps for the `governance_config` admin write endpoint (the M1 admin config panel mirrors it). Likely under `crates/api/api/src/governance/`. Read it as the structural sibling for the `governance_messaging_config` write path.
14. **Closest migration sibling** — the most recent `crates/db_schema/migrations/**` migration that adds a config-style table. Mirror its structure for `governance_messaging_config`.
15. `crates/apub/objects/src/protocol/person.rs:49` — confirm `matrix_user_id` is left UNTOUCHED (read-only; the plan must NOT modify this file).

### 3.5 Mandatory file-class lessons (per advisor-orchestrator.md §2.4 file-class table)

M1 IMPLEMENT file list (Tree B + the e2e additions — Tree A is a separate non-workspace tree; see note):
- `crates/db_schema/migrations/**` (CREATE — `governance_messaging_config`)
- a new Diesel model file under `crates/db_schema/src/source/` (CREATE)
- a new admin-config handler under `crates/api/api/src/` (CREATE)
- `crates/api/routes/src/lib.rs` (modify — route registration)
- `crates/api/api_common/src/` DTO file (modify — config DTOs)
- a PM-hook-wiring edit under `crates/api/api_crud/src/private_message/` or `crates/api/api_utils/src/` (modify — notify bridge)
- `crates/server/tests/e2e.rs` (modify — clean-posture test #3, validator test #5)

Walk against the §2.4 file-class table → these rows fire:

| Pattern matched | Mandatory lesson(s) for §3 Required reading |
|---|---|
| `crates/db_schema/migrations/**` (new migration) | `feedback_lemmy_migration_runner.md` (forbid_diesel_cli; `cargo run -p lemmy_diesel_utils --features full`); `feedback_postgres_jsonb_canonicalization.md` IF the config table stores JSONB |
| `crates/server/tests/e2e.rs` (any edit) | `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`. Mirror the sibling fixtures module error-shape (Case A or B) verbatim — read the sibling at its cited line range BEFORE authoring the test task. |
| `crates/server/tests/e2e.rs` (≥2 edits in this task or cohort) | `feedback_fix_impl_pre_locate_e2e_anchors.md` (pre-locate verbatim anchors) |
| Any handler doing 2+ DB writes | `feedback_multi_write_handlers_need_transactions.md` (the admin-config write may do config-row + audit; planner checks) |
| Any `#[cfg(feature = "full")]` gate | `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md` |
| Any newtype under `crates/db_schema/src/newtypes/` | `feedback_newtype_locations_lemmy_db_schema_vs_file.md` (if a `MessagingConfigId`-style newtype is introduced) |

**Tree A note:** `services/bridge/**` is NOT in `crates/` and NOT in the workspace — the file-class table above (which is keyed to `crates/` paths + the Lemmy cargo harness) does **not** apply to bridge code. Bridge code follows its own crate-local conventions (read sibling external-service patterns per the source-code-sibling rule, `feedback_read_canonical_before_writing_spec.md` Tier-2 extension). The planner MUST NOT apply the Lemmy-workspace lessons (e.g. `feedback_clippy_test_style.md` LemmyResult discipline) to bridge code — the bridge is its own crate with its own error type.

### 3.6 PMD-promoted patterns (load on demand)

- `pattern_cargo_feature_flag_propagation.md` — `--features full` / `--workspace` discipline (applies to Tree B; NOT Tree A).
- `pattern_spec_schema_co_commit.md` — migration + Diesel model + DTO + handler + route land in coherent task grouping.
- `pattern_context_is_finite.md` — plan body likely 600-900 lines (two trees); cite PRD by anchor, don't inline.

---

## 4. Constraints (hard rules — violating any is a process breach)

### 4.1 Plan-content discipline

a. **M1 sub-PRD is the contract; the umbrella is context.** Where they disagree, the sub-PRD wins (it's the schedule-time-decided scope). Chiefly: M1 touches the hash chain = **NO** (umbrella said YES for all three clusters); M1 backplane = **none** (deferred to M2+).

b. **Exactly one migration.** `governance_messaging_config` only. If the planner believes a second migration is needed, file a `kind: "blocker"` planner DQ — do not silently add it.

c. **No hash-chain / governance-log touch.** No `Room::*` consts, no `governance_log::append` calls. That's M2. A plan task that emits to the chain is a scope breach.

d. **No AP-wire change.** `crates/apub/objects/src/protocol/person.rs` and all AP serialization stay byte-unchanged. The puppet map is bridge-local. (Adding an actor-extension on the wire is a B-actor/ADR-016 concern → M2+.)

e. **No new PM plugin hooks.** Wire the existing 7 (verified §0). The bridge is notified via the existing seams; the PM path itself is unchanged.

f. **`services/bridge/` MUST be workspace-excluded.** The Brehon binary's `Cargo.toml` gains zero Matrix deps. The plan must specify the exclusion mechanism (the bridge is outside any `members` glob / in `exclude`) and a §16a checkpoint asserting `cargo build --workspace` pulls zero Matrix deps.

g. **ADR-015 identity-policy pin enforced from M1.** The admin-config validator rejects jury/appeals identity-policy overrides NOW, even though those rooms arrive in M2. The pin is structural, not deferred.

### 4.2 Validation-strategy design (the LOAD-BEARING planner task — see §2.3)

The plan §15 DoD and §13 per-task validation gates MUST explicitly design the **two-tree validation split**:
- Tree B (in-workspace) rides the standard path: `cargo check --workspace --features full`, e2e via the standard suite, validate-pending-laptop DQ.
- Tree A (`services/bridge/`, excluded) needs its OWN validation: a `cargo check`/`cargo test` scoped to `services/bridge/` (run from inside that dir), and the integration tests (`dm_round_trip.rs`) are docker-compose-gated (need live Tuwunel + bridge) — NOT part of the workspace e2e suite.
- The plan must state which checks are laptop-runnable and which are docker-compose-gated, so the advisor's validation handler knows what to run. **Do not assume the standard validate-pending-laptop cargo flow covers Tree A.**

This is the single most important thing the planner must get right that no prior Brehon phase has faced. If the planner is unsure how the advisor's validation machinery should handle a workspace-excluded daemon, file a `kind: "blocker"` planner DQ rather than guessing.

### 4.3 Decision-queue discipline

Planner writes `kind: "blocker"` (or `kind: "log"`) from `from: "planner"`; may pre-seed `resolved` entries with `answered_by: "planner"` (naming the advisor as source where applicable); NEVER writes `answered_by: "advisor"` or `"user"`; generates v3 ids via `bash scripts/brehon/dq-v3-new-entry.sh`; appends via `bash scripts/brehon/dq-v3-append-fragment.sh`; mid-task commit + push to the worker branch per `.claude/rules/decision-queue.md` "Mid-task visibility".

### 4.4 File ownership

Planner writes ONLY:
- `.claude/PRPs/plans/m1.plan.md`
- `.claude/decision-queue.json` (DQ entries via v3 helpers)

Planner does NOT write any `crates/**`, `services/**`, `docs/**`, `.claude/lessons/**`, or `.claude/rules/**` file.

### 4.5 Attribution integrity

Every commit subject on the planning worker branch matches the planner-attribution pattern: `chore(plan): m1 — <slug>` or `feat(plan): m1 — <slug>`.

### 4.6 Forbidden execution windows

Non-binding for the planning dispatch (planner is a Junior task; no cargo).

---

## 5. Pre-commit dogfood (per advisor-orchestrator.md §3.7)

This brief was walked through against (advisor session, 2026-06-03):
- `.claude/PRPs/prds/m1-chat-infrastructure.prd.md` (authored this session — §2 scope, §6 soft-pause, §7 phase details, §8 success criteria all parsed).
- `.claude/PRPs/prds/v2-messaging-rtc.prd.md` (umbrella — §Research-Summary Q1/Q5 parsed for the hook + greenfield claims).
- `docs/research/matrix-homeserver-selection-2026.md` §Recommendation (the four verify-before-committing items + architecture diagram parsed).
- `99-decisions-and-open-questions.md` ADR-016 (backplane deferral rationale), ADR-015 (identity pin), 2026-06-03 entry (OQ-V2-08 + M1 scope), 2026-06-01 entry (OQ-V2-09, OQ-V2-10).
- `crates/api/api_utils/src/plugins.rs` (read — dispatcher at `:33,:48,:103`).
- `crates/api/api_crud/src/private_message/create.rs` (read — hook call sites `:72,:75`).
- `crates/apub/objects/src/protocol/person.rs:49` (read — `matrix_user_id` to leave untouched).
- `.claude/PRPs/briefs/rt-r5-planning-1.md` (canonical sibling brief — §0-§6 structure mirrored).
- `.claude/commands/brehon-clarify.md` (read — authored this brief in clarify-ready shape).

What worked:
- §0 dependency-reality check confirmed greenfield + the PM hooks + the absent config table. Line numbers drifted from the umbrella's citations (rebase) — flagged for planner re-verification.
- The two-tree shape (workspace-excluded bridge + in-workspace Brehon-side) is the defining feature; §2.1 + §2.3 + §4.2 make it explicit so the planner doesn't conflate the validation paths.

What the planner must resolve (→ clarify questions below):
- The Tree-A validation strategy (workspace-excluded daemon + docker-compose-gated integration tests). The standard validate-pending-laptop flow does NOT cover it.
- The `matrix-sdk-appservice` version pin + the Tuwunel issue-#219/#465 interaction.
- Whether the M1 phase splits into 2 phase branches (Tree A + Tree B) or runs as one — the complexity score (§5) drives a likely split-or-proceed DQ.

---

## 6. Acceptance for this brief

Brief is queueable when:
- DQ pending count = 0 OR all pending entries are non-blocking for M1 planning.
- **Clarify gate COMPLETE** per advisor-orchestrator.md §3.3 — `/brehon-clarify .claude/PRPs/briefs/m1-planning-1.md` run, every clarify-DQ resolved (advisor self-answer with citation, or user-relay).
- Forbidden-window check at dispatch time (non-binding for planning dispatch).
- MCP `junior-brehon` connected (dispatch needs `mcp__junior-brehon__create_task`).

Brief is committed to `governance-v0` (planning briefs commit on trunk; no phase branch yet) with subject:
```
chore(advisor): author M1 planning brief
```
