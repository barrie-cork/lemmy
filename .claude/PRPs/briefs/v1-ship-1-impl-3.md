---
phase: v1-ship-1
role: impl-task
task: 3
brief_n: 3
plan: .claude/PRPs/plans/v1-ship-1-r1.plan.md
created: 2026-05-17
related_dq: null
cohort: A
cohort_peers: [2]
---

# [role:impl-task] v1-ship-1 task 3 — see .claude/PRPs/briefs/v1-ship-1-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-ship-1 task 3 — see .claude/PRPs/briefs/v1-ship-1-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 3
from `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 (Task 3 `[P]`, lines
~892-991). This is **Cohort A member 2** (parallel with Task 2, which
is file-disjoint — you will NOT touch any file Task 2 touches and
vice-versa). Both `requires:` Task 1's DTOs (already on
`phase-v1-ship-1`).

## §2 Scope

**Produce** (exactly one commit) — create the `get_source` handler,
declare its module, register its route:

1. `crates/api/api/src/site/source.rs` **(NEW FILE)** — create with
   the **verbatim** body from plan §13 Task 3 IMPLEMENT file 1 / §10.3:

   ```rust
   use actix_web::web::Json;
   use lemmy_db_views_site::api::GetSourceResponse;
   use lemmy_utils::error::LemmyResult;

   const AGPL_NOTICE: &str = include_str!("../../../../../AGPL-NOTICE.md");

   /// `GET /api/v4/source` — returns the verbatim AGPL-NOTICE.md body.
   ///
   /// Public endpoint; no auth required. Read-only; no DB access.
   /// Surfaces the AGPL §13 source-disclosure requirement to any connecting client.
   pub async fn get_source() -> LemmyResult<Json<GetSourceResponse>> {
     Ok(Json(GetSourceResponse {
       notice: AGPL_NOTICE.to_string(),
       license: "AGPL-3.0".to_string(),
     }))
   }
   ```

2. `crates/api/api/src/site/mod.rs` — append ONE line (plan: 8 lines at
   HEAD, new line is line 9). Verify with
   `cat crates/api/api/src/site/mod.rs` first (line count is the
   contract anchor):

   ```rust
   pub mod source;
   ```

3. `crates/api/routes/src/lib.rs` — two sub-edits, verbatim from plan
   §13 Task 3 IMPLEMENT file 3:
   - **Add `source::get_source,`** to the `lemmy_api::site::{...}`
     use-block (plan cites lines 107-124), as a new line in the
     `site::` group **alphabetically after
     `registration_applications::{...},`**. Verify the use-block
     location with `grep -n "registration_applications::" crates/api/routes/src/lib.rs`.
   - **Add `.route("/source", get().to(get_source))`** immediately
     after the `/site` scope's closing `,` (plan cites line 230) and
     immediately **before** the existing
     `.route("/modlog", get().to(get_mod_log))` (plan cites line 231).
     Verify with `grep -n '\.route("/modlog"' crates/api/routes/src/lib.rs`
     and read the surrounding context. The route is at the **`/api/v4`
     scope level, NOT inside `/site`** (GOTCHA 3 — getting this wrong
     produces `/api/v4/site/source` which contradicts the PRD).
     Target shape (context around the `/site` scope close):

     ```rust
             .route("/banner", delete().to(delete_site_banner)),
           )
           // AGPL §13 source disclosure — public, no auth, no DB
           .route("/source", get().to(get_source))
           .route("/modlog", get().to(get_mod_log))
     ```

**Commit message** (exactly):
`feat(api): add get_source handler + /api/v4/source route (task 3)`

**Do NOT** touch any Task 2 file:
`crates/db_views/site/src/api.rs`,
`crates/api/api_crud/src/site/read.rs`,
`crates/api/api_crud/build.rs`. Do NOT touch `e2e.rs` (Task 4).

## §3 Required reading

In this order:

1. **`.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 Task 3** (lines
   ~892-991) — the canonical step list + all 5 GOTCHAs (read every
   one; GOTCHA 1/3/4 are load-bearing).
2. **Plan §10.3, §10.5** — the federated_instances minimal-handler
   MIRROR + the site-scope sibling-route precedent.
3. **Plan §11** (lines ~570-617) — the FILES YAML confirming your 3
   files are disjoint from Task 2's 3 files.
4. **MIRROR ref:** `crates/api/routes/src/lib.rs:288`
   `.route("/federated_instances", get().to(get_federated_instances))`
   — the canonical "sibling route at `/api/v4` scope level, NOT inside
   `/site`" precedent. Your `/source` route mirrors this shape +
   placement. Also lines 107-124 (the existing
   `lemmy_api::site::{...}` use-block you extend).
