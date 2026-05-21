# v1-federation-inbound-c retro

> Phase tip at retro authorship: `77cbf1be5` (advisor: author Task 4 retro brief — DQ #338 pre-push mandate). This commit (`docs(advisor): v1-federation-inbound-c retro authored`) becomes the new tip on advisor manual finalize-merge.
> Plan: `.claude/PRPs/plans/v1-federation-inbound-c.plan.md` (`22f15bd9a` 2026-05-21 12:39 UTC; scope (a) only).
> Authored: 2026-05-21 by Junior planner #403.
> Phase wall-clock: bm-cut `6dc489c9e` 2026-05-21 17:11 UTC → retro tip ~22:30 UTC ≈ **5.3 hours** (scope (a) reader-side fix; one cohort + one barrier task + retro).
> §16a stories: **3/3 done** — Story 1 (Tasks 1+2 reader-side fix lands), Story 2 (Task 3 e2e regression asserts override behaviour), Story 3 (this retro).
> Phase 2 e2e: **103 passed / 0 failed / 5 ignored** (5 ignored are pre-existing v0-polish TODOs — GH issues #42/#43/#45 — NOT phase regressions). Per DQ #340 (resolved-pass, advisor-laptop, 2026-05-21T22:33Z; runtime 36m11s).

---

## Advisor signals

The advisor honoured the four hard gates this phase: §3.4 DoD smoke
(plan §15 commands dry-run against current HEAD at plan-approval time;
recorded via the bm-cut brief at `6dc489c9e`), §3.5 watchpoint
specificity (plan §4 + §10.1 cite specific file:line ranges —
`config.rs:740-769` for the canonical reader pattern, `inbox.rs:421-438`
for the Task 1 site, `publish_trust_attestation.rs:139-150` for the
Task 2 site), §3.6 canonical-schema-first (plan §10.2 cites the sibling
`per_peer_rate_limit_returns_429` at `e2e.rs:15600-15628` verbatim as
the Task 3 mirror), and PRECON-7 cohort DQ pre-reservation (DQ #328 +
#329 reserved at commit `aa3c77ab5` 18:52:51 UTC BEFORE the cohort
briefs were authored at `05975274c` 18:57:49 UTC — id-collision class
neutralised by construction).

The advisor also handled three high-judgment decisions inline this
phase without surfacing to the user beyond the planned gates:

1. **DQ #324 / #325 clarify pass (pre-planning gate)**: two
   advisor-mode clarifies authored at `48ae7249c` 12:07 UTC resolved
   the planner's two non-trivial design questions (INSERT vs UPDATE on
   the new e2e test; what to do with the misleading workaround comment
   at `e2e.rs:15605-15610`). Both resolved with citations — no
   user-relay required. The clarify-gate observably pre-shaped Task 3
   (impl-3 brief §4 + §10.2 verbatim quote both clarifies; the actual
   commit `9c7c43aff` mirrors both decisions correctly: INSERT with
   `valid_from.eq(diesel::dsl::now)`, comment swapped to the post-fix
   landscape note referencing
   `appended_config_override_takes_effect_returns_429`).

2. **Cohort 1 (Tasks 1+2) dispatched without pre-push mandate**: at
   18:57 UTC when the impl-1 + impl-2 briefs were authored, DQ #338
   (daemon wrong-ref reset bug) had not yet been observed. The daemon
   finalize-merge for Cohort 1 (`8fea680d3` 19:23 UTC) ran cleanly —
   no wrong-ref reset on `phase-v1-federation-inbound-c`. DQ #338 was
   filed at 20:16 UTC on `governance-v0` for a separate session's
   planning task #399. By the time Task 3 was authored (`110d7aae`
   20:47 UTC), the bug was known and the impl-3 brief carried the
   PRE-PUSH MANDATE explicitly. Task 3's manual finalize-merge
   (`877bd849c` 20:55 UTC) was the first to apply the mitigation on
   this phase.

3. **User gate 4 (Phase 2 e2e local vs dispatch)**: chosen LOCAL
   (default-recommended; ~26-36 min, zero billed). DQ #340 raised by
   the advisor at `6d36e9c8e` 21:52 UTC (`from: "advisor"`,
   `kind: "validate-pending-laptop-e2e"`) and mutated to pass at
   `a195421d8` 22:33 UTC. The `*-laptop-e2e` kind variant kept the
   classification clean in DQ scans.

