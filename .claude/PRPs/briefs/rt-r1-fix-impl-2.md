---
phase: v1-RT-r1
role: impl-task
task: fix-2
brief_n: 2
authored: 2026-05-12
parent_phase_tip: 37a62f9b4
parent_dq: 205
companion_brief: rt-r1-impl-8-9-10-bundle.md
---

# [role:impl-task] v1-RT-r1 fix-impl-2 — pad 8 remaining `ReputationEventInsertForm` literals — see .claude/PRPs/briefs/rt-r1-fix-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 fix-impl-2 — pad 8 remaining ReputationEventInsertForm literals across 3 files (create_endorsement, seed_founders, e2e)`

## §2 Scope

### §2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

> | Failure signature | Auto-fix | Source lesson |
> |---|---|---|
> | `error[E0063]: missing fields` `dedupe_key` and `source_event_type` in initializer of `ReputationEventInsertForm` | pad each `<Type> { ... }` struct-literal initializer with `dedupe_key: None,` + `source_event_type: None,` (callers pass `None`; NOT-NULL DB DEFAULT covers `source_event_type`, `dedupe_key` is nullable) | `feedback_insertform_default_propagation.md` |

**Callsite enumeration** (per `feedback_fix_impl_enumerate_all_callsites.md` and DQ #205):

```
crates/api/api_crud/src/governance/create_endorsement.rs:278  ReputationEventInsertForm { ... }
crates/api/api_crud/src/governance/create_endorsement.rs:291  ReputationEventInsertForm { ... }
crates/tools/seed_founders/src/main.rs:173                    ReputationEventInsertForm { ... }
crates/server/tests/e2e.rs:2902                               ReputationEventInsertForm { ... }
crates/server/tests/e2e.rs:2925                               ReputationEventInsertForm { ... }
crates/server/tests/e2e.rs:3935                               ReputationEventInsertForm { ... }
crates/server/tests/e2e.rs:5345                               ReputationEventInsertForm { ... }
crates/server/tests/e2e.rs:7986                               ReputationEventInsertForm { ... }
```

**Total: 8 callsites across 3 files.** Within L5's narrow-mechanical cap (N≤10, files≤5). No bundled work — pure fix-impl.

**Pre-flight verification (task-0):**

```bash
rg -n "ReputationEventInsertForm \{" crates/api/api_crud/src/governance/create_endorsement.rs \
                                      crates/tools/seed_founders/src/main.rs \
                                      crates/server/tests/e2e.rs
# Expected: exactly 8 hits at the line numbers above (±1-2 line drift OK if rebases shifted)
```

If `rg` returns ≠8 hits or different line counts per file, **STOP and file a DQ blocker** — do not improvise. Drift means another commit landed between this brief and the worker branch cut.

### §2.2 Specific fix — pattern identical at all 8 sites

The anchor is the line `expires_at: None,` inside each `ReputationEventInsertForm { ... }` block. After `expires_at: None,` (and before the closing `}` or `})`), append exactly two lines:

```rust
      dedupe_key: None,
      source_event_type: None,
```

Indentation must match the existing field indentation in each block (some sites use 6-space indent inside `let form = ReputationEventInsertForm { ... };`, others use 6-space inside `.values(ReputationEventInsertForm { ... })`). **Mirror the existing field indent at each site verbatim.** Do not normalize indentation across sites.

This is the same pattern fix-impl-1 applied at commit `d52f62124` to `sponsor_liability.rs:296` and `submit_jury_vote.rs:935`. Read that commit for reference shape:

```bash
git show d52f62124 -- crates/api/api/src/governance/sponsor_liability.rs | head -25
```

**Only 16 added lines total across 3 files** (2 lines × 8 sites). No deletes, no reorders, no comment changes, no field renames.

### §2.3 Validation

After editing, before commit:

```bash
# Verify exactly 10 sites carry the new fields (8 from this fix + 2 from fix-impl-1)
rg -nC1 "dedupe_key: None" crates/
# Expected: 10 hits (sponsor_liability + submit_jury_vote + create_endorsement × 2 + seed_founders + e2e × 5)

rg -nC1 "source_event_type: None" crates/
# Expected: 10 hits at the same locations
```

Then push the worker branch. The push triggers `cargo-validate-workspace.yml` (path filter `crates/**` matches).

**Raise a NEW `kind: "validate-pending"` DQ entry** with the fresh `workflow_run_id` from:

```bash
gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId
```

## §3 Required reading

**Mandatory per file-class table (`.claude/rules/advisor-orchestrator.md` §2.4):**

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — e2e.rs edit; existing fixtures use `Result<(), Box<dyn Error>>` outer. **5 of 8 sites are inside e2e.rs.** The pad does NOT change any helper signatures and does NOT change `?` propagation — pure field addition inside struct literals. No `.map_err` bridges needed. But re-read so you recognize you're NOT in Case A/B/C territory.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixtures pattern context. Pad must NOT touch the surrounding `let pool = ...` / `let mut conn = pool.get_conn().await?` plumbing.
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **≥2 e2e edits (5 sites in e2e.rs). MANDATORY anchor-based Edit, never full-file Read.** Use 5 separate `Edit` calls, one per e2e site, with sufficient `old_string` context (the `expires_at: None,` line plus 1-2 surrounding fields) to make each match unique. Do NOT `Read` the full 12,000-line e2e.rs file.

**Companion lessons:**

- `.claude/lessons/feedback_insertform_default_propagation.md` — why callers may pass `None` and rely on the NOT-NULL DB DEFAULT (substrate-only sub-phase; r2/r3 will start populating).
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — the discipline this brief operationalizes; L5 from v1-RT-r1 halt retro.
- `.claude/rules/decision-queue.md` Recipe 1 — DQ entry shape for `kind: "validate-pending"`.
- `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier — Shape-G two-phase validation flow + callsite-enumeration discipline.

**Reference reading (commit shape):**

- Commit `d52f62124` (fix-impl-1) — `git show d52f62124 -- crates/api/api/src/governance/sponsor_liability.rs` shows the exact two-line pad pattern.
- `crates/db_schema/src/source/governance/reputation_event.rs:33` — confirms `ReputationEventInsertForm` struct fields (Task 7 ship: `dedupe_key: Option<String>`, `source_event_type: Option<ReputationEventSourceType>`).

## §3a Handover from prior task

- **Parent task chain:** Cohort B-bundled tasks 6+7 (Junior #225, commits `03f6c670e` + `05cf5ae1d`) added the two new fields to `ReputationEventInsertForm`. fix-impl-1 (Junior #226, commit `d52f62124`) padded 2 of 10 callsites in `crates/api/api/src/governance/`. fix-impl-2 finishes the remaining 8 sites.
- **Why this is fix-impl-2, not fix-impl-1 retry:** fix-impl-1's brief §4 explicitly scope-bound the edit to `crates/api/api/src/governance/`. The 8 remaining sites are in `crates/api/api_crud/`, `crates/tools/`, `crates/server/tests/`. fix-impl-1 Junior (#226) correctly surfaced this as DQ #205 blocker rather than expanding scope unilaterally. This brief answers DQ #205 option-a.
- **DQ #205:** blocker, unanswered. Advisor will mutate to `answered_by: "advisor"`, `answer: "option-a — fix-impl-2 dispatched with full 8-site enumeration"`, after this brief is committed but **before** the Junior task is created (per L3 atomic raise-before-dispatch discipline, applied here for blocker resolution).
- **DQ #206:** validate-pending for fix-impl-1's workflow `25698028473`. Returned `failure` per `gh run view`. Advisor will mutate to `result: "fail"` with log slice as part of the pre-dispatch DQ housekeeping. Bookkeeping only — failure already understood (DQ #205 enumerated the missing sites).
- **Branch strategy:** standard Junior daemon flow. The daemon cuts a fresh worker branch off `phase-v1-RT-r1` tip `37a62f9b4` (or current tip if advisor's DQ #205+#206 housekeeping commits have landed; verify at task-0 with `git log -3 --oneline phase-v1-RT-r1`). Confirm at task-0:

  ```bash
  git rev-parse HEAD              # daemon-cut worker branch off phase-v1-RT-r1 tip
  git log -1 --format=%s          # most recent ancestor commit on phase
  git rev-parse phase-v1-RT-r1    # phase branch tip at task-creation time
  ```

  If the worker branch's merge-base is NOT `phase-v1-RT-r1` tip (i.e. somehow forked off an older commit), STOP and file DQ blocker.

## §4 Constraints

- **Files:** ONLY these 3 — `crates/api/api_crud/src/governance/create_endorsement.rs`, `crates/tools/seed_founders/src/main.rs`, `crates/server/tests/e2e.rs`. NO other files.
- **Edits:** exactly 16 lines added (2 per site × 8 sites). NO deletes, NO reorders, NO comment changes, NO field renames, NO indentation normalization across sites.
- **e2e.rs edit discipline:** 5 separate `Edit` calls with unique `old_string` anchors per site. NEVER `Read` the full e2e.rs file. NEVER use `replace_all`. Per `feedback_junior_worker_e2e_edit_hang.md`: worker hangs on full-file Edit operations over 8k lines.
- **Branch:** standard Junior daemon flow off `phase-v1-RT-r1` tip. Do NOT manually checkout any other branch.
- **Shape G:** after committing, push the worker branch; capture the resulting `cargo-validate-workspace` workflow_run_id; write a `kind: "validate-pending"` DQ entry per `decision-queue.md` Recipe 1.
- **DQ atomic raise:** when writing the validate-pending entry, the sequence is **(1) git add .claude/decision-queue.json && git commit && git push origin <branch>, (2) end task** — the ci-watcher is dispatched by the advisor AFTER seeing the DQ entry on origin, per L3 (`feedback_dq_raise_before_ci_watcher_queue.md`). Do not skip the push.
- **Encoding:** write the DQ entry with Python `json.dump(..., ensure_ascii=False, indent=2)` to match phase-v1-RT-r1's encoding convention. Do NOT let Python default to `ensure_ascii=True` — that produces `\u` escapes and forces an advisor normalize commit (per c-2 retro lesson + commit `c689da153`).
- **next_id calculation:** compute `next_id` across both `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json` files. Per decision-queue.md "Archive policy" + Hard refusal #2.
- **DQ mid-task push:** if you hit a DQ blocker (e.g. `rg` shows ≠8 sites; struct shape changed in a surprising way; an e2e site's `expires_at: None,` anchor isn't present), commit + push the DQ entry immediately per `decision-queue.md` "Mid-task visibility".
- **COMMIT MESSAGE:** `fix(v1-RT-r1): pad 8 remaining ReputationEventInsertForm literals across create_endorsement + seed_founders + e2e (fix-impl-2)`

## §5 Out of scope

- Tasks 8/9/10 (Cohort B-final) — separate brief at `.claude/PRPs/briefs/rt-r1-impl-8-9-10-bundle.md`, dispatched AFTER fix-impl-2 ci-watcher mutates DQ to `result: "pass"`.
- Any cargo-validate-migration consideration — fix-impl-2 changes no migration files.
- Adding non-`None` values for `dedupe_key` or `source_event_type` per call site — both stay `None` (substrate-only sub-phase; r2/r3 will start populating per PRD §5.3).
- Editing `ReputationEventInsertForm` struct itself — Task 7's struct definition is correct as shipped on this branch.
- Adding builder helpers to elide the `None`s — plan §10.7 did not introduce a builder; struct-literal pattern stays.
- Any work outside the 3 listed files — if `rg` shows additional call sites (e.g. macro expansions, new fixtures merged since this brief was authored), file a DQ pending entry and STOP. Do not expand scope unilaterally — that's the bug fix-impl-1 surfaced via DQ #205.
- Cohort A re-validation — Cohort A is shipped; DQ #194 + #203 + #204 are historical-fail and stay in pending[] per option-2 failure semantics.
