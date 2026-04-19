# Post-Phase-6 polish — intake runlog

**Started:** 2026-04-19T17:20Z
**Base:** `governance-v0` @ `08065e1a1` (PR #46 merge)
**Source memory:** `project_brehon_post_phase6_cleanup.md` (advisor-side homeserver)
**Target:** ship v0.0.0 tag after all polish PRs land on `governance-v0`

---

## Shipping strategy (from memory)

| PR | Items | Branch | Priority |
|---|---|---|---|
| **polish-1 critical bugs** | 1–3 (+ maybe 4) | `polish/critical-bugs` | 🔴 blocks tag |
| **polish-2 Bucket C** | 5–7 | `polish/bucket-c` | 🟠 ship if cheap |
| **polish-3 docs** | 12 | `polish/docs-sweep` | 🟠 low risk |
| **polish-N topic** | 13–21 | `polish/<topic>` per item | 🟡 evaluate |
| **polish-tag** | 23 | `polish/v0-tag` | 🟢 last |

Rules: base `governance-v0`, merge (not squash), CR critical-only discipline.

---

## Phase 0 — triage (in progress)

Running parallel research agents to answer:

1. **Which issues are still open?** Cross-reference memory's issue list (#33, #34, #35, #36, #37, #38, #39, #47, #48, #49, #50, #51, #52, #53, #54) against `gh issue list` current state. Some may already be closed (memory notes #49 should be closeable).
2. **What's the current code state for polish-1 items (#48, #35, #34, #33)?** Read the files, assess fix difficulty, confirm memory's scope against what's actually there.
3. **What trivial docs items (#38, #39, #50, #51, #52) can be bundled?** Confirm each exists + check cost.

Agents will return triage notes; advisor will then pick a starting PR and sequence the work.

---

## Agent assignments

- **Agent-T1 (issues)** — `gh issue list --state open --repo barrie-cork/lemmy` full dump + verify claims in memory
- **Agent-T2 (polish-1 code)** — read `governance_log::append`, `submit_jury_vote` post-decision block, `federation_outbox::send_local_sanction_notice`, `request_appeal` guard, `admin_assign_jury` pool filter
- **Agent-T3 (docs bundle)** — read each of #38/#39/#50/#51/#52, confirm scope, estimate combined diff size

## Next

Once triage agents return, the advisor writes a concrete PR sequence
(starting with polish-1) and spawns impl agents per PR.

---

## Research findings

### T1 — issue triage

**Verified against `gh issue list --repo barrie-cork/lemmy --state all` + auto-memory, 2026-04-19.**

#### Plan items verified correct (OPEN, plan-to-issue mapping valid)

- Item 1 → **#48** OPEN, labels `bug` + `risk:high`. Title matches plan's "atomicity + causal-ordering + hidden-write sweep".
- Item 2 → **#35** OPEN, labels `bug` + `risk:high`. Title matches "NOTIFY trigger fires on INSERT before signature UPDATE".
- Item 3 → **#34** OPEN, labels `bug` + `risk:high`. Title matches "request_appeal unreachable for Decided cases".
- Item 4 → **#33** OPEN, labels `bug` + `risk:critical`. Title matches "declining juror can be picked as own replacement".
- Items 5, 6, 7 → **#54** OPEN (single umbrella issue for PR #46 Bucket C follow-ups). Plan references sub-items `#2p-7`, `#2p-2`, `#2p-3`, `#2p-6` — these are thread anchors within #54's body, not separate issues. Valid.
- Item 8 → **#36** OPEN, labels `bug` + `risk:medium`. Matches "404 allowed in e2e route-registration gate".
- Item 9 → **#47** OPEN, labels `bug` + `risk:low`. Matches "task-hopper.schema.json lifecycle invariants".
- Item 10 → **#37** OPEN, labels `ops` + `risk:low`. Matches "cargo-test-e2e workflow silently overrides rust-toolchain.toml". Note: this is a CodeRabbit follow-up on PR #32 (merged `3bbf419da`); PR #32 added the workflow but did NOT fix the toolchain pin — so #37 is correctly still open.
- Item 12 → **#38, #39, #50, #51, #52** all OPEN, all labelled `documentation` + `risk:low`. Mapping valid.
- Item 22 → **#53** OPEN, labels `enhancement` + `v1` + `risk:low`. Tagged v1-deferred but still bundled in polish plan as 🔵 trivial — intentional per plan line 93.

#### Drift / stale references

- **Item 11 (plan line 60):** cites **#49** and claims "Already fixed in PR #46 at `9aa1a778a`. Verify and close without reopening the file." Status: **#49 is already CLOSED** (`closedAt 2026-04-19T17:47:00Z`, stateReason `COMPLETED`) — closed 32 min after PR #46 merged. **Action: strike item 11 from the plan — no action needed.** The "verify and close" step is done.

#### Memory reconciliation

- **`project_pr46_phase_6_review_response.md` line 73** — claim "Awaiting CodeRabbit re-review on `8ee45a3ca`. On green … merge PR #46 into governance-v0" — STATUS: **superseded**. `gh pr view 46` confirms **MERGED** at `08065e1a1` (`mergedAt 2026-04-19T17:15:37Z`, base `governance-v0`). Merge commit hash matches plan line 3.
- **`project_phase_6_handoff_ready.md`** — describes overnight Phase 6 run state at `15f8cbbd0`. Phase 6 has since shipped and merged. Historical; no action required.
- **`feedback_coderabbit_block_merge_critical.md`** — process rule, still accurate. Pattern held on PR #46 execution (both Critical findings #15 and #19 fixed in-PR at `41f1d0379` + `d70610980`). No reconciliation needed.
- **MEMORY.md index lines 40 + 49** — still reference "pending CI gates before merge" (PR #10) and "awaiting CodeRabbit re-review" (PR #46). Both PRs now merged. Stale but not load-bearing for polish intake.

#### Additional OPEN issues NOT in plan (advisory)

- **#45** OPEN (no labels) — `sponsor_liability_with_founder_multiplier accept_jury_assignment NotFound flake`. Test flake. Decide if it blocks v0 tag (test determinism) or defers to v1.
- **#43** OPEN (no labels) — `phase1_migrations_round_trip needs revert-list extension for federation tables (task 70 carry-forward)`. Test infra carry-forward from Phase 6. Likely v1.
- **#42** OPEN (no labels) — `ineligible_user_cannot_be_picked_for_jury cross-test contamination under --test-threads=1`. Test isolation. Likely v1 but could fold into item 17 clippy/test-harness sweep.
- **#6** OPEN (labels `governance/plan-drift` + `risk:high`) — "Plan drift — 2026-04-17 (11 row(s))". **Stale.** Pre-dates Phase 5c completion (shipped all 11 MVP endpoints per `project_phase_5c_complete.md`). **Action: re-run plan-drift check on current `governance-v0` HEAD `08065e1a1`; if all 11 endpoints wired, close #6.** Docs-hygiene close — fold into polish-3 or pre-tag verify step.
- **#40, #41, #30, #11–#29, #31, #53** — all `v1` labelled. Correctly excluded per plan line 146.

#### Recommendations

1. **Strike plan item 11 (#49)** — already closed; "verify and close" step is done. Reduces plan from 23 to 22 items.
2. **Add "verify-and-close #6"** to polish-3 docs sweep or as a standalone pre-tag check — plan-drift report 2 days stale; likely no-ops post Phase 5c+6.
3. **Triage #45, #43, #42 at polish-1 kickoff.** `#42` and `#43` feel v1; `#45` flake may warrant a cheap fix if reproducible.
4. **Memory-index refresh (low priority, post-polish-week)** — update `MEMORY.md` lines 40 + 49; reflect PR #10 + PR #46 merged state in their summary fields.
5. **No other drift** — plan item → issue mapping is otherwise clean. GH #54 umbrella-subitem convention (`#2p-N`) is valid.

### T2 — polish-1 scope

**Verified against worktree HEAD `65bfdb683` on `polish/critical-bugs`, 2026-04-19.** All four issue fixes have already landed as commits on this branch (`1bb622e8a`, `93c8922eb`, `034409128`, `5cb868827`); the analysis below maps the fix surfaces that **were** needed and confirms landed state, so the PR-grouping recommendation reflects what actually shipped.

#### #48 — governance_log atomicity / causal / hidden-write
- **Code surface:**
  - (a) atomicity — `crates/db_schema/src/source/governance/governance_log.rs:165-226` (INSERT + UPDATE in `append`)
  - (b) causal order — `crates/api/api/src/governance/submit_jury_vote.rs:452-494` (`case_decided` append vs. `send_local_sanction_notice` call)
  - (c) hidden pseudonym write — `crates/api/api/src/governance/federation_outbox.rs:159-160` (`actor_pseudonym_helper::get_or_create` with no matching log entry)
- **Fix shape:** (a) wrap INSERT+UPDATE in `conn.run_transaction(|conn| { ... }.scope_boxed())` with the reborrow pattern so nested callers collapse to a SAVEPOINT; (b) append `case_decided` **before** the `if FederatedRecommendation { send_local_sanction_notice }` block so the hash chain records the local determination ahead of any derived federation signal; (c) **deferred** — current `get_or_create` is idempotent under the unique constraint and introducing a read-only variant or bespoke ADR-015 entry_kind is larger than the polish-1 scope can carry without an ADR touch.
- **Blast:** (a) `governance_log.rs` +~40 LoC (transaction wrap + doc update); (b) `submit_jury_vote.rs` ~30 LoC move with causality comment; (c) one-line comment at call site. No migration. Tests that exercise `governance_log::append` transitively cover (a); `sanction_notice_round_trip` is the causal-order regression for (b).

#### #35 — NOTIFY trigger fires on INSERT before signature UPDATE
- **Code surface:** `migrations/2026-04-20-000100-0000_fix_governance_log_notify_trigger_after_sign/up.sql` — already replaces the original `AFTER INSERT` trigger with `AFTER UPDATE OF signature` gated by `OLD.signature IS NULL AND NEW.signature IS NOT NULL`. The trigger fix migration **pre-existed on `governance-v0`** (landed via PR #46 re-review Bucket C); the polish-1 branch adds no new migration.
- **Fix shape:** trigger already fires only at the NULL→NOT NULL signature transition (option A from the issue body). Combined with #48(a) atomicity wrap, the NOTIFY-before-signed-row window closes completely: subscribers can only observe the post-UPDATE state, and the INSERT+UPDATE now commit together.
- **Overlap with #48:** **high** — resolution requires both the trigger change (already shipped) **and** the atomicity wrap (#48(a)). Without the atomicity wrap, a crash between INSERT and UPDATE still leaves an unsigned row that the trigger never fires for (subscribers silently miss the event). The #48(a) tx wrap closes that gap; the commit message `1bb622e8a` cites both `#48 #35` for this reason. **One shared test** (`governance_log_hash_chain_holds` + a NOTIFY LISTEN probe added in commit 5) covers both.

#### #34 — request_appeal unreachable for Decided cases
- **Code surface:** `crates/api/api_crud/src/governance/request_appeal.rs:107-115` — the `within_window` guard.
- **Fix shape:** replace `if case.closed_at.is_some() { NotFound }` with `let within_window = case.closed_at.map(|c| c > Utc::now()).unwrap_or(false); if !within_window { NotFound }`. Semantic: appeal allowed iff `closed_at` is set and in the future; NULL `closed_at` on a Decided case is a data error and rejects.
- **Blast:** ~6 LoC plus e2e regression. No migration. Touches no `governance_log` code and no federation code; isolated to the appeal handler.
- **Overlap with #48/#35:** **none in code** — different crate (`api_crud` vs `api`) and different module. Overlap is purely **thematic** (both are PR #10 CR unresolved threads blocking v0 tag).

#### #33 — declining juror self-replacement
- **Code surface:** `crates/api/api/src/governance/decline_jury_assignment.rs:135-142` — `exclude_person_ids` build-up before the `select_eligible_jurors` call at line 147.
- **Fix shape:** after the `jury_assignment::table.filter(...).select(person_id).load(conn)` that produces `current_assignees`, unconditionally push `caller_id` onto the mutable vec before passing it as the exclude list. The `ne(Declined)` filter removed the declining juror's row from the query results, so the caller's id has to be re-injected explicitly.
- **Blast:** 2 LoC (rename to `mut` binding + `push(caller_id)`) plus e2e regression. No migration. Touches only the decline handler.
- **Overlap:** **none in code** — touches neither `governance_log::append`, the `submit_jury_vote` decision block, nor the `request_appeal` handler. Same crate (`api`) as #48(b) but a different module entirely.

#### Recommendation

**Bundle: Option A (all four in one PR: `polish/critical-bugs`).**

Justification, in order of weight:

1. **#48(a) and #35 are inseparable.** The NOTIFY trigger fix already shipped on `governance-v0` via PR #46, but it only becomes correct once the INSERT+UPDATE are atomic. Splitting them reintroduces the observable gap (unsigned row) until the second PR merges. A single PR with the shared test is the minimum safe shape.
2. **#48(b) shares the outer transaction with #48(a).** Both writes live inside `submit_jury_vote.rs`'s post-decision `run_transaction`; re-ordering the `case_decided` append is a 1-commit change that a reviewer naturally reads alongside the atomicity wrap.
3. **#34 and #33 are tiny and mechanical.** #34 is a 6-line guard inversion; #33 is a 2-line push. Bundling them into a governance-invariants PR adds ~10 LoC of production code plus ~60 LoC of e2e fixtures. Reviewer cost is lower than a second round-trip (CodeRabbit run + human re-review) on two separate PRs.
4. **All four share the thematic umbrella** ("governance invariants that survived Phase 5c / PR #10 CR review and must be green before v0 tag") — the PR body can cite all four issues with one validation matrix. CodeRabbit is configured for governance-v0-targeted reviews (`.coderabbit.yaml`); a single focused PR gives CR one review pass.
5. **Task-per-commit history is preserved.** The polish-1 branch's 4 fix commits (one per issue) are merge-able without squash, keeping the retro link between each CR finding and its fix, which matches `phase-branch.md` rule intent.

**Split justification if any:** none recommended. #33 is the weakest candidate for bundling (different module, no shared surface with the other three); the only reason to split it would be if its regression test turns out to be expensive or reveals a deeper jury-selection bug that drags scope. If that happens mid-validation, **cut a new `polish/jury-self-exclude` branch from `polish/critical-bugs@93c8922eb`** (cherry-pick the #33 commit off) and ship the remaining three first.

**Open questions to surface via DQ:** none from code analysis alone. The session-save at lines 172-175 already flags two real concerns that belong in the DQ or the PR body, not here:
- Whether the `run_transaction`-inside-`run_transaction` SAVEPOINT semantic holds across **all** `governance_log::append` call sites under e2e load (`governance_log_hash_chain_holds` is the authoritative check).
- Whether any existing e2e test asserts "request_appeal must fail after decide" and now false-reds under the #34 semantic shift (audit needed before merge).

Both are validation-phase concerns, not scoping concerns.

### T3 — docs-sweep scope

**Verified against worktree HEAD `a25e6bbe0` on `polish/critical-bugs` (tip of `governance-v0` + kickoff runlog commit), 2026-04-19.**

#### File inventory

| Issue | File(s) | Type | Line-range estimate |
|---|---|---|---|
| #38 | `crates/api/api/src/governance/accept_jury_assignment.rs` (lines 7, 12 — `//!` docstring) | Rust doc-comment only | ~2 lines |
| #38 | `crates/db_views/governance_case/src/impls.rs` (lines 171, 187 — `///` docstring) | Rust doc-comment only | ~2 lines |
| #38 | `.claude/PRPs/reports/phase-5c-complete-report.md` (line 27 — relative link) | md | 1 line |
| #39 | `docs/brehon-law-inspired-network/SUBSCRIPTIONS.md` (§43-area, gap semantics) | md | ~15–25 lines (split one paragraph into 3 bullets) |
| #50 | `.claude/PRPs/plans/phase-6-federation.plan.md` (fences at 97 + 848; enum names at 480/492/493) | md (ASCII + SQL fences) | 5 lines (2 fences + 3 enum-name swaps) |
| #51 | `.claude/decision-queue.json` (entry `id: 38`, DQ-6.7 `question`/`answer` text) | json (string fields only) | ~3–6 lines of string-value rewrites |
| #52 | `.claude/rules/task-hopper.md` (lines 152, 177, 181 area) | md (auto-loads in -p mode) | ~15–25 lines (Option B carve-out or Option A rewrite) |

#### Code-surface check

- **All files markdown/json documentation:** mostly — #38 touches 2 `.rs` files but the edits are entirely inside `//!` / `///` doc-comment blocks (grep-verified: only `plan §11.4`, `plan §11.7`, `line 377` matches appear inside doc-comment lines). No code semantics, no `#[doc = ...]` attributes that would feed macros. **Cargo-clean: yes.**
- **Rule files touched (auto-load in -p mode):** `.claude/rules/task-hopper.md` (#52). This file auto-loads when `claude -p` reads the rules dir. Edits are prose-only wording changes (no hook YAML, no slash-command invocation strings that scripts would grep). Low risk, but Option A (route recovery through `task-hopper.sh escalate`) would couple the doc edit to a script change — confirm the sweep uses **Option B** (prose-only "advisor-only repair" carve-out) to keep this in docs-only scope.
- **Hidden-code risks:**
  - **#50 SQL snippet (lines 475–499):** `CREATE TABLE federation_attestation (...)` is in a `sql` code fence inside the plan doc. Not executed; no tooling greps this plan for DDL. Safe.
  - **#50 ASCII fences (97 + 848):** tagging `text` is a markdownlint-only concern. No ripgrep / MD040 CI wiring consumes these tags for logic. Safe.
  - **#39 SUBSCRIPTIONS.md:** contains a SQL block (lines 33–38) as reference, but #39 edits the prose *following* it. No change to SQL. Safe.
  - **#51 decision-queue.json:** schema-wise this is a free-form `answer` string plus `question` string on an already-resolved entry (`answered_by: impl-self-resolved`). No `task-hopper.schema.json`-adjacent field. Per memory `feedback_python_utf8_encoding_windows.md`, use the Edit tool directly (not Python) to preserve `§` and `—`.
  - **#38 Rust doc edits:** `cargo doc` consumes these — a malformed link in a docstring would fail `cargo doc`, but the edits here are plain-text "plan §11.X" → "Phase 5c task N" substitutions. Safe. No `cargo check` exposure (docstrings are not compiled).

#### Overlaps

- **None.** Each issue touches disjoint files. Closest proximity:
  - #51 (`.claude/decision-queue.json`) and #52 (`.claude/rules/task-hopper.md`) both live under `.claude/` but are separate files.
  - #50 and #38 both touch the PRPs tree (`.claude/PRPs/plans/` vs `.claude/PRPs/reports/`) but distinct files.
- No same-file collision → edit order is free.

#### Recommendation

- **Single PR shape:** **yes.** One branch `polish/docs-sweep` off `governance-v0`, one commit `chore(docs): v0-polish docs sweep — #38 #39 #50 #51 #52` (or per-issue commits if task-per-commit is preferred for CodeRabbit review; diff stays small either way — ~60–90 lines total across 7 files).
- **Any splits:** **none.** #51 is genuinely docs-only — it edits `question`/`answer` string fields on an already-resolved DQ entry, not schema or active state. Keep with the sweep.
- **Ordering constraint:** **none between the 5 issues.** One constraint vs. other polish PRs: #52 should pick **Option B** (prose-only carve-out, per issue body's "faster to land and matches how the rule was actually used in Phase 6") — Option A would extend `scripts/brehon/task-hopper.sh`, pulling the sweep out of docs-only scope. If a later polish wave decides on Option A, reopen as a separate code PR.
- **Parallel-safety vs polish-1:** docs-sweep touches zero files that polish-1 (`#48 #35 #34 #33`) touches. `governance_log::append`, `submit_jury_vote`, `federation_outbox`, `request_appeal`, `admin_assign_jury` are all `.rs` handler sources. Docs-sweep is safe to run in parallel with polish-1 in a separate worktree.

---

## Session save (2026-04-19T17:50Z)

**Status:** 4 of 5 polish-1 commits shipped; validation + e2e tests still TODO.

**Branch:** `polish/critical-bugs` — pushed, tip at `5cb868827`.

### Commits on branch (chronological)

| SHA | Subject | Scope |
|---|---|---|
| `a25e6bbe0` | chore(runlog): v0-polish kickoff — Impl1+Impl2 assignments | runlog |
| `4a406cd74` | chore(runlog): T1 triage findings | runlog |
| `1bb622e8a` | fix(governance-log): wrap append INSERT+UPDATE in run_transaction | #48(a)+#35 |
| `93c8922eb` | fix(governance): log case_decided before federation_sanction_sent | #48(b) |
| `034409128` | fix(governance): appeal window compares closed_at to now | #34 |
| `5cb868827` | fix(governance): exclude declining juror from replacement pool | #33 |

### NOT yet done (for next session)

- **Validation sweep.** Need to run:
  1. `cargo check --workspace --features full`
  2. `cargo clippy --workspace --no-deps --features full -- -D warnings`
  3. `cargo test --test e2e --no-run -p lemmy_server`
  4. `cargo test --test e2e -p lemmy_server` (expect 14 passed / 0 failed / 3 ignored)
- **e2e regression tests.** #34 and #33 fixes need dedicated tests per plan §"PR structure" commit 5:
  - `appeal_golden_path_after_decide` — decide a case, immediately call `request_appeal`, expect 200
  - `decline_replacement_does_not_pick_decliner` — assign juror A, A declines, trigger replacement, assert A not in new panel
- **PR creation.** After validation green, `gh pr create --repo barrie-cork/lemmy --base governance-v0 --head polish/critical-bugs` with body referencing GH #48, #35, #34, #33.
- **#48(c) deferral note.** The hidden `actor_pseudonym` write in `federation_outbox.rs:159-160` is NOT fixed here — defer to polish-N per plan. Add a comment at the call site clarifying the idempotency invariant.

### Impl2 progress (external)

Impl2 is working `polish/docs-sweep` branch separately (GH #38/#39/#50/#51/#52). No file overlap with Impl1's scope. Check `origin/polish/docs-sweep` when resuming to confirm landing status.

### Open concerns

- **#48(a) behavioural change.** Wrapping `governance_log::append` INSERT+UPDATE in `conn.run_transaction` is load-bearing on the "nested run_transaction becomes SAVEPOINT" semantic. Diesel-async docs confirm this, but the e2e suite is the authoritative check. If `governance_log_hash_chain_holds` or `sanction_notice_round_trip` fails, the tx wrap may be interacting badly with caller-owned tx reborrows.
- **#34 semantic shift.** The old guard rejected every Decided case. Switching to `closed_at > now()` means existing Decided cases in test fixtures that have `closed_at` stamped by a fixture helper (not `submit_jury_vote`) may now accept where they previously rejected. Any test that relied on "appeal must fail" post-decide is now broken. Audit `tests/e2e.rs` for `request_appeal` and expected-failure assertions.
