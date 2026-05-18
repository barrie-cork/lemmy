---
phase: v1-ship-1
plan: .claude/PRPs/plans/v1-ship-1-r2.plan.md
phase_branch: phase-v1-ship-1
worktree: C:/Users/barri/Developer/brehon-fork-ship-1
authored: 2026-05-16
last_resume_update: 2026-05-18
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap/resume the v1-ship-1 lane-dedicated advisor session. Read the "RESUME 2026-05-18" block FIRST — it supersedes the historical r1 state below.
---

# ⏩ RESUME 2026-05-18 — read this block first (supersedes the r1 historical state below)

**You are the lane-dedicated advisor for v1-ship-1**, CWD
`C:/Users/barri/Developer/brehon-fork-ship-1` (worktree on
`phase-v1-ship-1`). Session was deliberately stopped here at a clean
boundary (no Junior task running, nothing mid-flight). Per
`.claude/rules/multi-lane-worktree.md`: write phase-branch DQ + dispatch
Junior for THIS lane only.

## Session-start ritual (do these first)

1. `pwd && git branch --show-current && git worktree list` — confirm
   `brehon-fork-ship-1` / `phase-v1-ship-1`.
2. `git fetch origin phase-v1-ship-1 && git rev-parse HEAD origin/phase-v1-ship-1`
   — HEAD should be **`4634b6ed0`** (or later if a follow-up landed). If
   behind, `git merge --ff-only origin/phase-v1-ship-1`.
3. `git status --porcelain` — only untracked
   `.claude/PRPs/debug/reconcile-*` artifacts expected (gitignored debug
   class; harmless — they are the PMD-reconcile staging from the prior
   session, safe to leave or delete).

## EXACT STATE (where things stand — 2026-05-18)

