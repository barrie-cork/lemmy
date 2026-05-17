---
phase: v1-ship-1
role: impl-task
task: 2
brief_n: 2
plan: .claude/PRPs/plans/v1-ship-1-r1.plan.md
created: 2026-05-17
related_dq: null
cohort: A
cohort_peers: [3]
---

# [role:impl-task] v1-ship-1 task 2 — see .claude/PRPs/briefs/v1-ship-1-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-ship-1 task 2 — see .claude/PRPs/briefs/v1-ship-1-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 2
from `.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 (Task 2 `[P]`, lines
~787-890). This is **Cohort A member 1** (parallel with Task 3, which
is file-disjoint — you will NOT touch any file Task 3 touches and
vice-versa). Both `requires:` Task 1's DTOs (already on
`phase-v1-ship-1`).

## §2 Scope

**Produce** (exactly one commit) — atomically add the
`source_disclosure` field, populate it, and create the build script:

1. `crates/db_views/site/src/api.rs` — add ONE field
   `source_disclosure: SourceDisclosure` to `GetSiteResponse`, appended
   **after** the `captcha_enabled: bool` field, before the struct's
   closing `}`. Plan cites struct keyword at line 337, closing `}` at
   line 355, `captcha_enabled` at line 354 — **verify with
   `grep -n "captcha_enabled: bool" crates/db_views/site/src/api.rs`
   first; the symbol is the contract, the line may have drifted** (it
   did drift in Task 1 — Task 1's 3 structs were appended after line
   355, so `GetSiteResponse` body is unchanged but absolute line
   numbers shifted). Verbatim addition from plan §13 Task 2 IMPLEMENT
   file 1:

   ```rust
     pub captcha_enabled: bool,
     /// AGPL §13 source-disclosure surface; see [`SourceDisclosure`].
     pub source_disclosure: SourceDisclosure,
   ```

2. `crates/api/api_crud/src/site/read.rs` — three sub-edits, verbatim
   from plan §13 Task 2 IMPLEMENT file 2:
   - **Update the import** (plan cites line 16): from
     `use lemmy_db_views_site::{SiteView, api::GetSiteResponse};` to
     `use lemmy_db_views_site::{SiteView, api::{GetSiteResponse, SourceDisclosure}};`
   - **Add three private consts** (plan cites between line 18
     `use std::sync::LazyLock;` and line 20 `pub async fn get_site`):

     ```rust
     const BREHON_FORK_COMMIT: &str = env!("BREHON_FORK_COMMIT");
     const BREHON_REPO_URL: &str = "https://github.com/barrie-cork/lemmy";
     const SOURCE_DISCLOSURE_URL: &str = "/api/v4/source";
     ```
   - **Extend the `Ok(GetSiteResponse { ... })` constructor** (plan
     cites lines 57-70; append after `captcha_enabled:
     is_captcha_plugin_loaded(),`):

     ```rust
         captcha_enabled: is_captcha_plugin_loaded(),
         source_disclosure: SourceDisclosure {
           license: "AGPL-3.0".to_string(),
           repo_url: BREHON_REPO_URL.to_string(),
           fork_commit: BREHON_FORK_COMMIT.to_string(),
           disclosure_url: SOURCE_DISCLOSURE_URL.to_string(),
         },
       })
     ```

3. `crates/api/api_crud/build.rs` **(NEW FILE)** — create with the
   **verbatim** body from plan §13 Task 2 IMPLEMENT file 3 / §10.4:

   ```rust
   fn main() {
     let commit = std::env::var("BREHON_FORK_COMMIT")
       .or_else(|_| {
         std::process::Command::new("git")
           .args(["rev-parse", "HEAD"])
           .output()
           .ok()
           .filter(|out| out.status.success())
           .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
           .ok_or(std::env::VarError::NotPresent)
       })
       .unwrap_or_else(|_| "unknown".to_string());
     println!("cargo:rustc-env=BREHON_FORK_COMMIT={commit}");
     println!("cargo:rerun-if-env-changed=BREHON_FORK_COMMIT");
     println!("cargo:rerun-if-changed=../../../.git/HEAD");
   }
   ```

**Commit message** (exactly):
`feat(api_crud): wire source_disclosure into GetSiteResponse + build.rs (task 2)`

**Do NOT** touch any Task 3 file: `crates/api/api/src/site/source.rs`,
`crates/api/api/src/site/mod.rs`, `crates/api/routes/src/lib.rs`. Do
NOT touch `e2e.rs` (Task 4).

## §3 Required reading

In this order:

1. **`.claude/PRPs/plans/v1-ship-1-r1.plan.md` §13 Task 2** (lines
   ~787-890) — the canonical step list + all 6 GOTCHAs (read every
   GOTCHA; GOTCHA 2/3/4/6 are load-bearing).
2. **Plan §10.1, §10.2, §10.4** — the DTO derive shape, the read_site
   constructor MIRROR, the verbatim build.rs body.
3. **Plan §11 + §11 R9** (lines ~570-617) — the FILES YAML +
   caller-enumeration. R9 confirms `read_site` is the ONLY constructor
   of `GetSiteResponse`; the `routes_v3/handlers.rs:249` destructure is
   `..` rest-pattern (additive-safe, NO edit needed — DQ #227
   RESOLVED).
4. **MIRROR ref:** `crates/db_views/site/src/api.rs` — the `SourceDisclosure`
   struct Task 1 added (for the field type) + the existing
   `GetSiteResponse` for the field-add location.
5. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace
   denies `unwrap`/`expect`/`#[allow]` under `-D warnings` (the
   build.rs uses `unwrap_or_else`, which is fine — not `unwrap()`).
6. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — capture
   cargo to file, check exit, tail separately.
7. **`.claude/lessons/feedback_plan_baseline_self_reference.md`** —
   plan-cited line numbers may have drifted (esp. after Task 1's
   append); follow grep, not the literal line number.

> **PMD note:** project-memory MCP is not wired in this lane worktree
> (gitignored, not propagated to `git worktree add`). §2.3 hybrid PMD
> search substituted by the direct `.claude/lessons/` reads above. No
> mandatory §2.4 file-class row fires (no e2e.rs / migration / new
> feature-gate / newtype / transaction-handler / wrapper edit — Task 2
> is struct-field-add + const + build.rs).

## §3a Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: ab897de07
    filesCreated: []
    filesModified:
      - crates/db_views/site/src/api.rs
    keyDecisions:
      - "Appended SourceDisclosure, GetSource, GetSourceResponse structs AFTER GetSiteResponse's closing brace (verbatim plan §10.1 bodies); OMITTED #[skip_serializing_none] (no Option fields)."
      - "Task 1 validated PASS via §5.2 advisor-laptop: cargo check 0err, clippy 0warn, test --no-run e2e exe built (DQ #238 resolved)."
    notes: "Task 1's append shifted absolute line numbers in api.rs; GetSiteResponse struct body itself is unchanged. Use grep for captcha_enabled, not plan line 354. SourceDisclosure type is now present + compiles for your field-add."
```

## §4 Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-ship-1`. Finalize
  merges your worktree branch back; do NOT push to `phase-v1-ship-1`
  directly.
- **One commit.** If clippy/check fails on the advisor-laptop run,
  that's a real signal the advisor triages — do NOT `#[allow]`-spam or
  split commits.
- Mid-task DQ visibility: if you raise a `pending` entry, `git add
  .claude/decision-queue.json && git commit && git push origin
  <worktree-branch>` immediately. Compute `next_id` across
  `.claude/decision-queue.json` + `.claude/decision-queue-archive-*.json`
  (DQ #50 collision lesson — live max id is currently 238; archives
  may hold higher).
- Attribution: `from: "impl"`, `answered_by: null`. NEVER
  `from: "advisor"` / `answered_by: "advisor"` / `answered_by: "user"`
  / `kind: "clarify"`.

### Cohort-A file disjointness (HARD)

You are Cohort A member 1, parallel with Task 3. Task 3 owns
`crates/api/api/src/site/source.rs`, `crates/api/api/src/site/mod.rs`,
`crates/api/routes/src/lib.rs`. **You must NOT read-modify-write any of
those.** Your files (`crates/db_views/site/src/api.rs`,
`crates/api/api_crud/src/site/read.rs`, `crates/api/api_crud/build.rs`)
are disjoint from Task 3's by construction (plan §11 verified
`intersect == ∅`). If you find yourself needing to edit a Task 3 file,
STOP and raise a `kind: "blocker"` DQ — that means the plan's `[P]`
disjointness assumption is wrong.

### Plan-cited line numbers may have drifted

The plan cites specific lines (api.rs 337/354/355, read.rs 16/18/20/57-70).
**Task 1 already appended 3 structs to api.rs** — absolute lines below
the append shifted. `grep -n` for the anchor symbols
(`captcha_enabled: bool`, `use lemmy_db_views_site::{SiteView`,
`use std::sync::LazyLock;`, `is_captcha_plugin_loaded(),`) and edit by
symbol, not by the plan's literal line number (per
`feedback_plan_baseline_self_reference.md`). Symbol presence is the
contract.

### GOTCHA digest (read §13 Task 2 for full text)

- **G2/G3:** `lemmy_api_crud` does NOT depend on `lemmy_api_routes`
  (verified); do NOT re-export `BREHON_FORK_COMMIT` through routes —
  circular dep. Const stays private in `read.rs`.
