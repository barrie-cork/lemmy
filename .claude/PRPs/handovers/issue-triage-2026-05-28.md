# Handover: Open-issue triage execution plan (2026-05-28)

**Author:** advisor session (canonical brehon-fork checkout, on `governance-v0`)
**Audience:** future advisor session that will execute this plan
**Status:** PLAN — nothing dispatched, nothing committed beyond the roadmap entry + 7 issue comments + 2 issue closes
**Trunk tip at authoring:** `5d086e79b chore(roadmap): add v1-quality-r2 (PR #155 carry-forward bundle, gated on RT-r3)`

---

## RESUME (read this first)

You are picking up an open-issue triage that completed the analysis phase in a prior advisor session and saved the execution plan here. Your job is to author briefs + dispatch Junior, NOT redo the triage.

**Concurrent state to verify before any tool call:**

```bash
git worktree list                           # other active lanes?
git log -1 --format='%H %s' origin/governance-v0   # has trunk moved since 5d086e79b?
git log -1 --format='%H %s' origin/phase-v1-RT-r3  # has RT-r3 shipped yet?
gh issue list --repo barrie-cork/lemmy --state open --json number,title | jq 'length'
```

If RT-r3 has merged to `governance-v0` since this handover (2026-05-28), the v1-quality-r2 bundle becomes available too — see §"Post-RT-r3 unlock" below.

---

## What was decided this session

| Issue | Disposition | Why |
|---|---|---|
| #142 | **Closed-as-completed 2026-05-28** | Schema-v3 work shipped via v1-dq-schema-r1; commits `92e0ed425` + `1803b8546` + `c858aa7ab`. See close comment. |
| #143 | **Closed-as-duplicate-of-#142 2026-05-28** | CR-filed parallel issue. |
| #134 | **Left open as tracking pointer** | Daemon defect; fix lives in `MCPs/junior-mcp` source, not this repo. |
| #156, #157, #158, #159, #160 | **Bundled into v1-quality-r2; gated on RT-r3** | Roadmap entry added `5d086e79b`. Tracking comment posted on each. v1-quality-r2 is unstarted in `.claude/PRPs/v1-roadmap.json` → `lanes.quality.sub_phases.v1-quality-r2`. |

The remaining 22 open issues were triaged by RT-r3-collision check. Those identified as RT-r3-independent and valuable are scoped below.

---

## Execution plan — 3 code lanes + 2 docs one-liners

User authorized 2026-05-28: **author all 3 briefs**, drive **#58 + #96 in parallel (Mode B both)**, defer e2e bundle. Plus the two trivial docs one-liners (#50, #63) direct to trunk.

### Lane A — `phase-v1-redaction-r1` (Issue #58, P0 GDPR)

**Issue:** https://github.com/barrie-cork/lemmy/issues/58 — "v0-polish: harden redaction regex (GDPR-critical)"

**Scope:** focused hardening pass on `crates/api/api/src/governance/redaction.rs` (139 lines, 3 regexes + `scrub_json` recursion). Per `IMPLEMENTATION-PLAN-v0.md` §7.1: "leak of a username defeats right-to-delete permanently." P0 ship-blocker.

**Key file anchors:**
- `crates/api/api/src/governance/redaction.rs` (whole file)
- Tests for redaction: `rg "redaction|scrub_json|scrub_username" crates/server/tests/e2e.rs` — re-enumerate at brief-author time
- ADR-015 (GDPR pseudonym discipline) — required reading

**Lane mode:** Mode B (mobile remote-control). Brief authored on `governance-v0` in canonical session, trunk→phase sync via daemon SSH per `multi-lane-worktree.md` §"Brief location and trunk→phase sync".

**Expected shape:** planning task → 1-3 impl tasks (regex pass + e2e tests + adversarial probe) → bm-pr → CR triage → bm-merge. Complexity 4-5/10.

**RT-r3 collision:** ✅ none. `git diff governance-v0..origin/phase-v1-RT-r3 -- crates/api/api/src/governance/redaction.rs` is empty (verified 2026-05-28).

**Watchpoints for the planner:**
- ADR-015 says `actor_pseudonym` is mandatory; redaction's role is to scrub IDENTIFIERS from log payloads. The regex MUST not over-scrub (e.g. scrubbing `community_id` integers breaks rollups) — preserve schema-typed fields, scrub only string identifiers.
- Adversarial test cases: username containing regex metacharacters, Unicode confusables, embedded newlines, mid-word usernames inside long-form `reasoning` text.
- Test the recursion termination on deeply-nested JSON.

**Brief target path:** `.claude/PRPs/briefs/v1-redaction-r1-planning-1.md`

---

