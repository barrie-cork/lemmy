# v1-ship-1 Runlog

Append-only ledger of state-changing actions on the `phase-v1-ship-1`
lane. `bm:` lines = Branch Manager actions; `advisor:` lines = lane-advisor
orchestration actions. Impl/planning artifacts are tracked in git commits,
not here.

## bm: cut phase-v1-ship-1 off governance-v0 @ 8c271285e

bm-cut completed (Junior #274). `phase-v1-ship-1` created at
`8c271285e` (= governance-v0 tip; descends correctly via
`git merge-base --is-ancestor`). Pushed to origin. Plan
`.claude/PRPs/plans/v1-ship-1-r1.plan.md` APPROVED at User Gate 1
(2026-05-16). The original bm-cut runlog write fell back to
worktree-root on the worker branch (PMD#998 worktree-guard, NOT CC#10);
this file is the proper recreation on the lane worktree per bootstrap §0.

## advisor: lane session resumed 2026-05-16

Lane-dedicated advisor session resumed from
`.claude/PRPs/handovers/v1-ship-1-bootstrap.md`. CWD =
`C:/Users/barri/Developer/brehon-fork-ship-1`, branch =
`phase-v1-ship-1`. Pre-phase harness audit (4 wrapper probes) running.
Next: Task 0 pre-flight brief dispatch per plan §13.

## advisor: daemon-local ref synced + Task 0 dispatched 2026-05-16

Daemon-local `phase-v1-ship-1` was STALE (`bf7baa562`; daemon checkout
on `phase-v1-federation-inbound-a`). Lane-safe refspec-fetch
`git fetch origin phase-v1-ship-1:phase-v1-ship-1` → SYNC at
`3d098d5f2`; daemon checkout unchanged (federation-inbound-a lane
undisturbed) per the #273 lesson + `.claude/rules/multi-lane-worktree.md`.
Task 0 dispatched as Junior **#284**
(`[role:impl-task]`, `base_branch=phase-v1-ship-1` @ `3d098d5f2`).
Non-`[P]`, verification-only (12 probes, no commit). Awaiting complete.

## advisor: Task 0 (#284) PASS + Task 1 (#285) dispatched 2026-05-16

Junior #284 done (succeeded, 22:28:48→22:34:34, ~5m46s, no commit as
expected). Probe verdict (log read via general-purpose subagent per
the token-cap discipline): **all substantive probes PASS** —
Probe5 AGPL-NOTICE.md=3595B, Probe6 GetSiteResponse@337,
Probe7 captcha_enabled@69, Probe8 bootstrap count=**2** (DQ #226
dual-bootstrap sanity holds), Probe9 governance bootstrap@**801**
(exact, Task-4 Case-A target), Probe10 federated_instances@288,
Probe11 concurrent-PR empty (no collision), Probes 0/2/4 PASS.
**Advisor judgment call:** Probes 1 & 3 (brief-designated *blocker*
probes) were self-downgraded to WARN by the worker — Probe 1
(`branch==phase-v1-ship-1`) is structurally unsatisfiable for any
Junior worktree (`junior/*` branch by design); Probe 3
(`merge-base --is-ancestor governance-v0 HEAD`) WARNs because
governance-v0 advanced 5 meta-commits post-bm-cut (expected
multi-lane topology). Worker self-resolved from rules corpus
(`multi-lane-worktree.md` + `feedback_parallel_agents_one_worktree_per_agent`);
reasoning is defensible — substantive topology (descends from correct
bm-cut tip, all anchors present) is green. Proceeded to Task 1, no
catch-fire, no DQ. **Retro watch-item:** Task 0 brief Probe 1/3
wording needs a Junior-worktree-aware revision for future ship-*
phases (literal `git branch --show-current == phase-v1-ship-1` /
direct-ancestor checks can never pass on a Junior worktree branch
post-cut). Verify-before-trust confirmed: canonical resolver shows
only #229 pending (no #284 blocker); worker branch absent from origin
(expected — no commit/push on a verification-only task).
Daemon-local `phase-v1-ship-1` re-synced `3d098d5f2→6661fc827`
(lane-safe refspec-fetch; daemon checkout still federation-inbound-a).
Task 1 (DTOs solo barrier) dispatched as Junior **#285**
(`base_branch=phase-v1-ship-1` @ `6661fc827`). Awaiting complete +
§5.2 validate-pending-laptop.

## advisor: Task 1 (#285) verified + finalize-merged + §5.2 running 2026-05-16

Junior #285 done (succeeded, 22:42:03→22:44:03, ~2 min). Verify-before-trust
(bootstrap §4): worker branch `git fetch` + `git log` confirmed TWO
commits — `ab897de07` `feat(db_views_site): add SourceDisclosure,
GetSource, GetSourceResponse DTOs (task 1)` (+36 lines api.rs, 3 structs)
and `239256729` `chore(decision-queue): impl raised DQ #238 —
v1-ship-1 task1 validate-pending-laptop`. DQ #238 read via short-SHA-safe
`git show <short-sha>:path` (slashed-ref trap avoided per
`feedback_windows_bash_python_git_show_tmp_traps`): `kind:
"validate-pending-laptop"`, `from: "impl"`, `result: null`, 3 commands
(cargo check/clippy/test-no-run, `--workspace --features full`).
**Daemon did NOT finalize-merge** (worker pre-pushed → finalize skips,
`feedback_junior_finalize_skips_when_worker_pre_pushes`); advisor
manual `--no-ff` finalize-merge → `97850b363` (conflict-free: worker
api.rs+DQ vs phase runlog disjoint; task-per-commit history preserved;
3 DTOs confirmed present via grep -c). Pushed `599139a94..97850b363`.
Forbidden-window clear (22:53 UTC Sat, primary window). §5.2
validate-pending-laptop running in background (chain ID `bnichsihk`):
cargo-check.bat → cargo-clippy.bat → cargo-test.bat --no-run, chained
with short-circuit, logs at `C:/Users/barri/.claude/logs/validate-laptop-238-cmd{1,2,3}.log`
+ verdict file (Shape G suspended; bat wrappers per
`feedback_windows_e2e_requires_bat_wrapper`). On ALL_PASS → mutate
DQ #238 `answered_by:"advisor-laptop"` `result:"pass"` → resolved →
daemon-sync → Cohort A (Tasks 2+3 parallel, file-disjoint).

NOTE (carry-forward): project-memory MCP handle still HELD (PID
23336/33172); junction fix (task #6) remains deferred to next clean
MCP-disconnect. Non-blocking for orchestration.

## advisor: §5.2 chain-invocation trap + corrected re-run 2026-05-16

First §5.2 attempt (bg chain `bnichsihk`) returned "completed exit 0"
in ~seconds with ZERO logs + no verdict file — exit-summary lied
(`feedback_task_notification_exit_summary_unreliable` +
`feedback_background_task_notification_lies`). RCA: packed 3 `.bat`
invocations + nested `()` + `>>` redirects into ONE `cmd //c` string;
calling a `.bat` from `cmd /c` WITHOUT `call` transfers control and
never returns (classic Windows batch trap) — cargo-check.bat was
entered but cmd never came back to write logs/run cmd2/cmd3. **LESSON
(reusable, §5.2 on Windows):** run each cargo command as its OWN
background Bash call: `cmd //c "cd /d <repo-abs> && call
scripts\brehon\cargo-X.bat ... > log 2>&1 && echo EXIT_0>>log || echo
EXIT_NONZERO>>log"` — one cmd//c per command, explicit `call`, explicit
`cd /d`, dedicated log+marker. NEVER a mega-chain. Corrected: cmd1
(cargo-check) re-dispatched as bg `bdwymbji2`; cmd2 (clippy)/cmd3
(test-no-run) chained sequentially only AFTER reading each prior log's
EXIT marker (not trusting the bg-completion summary). → candidate for a
new lesson `feedback_win_bat_chain_needs_call_per_cmd` at retro.

## advisor: §5.2 cmd1 PASS, cmd2 (clippy) running 2026-05-17

cargo-check (bg `bdwymbji2`) PASS — verified by EVIDENCE not bg-summary
(trap discipline): `CMD1_EXIT_0` marker present, `Finished dev profile
in 8m 07s` (clean through lemmy_server), 0 `error[E*]`/`error:` lines.
DTOs compile clean. cmd2 (cargo-clippy `--workspace --features full
--no-deps -- -D warnings`) dispatched bg `bvhmvth60` (same corrected
one-cmd//c+call pattern; target/ warm). Awaiting CMD2 EXIT marker;
cmd3 (test --no-run) chains only on CMD2_EXIT_0.

## advisor: §5.2 cmd2 (clippy) PASS, cmd3 (test --no-run) running 2026-05-17

cargo-clippy (bg `bvhmvth60`) PASS — EVIDENCE: `CMD2_EXIT_0` marker,
`Finished dev profile in 5m 36s` (clean through lemmy_server), **0
warnings 0 errors** (no `-D warnings` denials). DTOs clippy-clean
(verbatim §10.1 bodies as expected). cmd3 (cargo-test `--no-run -p
lemmy_server --test e2e`) dispatched bg `bauk7u7j7` — compiles e2e
binary only, no Docker/testcontainers. On `CMD3_EXIT_0` → all 3 §5.2
commands pass → mutate DQ #238 `answered_by:"advisor-laptop"`
`result:"pass"` → resolved → daemon-sync → Cohort A.

## advisor: §5.2 ALL PASS, DQ #238 resolved, Cohort A next 2026-05-17

cargo-test --no-run (bg `bauk7u7j7`) PASS — EVIDENCE: `CMD3_EXIT_0`,
`Finished test profile in 8m07s`, `Executable tests\e2e.rs
(target\debug\deps\e2e-6c3527550c4cf788.exe)` built, 0 errors.
**ALL 3 §5.2 commands PASS** (check 8m07s/0err · clippy 5m36s/0warn·err
· test-no-run 8m07s/e2e-exe-built). Task 1 DTOs fully validated vs
phase tip 7f58e6cf6. DQ #238 mutated `pending→resolved`
(`result:"pass"`, `answered_by:"advisor-laptop"`, resolved_at
2026-05-16T23:19:22Z) via Temp script + `python <file>` (Windows
inline-python trap avoided). Pending now `[229]` only (expected
Shape-G-reenable log). **Task 1 (solo DTO barrier) COMPLETE.** Next:
daemon-local-trunk-sync → author + dispatch Cohort A (Tasks 2+3,
file-disjoint `[P]`, parallel `create_task` single message; verify
§11 + FILES YAML overlap + `requires:`Task1-on-phase-branch first).

## advisor: INCIDENT — cross-lane reset --hard via TOCTOU race 2026-05-17

**Severity:** high (cross-lane corruption) / **Data loss:** ZERO /
**Origin impact:** NONE (daemon-local only; nothing pushed).

**Sequence:** Cohort-A pre-dispatch daemon-sync. Pre-flight found
daemon `/srv/brehon-fork` checkout had DIVERGED on `phase-v1-ship-1`
(daemon's own redundant finalize-merge `0decd9971` of #285 + untracked
`allow-prp-deliverables.sh` bootstrap-§3 hook). Provenance investigated
+ user-authorized a `stash -u → reset --hard origin/phase-v1-ship-1 →
stash pop`. **TOCTOU race:** between investigation and command, the
Junior daemon ROTATED the shared checkout `phase-v1-ship-1 →
phase-v1-AD-e` (daemon cycles lanes for concurrent processing). The
`reset --hard origin/phase-v1-ship-1` therefore landed on the WRONG
branch — moved `phase-v1-AD-e` ref `32dd62b1a → 6c03b9e97`. Then the
daemon's bm-pr finalize #292 ran @08:16:56 (before the daemon stop
took effect @08:17:47), building `8f6387e9a` (real AD-e bm-pr payload
3587 lines) on the corrupted `6c03b9e97` base = cross-lane DQ #238
contamination fused into AD-e local.

**Containment:** AD-e is HEALTHY on origin — PR #133 OPEN
(`headRefOid c5622ec5`, awaiting CodeRabbit), driven by
`origin/phase-v1-AD-e=eddc8ab5d`. Corruption was DAEMON-LOCAL ONLY
(`8f6387e9a`/`32dd62b1a` never pushed, never fed PR #133).
`origin/AD-e` already had the task-5 work; daemon-local `32dd62b1a`
was a redundant local finalize-merge. All objects reachable
(`32dd62b1a`/`8f6387e9a`/`#292 branch 0a50803e5`/`origin eddc8ab5d`).

**Recovery (user-authorized, user-executed at EliteDesk; advisor
read-only verify):** daemon STOPPED (`systemctl stop
junior@brehon-fork.service`; siblings dog-shelter/food-producer
untouched; app exited 0/SUCCESS, "failed" = cosmetic SIGTERM).
CAS-guarded `git update-ref` (race-free, no checkout touch):
(a) `phase-v1-AD-e` → `origin/phase-v1-AD-e` (discard contaminated
`8f6387e9a` + redundant `32dd62b1a`; CAS from `8f6387e9a`);
(b) `phase-v1-ship-1` → `origin/phase-v1-ship-1` (the original sync
goal; CAS from `0decd9971`). First CAS-guarded attempt CORRECTLY
ABORTED (`is at 8f6387e9a but expected 6c03b9e97`) — the guard caught
the daemon's #292 merge; re-investigated before retrying. Untracked
hook preserved (sha256 unchanged across all ops).

**LESSONS (→ Task 5 retro §5 + new lesson candidates):**
1. `feedback_daemon_shared_checkout_toctou_race` — the daemon's
   single rotating `/srv/brehon-fork` checkout means ANY advisor
   `git checkout`/`reset --hard`/`merge` against a daemon-local
   *branch* is a TOCTOU hazard: the daemon may rotate the checkout
   between the advisor's read and write. ONLY `git update-ref
   <ref> <new> <old-CAS>` (atomic, checkout-independent, CAS-guarded)
   is safe against daemon-local refs. NEVER `reset --hard
   origin/<X>` when the daemon may be on a different branch — it
   resets whatever is checked out, not `<X>`. The lane-safe
   refspec-fetch (`git fetch origin X:X`) is safe ONLY when `<X>` is
   not the checked-out branch; when it might be, pause the daemon
   first OR use update-ref.
2. CAS-guarded `update-ref` (`git update-ref ref new old`) is the
   mandatory primitive for ALL advisor daemon-local ref mutations —
   it makes the TOCTOU class structurally impossible (refuses on
   unexpected current value) where `reset --hard` blindly overwrites.
3. Pausing the daemon (`systemctl stop junior@<repo>.service`) before
   ANY shared-checkout git surgery is mandatory, not optional —
   spatial isolation is insufficient against a rotating checkout;
   temporal isolation (daemon down) is required.
4. Multi-lane discipline gap: `.claude/rules/multi-lane-worktree.md`
   covers the HUMAN-side worktree-per-lane but the DAEMON side still
   uses one rotating checkout. The daemon-side analog (worktree per
   active phase OR a hard "advisor never mutates daemon-local
   branch refs except via paused-daemon CAS-update-ref") needs a
   rule. → propose at retro.

**Status:** recovery commands handed to user; awaiting EliteDesk
execution + output. Daemon stays STOPPED until refs verified. v1-ship-1
Cohort A dispatch BLOCKED until daemon resumed + ship-1 ref re-verified
with the tighter anti-TOCTOU procedure (pause-daemon-then-update-ref,
never reset --hard).

## advisor: INCIDENT RECOVERED — daemon checkout clean 2026-05-17

**RESOLVED. Zero data loss. Zero origin impact.** Recovery executed by
USER at EliteDesk terminal (advisor read-only verify throughout; the
auto-mode classifier correctly blocked advisor SSH `reset --hard` on
shared infra post-incident — required human execution or a Bash
permission rule; AskUserQuestion is NOT classifier-recognized auth for
a destructive shared-infra shell command, per bootstrap §3 precedent).

Sequence: (1) USER ran CAS-guarded `git update-ref refs/heads/phase-v1-AD-e
<origin> 8f6387e9a` + `git update-ref refs/heads/phase-v1-ship-1
<origin> 0decd9971` at EliteDesk (refs → origin; discarded contaminated
`8f6387e9a` + redundant `32dd62b1a`/`0decd9971`). (2) `update-ref`
deliberately doesn't touch the worktree → `staged=12` artifact delta
remained. (3) One mis-target: USER ran `reset --hard HEAD` on the
LAPTOP canonical `brehon-fork` (governance-v0) first — HARMLESS no-op
(HEAD→HEAD, only untracked files present, nothing tracked-uncommitted
to lose; canonical checkout still `8479b4c95==origin/governance-v0`).
Distinction surfaced: `C:\…\brehon-fork` (laptop, governance-v0,
canonical) ≠ `/srv/brehon-fork` (EliteDesk, daemon rotating checkout).
(4) USER re-ran `cd /srv/brehon-fork && git reset --hard HEAD` on the
EliteDesk (no ref move — HEAD stayed `c5622ec57`; cleared the stale
staged delta to match the already-correct ref; untracked hook
preserved).

**Final verified state (advisor read-only):** EliteDesk `/srv/brehon-fork`
on `phase-v1-AD-e`, HEAD `c5622ec57`; `phase-v1-AD-e == origin`
(`c5622ec57` = PR #133 head — AD-e healthy, awaiting CR);
`phase-v1-ship-1 == origin` (`de1e92192`); `staged=0 worktree=0`;
`allow-prp-deliverables.sh` untracked-preserved (sha `9b08464e…`);
`git status --porcelain` shows ONLY the untracked hook; daemon STOPPED.
AD-e PR #133 never touched (origin-driven; my corruption was
daemon-local-only, never pushed). 4 incident lessons stand (above) for
Task 5 retro. Next: resume daemon → re-verify ship-1 daemon-local sync
via SAFE pattern (pause-daemon + CAS-`update-ref`, NEVER `reset --hard
origin/<branch>`) → dispatch Cohort A.

## advisor: post-incident resume + Cohort A dispatched 2026-05-17

Daemon resumed by USER (`systemctl reset-failed + start
junior@brehon-fork.service`; classifier correctly required user
execution — not advisor SSH). Verified healthy (new PID 2559894,
Active=0 Queued=0). Daemon-local `phase-v1-ship-1` re-synced via the
SAFE anti-TOCTOU primitive: read-only check → GUARD
(`checked-out=phase-v1-AD-e ≠ phase-v1-ship-1` → SAFE-REFSPEC-FETCH) →
`git fetch origin phase-v1-ship-1:phase-v1-ship-1` (NOT `reset --hard`;
no checkout switch; AD-e undisturbed). Two sync cycles
(`de1e92192→c8a689778`, `c8a689778→c13e5a35b`) both clean ff (behind 1
ahead 0 each — no daemon-local divergence post-recovery). Cohort A
briefs authored (`v1-ship-1-impl-{2,3}.md`, committed `c13e5a35b`;
user explicitly re-authorized the Task 3 brief write after a
classifier interrupt-then-retry block — AskUserQuestion answer was not
classifier-sufficient, plain explicit instruction was). §11 verified
file-disjoint (`intersect(T2{api.rs,read.rs,build.rs},
T3{source.rs,mod.rs,lib.rs})=∅`); both `requires: task 1`
(`ab897de07` on phase branch). No §2.4 mandatory row. Forbidden-window
clear. **Cohort A dispatched SIMULTANEOUSLY** (parallel `create_task`,
single message): Task 2 = Junior **#293**, Task 3 = Junior **#294**
(both `base_branch=phase-v1-ship-1` @ `c13e5a35b`, both running,
correct worktree branches). Await BOTH → finalize-merge each (daemon
may skip if pre-pushed) → §5.2 validate-pending-laptop ×2 SERIALLY
(shared `target/`) → cohort barrier clears on both pass → Task 4 →
Task 5.

## advisor: Cohort A reconciled (#293+#294 merged, DQ collision fixed) 2026-05-17

Both Cohort A workers `done` (single run each, succeeded): #293 task2
(`62cf0344a feat(api_crud): wire source_disclosure into GetSiteResponse
+ build.rs`), #294 task3 (`c4e5dcdf6 feat(api): add get_source handler
+ /api/v4/source route`). Daemon did NOT auto-finalize-merge (Active=0,
phase tip stayed `6e169f4d4`) — advisor-side reconcile per
`feedback_junior_finalize_skips_when_worker_pre_pushes` (workers did not
pre-push either; pure advisor merge). **Verify-before-trust:** fetched
both worker branches, `git diff --stat` confirmed each matches its brief
EXACTLY (T2={build.rs,read.rs,api.rs} +29/-1; T3={mod.rs,source.rs,
lib.rs} +20); zero file overlap (cohort disjointness held).

**DQ id collision (parallel-cohort, DQ #50 class):** both workers
independently computed `next_id` against their own worktree view (each
forked from `c13e5a35b`, max id #238) → BOTH raised `id: 239`. Resolved
during merge: Task 2's validate-pending-laptop KEPT #239 (lower task
no. wins, deterministic); Task 3's RENUMBERED to #240. Merge sequence on
THIS lane worktree (`brehon-fork-ship-1`, `phase-v1-ship-1`):
`--no-ff origin/...task-2...` → clean (`e9a6a3864`); `--no-ff
origin/...task-3...` → conflict ONLY on `.claude/decision-queue.json`
(code files disjoint, auto-merged clean) → resolved via stage-2/stage-3
Python merge (ours-base + theirs' task3 entry renumbered 239→240;
asserted resolved[] identical across stages, asserted final id-
uniqueness) → committed `6ceb7683a`. Pushed `6e169f4d4..6ceb7683a`
(local==origin). Merged tree verified: all 6 code files present,
combined code diff = exactly 6 files +49/-1, DQ pending=[229,239,240]
no dup ids across full 220+ id space.

NEXT: §5.2 validate-pending-laptop. Both #239+#240 carry IDENTICAL
commands (cargo-check / cargo-clippy / cargo-test --no-run -p
lemmy_server --test e2e) and validate the SAME merged tip `6ceb7683a`
→ run §5.2 cargo chain ONCE on the combined tip, apply result to BOTH
DQ entries (avoids redundant ~22-min second run on identical tree;
§5.2 serial-cargo constraint satisfied trivially since it's one run).
Then cohort barrier clears on both pass → Task 4 (e2e, requires 2+3) →
Task 5 retro.

## advisor: §5.2 cmd2 FAIL — clippy::unused_async (CATCH-FIRE) 2026-05-17

§5.2 validate-pending-laptop on Cohort A merged tip `6ceb7683a`
(canonical checkout detached@`5eb001512`, code identical):
- cmd1 `cargo-check --workspace --features full`: **PASS** (4m12s, 0
  err/warn; CMD1_EXIT_0 marker authoritative). Both Task 2 + Task 3
  crates compile across full workspace.
- cmd2 `cargo-clippy --workspace --features full --no-deps -- -D
  warnings`: **FAIL** (CMD2_EXIT_NONZERO). `error: unused 'async' for
  function with no await statements --> crates/api/api/src/site/
  source.rs:11` — Task 3's `get_source` declared `async` but body is
  `Ok(Json(GetSourceResponse{...}))` with an `include_str!` const, no
  `.await`. `-D clippy::unused-async` makes it fatal; `could not
  compile lemmy_api`.
- cmd3 `cargo-test --no-run -p lemmy_server --test e2e`: **NOT RUN**
  (serial `&&` chain correctly stopped at cmd2 non-zero).

**§G4 classifier verdict: NON-ALLOWLIST → CATCH-FIRE.**
`clippy::unused_async` is not in the §5.3 §G4 allowlist (allowlist =
doc_lazy_continuation, E0432, deprecated-api, E0277 LemmyError 4a/4b/4c,
trait-bound-with-lesson, map_err_ignore, missing-macro, E0599). Per
"Any failure whose log slice doesn't match an allowlist row →
Catch-fire" + "allowlist is conservative by design; grow only on retro
evidence". NO auto-fix queued; surfaced to user.

DQ mutated + pushed (`0d13a89bf`): **#240** (Task 3) `result:fail`,
stays `pending[]` for triage, log_slice + recommended fix recorded,
`answered_by:advisor-laptop`. **#239** (Task 2) `result:null` (NOT a
fail — cmd1 passed for its crates; cmd2/cmd3 unreached due to #240's
abort on the shared merged tip), `context` annotated BLOCKED-BY-COHORT.
Re-running §5.2 on the post-fix merged tip clears BOTH #239 and #240.

Recommended fix (mechanical, clippy-suggested, ≤1 file): remove `async`
from `pub fn get_source() -> LemmyResult<Json<GetSourceResponse>>` in
`crates/api/api/src/site/source.rs:11`. Cross-file check: route is
`.route("/source", get().to(get_source))` in `crates/api/routes/src/
lib.rs` — actix `.to()` accepts sync handlers, so de-async is
registration-safe (NO change needed in lib.rs). e2e test (Task 4, not
yet authored) would call the endpoint over HTTP — sync vs async handler
is transparent to the client. AWAITING USER: authorize narrow
fix-impl-task vs add clippy::unused_async to §G4 allowlist vs other.

## advisor: ONE-TIME four-role deviation — de-async fix (user-authorized) 2026-05-17

§5.2 cmd2 catch-fire resolution. User chose "I'll fix it inline myself"
→ then "Explicitly authorize me, this once" (AskUserQuestion, explicit
instruction per four-role-model deviation requirement). Advisor applied
+ commits the SINGLE mechanical edit: `crates/api/api/src/site/
source.rs:11` `pub async fn get_source` → `pub fn get_source` (remove
`async`; clippy::unused_async; body has no `.await`). NO other content
authored. This is a documented ONE-TIME deviation from "advisor never
authors crates/**" — user-instructed, recorded here for audit. The
Claude Code auto-mode classifier correctly blocked the FIRST commit
attempt (user had not yet given explicit per-deviation instruction); the
second attempt proceeds under the explicit authorization above. Next:
re-run §5.2 full chain on canonical checkout at the post-fix tip →
verify compile + clippy (compiler-verified, NOT assumed: route
`.route("/source", get().to(get_source))` in routes/src/lib.rs — actix
`.to()` sync-handler acceptance to be CONFIRMED by the green clippy run,
not by assertion) → mutate BOTH #239 + #240 result:pass → cohort
barrier clears → Task 4 (e2e).

## advisor: de-async REVERTED — actix Handler requires async (#[expect] applied) 2026-05-17

§5.2 re-run cmd1 on f18308092 FAILED: `error[E0277]: the trait bound
{get_source}: Handler<_> is not satisfied --> crates/api/routes/src/
lib.rs:233` (`could not compile lemmy_api_routes`). ADVISOR ERROR
ACKNOWLEDGED: the prior de-async fix (f18308092) was WRONG — I asserted
"actix .to() accepts sync handlers" WITHOUT compiler verification
(violates `feedback_verify_automated_reviewer_claims_against_compiler`
— applies to advisor's own framework claims, not just CR/Copilot).
actix-web's `Handler<Args>` is implemented ONLY for handlers returning
a `Future`; the `async` on `get_source` is STRUCTURALLY REQUIRED by the
route registration even though the body has no `.await`. Junior #294's
original `async fn` was CORRECT; `clippy::unused_async` is a
false-positive in this actix-handler context.

Corrected fix (user-authorized continuation of the one-time deviation,
AskUserQuestion 2026-05-17 "Authorize me: revert + apply #[expect]"):
restored `pub async fn get_source` (= Junior c4e5dcdf6 original) +
added canonical `#[expect(clippy::unused_async, reason="actix-web
Handler trait ... structurally required ... E0277 without it")]`.
CANONICAL PRECEDENT followed: `crates/server/tests/e2e.rs:8025`
`seed_case` uses the identical `#[expect(clippy::unused_async,
reason=...)]` shape (per `feedback_read_canonical_before_writing_spec`
— read sibling instance first). `unused_async = "deny"` is a
`[workspace.lints.clippy]` entry (Cargo.toml:106), so `#[expect]` is
the correct workspace-consistent suppression. Diff vs c4e5dcdf6 = ONLY
the #[expect] block (async restored, nothing else). Next: re-run §5.2
full chain on the new tip — COMPILER-verified this time (cmd1 must
pass E0277-clean; cmd2 clippy must pass with #[expect] consuming the
lint; cmd3 test --no-run) → mutate BOTH #239 + #240 result:pass.

RETRO CARRY (Task 5 §5): advisor committed an unverified framework
assumption (de-async) → 1 wasted §5.2 cycle (~5 min warm) + 1 extra
commit + classifier-block. Lesson: framework-trait claims (actix
Handler, Diesel DSL, etc.) are hypotheses — compile-verify on the lane
BEFORE commit, never assert-then-commit. Ties to PR #132 cr-2 precedent
in `feedback_verify_automated_reviewer_claims_against_compiler`.

## advisor: §5.2 r3 ALL 3 PASS — Cohort A barrier CLEAR 2026-05-17

Corrected tip `48435569d` fully validated (compiler+clippy+e2e-compile,
all EXIT markers authoritative, not bg summaries):
- cmd1 cargo-check --workspace --features full: PASS 1m29s, E0277 gone.
- cmd2 cargo-clippy --workspace --features full --no-deps -- -D
  warnings: PASS 2m31s, 0 warnings (#[expect(clippy::unused_async)]
  consumed lint, no unfulfilled_lint_expectations).
- cmd3 cargo-test --no-run -p lemmy_server --test e2e: PASS 9m24s,
  e2e-aebb6c76db6b1818.exe built, 0 err.

DQ #239 (Task 2) + #240 (Task 3) both mutated → result:pass,
answered_by:advisor-laptop, moved to resolved[]; #239's stale
blocked-by-cohort context annotation stripped (now genuinely pass).
Committed+pushed `cbe0fa779`. pending[] now only [#229] (Shape-G
reenable reminder — not a blocker). **Cohort A barrier CLEAR** —
Tasks 2+3 delivered + validated on the merged+fixed phase tip.

NEXT: Task 4 (e2e test, plan §13; `requires:` tasks 2+3 — both now on
phase branch). Author brief .claude/PRPs/briefs/v1-ship-1-impl-4.md
(§2.4 MANDATORY e2e.rs lessons: feedback_lemmy_error_no_std_error +
feedback_async_pool_test_pattern; +feedback_junior_worker_e2e_edit_hang
if ≥2 edits; mirror v1-SL/v1-JM sibling fixtures error-shape case
A/B per canonical-schema-first gate) → §2.3 PMD presearch → dispatch
[role:impl-task] Junior base_branch=phase-v1-ship-1 (daemon-local sync
via safe anti-TOCTOU first). Then §5.2 validate Task 4 (e2e RUN — needs
Docker; testcontainers) → /brehon-verify → bm-pr → CR → triage → merge
→ Task 5 retro.

## advisor: Task 4 dispatch DEFERRED — daemon busy on AD-e #297 (cross-lane serialization) 2026-05-17

Task 4 brief authored + pushed (`123b75c05`, .claude/PRPs/briefs/
v1-ship-1-impl-4.md). Pre-dispatch SAFE anti-TOCTOU daemon-local check
revealed TWO blockers for an immediate dispatch:

1. **Daemon-local `phase-v1-ship-1` diverged (behind 3 / ahead 11 vs
   origin).** The "ahead 3" = STALE daemon finalize-merge commits
   `08363bf2c`(task2 fm) / `f5d0c2639`(merge) / `9082b08d3`(task3 fm),
   merge-base `e134c6d9d`. These are the daemon's OWN finalize-merge of
   Tasks 2+3, created daemon-local but NEVER pushed (origin doesn't
   have them) — a DEAD PARALLEL PATH superseded by the advisor reconcile
   (origin `123b75c05` already has Tasks 2+3 code: verified `get_source`
   ×2 in source.rs, `BREHON_FORK_COMMIT` ×3 in build.rs — PLUS the
   clippy de-async fix, DQ #239→#240 reconcile, pass mutations, Task 4
   brief, all of which the stale commits LACK). This is the
   daemon-local-trunk-stale pattern (`feedback_daemon_local_trunk_stale_
   multi_lane` / lesson #273), NOT the AD-e TOCTOU corruption class.
   Discarding the 3 stale commits loses nothing on origin.

2. **Daemon is RUNNING task #297** = `[role:impl-task] v1-AD-e
   fix-impl-1` (PR #133 CR fix-in-PR, the OTHER lane, baseBranch=
   phase-v1-AD-e). Daemon checkout is on phase-v1-AD-e; AD-e
   daemon-local ahead of origin because #297 is actively building.

DECISION: **DEFER Task 4 dispatch until #297 completes + daemon idle.**
Rationale: (a) a force `git update-ref` on daemon-local ship-1 while a
peer worker (#297) is live on the shared /srv/brehon-fork/.git is the
exact TOCTOU race that corrupted AD-e (2026-05-17 incident) — must NOT
repeat; (b) pausing the daemon would kill #297 (peer lane's live CR
fix) — unacceptable cross-lane interference; (c) dispatching now would
branch the Task-4 worker from STALE `9082b08d3` (missing the Task 4
brief itself + clippy fix + DQ reconcile) → guaranteed-broken worker.
This is NORMAL cross-lane serialization (two lanes, one daemon
checkout — ref ops serialize behind the active lane's worker), NOT a
catch-fire. Poll #297; when done + daemon Active=0: re-run the SAFE
anti-TOCTOU check, CAS-guarded `update-ref` daemon-local ship-1 →
origin/phase-v1-ship-1 (guarded from 9082b08d3; daemon idle so no
concurrent rotation), THEN dispatch Task 4. NO reset --hard. NO daemon
pause. RETRO CARRY (Task 5 §5): daemon-side multi-lane ref-isolation
gap — two lanes sharing one /srv/brehon-fork checkout forces this
serialization + makes every cross-lane daemon-local sync a TOCTOU
hazard; candidate structural fix = per-lane daemon checkout (mirror of
the human-side worktree-per-lane rule). Ties to the 2026-05-17 AD-e
incident lessons already carried.

## advisor: peer-lane #297 done → daemon-local ship-1 CAS-synced → Task 4 dispatched (#298) 2026-05-17

Peer-lane #297 (v1-AD-e fix-impl-1) DONE (run succeeded 11:00:31).
Daemon idle (Active=0, Queued=0). Cross-lane serialization window
closed. FRESH SAFE anti-TOCTOU read-only check: daemon checked-out
still phase-v1-AD-e (ship-1 not the checkout — GUARD OK);
daemon-local ship-1 still STALE `9082b08d3` (CAS-from re-verified
fresh, NOT carried-stale; #297 was AD-e so didn't touch ship-1 ref);
origin ship-1 = `88148edd0` (authoritative); behind 3/ahead 12.

USER-AUTHORIZED CAS-guarded `git update-ref refs/heads/phase-v1-ship-1
<origin-sha> 9082b08d382117d89fc704ac3ec84ada3d84a2f8` (AskUserQuestion
"Authorize me: CAS update-ref now"). EXIT=0, CAS guard passed (ref was
exactly 9082b08d3, no concurrent move). POST: daemon-local ship-1
`9082b08d3`→`88148edd0` = origin → SHIP-1 SYNCED OK. AD-e UNTOUCHED
(94915e07c unchanged). checked-out still phase-v1-AD-e (NO checkout
switch, NO reset --hard, NO daemon pause — daemon was idle so the safe
CAS primitive sufficed). 3 stale dead-end finalize-merge commits
orphaned (unreferenced/unpushed; content already on origin via advisor
reconcile — zero loss).

Pre-dispatch re-verify: daemon STILL idle (no race window),
daemon-local ship-1 stable `88148edd0`, Task 4 brief + plan §13 Task 4
both readable on daemon-local ship-1. **Task 4 DISPATCHED** = Junior
**#298** (`base_branch=phase-v1-ship-1` @ `88148edd0`, branch
junior/role-impl-task-v1-ship-1-task-4-...-298, status running, single
run, picked up immediately). NEXT: await #298 → verify-before-trust
(worker commit `test(e2e): assert AGPL §13 disclosure surface ...
(task 4)` + validate-pending-laptop DQ on worker branch) →
finalize-merge → §5.2 validate-pending-laptop (§15.1-3 workspace cmds
LOCALLY — Shape G suspended DQ #229) → e2e RUN = Phase-2 user-gate-4
(local vs dispatch) → /brehon-verify §16a Story 3 → bm-pr → CR →
bm-triage (gate 3) → /brehon-verify ✓ → gate 5 → bm-merge → Task 5
retro (gate 6) → /brehon-phase-transition.

## advisor: Task 4 #298 done → verified → finalize-merged → §5.2 cmd1 launched 2026-05-17

Task 4 = Junior #298 DONE (run succeeded 13:25:05, ~4.5 min). Branched
correctly from synced `88148edd0` (proves CAS-sync worked — worker read
the brief). VERIFY-BEFORE-TRUST passed: commit `3f9d53274 test(e2e):
assert AGPL §13 disclosure surface via /api/v4/site + /api/v4/source
(task 4)` + DQ #241 raised (`e59bf4215`); diff = ONLY
crates/server/tests/e2e.rs +63 lines, SINGLE hunk `@@ -14860,3
+14860,66 @@` at file end (GOTCHA 3 honored — one anchor-Edit, no
Junior hang); test at e2e.rs:14865 outer `lemmy_utils::error::
LemmyResult<()>` (Case A ✓); `governance_fixtures::bootstrap()` NOT
admin_config (GOTCHA 2 / DQ #226 ✓); both /api/v4/site + /api/v4/source
GETs present; no `.map_err(` (bare ? Case A ✓); test name unique.
DQ #241 = max+1, NO collision (single task, not cohort).

Daemon did NOT finalize-merge (phase tip stayed e5ff52a9b; worker
pre-pushed → `feedback_junior_finalize_skips_when_worker_pre_pushes`).
ADVISOR-SIDE finalize-merge on THIS lane worktree: `git merge --no-ff
origin/junior/...task-4...-298` → CLEAN (ecba04523, no conflict).
CRITICAL DQ-merge verify: the ort merge unioned worker-DQ (had #241 +
PRE-pass #239/#240 from its 88148edd0 base) with phase-tip DQ (POST-pass
#239/#240); verified #239+#240 STILL result:pass in resolved[] (NOT
reverted — no resurrection bug), #241 pending result:None, 224 ids no
dups. Pushed `e5ff52a9b..ecba04523`.

§5.2 validate-pending-laptop for DQ #241 (Shape G suspended DQ #229):
canonical checkout C:/Users/barri/Developer/brehon-fork detached@
ecba04523 (clean pre-flight, no concurrent cargo, new test present ×1).
cmd1 cargo-check --workspace --features full bg=b3gcqkbtq launched
(warm). NEXT serially: cmd2 cargo-clippy --workspace --features full
--no-deps -- -D warnings; cmd3 cargo-test --no-run -p lemmy_server
--test e2e. Read EXIT markers NOT bg summaries. ALL 3 pass → mutate
DQ #241 result:pass answered_by:advisor-laptop → THEN e2e RUN =
Phase-2 user-gate-4 (AskUserQuestion local vs dispatch) → on green
/brehon-verify §16a Story 3 → bm-pr → CR → bm-triage (gate 3) →
/brehon-verify ✓ → gate 5 → bm-merge → Task 5 retro (gate 6) →
/brehon-phase-transition.

## advisor: §5.2 Task 4 ALL 3 PASS — DQ #241 resolved, e2e RUN gate next 2026-05-17

Task 4 Phase-1 workspace validation on ecba04523 (canonical detached):
- cmd1 cargo-check --workspace --features full: PASS 2m07s, 0 err.
- cmd2 cargo-clippy --workspace --features full --no-deps -- -D
  warnings: PASS 1m47s, 0 warnings (new e2e test = no clippy debt).
- cmd3 cargo-test --no-run -p lemmy_server --test e2e: PASS 1m58s,
  e2e-aebb6c76db6b1818.exe built — agpl_source_disclosure_surface_
  returns_notice COMPILES clean (Case A, governance_fixtures::bootstrap
  resolves, both endpoint imports resolve).

DQ #241 mutated → result:pass, answered_by:advisor-laptop, resolved[].
Pushed `363a3424e`. pending=[#229] only (Shape-G reminder, not a
blocker). All 6 plan tasks' CODE now delivered + Phase-1-validated
(Task 0 audit, Task 1 DTOs, Cohort A Tasks 2+3, Task 4 e2e; Task 5
retro is the closer).

NEXT: **Phase-2 e2e RUN = USER GATE 4** (local vs dispatch — never
auto-pick post-PR-#105 per feedback_e2e_local_or_dispatch_user_choice).
This runs the ACTUAL e2e test (testcontainers Postgres + Docker), not
just compile. Recommended default = local (feedback_default_local_
testing + project_laptop_canonical_cargo_runner — laptop 64GB canonical
runner, zero billed; Shape G suspended so GH dispatch also possible via
gh workflow run). Surfacing AskUserQuestion now.

## advisor: Phase-2 e2e RUN FAILED — DQ #242 result:fail (NON-ALLOWLIST → catch-fire)

bg `bfkex0oe1` on canonical checkout detached@`dab15ec56`
(cargo-test.bat --workspace --test e2e --features full, testcontainers
Postgres) finished in 1957.28s. Marker = `E2E_EXIT_NONZERO` (the bg
task-notification's "exit 0" was the bg-wrapper's own exit, NOT
cargo's — `feedback_background_task_notification_lies` again; the log
EXIT marker is authoritative).

`test result: FAILED. 89 passed; 1 failed; 5 ignored`. **ONLY the new
test failed — NO pre-existing e2e regression** (89 passed = full prior
suite green). Failure:

```
thread 'agpl_source_disclosure_surface_returns_notice' panicked at
crates\server\tests\e2e.rs:14883:3:
assertion `left == right` failed: /api/v4/site must return 200
  left: 500
 right: 200
```

The panic is on the test's FIRST assertion (`GET /api/v4/site` == 200),
BEFORE `/api/v4/source` is exercised. `/api/v4/site` is a PRE-EXISTING
endpoint that Task 2 modified (added `source_disclosure:
SourceDisclosure` to GetSiteResponse + populated it in
`crates/api/api_crud/src/site/read.rs`). HTTP 500 (not 404, not a
deserialize panic) ⇒ the read.rs handler errors at request time
constructing/serializing the new field. Most likely
`env!("BREHON_FORK_COMMIT")` unresolved at runtime (build.rs in
`crates/api/api_crud/` not emitting `cargo:rustc-env`, or env! in the
wrong crate), or a panic/Err in the source_disclosure build path.

This is an **IMPL bug in Task 2's Story-1 wiring** (not a test bug, not
a flake). §G4: runtime assertion / HTTP 500 = **NON-ALLOWLIST** (not
E0432/deprecated/clippy-doc). No auto-fix, no auto-retry.

DQ #242 mutated `result:fail`, **STAYS in pending[]** (pending = [#229,
#242]); log_slice = full failures: block + root-cause hypothesis.
Catch-fired to user — needs judgment: fix-impl on read.rs/build.rs
(Task 2 surface) vs re-plan. **Blocks bm-pr until resolved.** Full run
log: `.claude/runlog/e2e-v1-ship-1-dab15ec56.log` lines 150-167.

## advisor: ROOT CAUSE PINNED — plan defect (governance_fixtures::bootstrap seeds no local_site)

Investigated (read-only, no fix applied — catch-fire to user):

- Test panics on FIRST assertion (`/api/v4/site` == 200; got 500), before
  `/api/v4/source` is reached. So Task 3 wiring is irrelevant to this fail.
- Task 2 impl is structurally CORRECT: `build.rs` has a sound
  `env::var → git rev-parse → "unknown"` fallback; `env!("BREHON_FORK_COMMIT")`
  resolves at compile time (test binary built clean in Phase-1); the
  `SourceDisclosure { .. }` construction is plain `.to_string()` — nothing
  that panics or Errs. The 500 is NOT in the new source_disclosure code.
- `crates/server/tests/e2e.rs:801-836` `governance_fixtures::bootstrap()`
  does **schema-apply ONLY** (`apply_all_schema`) — seeds ZERO rows. No
  `instance`/`site`/`local_site`. Callers seed their own via
  `seed_user`/`seed_community`.
- `/api/v4/site` → `read_site` → `SiteView::read_local(&mut pool)` REQUIRES
  a `local_site`+`site`+`instance` row. Empty schema-only DB → `read_local`
  returns Err(NotFound) → `?` → `get_site`'s
  `.map_err(|e| anyhow::anyhow!("Failed to construct site response"))?`
  → **HTTP 500**.
- The plan §10.7 precedent `all_mvp_endpoints_return_non_404`
  (`e2e.rs:3780`) the brief told impl to mirror **never calls
  `/api/v4/site`** — it sweeps only `/api/v4/governance/*`. NO existing
  e2e test exercises `/api/v4/site`, so the bootstrap-data gap was
  invisible until runtime.

VERDICT: **plan/brief design defect** (the DQ #226-mandated
`governance_fixtures::bootstrap()` cannot satisfy the test's first
assertion — `/api/v4/site` needs site-bootstrap rows the governance
fixture deliberately doesn't seed). NOT an impl bug (impl followed the
brief verbatim). NOT a test-author bug (followed §10.7 + DQ #226). NOT
§G4-auto-fixable. Needs user judgment on the fix path. Catch-fired.
**Blocks bm-pr.** DQ #242 stays pending (result:fail).

## advisor: fix-impl-5 dispatched (Junior #300) — plan-defect recovery

User chose fix-impl path (vs re-plan) for the Phase-2 e2e fail. Authored
`.claude/PRPs/briefs/v1-ship-1-fix-impl-5.md` (committed `f84e03e42` on
phase-v1-ship-1): single anchor-Edit into `crates/server/tests/e2e.rs`
adding (A) `lemmy_db_schema::source::{instance,local_site,
local_site_rate_limit,person,site}` + `lemmy_diesel_utils::traits::Crud`
imports inside the test fn, and (B) a `Instance::read_or_create → Site
→ Person sysacct → LocalSite → LocalSiteRateLimit` seeding block right
after `governance_fixtures::bootstrap()` and before the first
`/api/v4/site` TestRequest. Mirrors the VERBATIM compile-tested
canonical precedent at `e2e.rs:4744-4761`
(`governance_outbox_emits_remote_sanction_notice_on_local_sanction`,
which seeds the same scaffold for the same `SiteView::read_local`
reason). Constructor signatures advisor-verified by reading the
precedent + its imports (e2e.rs:4598-4608). No production-code change,
no new fixture helper, assertions unchanged. Brief = NON-allowlist
fix-impl (runtime assertion class) → hand-authored recipe + cited
precedent (§G4 verbatim-row blockquote gate is allowlist-only, N/A).

Daemon pre-flight (anti-TOCTOU, safe pattern): `daemon_status` =
running, **0 active / 0 queued**, uptime 1m34s (clean restart);
list_tasks cross-checked = zero running/queued lines. Daemon-local
`refs/heads/phase-v1-ship-1` was divergent at `d31a84b9d` (one
local-only commit = the daemon's OWN superseded Task-4 finalize-merge;
origin `f84e03e42` already carries equivalent Task-4 content via the
laptop-side lineage). Checked-out branch = `phase-v1-AD-e` (NOT ship-1
→ working tree untouched by a ship-1 ref move). User-authorized
**CAS-guarded `git update-ref refs/heads/phase-v1-ship-1 f84e03e42
d31a84b9d`** (expected-old guard = anti-TOCTOU; aborts if ref moved
since read). CAS_OK → new=`f84e03e42`, AD-e checkout still intact.
NEVER reset --hard, NEVER paused daemon (idle anyway). Per the durable
AD-e cross-lane TOCTOU lesson.

`mcp__junior-brehon__create_task base_branch=phase-v1-ship-1` →
**Junior #300** queued. Next: poll #300 → on done, verify-before-trust
(fetch worker branch, diff-stat = e2e.rs only, grep the seeding block
+ assertions-unchanged), finalize-merge (with post-merge
DQ-resurrection re-assert of #239/#240/#241 pass + #242 still
fail-pending), §5.2 Phase-1 re-validate (the new validate-pending-
laptop the worker raises ≈ DQ #243), then advisor RE-RUNS Phase-2 e2e
local on the new tip. On E2E_EXIT_0 + agpl test passed + no regression
→ mutate #242 pass → /brehon-verify → bm-pr. DQ #242 stays pending
(result:fail) until the re-run passes.

## advisor: fix-impl-5 (#300) verified + finalize-merged → b7286ac8e

Junior #300 status:done (run #1 succeeded 19:04:37→19:07:24, ~3min).
Worker branch `junior/role-impl-task-v1-ship-1-fix-impl-5-...-300`.

**Verify-before-trust PASS** (status:done ≠ proof):
- e2e.rs diff = +26 insertions, 0 deletions, ONLY
  `crates/server/tests/e2e.rs` (test commit `50f06a7ab`).
- Imports block added inside fn (`lemmy_db_schema::source::{instance,
  local_site,local_site_rate_limit,person,site}` + `Crud`).
- Seeding block (`Instance::read_or_create → Site → Person sysacct →
  LocalSite → LocalSiteRateLimit`) inserted AFTER
  `governance_fixtures::bootstrap()` (e2e.rs:14878) and BEFORE the
  first `/api/v4/site` TestRequest (assert at 14909) — verbatim mirror
  of canonical precedent e2e.rs:4751-4761.
- Unique markers `"agpl test site"` / `"agpl_sysacct"` count = 1 each
  (no duplication; single block in the agpl test only).
- Assertions UNCHANGED (purely additive diff; 0 deletions).
- Case A preserved (`LemmyResult<()>` outer, bare `?`, no .map_err).
- Commit msg verbatim: `test(e2e): seed local_site so /api/v4/site
  returns 200 in agpl disclosure test (fix-impl-5)`.
- DQ commit `3c6549024` raised DQ #243 (validate-pending-laptop,
  from:impl, phase_task:4, result:null, commands = §15.1-3 .sh).

**Pre-merge DQ divergence analysis** (resurrection-risk check):
worker DQ (branched from `f84e03e42`) vs phase tip (`82fc97a34`) =
IDENTICAL except #243 (only on worker). ids-only-on-phase-tip = []
(worker base already had every laptop-side #242-fail/#241-pass
mutation — `f84e03e42` descends from `dd188e854`/`ccda77539`). Cleanest
possible merge case; the +570/-544 DQ churn = json.dump key-reorder
noise, NOT semantic divergence (id-set comparison identical-except-243).

**Finalize-merge** (daemon idle 0/0 re-confirmed; worker pre-pushed so
daemon finalize skipped → advisor manual merge): `git merge --no-ff`
worker → `b7286ac8e`. e2e.rs diff vs pre-merge tip = exactly +26 add
(no merge corruption). **POST-MERGE DQ-RESURRECTION RE-ASSERT
(mandatory)**: #239/#240/#241 = RESOLVED(pass) STILL ✓, #242 =
PENDING(fail) STILL ✓, #243 = PENDING(result:null) present ✓,
id-unique:True, 223 resolved. No resurrection. Pushed
`82fc97a34..b7286ac8e`.

NEXT: §5.2 Phase-1 re-validate DQ #243 on CANONICAL checkout
C:\Users\barri\Developer\brehon-fork (cargo runs there, NOT lane
worktree): 3× §15.1-3 commands, ONE cmd //c per command,
run_in_background, read EXIT marker not bg summary. ALL EXIT_0 →
mutate #243 pass → advisor RE-RUN Phase-2 e2e local on b7286ac8e tip
(user already chose LOCAL gate-4; do NOT re-ask). On agpl test `ok` +
no regression → mutate #242 pass (supersede the original fail) →
/brehon-verify → bm-pr → CR → gates 3/5/6 → ship.

## advisor: fix-impl-5 §5.2 Phase-1 ALL 3 PASS — DQ #243 resolved

§5.2 validate-pending-laptop on fix-impl-5 merged tip `8e1bb57bb`
(canonical checkout detached; Shape G suspended DQ #229 → local per
advisor-orchestrator.md §5.2):

- cmd1 `cargo-check --workspace --features full`: PASS, Finished 1m35s,
  0 err. (EXIT marker read from log, NOT bg summary —
  feedback_background_task_notification_lies.)
- cmd2 `cargo-clippy --workspace --features full --no-deps -- -D warnings`:
  PASS, Finished 2m25s, **0 warnings** — fix-impl-5 imports block +
  Instance/Site/LocalSite/LocalSiteRateLimit seeding block are
  clippy-clean (mirrors clean canonical precedent e2e.rs:4751-4761).
- cmd3 `cargo-test --no-run -p lemmy_server --test e2e`: PASS, Finished
  2m14s, `Executable e2e-aebb6c76db6b1818.exe` built, 0 err.

DQ #243 mutated → resolved (result:pass, answered_by:advisor-laptop).
Pending now [#229, #242]. #242 (original e2e fail) stays fail-pending
until the Phase-2 e2e re-run on `8e1bb57bb` confirms the agpl test now
passes, at which point #242 is superseded-resolved alongside #244.

NEXT: advisor RE-RUN Phase-2 e2e LOCAL on `8e1bb57bb` (user gate-4
already = local, NOT re-asked per auto-phase invariant 4). Atomic
raise DQ #244 (validate-pending-laptop-e2e, from:advisor) BEFORE
launch, then `cargo-test.bat --workspace --test e2e --features full`
bg. On agpl `... ok` + suite 90/0 + no regression → mutate #242+#244
pass → /brehon-verify v1-ship-1 → bm-pr → CR → gates 3/5/6 → ship.
Second consecutive e2e fail = §G4 cycle-count HARD REFUSAL re-plan
signal (no auto fix-impl-6).

## advisor: Phase-2 e2e re-run after fix-impl-5 FAILED — §G4 hard-refusal catch-fire (2026-05-17)

DQ #244 mutated → **result:fail** (STAYS in pending; answered_by:advisor-laptop).
Pending now [#229, #242, #244]. #242 left as-is (original plan-defect
record, result:fail, pending).

Phase-2 e2e re-run on `phase-v1-ship-1` tip `026189102` (e2e.rs
byte-identical to fix-impl-5 merged tip `8e1bb57bb`; canonical checkout
`C:/Users/barri/Developer/brehon-fork` detached@`e706cdefe`). Command
`cmd //c scripts\brehon\cargo-test.bat --workspace --test e2e --features
full`, testcontainers Postgres. The `--workspace` build fingerprint
differed from the prior `-p lemmy_server` cmd3 build → recompiled (new
binary `e2e-07c133f22204027f.exe`), ran 95 tests in 1949.47s.

**RESULT: `E2E_EXIT_NONZERO`. `test result: FAILED. 89 passed; 1 failed;
5 ignored`.** ONLY `agpl_source_disclosure_surface_returns_notice`
failed — the prior-89 ALL still passed, ignored=5 unchanged: **NO
pre-existing regression**.

Panic (e2e.rs:14909:3), IDENTICAL to the original DQ #242 fail:
```
assertion `left == right` failed: /api/v4/site must return 200
  left: 500
 right: 200
```

**Classification (a) — SAME /api/v4/site 500.** fix-impl-5 (#300,
commit `50f06a7ab` — seed instance+Site+LocalSite+LocalSiteRateLimit
before the /api/v4/site request, mirroring precedent e2e.rs:4751-4761)
was **INEFFECTIVE**. The first assertion (`/api/v4/site == 200`) still
fails with HTTP 500, so `SiteView::read_local` needs MORE than those 4
rows, OR the mirrored precedent never actually exercises /api/v4/site
at runtime (so the seed scaffold is wrong-shaped). Rules out (b)
deeper-assert (the FIRST site assertion still fails — source_disclosure
/ /api/v4/source asserts never reached). Rules out (c) cross-test
side-effect (no pre-existing test regressed).

**§G4 cycle-count meta-rule: 2nd consecutive Phase-2 e2e fail on the
same agpl surface, SAME panic = HARD REFUSAL.** No auto fix-impl-6.
Surfaced to user as a RE-PLAN signal. **BLOCKS bm-pr** until the user
decides re-plan vs deeper-fix. Recommended next: planner Junior re-plan
of §13 Task 4 + §10.6/§10.7 — the Task-4 fixture strategy and the
"mirror precedent" that must ACTUALLY exercise /api/v4/site at runtime
(deeper SiteView::read_local seed requirement than the current
4-row scaffold).

## advisor: deeper-fix authorized — fix-impl-6 dispatched (#302) (2026-05-17)

User chose **Deeper-fix** (explicit §G4 override past the mechanical-fix
threshold) + **Mirror setup_local_site** after the 2nd same-surface Phase-2
e2e fail. Plan-mode plan approved (`idempotent-stargazing-pizza.md`).

**Root cause (4 read-only Explore passes, evidence-grounded):** plan §10.7
"mirror precedent" cited `governance_outbox_emits_remote_sanction_notice_on_local_sanction`
(e2e.rs:4744-4761) which seeds the site rows but only calls `SiteView::read_local`
**internally** — it NEVER HTTP-calls `/api/v4/site`. The bare
`SiteInsertForm::new("agpl test site", instance.id)` (the canonical *db-layer*
`TestData::create` form) leaves `ap_id`/`inbox_url`/`public_key`/`private_key`/
`last_refreshed_at` = `None`; the `Site` struct types those non-Option; the
HTTP handler path (`read_site` → `SiteView::read_local` `.select(Self::as_select())
.first().optional()?`) cannot materialise the incomplete row → 500. The exact
failing column is unconfirmed because the test asserted only `status==200` and
**discarded the response body**. Refuted: stale-cache (moka TTL=0 disables it
in debug), missing `language`/`site_language` (migrations seed languages;
`Site::create` auto-populates site_language), pool isolation. `agpl_*` is the
ONLY e2e test that HTTP-drives `/api/v4/site`.

**Fix-impl-6 brief** (`9d84c33c1`, committed): ONE impl-task, ONE commit,
TWO coupled parts into the single agpl fn — Part A (read body BEFORE the
status assert, fold into panic message; also for `/api/v4/source`) so the
real `LemmyError` is legible; Part B (replace bare `SiteInsertForm::new`
with the complete form mirroring proven `setup_local_site.rs:80-96`:
`generate_actor_keypair()` + parseable `ap_id`/`inbox_url` for `test.invalid`
+ `last_refreshed_at` + `private_key`/`public_key`). Bounded literal-fallback
if production helpers unreachable from the test crate. §2.4 mandatory e2e
lessons injected. next DQ id = 245.

**Daemon ref recovery (lane-safe, user-approved):** daemon-local
`phase-v1-ship-1` was `02e13d800` (daemon's redundant own fix-impl-5
finalize-merge; same code as origin via `9d84c33c1`, different SHA — the
known daemon multi-lane ref-isolation gap) with a LEFTOVER conflicted index
(`UU .claude/decision-queue.json`, no MERGE_HEAD). No live Junior task
(0 active/0 queued); no other worktrees (AD-e etc = branches only). Recovery:
`git reset` (mixed, NO --hard) → `git checkout -- .claude/decision-queue.json`
(discard leftover UU; origin authoritative) → `git checkout -B phase-v1-ship-1
origin/phase-v1-ship-1`. VERIFIED: daemon-local ref == origin == HEAD ==
`9d84c33c1`, work-tree clean, fix-impl-6 brief PRESENT on tip. No --hard,
no force, no data loss.

Dispatched **Junior #302** `[role:impl-task]` `base_branch=phase-v1-ship-1`.
NEXT: poll → verify-before-trust (≤1 file, e2e.rs only, Part A read_body +
Part B setup_local_site shape) → finalize-merge → post-merge DQ-resurrection
re-assert → §5.2 Phase-1 (3 cmds) → Phase-2 e2e (local; gate-4 cached) →
on agpl `... ok` + 90/0/5 → mutate #242+#244 pass → /brehon-verify → bm-pr →
CR → gates 3/5/6 → merge → Task 5 retro → phase-transition. 3rd same-surface
fail = re-plan catch-fire (NO auto fix-impl-7).

## advisor: fix-impl-6 #302 verified + finalize-merged 2026-05-17

Junior #302 `[role:impl-task]` DONE (run #1 succeeded, ~9.5 min,
21:15:49→21:25:18Z). **Verify-before-trust CLEAN:** worker branch diff =
exactly 2 files — `crates/server/tests/e2e.rs` (+26/-... single fn
`agpl_source_disclosure_surface_returns_notice`) + `.claude/decision-queue.json`
(worker-raised DQ #245 `validate-pending-laptop`). NO production code touched
(no crates/api|db_schema|db_views|routes). Part A confirmed: both `/api/v4/site`
and `/api/v4/source` asserts restructured to `let status=...; let body_bytes=
test::read_body(resp).await; assert_eq!(status, 200, "... — body: {}",
String::from_utf8_lossy(&body_bytes))` — body legible on failure. Part B
confirmed: bare `SiteInsertForm::new` → complete form with
`ap_id/last_refreshed_at/inbox_url/private_key/public_key = Some(...)` via
`generate_actor_keypair()?` + `url::Url::parse(...)?.into()` + `chrono::Utc::now()`,
`..SiteInsertForm::new("agpl test site", instance.id)` — mirrors
`setup_local_site.rs:88-95` shape exactly. Commit body documents Part-B path =
production-helper-mirrored (one-edit discipline; fully-qualified path to avoid
expanding the use block). Test fn outer still `LemmyResult<()>` (Case A), bare
`?`, single fn, no new test.

**Finalize-merge:** worker base `9d84c33c1` was behind origin tip `073a9ac89`
(runlog commit landed after #302 dispatch) → true `git merge --no-ff` (NO
--hard, NO force), ort strategy, 0 conflicts. **Post-finalize-merge
DQ-resurrection re-assert: CLEAN** — all 10 invariants PASS: #242==fail-pending,
#244==fail-pending, #243==pass-resolved, #245==None-pending (worker-raised),
#229 pending, resolved=224, all ids unique. NO re-apply needed (worker DQ base
was consistent with lane — only the runlog commit differed, which doesn't touch
DQ). Pushed `073a9ac89..301986758` phase-v1-ship-1.

NEXT: §5.2 Phase-1 on merged tip `301986758` (canonical checkout, detached) —
DQ #245 3 cmds serialized (check → clippy → test --no-run) → on all 3 EXIT_0
mutate #245 pass → §5.2 Phase-2 e2e (local, gate-4 cached) → on agpl `... ok` +
90/0/5 → mutate #242+#244 pass → /brehon-verify → bm-pr → CR → gates 3/5/6 →
bm-merge → Task 5 retro → /brehon-phase-transition. 3rd same-surface fail =
re-plan catch-fire (Part A now makes panic body the literal LemmyError).
