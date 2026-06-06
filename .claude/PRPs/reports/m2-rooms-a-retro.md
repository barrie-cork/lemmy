# Retro — m2-rooms-a (Matrix Room Provisioning + Hash-Chain Emission)

**Phase:** m2-rooms-a · **PR:** #191 (merged daemon-local @ `9a878d10e`, pushed `205ba2399`) · **Trunk:** `governance-v0` @ `9e65cb9d6`
**Authored:** 2026-06-06 (advisor, brehon-fork-m2rooms-a lane).
**Ran under:** four-role model (advisor + planning + impl + BM), Mode A dedicated lane worktree.

This retro covers the full lifecycle: planning (prior session), T0–T6 + T1w impl (this session), CR triage + cr-fix-1 recovery, and merge. The factual record is the verify report (`m2-rooms-a-verify.md`) + the session runlog (`m2-rooms-a-runlog.md`); this interprets it.

---

## What surprised us

- **Advisor (OOM cancel):** #635 (cr-fix-1) **violated NO-CARGO-ON-ELITEDESK** by running `cargo check --workspace --features full` on the daemon instead of the bridge-scoped `cd services/bridge && cargo check`. The brief said "run `cd services/bridge && cargo check`" (positive instruction) but did NOT say "do NOT run `--workspace`." The worker treated the workspace check as a complementary verification step. Result: concurrent with #634 on another lane, swap exhausted to 48 Ki, load 38, D-state claude process. **The brief's positive instruction was not sufficient — the prohibition must be stated explicitly.** Recovery via pre-cancel tar + advisor-laptop direct application was clean (≤30 min), but an OOM near-miss on a 2-fix-line task is avoidable.

- **Advisor (T2 juror-sourcing, DQ -055):** The plan assumed `juror_pseudonyms` were reachable from `bridge_notify.rs` via `crates/api/api`. They are not — `api` is not reachable from `api_utils`. The plan's §5.2 ≤2-crate ceiling created ambiguity about whether the join query belonged in `api_utils` or `api`. DQ -055 resolved inline per user directive (option-a: join as free fn in `bridge_notify.rs`, `api_utils` directly, 2 crates). **The T2 data-flow seam was novel enough to warrant a pre-impl data-flow trace — lesson already authored (`feedback_trace_data_flow_precedent.md`).**

- **Advisor (bm-merge path):** BM task #637 performed a **daemon-local finalize merge** (the same pattern used for impl-task branches) rather than calling `gh pr merge 191`. The BM brief specified the exact `gh pr merge` command; the worker followed the daemon-finalize-merge pattern from its SKILL context instead. PR #191 was left OPEN on GitHub after the daemon push. The close + head-branch delete required manual advisor intervention. **The bm-merge brief must more aggressively distinguish "daemon finalize" (for worker branches) from "GitHub PR merge" (for phase branches via `gh pr merge`).**

- **Advisor (merge-forward DIRTY state):** When the advisor pushed governance-fix-high-1 (cherry-pick) + meta commits to governance-v0 between T6 and bm-pr, PR #191's `mergeStateStatus` flipped to `DIRTY`/`CONFLICTING`. The merge-forward required merging `origin/governance-v0` into `phase-m2-rooms-a` and resolving a DQ conflict (`--ours` for the 320-entry superset). **The timing of cross-phase fixes to governance-v0 creates merge-forward friction for any open PR — the pattern is expected, but the DQ dedup discipline (scan both arrays after conflict) is load-bearing.**

- **Impl (T4a §G4 cycle):** T4a required one §G4 fix cycle (E0432 — missing `diesel`/`diesel_async` deps in `api_utils/Cargo.toml`). This is the same class as T1w's fix cycle. **Two fix cycles for the same error class on the same phase (E0432 missing deps in api_utils) indicates the brief template for `api_utils`-touching tasks should pre-check Cargo.toml deps as a mandatory step.**

---

## What to change

- **Impl-task briefs touching `services/bridge/`:** Add an explicit hard prohibition: "Do NOT run `cargo check --workspace --features full`. ONLY `cd services/bridge && cargo check`. Running the workspace check on the daemon violates NO-CARGO-ON-ELITEDESK." Positive instruction alone is insufficient for cargo scope.

- **bm-merge brief template:** Add a hard-refusal callout distinguishing daemon-finalize (worker branches, automatic) from `gh pr merge` (phase PRs, explicit API call). The current template lists the exact command but does not distinguish it from the daemon-finalize pattern a BM worker may have in its SKILL context.

- **`api_utils`-touching impl-task briefs:** Before dispatching, check `crates/api/api_utils/Cargo.toml` deps against the planned imports and add missing deps to §4 Constraints. The E0432 class recurred twice (T1w and T4a) on the same file class — it belongs in the mandatory file-class lesson injection table.

---

## What to carry forward

- **Advisor:** The **pre-cancel SSH tar pattern** (`ssh homeserver "tar czf /tmp/job-<id>-recovery-... -C .junior/worktrees job-<id>"`) is now confirmed effective for OOM-cancelled workers with uncommitted code (2 prior uses in catch-fire entries; this session confirmed it at the advisor-apply step). Continue.

