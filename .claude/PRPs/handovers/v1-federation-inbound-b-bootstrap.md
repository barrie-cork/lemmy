---
phase: v1-federation-inbound-b
plan: (not yet authored — .claude/PRPs/plans/v1-federation-inbound-b.plan.md to be created via /prp-core:prp-plan after the planning Junior)
phase_branch: phase-v1-federation-inbound-b   # not yet created — cut at bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-fed-in-b   # created at bm-cut; until then canonical C:/Users/barri/Developer/brehon-fork
authored: 2026-05-19
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-federation-inbound-b advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-federation-inbound-b.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-fed-in-b` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session — the homeserver advisor-context chain is frozen historical (the fed-in-b advisor-context the *prior* transition wrote homeserver-side is superseded by THIS file; do not read it).

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. Until bm-cut, you are on canonical `brehon-fork` / `governance-v0` (meta-edit lane: briefs + plan land on trunk).
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0 origin/governance-v0` — local must be `39c6d3e02`. **At handoff, local gov-v0 `39c6d3e02` was 1 commit AHEAD of `origin/governance-v0` `0c16b8806`** (the fed-in-a retro commit + this transition's bootstrap commit were unpushed). The transition's own commit pushes; verify `git push origin governance-v0` succeeded and `origin/governance-v0` now contains `39c6d3e02` + the `chore(brehon): close v1-federation-inbound-a, bootstrap v1-federation-inbound-b` commit. If still behind, push before any Junior dispatch (Junior workers branch from the committed+pushed trunk tip).
3. Read `.claude/decision-queue.json` (and `scripts/brehon/resolve-dq-canonical.sh v1-federation-inbound-b` once a phase branch exists) for pending entries since handoff; compare against §"Decision-queue snapshot" below. At handoff: **pending=[] (empty)**.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_federation_inbound_b.md` is the running-state scratchpad (most-recent "Session handoff block" is authoritative on resume). Read `workflow_state_v1_federation_inbound_a.md` ONCE for carry-forward.

## Next concrete action

No plan exists yet. **Author `.claude/PRPs/briefs/v1-federation-inbound-b-planning-1.md`** (scope = fed-in PRD §5.2 wrapper-routing + §9.2 wrapper signature + §9.3 per-handler patches + §9.4 `receive_remote_moderation_label` body + §7 rate-limit/replay enforcement + §5.4/§11.4 Phase-6 round-trip fixture update + handler e2e; consumes fed-in-a's helpers/tables/columns/9 ENTRY_KIND consts/11 config keys per fed-in-a plan §6 row `-b`), commit it to `governance-v0`, then **run `/brehon-clarify .claude/PRPs/briefs/v1-federation-inbound-b-planning-1.md`** → resolve every clarify-DQ → queue the `[role:planning]` Junior task. Skipping `/brehon-clarify` on a planning brief is a process breach (advisor-orchestrator §3.1).

---

## 1. v1-federation-inbound-b in one paragraph

fed-in-b is the **HTTP-path enforcement layer** of the federation-inbound v1 track. fed-in-a (PR #138, merged 2026-05-19, `7af873731`) shipped the additive schema/model/trust-state foundation with **zero HTTP-path change** — 4 tables, 2 enums, 11 config keys, 9 `ENTRY_KIND_FEDERATION_*` consts, trust-state helpers in `federation_peer.rs`, all currently unused by any handler. fed-in-b wires them in: a thin `wrap_governance_inbound` wrapper (PRD §9.2, in `crates/apub/apub/src/governance/inbox.rs`) that gates every governance `Activity::receive` through peer-trust (403 Blocklisted) → size-cap (413) → schema (400, serde-layer) → per-peer + per-actor rate-limit (429) → replay-nonce (409) → then Phase-6's `receive_remote_*`; per-handler patches turning each Phase-6 `Activity::receive` body into a single `wrap_governance_inbound` call (PRD §9.3); the new `receive_remote_moderation_label` handler body (PRD §9.4, fills the Phase-6 stub); and the Phase-6 `sanction_notice_round_trip` fixture update (PRD §5.4/§11.4 — the direct-call test now goes through the checks; set the test peer Allowlisted, NO `_unchecked` variant). DoD: wrapper + 3 per-handler patches + label handler + 6 new `LemmyErrorType` variants mapped to HTTP codes + rate-limit/replay enforcement reading the 11 fed-in-a config keys + Phase-6 round-trip green + handler-level e2e. The 3 admin REST endpoints + pseudonym rendering + step-up are explicitly `-c`.

## 2. Why v1-federation-inbound-b is harder than v1-federation-inbound-a

- **Not easier — this is the first HTTP-path change in the federation-inbound track.** fed-in-a was pure additive schema/model (confidence 8–9/10 across the board, "verbatim PRD §8.2 mirror"). fed-in-b touches live request handling in `crates/apub/` — the inbound governance dispatch path — with real failure-mode semantics (6 new error variants → 6 HTTP status codes) and real concurrency (rate-limit counters, replay-nonce window). A bug here drops or mis-routes real federation traffic, not just a migration round-trip.
- **Not easier — it breaks an existing green test by design.** PRD §5.4: Phase-6's `sanction_notice_round_trip` calls `PublishSanctionNotice::receive(...)` directly; once the wrapper lands that call goes through trust/rate/replay. fed-in-b MUST update the fixture (Allowlist the test peer before receive) — and the PRD explicitly forbids the tempting `_unchecked` variant escape hatch ("it's a footgun"). The plan must name the exact fixture-update path.
- **Easier in one respect:** the schema/config/consts substrate already exists and is proven green (fed-in-a e2e #4 = 97/0/5 post-merge). fed-in-b reads `federation_peer` trust state, the 11 `federation.inbound.*` config keys, and the 9 `ENTRY_KIND_FEDERATION_*` consts that fed-in-a landed — no migration in fed-in-b (if the plan proposes one, that is a scope violation, see tripwires). The trust-state helpers in `federation_peer.rs` (DQ #230 binding) are the wrapper's call surface, already DB-tested.
- **Easier:** the wrapper signature, per-handler patch shape, and `receive_remote_moderation_label` body are spelled out almost verbatim in PRD §9.2/§9.3/§9.4 — the plan's job is mirror-and-wire, not design.

## 3. Lessons from v1-federation-inbound-a that apply to v1-federation-inbound-b

Reference by filename — do not duplicate. From the fed-in-a retro (`.claude/PRPs/reports/v1-federation-inbound-a-retro.md`) per-role signals + carry-forward:

**Advisor-side:**
- `feedback_bm_false_success_advisor_post_condition_catch` — held **2×** in fed-in-a (#328 bm-triage wrote gitignored deliverables to a cleaned per-job worktree + self-reported `done`; #329 phase-tip-unchanged). After EVERY Junior "done", verify the real-world effect (PR state, branch tip, file existence), never the self-report.
- `feedback_background_task_notification_lies` — bg-completion notification lied **3×** in fed-in-a. For any local bg cargo/e2e, read the explicit `E2E_EXIT_0`/`CHECK_EXIT_0`/`CHAIN_DONE` marker; never trust the harness completion notification.
- New lesson candidate **`feedback_3way_eof_splice_use_index_stages_not_markers`** (NOT yet authored — fed-in-a §3 action): when a 3-way merge conflict involves a file BOTH lanes append to AND one lane mid-edited (line-shift), derive block boundaries from `git show :1:`/`:2:`/`:3:` blobs + per-branch grep FIRST; conflict-marker line arithmetic is unreliable. fed-in-a's `e2e.rs` resolution took 4 iterations from trusting marker math. If fed-in-b hits a cross-lane `e2e.rs` merge (likely — another lane WILL advance trunk), apply this and author the lesson.
- `feedback_default_local_testing` / `project_laptop_canonical_cargo_runner` — Shape G is SUSPENDED until 2026-06-01 (DQ #229). All cargo runs laptop-shape (`kind: "validate-pending-laptop"`, advisor-laptop mutates). If fed-in-b crosses 2026-06-01, re-check DQ #229 before assuming Shape G.

**Planning-side:**
- **Crate-qualify Lemmy-1.0 schema/file paths.** fed-in-a §16a Story 1 said bare "schema.rs" but the real path is the separate crate `crates/db_schema_file/src/schema.rs`. fed-in-b's plan §16a Brief-Scope outputs and §10 MIRROR refs must crate-qualify every path (the wrapper is `crates/apub/apub/src/governance/inbox.rs`; per-handler patches are `crates/apub/activities/src/governance/publish_*.rs` — see PRD §9.1 module layout). A `[descriptor-note]` in fed-in-a's verify report flagged this; do not repeat.
- §2.4 mandatory file-class lesson injection: any `crates/server/tests/e2e.rs` edit → `feedback_lemmy_error_no_std_error.md` (Case A — mirror the existing `v1_federation_inbound_a_fixtures` sibling module's `LemmyResult<()>` shape verbatim, it's in the same file) + `feedback_async_pool_test_pattern.md`; ≥2 e2e edits → + `feedback_junior_worker_e2e_edit_hang.md`. Any handler doing 2+ DB writes → `feedback_multi_write_handlers_need_transactions.md` (the wrapper + drop-log + Phase-6 insert is multi-write).

**Impl-side:**
- `feedback_fix_impl_pre_push_cargo_check` / `feedback_fix_impl_enumerate_all_callsites` — fed-in-a fix-cr-1 #329 was clean *because* the brief enumerated the exact file set and the worker ran pre-push cargo-check. The per-handler patch touches 3 `publish_*.rs` files + the wrapper file + the error-type-mapper — enumerate ALL callsites of each patched `Activity::receive` before any struct/signature change brief.
- fed-in-a impl flagged a "pre-existing lemmy_email build-script failure" that did NOT recur on the laptop — empirical verification (run it, don't trust the worker's word) was the right call. Keep that posture.

**BM-side:**
- **HIGH structural, decide before fed-in-b's first CR cycle:** gitignored bm-verb deliverables (`pr-N-findings.yaml`, `pr-N-comment.md`) die with cleaned per-job worktrees (fed-in-a #328). Pick before bm-poll-cr/bm-triage runs: (a) bm verbs commit to a non-gitignored phase-branch path, (b) bm verbs echo the full artifact into task output for advisor reconstruction (fed-in-a's workaround), or (c) daemon retains per-job worktrees for bm-task verbs. This is in the fed-in-b workflow-state watchlist.

## 4. v1-federation-inbound-b-specific watchlist

Each cites a specific file/table/line + a forward gate:

1. **`crates/apub/apub/src/governance/inbox.rs`** — Phase 6 created this file with `receive_remote_*` fns; fed-in-b ADDS `wrap_governance_inbound` + the 5 check helpers + `receive_remote_moderation_label` alongside them. Plan §13 must NOT create a new module — the wrapper lives in the existing Phase-6 file (PRD §5.2 + §9.1). Gate: plan §11 "Files to change" lists `inbox.rs` as MODIFY, not the parent `governance/` as a new dir.
2. **`crates/apub/activities/src/governance/publish_sanction_notice.rs` / `publish_trust_attestation.rs` / `publish_label.rs`** — the 3 per-handler patches (PRD §9.3). Each `async fn receive` body becomes a single `wrap_governance_inbound(self, context, |a,c| async move { ... })` call. Gate: plan §13 enumerates all 3 files explicitly; the §G4 callsite-enumeration discipline applies (`rg "Activity::receive" crates/apub/activities/src/governance/` before authoring the patch brief).
3. **Phase-6 `sanction_notice_round_trip` in `crates/server/tests/e2e.rs`** — PRD §5.4: this direct-call test breaks when the wrapper lands. Gate: the first test the plan touches must be this fixture update, asserting the path is "set test peer to `Allowlisted` in `federation_peer` before `receive`", and the plan §12 NOT-building list explicitly states "no `receive_remote_*_unchecked` variant" (PRD-named footgun).
4. **`LemmyErrorType` enum + the Lemmy error→HTTP mapper** — PRD §9.2 names 6 new variants (`FederationPeerBlocklisted`→403, `FederationPayloadTooLarge`→413, `FederationSchemaInvalid`→400, `FederationPeerRateLimitExceeded`→429, `FederationActorRateLimitExceeded`→429, `FederationActivityReplayed`→409). Gate: plan §13 has a task adding all 6 variants AND extending the existing error→response mapper; the e2e must assert at least the 403 + 429 + 409 status codes (not just "an error").
5. **The 11 `federation.inbound.*` config keys (fed-in-a-seeded) + the trust-state helpers in `federation_peer.rs`** — fed-in-b READS these; it must not re-seed or re-define them. Gate: plan §10 MIRROR ref points at fed-in-a's `governance_config` rows + `federation_peer.rs` helper signatures by name; no new migration (see tripwire). `EXPECTED_SEED_COUNT_V1_FED_IN` stays 11.
6. **`receive_remote_moderation_label` `peer_trust_level_at_receipt` plumbing** — PRD §9.4 leaves `peer_trust_level_at_receipt: /* from wrap_governance_inbound context */` as a fill-in. Gate: plan must specify how the wrapper threads the resolved trust level into the inner handler (the wrapper computes `trust` at step 1; the label handler needs it). This is the one genuine design fill-in PRD §9 leaves open — flag it in `/brehon-clarify`.

## 5. Operational rules

Carry fed-in-a's rule set forward; adjust per the retro change bullets:

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks` (status only); on transition `show_task` + `git fetch` + read DQ + triage + queue next per advisor-orchestrator §3.1 stage-shape.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-federation-inbound-b-<role>-<n>.md`, committed to `governance-v0` BEFORE the Junior task. Dispatch string <100 chars: `[role:<role>] <slug> — see .claude/PRPs/briefs/<file>.md`. Pre-queue: `memory_search_hybrid` (limit 5) + `/precheck` + §2.4 mandatory file-class lesson injection (walk the file list against the table — no judgment call).
- **LemmyResult Case A override for any e2e brief:** the `v1_federation_inbound_a_fixtures` sibling module already exists in `crates/server/tests/e2e.rs` using `LemmyResult<()>`. Any fed-in-b e2e edit mirrors that shape verbatim (Case A per `feedback_lemmy_error_no_std_error.md`) — read the sibling at its line range BEFORE authoring the brief (canonical-schema-first gate).
- **Shape G status:** SUSPENDED until 2026-06-01 (DQ #229, `project_shape_g_suspended_2026_05_16`). fed-in-b runs `kind: "validate-pending-laptop"` per advisor-orchestrator §5.2 — impl-task pushes the worker branch + raises the entry naming §15 DoD commands verbatim; advisor-laptop runs each command, mutates the entry. If fed-in-b crosses 2026-06-01, re-check DQ #229 before assuming Shape G is back.
- **Windows e2e invocation:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. NEVER bare `cargo test` (libpq.dll needs the bat wrapper's vcpkg PATH); NEVER `-p lemmy_server --features full` (no `full` feature; use `--workspace`). Read the explicit exit marker, never the bg notification (`feedback_windows_e2e_requires_bat_wrapper`, `feedback_background_task_notification_lies`).
- **Cohort/serial discipline:** laptop is the canonical cargo runner (P50 64 GB); EliteDesk = Junior orchestration only. Per-task `[P]` cohort dispatch per advisor-orchestrator §4 (YAML overlap check + `requires:` dependency check + budget check). The wrapper is a serial dependency for the 3 per-handler patches (they call it) — expect `requires:` chains, likely a non-`[P]` wrapper task before a `[P]` patch cohort.
- **Model tiering:** Planning→Opus/xhigh, Impl→Sonnet/medium, BM/ci-watcher→Haiku/low (`feedback_brehon_subagent_model_effort_assignments`). Non-Anthropic in daemon logs = drift.
- **The 6 user gates (never skip):** plan approval (after DoD smoke + watchpoint-specificity), judgment-heavy DQ, CR triage, Phase-2 e2e local-vs-dispatch (never auto-pick — `feedback_e2e_local_or_dispatch_user_choice`; recommend local first per `feedback_default_local_testing`), merge confirm, retro sign-off.
- **DQ attribution:** advisor commits writing `answered_by` must have subject `^(chore|docs)\((advisor|decision-queue)\)`. Mid-task Junior DQ writes push to the worker branch immediately.
- **Memory headroom:** no bulk `crates/server/tests/e2e.rs` reads (it is ~8900+ lines — Junior workers hang on full-file Edit per `feedback_junior_worker_e2e_edit_hang`; advisor reads only targeted line ranges).
- **Multi-lane:** if another `phase-v1-*` lane is active, fed-in-b gets its own worktree `brehon-fork-fed-in-b` at bm-cut (per `.claude/rules/multi-lane-worktree.md`); the lane-dedicated session writes phase-branch DQ for fed-in-b only. PMD is cross-lane shared (canonical `brehon-fork/.project-memory/memory.db`); the worktree's `.mcp.json` MUST pin the absolute canonical `PROJECT_MEMORY_DB` (`feedback_pmd_cross_lane_canonical_db`).

## 6. What changed from v1-federation-inbound-a's rule set

- **fed-in-a was schema-only (laptop-shape, no HTTP path); fed-in-b is HTTP-path.** The §2.4 file-class injections that fire change: fed-in-a fired migration + JSONB lessons; fed-in-b fires `feedback_multi_write_handlers_need_transactions` (the wrapper + drop-log + Phase-6 insert is a multi-write handler) and the e2e Case-A trio.
- **fed-in-a had NO existing-test-breakage; fed-in-b breaks `sanction_notice_round_trip` by design** (PRD §5.4). The plan must front-load the fixture update as a named task, not discover it as a regression.
- **fed-in-a's L14-blocking topology is RESOLVED.** At this transition (2026-05-19) the canonical `brehon-fork` was verified attached to `governance-v0` (HEAD → `refs/heads/governance-v0`, gov-v0 bound to canonical checkout, no `brehon-fork-ship-1` worktree). fed-in-b's bm-merge L14 post-merge runlog path is unblocked — UNLESS another lane re-binds gov-v0 to its worktree mid-fed-in-b (re-check at bm-merge time).
- **Crate-qualify-schema-paths** is now an explicit planning-side rule (new from fed-in-a's `[descriptor-note]`).

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 (Junior writes `crates/**` from a non-authorising brief; `answered_by:"advisor"` in a non-`chore|docs(advisor|decision-queue)` commit; subagent commits to `governance-v0`/`main` directly; bm-task opens PR into `main`; phase branch dirty when Junior reports complete; ci-watcher timeout) PLUS fed-in-b-specific:

- **BM "done" + PR still OPEN** (or findings YAML / triage comment absent) → auto-catch-fire; verify PR state via `gh pr view`, never trust the Junior self-report (fed-in-a #322/#328 precedent — `feedback_bm_false_success_advisor_post_condition_catch`).
- **Plan proposes a new migration under `crates/db_schema/migrations/**` or `migrations/**`** → catch-fire: fed-in-b is handler-only; the schema substrate is fed-in-a's. A new migration is a scope violation.
- **Plan proposes a `receive_remote_*_unchecked` variant** → catch-fire: PRD §5.4 explicitly names this a footgun; the fixture-update path is mandatory.
- **§G4 cycle-count meta-rule:** if a validate-pending fails 3× with the same `(error_class, file_basename)` for one cohort member → HARD REFUSAL catch-fire regardless of allowlist (fed-in-a's gov-v0-merge e2e.rs took 4 iterations — a near-miss; do not auto-queue a 3rd fix-impl on a repeated tuple).
- **Phase-6 `sanction_notice_round_trip` red after the wrapper lands AND the fixture-update task is marked done** → catch-fire: the fixture path is wrong; surface, do not auto-fix-forward (this is a designed-breakage, not a flake).

## 8. Archive after v1-federation-inbound-b

The standard close: run `/brehon-phase-transition v1-federation-inbound-b v1-federation-inbound-c`. The skill will: mark `workflow_state_v1_federation_inbound_b.md` CLOSED, delete the two-phases-ago record (which by then is `workflow_state_v1_federation_inbound_a.md` — the unambiguous predecessor in this lane chain, so the delete is safe at that point), create the `-c` skeleton, write `.claude/PRPs/handovers/v1-federation-inbound-c-bootstrap.md`, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move. (The fed-in-b → fed-in-c transition is the first to run fully brehon-fork-native with no homeserver touch — the prior homeserver advisor-context chain ends here.)

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD (local): `39c6d3e02` (captured 2026-05-19) — `docs(retro): v1-federation-inbound-a retro — land on governance-v0 trunk`
- governance-v0 (origin): `0c16b8806` — `docs(advisor): fed-in-a bm-merge L14 runlog re-apply + cohort-a serial-recovery retro` — **local is 1 ahead; this transition's commit + the unpushed retro commit push together. Verify origin == local + transition commit at session start.**
- Phase branch HEAD: not yet created (branch `phase-v1-federation-inbound-b` cut at bm-cut)
- fed-in-a merge: PR #138 → `governance-v0`, merge commit `7af873731`, merged 2026-05-19T00:33:36Z, head branch `phase-v1-federation-inbound-a` deleted on origin
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  39c6d3e02 docs(retro): v1-federation-inbound-a retro — land on governance-v0 trunk
  0c16b8806 docs(advisor): fed-in-a bm-merge L14 runlog re-apply + cohort-a serial-recovery retro
  b3b10aa4f chore(decision-queue): archive entries up to #224 for pre-v1-AD-e retro
  7af873731 Merge pull request #138 from barrie-cork/phase-v1-federation-inbound-a
  1b41b6182 chore(decision-queue): advisor mutated DQ #272 — pass Phase-2 e2e #4 post gov-v0 merge
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff) — .claude/decision-queue.json pending=[] (resolved=69, schema_version=2). Working-tree shows a no-op trailing-newline/CRLF diff on decision-queue.json (NOT an entry change) — pre-existing, not from this transition; left as-is per skill commit-scope rule.
Archived-but-relevant: DQ #229 (advisor 2026-05-16, ADVISORY-LOG) — Shape G re-enable 2026-06-01. NOT pending (resolved-on-trunk via fed-in-a cross-lane union) but a live forward reminder: if fed-in-b crosses 2026-06-01, re-check whether Shape G validation is back before assuming validate-pending-laptop. Now lives in a decision-queue-archive-*.json (entries up to #224 archived at b3b10aa4f; #229 itself was resolved post-archive — confirm via scripts/brehon/resolve-dq-canonical.sh at session start).
```

## Stop-and-ask tripwires

- Stop and ask if: the generated plan introduces a new migration under `crates/db_schema/migrations/**` or `migrations/**` — fed-in-b is handler-only; the schema substrate (4 tables, 2 enums, 11 config keys, 9 consts) is fed-in-a's and already on `governance-v0`. A new migration is a scope violation.
- Stop and ask if: the plan adds a `receive_remote_sanction_notice_unchecked` (or any `_unchecked`) variant to satisfy the Phase-6 round-trip test — PRD §5.4 explicitly names this a footgun; the only sanctioned path is the test-fixture update (Allowlist the test peer in `federation_peer` before `receive`).
- Stop and ask if: `scripts/brehon/resolve-dq-canonical.sh v1-federation-inbound-b` (once a phase branch exists) shows pending DQ entries that are NOT in the §"Decision-queue snapshot" above — a mid-flight blocker landed; triage before continuing the stage-shape.
- Stop and ask if: at bm-merge time, `git -C C:/Users/barri/Developer/brehon-fork worktree list` shows `governance-v0` bound to a worktree OTHER than the canonical `brehon-fork` checkout, OR canonical HEAD is detached — the L14 post-merge runlog path needs canonical = attached gov-v0 (fed-in-a's L14 was skipped for exactly this topology; it was resolved at this transition but another lane could re-break it).
- Stop and ask if: a validate-pending entry fails a 3rd time with the same `(error_class, file_basename)` tuple for one cohort member — §G4 hard-refusal (the recipe family is wrong-shaped; do not auto-queue a 3rd fix-impl; this is a re-plan signal, per fed-in-a's near-miss 4-iteration e2e.rs splice).