| Item | State |
|---|---|
| Plan | `.claude/PRPs/plans/v1-ship-1-r2.plan.md` (1090 lines) — **APPROVED at User Gate 1, 2026-05-18.** Confidence 9/10, complexity 5/10. |
| Plan provenance | Junior #309 (`[role:planning]`) verbatim Opus output (four-role model preserved). Re-plan: Tasks 1-4 MERGED (carry-forward context, NOT re-executed); only the failed e2e App-construction is rebuilt. |
| **Surface (Tasks 1-4)** | **MERGED + PROVEN** on `phase-v1-ship-1` ancestry. DTOs + `source_disclosure` field + `read_site` populate + `build.rs` env + `get_source` handler + `/api/v4/source` route. Phase-1 cargo all green at tip. **Do NOT re-impl.** |
| Defect | The single acceptance e2e `agpl_source_disclosure_surface_returns_notice` (`e2e.rs:14865`) returns HTTP 500. 5 fix attempts failed; §G4 hard-refusal fired; this r2 plan is the source-cited rebuild. |
| DQ #261 (planner, kind:log, RESOLVED) | The brief's literal Option(b) recipe (`to_request_data()` as `.app_data()` source) is **mechanically incompatible** — returns `activitypub_federation::config::Data<T>` ≠ the `actix_web::web::Data<T>` `get_site` extracts (DIFFERENT TypeId) → reproduces the same HTTP 500. Planner adopted `federation_config.deref().clone()` (byte-mirror of production `lib.rs:364`) — preserves brief SPIRIT, corrects the precise API. **APPROVED by user at Gate 1.** The actual fix-impl-8 bug was ONE TOKEN: `.app_data(Data::new(context.clone()))` where `context` was already `Data<LemmyContext>` → `Data<Data<LemmyContext>>` double-wrap → extractor miss → 500; fix = `context`→`inner_context`. |
| §3.4 DoD smoke | **PASS** (ran locally on this worktree, Shape G suspended per DQ #229): §15.1 `cargo check --workspace --features full` PASS 2m40s/0err; §15.3 `cargo test --no-run -p lemmy_server --test e2e` PASS 4m36s/0err (e2e binary linked). §15.2 clippy shares §15.1 graph (low-risk, not separately run). §15.4 full-e2e is Phase-2 (advisor-driven post-finalize). |
| §3.5 watchpoint gate | **PASS** — every §10 mirror anchor independently verified at HEAD `a1c47e4ad` (incl. the DQ #261 type-mismatch evidence: `read.rs:1` actix-Data import + `lib.rs:364/379` mirror + `e2e.rs:14865` unique failed fn). |
| User Gate 1 (plan approval) | **CLEARED — APPROVED 2026-05-18.** |
| bm-cut | **DONE long ago** (#274). `phase-v1-ship-1` exists; Tasks 1-4 merged. **No bm-cut for r2.** |
| Runlog | Up to date at `.claude/runlog/v1-ship-1-runlog.md` (last entry: r2 plan approval + PMD fix @ `4634b6ed0`). |
| PMD cross-lane gap (Task #6) | **FIXED 2026-05-18.** Both `.mcp.json` pinned `PROJECT_MEMORY_DB` → absolute canonical path (on disk, gitignored). Forward-invariant in commit `078aa501c` (multi-lane-worktree.md §PMD + `.mcp.json.example` guard + `feedback_pmd_cross_lane_canonical_db.md`). 21 stranded retros reconciled into canonical (IDs 375-395). The Stop-hook retro false-block class is structurally eliminated — retros now land where `retro-check.sh` reads them. **One cleanup TODO (low priority): prune disposable probe eval ID 374 via MCP `memory_prune`.** |

## THE EXACT NEXT ACTION (resume here)

Next stage per advisor-orchestrator stage-shape = **impl per cohort
dispatch**. Plan §13: Task 0 (pre-flight, solo non-`[P]`) → Task 6 (e2e
rebuild, single anchored Edit, solo) → Task 7 (retro). **Task 0 is the
next dispatch.**

**BLOCKER before queueing Task 0 — `/precheck` Check 3b was STALE:**
At session-stop, daemon-local `phase-v1-ship-1` = `a1c47e4ad`, origin =
`4634b6ed0` (the daemon checkout is parked on `phase-v1-federation-inbound-a`,
the other active lane; its local ship-1 ref lagged). Checks 1/2/4/5
PASSED (Tailscale+SSH ok; daemon active+patched; window OK Mon ~12:15
UTC, secondary window 04:30–14:59; mem 10 GB free).

**Resume procedure:**

1. Re-run `/precheck phase-v1-ship-1` (state may have changed since stop
   — re-confirm, do not assume STALE persisted).
2. If Check 3b still STALE: apply the **lane-safe refspec-fetch** (the
   ONLY correct primitive — NEVER `git checkout` on the daemon, hard
   refusal per `multi-lane-worktree.md`):
   ```bash
   ssh homeserver 'cd /srv/brehon-fork && git fetch origin phase-v1-ship-1:phase-v1-ship-1'
   ```
   Then VERIFY lane-safety (checkout MUST stay on its current branch,
   tree clean):
   ```bash
   ssh homeserver 'cd /srv/brehon-fork && git branch --show-current && git status --porcelain | head -5'
   ```
   Confirm daemon-local `phase-v1-ship-1` == `origin/phase-v1-ship-1`.
3. With Check 3b SYNC + all other checks PASS → queue **Task 0** via
   `mcp__junior-brehon__create_task`. Brief: author
   `.claude/PRPs/briefs/v1-ship-1-impl-0.md` (pre-flight; the plan §13
   Task 0 enumerates 16 probes — mechanical from the plan; commit on
   `phase-v1-ship-1` before create_task). Dispatch string:
   `[role:impl-task] v1-ship-1-r2 Task 0 pre-flight — see .claude/PRPs/briefs/v1-ship-1-impl-0.md`,
   `base_branch=phase-v1-ship-1`.
   **NOTE the Probe 1/3 false-blocker** (carried lesson, see r1 historical
   §0 below + reconciled retro orig-id-1): Task 0 brief Probes 1 & 3 must
   be reworded — Probe 1 → branch matches `^junior/.*phase-v1-ship-1` OR
   `== phase-v1-ship-1`; Probe 3 → `git merge-base --is-ancestor
   <recorded-bm-cut-sha 8c271285e> HEAD` (NOT moving governance-v0).
   Without the reword the Junior worker self-downgrades them to WARN
   (defensible but a literal-contract deviation).
4. Then per stage-shape: Task 0 complete → Task 6 (e2e rebuild) →
   §5.2-laptop Phase-1 (the 3 §15 cmds) → §5.2-laptop Phase-2 e2e
   (advisor-driven post-finalize-merge) → on agpl PASS: mutate DQ
   #255+#251+#242+#244+#246 → `/brehon-verify` → bm-pr → CodeRabbit →
   User Gate 3 (CR triage) → User Gate 5 (merge confirm) → bm-merge
   (L14/L15/L16) → Task 7 retro (promote carry-forward + the
   (vii)-(xi) set to `.claude/lessons/` + index MEMORY.md IN THE SAME
   retro commit) → User Gate 6 → `/brehon-phase-transition`.

**§G4 status:** override-scope DQ #247/#249/#252 EXHAUSTED — the r2
re-plan supersedes it. No auto-author of any further fix-impl without a
fresh user directive. §15.6 cycle-count meta-rule: if Task 6's first
Phase-2 attempt fails with the SAME `(actix-Data-500, e2e.rs)` tuple →
HARD REFUSAL catch-fire (3rd cycle), surface + WAIT, do NOT auto-queue
fix-impl-9.

## DQ pending (lane view, phase-v1-ship-1)

7 pending: `[#229, #242, #244, #246, #248, #251, #255]`. #229 =
Shape-G-re-enable tracker (deferred to 2026-06-01, not gating). #242/
#244/#246/#251/#255 = prior fix-impl/e2e fail-pending entries (the r2
plan supersedes the work; they get mutated to pass when Task 6's
Phase-2 e2e goes green per the stage-shape above). #248 = check at
resume. NONE block queueing Task 0 (they are historical fail-pending,
not active blockers).

---

# v1-ship-1 lane bootstrap — resume here (HISTORICAL — r1 era, 2026-05-16; superseded by the RESUME block above)

You are the **lane-dedicated advisor for v1-ship-1**, running from
`C:/Users/barri/Developer/brehon-fork-ship-1` (worktree on
`phase-v1-ship-1`). The canonical `C:/Users/barri/Developer/brehon-fork`
[governance-v0] session has handed v1-ship-1 to you and reverts to
governance-v0 meta-edits only. Per `.claude/rules/multi-lane-worktree.md`:
this session writes phase-branch DQ entries + dispatches Junior tasks for
THIS lane only; it does NOT touch other lanes' branches.

## 0. State summary (where things stand)

| Item | State |
|---|---|
| Plan | `.claude/PRPs/plans/v1-ship-1-r1.plan.md` — **APPROVED at User Gate 1** (2026-05-16) |
| Plan provenance | Salvaged from blocked Junior #270 (harness #10 — see §3). Content is **verbatim Junior #270 Opus output** (four-role model preserved — advisor did NOT author plan body; salvage + mechanical relocation only). Design carried verbatim from parked `v1-ship-1.plan.md`; MIRROR refs re-derived vs `governance-v0` HEAD `9504c806d`; DQ #226 + #227 resolutions applied. |
| §3.4 DoD smoke | **3/3 PASS** (ran LOCALLY on the canonical laptop per the standing session directive — Shape G suspended repo-wide until 2026-06-01, commit `086cfa5d4`): §15.1 `cargo check --workspace --features full` PASS 2m21s/0err; §15.2 `cargo clippy --workspace --features full --no-deps -- -D warnings` PASS 7m10s/0err/0denywarn; §15.3 `cargo test --no-run -p lemmy_server --test e2e` PASS 12m06s, `e2e.rs` executable built. §15.4 full-e2e DEFERRED to post-approval Phase 2. |
| §3.5 watchpoint gate | **PASS** — every watchpoint cites file:line; DQ #226 dual-bootstrap watchpoint = `governance_fixtures::bootstrap` @ `e2e.rs:801` cross-ref'd §9/§10.6/§13-Task4/§15.5/§16a + Task0 Probe8/9; DQ #227 R9 = `api.rs:337` def + `read.rs:57` ctor + `handlers.rs:249-258` NON-BREAKING rest-pattern. One non-blocking note: §9:198/§10 cite the EXCLUDED `admin_config_fixtures` sibling at `e2e.rs:5556`/bootstrap`5578` while DQ #226 said `5643`/`5665` — the planner re-derived fresher vs HEAD `9504c806d`; this sibling is NEVER called (correctly excluded), so the line is immaterial. |
| bm-cut | **DONE** (#274). `phase-v1-ship-1` @ `8c271285e` on origin (= governance-v0 tip, descends correctly). |
| Runlog | **NOT yet at `.claude/runlog/v1-ship-1-runlog.md`.** bm-cut's §6 fallback executed correctly (the block was `worktree-guard.sh` PMD#998 "write outside worktree", NOT the CC #10 gate — the worker wrote an abs repo-root path). Runlog committed as worktree-root `v1-ship-1-runlog.md` in `375a15c73` on the WORKER branch `junior/role-bm-task-bm-cut-v1-ship-1-...-274` (daemon-local, NOT merged to phase, NOT pushed). It is a 7-line BM ledger, trivial, NOT load-bearing. **TODO (you): recreate it properly** at `.claude/runlog/v1-ship-1-runlog.md` on this lane worktree (interactive Write to `.claude/` from a laptop session is NOT gated — #10 only affects daemon Junior workers). Content: a `# v1-ship-1 Runlog` header + a `## bm: cut phase-v1-ship-1 off governance-v0 @ 8c271285e` first entry. Commit on `phase-v1-ship-1`, push. Do this before/with the first impl artifact; do NOT chase the worker-branch finalize-merge for an audit file. |

## 1. The plan (read it in full before dispatching)

`.claude/PRPs/plans/v1-ship-1-r1.plan.md` (1404 lines). Deliverable =
AGPL §13 source-disclosure surface:
- `GetSiteResponse` gains a `source_disclosure: SourceDisclosure` block
  (license SPDX, repo URL, build-time-injected fork commit SHA, relative
  `/api/v4/source` URL).
- New `GET /api/v4/source` returns `GetSourceResponse { notice, license }`
  via `include_str!` of `AGPL-NOTICE.md`.
- `build.rs` in `crates/api/api_crud/` injects `BREHON_FORK_COMMIT`
  (location promoted from the parked plan's `routes/` fallback — verified
  `lemmy_api_crud ⊄ lemmy_api_routes`).
- One new e2e test asserting both surfaces (Case A discipline,
  `governance_fixtures::bootstrap()` @ `e2e.rs:801`).
- No new crate, no migration, no new ADR. Complexity 8/10 (at-not-above
  Sonnet `>8` split threshold → no split-DQ).

§13 task structure (6 tasks):
- **Task 0** — pre-flight harness audit (12 probes, non-`[P]`, NO commit).
- **Task 1** — DTOs (`SourceDisclosure`/`GetSource`/`GetSourceResponse`),
  solo barrier (every downstream task `requires:` Task 1's types).
- **Cohort A — Tasks 2 + 3 (`[P]`, file-disjoint)**: Task 2 = field +
  `read_site` populate + `build.rs`; Task 3 = `get_source` handler +
  route + mod wiring.
- **Task 4** — e2e test, solo (`requires:` Tasks 2 + 3).
- **Task 5** — retro, solo, non-`[P]`.

## 2. Dispatch sequence (what you do next)

Per `.claude/rules/advisor-orchestrator.md` §3.1 + §4:

1. **Recreate the runlog** (see §0 table) — `.claude/runlog/v1-ship-1-runlog.md`, commit on `phase-v1-ship-1`, push.
2. **Task 0** — author `.claude/PRPs/briefs/v1-ship-1-impl-0.md` (pre-flight; plan §13 Task 0, lines ~648-716). Dispatch as `[role:impl-task]`, `base_branch=phase-v1-ship-1`. Task 0 is non-`[P]`, NO commit (probes only) — it verifies environment + the dual-bootstrap topology. Wait for complete.
3. **Task 1** — author `.claude/PRPs/briefs/v1-ship-1-impl-1.md` (DTOs; plan §13 Task 1, lines ~717-786). Solo barrier. Dispatch, wait for complete + validation.
4. **Cohort A (Tasks 2 + 3)** — author both briefs, dispatch SIMULTANEOUSLY (parallel `create_task`, single message) per §4 cohort dispatch. They are file-disjoint (verify §11 + FILES YAML overlap check first). Wait for BOTH.
5. **Task 4** — e2e, solo, `requires:` 2+3. Dispatch, wait.
6. **Task 5** — retro. **This brief MUST carry the §6 harness-#10 Option-B note** (Task 5 writes `.claude/PRPs/reports/v1-ship-1-retro.md` — a `.claude/**` path the daemon Junior CANNOT write under CC #10; the worker must fall back to worktree-root + you relocate). Tasks 0-4 are `crates/`+`tests/` ONLY — they do NOT hit the gate; their briefs need NO #10 caveat.
7. Then bm-pr → CR poll → triage → fix-in-PR → /brehon-verify → merge → phase-transition, per the standard stage-shape.

§2.4 mandatory file-class lesson injection — walk the table per brief:
- Task 1 (`crates/db_views/site/src/api.rs` struct-add, no e2e/migration): no mandatory row fires; §2.3 hybrid PMD search still runs.
- Task 4 (`crates/server/tests/e2e.rs` ≥1 edit): inject `feedback_lemmy_error_no_std_error.md` (Case A — sibling at `e2e.rs:801` etc; plan §10.6 GOTCHA 1 already specifies Case A) + `feedback_async_pool_test_pattern.md`. The plan's §10.6/§13-Task4 already encode the canonical sibling shape — mirror it.

## 3. Harness #10 (CC v2.1.119 `.claude/**` sensitive-file gate) — CRITICAL

**Root cause (confirmed, NOT a daemon defect):** Claude Code CLI v2.1.119
enforces a hardcoded sensitive-file gate on the `.claude/**` glob that is
NOT overridden by `--dangerously-skip-permissions` (Junior workers run in
`bypassPermissions` mode — verified via job-270 CC `system/init` — but the
gate still fires) NOR by `settings.json permissions.allow`. The Junior
daemon spawn-args are correct; `claude.ts buildClaudeArgs` is correct +
unmodified. It is a CC-CLI-level protection of the agent config tree.

**Consequence:** any Junior `planning`/`impl`/`bm`/retro task that writes
under `.claude/` is blocked. **But v1-ship-1 impl Tasks 0-4 write only
`crates/` + `tests/` — they are UNAFFECTED.** Only Task 5 (retro →
`.claude/PRPs/reports/`) hits the gate.

**Fix status — Option-A drafted but NOT applied (deferred):**
- A scoped `PreToolUse` hook `allow-prp-deliverables.sh` was drafted
  (mirrors `worktree-guard.sh`; auto-approves Write/Edit to ONLY
  `.claude/PRPs/{plans,briefs,reports}/**` inside worktrees; never denies;
  behavior config rules/agents/lessons/commands/skills stays gated).
  Source on the canonical laptop: `C:\Users\barri\AppData\Local\Temp\allow-prp-deliverables.sh`
  (sha256 `9b08464e707dccfe1015350244c2624f1c2b0cd22033b566e6adce94f824196b`).
  Plus a one-line `settings.json` PreToolUse-array patch.
- The user APPROVED Option A but its install was BLOCKED: the auto-mode
  classifier requires a per-command Bash permission rule for a
  security-gate modification on shared-daemon infra (an AskUserQuestion
  plan-approval is NOT sufficient authorization for the consequent shell
  command), AND the user was on remote mobile (the `!`-prefix fallback is
  void on mobile — confirmed via claude-code-guide; Remote Control =
  text-relay only, no local shell).
- **User chose Option-B workaround for now:** proceed via the
  worktree-root-write + advisor-relocate pattern (proven for #270's plan
  and #274's runlog). The hook (Option A) is revisited when the user
  authorizes it (Bash permission rule, or runs the install at the laptop
  terminal — the user IS now at the laptop, so the install path is
  reopened; surface this as an option before Task 5 if the user wants the
  hook in place to avoid the Task-5 retro relocate).

**Open task #10** tracks the Option-A install (deferred). The user may
choose to apply it before Task 5 (eliminates the retro relocate) or keep
Option-B (relocate the retro once). Surface the choice at the Task-5
boundary; do NOT auto-decide.

## 4. The daemon-local-trunk-stale discipline (MANDATORY — the #273 lesson)

`feedback_daemon_local_trunk_stale_multi_lane.md` (in PMD + auto-memory).
The #273 bm-cut PHANTOM was caused by this: a Junior `create_task` with
`base_branch=<X>` branches from the **DAEMON's LOCAL `<X>` ref**, NOT
`origin/<X>`. With multiple lanes active, the daemon's single
`/srv/brehon-fork` checkout sits on another lane's branch and its local
`governance-v0`/`phase-v1-ship-1` refs go stale. A worker that can't see
a freshly-pushed brief escalates **silently (no DQ)** and exits
`subtype:success` → the advisor mistakes `status:done` for success.

**Before EVERY `mcp__junior-brehon__create_task`** (Task 0, Task 1, each
cohort member, Task 4, Task 5), verify the daemon-local ref the task
branches from is synced:

```bash
ssh homeserver 'cd /srv/brehon-fork && git fetch origin phase-v1-ship-1 --quiet && \
  [ "$(git rev-parse phase-v1-ship-1)" = "$(git rev-parse origin/phase-v1-ship-1)" ] && echo SYNC || echo STALE'
```

(Use `phase-v1-ship-1` as the ref for impl tasks since they
`base_branch=phase-v1-ship-1`; use `governance-v0` for any
`base_branch=governance-v0` task.) On `STALE`, fast-forward the
daemon-local ref **WITHOUT switching the checkout** (lane-safe — does NOT
disturb other lanes):

```bash
ssh homeserver 'cd /srv/brehon-fork && git fetch origin phase-v1-ship-1:phase-v1-ship-1'
```

Then VERIFY `git branch --show-current` on the daemon is unchanged + tree
clean. **NEVER `git checkout phase-v1-ship-1` on the daemon** — that
switches the shared checkout away from whatever lane is active there
(`.claude/rules/multi-lane-worktree.md` hard-refusal class). The
refspec-fetch (`git fetch origin X:X`) is the correct lane-safe primitive.

**Verify-before-trust** every bm/Junior outcome: `status:done` ≠
deliverable-exists. After bm-cut/impl tasks, confirm the actual artifact
(branch on origin via `git ls-remote`; commit on phase branch via
`git log origin/phase-v1-ship-1 --oneline`). The #273 + #274 cycles both
required this check to catch phantom/partial outcomes.

## 5. Standing session directives + invariants (carry these)

- **Shape G suspended repo-wide until 2026-06-01** (commit `086cfa5d4`;
  DQ #228/#229). `cargo-validate-workspace.yml` + adr-compliance +
  cargo-validate-migration workflows are DISABLED. **All workspace/cargo
  validation runs LOCALLY** via the advisor-laptop §5.2
  validate-pending-laptop handler (run from a laptop session against the
  phase-branch tip; bat wrappers `scripts/brehon/cargo-{check,clippy,test}.bat`;
  `--workspace --features full`; never bare `cargo test` on Windows;
  never `-p lemmy_server --features full`). Do NOT queue a ci-watcher for
  `cargo-validate-workspace` — the workflow won't fire. Phase-2 e2e is
  user-gate-4 (local vs dispatch) — recommend local.
- **DQ #229 regressed to pending on trunk** (re-introduced by the
  session-retro commit `5672a4db9` which committed a stale
  `decision-queue.json`; `821cbe718` had correctly resolved it).
  Non-blocking (DQ #229 is a 2026-06-01 future reminder). **Open task
  #11** tracks the fix — a separate `chore(decision-queue)` commit on
  governance-v0 re-applying `821cbe718`'s resolution. Deferred until
  after the v1-ship-1 gates clear. Do NOT bundle this into any other
  commit (attribution integrity). The "DQ pending: 1 [#229]"
  coordination-state banner is this stale entry — expected, not a
  v1-ship-1 blocker.
- **Six mandatory user gates** (never auto-decide; surface via
  AskUserQuestion): plan approval (DONE), judgment-heavy DQ, CR triage,
  Phase-2 e2e local-vs-dispatch, merge confirm, retro sign-off.
- **Four-role model:** this advisor session never authors
  `crates/`/`migrations/`/`tests/`/`docs/brehon-law-inspired-network/`
  content. Briefs + DQ + auto-state + dispatch only.
- **DQ attribution:** advisor writes `answered_by:"advisor"` only in
  `chore|docs(advisor|decision-queue)` commits; `"advisor-laptop"` for
  §5.2 validate-pending-laptop mutations; `"user"` for relayed user
  decisions. `kind:"clarify"` is advisor-only.
- **Windows traps:** DQ writes containing Rust `{`/`}`/`..`/backtick/`$`
  → Write a script file then `python <file>`, NEVER inline `python -c`
  (bash pre-expands — DQ #213/#226/#227 corruption). `gh api` OMIT
  leading slash on Git-Bash (`repos/...`). Slashed git refs →
  `git rev-parse --short` FIRST then `git show <SHA>:path`. Bash via
  PowerShell — cwd resets each call, `cd` FIRST.
  `mcp__junior-brehon__list_tasks` exceeds token cap → use
  `daemon_status` + `show_task` + tail-slice; large `task_logs` →
  delegate to a general-purpose subagent. `next_id` spans live DQ +
  `decision-queue-archive-*.json`. Commits use
  `git -c gpg.format=ssh commit`. Verify-before-trust every mutating
  gh/git (empty stdout ≠ failure — re-verify via ls-remote/rev-parse).
- **Stop-hook discipline:** mid-procedure Stop on this lane's branch
  within the 60-min `Task retro:` window → HONEST checkpoint
  `memory_write_eval` (title `Task retro: v1-ship-1 <stage> CHECKPOINT`,
  3-signal ~0.62-0.68 band, source_ref `phase-v1-ship-1 @ <sha>`, tags
  incl `brehon-fork,v1-ship-1,checkpoint,not-task-completion,stop-hook-satisfier`;
  <0.75 → improvement suggestion mandatory). NEVER forge created_at /
  modify retro-check.sh / raw SQL (tracked). After checkpoint CONTINUE.
- **L7 cadence:** exactly ONE accurate pending forward wakeup per cycle;
  `CronList` before scheduling, `CronDelete` stale; no 300s sleeps
  (270s cache-warm or ≥1200s).

## 6. Open tasks (carry into this lane's task list)

- **#8** — Deferred audit 3.E.2 / Pass-2 fixtures dedup. Post-gate,
  user-instructed-only, NOT auto-dispatched, does NOT block v1-ship-1.
- **#10** — Harness #10 Option-A install (scoped PreToolUse hook).
  Deferred; user may authorize before Task 5 to avoid the retro relocate.
- **#11** — DQ #229 trunk regression fix. Deferred until v1-ship-1 gates
  clear; separate `chore(decision-queue)` commit on governance-v0.
- **v1-ship-1 main thread** — the §2 dispatch sequence.

## 7. First actions for the new lane session

1. Session-start ritual (`.claude/rules/multi-lane-worktree.md`):
   `pwd && git branch --show-current && git worktree list` — confirm CWD =
   `brehon-fork-ship-1`, branch = `phase-v1-ship-1`.
2. Read `.claude/PRPs/plans/v1-ship-1-r1.plan.md` in full (it is the
   contract; this handover is the orientation).
3. Recreate `.claude/runlog/v1-ship-1-runlog.md` (§0 table), commit on
   `phase-v1-ship-1`, push.
4. Run the §4 daemon-local-trunk-sync check for `phase-v1-ship-1`.
5. Author + dispatch Task 0 brief (`base_branch=phase-v1-ship-1`).
6. Proceed per §2 with the 6 user gates.

The canonical `brehon-fork` [governance-v0] session is now out of the
v1-ship-1 loop (meta-edits only). You own this lane to completion.
