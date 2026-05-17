---
phase: v1-AD-e
role: impl-task
task: 0
brief_n: 0
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: null
canonical_sibling: ".claude/PRPs/briefs/sl-e-impl-2.md (§N house-style + §3 mandatory-lesson shape)"
---

# [role:impl-task] v1-AD-e task 0 — pre-flight harness audit + branch verification — see .claude/PRPs/briefs/v1-AD-e-impl-0.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e task 0 — pre-flight harness audit + branch verification`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 0**
from `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` §13 (lines
332–376). Task 0 is the **non-`[P]` barrier**: it verifies the
environment is ready for v1-AD-e and produces **NO commit, no file
edit, no DQ entry on the happy path**. Its only output is the EXPECT
block result reported back to the advisor.

## §2 Scope

**Run the 11 probes (Probe 0 through Probe 10) verbatim from plan §13
Task 0**, in order, on the `phase-v1-AD-e` worktree branch. The probe
block is reproduced in §5 below for convenience, but the plan §13
Task 0 text is the contract — if this brief and the plan disagree,
the plan wins.

**Produce:**

- The EXPECT-block evaluation (Probes 0–7, 9 exit 0; Probe 8 prints
  `NEG OK`; Probe 10 empty output).
- Two clippy-baseline log files written under `.claude/PRPs/debug/`
  (Probe 7 writes `v1-AD-e-task0-clippy-baseline.log`). These debug
  logs are the ONLY filesystem writes Task 0 makes.

**Do NOT** in this task:

- Edit any `crates/**`, `Cargo.toml`, `migrations/**`, `tests/**`,
  `docs/**`, route files, or any source file (Task 0 is read-only
  verification — Tasks 1–5 do the work).
- Create a commit (plan §13: "**No commit at Task 0.**").
- Add the engine dependency (that is Task 1).
- Write a `kind: "validate-pending-laptop"` DQ entry (Task 0 has no
  cargo-DoD validation gate of its own — its clippy run is a
  *baseline probe*, not a task validation; the result is reported
  inline, not via DQ).

**No commit message** — Task 0 produces no commit.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries** — confirm
   DQ #237 (scope-cut = (a) Dashboard+Audit only) and DQ #238
   (engine = (a) maud) are both in `resolved[]`. Probes 5 + 6 assert
   this programmatically; read the two entries so you understand
   what scope + engine the rest of the phase assumes.
