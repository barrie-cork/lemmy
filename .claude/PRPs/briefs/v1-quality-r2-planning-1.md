# Planning Brief — v1-quality-r2: PR #155 carry-forward bundle (env-guard refactor + DQ duration lint + fixtures doc audit)

**Phase:** v1-quality-r2
**Branch:** phase-v1-quality-r2 (to be cut from `governance-v0` — exact SHA at lane-cut time)
**Authored:** 2026-05-28
**Authored by:** advisor (canonical brehon-fork session)
**PRD:** (none — quality/cleanup bundle from PR #155 BM triage; no PRD needed)
**Plan target:** `.claude/PRPs/plans/v1-quality-r2.plan.md`

**STATUS: AUTHORED 2026-05-28, AWAITING BM-CUT.** Roadmap entry exists at `.claude/PRPs/v1-roadmap.json` `lanes.quality.sub_phases.v1-quality-r2` (status `unstarted`, `deferred_until` "post-v1-RT-r3 merge"). Hard precondition satisfied 2026-05-28: PR #155 (v1-RT-r3) merged via `5ebd8ae23`, trunk at `1edb8b94c`. All 4 RT-r3-introduced symbols this brief references — `EnvVarGuard`, `v1_rt_r3_fixtures`, `emit_reputation_event_local`, `emit_reputation_event` (federated path) — are present on `governance-v0` post-merge (verified 2026-05-28 via `git grep`).

**Lane mode:** Mode B (mobile remote-control) per `multi-lane-worktree.md` §"Lane modes". Brief authored on `governance-v0` in canonical brehon-fork checkout; trunk→phase sync via daemon SSH after bm-cut.

**Concurrent lanes at brief-author time:**
- `phase-v1-redaction-r1` Lane A — IN-FLIGHT (planning Junior dispatched 2026-05-28 as task #485). Files touched: `crates/db_schema/src/source/governance/redaction.rs` + e2e tests for redaction. **Zero overlap** with this lane (verified: redaction.rs is in `db_schema`, not in any file this lane touches; e2e redaction tests are in a different region of e2e.rs from the fixtures modules this lane audits).
- `brehon-fork-rt-r3` lane worktree — RT-r3 SHIPPED, worktree pending cleanup. Not active.
- No other active lanes.

**Two-lane concurrency safety per `feedback_cohort_shared_git_index_contention.md`:** Lane A + this lane = 2 concurrent lanes on the single daemon `.git/`. Below the cohort-≥3 `.git/index.lock` contention threshold. Safe to run in parallel for planning + early impl stages. If both lanes hit their e2e gates within the same window, serialize on the laptop (per handover §"Parallel-lane safety constraints" point 3).

---

## 1. What this phase delivers

A targeted quality sweep retiring **5 PR #155 carry-forward issues** filed during the v1-RT-r3 BM triage (2026-05-28). The 5 issues group into **3 logical clusters** by defect class, not 5 independent tasks. The planner MUST decide whether to ship as one PR or split — see §3 watchpoints WP-Cluster.

### The 5 issues (issue bodies read at brief-author time 2026-05-28)

| Issue | Severity | Source CR id | File(s) | Cluster | Defect class |
|---|---|---|---|---|---|
| #156 | major (cr-2) | PR #155 | `crates/server/tests/e2e.rs` (ALL `*_fixtures` modules) | **C1** | Doc-vs-implementation drift on safety claims for process-env mutation |
| #157 | minor (cr-4) | PR #155 | `.claude/decision-queue.json` + new `scripts/brehon/dq-lint-durations.sh` + `scripts/brehon/precheck.sh` | **C2** | DQ entries with `resolved_at < timestamp` (negative duration) |
| #158 | nit (cr-7) | PR #155 | `crates/api/api/src/governance/{admin_emergency_remove,submit_jury_vote}.rs` + new `governance/reputation_helpers.rs` | **C3** | Code duplication: `emit_reputation_event_local` vs `emit_reputation_event` |
| #159 | minor (cp-4) | PR #155 | `crates/server/tests/e2e.rs` (ALL `boot_context` callsites, ~15-25 sites) | **C4** | Env-leak via early-`?`: `boot_context` env mutations not wrapped in `EnvVarGuard` |
| #160 | minor (cp-5) | PR #155 | `crates/server/tests/e2e.rs` (ALL `LEMMY_DATABASE_URL` setter sites) | **C4** | Env-leak via early-`?`: `LEMMY_DATABASE_URL` not wrapped (companion to #159) |

### Clusters

- **C1 (Issue #156) — fixtures doc-comment audit.** Audit every `*_fixtures` module in `e2e.rs` for the overstated safety claim on process-env mutation; either tighten doc-comment in every module to cite the `--test-threads=1` constraint, OR introduce a `#[brehon_single_thread_test]` attribute macro that asserts the constraint at compile/load time. **Planner decision** (§3 WP-1): doc-only sweep (low risk, all modules) vs macro introduction (higher risk, scope creep, new dep on syn/quote).

- **C2 (Issue #157) — DQ duration lint + bulk sweep.** Add `scripts/brehon/dq-lint-durations.sh` that scans every DQ entry for `resolved_at < timestamp` and fails non-zero. One-shot sweep of currently back-dated entries. Wire into `scripts/brehon/precheck.sh` so the issue can't recur. Pure-tooling cluster: zero `crates/**` edits, zero e2e impact.

- **C3 (Issue #158) — `emit_reputation_event` helper extraction.** Extract `emit_reputation_event_local` + `emit_reputation_event` into shared `crates/api/api/src/governance/reputation_helpers.rs`. **Planner gate** (§3 WP-2): the issue body says "wait until a third consumer materialises before introducing the abstraction (premature DRY is its own footgun)". As of 2026-05-28 there are 2 consumers. If the planner cannot identify a near-term 3rd consumer in the v1-fed-in-* roadmap, this cluster should be **DEFERRED** (filed as a `kind: "log"` DQ on this phase with rationale; issue #158 stays open with updated "blocked-on-3rd-consumer" comment). Do NOT extract speculatively.

- **C4 (Issues #159 + #160) — `EnvVarGuard` retrofit for legacy env mutations.** Refactor `boot_context` to return `(BootContext, EnvVarGuard)` instead of mutating env directly. Audit every `env::set_var("LEMMY_DATABASE_URL", ...)` callsite + every `boot_context` callsite in `e2e.rs`. Replace raw mutation with `EnvVarGuard::set()`. Verify each fixtures module's helpers thread the guard through so it's dropped on early-`?` from outer test fn. **Largest cluster by edits** (~15-25 callsite touches per #159; #160 is structurally the same fix at a different env-var). #159 and #160 ship together — solo #159 is "half a fix" per the #160 issue body.

### Cluster sizing summary

| Cluster | Issues | Files | Edit estimate | e2e.rs touch | Risk |
|---|---|---|---|---|---|
| C1 | #156 | e2e.rs fixtures modules (count TBD by planner) | doc-only (low) or macro (medium) | YES, all `*_fixtures` modules | low-medium |
| C2 | #157 | scripts/ + DQ JSON | ~80 LOC bash + N JSON edits | NO | low |
| C3 | #158 | governance/*.rs + new file | ~50 LOC helper + 2 callsite replacements | NO | **DEFERRABLE** per planner gate |
| C4 | #159 + #160 | e2e.rs (heavy) | 15-25 callsite touches across all fixtures modules | YES, heavy | medium-high |

**Total e2e.rs edits across all clusters:** C1 + C4 both touch e2e.rs. C1 may be doc-only (no Junior-worker-edit-hang per `feedback_junior_worker_e2e_edit_hang.md`). C4 is structural and DOES trigger the e2e-edit-hang risk. Planner MUST split C4 into multiple smaller impl tasks (≤200 lines each per the lesson).

---

## 2. Key file anchors (verify these before authoring the plan)

| File | Anchor | Cluster | Purpose |
|---|---|---|---|
| `crates/server/tests/e2e.rs` | mod `v1_rt_r3_fixtures` @ L17101 (governance-v0 post-merge) | C1, C4 | Canonical reference for the `EnvVarGuard` pattern; also one of the `*_fixtures` modules to audit for C1 |
| `crates/server/tests/e2e.rs` | `struct EnvVarGuard` @ L17179, `impl EnvVarGuard` @ L17184, `impl Drop` @ L17195 | C4 | The guard type C4 retrofits onto legacy callsites |
| `crates/server/tests/e2e.rs` | `let _guard = EnvVarGuard::set(...)` usage examples @ L17576, L17631, L17671, L17715 | C4 | Pattern to mirror in retrofitted callsites |
| `crates/server/tests/e2e.rs` | `pub fn boot_context(...)` definition + ALL callsites — planner MUST `rg "boot_context" crates/server/tests/e2e.rs` to enumerate | C4 | Refactor target for #159 |
| `crates/server/tests/e2e.rs` | `env::set_var("LEMMY_DATABASE_URL", ...)` callsites — planner MUST `rg 'env::set_var\("LEMMY_DATABASE_URL"' crates/server/tests/e2e.rs` to enumerate | C4 | Refactor target for #160 |
| `crates/server/tests/e2e.rs` | every `mod *_fixtures` declaration — planner MUST `rg "^mod \w+_fixtures \{" crates/server/tests/e2e.rs` to enumerate | C1 | Modules to audit for overstated-safety doc-comments |
| `crates/api/api/src/governance/admin_emergency_remove.rs:448` | `async fn emit_reputation_event_local(...)` | C3 | One of two helpers C3 unifies |
| `crates/api/api/src/governance/submit_jury_vote.rs` | `emit_reputation_event` — planner MUST `rg "fn emit_reputation_event" crates/api/api/src/governance/` to confirm location | C3 | Other helper C3 unifies |
| `crates/api/api/src/governance/` | (new file) `reputation_helpers.rs` | C3 | Proposed extraction target IF planner gate clears |
| `.claude/decision-queue.json` | entries 3999, 4007, 4016, 4035 (per #157 issue body) + any other back-dated entries the lint surfaces | C2 | Bulk sweep candidates |
| `scripts/brehon/dq-lint-durations.sh` | (new file) | C2 | Lint script C2 ships |
| `scripts/brehon/precheck.sh` | wire dq-lint-durations into precheck flow | C2 | Recurrence prevention |
| `.claude/PRPs/reviews/pr-155-findings.yaml` | rows `cr-2`, `cr-4`, `cr-7`, `cp-4`, `cp-5` (per issue bodies) | all | Source-of-truth findings YAML |

**Planner MUST re-enumerate at plan-author time** — per `feedback_fix_impl_enumerate_all_callsites.md`. If `rg "boot_context" crates/server/tests/e2e.rs | wc -l` returns < 15 or > 25, OR if `rg 'env::set_var\("LEMMY_DATABASE_URL"' crates/server/tests/e2e.rs | wc -l` returns < 1, file a `kind: "blocker"` DQ before authoring.

---

## 3. Watchpoints for the planner

### WP-Cluster — single-PR vs multi-PR decision

Two valid framings:

**Option A — single PR.** All 4 in-scope clusters (C1/C2/C3-if-cleared/C4) ship in one PR `phase-v1-quality-r2`. Pros: amortizes one e2e gate (~26 min on laptop); aligns with the v1-deps-r2 precedent (4 unrelated change classes in one PR). Cons: a CR finding on C4 blocks C2's pure-tooling ship.

**Option B — two PRs.** Split into `phase-v1-quality-r2a` (C2 tooling + C3 helper extraction if gate clears) and `phase-v1-quality-r2b` (C1 doc audit + C4 EnvVarGuard retrofit, both touching e2e.rs). Pros: tooling cluster lands fast; e2e.rs-touching clusters serialize naturally with Lane A redaction's e2e gate. Cons: two CR cycles, two merge gates.

**Planner decision** — record rationale in plan §1 commentary. Default to Option A unless C4's callsite enumeration exceeds 25 sites (then split because the impl-task count balloons).

### WP-1 — C1 (Issue #156) doc-only vs attribute-macro decision

The issue body offers two paths:
- **(a) Doc-only sweep.** Tighten every `*_fixtures` module's doc-comment to cite the `--test-threads=1` constraint explicitly. Mechanical. Zero new code surface. Locks in the contract via prose.
- **(b) Attribute macro.** Introduce `#[brehon_single_thread_test]` that asserts the constraint at compile/load time. Stronger guarantee. Adds proc-macro dependency (`syn`, `quote`, `proc-macro2`) to the test crate. Larger surface for review.

The issue body recommends (a) "tighten the doc-comment in every module" as the default and (b) "or introduce" as the more-rigorous alternative. **Default to (a)** unless the planner identifies a concrete near-term risk that (b) prevents — e.g., a recurring pattern of contributors adding new fixtures modules without the constraint comment. Record decision rationale in plan §3.

### WP-2 — C3 (Issue #158) premature-DRY gate

The issue body is explicit: "wait until a third consumer materialises before introducing the abstraction (premature DRY is its own footgun)". As of 2026-05-28, there are 2 consumers (`admin_emergency_remove` + `submit_jury_vote`).

**Planner gate procedure:**
1. Read `.claude/PRPs/v1-roadmap.json` for v1-fed-in-* sub-phases (`lanes.federation.sub_phases`).
2. Check if any unstarted/in-flight sub-phase will introduce a 3rd reputation-event-emitter (likely inbound federation handlers that translate remote sanctions into local rep deltas).
3. If yes (3rd consumer concretely in scope within next ~2 weeks) → INCLUDE C3 in this phase.
4. If no → **DEFER C3**: file a `kind: "log"` DQ on this phase with body `"C3 (Issue #158) helper extraction deferred per premature-DRY gate; 2 consumers as of plan author time; revisit when 3rd materialises (likely v1-fed-in-* sub-phase)"`. Update issue #158 with a comment citing the deferral. Re-scope this phase to C1+C2+C4 only.

The DEFER path is the expected default per the issue body's tone. Do NOT extract the helper just because the brief lists it — the gate is real.

### WP-3 — C4 (Issues #159+#160) e2e.rs edit-hang risk

Per `feedback_junior_worker_e2e_edit_hang.md` + `feedback_fix_impl_pre_locate_e2e_anchors.md`: Junior `impl-task` workers hang or fail when editing into e2e.rs (8945+ lines) with large or unbounded Edits. Hard rules for this cluster:

- Split C4 into ≥2 impl tasks. Suggested split: Task A = `boot_context` refactor + ~half the callsites; Task B = remaining callsites + `LEMMY_DATABASE_URL` retrofit. Plan-§5.1 complexity must remain ≤6 per task.
- Each Junior impl-task brief MUST pre-locate verbatim `old_string` / `new_string` anchors per `feedback_fix_impl_pre_locate_e2e_anchors.md` (scope gate ≤150 lines, ≤2 file edits, ≤2 Edits/file). This planning brief does NOT yet pre-locate — that's the planner's job at plan-§13 task-level.
- Plan §3 Required Reading MUST include both lessons + a §16a verification story per cluster cluster.

### WP-4 — C2 (Issue #157) lint script must handle schema-v3 composite ids

The DQ lint walks every entry in `.claude/decision-queue.json`. Schema-v3 entries have composite ids `<session_id>-<sequence>` (e.g. `a1b2c3d4e5f6-001`) per `.claude/rules/decision-queue.md` §"Schema (v3)". Pre-v3 entries have integer ids. The lint MUST handle both id shapes and report by id verbatim.

Lint output format suggestion: `DQ-LINT FAIL: entry "<id>" has resolved_at (<rfc3339>) earlier than timestamp (<rfc3339>) by <duration>`. Exit non-zero on any finding.

### WP-5 — C2 bulk sweep policy decision

The issue body offers two sweep approaches for currently-back-dated entries:
- **(a) Floor.** Set `resolved_at = max(timestamp, current_value)`. Zero-effort, lossy: the actual resolution timestamp is hidden under a synthetic floor.
- **(b) Re-author.** Re-derive the timestamp pair if the resolution actually happened earlier in real time (e.g. consult git log for the resolving commit).

Issue body says "or re-author the timestamp pair if the resolution actually happened earlier in real time". Default to (a) for entries where git history doesn't unambiguously reveal the resolution moment; use (b) only for entries where a single resolving commit is identifiable. Plan §13 should NOT mass-execute (b) — the planner picks the policy.

### WP-6 — C1+C4 + Lane A redaction e2e.rs concurrency

Lane A (`phase-v1-redaction-r1`) is in-flight at brief-author time. Lane A's planning Junior may add new e2e.rs redaction tests. C1 (this lane) audits all `*_fixtures` modules — including any new module Lane A adds. C4 (this lane) touches every `boot_context` callsite — including any new callsite Lane A adds.

**Mitigation:** Lane A's planning task #485 will finish before this lane's impl tasks start (planning is a single Junior task, ~5-15 min; this lane's bm-cut + planning + impl-dispatch is sequential). By the time C1/C4 impl tasks run on this lane, Lane A's planning + bm-pr cycle will have either landed or be visible. The planner SHOULD `git fetch origin phase-v1-redaction-r1` at plan-author time and audit any e2e.rs delta from Lane A.

**Hard rule:** if Lane A merges to `governance-v0` between this lane's bm-cut and this lane's first impl-task dispatch, advisor MUST sync `governance-v0` into `phase-v1-quality-r2` (same SSH-merge procedure as the bm-cut→planning sync) before dispatching any C1/C4 impl task.

### WP-7 — Schema-v3 DQ writes during phase (mid-task push discipline)

Per `.claude/rules/decision-queue.md` Hard refusal #9: never use the abolished `next_id = max(all_ids)+1` recipe; always `bash scripts/brehon/dq-v3-new-entry.sh`. Junior workers raising DQs on this phase MUST follow Recipe 1/2/3 in `.claude/refs/dq-recipes.md`. The C2 lint script does NOT mutate DQ entries; it only reports. The C2 bulk sweep is a separate one-shot edit, NOT a DQ write — it edits existing entries' `resolved_at` field in place via a script (`scripts/brehon/dq-fix-back-dated-resolved-at.sh` suggested name).

### WP-8 — Schema-v3 entry id collision across lanes

Lane A (redaction) + this lane (quality-r2) may each raise DQ entries during impl. Per `.claude/rules/multi-lane-worktree.md` §"Worktree-aware DQ id discipline" + v3 schema, each CC session generates its own UUID prefix in `.claude/.dq-session-id`; collisions structurally impossible. **Planner does NOT need to coordinate ids** — the v3 mechanism handles it.

---

## 4. Out of scope

- **All other open PR #155 carry-forwards** that have an explicit different target sub-phase per their issue body (none as of 2026-05-28 — #156-#160 are the complete v1-quality-r2 set).
- **e2e flakes #42, #43, #45.** Per handover `.claude/PRPs/handovers/issue-triage-2026-05-28.md` §"Lane C", user deferred these as "e2e.rs is a shared-large-file hazard". They are NOT in v1-quality-r2 scope.
- **Pi scaffolding cleanup #114, #115, #116, #120.** Lower priority `.pi/*` harness work, user-deferred for a separate Pi-focused phase.
- **Issue #134 daemon defect.** Tracked-only; fix lives in `MCPs/junior-mcp` source, not this repo.
- **C3 helper extraction IF the premature-DRY gate (WP-2) defers it.** Planner re-scopes to C1+C2+C4 and files the deferral DQ.
- **Macro introduction for C1 (WP-1 option b)** unless planner identifies a concrete recurrence risk that warrants it. Default to doc-only sweep.
- **Touching code in `crates/db_schema/src/source/governance/redaction.rs` or its tests** — that's Lane A's territory.
- **Touching `crates/api/api/src/governance/redaction.rs`** (it's a 23-line re-export shim of the db_schema canonical) — also Lane A's territory.

---

## 5. Constraints

- **DQ HARD REFUSALS** (per `.claude/rules/decision-queue.md` Hard refusals — applies to all Junior workers on this phase):
  - #1: NEVER write `answered_by: "advisor"` from non-advisor sessions.
  - #5: NEVER ask open-ended questions; always provide ≥2 concrete options.
  - #8: NEVER write `approved_by` from non-advisor sessions.
  - #9: NEVER use `max(all_ids)+1` recipe; use `bash scripts/brehon/dq-v3-new-entry.sh`.
- **Schema-v3 DQ append discipline:** all new DQ entries via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json>` per `feedback_dq_v3_append_via_helper_script.md` — no inline Python, no heredoc-EOF.
- **Mid-task push discipline:** Junior workers writing pending DQ entries MUST commit + push the DQ in one atomic sequence (per `decision-queue.md` §"Mid-task visibility").
- **Attribution:** all DQ entries `from: "planner"` (planning task) or `from: "impl"` (impl tasks). Junior may self-resolve `kind: "log"` directly to `resolved[]` per Recipe 2.
- **File-class lesson injection (per `advisor-orchestrator.md` §2.4):**
  - Every C1/C4 impl-task brief touching `crates/server/tests/e2e.rs` MUST include `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` in §3 Required reading.
  - If C1 or C4 task involves ≥2 e2e.rs Edits in the task or cohort, also include `feedback_junior_worker_e2e_edit_hang.md`.
  - Any fix-impl-task targeting e2e.rs MUST include `feedback_fix_impl_pre_locate_e2e_anchors.md`.
  - C3 (if not deferred) impl-task creating `governance/reputation_helpers.rs` is a NEW file — no file-class lesson injection trigger, but planner SHOULD verify with `feedback_newtype_locations_lemmy_db_schema_vs_file.md` that the helper crate location is correct.
- **Forbidden execution windows non-binding** under Shape G (validate-pending-laptop active per `project_shape_g_suspended_2026_05_16.md`). Cargo runs on laptop, not EliteDesk.
- **Shape G note:** Shape G SUSPENDED until 2026-06-01. v1-validate-agent runs are NOT triggered by phase-v1-quality-r2 pushes — cargo gates run via validate-pending-laptop on the canonical session. Per `feedback_laptop_default_for_validate_pending.md`.
- **No CR-baiting commit messages.** Plan §5 retro instructions per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md`.
- **No conformance-audit invocation for this lane** unless the planner introduces NEW code in `crates/api/api/src/governance/**.rs` (C3's new `reputation_helpers.rs` qualifies IF C3 not deferred). Per `.claude/skills/brehon-conformance-audit/` invocation criteria.

---

## 6. Expected shape

| Stage | Junior tasks |
|---|---|
| Plan author | 1 planning task (`[role:planning]`) — this brief drives it |
| Plan approval | Advisor + user gate 1 (`advisor-orchestrator.md` §3.2 gate 1 + §3.4 DoD smoke test + §3.5 watchpoint specificity) |
| Impl | Depends on cluster split decision (WP-Cluster) — likely 3-5 impl tasks: C2 tooling (1 task), C1 doc audit (1 task, doc-only path), C4 boot_context refactor (1-2 tasks per WP-3 split), C4 LEMMY_DATABASE_URL retrofit (1 task; may bundle with C4 boot_context part B). C3 either 1 task or deferred (file DQ). |
| Validation | validate-pending-laptop (Shape G suspended) — workspace check + e2e test pass |
| bm-pr | 1 bm-task (`bm-pr`) into `governance-v0` |
| CR triage | bm-poll-cr + bm-triage; user gate 3 |
| Merge | bm-merge after critical findings clear; user gate 4 (e2e local-vs-dispatch) + gate 5 (merge confirm) |
| Retro | Advisor authors retro per `feedback_retro_not_report.md`; user gate 6 |

**Complexity estimate (Plan §5.1):** 6/10. Driver = C4 callsite count + e2e.rs touch surface. If C3 not deferred, bumps to 7/10. If WP-Cluster Option B (split into 2a + 2b), each sub-PR is 4-5/10.

---

## 7. Issue close conditions

On merge of phase-v1-quality-r2 to `governance-v0`:
- **#157** → close with reference to merge commit + PR; cite that `dq-lint-durations.sh` now blocks recurrence via precheck.
- **#159 + #160** → close together; cite the merge commit, the boot_context refactor, the LEMMY_DATABASE_URL retrofit, and the callsite count touched.
- **#156** → close with reference to merge commit + PR; cite the doc audit pass (or macro introduction) and the modules updated.
- **#158** → close ONLY IF C3 included; otherwise post deferral comment ("blocked-on-3rd-consumer; revisit when v1-fed-in-* introduces remote-sanction rep emission") and leave issue open.

---

## 8. Plan structure hints (for the planner)

Per `.claude/PRPs/templates/plan.template.md`:

- **§1 Goal** — 2-sentence summary: retire 5 PR #155 carry-forward issues from quality bundle; refactor legacy env mutations to use the RT-r3 EnvVarGuard pattern.
- **§2 Why** — quality debt accumulated during RT-r3 ship; carry-forwards filed 2026-05-28 against issue-tracker.
- **§3 Required reading** — this brief, `feedback_junior_worker_e2e_edit_hang.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`, `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`, `feedback_dq_v3_append_via_helper_script.md`, `feedback_fix_impl_enumerate_all_callsites.md`, PR #155 finalist findings YAML, the 5 issue bodies.
- **§4 Watchpoints** — copy WP-1 through WP-8 from §3 of this brief.
- **§5 Complexity score** — 6/10 baseline; split adjustment if Option B.
- **§13 Tasks** — see §6 expected shape; mark `[P]` only for tasks with verified file disjointness in YAML.
- **§15 Definition of Done** — `cargo check --workspace --features full` + `cargo clippy --workspace --features full -- -D warnings` + `cargo test --workspace --features full` + DQ lint exit 0 + `scripts/brehon/precheck.sh` exit 0.
- **§16a Stories** — one verification story per cluster (C1 = "every fixtures module's doc-comment cites --test-threads=1"; C2 = "dq-lint-durations.sh exits non-zero on a synthetic back-dated entry, zero on current trunk"; C3 = "helper exported from new module, both callsites use it" OR "deferral DQ filed"; C4 = "every boot_context callsite + every LEMMY_DATABASE_URL setter uses EnvVarGuard").

---

## 9. References

- `.claude/PRPs/handovers/issue-triage-2026-05-28.md` §"Post-RT-r3 unlock" + §"Audit trail"
- `.claude/PRPs/v1-roadmap.json` `lanes.quality.sub_phases.v1-quality-r2`
- `.claude/PRPs/briefs/v1-deps-r2-planning-1.md` — canonical deferred-precedent shape (this brief mirrors)
- `.claude/PRPs/reviews/pr-155-findings.yaml` — source-of-truth findings
- `.claude/rules/decision-queue.md` v3 schema + composite-id discipline
- `.claude/rules/multi-lane-worktree.md` §"Lane modes" + §"Brief location and trunk→phase sync"
- `.claude/rules/advisor-orchestrator.md` §2 (brief authoring), §3 (gates), §3.6 (canonical-schema-first gate fired this session — read v1-deps-r2 precedent first), §5 (validation)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md`
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md`
- `.claude/lessons/feedback_cohort_shared_git_index_contention.md`
- `.claude/lessons/feedback_dq_v3_append_via_helper_script.md`
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md`
- GitHub issues: #156, #157, #158, #159, #160 (all open as of 2026-05-28; bodies read at brief-author time)

---

## 10. Audit trail (this brief)

| Field | Value |
|---|---|
| Brief authored | 2026-05-28 (this session) |
| Trunk SHA at brief-author time | `1edb8b94c docs(retro): v1-RT-r3 phase retro - PR #155 merged 5ebd8ae23` |
| Hard precondition (RT-r3 merged) | SATISFIED 2026-05-28 via PR #155 merge `5ebd8ae23` |
| Symbol-existence checks on `governance-v0` | `EnvVarGuard` ✓ (e2e.rs L17179), `v1_rt_r3_fixtures` ✓ (e2e.rs L17101), `emit_reputation_event_local` ✓ (admin_emergency_remove.rs L448), `EnvVarGuard::set` callsites ✓ (e2e.rs L17576, L17631, L17671, L17715) |
| Issue bodies read | #156, #157, #158, #159, #160 — all read 2026-05-28 in this session |
| Concurrent-lane check | Lane A (`phase-v1-redaction-r1`) in flight; zero file overlap; below cohort-≥3 `.git/index.lock` threshold |
| Lane A planning Junior | Task #485 dispatched 2026-05-28 |