### Recurrence-class observations (load-bearing — promote to lessons if 3+ occurrences)

#### A) `/tmp` Bash↔Python path mismatch recurrence (2× this phase; canonical lesson exists; muscle-memory default still wins)

The canonical lesson `feedback_windows_bash_python_git_show_tmp_traps.md`
was authored 2026-05-09 (12 days ago at retro time) and explicitly
documents trap #2: `/tmp` resolves differently for Bash (Git-Bash maps
to `C:/Users/barri/AppData/Local/Temp`) vs Python (Windows native:
`/tmp` is non-existent unless drive root). The recipe is to use
`$LOCALAPPDATA/Temp/<name>` or `.claude/scratch/<name>.json`, never
`/tmp/`. Despite the canonical lesson being indexed in PMD, the trap
fired **twice this session** during the gov-v0 forward-merge DQ
reconcile (runlog lines 63-106; user-flagged in-channel as
"Note this for retro: /tmp path mismatch again"). Both recoveries were
mechanical (re-issue to `C:/Users/barri/AppData/Local/Temp`); zero
phase impact; but the recurrence is the signal.

Root cause hypothesis (per runlog): the PMD-search-pre-queue pattern
(advisor-orchestrator.md §2.3) is wired for **brief-authoring time**,
not for **ad-hoc inline scripts** during merge reconcile or DQ scans.
The advisor's "write a quick Python one-liner" muscle-memory defaults
to `/tmp` (Linux Bash convention) because the Windows-specific override
only kicks in when the FileNotFoundError forces it. The lesson is
unreachable through the current `memory_search_hybrid` flow because no
brief is being written.

**PROMOTION RECOMMENDATION:** add a NEW lesson
`feedback_tmp_path_mismatch_promote_to_session_start_default.md` that
elevates the trap-recipe to a session-start-default cue (or a Bash-tool
pre-execution hook flagging `/tmp/...` paths and suggesting
`$LOCALAPPDATA/Temp` / `.claude/scratch/`). Cross-phase pattern (this
recurrence is the 3rd+ documented occurrence per
`feedback_windows_bash_python_git_show_tmp_traps.md` originSessionId
12 days ago + 2 this phase). DEFERRED to a follow-up commit on
`governance-v0` (not authored inline in this retro commit per Junior
file-ownership boundary — lesson authorship is fine in retro commit,
but the underlying hook/CLAUDE.md change is advisor-side meta work);
recommended path: advisor commits the new lesson + Bash-tool guard
into `governance-v0` directly in a `chore(lessons):` commit
post-merge.

#### B) Lane bootstrap submodule init miss (this lane; canonical lesson exists; gap is execution discipline, not lesson absence)

Per runlog lines 30-61 (advisor pre-phase audit + submodule init
recovery): lane worktree creation via `git worktree add
../brehon-fork-fed-in-c phase-v1-federation-inbound-c` did NOT
auto-init the `crates/email/translations` submodule. First-run Probe 3
(`cargo-test.bat --test e2e --no-run -p lemmy_server`) failed with
`lemmy_email build.rs read_dir("translations/backend/") returned Os
code 3 NotFound`. Recovery: `git submodule update --init --recursive
crates/email/translations` (lemmy-translations checkout
`a3f9e4669b53f041b92fbbe6bdd80c8db0619c20`); re-ran probe → exit 0
in 18m07s.

The canonical lesson `feedback_phase_lane_worktree_bootstrap_checklist.md`
exists and Step 1 explicitly names `git submodule update --init
--recursive` as part of the lane-bootstrap sequence. The lesson was
NOT applied at lane-creation time. The gap is checklist-execution
discipline, not lesson absence.

**PROMOTION RECOMMENDATION:** the lesson is fine as-is; the structural
fix is to convert the checklist into a one-command idempotent runner
`scripts/brehon/lane-bootstrap.sh <phase-name>` that does worktree-add
+ submodule-init + .mcp.json copy + settings.local.json copy + PMD-
canonical-path verification in one step. File as a forward-scope
sub-phase candidate (NOT shipped in this retro commit). Pattern is
known + the workaround is fast (one `git submodule update`) — low
urgency, but a script eliminates the recurrence class.