### Lane B — `phase-v1-jm-b-followups-r1` (Issue #96, PR #95 7 follow-ups)

**Issue:** https://github.com/barrie-cork/lemmy/issues/96 — "carry-forward from PR #95: 7 follow-ups (3 Major, 2 Low, 2 Nit)"

**Scope:** 7 CR carry-forward findings from PR #95 (Phase v1-JM-b — jury-mechanics handler), merged 2026-04-25, ~5 weeks of drift.

**The 7 findings (per issue body):**

| ID | Severity | File | Summary |
|---|---|---|---|
| cr-5 | Major | `crates/api/api/src/governance/config.rs:439` | Cascade resolver uses bare-namespace const fallback; ignores candidate-specific const defaults. Fix: walk candidates most-specific-to-least, call `const_default_int(candidate)`/`const_default_float(candidate)` for each, return first `Some`. |
| cr-3/cr-4 | (Major, addressed inline in #95) | — | already shipped |
| cr-8 | Major | `crates/api/api/src/governance/decline_jury_assignment.rs:164-186` | Store replacement pick's `ConstraintRecord` on `JuryAssignmentInsertForm` (currently dropped to `None`). Mirror cr-4 fix shape (capture `ConstraintRecord` and pass via `Some(record.to_json())`). |
| cr-1 | Low | `.claude/PRPs/reports/v1-JM-b-retro.md:100` | Markdownlint fence-tag |
| cr-2 | Nit | `crates/api/api/src/governance/admin_assign_jury.rs:714-718` | Remove no-op `let _ = current_geo_enabled;` binding |
| cr-6 | Nit | `crates/server/tests/e2e.rs:6961-7053` | Pull `bootstrap`, `seed_user`, `seed_community`, `seed_jurors` + `SIGNING_SEED_HEX` into existing `governance_fixtures` module |
| cr-7 | Low | `crates/server/tests/e2e.rs:7060-7089` | `seed_case` can produce inconsistent severity vs severity_tier |

**Lane mode:** Mode B.

**Expected shape:** planning task → 2-4 impl tasks (cluster 1: config.rs + decline_jury_assignment.rs; cluster 2: e2e fixture relocate; cluster 3: nits) → bm-pr → CR triage → bm-merge. Complexity 5/10.

**RT-r3 collision:** ✅ none.
- `git diff governance-v0..origin/phase-v1-RT-r3 -- crates/api/api/src/governance/admin_assign_jury.rs crates/api/api/src/governance/decline_jury_assignment.rs crates/api/api/src/governance/config.rs` returned empty (verified 2026-05-28).
- e2e fixture relocation (cr-6) touches `e2e.rs` lines 6961-7053. RT-r3's `v1_rt_r3_fixtures` module is at lines ~17170+. Disjoint.

**Watchpoints:**
- cr-5 is a behavior change to a cascade resolver — needs e2e coverage of "candidate-specific const default beats bare-namespace default" scenario.
- cr-8 + cr-4 (already shipped) share the same fix-shape; verify by Read-ing the cr-4 commit on `governance-v0` and mirroring.
- e2e fixture relocation (cr-6) is mechanical but touches the large `e2e.rs` file — per `feedback_junior_worker_e2e_edit_hang.md`, plan as multiple smaller Edits (≤200 lines each), not one big Edit.

**Brief target path:** `.claude/PRPs/briefs/v1-jm-b-followups-r1-planning-1.md`

---

### Lane C — `phase-v1-e2e-stability-r1` (Issues #42 + #43 + #45) — DEFERRED to a later turn

User deferred this lane in the 2026-05-28 session: "Skip #42+#43+#45 for later (e2e.rs is a shared-large-file hazard)". Briefs NOT to be authored in this execution. Issues stay open; revisit after Lane A + Lane B ship.

The triage record is preserved here so the next session doesn't re-do the work:

| Issue | Symptom | e2e.rs section |
|---|---|---|
| #42 | `ineligible_user_cannot_be_picked_for_jury` fails on 2nd invocation + cross-test contamination under `--test-threads=1` | L4401-ish |
| #43 | `phase1_migrations_round_trip` fails: missing revert for federation tables (federation_attestation + remote_sanction_notice) | L397-ish |
| #45 | `sponsor_liability_with_founder_multiplier` `accept_jury_assignment` `NotFound` flake | L3238-3276 |

All three are real reproducible flakes in disjoint sections, but the e2e.rs Junior-worker-edit-hang risk argued for sequencing this lane AFTER Lane A + Lane B clear (so the e2e.rs file isn't being touched concurrently).

---

### Docs one-liners — direct to `governance-v0` (no phase branch)

Per `phase-branch.md` "Direct on governance-v0 (no phase branch, no PR)": meta/docs commits that touch only `.claude/PRPs/plans/` or `docs/` go direct, no PR flow.

**#50 — phase-6 plan MD040 + stale enum names** (https://github.com/barrie-cork/lemmy/issues/50)

Two edits in `.claude/PRPs/plans/phase-6-federation.plan.md`:

1. **MD040:** add `text` or `ascii` language tag to opening fences at lines 97-127 (AFTER ASCII diagram) and 848-877 (second ASCII block). PR #46 commit `297341c8b` fixed §77 but missed these two.
2. **Stale enum names:** lines 480-493 reference `attestation_type_enum`, `sanction_action_enum`, `sanction_scope_enum` with trailing `_enum` suffix in CREATE TABLE snippets. Trim the `_enum` suffix.

Commit subject: `docs(plans): phase-6 federation plan — MD040 fences + drop _enum suffix from type names (#50)`.

**#63 — PRD entry-kind enumeration omission** (https://github.com/barrie-cork/lemmy/issues/63)

One add to `.claude/PRPs/prds/v1-federation-inbound.prd.md` line 943: add `federation_inbound_persist_failed` to the §15.1 entry-kind enumeration. Referenced at line 242 in the error-response table but missing from line 943's master list.

Commit subject: `docs(prd): v1-federation-inbound §15.1 — add federation_inbound_persist_failed entry-kind (#63)`.

**Skipped this session per user 2026-05-28:** Pi scaffolding cleanups (#114, #115, #116, #120). All are `.pi/*` harness work, real but lower priority. User answered "Yes — do #50 + #63 only (PRD/plan one-liners)" — Pi deferred for later.

---

## Order of operations (next session)

**Phase 1 — author all briefs (this advisor session, no Junior dispatch):**

1. Author `.claude/PRPs/briefs/v1-redaction-r1-planning-1.md` (Lane A).
2. Author `.claude/PRPs/briefs/v1-jm-b-followups-r1-planning-1.md` (Lane B).
3. Apply the two docs one-liners (#50, #63) direct to `governance-v0`, commit + push.
4. Commit + push both briefs to `governance-v0`.

**Phase 2 — Mode B parallel dispatch (Junior):**

1. **Lane A first:** dispatch `[role:bm-task]` `bm-cut phase-v1-redaction-r1` from `governance-v0` (no laptop worktree in Mode B; daemon-side phase branch only). Wait for completion.
2. **Lane B second:** dispatch `[role:bm-task]` `bm-cut phase-v1-jm-b-followups-r1` from `governance-v0`. Wait for completion.
3. Verify both phase branches exist on origin via `gh api repos/barrie-cork/lemmy/branches/<branch>`.
4. **Trunk→phase sync for both briefs:** per `multi-lane-worktree.md` §"Brief location and trunk→phase sync" Mode B procedure. SSH to daemon, merge `governance-v0` into each phase branch separately, push.
5. Dispatch planning Junior on Lane A (`base_branch=phase-v1-redaction-r1`).
6. Dispatch planning Junior on Lane B (`base_branch=phase-v1-jm-b-followups-r1`).
7. Poll per `/auto-phase` discipline.

**Phase 3 — gate cadence (per advisor-orchestrator.md §3.2):**

User gates 1 (plan approval), 3 (CR triage), 4 (e2e local-vs-dispatch), 5 (merge confirm), 6 (retro sign-off) fire on EACH lane independently. Two lanes ⇒ ~10 gates to walk through. Don't auto-merge.

**Phase 4 — close issues on merge:**

- Lane A merges → close #58 with reference to the merge commit + PR.
- Lane B merges → close #96 with reference to the merge commit + PR (carry-forward findings cr-5, cr-8 etc shipped).

---

## Parallel-lane safety constraints

User authorized "#58 + #96 in parallel (Mode B both)" 2026-05-28. The constraints under which this is safe:

1. **File disjointness verified.** Lane A = `crates/api/api/src/governance/redaction.rs` only (plus tests). Lane B = `crates/api/api/src/governance/{config,admin_assign_jury,decline_jury_assignment}.rs` + e2e fixture section L6961-7089. Zero file overlap.
2. **Daemon `.git/index.lock`.** Per `feedback_cohort_shared_git_index_contention.md`, cohorts ≥3 on a single daemon `.git/` degrade to serial. Two concurrent lanes (Lane A + Lane B), each with its own planning task → impl cohorts of size 1-2, is below the size-3 threshold. Safe.
3. **Laptop e2e gate serialization.** Shape G is SUSPENDED until 2026-06-01 (per `project_shape_g_suspended_2026_05_16.md`). Cargo + e2e gates run on the laptop. If both lanes hit their e2e gates near-simultaneously, they MUST serialize — one e2e run at a time (~26 min each). Schedule the dispatches to stagger.
4. **RT-r3 in-flight.** RT-r3 Task 4 advisor-side authorship is in progress (per `workflow_state_v1_RT_r3.md` 2026-05-26 handover). If a third lane lands during RT-r3 ship, the daemon may see contention. Verify RT-r3 state before dispatching Lane B.
5. **PMD canonical path.** Both lanes (Mode B) drive from this canonical brehon-fork session; PMD writes land in the canonical `.project-memory/memory.db` automatically. No cross-lane stranding risk.

---

## What does NOT happen in the next session

- **No code edits from this advisor session.** Advisor is meta-oversight only.
- **No bm-cut from a different CWD.** Mode B = canonical `brehon-fork`, on `governance-v0`. If you find yourself in `brehon-fork-rt-r3` or another worktree, switch back before dispatching.
- **No phase-branch DQ writes from canonical.** Phase-branch DQ entries are daemon-worker-written. Canonical writes plan-time DQ (clarify entries, planning-time blockers) only — and even those require the atomic read-mutate-commit protocol per `multi-lane-worktree.md` Hard refusal #6.
- **No conformance-audit on these lanes.** Per `.claude/skills/brehon-conformance-audit/`, the audit triggers on `crates/apub/activities/src/governance/**.rs` or `crates/api/api/src/governance/**.rs` or `crates/db_schema/src/source/governance/**.rs`. Lane A's `redaction.rs` IS in `crates/api/api/src/governance/` — invoke the skill at brief-author time per advisor-orchestrator.md §3.1.1.
- **No premature #114/#115/#116/#120 (Pi).** Defer until you've read each issue body in full.

---

## Post-RT-r3 unlock (future session, not this one)

When RT-r3 merges to `governance-v0`, the v1-quality-r2 bundle (#156-#160) becomes authorable. The roadmap entry is already in place (`.claude/PRPs/v1-roadmap.json` → `lanes.quality.sub_phases.v1-quality-r2`). At that point:

1. Re-read all 5 issues (#156, #157, #158, #159, #160) for symbol-existence verification.
2. Author `.claude/PRPs/briefs/v1-quality-r2-planning-1.md` per the v1-deps-r2 pattern (status-deferred precedent).
3. Cut `phase-v1-quality-r2` via Mode B.

**Do NOT do this until RT-r3 ships.** The 4 of 5 issues reference symbols that don't exist on trunk yet.

---

## Cross-references

- `.claude/rules/advisor-orchestrator.md` §1 (polling), §2 (brief authoring), §3 (gates), §4 (cohort dispatch), §5 (validation)
- `.claude/rules/branch-manager.md` §"Phase-branch discipline"
- `.claude/rules/phase-branch.md` (PR flow vs direct-on-trunk policy)
- `.claude/rules/multi-lane-worktree.md` §"Lane modes" (Mode A vs Mode B), §"Brief location and trunk→phase sync" (Mode B trunk-sync procedure)
- `.claude/rules/decision-queue.md` v3 schema + composite-id discipline
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` (e2e.rs Edit safety)
- `.claude/lessons/feedback_cohort_shared_git_index_contention.md` (daemon `.git/index.lock`)
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md` (lane-worktree bootstrap; Mode A only)
- `.claude/PRPs/briefs/v1-deps-r2-planning-1.md` (canonical deferred-brief precedent — what a complete planning brief looks like)
- `.claude/PRPs/v1-roadmap.json` (lane status truth)
- `workflow_state_v1_RT_r3.md` (RT-r3 current state)

---

## Audit trail this session (2026-05-28)

| Action | Result |
|---|---|
| Read 8 recent open issues (#134, #142, #143, #156-#160) | Triaged |
| Closed #142 with v1-dq-schema-r1 ship reference | https://github.com/barrie-cork/lemmy/issues/142#issuecomment-4565945540 |
| Closed #143 as duplicate-of-#142 | https://github.com/barrie-cork/lemmy/issues/143#issuecomment-4565946660 |
| Added v1-quality-r2 to roadmap | Commit `5d086e79b` on `origin/governance-v0` |
| Posted tracking comments on #156-#160 | 5× `issuecomment-456626*` |
| Triaged 22 remaining open issues against RT-r3 collision | This document |
| Authored execution plan | This file (`.claude/PRPs/handovers/issue-triage-2026-05-28.md`) |

Zero Junior tasks dispatched this session. Zero brief files authored this session (this handover is the durable record + execution-plan; the briefs are authored in the next session).
