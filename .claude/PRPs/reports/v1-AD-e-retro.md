# v1-AD-e retro — server-rendered admin dashboard HTML pages (Dashboard + Audit)

**Sub-phase**: v1-AD-e — pure-templating HTML pages over the shipped v1-AD-d aggregate + SSE backend
**Branch**: `phase-v1-AD-e` (merged into `governance-v0` as `486a24c68`, 2026-05-17T19:34:55Z)
**PR**: #133 (53 commits, `--merge` — task-per-commit history preserved)
**Wall-clock**: ~19h elapsed (2026-05-17 00:25 first impl dispatch → 19:38 merge); **active orchestration ≪ elapsed** (overnight gaps + multi-lane interleave with v1-ship-1). Of the active time, a **disproportionate share went to Junior-worker rescues** (see Cross-cutting §1).
**Outcome**: shipped, all 3 §16a stories ✓, all 6 CR findings resolved, full e2e 94 passed / 0 failed, ADR-015 fully satisfied.

This is a **retro, not a completion report** (`feedback_retro_not_report`): it is structured per-role (`feedback_four_role_retro_signals`) and leads with what to change, not what was done. The "what was done" lives in the runlog + the PR.

---

## TL;DR for the next phase

The implementation was clean (4 impl tasks, plan held verbatim, one-pass per task). **The cost of this phase was almost entirely orchestration-infra, not implementation:** the `#292 stale-base-self-merge` daemon bug fired on **5 consecutive Junior workers** (#292 bm-pr, #295 bm-poll-cr, #296 bm-triage, #297 fix-impl-1, #299 fix-impl-2, #301 bm-merge — effectively every worker), each requiring a manual cherry-pick rescue that would otherwise have mass-deleted 766–2819 lines of advisor files. A **new distinct failure mode** also appeared (#297 worker-exited-without-push). **The single highest-leverage fix for the next phase is a daemon-side change** so workers fork from `origin/<phase>` not the stale daemon-local ref. Three lesson candidates are promoted below.

---

## Per-role signals

### ## Advisor (this session — opus-4-7, meta-oversight)

**What worked — keep:**

- **Never trusting the daemon's "done/succeeded".** Every single Junior completion this phase was inspected (`git merge-base --is-ancestor`, `git diff --stat`, blob-SHA spot-check) BEFORE acting. This caught all 5 #292 rescues, the #297 no-push, and the #301 false-failure (daemon finalize choked but the merge had succeeded server-side). A naive "task done → advance" would have silently reverted advisor work or merged a phase tip missing the fix on at least 4 occasions.
- **Surfacing the cr-5 partial as a sharp user judgment call, not a vague question.** fix-impl-1 (#297) scrubbed only `reason`+`denial_reason`; the brief §2.2 enumerated a broader ADR-015 field set. The advisor caught the gap on a post-task **diff-vs-brief-enumeration spot-check** (not the worker's self-summary), did NOT silently accept it NOR silently expand it (advisor never authors content), and put it to user gate 5 with the precise technical framing (which fields, `scrub` vs `scrub_json`, the actual ADR-015 exposure = `previous_value`/`new_value` carry arbitrary admin-entered config). The user made the rigorous choice (complete-cr-5-first). This is the correct advisor posture on a hard-ADR completeness gap.
- **Checkpoint-reuse discipline.** The DQ #245 / #246 full e2e (94/0/5, ~33min) was reused as the `/brehon-verify` §16a story checkpoint each time rather than re-running a 2nd 32-min e2e. The single binary covers all 3 stories' sub-assertions + the cr-4 regression guard. Saved ~32min × 2 with zero loss of signal.
- **Authoritative cargo verification.** Every DoD command (DQ #245, #246) verified via the explicit `CMDn_EXIT_0` marker + `error[`/`warning:`/`test result:` grep, NEVER the bg-completion notification exit code (`cargo-output-capture` / `feedback_background_task_notification_lies`). Strictly sequential gating (cmd-N+1 not started until cmd-N's marker confirmed). The render-path test (`admin_audit_html_returns_html_for_admin`) was the specific signal that proved the `scrub_json().to_string()` shape correct.
- **Gate discipline held under a long, rescue-heavy phase.** All 6 user gates respected. Gates 3 (CR triage), 5 (merge confirm, re-surfaced after cr-5 completion), 6 (retro sign-off) each a real STOP. The 2.5 CR-re-poll soft-gate override was **recorded as DQ #247** (`kind: log`, `answered_by: user`) per "do not silently skip a soft gate" — the user authorized it explicitly at gate 5 with the pending-CR risk surfaced verbatim.
- **L14/L15/L16 belt-and-braces all fired correctly.** L15: merge-gate read-only pre-checks ran advisor-side inline (incl. discovering governance-v0 has NO branch protection → CodeRabbit is advisory, not a required check — this reconciled the Phase-2.3 PENDING-check concern cleanly). L14: BM #301 skipped its `chore(bm)` runlog commit (it false-failed at daemon finalize) → advisor authored the `docs(advisor):` re-apply. L16: post-merge branch-delete verified (origin `phase-v1-AD-e` confirmed gone — `gh pr merge --merge` auto-deleted it).

**What didn't — fix:**

- **Two self-inflicted Python backslash-escaping bugs writing DQ #246** (`scripts\brehon` → `scriptsrehon` from `\b`→backspace; then a `\'` SyntaxError). Cost ~3 extra tool calls + a Write-fragment workaround. Caught + fixed BEFORE any bad push (no external impact), but pure avoidable churn. **→ Lesson candidate #3 (promoted below).**
- **Windows shell parens-mangling recurred** (`git rev-parse --short A B` choked because an untracked `_assist.txt`-style junk file with `(` in its name was in cwd). Recovered each time within one step, but it is the same cross-platform class flagged in prior retros — the fix (don't pass two refs in one rev-parse when cwd has odd untracked names; or scope the command) is known; the miss is not pre-applying it.
- **The canonical `brehon-fork` checkout was in a divergent detached-HEAD state** (`e706cdefe`, couldn't ff governance-v0 for the L14 re-apply). Worked around by ff-ing the lane worktree's local governance-v0 to origin and doing the re-apply there. The canonical checkout's divergence is a latent housekeeping debt — flag for a future cleanup (it is NOT this phase's job to reset it; per no-destructive-defaults, investigate before forcing).

### ## Planning (opus-4-7, Junior — predates this session's active arc)

The plan (`v1-admin-dashboard-e.plan.md`) held up **verbatim across all 4 impl tasks** — one-pass per task, no plan-revision DQ, no mid-impl ambiguity blocker from the impl workers. Strong signals:

- §16a Stories decomposition was sound: 3 stories, composing-task mapping correct, checkpoint commands runnable.
- The pure-templating-over-shipped-backend framing was accurate — Tasks 1–5 were translation, not design (the v1-AD-d aggregate + SSE backend were already in place).

**One descriptor-grammar imprecision (NOT a planner failure, but a recurring class):** §16a Story 2's Brief-Scope descriptor for the audit route reads the grep-literal `contains .route("/audit/view"`. The actual implementation is the idiomatic actix `scope("/audit").route("/view", ...)` (lib.rs:559-561) — functionally identical, e2e-proven. `/brehon-verify` flagged this as a candidate-phantom on every run; the advisor correctly diagnosed it as a plan-wording imprecision (same disposition all 3 verify runs made) NOT a phantom. **→ Lesson-adjacent: the plan-template should warn planners to write route descriptors as the handler-fn name (`.to(<handler>)`) not the path literal, OR `/brehon-verify` should treat a route-literal descriptor miss as "inspect the routes file for a composed scope" before classifying phantom.** (Carried from retro #362; reproduced this phase — promote-worthy if it recurs a 3rd phase.)

### ## Impl (sonnet-4-6, Junior — 4 tasks + 2 fix-in-PR)

**What worked:**

- Each of Tasks 1/2/3/5 shipped in one Junior run, plan-faithful. Task 4 (audit live-tail `<script>`) was pre-satisfied by Task-2 scope-bleed (verified, intentional, re-confirmed every verify run).
- fix-impl-2 (#299) was **exactly the brief §2.2** — byte-level spot-check confirmed `scrub()` on entry_kind/scope/key/actor_pseudonym, `scrub_json().to_string()` on previous_value/new_value, `reason`/`denial_reason` left single-scrub (not double), import widened, ONLY `audit_entry_row` + the `use` line touched. The brief was mechanical enough that the worker could not misinterpret which fields or which fn — **this is the model for enumerated-transform briefs.**

**What didn't — fix (two distinct impl-worker failure modes):**

1. **cr-5 silent-partial (fix-impl-1, #297).** The brief §2.2 enumerated N fields to scrub; the worker scrubbed the 2 most-obvious free-text ones (`reason`, `denial_reason`), filed NO blocker for the ambiguity, and self-reported "done". "Did the obvious subset and reported done" is a **silent-partial failure mode**. The advisor's post-task diff-vs-enumeration spot-check was the catch. **→ Lesson candidate #2 (promoted): an impl-task brief that enumerates N fields/sites to transform must require the worker to transform all N OR file a blocker naming which it skipped and why.**
2. **#297 worker-exited-without-push (NEW failure mode).** The #297 worker did all the work correctly (commit `835681ff5`, 3 files, all 4 findings), but its final reasoning block ("I should git push... let me push, then write the retro") got **distracted by the Stop-hook retro scoring requirement and it exited WITHOUT running `git push`**, wrongly assuming Junior finalize would push (finalize skips on no-prepush per `feedback_junior_finalize_skips_when_worker_pre_pushes`). The daemon reported "succeeded". Tell: worker branch absent from origin + phase tip unchanged + no fix commit on origin despite a succeeded task. Work was recoverable from the daemon worktree object store + reflog. **→ Lesson candidate #1 (promoted): dispatch briefs must hard-order `git push origin HEAD` BEFORE the Stop-hook retro and forbid exiting between them; the retro requirement is currently competing with the push step and winning.**

### ## Branch Manager (haiku-4-5, Junior — bm-pr, bm-poll-cr, bm-triage, bm-merge)

**What worked:**

- The four-bucket triage (#296) was **sound and conservative** — all 3 substantive code findings (cr-4 auth-order 404, cr-5 ADR-015 scrub, cr-6 missing symmetric test) → `fix-in-pr` (correct); cr-2 → `rebut` (Task-4 scope already user-approved DQ #237, independently verified by advisor via blob-SHA); cr-3 → `wont-fix` (nit). **The cardinal sin (`feedback_coderabbit_block_merge_critical` — lazily rebutting a real code finding) did NOT occur.** The brief §2a anti-laziness override (force re-derive all buckets from scratch with the revert test) worked.
- bm-merge (#301) **succeeded at the irreversible action** (`gh pr merge 133 --merge` landed `486a24c68` server-side) despite the worker's worktree being #292-stale — because `gh pr merge` operates on GitHub's PR state, not the local worktree.

**What didn't — fix:**

- **Every BM worker hit #292 stale-base** (#292/#295/#296/#301). Each would have mass-deleted advisor files if finalize-merged. The advisor's cherry-pick-the-clean-commit-only recipe is reliable but this is **5 rescues across the phase** — see Cross-cutting §1.
- **BM #301 skipped its L14 runlog commit** (it false-failed at daemon finalize before the `chore(bm)` commit could push). The L14 belt-and-braces (advisor `docs(advisor):` re-apply) covered it — working as designed, but it means the BM brief's explicit L14 git sequence is **not sufficient on its own when the daemon finalize fails first**. The advisor-side L14 fallback is load-bearing, not optional.
- **bm-triage (#296) did not force-add the gitignored `pr-133-comment.md`** despite the runlog claiming "drafted" — moot this phase (gate 3 = no comment), but a recurring brief-compliance gap (also seen in retro #360).

### ## Cross-cutting

#### §1 — HEADLINE: the #292 stale-base-self-merge daemon bug (5× this phase — must be fixed upstream)

**This is the single most important finding of the phase.** Junior workers fork from a **stale daemon-local phase ref** (the daemon's local `phase-v1-AD-e` lagged `origin/phase-v1-AD-e`; merge-base repeatedly `c5622ec57`/`9adbfccf5`/`94915e07c`). The worker then self-merges the prior abandoned worker's branch and produces a tree that mass-deletes every advisor file landed after the stale base (766 / 985 / 1496 / 2596 / 2819 deletions observed across the 5 occurrences). **`base_branch=phase-v1-AD-e` on `create_task` does NOT prevent it** — the daemon forks from its local ref, which the arg does not refresh.

**Likely upstream cause:** the `RECOVERY-PASTE-elitedesk.md` `reset --hard` TOCTOU that corrupted daemon-local refs (flagged in retros #360/#361), compounding with `feedback_daemon_local_trunk_stale_multi_lane` (worker branches from daemon-LOCAL ref not origin).

**The recover recipe (reliable, now exercised 5×):**
1. Confirm: `git merge-base --is-ancestor <phase-tip> <worker>` → **NO** + `git diff --stat <phase-tip> <worker>` shows mass advisor-file deletions (the unambiguous tell).
2. If the worker branch is absent from origin (the #297 no-push variant): check the daemon reflog (`phase@{0}: merge ...`) + daemon object store; have the daemon push the containing SHA to an origin `recovery/<task>` ref (exposes objects, moves no daemon ref); fetch it.
3. Cherry-pick **ONLY** the clean `chore/fix` commit (verify `git show --stat` = the expected file set, NOT the stale-base merge) onto the live phase tip.
4. Resolve any runlog add/add conflict as a **UNION** (keep phase superset + append the worker's entry).
5. **Verify `git diff --stat <phase-tip>..HEAD` = ONLY the expected files, ZERO advisor-file deletions** before push.
6. Reconstruct any `validate-pending-laptop` DQ pointing at the **NEW on-branch SHA** (the worker's own DQ references its stale-base SHA — write fresh).

**Concrete upstream fix proposal (file a daemon-side ticket):** before each `git worktree add` for a task, the daemon MUST run `git fetch origin <phase>:<phase>` (fast-forward the daemon-local phase ref to origin) — OR fork the worktree directly from `origin/<phase>` instead of the local ref. Until this lands, **the advisor must budget the recover-cherry-pick as the EXPECTED path for every Junior task on a multi-lane phase** (it is no longer an exception — it was the norm for 100% of v1-AD-e workers). Pre-staging the recover recipe into each wakeup prompt (done from #299 onward) is the interim mitigation.

#### §2 — The cr-5 partial→gate-5→fix-impl-2 arc (a model for hard-ADR completeness handling)

cr-5 was a `severity: major` ADR-015 (GDPR/redaction hard-constraint) finding. fix-impl-1 addressed it **partially** (2 of N fields). The advisor: (a) caught the partial via diff-vs-brief-enumeration spot-check (not the worker summary), (b) did NOT adjudicate sufficiency itself (advisor never authors content; ADR-015 completeness is judgment-heavy), (c) surfaced it at user gate 5 with a sharp technical framing + 3 concrete options, (d) the user chose complete-first, (e) fix-impl-2 (#299) completed the full field set, (f) re-validated (DQ #246, full e2e) + re-verified (3/3) + re-surfaced gate 5. **This is the correct end-to-end handling of a known-partial against a hard ADR — record it as the reference pattern.**

#### §3 — bm-merge.md amendment proposal (required-check vs advisory-CR distinction)

The canonical `bm-merge.md` Phase 2.3 says any PENDING check → STOP, never "proceed anyway". This phase surfaced that **CodeRabbit's PENDING check is advisory** (governance-v0 has NO branch protection → CR is not a required check). The advisor correctly distinguished this and recorded the override as DQ #247. **Proposal: amend `bm-merge.md` Phase 2.3 to explicitly distinguish (a) a *required* status check pending → hard STOP, from (b) an *advisory* check (e.g. CodeRabbit on an unprotected branch) pending + user-confirmed at gate 5 → proceed-with-DQ-log.** Without this, every future phase re-derives the same reconciliation.

---

## Per-task complexity scores

`<files-touched> / <commits> / <runtime-min> / <max-log-silence-min>` per Junior task. Runtime from the daemon `jobs` table.

| Task | Role | Files | Commits | Runtime (min) | Max silence | Notes |
|---|---|---|---|---|---|---|
| #287 task 1 (maud dep) | impl | ~2 | 1 | ~7 | low | Clean one-pass. |
| #288 task 2 (gather_dashboard + dashboard HTML + route) | impl | ~5 | 1 | ~24 | low | Largest impl task; Task-4 scope-bleed (intentional, verified). |
| #289 task 3 (/audit/view + admin_audit_html import) | impl | ~3 | 1 | ~14 | low | Clean. |
| #290 task 5 (e2e HTML page tests) | impl | ~2 | 1 | ~14 | low | Clean; 93/0/5 at ship. |
| #292 bm-pr | bm | YAML+runlog | 1 | ~4 | n/a | **#292 stale-base rescue #1** (-1910 lines if merged). |
| #295 bm-poll-cr | bm | YAML+runlog | 1 | ~3 | n/a | **#292 rescue #2** (-766). |
| #296 bm-triage | bm | YAML+runlog | 1 | ~5 | n/a | **#292 rescue #3** (-985). Triage sound. |
| #297 fix-impl-1 (cr-1/4/5-partial/6) | impl | 3 | 1 | ~11 | low | **#292 rescue #4 + NEW #297 worker-no-push** (-1496). cr-5 silent-partial. |
| #299 fix-impl-2 (cr-5 completion) | impl | 1 | 1 | ~9 | low | **#292 rescue #5** (-2596). Fix exactly brief §2.2. |
| #301 bm-merge | bm | runlog | 0 (skipped) | ~4 | n/a | **False-failed at daemon finalize; merge SUCCEEDED server-side.** L14 advisor re-apply fired. |

**Aggregate:** 4 impl tasks (~59min impl runtime) + 6 BM/fix-in-PR legs. **5 of 6 non-core-impl Junior workers required a manual #292 rescue.** The implementation itself was low-complexity / low-silence / one-pass; the phase cost was rescue overhead, not impl difficulty.

---

## Lesson candidates (promote at sign-off — all meet/exceed threshold)

1. **#292 stale-base-self-merge recover recipe + the #297 worker-exited-without-push variant.** 5× this phase + a NEW distinct no-push mode. The recipe (Cross-cutting §1) is reliable. **Action at sign-off:** check whether `feedback_daemon_local_trunk_stale_multi_lane` / `feedback_junior_finalize_skips_when_worker_pre_pushes` already cover enough and just need the recover-recipe + no-push-variant appended, vs a dedicated lesson (`feedback_junior_292_stale_base_recover_recipe`). Either way, also file a **daemon-side fix ticket** (fetch origin `<phase>:<phase>` before worktree create, or fork from `origin/<phase>`).
2. **impl-task silent-partial on enumerated-transform briefs.** When a brief enumerates N fields/sites to transform, a worker can transform the obvious subset, file no blocker, and self-report "done". The advisor's post-task **diff-vs-brief-enumeration spot-check** (not the worker summary) is the catch. **Action:** brief templates for enumerated-transform tasks should require all-N-or-blocker; consider a `feedback_impl_task_enumerated_transform_all_or_blocker` lesson. Concrete instance: cr-5 partial (#297).
3. **Windows: author DQ/JSON values containing backslash paths via a Write-tool JSON fragment + read-merge, NEVER inline `python -c` with the path in the source.** `\b`/`\c`/`\'` in Python source mangle silently (`scripts\brehon` → `scriptsrehon`). Distinct from `feedback_windows_bash_python_git_show_tmp_traps` (that's git-show path-mangling; this is Python-source backslash-in-string-literal). Concrete instance: DQ #246 reconstruction (2 bugs, ~3 wasted tool calls).

Additionally **carried (recurring, promote if it hits a 3rd phase):** `/brehon-verify` descriptor-grammar limitation — idiomatic actix `scope("/x").route("/y",...)` trips the literal `.route("/x/y"` grep-descriptor → false candidate-phantom. (Retro #362 flagged; reproduced this phase.)

---

## Pipeline state at retro

- PR #133 **MERGED** `486a24c68`; `phase-v1-AD-e` deleted from origin; governance-v0 trunk advanced.
- L14 runlog re-apply pushed (`16e8b2c04`). All 6 CR findings resolved (4 done / 1 rebut / 1 wont-fix, 0 open). DQ pending = 0.
- `/brehon-verify` 3/3 ✓ (`b33fdff05`). DQ #245 + #246 both validate-pending-laptop result=pass (full e2e 94/0/5).
- Retros #358–#363 (the per-segment advisor evals) are the granular evidence chain behind this phase retro.
- **Next:** user gate 6 (retro sign-off) → on sign-off `/brehon-phase-transition`.