5. **MIRROR ref:** the existing `federated_instances` handler in
   `crates/api/api/src/site/` — the minimal no-param/no-Data/no-Query
   handler shape your `get_source` mirrors.
6. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace
   denies `unwrap`/`expect`/`#[allow]` under `-D warnings` (your
   handler uses `?`-free `Ok(Json(...))` — clean).
7. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — capture
   cargo to file, check exit, tail separately.
8. **`.claude/lessons/feedback_plan_baseline_self_reference.md`** —
   plan-cited line numbers (mod.rs 8/9, lib.rs 107-124/230/231) may
   have drifted; follow grep, not the literal line.

> **PMD note:** project-memory MCP not wired in this lane worktree
> (gitignored, not propagated to `git worktree add`). §2.3 hybrid PMD
> search substituted by the direct `.claude/lessons/` reads above. No
> mandatory §2.4 file-class row fires (Task 3 = new handler + mod decl
> + route reg; no e2e.rs / migration / new feature-gate / newtype /
> 2+-DB-write transaction handler — the handler is read-only, no DB).

## §3a Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: ab897de07
    filesCreated: []
    filesModified:
      - crates/db_views/site/src/api.rs
    keyDecisions:
      - "Appended SourceDisclosure, GetSource, GetSourceResponse structs AFTER GetSiteResponse (verbatim plan §10.1). GetSourceResponse { notice: String, license: String } is the type your handler returns."
      - "Task 1 validated PASS via §5.2 advisor-laptop (DQ #238 resolved): cargo check 0err, clippy 0warn, test --no-run e2e exe built."
    notes: "GetSourceResponse is now present + compiles in lemmy_db_views_site::api. Your handler `use lemmy_db_views_site::api::GetSourceResponse;` resolves. AGPL-NOTICE.md confirmed present at repo root (3595 bytes, Task 0 Probe 5) — your include_str! target exists."
```

## §4 Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-ship-1`. Finalize
  merges your worktree branch back; do NOT push to `phase-v1-ship-1`
  directly.
- **One commit.** clippy/check failure on the advisor-laptop run is a
  real signal the advisor triages — do NOT `#[allow]`-spam or split.
- Mid-task DQ visibility: if you raise a `pending` entry, `git add
  .claude/decision-queue.json && git commit && git push origin
  <worktree-branch>` immediately. Compute `next_id` across
  `.claude/decision-queue.json` + `.claude/decision-queue-archive-*.json`
  (DQ #50 collision lesson — live max id currently 238; archives may
  hold higher; **NOTE: Task 2 runs in parallel and will also raise a
  validate-pending-laptop DQ — compute next_id defensively; if you and
  Task 2 collide on an id, the advisor renumbers at finalize-merge,
  but pick the highest you can see**).
- Attribution: `from: "impl"`, `answered_by: null`. NEVER
  `from: "advisor"` / `answered_by: "advisor"` / `answered_by: "user"`
  / `kind: "clarify"`.

### Cohort-A file disjointness (HARD)

You are Cohort A member 2, parallel with Task 2. Task 2 owns
`crates/db_views/site/src/api.rs`,
`crates/api/api_crud/src/site/read.rs`,
`crates/api/api_crud/build.rs`. **You must NOT read-modify-write any
of those.** Your files (`crates/api/api/src/site/source.rs`,
`crates/api/api/src/site/mod.rs`, `crates/api/routes/src/lib.rs`) are
disjoint from Task 2's by construction (plan §11 verified
`intersect == ∅`). If you find yourself needing to edit a Task 2 file,
STOP and raise a `kind: "blocker"` DQ — the plan's `[P]` disjointness
assumption would be wrong.

### Plan-cited line numbers may have drifted

Plan cites mod.rs lines 8/9, lib.rs 107-124/230/231. `grep -n` for the
anchors (`registration_applications::`, `.route("/modlog"`,
`.route("/federated_instances"`) and edit by symbol/context, not the
literal line number (per `feedback_plan_baseline_self_reference.md`).

### GOTCHA digest (read §13 Task 3 for full text)

- **G1:** `include_str!` is relative to the SOURCE FILE. From
  `crates/api/api/src/site/source.rs`, **5 `..` segments** reach repo
  root: `include_str!("../../../../../AGPL-NOTICE.md")`. Verbatim —
  re-count if you doubt it, but the plan verified 5.
- **G2:** `include_str!` is compile-time; if AGPL-NOTICE.md missing,
  build fails loudly (desired; Task 0 Probe 5 already confirmed it's
  present, 3595 bytes).