- **G3:** after commit, `rg "GetSiteResponse \{" crates/ --include="*.rs"`
  must show exactly 3 lines (def + routes_v3 rest-pattern destructure +
  the one read.rs constructor). Run this check yourself.
- **G4:** `routes_v3/handlers.rs:249-258` uses `..` rest-pattern —
  additive-safe, NO edit to that file.
- **G6:** `rerun-if-changed=../../../.git/HEAD` — 3 `..` from
  `crates/api/api_crud/` reaches repo root. Verbatim; do not "fix" it.

### Shape-G suspended until 2026-06-01 — validation handoff

**Shape G is suspended repo-wide** (DQ #228/#229). The plan §13 Task 2
VALIDATE block is written in Shape-G form (`git push origin
junior/v1-ship-1-task-2` + capture `workflow_run_id`). **Override:**
after committing + pushing your worktree branch, write a
`kind: "validate-pending-laptop"` DQ entry (NOT `kind:
"validate-pending"`; do NOT capture a `workflow_run_id` — the GH
Actions workflow `cargo-validate-workspace.yml` is DISABLED). Entry
shape per `.claude/rules/decision-queue.md` §"kind:
validate-pending-laptop":

```
{
  "id": <next_id across live + archives>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-ship-1 task 2 source_disclosure field+build.rs — laptop cargo validation pending",
  "branch": "<your worktree branch>",
  "phase_task": 2,
  "commands": [
    "bash scripts/brehon/cargo-check.sh --workspace --features full",
    "bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings",
    "bash scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "context": "Task 2 added source_disclosure field to GetSiteResponse + populated in read_site + created crates/api/api_crud/build.rs; awaiting advisor-laptop §5.2 validate-pending-laptop run.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit + push that DQ entry on your worktree branch (subject:
`chore(decision-queue): impl raised DQ #<id> — v1-ship-1 task2 validate-pending-laptop`).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 2 authors no e2e test. No `LemmyResult` /
`Box<dyn Error>` Case A/B/C decision (that is Task 4, pre-resolved as
Case A in plan §10.6 GOTCHA 1).

## §5 Validation gates

Shape G suspended. Sequence:
1. Make the 3-file change per §2 (one commit, exact commit message).
2. Run the §11 R9 self-check: `rg "GetSiteResponse \{" crates/
   --include="*.rs"` — confirm exactly 3 lines (def, routes_v3
   rest-pattern, read.rs constructor). If not 3, you introduced an
   extra constructor or broke a site — STOP and re-read GOTCHA 3.
3. Push your worktree branch (the daemon-cut `junior/...` branch).
4. Write the `kind: "validate-pending-laptop"` DQ entry per §4.
5. Return the §6 expected output. Do NOT run cargo yourself (worker
   has `MemoryMax=10G`; cargo on worker forbidden — advisor-laptop
   runs it).

## §6 Expected output (return to advisor)

```
## Task 2 complete — v1-ship-1 source_disclosure field + build.rs

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/db_views/site/src/api.rs (+1 field source_disclosure on GetSiteResponse, after captcha_enabled at line <actual>)
  - crates/api/api_crud/src/site/read.rs (import + 3 consts + constructor field; lines <actual>)
  - crates/api/api_crud/build.rs (NEW, +<n> lines, BREHON_FORK_COMMIT injection)
**R9 self-check:** `rg "GetSiteResponse \{"` → 3 lines (def + routes_v3 rest-pattern + read.rs ctor) ✓
**Validation:** kind:"validate-pending-laptop" DQ #<id> raised on <worktree-branch> (Shape G suspended — advisor-laptop runs cargo)
**Next:** advisor runs §5.2 validate-pending-laptop; cohort barrier waits on Task 2 + Task 3 both pass
```

Plus any DQ #N references if you raised a blocker mid-task.

## §7 Why this brief differs from the plan

One deviation, session-state-driven (not a plan defect):

1. **VALIDATE block: Shape-G → Shape-G-suspended-laptop.** Plan §13
   Task 2's VALIDATE block captures a `workflow_run_id` from
   `cargo-validate-workspace.yml`. That workflow is DISABLED repo-wide
   until 2026-06-01 (DQ #228/#229; bootstrap §5; commit `086cfa5d4`).
   Per `.claude/rules/decision-queue.md` §"kind:
   validate-pending-laptop" + `.claude/rules/advisor-orchestrator.md`
   §5.2, impl-task writes `kind: "validate-pending-laptop"` with the
   §15 DoD commands verbatim instead; the advisor laptop session runs
   them. This is the standing session directive, not a plan
   correction — the plan's design + IMPLEMENT blocks are unchanged.