- **Advisor:** **bm-merge UNSTABLE bypass via `--admin`** is correct for the adr-compliance advisory check. Pattern now confirmed on 3 consecutive phase merges (m1-b PR #177, m2-core-hook PR #184, m2-rooms-a PR #191). The lesson `feedback_bm_merge_unstable_admin_bypass.md` covers this; no new lesson needed.

- **Advisor:** The **sensitive-file classifier** blocking BM workers from writing `.claude/PRPs/reviews/*.yaml` and `.claude/runlog/*.md` has now recurred 3 times (m1-a #597, deps-r2 #603, m2-rooms-a #635/this session). **MEMORY.md already flags this as action-needed.** Resolution: add these paths to the BM agent allow-list. Carry forward explicitly to next session.

- **Impl:** The **rusqlite version conflict** (libsqlite3-sys, bridged to 0.37 via `56de6da77`) is an expected artifact of `matrix-sdk-sqlite`. Any future task adding a new SQLite-adjacent dep to `services/bridge/Cargo.toml` must verify compatibility with the pinned rusqlite version.

---

## Lessons promoted this phase

| Lesson | Status | Note |
|---|---|---|
| `feedback_trace_data_flow_precedent.md` | **already authored** | T2 juror-sourcing seam — trace data flow before proposing missing-data seam. Authored from DQ -055. |
| `feedback_bm_merge_unstable_admin_bypass.md` | **confirmed (3rd use)** | No new lesson; pattern solid. |
| Impl cargo scope on daemon (positive instruction insufficient) | **NEW — author next session** | Brief must state both what to run AND what NOT to run for cargo scope. |
| `api_utils` E0432 recurrence | **file-class lesson injection needed** | Add `crates/api/api_utils/Cargo.toml`-touching tasks to mandatory dep-check step in brief template. |

---

## §5 Quantified outcomes

**Tasks:** T0 (preflight/config/store) + T1 (bridge_room.rs store) + T1w (juror_pseudonyms producer, inline-planned) + T2 (room_provisioner.rs consumer) + T3 (hash-chain append_room_event) + T4a (bridge_auth.rs + room_event_handler.rs) + T4b (bridge_read messaging-status route) + T5 (soft_pause bearer auth + OQ-009 threshold wire) + T6 (room_provisioning test suite). **8 tasks + 1 inline-planned task addition (T1w) + 2 §G4 fix cycles (T1w-fix-1, T1w-fix-2, T4a-fix).** Total: 11 Junior dispatches (impl) + 1 bm-pr + 1 bm-merge + 1 cancelled cr-fix-1.

**CR triage (PR #191):** 8 findings, 0 critical, 3 done (cr-5/6/7 — bridge fixes), 5 rebut (cr-1/2/3/4/8). 3 of 8 actioned (37.5%). All fixes landed in commit `521e949e3` (advisor-laptop direct apply after OOM recovery). open_critical=0, fix-in-pr=0 at gate 5.

**Gates:** 6 user gates — plan approval (prior session), DQ -055 inline resolution (user directive, gate 1 variant), CR triage (gate 3), merge confirm (gate 5, "yes"), retro (gate 6, this). 1 catch-fire adjacent (OOM + cancel + recovery). 0 forbidden-window deferrals.

**Validation:** Bridge cargo check (advisor-laptop, EXIT 0, `521e949e3`). 8 validate-pending-laptop DQ entries across all tasks, all resolved. `/brehon-verify` 3 stories all ✓ (`65f059f62`). Linux gate: skipped (no Cargo.toml/Cargo.lock diff scope trigger beyond `bridge/Cargo.toml`; bridge-only workspace is excluded from workspace-level Linux check scope).

**DQ:** 13 m2-rooms-a DQ entries total: 8 validate-pending-laptop (all pass→resolved), 1 validate-pending-laptop cr-fix-1 (pass→resolved, advisor-laptop), 1 clarify-DQ -055 (resolved inline), 1 blocker T2 juror-sourcing DQ -055 (resolved inline), 2 log entries. 0 pending at merge.

**Per-task complexity** (files/commits/runtime from runlog):
| Task | Files | Commits | Runtime | Max silence |
|---|---|---|---|---|
| T0 (preflight+config+store) | 3 | 2 | ~8 min | <5 min |
| T1 (bridge_room.rs) | 3 | 2 | ~8 min | <5 min |
| T1w (juror_pseudonyms, inline) | 2 | 1 | 4.5 min → fix-1 (2.5 min) → fix-2 (1.5 min) | <3 min |
| T2 (room_provisioner.rs consumer) | 2 | 1 | 8.5 min | <5 min |
| T3 (hash-chain emit) | 3 | 1 | 16 min | <8 min |
| T4a (bridge_auth.rs + room_event_handler.rs) | 4 | 1 + fix (§G4) | ~18 min | <8 min |
| T4b (bridge_read messaging-status) | 2 | 1 | ~10 min | <5 min |
| T5 (soft_pause wire + auth) | 3 | 1 | ~12 min | <5 min |
| T6 (room_provisioning tests) | 2 | 1 | ~18 min | <8 min |
| cr-fix-1 (config + oq009 + dead fns) | 2 | advisor-direct | ~20 min total (recovery) | n/a |

No task flagged >8 files. One anomaly: T1w required 2 §G4 fix cycles (E0432 diesel deps, E0599 JoinOnDsl) due to `api_utils` Cargo.toml gap. T4a required 1 §G4 fix cycle (same class — E0432 diesel deps). **Both from the same root cause; briefs should pre-check `api_utils` deps.**

---

## Outstanding (next session)

- **DQ `b2fcddebf2f8-001` pending** — `governance-fix-med-1` validate-pending-laptop (separate lane task #636). Triage at next session start.
- **Sensitive-file classifier fix** — add `.claude/PRPs/reviews/*.yaml` + `.claude/runlog/*.md` to BM agent allow-list. 3rd recurrence; action-needed.
- **`api_utils` dep pre-check** — add lesson/brief-template step for `api_utils`-touching tasks.
- **bm-merge brief template** — add daemon-finalize vs `gh pr merge` distinction callout.
- **/brehon-phase-transition** — mark m2-rooms-a closed in roadmap, update MEMORY.md workflow state.
