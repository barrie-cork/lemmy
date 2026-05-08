# v1-SL-c-1 retro — Sponsor Liability sub-phase C-1 (sponsor_liability_grace scheduler module + clokwerk wiring)

**Sub-phase:** v1-SL-c-1
**Branch:** `phase-v1-SL-c-1` (cut from `governance-v0` @ `c93cf7e90` at `477f0c55c`)
**Tip at retro:** `3d13b6394` (post-Task-2 ci-watcher #147 mutation merge)
**Plan:** `.claude/PRPs/plans/v1-sponsor-liability-c.plan.md` (sub-phase c is split c-1 + c-2; this retro covers c-1)
**Tasks shipped:** Task 1 (sponsor_liability_grace module + 4 pub fns) + fix-impl-1 (PersonId + DbConn) + advisor-laptop hand-fix (clippy::as_conversions) + Task 2 (scheduler block + atomic guard) + Task 3 (this retro)
**PR:** not yet opened (sl-c-2 ships after c-1; c-1 may merge solo or bundled per user gate)
**Started:** 2026-05-07 21:43 UTC (cut)
**Ended:** 2026-05-08 02:05 UTC (Task 2 ci-watcher #147 mutation pushed)
**Wall-clock:** ~4h 21min cut-to-greengate

---

## TL;DR for the next advisor

**SL-c-1 wires the sponsor_liability_grace scheduler into the clokwerk runtime. Two impl tasks shipped (module + scheduler-block); one Junior fix-impl recovered an E0432 + DbConn type miss; one advisor-laptop hand-fix recovered a workspace-deny `clippy::as_conversions` lint that the planner left as a bare `as` cast in §13 Task 1's IMPLEMENT body. One ci-watcher (#145) violated Hard refusals #2 + #7 — wrote a NEW DQ entry instead of mutating the existing one — recovered via EliteDesk hard-reset + advisor canonical mutation. The replacement ci-watcher (#147) honored the contract cleanly. Green-gate closed at 02:05 UTC.** Highlights:

- **Two impl tasks Junior-shipped clean modulo one fix-impl.** Task 1 (module + 4 pub fns) needed fix-impl-1 for PersonId import path + DbConn type — both citations in the brief but Junior chose plausible-but-wrong defaults. Task 2 (scheduler block + atomic guard pair) shipped first-try clean.
- **Plan §13 Task 1 prescribed `as f64` + `as i64` casts in `check_grace_staleness` body.** Per `Cargo.toml:109`, `as_conversions = "deny"` workspace-wide. The lint is **not** in the §G4 allowlist (which contains doc_lazy_continuation / E0432 / deprecated-API). User authorised file-ownership boundary override 2026-05-07 23:25 UTC: advisor-laptop added a 6-line `#[expect(clippy::as_conversions, clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = ...)]` block matching the canonical fork pattern at `admin_assign_jury.rs:1049-1053`. Cost: ~30 min (audit + edit + force-push to junior trigger branch + new validate-pending DQ from advisor + ci-watcher cycle).
- **ci-watcher #145 violated Hard refusals #2 + #7.** Wrote a NEW DQ #162 (id collision with the canonical #162 advisor entry) and self-mutated it; preserved `from: ci-watcher` instead of `from: impl`/`advisor`; treated the entry's `kind: "validate-pending"` write as if it were within scope. Junior daemon then merged the bad commits onto EliteDesk's local `phase-v1-SL-c-1` (didn't push). Recovery: user-authorised hard-reset of EliteDesk + advisor-canonical mutation of the real #162 + push. Cost: ~25 min. Replacement ci-watcher #147 (this session) honored the contract cleanly.
- **Daemon-finalize-merge skip pattern hit again on 2/3 ci-watcher merges + 2/2 impl merges.** 4-of-5 finalize-merge events did NOT push the merge commit to origin — only the daemon-merge-on-EliteDesk commit was visible via `ssh homeserver`. Manual `ssh homeserver 'cd /srv/brehon-fork && git push origin phase-v1-SL-c-1'` was the recovery. Pattern is `feedback_junior_daemon_finalize_skips_when_worker_pre_pushes` empirically systemic at >80% skip rate — now 8+ confirmed instances cumulatively across SL-a + SL-c-1.
- **Auto mode functioned cleanly under user-typed Git Destructive grants.** AskUserQuestion answers don't satisfy the `claude_no_destructive_default` hook policy; force-push + `reset --hard` both required typed transcript-text grants. User typed all required confirmations; recovery from #145's breach was clean.

DQ count summary (this sub-phase): pending 3 at retro tip (`#156` SL-b carryover + `#160` SL-c-1 Task 1 first-attempt fail-audit + `#161` SL-c-1 fix-impl-1 fail-audit). Both SL-c-1 fail records preserved in `pending[]` per validate-pending-laptop handler step-3 (audit-trail discipline). All operational `validate-pending` entries (`#162` advisor hand-fix + `#163` Task 2) resolved with `result: pass`.

---

## What surprised us

Per `feedback_retro_not_report.md` canonical-header requirement.

- **Plan §13 Task 1 IMPLEMENT body prescribed deny-listed casts.** The plan author transcribed `let threshold_hours_f = (max_grace_hours as f64) * multiplier;` and `let threshold_hours_i = threshold_hours_f.round() as i64;` directly into the IMPLEMENT block without checking that `as_conversions` is workspace-deny per `Cargo.toml:109`. Junior implemented exactly as written; clippy then fired on the next-stage CI run after the E0432 fix cleared. The canonical fork-side pattern at `admin_assign_jury.rs:1049-1053` (`#[expect(clippy::as_conversions, clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "...")]`) was discoverable but the planner missed it. Watch-item: planner-side `cargo clippy --workspace --features full --no-deps -- -D warnings` dry-run on every IMPLEMENT block that contains a numeric cast keyword (§3.1).
- **ci-watcher #145 ignored a ~50-line brief that explicitly cited Hard refusals #2 + #7.** The brief at `sl-c-1-ci-watcher-2.md` (committed `a4e25e9d4`) was byte-correct: it pointed at the deprecated `validate-result | validate-failed` kinds, named PMD #156, said "MUTATE the existing entry, do NOT write a new one." Junior ci-watcher subagent disregarded those constraints, wrote a new #162 with `from: ci-watcher` + `kind: validate-pending`, and self-mutated it. The brief format works for ~95% of dispatches; this one didn't take. Surprise: brief content alone is not a sufficient guarantee — the subagent's behavior is governed by both the brief AND its agent-spec at `.claude/agents/ci-watcher.md`. The replacement ci-watcher-3 brief (committed `8005e5472`) added stronger inline reminders + a canonical Python recipe + an explicit "what success looks like" footer; #147 honored it cleanly. Watch-item: `.claude/agents/ci-watcher.md` review for first-task-line emphasis on Hard refusal #7 (§3.2).
- **8th occurrence of daemon-finalize-merge skip — pattern is empirically systemic.** Junior daemon DOES finalize-merge (the merge commit appears on the EliteDesk's local `phase-v1-SL-c-1`) but does NOT push it to origin. Cumulative count across SL-a (13 instances) + SL-c-1 (4+ instances) is now 17+. Recovery is mechanical (`ssh homeserver 'git push origin phase-v1-SL-c-1'`) but the cumulative process-friction signal is high. Carry-forward: queue infra issue against Junior daemon's finalize-merge stage to add `git push origin <phase>` after the merge commit lands locally.
- **Auto mode handled three distinct Git Destructive operations cleanly.** Force-push to junior trigger branch (post-as_conversions hand-fix), `reset --hard` on EliteDesk's local `phase-v1-SL-c-1` (post-#145 breach recovery), and SSH push of the daemon-merged tip — all required transcript-text grants beyond the AskUserQuestion answer. The pattern works: surface the operation, ask for typed confirm, proceed only on plain-text "yes proceed with X". User-typed grants = audit trail.

## What to change

Per `feedback_retro_not_report.md`. Forward-going changes the next advisor / sub-phase should adopt.

- **Planner must dry-run `clippy --workspace --features full --no-deps -- -D warnings` against any IMPLEMENT block with `as` keyword.** When the plan body contains a numeric cast (`as f64`, `as i64`, `as usize`, etc), the planner should either: (a) replace with `try_from` + error path, OR (b) wrap the entire block in the canonical `#[expect(clippy::as_conversions, ...)]` pattern with a `reason = ...` capturing safety bounds, citing the canonical fork-side instance at `admin_assign_jury.rs:1049-1053`. Cost: 30 sec planner-side; saves ~30 min advisor + Junior cycle when the §G4 hand-fix is otherwise needed.
- **Add `clippy::as_conversions` to the §G4 allowlist as `numeric-cast-needs-expect-block`.** The fix is mechanical when the canonical pattern exists: add the `#[expect(...)]` block above the affected fn or block with safety-bounds reason. Allowlisting collapses the user-gate to a self-resolved fix-impl dispatch the same way migration-no-transaction-line-1 should be after SL-a.
- **Strengthen `.claude/agents/ci-watcher.md` first-line emphasis on Hard refusal #7.** Move the "MUTATE, do not write new" rule to the first sentence of the agent spec (currently in the "Hard refusals" sub-section). One-line fix; eliminates ambiguity at brief-read time.
- **Inline canonical Python recipe in every ci-watcher brief.** The brief template should embed the mutate-in-place Python snippet (8 lines) + an explicit "what success looks like" verification footer. The replacement ci-watcher-3 brief that #147 read cleanly had both; #145's brief had neither. Update `.claude/PRPs/templates/ci-watcher-brief.template.md` to add both.

## What to carry forward

Per `feedback_retro_not_report.md`. Patterns and discipline the next advisor should explicitly inherit.

- **Mid-DQ-mutation detection via diff inspection.** When ci-watcher mutates a paired entry, check `git show <sha> -- .claude/decision-queue.json` for `{` / `}` brace counts before merging. The #145 breach was visible at first inspection (NEW entry append visible as a fresh `+{` block; legitimate mutation appears as a `-` and `+` of the same `id`). Catch-fire moment: anything other than a clean `id: N` removed-and-re-added block is a breach.
- **EliteDesk hard-reset is the recovery mechanism for daemon-merged-but-bad commits.** When a Junior breach lands on EliteDesk's local `phase-v1-SL-c-1` (or any phase branch) before push, `ssh homeserver 'cd /srv/brehon-fork && git checkout phase-X && git fetch origin && git reset --hard origin/phase-X'` is the sole reliable recovery. User-typed grant required; mechanical otherwise.
- **AskUserQuestion answers ≠ Git Destructive grants.** The hook policy explicitly requires plain-text typed confirmation in the user transcript for force-push, reset --hard, and (probably) other destructive ops. Surface the operation in plain text; wait for typed reply; proceed only on "yes proceed with X". Keep this discipline through the SL-c-2 PR cycle.
- **`Lesson candidate L10 — ci-watcher ensure_ascii=False in mutate path`** (to be promoted Task 3 commit). #147's mutation re-serialized with `ensure_ascii=True` (Python's default), expanding em-dashes (`—`) and §-signs to `—` / `§` escapes throughout the file. Functionally equivalent (round-trip identical) but cosmetically noisy in the diff (~450 lines of escape-only edits). Per `feedback_json_dump_ensure_ascii_false`. Worth a one-line addition to the ci-watcher brief template: "When writing the file, use `json.dump(d, f, indent=2, ensure_ascii=False)`."

---

## 1. What worked — keep doing

### 1.1 Two-stage §G4 classifier handled both auto-fix and user-gate cleanly
DQ #160 (Task 1 first-attempt) failed with `error[E0432]: unresolved import` + `error[E0599]: no method named run_transaction` — both clean allowlist matches. Junior fix-impl-1 dispatched; PersonId path + DbConn type fixes landed in 1 commit (`54eb1c380`). The next-stage clippy then fired `as_conversions` (DQ #161); not in allowlist; user-gate escalation; user picked option C (advisor hand-fix). Both stages followed the rule cleanly without ambiguity.

### 1.2 Force-push to junior trigger branch is the §G4 advisor-manual workflow path
Workflow `cargo-validate-workspace.yml` triggers on push to `junior/*` branches only (path-trigger excludes `phase-v1-*`). After the advisor-laptop `#[expect]` hand-fix landed on `phase-v1-SL-c-1`, the workflow couldn't fire from the phase branch directly. Recovery: force-push the new phase-tip to the existing junior fix-impl-1 worker branch; workflow fires from there; ci-watcher polls + mutates. Cost: ~5 min mechanical. Per `project_brehon_v1_sl_a_notes` (where this pattern was first established).

### 1.3 Mutation diff inspection caught #145's breach in <30 sec
After Junior #145 reported done with a successful workflow conclusion, a 30-second `git show 9727aaf59 -- .claude/decision-queue.json | head -80` inspection surfaced the NEW `+{` entry append + the `-{` removal of #163 metadata. The breach was visible structurally; not behaviorally. Carry forward: post-mutation diff inspection on every ci-watcher commit (already implicit but should be checklist-explicit).

### 1.4 ci-watcher-3 brief recovery
The replacement brief (`8005e5472`) added: explicit Hard refusal #7 reminder in the body header, canonical Python mutate-in-place recipe (8 lines), "what success looks like" verification footer. #147 read it and produced a clean mutation (modulo the `ensure_ascii=True` cosmetic issue noted in §3.4). Demonstrates: brief content matters, even when agent-spec is the same.

### 1.5 DQ schema-v2 routing held under all four canonical paths
4 distinct routing flows fired this sub-phase:
- `(validate-pending, pending) → ci-watcher mutate → (validate-pending, resolved)` for #163 [pass]
- `(validate-pending, pending) → advisor-laptop mutate → stays pending` for #161 [fail-audit]
- `(validate-pending, pending) → ci-watcher mutate → stays pending` for #160 [fail-audit, post-fix-impl-resolve]
- `(validate-pending, pending) → advisor mutate → moves to resolved` for #162 [pass via canonical-mutation cleanup post-#145-breach]

All four landed correctly. The schema-v2 routing in `decision-queue.md` did the work even under the breach-recovery edge case.

---

## 2. Per-role signals (four-role)

### 2.1 Advisor signals — one Junior breach recovery handled cleanly under auto mode

Two impl tasks dispatched + one fix-impl + one ci-watcher breach recovery + Task 2 dispatch + retro author. Auto-mode discipline held: every Git Destructive op surfaced + waited for typed grant; AskUserQuestion answers used for non-destructive choices (e.g. "auto-queue Junior fix-impl-2"); transcript-grade typed grants for force-push, reset --hard, and SSH-push.

Notable advisor moments:
- **#145 breach detection + recovery (~25 min):** mutation-diff inspection caught it in 30 sec; surface to user with 4-option AskUserQuestion; user picked hard-reset; SSH reset; advisor authored canonical #162 mutation; pushed. End state: clean canonical history with user-typed grants for both reset-hard and SSH-push.
- **as_conversions hand-fix (~30 min):** §G4 classifier non-match; surface 4 options; user picked C (hand-fix); read canonical pattern at admin_assign_jury.rs:1049-1053; applied 6-line `#[expect]` block; force-push to junior branch; raise advisor #162 (kind: validate-pending); ci-watcher cycle. Watch: this sequence is repeatable and could be allowlisted (§3.1).

8+ daemon-skip recoveries handled mechanically (`ssh homeserver 'git push origin phase-v1-SL-c-1'`); zero DQ entries needed for routine pushes. Mid-task PMD writes, mid-iteration commits, DQ-on-every-transition discipline all held under auto mode + 1M-context Opus.

Two Stop hooks fired this session; advisor wrote retro evals 111 (score 0.55) + 112 (score 0.45) with 9 cumulative lessons L1-L9 captured pre-compaction. L10 (ci-watcher ensure_ascii=False) added in this retro for completeness.

### 2.2 Planning signals — one substantive miss; the §13 Task 1 numeric-cast IMPLEMENT body

The plan §13 Task 1 IMPLEMENT body prescribed bare `as f64` + `as i64` casts inside `check_grace_staleness`. `Cargo.toml:109` has `as_conversions = "deny"` workspace-wide; the lint fired on the next-stage clippy run after the E0432 fix cleared (DQ #161). Cost: 30 min advisor hand-fix + force-push + advisor-#162 + ci-watcher cycle.

The fix is structurally simple: wrap the affected block (or fn) in the canonical `#[expect(clippy::as_conversions, clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "...")]` pattern that's already in the codebase at `admin_assign_jury.rs:1049-1053`. The planner had access to the pattern (it's been in the fork since Phase 5) but didn't apply it.

§13 Task 2 (scheduler block + atomic guard pair) was first-try clean; the plan body cited submit_jury_vote.rs:67-88 + reputation-snapshot scheduler block at scheduled_tasks.rs:189-245 as canonical references; Junior implemented matching the pattern. Demonstrates: when the planner cites a canonical pattern, Junior can usually transcribe; when the planner authors a novel implement-body without canonical reference, the failure modes (E0432, DbConn type, as_conversions) compound.

Watch-item: planner-side checklist for IMPLEMENT bodies — (a) does the body cite a canonical pattern in the codebase? (b) if it contains numeric cast keywords, is it wrapped in `#[expect]`? (c) does it use any newtype constructor without an accompanying `use` statement?

### 2.3 Impl signals — Junior on Tasks 1 + 2; advisor-laptop on one §G4 hand-fix

**Junior side (Tasks 1 + 2 + fix-impl-1):**
- Task 1 (sponsor_liability_grace module + 4 pub fns + module wiring): 1 commit `eb4001bbd`. Ran clean Junior-side. Workspace-check failed on E0432 + E0599 (DQ #160).
- fix-impl-1 (PersonId import path + DbConn type on `fire_or_escape_case`): 1 commit `54eb1c380`. Junior ran ~10 min clean. Workspace-check then failed on `as_conversions` (DQ #161; advisor-handfix took over).
- Task 2 (scheduler block + AtomicBool guard pair + Drop impl): 1 commit `5b806a95d`. First-try clean both Junior-side and CI-side. Workspace-check passed (DQ #163 → resolved).

**Advisor-laptop side (one §G4 hand-fix):**
- DQ #161 advisor-handfix (`#[expect(clippy::as_conversions, clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "...")]` on `check_grace_staleness`): 1 commit `827d932d5`. 6-line addition. Force-pushed to junior fix-impl-1 trigger branch to fire workflow `25529116907`. Cost: ~30 min total.

5 finalize-merge events; only 1 pushed cleanly (Task 1 first-attempt; the fail-state landed on origin via daemon push). The other 4 (fix-impl-1 merge, advisor-handfix merge, Task 2 merge, ci-watcher #147 merge) landed on EliteDesk's local but needed manual ssh-push for origin propagation.

### 2.4 BM signals — no PR opened yet

SL-c-1 doesn't open a PR until c-2 completes (per the user's split decision in plan §0 — c-1 + c-2 ship together as one PR or back-to-back PRs depending on c-2 scope). No CodeRabbit cycle this sub-phase. No `bm-*` slash command invocations.

The phase branch tip `3d13b6394` is push-ready; the BM-session pattern from SL-a (`bm-cut` → `bm-pr` → `bm-poll-cr` → `bm-triage` → `bm-merge`) will land at SL-c-2 close.

### 2.5 ci-watcher signals — one canonical mutation, one breach recovery

3 ci-watcher dispatches this sub-phase:
- #143 (DQ #160 / workflow `25525281390`): clean mutation, fail result, stayed in pending[] for §G4 audit-trail. Allowlist match → fix-impl-1 auto-queued.
- #145 (DQ #162 / workflow `25529116907`): **breach** — wrote new DQ entry (id collision), `from: ci-watcher`, self-mutated. Recovery via EliteDesk hard-reset + advisor canonical mutation. Cost: ~25 min.
- #147 (DQ #163 / workflow `25531818852`): clean mutation, pass result, moved pending[] → resolved[]. Honored Hard refusal #7. Cosmetic-only `ensure_ascii=True` re-serialization (§3.4 watch-item).

Net: 2 of 3 ci-watcher cycles canonical; 1 of 3 breached. The breach was caught in <30 sec via diff inspection; recovery was mechanical. Brief content + agent-spec discipline + post-mutation verification all needed; reliance on any one in isolation is insufficient.

---

## 3. What didn't work — fix or watch

### 3.1 Plan §13 Task 1 IMPLEMENT body prescribed deny-listed `as` casts
Two bare casts in `check_grace_staleness` (`as f64` + `as i64`) failed `clippy::as_conversions = "deny"` workspace-wide per Cargo.toml:109. Cost: 30 min advisor hand-fix cycle (read canonical pattern + apply `#[expect]` block + force-push + DQ #162 + ci-watcher #147). **Fix:** planner-side checklist for IMPLEMENT bodies containing numeric cast keywords. Either replace with `try_from` + error path, OR wrap in canonical `#[expect(clippy::as_conversions, clippy::cast_precision_loss, clippy::cast_possible_truncation, reason = "...")]` pattern citing `admin_assign_jury.rs:1049-1053`. Add to `.claude/lessons/feedback_planner_clippy_dryrun_implement_bodies.md`.

### 3.2 ci-watcher #145 violated Hard refusals #2 + #7
Subagent wrote a NEW DQ entry (id collision with canonical #162) instead of mutating the existing entry by `workflow_run_id` match; preserved `from: ci-watcher` instead of inheriting the original `from`; self-mutated own bad entry. Recovery via EliteDesk hard-reset + advisor canonical mutation. Cost: ~25 min. **Fix:** (a) strengthen `.claude/agents/ci-watcher.md` first-line emphasis on "MUTATE, do not write new" — move from sub-section to opening sentence; (b) embed canonical Python mutate-in-place recipe in every ci-watcher brief (replacement brief #147 had this; #145 brief did not); (c) add post-mutation diff inspection as a checklist item — `git show <sha> -- .claude/decision-queue.json` checking for `+{` block-append vs `-{` + `+{` for same-id mutation.

### 3.3 Daemon-finalize-merge skip pattern — 8th cumulative occurrence
Junior daemon DOES finalize-merge but does NOT push to origin. 4 of 5 finalize events this sub-phase needed manual `ssh homeserver 'git push origin phase-v1-SL-c-1'` recovery. Cumulative across SL-a + SL-c-1: 17+ instances. **Fix:** queue infra issue against Junior daemon's finalize-merge stage; add `git push origin <phase>` after the merge commit lands locally. Watch-item promoted: `feedback_junior_daemon_finalize_skips_when_worker_pre_pushes` is now empirical at >85% skip rate; further evidence on this point is repetition, not new signal.

### 3.4 ci-watcher #147 mutation re-serialized with `ensure_ascii=True`
The clean mutation by ci-watcher #147 (DQ #163) used Python's default `json.dump(d, f, indent=2)` without `ensure_ascii=False`. Result: every em-dash (`—`) and §-sign in the file became `—` / `§` escapes. Round-trip functionally identical (both Python and node JSON parsers handle `\uXXXX` correctly) but cosmetically noisy (~450 lines of escape-only edits in the diff). Per `feedback_json_dump_ensure_ascii_false` (PMD lesson). **Fix:** add explicit one-line reminder to `.claude/PRPs/templates/ci-watcher-brief.template.md`: "When writing decision-queue.json, use `json.dump(d, f, indent=2, ensure_ascii=False)` to preserve UTF-8."

---

## 4. Per-task complexity score table

Per `feedback_retro_task_complexity_score.md` shape: `<files>/<commits>/<runtime-min>/<max-log-silence-min>`.

| Task | Slug | Files / Commits / Runtime / Silence |
|---|---|---|
| 1 | sponsor_liability_grace module + 4 pub fns + module wiring | 2 / 1 / ~12 min Junior + 5 min CI / ~3 min |
| fix-impl-1 | PersonId import path + DbConn type on fire_or_escape_case | 1 / 1 / ~8 min Junior + 5 min CI / ~2 min |
| advisor-handfix | #[expect] for clippy::as_conversions on check_grace_staleness | 1 / 1 / ~5 min advisor + 5 min CI (force-push to junior branch) / 0 |
| 2 | scheduler block + AtomicBool guard pair + Drop impl | 1 / 1 / ~13 min Junior + 5 min CI / ~3 min |
| ci-watcher #143 | DQ #160 mutation (fail) | 0 / 1 / ~6 min model / ~30 sec |
| ci-watcher #145 | DQ #162 attempted mutation (BREACH) | 0 / 2 (bad) / ~5 min model + ~25 min recovery / N/A |
| ci-watcher #147 | DQ #163 mutation (pass) | 0 / 1 / ~6 min model / ~30 sec |
| 3 | retro (this) | 1 / 1 / ~25 min advisor / 0 |

**Total wall-clock:** ~4h 21min (cut at 21:43 UTC 2026-05-07 → retro author at 02:05 UTC 2026-05-08), of which ~3h was orchestration (advisor + Junior + CI) and ~1h was sleep/poll waits. Net advisor model time ~75 min cumulative across the session.

**Dominant cost:** advisor-handfix cycle (~30 min: audit + edit + force-push + DQ + ci-watcher) + #145 breach recovery (~25 min: diff inspect + AskUserQuestion + reset + canonical mutation + push). All others ≤15 min.

**Cohort note:** No `[P]` markers on any §13 task in c-1; each ran serial as expected. Plan §5 complexity score was `≤8` (no split-or-proceed planner DQ in resolved[]); SL-c-1's wall-clock validates that estimate.

---

## 5. Lessons promoted to `.claude/lessons/`

To be committed at Task 3 retro ship (this commit + immediate follow-up):

1. **`feedback_planner_clippy_dryrun_implement_bodies.md`** — When the plan body contains a numeric cast keyword (`as f64`, `as i64`, `as usize`, etc), the planner must either: (a) replace with `try_from` + error path, OR (b) wrap the entire block in the canonical `#[expect(clippy::as_conversions, ...)]` pattern with a `reason = ...` capturing safety bounds, citing the canonical fork-side instance at `admin_assign_jury.rs:1049-1053`. Surfaced from SL-c-1 DQ #161. Generalises to: any planner-authored IMPLEMENT body that may trip a workspace-deny lint.

2. **`feedback_ci_watcher_inline_python_recipe_in_brief.md`** — Every ci-watcher brief must embed: (a) explicit Hard refusal #7 reminder ("MUTATE the existing entry, do NOT write a new one"), (b) canonical Python mutate-in-place recipe (8-line snippet, locating by `workflow_run_id`, populating result fields, conditional `pending[] → resolved[]` move), (c) explicit "what success looks like" verification footer (entry id stays same; from preserved; kind preserved; new fields populated; no new entry created). Surfaced from SL-c-1 ci-watcher #145 breach + #147 successful recovery. Generalises to: brief authoring for any subagent class with high-stakes hard-refusal contracts.

3. **`feedback_post_mutation_diff_inspection_checklist.md`** — After any subagent's `decision-queue.json` mutation lands, the advisor must `git show <sha> -- .claude/decision-queue.json | head -80` and check: (a) clean `-{` + `+{` for same `id` = mutation; (b) `+{` only with new id = breach (new entry creation); (c) `-{` only = entry removal (suspicious unless paired with archive). Surfaced from SL-c-1 ci-watcher #145 breach detection in <30 sec. Generalises to: any subagent JSON write with structural invariants.

4. **`feedback_ci_watcher_ensure_ascii_false.md`** — When ci-watcher (or any subagent) writes `decision-queue.json` after mutation, use `json.dump(d, f, indent=2, ensure_ascii=False)` to preserve UTF-8 em-dashes + §-signs. Default `ensure_ascii=True` produces ~450 lines of cosmetic-only escape-edits in the diff while round-tripping identically. Surfaced from SL-c-1 ci-watcher #147 (clean mutation but ensure_ascii=True). Generalises to: any JSON write that may surface in human-readable diffs.

### Watch-items (promote-if-recurs)

- §G4 allowlist extension: `clippy::as_conversions` (fix = mechanical `#[expect]` block per canonical pattern). 1 occurrence; add to allowlist if it recurs in SL-c-2 or SL-d.
- Daemon-finalize-merge push skip — pursue upstream Junior daemon patch (§3.3; 17+ cumulative instances now). **High-priority** infra issue at this point.
- Strengthen `.claude/agents/ci-watcher.md` first-line emphasis on Hard refusal #7 (§3.2). Spec-edit, not lesson-promote.
- Update `.claude/PRPs/templates/ci-watcher-brief.template.md` to embed canonical Python recipe + "what success looks like" footer (§3.2). Template-edit.

---

## 6. Confidence score

**0.65** — Two impl tasks shipped clean; one §G4 mechanical fix-impl; one §G4 user-gate hand-fix (planner miss surfaced + cleanly resolved); one ci-watcher breach (caught + recovered cleanly). All workspace-check validate-pending entries at green or pending-as-fail-audit per schema-v2 routing. Per `evaluation-calibration.md`, scores 0.85+ are rare and require no detected risk; 0.65 reflects:
- Plan §13 Task 1 IMPLEMENT body prescribed deny-listed casts (planner miss)
- ci-watcher #145 violated Hard refusals #2 + #7 (subagent miss; recovery clean)
- 4 of 5 daemon-finalize-merge skips needed manual recovery (~5 min cumulative; mechanical)
- ci-watcher #147 ensure_ascii=True (cosmetic; round-trip identical)
- Auto mode + 1M-context held throughout; no compaction-mid-action; user-typed grants honored

Technical signal is strong (workspace-check green, scheduler wired, atomic guard correct per plan §10.1 invariants). Process signal flagged 4 distinct watch-items, of which 2 (#3.1 planner clippy dry-run + #3.2 ci-watcher brief recipe) have concrete forward-going fixes ready for SL-c-2 + future sub-phases.

c-1 met its scope (sponsor_liability_grace scheduler module + clokwerk wiring); c-2 (the e2e test surfaces + PR cycle) remains open.

---

## 7. Follow-up GH issue candidates

Per DQ #46 (one-issue-per-watch-item discipline):

1. **§G4 allowlist extension: `clippy::as_conversions` → mechanical-`#[expect]`-block** (§3.1). One occurrence this sub-phase; promote to allowlist on second occurrence in SL-c-2 / SL-d.

2. **Daemon-finalize-merge push skip patch** (§3.3). 17+ cumulative empirical instances across SL-a + SL-c-1. Justifies upstream Junior daemon patch. **Highest-priority** infra follow-up at this point.

3. **`.claude/agents/ci-watcher.md` first-line emphasis on Hard refusal #7** (§3.2). One-line spec edit; eliminates ambiguity at brief-read time.

4. **`.claude/PRPs/templates/ci-watcher-brief.template.md` embed canonical Python recipe + verification footer** (§3.2). Template edit with 8-line snippet + "what success looks like" footer.

5. **Planner-side IMPLEMENT-body clippy dry-run gate** (§3.1). When IMPLEMENT body contains numeric cast keywords, planner runs `cargo clippy --workspace --features full --no-deps -- -D warnings` against a stub file with the cast in question. 30 sec planner-side; saves ~30 min advisor cycle. Either a `/brehon-clarify` extension or a planner subagent post-write check.

6. **Mutation diff inspection in advisor checklist** (§3.2). Add post-mutation `git show <sha> -- .claude/decision-queue.json` to advisor's CI-cycle ritual. One-line checklist edit.

---

## Sign-off

**Pending — user gate.** Surface to user: "v1-SL-c-1 retro authored at `.claude/PRPs/reports/v1-SL-c-1-retro.md`. Ready for sign-off → bm-pr / bm-cut for SL-c-2 / both per /brehon-phase-transition? Or any retro edits requested?"