#### C) DQ id-collision recurrence avoided (PRECON-7 Option 3 held) + cross-lane forward-merge collision noted

This phase's cohort dispatch (Tasks 1+2) used PRECON-7 Option 3:
advisor pre-reserved DQ ids #328 + #329 in `pending[]` BEFORE the
cohort briefs were authored. Each impl brief named its assigned id in
§4 Constraints. The collision class observed in prior phases (DQ #50
in Phase 6 `e9fa1e01a`) did NOT recur — confirmation that Option 3
works for intra-cohort id reservation.

A **separate** collision class did fire mid-phase: the
`brehon-conformance-audit` lane independently filed a DQ #326 (runlog-
deletion RCA) at `cb37dc584` 17:58 UTC on this lane while gov-v0 also
held a #326 (conformance-audit task 8 stale-pending). On
gov-v0→fed-in-c forward-merge (`7f02254fe` 19:56 UTC), conflict
resolution renumbered the fed-in-c #326 to #337 (`chore(merge):
forward-merge governance-v0 -> phase-v1-federation-inbound-c (108
commits; DQ #326 collision-renumber to #337)`). This is a cross-lane
issue, not an intra-cohort one — Option 3 doesn't address it because
two independent lanes both compute `next_id` against their own
working-tree views and converge on the same number.

**PROMOTION RECOMMENDATION:** structural fix is to extend
`scripts/brehon/resolve-dq-canonical.sh` to walk `git worktree list`
output + `git show <other-lane>:.claude/decision-queue.json` and
compute `next_id` across all visible refs (already half-implemented per
`multi-lane-worktree.md` §"Worktree-aware DQ id discipline"). NEW
lesson candidate: `feedback_multi_lane_dq_id_collision_walk.md` (or
extend `feedback_cohort_dq_id_collision.md` if the patterns are close
enough). Forward-scope; not shipped in this retro.

#### D) DQ #338 daemon wrong-ref reset bug AVOIDED via pre-push mandate (single-occurrence — pattern not yet "recurrence")

DQ #338 (gov-v0, `kind: "blocker"`, `from: "advisor"`, filed
2026-05-21T20:16Z; still `pending` at retro authorship) records that
the Junior daemon reset `governance-v0` to
`origin/phase-v1-federation-inbound-c` immediately after finalize-
merging planning task #399's plan commit, destroying that plan commit
(recovered via cherry-pick at `c02dc8617` then push to
`recovery/v1-dq-schema-r1-plan`). The recurrence class is: daemon's
finalize step issues an unintended `git reset --hard` against the
wrong ref when a phase branch is concurrently active.

The mitigation applied for the remainder of this phase: Task 3 brief
(`110d7aaee` 20:47 UTC) carried the PRE-PUSH MANDATE — worker pre-
pushes its commit to `origin/junior/<branch>` BEFORE the daemon's
finalize step runs; advisor manually finalize-merges from
`origin/junior/<branch>`. Applied at `877bd849c` 20:55 UTC and again
at this retro commit. The daemon's wrong-ref reset never fired on
`phase-v1-federation-inbound-c` (verified by Cohort 1 `8fea680d3`
landing cleanly + Task 3 advisor manual merge cleanly).

**PROMOTION RECOMMENDATION:** the lesson
`feedback_junior_finalize_skips_when_worker_pre_pushes.md` already
exists and was load-bearing here. Confirm at next sub-phase plan-
write that the file-class lesson injection table
(advisor-orchestrator.md §2.4) gains a row for "any plan touching
`phase-v1-*` AND daemon finalize-merge → inject PRE-PUSH MANDATE
into impl brief §4 Constraints". Carry-forward: until DQ #338's
structural fix lands (option-a investigation of
`/opt/junior-src/src/daemon/executor.ts` finalize-merge code path),
**every impl-task brief MUST carry the pre-push mandate**. This is
documented as a process invariant in the Carry-forward block below.

---

## Planning signals

The plan-write quality this phase was high. Concrete evidence:

### Specific dogfood checks (verified at retro time, not at plan-write time)

- **§15.1 cargo-check** (per-task workspace check): passed at Tasks 1
  (DQ #328, 8m47s cold), 2 (DQ #329, 1m37s warm), 3 (DQ #339, 1m26s
  warm). Three consecutive passes, no compile drift, no surprise
  warnings.
