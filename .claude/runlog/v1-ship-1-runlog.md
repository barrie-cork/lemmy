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
