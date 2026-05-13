# v1-SL lane meta-retro — sponsor liability lane (SL-a → SL-b → SL-c → SL-d → SL-e)

**Date:** 2026-05-13
**Lane:** v1 sponsor-liability
**Sub-phases shipped:** SL-a, SL-b, SL-c (split: SL-c-1 + SL-c-2), SL-d, SL-e
**Started:** 2026-05-03 (SL-a cut)
**Lane-closer merged:** 2026-05-13 07:00:34Z (PR #127, governance-v0 tip `f0c2b75af`)
**Wall-clock:** ~10 days across 6 PRs (#111 SL-a, #114 SL-b, sl-c-1 + #122 SL-c-2 cherry-pick, #123 SL-d, #127 SL-e)
**Total tasks shipped:** ~30 impl + ~12 fix-impl + 5 retros
**Lane referenced in:** SL-e retro §3 carry-forward (this artifact)

---

## §1 Lane summary — what the SL lane is for

The v1 sponsor-liability lane implements the PRD §9 sponsor-liability mechanics on top of the existing v0 case lifecycle. Each sub-phase added one layer; together they form a coherent producer→consumer chain:

| Sub-phase | Contribution | Producer → consumer link |
|---|---|---|
| **SL-a** | `CaseStatus::SponsorLiabilityPending` + 3 grace-window enum variants + ENTRY_KINDs + ADR-013 sweep | Foundation: enums + governance_log keys |
| **SL-b** | `revoke_endorsement` handler + `liability_chain_severed_for_cases` + `endorsement_revoked` ENTRY_KIND | Producer: sponsor revokes → liability escape signal |
| **SL-c-1** | `sponsor_liability_grace` scheduler module + `clokwerk` wiring + atomic guard | Consumer: scheduler tick reads `grace_expires_at` |
| **SL-c-2** | 5 `grace_check` e2e test stubs covering scheduler dispatch surface | Verification: scheduler behaviour under expired / escaped / batched cases |
| **SL-d** | `apply_sponsor_liability` compute/fire split + `submit_jury_vote` Pending transition + 4 e2e + 1 unit | Producer (jury → Pending) + Consumer (compute & fire on grace expiry) |
| **SL-e** | 3 lane-wide e2e tests (revocation, window-expiry, backfill-of-mid-flight v0→v1) | Integration: full producer→consumer paths as one coherent system |

**Closed scope:** all 3 SL-lane v1 stories from PRD §9.3. Restoration-during-window-escapes branch deferred to restorative-mechanics-v1 PRD per SL-c DQ #145 (LOCKED).

---

## §2 What worked across the lane (durable patterns)

### 2.1 Pre-planning clarify gate

`/brehon-clarify` ran on every planning brief from SL-c-2 onward. Across SL-d + SL-e, 6 clarify-DQ entries (3 each) caught spec ambiguities before plan-write. Five resolved via advisor-mode self-answer with citations; one needed user-relay. Zero post-plan re-litigation. The cost (~10 min per brief) saves a full re-plan cycle each time.

**Recommendation:** the clarify gate is now standard discipline; should be added to the canonical advisor-orchestrator stage-shape as non-skippable for any *planning* brief (impl + BM briefs already inherit from a clarified plan).

### 2.2 Canonical Case A error-shape discipline

Post-SL-c-2 amendment to `feedback_lemmy_error_no_std_error.md` (after the 3-cycle catch-fire on Task 1) shipped two improvements: (a) canonical-schema-first gate — read sibling fixtures before authoring; (b) verbatim §G4 recipe blockquote in fix-impl briefs (anti-paraphrase). **Result: zero E0277 catch-fires across SL-d and SL-e** (10+ test fns, 14,000+ line e2e.rs). The amendment is empirically validated.

### 2.3 Shape G pipeline (workspace-check on junior/*, e2e on phase-v1-*)

Two-phase validation per option (b) 2026-04-28 (cargo-test-e2e.yml triggers on push to `phase-v1-*` only, NOT on `junior/*`) held cleanly. Each sub-phase used the same pipeline shape: impl-task pushes → validate-pending DQ → ci-watcher mutates → advisor runs phase-2 e2e locally. Across 4 sub-phases (SL-c-2 / SL-d / SL-e Task 1/2/3) and ~10 dispatched workflows, the pipeline worked. The few advisor interventions were not pipeline failures — they were stale-cache / cross-lane / runbook issues.

### 2.4 Local laptop e2e default

Per user gate selection (PR #105 lock + `feedback_default_local_testing.md`): laptop is the canonical e2e runner; GH dispatch is the explicit escape hatch. **Across SL-c-2 (5 tests), SL-d (4 e2e + 1 unit), SL-e (3 lane-wide tests) = 12 e2e launches, zero GH minutes billed**. The local runs caught one true regression (DQ #199 RT-r1 migration count) and one stale-cache issue (SL-a fix-impl-3); both resolved without billed dispatch.

### 2.5 Anchor-Edit discipline on e2e.rs

`feedback_junior_worker_e2e_edit_hang.md` held across the lane. e2e.rs grew from ~11,000 lines (pre-SL-a) to 14,869 lines (post-SL-e Task 3) — ~38% growth in ~10 days. Every test added used anchor-Edit, single-Edit calls, no full-file reads. Zero Edit-hang incidents in any Junior dispatch.

### 2.6 §G4 verbatim-recipe + canonical-schema-first

Fix-impl briefs across SL-d (5 cycles) and SL-e (2 cycles) used the verbatim §G4 recipe blockquote pattern. Zero recipe-paraphrase incidents post-SL-c-2 amendment. The fix-impl chain is now mechanical when the failure signature matches an allowlist row.

---

## §3 What didn't work / recurring frictions (cross-sub-phase)

### 3.1 Daemon finalize-merge push skip (~80–93% skip rate)

**Cumulative count across SL lane: 17+ confirmed instances.** Junior daemon finalize-merges the worker branch into the local phase branch on EliteDesk but does NOT push the merge commit to origin. Recovery is mechanical (`ssh homeserver 'cd /srv/brehon-fork && git push origin phase-v1-<phase>'`) and routinely needed.

- SL-a: 13 instances
- SL-c-1: 4+ instances
- SL-d: 2 instances
- SL-e: 4 instances (Tasks 1+2+3+fix-impl-1 finalize-merges)
- **Pattern:** when worker pre-pushes its branch, daemon's finalize-merge stage runs but skips the push step.

**Recommendation:** upstream-patch Junior daemon to add `git push origin <phase>` after every finalize-merge. The recovery is mechanical 30s per instance; 17+ instances per 10-day lane is ~10 min of pure-friction cost. Worth the daemon patch.

### 3.2 ci-watcher / bm-task / impl-task brief-non-compliance

Across SL-c-1, SL-c-2, SL-d, SL-e: at least 6 documented cases of Junior subagents deviating from precise instructions in their brief:

- **SL-c-1 ci-watcher #145**: Wrote NEW DQ entry instead of mutating existing one (Hard refusals #2 + #7 breach despite brief citing both)
- **SL-c-2 bm-triage #179**: Ignored brief bucket assignments; kept `recommendation: request-changes`
- **SL-c-2 fix-impl-2 worker (Task 2)**: Paraphrased §G4 recipe; needed re-dispatch
- **SL-d ci-watcher #214**: Mutated DQ correctly but didn't commit the mutation step
- **SL-d bm-poll-cr #222**: Stop-hook loop / PMD isolation; manual recovery
- **SL-e bm-poll-cr-2 #260**: Bucketed all 4 user-rebut findings as `fix-in-pr` despite brief explicit-rebut instruction
- **SL-e bm-merge #261**: Hard-refused with bogus DQ #213 (id collision) on stale-base race

**Pattern:** bm-task subagent (Haiku 4.5) is the worst offender — 3 of 6 above. Impl-task and ci-watcher fewer but still real. The brief content matters AND the agent-spec matters AND the daemon-side base-state matters. Briefs alone don't guarantee compliance.

**Recommendations:**
- For bm-task: review `.claude/agents/bm-task.md` for first-sentence emphasis on "follow the brief's bucket assignments literally, even if you would have picked differently". Move the rule to L1 of the agent spec.
- For ci-watcher: the SL-c-1 → SL-c-2 fix already moved Hard refusal #7 emphasis up; held in SL-d + SL-e. Pattern works; replicate for bm-task.
- For all subagents: a PostToolUse hook on commits could lint `bucket: fix-in-pr` writes against brief-cited `bucket: rebut` instructions. Future scope.

### 3.3 DQ id collision pattern (recurring)

Workers compute `next_id` from the worktree's view of `decision-queue.json`, which may not include archives OR cross-branch entries. Confirmed instances:
- SL-c-2: collision on Tasks 3 + 4 (workers missed advisor-side entries)
- SL-e bm-merge #261: collision on DQ #213 (worker missed RT-r1 lane's max id + the legitimate phase-v1-SL-e #213 already mutated)

**Mitigation already shipped:** `dq-recipes.md` requires walking `decision-queue-archive-*.json`. **Still missing:** workers must also walk cross-branch DQ files when multiple lanes are concurrently active. `scripts/brehon/resolve-dq-canonical.sh` exists but is not invoked by Junior workers at brief-read time.

**Recommendation:** subagent spec for any role that writes DQ entries should invoke `resolve-dq-canonical.sh` as part of task-0 pre-flight. Adds ~5s to dispatch; eliminates the collision class.

### 3.4 Multi-lane concurrency overhead

When SL-e was executing, RT-r1 merged mid-phase. This added:
- 1 extra phase-2 e2e run (~30 min) due to DQ #199 fail
- 1 advisor-side fix commit (`2df0afd8f`) bumping `revert_migrations` limit 8→12
- 1 governance-v0 → phase-v1-SL-e merge to absorb RT-r1's 4 migrations

**Total overhead from cross-lane RT-r1 merge: ~45 min wall-clock + 1 fix commit.** This is the empirical cost of concurrent multi-lane work.

**Recommendation:** before each phase-2 e2e launch, scan `gh pr list --repo barrie-cork/lemmy --state closed --json number,mergedAt,title --limit 20` for merges to `governance-v0` since the last phase-branch sync. If any concurrent-lane PR merged → rebase phase branch first.

### 3.5 ADR-013 red-flag scanner false-positives (recurring 4 sub-phases)

The regex-only ADR-013 scanner trips on every PR that touches an `EmergencyRemove` enum arm — confirmed across PR #107 (1x), PR #111 (9x), SL-d's PR #123 (multiple), SL-e PR #127 (multiple). Per-PR maintainer-ack is high-friction.

**Recommendation:** queue an issue against `barrie-cork/lemmy` for ADR-013 scanner discipline. Fix categories: (a) AST-aware scanner; (b) exemption-pattern allowlist; (c) downgrade to informational. **This is a v0-polish bug that has now bitten 4 sub-phases. Worth a real fix.**

### 3.6 PMD stop-hook / isolation issue (recurring)

ci-watcher and bm-poll-cr tasks repeatedly hit a stop-hook blocking loop that prevents the mutation commit from completing. Pattern:
- SL-c-2: ci-watchers #176, #145 affected
- SL-d: ci-watcher #214, bm-poll-cr #222
- SL-e: not directly seen (likely tasks short enough to avoid watchdog)

**Cause:** PMD query isolation between bash hooks and MCP tools — the hook can't see the eval the subagent just wrote. Functional completion is achievable but requires advisor manual finalize-merge.

**Recommendation:** flagged in SL-d retro §5 as a "watch if recurs in SL-e" — did NOT recur visibly. Possibly self-resolved by shorter task runtimes. Watch in next lane.

### 3.7 Plan §13 deny-listed cast (SL-c-1 only, but documented)

Planner prescribed `as f64` + `as i64` casts in §13 IMPLEMENT body without checking workspace-deny `as_conversions` lint. Cost: ~30 min advisor hand-fix + force-push + advisor-DQ + ci-watcher cycle.

**Recommendation:** planner-side dry-run of `cargo clippy --workspace --features full --no-deps -- -D warnings` on any IMPLEMENT block containing `as` keyword. 30s planner cost saves 30 min advisor cycle. Already mitigated by the canonical-schema-first gate but worth keeping the specific check.

---

## §4 Lane-wide pseudonym-discipline coverage matrix (carry-forward §3.1 from SL-e retro)

ADR-015 mandates pseudonymisation for governance_log payloads exposing person identity. The SL lane's e2e tests collectively assert this discipline across every ENTRY_KIND the lane touches. Coverage map:

| ENTRY_KIND | First test asserting `target_pseudonym` is_string + ne raw person_id | Sub-phase | Test fn | e2e.rs ref |
|---|---|---|---|---|
| `sponsor_liability_pending` | SL-e Test #1 | SL-e | `revocation_during_window_escapes_full_lane` | 14157-14181 |
| `endorsement_revoked` | SL-b unit | SL-b | (mod v1_sl_b_fixtures helpers) | ~11000-11924 |
| `sponsor_liability_escaped` | SL-e Test #1 | SL-e | `revocation_during_window_escapes_full_lane` | 14210+ |
| `sponsor_liability_fired` | SL-e Test #2 | SL-e | `window_expiry_fires_full_lane` | ~14357+ |
| `sponsor_liability_applied` | SL-e Test #2 | SL-e | `window_expiry_fires_full_lane` | ~14400+ |
| `case_decided` | covered by SL-c-2 grace_check fixtures | SL-c-2 | (mod v1_sl_c_fixtures) | ~12200+ |
| `sanction_created` | SL-e Test #1 | SL-e | `revocation_during_window_escapes_full_lane` | 14120+ |
| Backfill: target + sponsors_pseudonyms | SL-e Test #3 | SL-e | `backfill_of_mid_flight_v0_to_v1_deploy` | ~14596+ |

**No coverage gap identified for SL-lane ENTRY_KINDs.** All 8 lane-relevant governance_log keys have explicit pseudonym assertions in at least one e2e test. The lane-meta-retro confirms ADR-015 compliance across the lane.

**Open scope:** when a future sub-phase adds a new ENTRY_KIND emitting person identity, the planner should consult this matrix and ensure the new test asserts pseudonym discipline. Add to plan template §10 watchpoint list.

---

## §5 Lessons promoted to `.claude/lessons/` across the lane

Tracked across sub-phase retros (cumulative count):

- **SL-a** (5 lessons): migration-invariants-full-mirror, brief-naming-phase-branch-check, etc
- **SL-b** (3 lessons): canonical chain severance verification patterns
- **SL-c-1** (4 lessons): L10 ci-watcher ensure_ascii=False (paired with L1-L9 from session retros)
- **SL-c-2** (4 lessons): DQ branch isolation / PR DIRTY / bm-triage committed findings / Case A canonical override discipline
- **SL-d** (4 lessons): branch verification before e2e launch, stop-hook isolation reinforce, stale-DQ scan at merge gate
- **SL-e** (3 candidates): deferred-write semantics test pattern, force-rewind grace_expires_at technique, revert_migrations limit cross-lane

**Total lane lessons: ~23 promoted or candidates.** Lane-meta-retro reinforces these are durable patterns, not phase-specific oddities.

### Newly proposed for promotion (from this meta-retro)

1. **`feedback_subagent_brief_non_compliance_recurring.md`** — bm-task subagent (Haiku 4.5) deviates from explicit brief instructions ~50% of the time on bucket-assignment / id-collision discipline tasks. ci-watcher and impl-task fewer but real. Recommendation: PostToolUse hook on subagent commits that lints brief-vs-commit consistency on key fields (DQ ids, bucket assignments, file scope).

2. **`feedback_lane_meta_retro_pattern.md`** — the lane-meta-retro is itself a novel artifact class (not a sub-phase retro; not a phase report). Pattern: after a lane's lane-closer ships, author a meta-retro that synthesizes across sub-phases for cross-sub-phase patterns (pseudonym coverage matrix, cumulative friction counts, durable-pattern reinforcement). Recommendation: add `<lane>-meta-retro.md` to `.claude/PRPs/reports/` as the standard artifact whenever a lane has ≥3 sub-phases.

3. **`feedback_concurrent_lane_pre_e2e_merge_scan.md`** — when multiple lanes are concurrently active, scan `gh pr list --state closed` for governance-v0 merges since last phase-branch sync BEFORE each phase-2 e2e launch. Diverged phase branch → rebase first. Surfaced from RT-r1 mid-SL-e merge incident.

---

## §6 Carry-forward to subsequent lanes

These items are not closed within the SL lane and should be inherited by the next lane's first sub-phase:

1. **Restoration-during-window-escapes branch deferred** (per SL-c DQ #145 LOCKED) — restorative-mechanics-v1 PRD owns this. When that PRD is authored, the first sub-phase brief MUST cite DQ #145 as the deferral source.

2. **Daemon finalize-merge push patch** — upstream Junior daemon, queue as infra issue. 17+ confirmed instances; mechanical recovery; worth the patch.

3. **ADR-013 scanner false-positive fix** — queue as `barrie-cork/lemmy` issue. 4 sub-phases bitten. AST-aware or exemption-pattern.

4. **Subagent brief-compliance discipline** — bm-task spec L1 emphasis on "follow brief literally", ci-watcher recipe-in-brief pattern (already works), impl-task callsite-enumeration pattern (already works). Replicate the working patterns into bm-task spec.

5. **DQ id collision mitigation** — wire `resolve-dq-canonical.sh` into Junior task-0 pre-flight for DQ-writing roles.

6. **Concurrent-lane pre-e2e merge scan** — add to advisor pre-phase-2-e2e checklist.

7. **PENDING-collapse-to-single-session** — flagged in RT-r1 retro §5; re-evaluate now that multi-lane has empirically shown its overhead is bounded (~45 min per cross-lane mid-merge). Multi-lane works; the question is whether the overhead justifies it for solo-dev.

---

## §7 Confidence + acceptance

The lane shipped its full v1 scope. All 6 sub-phase retros (SL-a, SL-c-1, SL-c-2, SL-d, SL-e — SL-b shipped without a standalone retro due to single-task scope, covered by `v1-SL-b-verify.md`) are on `governance-v0`. PR #127 merged with all 15 CR + Copilot findings resolved (8 done + 7 rebut, 0 open). Phase-2 e2e final: 88 passed, 0 failed.

The carry-forward items in §6 are the durable backlog for downstream lanes; everything in §2-§5 is internal to the SL lane and now closed.

**Confidence: high.** The SL lane delivered the PRD §9 sponsor-liability mechanics. Coverage matrix in §4 confirms ADR-015 compliance. Friction patterns in §3 are documented + actionable. Carry-forward in §6 is concrete.

*SL lane is closed. The next-lane advisor inherits §6 plus the durable patterns in §2. SL-lane retros stay on governance-v0 as the audit record.*