- **G3 (CRITICAL):** route at `/api/v4` scope level, NOT inside
  `/site`. Placement is between the `/site` scope's closing `)` and
  `.route("/modlog", ...)`. Inside `/site` → wrong URL
  `/api/v4/site/source`.
- **G4:** handler takes **NO parameters** — no `context:
  Data<LemmyContext>`, no `Query`. Keep minimal (no DB, no input).
- **G5:** `use actix_web::{guard, web::*};` at lib.rs line 1 already
  brings `get`/`scope` into scope. The ONLY new `use` is the
  `source::get_source,` entry in the `lemmy_api::site::{...}` block.

### Shape-G suspended until 2026-06-01 — validation handoff

**Shape G is suspended repo-wide** (DQ #228/#229). Plan §13 Task 3
VALIDATE is Shape-G form (push + `workflow_run_id`). **Override:**
after committing + pushing your worktree branch, write a
`kind: "validate-pending-laptop"` DQ entry (NOT `kind:
"validate-pending"`; NO `workflow_run_id` — `cargo-validate-workspace.yml`
is DISABLED). Entry shape per `.claude/rules/decision-queue.md`
§"kind: validate-pending-laptop":

```
{
  "id": <next_id across live + archives — pick highest visible; Task 2 parallel>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-ship-1 task 3 get_source handler+route+mod — laptop cargo validation pending",
  "branch": "<your worktree branch>",
  "phase_task": 3,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full",
    "bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings",
    "bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "context": "Task 3 created crates/api/api/src/site/source.rs (get_source handler) + mod.rs decl + lib.rs route/use; awaiting advisor-laptop §5.2 validate-pending-laptop run.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit + push that DQ entry on your worktree branch (subject:
`chore(decision-queue): impl raised DQ #<id> — v1-ship-1 task3 validate-pending-laptop`).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 3 authors no e2e test. No `LemmyResult` /
`Box<dyn Error>` Case A/B/C decision (Task 4, pre-resolved Case A in
plan §10.6 GOTCHA 1). NOTE: your handler returns
`LemmyResult<Json<GetSourceResponse>>` — that is the handler's normal
signature (mirrors `federated_instances`), NOT a test Case decision.

## §5 Validation gates

Shape G suspended. Sequence:
1. Create source.rs + edit mod.rs + edit lib.rs per §2 (one commit,
   exact commit message).
2. Sanity self-check: `grep -n "source::get_source" crates/api/routes/src/lib.rs`
   (must appear once in the use-block) and
   `grep -n '\.route("/source"' crates/api/routes/src/lib.rs` (must
   appear once, at `/api/v4` scope level NOT inside `/site` — verify
   the surrounding context per GOTCHA 3).
3. Push your worktree branch.
4. Write the `kind: "validate-pending-laptop"` DQ entry per §4.
5. Return §6 expected output. Do NOT run cargo yourself (worker
   `MemoryMax=10G`; advisor-laptop runs cargo).

## §6 Expected output (return to advisor)

```
## Task 3 complete — v1-ship-1 get_source handler + route

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/site/source.rs (NEW, +<n> lines, get_source handler, include_str! AGPL-NOTICE.md 5x ..)
  - crates/api/api/src/site/mod.rs (+1 line `pub mod source;` at line <actual>)
  - crates/api/routes/src/lib.rs (+source::get_source to use-block; +.route("/source") at /api/v4 scope level before /modlog; lines <actual>)
**Self-check:** source::get_source in use-block ✓; .route("/source") at /api/v4 scope (NOT /site) ✓
**Validation:** kind:"validate-pending-laptop" DQ #<id> raised on <worktree-branch> (Shape G suspended — advisor-laptop runs cargo)
**Next:** advisor runs §5.2 validate-pending-laptop; cohort barrier waits on Task 2 + Task 3 both pass
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

One deviation, session-state-driven (not a plan defect):

1. **VALIDATE block: Shape-G → Shape-G-suspended-laptop.** Plan §13
   Task 3's VALIDATE block captures a `workflow_run_id` from
   `cargo-validate-workspace.yml`. That workflow is DISABLED repo-wide
   until 2026-06-01 (DQ #228/#229; bootstrap §5; commit `086cfa5d4`).
   Per `.claude/rules/decision-queue.md` §"kind:
   validate-pending-laptop" + `.claude/rules/advisor-orchestrator.md`
   §5.2, impl-task writes `kind: "validate-pending-laptop"` with the
   §15 DoD commands verbatim; the advisor laptop session runs them.
   Standing session directive, not a plan correction — plan design +
   IMPLEMENT blocks unchanged.