- **§15.2 clippy** (`--workspace --features full --no-deps -- -D
  warnings`): passed uniform at Tasks 1 (7m25s), 2 (2m24s), 3 (2m34s).
  The new federation-mod-roots `#![deny(clippy::disallowed_methods)]`
  attribute (landed via the gov-v0 forward-merge from
  `brehon-conformance-audit` PR #141) produced **zero** firings on the
  new fed-in-c code — confirms that the canonical reader pattern
  (`.order_by(governance_config::valid_from.desc()).first(...)`) is
  not on the disallowed list, and that no new code introduced a
  disallowed method.
- **§15.3 cargo test --no-run**: passed at Task 3 (DQ #339, 18m13s).
  Re-link succeeded after the new test fn insertion + comment swap.
- **§15.4 Phase 2 e2e** (`--workspace --test e2e --features full`):
  **103 passed / 0 failed / 5 ignored** in 36m11s on phase tip
  `e06e918e2` (per DQ #340 resolved-pass). The new test
  `appended_config_override_takes_effect_returns_429` is GREEN — proves
  Task 1's `.order_by(valid_from.desc())` makes append-history
  INSERT-with-newer-`valid_from` overrides effective. Pre-existing
  fed-in-b and fed-in-a tests still pass — no regression.
- **§15.6 cross-cutting verification (21 boxes)** — walked at retro
  time against phase tip `77cbf1be5`:
  - ✓ `grep -n "order_by(governance_config::valid_from.desc())" crates/apub/activities/src/governance/inbox.rs` → 1 match in `get_inbound_config_int`.
  - ✓ `grep -n "order_by(governance_config::valid_from.desc())" crates/apub/activities/src/governance/publish_trust_attestation.rs` → 1 match in `actor_cap` block.
  - ✓ `grep -n "fn get_inbound_config_int" inbox.rs` → line 421 (signature unchanged).
  - ✓ `grep -n "Mirrors the .get_inbound_config_int. helper in inbox.rs" publish_trust_attestation.rs` → line 135 (helper-duplication comment intact; PRECON-3 honoured).
  - ✓ `grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` → **55** (no kind additions).
  - ✓ `grep -c '^\s*ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs` → **55** (shim parity unchanged).
  - ✓ `grep -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs` → line 15510 (sibling module retained).
  - ✓ New test `appended_config_override_takes_effect_returns_429` at line 15630 — between `per_peer_rate_limit_returns_429` (15600-15627) and `replayed_activity_returns_409` (15672+).
  - ✓ Workaround comment at `e2e.rs:15606-15609` updated to the post-fix landscape note referencing the new test by name (DQ #325 honoured).
  - ✓ Zero edits to `crates/db_schema/src/source/governance/governance_config.rs`.
  - ✓ Zero edits to `migrations/**` (PRECON-5 honoured).
  - ✓ Zero new files under `crates/apub/activities/src/governance/` (PRECON-3 honoured; helper not extracted).
  - ✓ R1 (no `i32 as i64` casts in new code): Task 3 uses `2_i64` literal and `0..2` u32 counter cleanly.
  - ✓ R6 honoured: every §15.2 invocation uses `--no-deps -- -D warnings`.
  - ✓ R7 honoured: Task 3 ran `cargo test --no-run` (DQ #339 §15.3).
  - ✓ PRECON-1 (scope (a) only) honoured: no DoS-hardening edits, no convention-audit edits, no helper extraction.
  - ✓ PRECON-2 (canonical-mirror): both fix sites cite `config.rs:740-769` per impl-1.md:85 + impl-2.md:90.
  - ✓ PRECON-3 (no helper extraction) honoured.
  - ✓ PRECON-4 (one e2e test, append-history-aware) honoured — INSERT-with-newer-`valid_from` per DQ #324.
  - ✓ PRECON-6 (validate-pending-laptop DoD) honoured — DQ #328 / #329 / #339 all `kind: validate-pending-laptop` with `answered_by: advisor-laptop`.
  - ✓ PRECON-7 (cohort DQ pre-reservation) honoured: ids #328 + #329 pre-reserved at `aa3c77ab5` BEFORE the cohort briefs.
- **§15.7 ADR / OQ compliance** (5 boxes):
  - ✓ ADR-006 (advisory-only inbound persistence) — unchanged; reader-side fix only.
  - ✓ ADR-013 (illegal content / `CaseStatus::EmergencyRemove`) — not in code path.
  - ✓ ADR-014 (governance signals fork-only AP types) — unchanged.
  - ✓ ADR-015 (pseudonymisation) — no new TEXT column; no raw-id surface.
  - ✓ Append-only contract preserved — reader now correctly surfaces latest row.

### Plan-write observations beyond §15

- **§10.1 MIRROR pattern citation discipline held**: both impl-1.md (line 85) and impl-2.md (line 90) cite plan §10.1 lines 163-229 verbatim with the canonical-reader-pattern range `config.rs:740-769`. Workers committed exactly the expected one-line insert each.
- **§13 Task 3 spec precision** (line numbers + anchor-by-text rules): the brief named the anchor "between `per_peer_rate_limit_returns_429` and `replayed_activity_returns_409` inside `mod v1_federation_inbound_b_fixtures`" + the line-range hints. The impl worker (#402) chose `appended_config_override_takes_effect_returns_429` from the 3 candidate names listed in plan §13 Task 3 GOTCHA, mirrored sibling Case A error-shape verbatim, and updated the workaround comment in the same commit (single-edit budget of 47 insertions / 6 deletions, well below the <=200 cap).
- **Complexity score 2/10 was accurate**: plan §5.1 predicted 2/10 (3 impl tasks all 1-file mechanical). Actual outcome confirms: zero cycles, zero fix-impl spawns, zero §G4 classifier firings. Plan-shape uniformity with canonical sibling was correct.
- **Dogfood gate held**: §3.7 dogfood-gate requirement for new commands didn't fire because this phase added no new slash commands; §15 cargo command set is the same as v1-federation-inbound-b's.

---

## Impl signals

### Per-task complexity scores

Format: `<files>/<commits>/<wall-clock-min>/<max-log-silence-min>` per `feedback_retro_task_complexity_score.md`. Source: `git log --format="%H %ai %s"` on phase branch + DQ resolved-at timestamps for validate runtime. Junior `mcp__junior-brehon__show_task` records are NOT accessible from this planning subagent's environment — wall-clock derived from advisor brief-author commit → worker feat commit timestamps (includes daemon-pickup + dispatch); max-log-silence cited as `n/a` where Junior MCP data is the only source.

```
Task 1 (Junior #397): 1/1/~3min/n/a   — inbox.rs +1 line; cohort 1 with Task 2; brief 17:57:49 UTC → feat 19:00:17 UTC (~62 min advisor→commit including daemon pickup; pure worker edit+commit ≤3 min for a 1-line mechanical insert)
Task 2 (Junior #398): 1/1/~3min/n/a   — publish_trust_attestation.rs +1 line; cohort 1 with Task 1; brief 17:57:49 UTC → feat 19:00:18 UTC; same envelope as Task 1
Task 3 (Junior #402): 1/1/~5min/n/a   — e2e.rs +47/-6 lines (1-line import + 33-line new test fn + 6-line comment swap + 7 other); brief 20:47:34 UTC → feat 20:52:09 UTC (~4m35s advisor→commit; pure worker edit+commit ≤5 min)
Task 4 (Junior #403 — this retro): 1/1/~20min/n/a — .claude/PRPs/reports/v1-federation-inbound-c-retro.md (single new file); planner read budget ~10 min (plan §13/15/16a/17 + 5 briefs + runlog + DQ #324-340 + commit graph + §15.6 cross-cutting greps); author budget ~10 min
```

Validate-pending-laptop runtimes (advisor-laptop side, per DQ resolved-at):

```
DQ #328 (Task 1):  cold cargo-check 8m47s + clippy 7m25s   = 16m12s total
DQ #329 (Task 2):  warm cargo-check 1m37s + clippy 2m24s   =  4m01s total
DQ #339 (Task 3):  warm cargo-check 1m26s + clippy 2m34s + test-norun 18m13s = 22m13s total
DQ #340 (Phase 2 e2e local): full e2e suite                 = 36m11s
```

### Aggregate signal section

```
Total wall-clock (bm-cut → retro tip): ~5.3 hours
Total advisor user-gates: 4 (gate 1 plan approval; gate 4 e2e local-vs-dispatch chosen LOCAL; the DQ #338 pre-push gate mid-phase; the Task 4 retro gate)
Total catch-fires: 0
Total breaches: 0 (no advisor authoring content on Junior worktrees; no advisor-attribution in non-advisor commits; no DQ raised without atomic push; no Junior write to forbidden file classes)
Total fix-impl cycles: 0 (clean impl pass on all 3 tasks; no §G4 classifier firings)
Total CR-fix-in-PR cycles: pending (bm-pr happens AFTER this retro per stage-shape orchestration; the BM section caveats below)
```

### Impl-specific observations

- **Cohort 1 perfect cohort dispatch (Tasks 1+2 [P]):** Tasks 1+2 ran
  in parallel; zero file overlap (modifies arrays disjoint:
  `inbox.rs` vs `publish_trust_attestation.rs`); YAML overlap check
  passed at advisor-side dispatch; both committed in the same second
  (commits `0ed04fc7` 19:00:17 UTC + `01ed9241f` 19:00:18 UTC);
  cohort barrier on both validate-pending-laptop result:pass cleared
  cleanly via the daemon finalize merge `8fea680d3` 19:23 UTC + DQ
  mutations `4a4b0f4` 19:45 UTC + `003c0f9` 19:51 UTC. No collision
  on the pre-reserved DQ ids #328 + #329. No flake on the second
  cohort member.
- **Task 3 textbook execution:** worker (Junior #402) chose
  `appended_config_override_takes_effect_returns_429` from the 3
  candidate names in plan §13 GOTCHA; mirrored sibling Case A error
  shape verbatim (`LemmyResult<()>` outer, bare `?` propagation, no
  `Box<dyn Error>` bridges); added the missing `governance_config`
  import correctly (the brief had pre-identified it as needed in
  §3.1); HANDOVER trailer well-formed in the feat commit body. The
  47-line insertion + 6-line comment swap fits well within the
  <=200-line edit budget and avoided full-file Edit on `e2e.rs`
  (15773 lines).
- **No `governance_config_current` view drift:** PRECON-2 GOTCHA was
  honoured by both Task-1 and Task-2 workers; both used the
  Diesel-typed base-table chain `governance_config::table.filter(...).select(...).order_by(governance_config::valid_from.desc()).first::<Option<i64>>(conn)`.
- **No helper extraction across files:** PRECON-3 honoured; the
  helper-duplication comment at `publish_trust_attestation.rs:135-138`
  remains intact (cited in §15.6 cross-cutting box 4 above).

---

## BM signals

### Pre-bm-pr signals (BM section is partial — bm-pr / bm-merge happen AFTER this retro per stage-shape orchestration)

- **`bm-cut` (Junior #393) finalize deleted runlog as index-only
  file** — captured in DQ #337 (`kind: "log"`, advisor-self-resolved
  at 2026-05-21T17:35Z; renumbered from #326 during the cross-lane
  gov-v0→fed-in-c forward-merge) + advisor re-apply commit
  `21583e798` 17:57 UTC. Root cause: Junior's plumbing-path
  workaround (`git hash-object` + `update-index` per commit
  `9df6879a6` on the worker branch) for the PostToolUse hook
  (`allow-prp-deliverables.sh`) blocking direct `Write` on
  `.claude/runlog/**`. The plumbing path created the file in HEAD
  but NOT in the working tree; finalize-agent's `git status` saw
  the file as "deleted" and committed the deletion before push.
  **Surface here in case bm-cut behaviour needs revision before
  next sub-phase** — either the PostToolUse hook should allow
  `Write` on runlog files for bm-cut briefs specifically, or the
  brief should explicitly route runlog authorship via the plumbing
  path AND the finalize-agent should retain index-only files (the
  symptom is asymmetric tooling: hash-object writes the index but
  status compares HEAD↔working-tree, missing index-only state).
- **No bm-pr / bm-merge / CR triage signals YET** — those happen
  post-retro, post-/brehon-verify Story 3 checkpoint, pre-merge.
  If this retro is being read AFTER bm-merge, ignore this caveat
  and fill in the BM section with bm-pr workflow signals + CR
  finding counts (four-bucket) + any merge-time issues. The
  expectation is: small clean PR (3 feat commits + 1 retro + meta
  commits), no surprise CR findings (clippy is already
  `-D warnings` clean, Phase 2 e2e is 103/0/5).

### BM observations across the phase (anticipatory — for cross-sub-phase calibration)

- **Two-session split worked cleanly:** advisor session (this lane,
  CWD `brehon-fork-fed-in-c`) handled DQ writes + brief authorship +
  finalize-merges; daemon-side Junior workers (#393, #397, #398, #402,
  this #403) handled the discrete work units. No file-ownership
  breaches observed.
- **Forward-merge from gov-v0 was substantial (108 commits) but
  uneventful:** `7f02254fe` 19:56 UTC merged the
  brehon-conformance-audit PR #141 work (federation mod-roots deny
  attribute + clippy.toml + lesson files + skill files) into this
  lane. The new `#![deny(clippy::disallowed_methods)]` attribute on
  3 federation `mod.rs` files passed §15.2 clippy clean on all
  three impl tasks + Phase 2 e2e — zero violations introduced by
  fed-in-c code.

---

## Lessons promoted this phase

(none promoted in this retro commit — observations stay in the
recurrence-class section above. Three promotion candidates surfaced
(A `/tmp` path mismatch elevation; B lane-bootstrap one-command runner
script; C cross-lane DQ id-collision walk extension) but each requires
either a NEW lesson file authored on `governance-v0` directly (advisor
meta-edit lane per `phase-branch.md` "Direct on `governance-v0`") or a
follow-up sub-phase scope. Promote at v1-federation-inbound-d or in a
parallel `chore(lessons):` advisor commit if patterns persist.)

---

## Carry-forward to next sub-phase

### Decision-point: v1-federation-inbound-d — (b) Copilot DoS-hardening family in/out?

The (b) Copilot DoS-hardening family was deferred per advisor
2026-05-21 (brief §0.1.1 + §0.2 item #1). Now that (a) reader-side
append-history is shipped + e2e-validated (103/0/5 clean), evaluate:

- **Is (b) still the next-best-value follow-up?** Weigh against other
  fed-in-* gaps (e.g. additional federation-mod-roots Phase-6
  convention coverage; ADR-014 fork-only AP types audit;
  rate-limit-counter cleanup scheduler hardening).
- **If yes, what scope subset of (b) is most load-bearing for an
  MVP-pilot perspective?** The original (b) family enumerated 4+
  items (per the planning-1 brief's §0.1.1 capture); the in-scope
  cut may be just 1-2 (e.g. inbox-size cap pre-check; per-actor
  attestation cap independent of per-peer rate).
- **Is (b) decision-blocked on anything else?** No pending ADR or
  PRD scope refresh known at retro time; advisor + user to confirm.

**Recommended advisor surface at v1-fed-in-c phase-transition gate**:
- option-a: ship (b) full next sub-phase (largest scope; ~5-8 tasks)
- option-b: ship narrow (b) subset (2 tasks: inbox-size cap +
  per-actor attestation cap, e2e tests for each)
- option-c: defer (b) for a different fed-in-* gap
- option-d: defer (b) until pilot threshold

### Residual conformance-audit gap evaluation

The `brehon-conformance-audit` ship (PR #141 merged into governance-v0
at `7cfc21c23` 18:35 UTC + forward-merged into this phase at
`7f02254fe` 19:56 UTC) installed `#![deny(clippy::disallowed_methods)]`
on 3 federation `mod.rs` files + `clippy.toml` workspace-allow. Audit
found Tier-1 violations only on `lemmy_diesel_utils` (DQ #326 leftover,
blocked on DQ #307 fix-impl-2 from the audit lane). Evaluate at this
retro:

- **Did this phase's edits (`order_by` additions + new e2e test)
  introduce ANY conformance-audit-relevant violations?** Walked
  manually at retro time: NO. New code uses the canonical
  Diesel-typed reader pattern (axis #1-4 clean: conn-type, append
  reborrow, trait-bound, error idiom); no helper extraction
  (axis #5); no ADR-015 raw-id surface (axis #6). The
  conformance-audit skill in `phase-diff` mode at advisor-side
  pre-bm-merge should confirm.
- **Residual gaps in coverage?** Yes — the
  `brehon-conformance-audit` skill currently scopes to 3 federation
  `mod.rs` roots + `crates/db_schema/src/source/governance/**.rs` +
  `crates/api/api/src/governance/**.rs`. Future expansion candidates
  (NOT shipped this phase): db_schema_file/governance subtree,
  governance_db_views, `lemmy_diesel_utils` itself (DQ #307 blocker
  permitting). Sub-phase candidate for next planning iteration if
  the audit lane closes DQ #307.

### DQ #338 daemon-bug structural fix (cross-session)

DQ #338 (gov-v0, `kind: "blocker"`, `from: "advisor"`, filed
2026-05-21T20:16Z) is in another advisor session's investigation
queue. Until it resolves (option-a structural fix to the daemon's
finalize code path in `/opt/junior-src/src/daemon/executor.ts`),
**every impl-task brief that involves finalize-merge MUST carry
the pre-push mandate** explicitly in §4 Constraints. Document as a
process invariant in the next plan's PRECON enumeration. Worker
self-pushes its branch to `origin/junior/<branch>` BEFORE the
daemon's finalize step runs; advisor manually finalize-merges from
`origin/junior/<branch>`. The cohort-1 + Task-3 + this-retro
patterns are the working template.

### DQ #326 stale leftover housekeeping

DQ #326 (`kind: "validate-pending-laptop"`, `from: "impl"`, `result:
pass`, but still in `pending[]` post-mutate) is the
conformance-audit-task-8 validate-pending entry that landed via the
gov-v0→fed-in-c forward-merge. It's a stale-pending artefact of the
cross-lane merge: the entry was already mutated to pass on the
conformance-audit lane but the renumbering during merge conflict
resolution moved it from gov-v0's view into fed-in-c's `pending[]`
slot. Action: out-of-scope here — either (a) the
conformance-audit lane's next sub-phase resolves DQ #307 and #326
moves to `resolved[]` naturally, or (b) advisor explicitly migrates
#326 to `resolved[]` in a `chore(decision-queue):` commit on
`governance-v0`. **Recommend (a)** — let the audit lane handle its
own bookkeeping; do not cross-lane-edit fed-in-c's DQ for an audit-
lane entry.

### Carry-forward: PMD path-mismatch hook proposal

From recurrence-class observation A above: propose a Bash-tool
PreToolUse hook (or a `/check-tmp-paths` lint script) that flags
`/tmp/...` paths in any Bash invocation and suggests
`$LOCALAPPDATA/Temp` / `.claude/scratch/<name>.json`. The hook fires
at execution time, not at brief-authoring time, closing the gap that
PMD-search-pre-queue (advisor-orchestrator.md §2.3) leaves open for
ad-hoc inline scripts. Forward-scope; track as a candidate for the
next harness-audit sub-phase or the next session-retro session.

---

HANDOVER:
  filesCreated: [.claude/PRPs/reports/v1-federation-inbound-c-retro.md]
  filesModified: []
  keyDecisions:
    - "Recurrence-class A (/tmp path mismatch, 2x this phase + canonical lesson 12 days old) flagged for promotion to session-start default — DEFERRED to advisor meta-commit on governance-v0"
    - "Recurrence-class B (lane bootstrap submodule init miss, this lane) flagged for one-command idempotent runner script scripts/brehon/lane-bootstrap.sh — DEFERRED to forward sub-phase"
    - "Recurrence-class C (cross-lane DQ id collision, this phase #326->#337) flagged for cross-lane next_id walk extension of scripts/brehon/resolve-dq-canonical.sh — DEFERRED to forward sub-phase"
    - "DQ #338 daemon-bug PRE-PUSH MANDATE documented as process invariant in §Carry-forward until structural fix lands; cohort-1 + Task-3 + this-retro patterns are the working template"
    - "(b) Copilot DoS-hardening family decision-point surfaced to advisor for v1-fed-in-c phase-transition gate (options a/b/c/d)"
    - "No new lessons promoted in this retro commit (3 candidates DEFERRED to follow-up commits)"
  notes: "Final commit of v1-federation-inbound-c work. Worker pre-pushes per DQ #338 daemon-bug mitigation. /brehon-verify runs next (advisor inline) checking §16a Story 3 + Brief-Scope outputs; then bm-pr; then CR triage; then bm-merge; then /brehon-phase-transition."