2. **Plan §13 Task 0** (`.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
   lines 332–376) — the canonical 11-probe list + EXPECT block. This
   is the contract.
3. **Plan §5** (lines ~88–100) — complexity 4/10, branch metadata
   (`phase-v1-AD-e`), engine-parameterised note.
4. **Plan §7** — scope-cut DQ rationale (why Dashboard+Audit only).
5. `.claude/lessons/feedback_pipes_mask_exit_codes.md` — **always for
   cargo work.** Probe 7 runs clippy via the bat wrapper with full
   output captured to a log; check the exit code separately, then
   tail the log. Never pipe the clippy run through tail/grep when you
   need its exit status.
6. `.claude/lessons/feedback_clippy_test_style.md` — the workspace
   denies `unwrap`/`expect`/`#[allow]` escape hatches; Probe 7's
   clippy baseline must be genuinely clean (exit 0) for the phase to
   proceed. If the *baseline* is dirty (pre-existing lint on
   `lemmy_api` unrelated to v1-AD-e), that is a STOP-and-surface
   condition, not something to patch.

## §3a Handover from prior task

`(none — Task 0 is the first task of the phase; pre-flight barrier.)`

## §4 Constraints

- **Read-only task.** The ONLY filesystem writes permitted are the
  two clippy-baseline debug logs under `.claude/PRPs/debug/`
  (Probe 7). Any edit to `crates/**`, `Cargo.toml`, `tests/**`,
  `docs/**`, route files, or `.claude/decision-queue.json` is a hard
  refusal — STOP and report.
- **No commit.** Junior's finalize stage has nothing to merge for
  Task 0 (only gitignored debug logs change). That is correct and
  expected — do NOT manufacture a commit to "have something to
  finalize".
- **Branch:** you start on a Junior worktree off `phase-v1-AD-e`.
  Probe 0 asserts `git branch --show-current = phase-v1-AD-e`. If
  Probe 0 fails (wrong branch), STOP immediately and report —
  do not `git checkout` to "fix" it.
- **Probe exit-code discipline** (per
  `feedback_pipes_mask_exit_codes.md`): each probe's `&&/||`
  short-circuit + explicit `echo "... exit: $?"` is the contract.
  Run each probe as written; capture cargo/clippy to a file; read
  the exit code; THEN tail. Do not collapse the probe block into one
  piped pipeline.
- **EXPECT block is the pass/fail oracle.** Probes 0–7 and 9 must
  exit 0; Probe 8 must print `NEG OK (propagation works)`; Probe 10
  must produce empty output (lane-isolation check — v1-ship-1 /
  v1-federation-inbound-a must not touch our files). ANY deviation =
  STOP and surface to advisor with the failing probe number + its
  output. Do NOT proceed past a failed probe and do NOT attempt a
  fix — Task 0's job is to *detect*, not repair.
- **If a probe legitimately can't run** (e.g. Probe 9 Docker daemon
  down, Probe 10 `gh` unauth): report it as a BLOCKING result with
  the exact error; the advisor decides remediation. Do not skip-and-
  continue.
- Attribution: if you must raise a DQ (e.g. a probe reveals trunk
  drift), it is `from: "impl"`, `kind: "blocker"`, `answered_by:
  null`. NEVER `"advisor"` / `"user"` / `"planner"`.
- **CC v2.1.119 sensitive-file gate** (per the bm-cut brief §6 +
  observed on Junior #282): writes under `.claude/**` MAY be blocked
  even in `bypassPermissions` mode. Probe 7 writes
  `.claude/PRPs/debug/v1-AD-e-task0-clippy-baseline.log`. If that
  write is denied with a "sensitive file" error: re-run the clippy
  command redirecting to `<worktree-root>/v1-AD-e-task0-clippy-baseline.log`
  instead, note the relocation in your report, and continue (the
  clippy *exit code* is the load-bearing signal — the log location
  is recoverable). Do NOT treat the log-write block as a Task 0
  failure; only a non-zero clippy exit is a failure.

## §5 Probe block (verbatim from plan §13 Task 0 — the plan is the contract)

```bash
# Probe 0 — branch is phase-v1-AD-e
test "$(git branch --show-current)" = "phase-v1-AD-e" && echo "BRANCH OK" || { echo "WRONG BRANCH"; exit 1; }

# Probe 1 — v1-AD-d substrate intact on base: AdminDashboardResponse present
grep -q "pub struct AdminDashboardResponse" crates/api/api_common/src/governance.rs && echo "AD-d DTO OK" || { echo "AD-d DTO MISSING"; exit 1; }

# Probe 2 — admin_dashboard handler + gather targets present
grep -q "pub async fn admin_dashboard" crates/api/api/src/governance/admin_dashboard.rs && echo "AD-d HANDLER OK" || { echo "MISSING"; exit 1; }

# Probe 3 — config key seeded (v1-AD-a)
grep -q "governance.dashboard.html_pages_enabled" crates/api/api/src/governance/config.rs && echo "FLAG KEY OK" || { echo "FLAG KEY MISSING"; exit 1; }

# Probe 4 — route anchor present (lib.rs :544 region)
grep -q '\.route("/dashboard", get()\.to(admin_dashboard))' crates/api/routes/src/lib.rs && echo "ROUTE ANCHOR OK" || { echo "ROUTE ANCHOR MOVED — re-locate"; exit 1; }

# Probe 5 — engine DQ resolved (must be in resolved[] with an engine answer)
python -c "import json,io; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); ids=[e for e in d['resolved'] if 'v1-AD-e' in str(e.get('question','')) and 'engine' in str(e.get('question','')).lower()]; print('ENGINE DQ RESOLVED' if ids else 'ENGINE DQ UNRESOLVED'); exit(0 if ids else 1)"

# Probe 6 — scope-cut DQ resolved
python -c "import json,io; d=json.load(io.open('.claude/decision-queue.json',encoding='utf-8')); ids=[e for e in d['resolved'] if 'v1-AD-e' in str(e.get('question','')) and ('scope' in str(e.get('question','')).lower() or 'page' in str(e.get('question','')).lower())]; print('SCOPE DQ RESOLVED' if ids else 'SCOPE DQ UNRESOLVED'); exit(0 if ids else 1)"

# Probe 7 — clippy baseline clean on lemmy_api (pre-existing state)
cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task0-clippy-baseline.log 2>&1"
echo "clippy baseline exit: $?"   # EXPECT 0

# Probe 8 (NEGATIVE — exit-code propagation sanity) — a guaranteed-fail grep must exit non-zero
grep -q "THIS_STRING_DOES_NOT_EXIST_ANYWHERE_v1ADe" crates/api/api/src/governance/mod.rs && echo "NEG FAIL (should not print)" || echo "NEG OK (propagation works)"

# Probe 9 — Docker daemon (e2e Task 5 needs testcontainers later; verify now)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 10 — concurrent-PR check (no open PR touches our files)
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("crates/api/api/src/governance/admin_dashboard|crates/api/routes/src/lib.rs|crates/api/api/Cargo.toml")) | {number,title,headRefName}'
# EXPECT empty (v1-ship-1 must not touch these — verify lane isolation)
```

**EXPECT block:** Probes 0–7, 9 exit 0; Probe 8 prints `NEG OK`;
Probe 10 empty output. **No commit at Task 0.**

## §6 Expected output (return to advisor)

```
## Task 0 complete — v1-AD-e pre-flight harness audit

**Result:** PASS | FAIL
**Probe results:**
  - Probe 0 (branch=phase-v1-AD-e): <BRANCH OK | WRONG BRANCH>
  - Probe 1 (AdminDashboardResponse DTO): <OK | MISSING>
  - Probe 2 (admin_dashboard handler): <OK | MISSING>
  - Probe 3 (html_pages_enabled key): <OK | MISSING>
  - Probe 4 (route anchor /dashboard): <OK | MOVED>
  - Probe 5 (engine DQ #238 resolved): <RESOLVED | UNRESOLVED>
  - Probe 6 (scope-cut DQ #237 resolved): <RESOLVED | UNRESOLVED>
  - Probe 7 (clippy baseline lemmy_api): exit <N>  (log: <path>)
  - Probe 8 (negative propagation): <NEG OK | NEG FAIL>
  - Probe 9 (Docker daemon): <OK | NOT RUNNING>
  - Probe 10 (concurrent-PR isolation): <empty=OK | listed PRs=COLLISION>
**Commit:** none (Task 0 is read-only — correct)
**Next:** advisor queues Task 1 (add maud engine dep, isolated commit)
```

Plus any DQ #N reference if you raised a `kind: "blocker"` (only on a
genuine STOP condition like trunk drift / wrong branch / dirty
clippy baseline / lane collision).

## §7 Why this brief differs from the plan

Clean execution of plan §13 Task 0 — no overrides. The probe block in
§5 is copy-pasted verbatim from the plan; the plan text remains the
contract. The only additions are: (a) the explicit CC v2.1.119
sensitive-file-gate fallback for the Probe 7 debug-log write (§4
last bullet — observed on Junior #282 at bm-cut), and (b) the
explicit "no commit / nothing to finalize is correct" framing so the
Junior does not manufacture a commit.
