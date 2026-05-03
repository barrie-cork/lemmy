---
role: impl-task
plan_task: 7
phase: v1-SL-a
cohort: A
created: 2026-05-03
related_dq: []
---

# Brief — v1-SL-a Task 7 [P] — UPDATE 3 files atomically — 5 new ENTRY_KIND consts + registry section

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 7 — see .claude/PRPs/briefs/sl-a-impl-7.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 7
of cohort A on `phase-v1-SL-a`. Cohort A = Tasks 1+2+3+6+7 (5-way
parallel); you own three files (atomic commit) — none of which
overlap with any other cohort A task.

## 2. Scope

**Produce:**

1. ONE atomic commit modifying **three files together**:
   - `crates/db_schema/src/source/governance/governance_log.rs` —
     define 5 new `ENTRY_KIND_*` consts after the v1-JM-a block at
     line 185.
   - `crates/api/api/src/governance/governance_log.rs` — re-export
     5 new consts at strict-alphabetical positions in lines 39–76.
   - `.claude/rules/governance-log-entry-kind-registry.md` —
     replace the `### sponsor-liability-v1 (reserved ...)` stub at
     lines 160–165 with 5 populated rows; update Acceptance
     invariants count from 33 → 38 at lines 187–193.
2. ONE `kind: "validate-pending"` DQ entry referencing
   `cargo-validate-workspace.yml`.
3. Push your worker branch. Junior daemon finalize-merges into
   `phase-v1-SL-a`.

**Do NOT** in this task:

- Touch any other file (no `migrations/**`, no `enums.rs`, no
  `schema.rs`, no `config.rs`, no `tests/**`, no other `crates/**`
  files, no other `.claude/**` files). Three files exact.
- Split the three files into separate commits. Per plan Task 7
  GOTCHA: schema definition without shim re-export breaks api-path
  callers; shim re-export without definition fails to compile;
  registry update without either is informational drift. THE THREE
  FILES LAND TOGETHER.
- Insert new const names out of strict alphabetical order in the
  shim file. Per plan Task 7 GOTCHA: `pub use` ordering is STRICT
  alphabetical, not "alphabetical within the new additions".
- Forget the `(pending)` marker + downstream-plan citation in the
  registry. Each of the 5 new rows MUST cite its emitting plan/site.
- Run cargo locally for validation. Shape G — workspace-check fires
  on push.

## 3. Required reading

Read in this order before writing any code:

1. **Plan Task 7 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1836–1912 — task header, FILES YAML, IMPLEMENT directives,
   GOTCHAs (4 of them).
2. **Plan §10.7 (canonical 3-file edit shape):**
   `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 996–1066
   for the verbatim source — including the exact alphabetical
   positions for the api-shim file.
3. **Plan §"v1-SL-a entry kinds (5, this sub-phase)":** lines 1045–
   1101 — describes the 5 ENTRY_KIND const names and their
   downstream emitters (SL-d submit_jury_vote.rs, SL-c
   sponsor_liability_grace.rs, SL-b revoke_endorsement.rs,
   SL-restoration restoration_complete.rs). The registry rows MUST
   match these emitter assignments.
4. **Read each file you'll edit FIRST:**
   - `crates/db_schema/src/source/governance/governance_log.rs` —
     confirm line 185 has
     `pub const ENTRY_KIND_SEVERITY_TIER_FROZEN: &str = "severity_tier_frozen";`
     (the v1-JM-a block tail).
   - `crates/api/api/src/governance/governance_log.rs:39-76` —
     confirm the existing `pub use` block is alphabetically sorted.
     Read all 33 entries (current count) to determine each new
     const's correct insertion position.
   - `.claude/rules/governance-log-entry-kind-registry.md:160-165` —
     confirm the `### sponsor-liability-v1 (reserved ...)` stub
     exists. Confirm Acceptance invariants section at lines 187–193
     currently claims `33`.
5. **`.claude/lessons/feedback_handover_trailer_cohort_propagation.md`**
   — this task's `keyDecisions` propagate to cohort B (Tasks 4+5)
   via the brief-injection step.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 0
    commit: 1f6131bf823f006bd2ace06b2931e2c1b56cd8ad
    filesCreated: []
    filesModified:
      - scripts/brehon/migrate-roundtrip.sh
    keyDecisions:
      - replaced stub per DQ #114 (Option A: Task 0 of SL-a)
      - "lemmy_diesel_utils binary surface — no CLI sub-commands; uses
        LEMMY_DATABASE_URL env var only (binary at
        crates/diesel_utils/src/main.rs reads from env, takes no positional
        args). Adapted plan §13 GOTCHA cargo invocations accordingly."
    notes: |
      Probe 2 ENTRY_KIND_ count = 33 (PASS) on phase-v1-SL-a tip — confirms
      Task 7 starts from 33 and lands at 38.
```

Task 0's Probe 2 result is your pre-edit baseline: ENTRY_KIND_ count
= 33 → post-edit count = 38.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (tip `1b47a3ff4`).
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit (3 files in one commit). Daemon finalize
  absorbs into `phase-v1-SL-a`.

### File discipline

- Files you may MODIFY (atomic — same commit):
  - `crates/db_schema/src/source/governance/governance_log.rs`
  - `crates/api/api/src/governance/governance_log.rs`
  - `.claude/rules/governance-log-entry-kind-registry.md`
- **Files you may NOT touch:** any other file in the repo.

### Edit discipline (per file)

**File 1: `crates/db_schema/src/source/governance/governance_log.rs`**

- Use Edit (not Write). Append 5 new `pub const ENTRY_KIND_*`
  declarations AFTER line 185 (`ENTRY_KIND_SEVERITY_TIER_FROZEN`).
- Verbatim from plan §10.7 — including doc-comments citing the
  emitting downstream plan.
- Const name ↔ string-literal mapping (per plan §10.7):
  - `ENTRY_KIND_SPONSOR_LIABILITY_PENDING = "sponsor_liability_pending"`
  - `ENTRY_KIND_SPONSOR_LIABILITY_FIRED = "sponsor_liability_fired"`
  - `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED = "sponsor_liability_escaped"`
  - `ENTRY_KIND_ENDORSEMENT_REVOKED = "endorsement_revoked"`
  - `ENTRY_KIND_RESTORATION_COMPLETED = "restoration_completed"`
  String literals must be lower_snake_case (matches existing
  ENTRY_KIND convention).

**File 2: `crates/api/api/src/governance/governance_log.rs:39-76`**

- Use Edit (not Write). Insert 5 new `pub use` lines at the strict-
  alphabetical positions per plan §10.7:
  - `ENTRY_KIND_ENDORSEMENT_REVOKED` (alphabetically falls before
    `ENTRY_KIND_ESCALATION_*` if present, etc — read the existing
    block to compute the exact position).
  - `ENTRY_KIND_RESTORATION_COMPLETED` (between `RESCIND_*` and
    `REVOKE_*` if present).
  - `ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED`,
    `ENTRY_KIND_SPONSOR_LIABILITY_FIRED`,
    `ENTRY_KIND_SPONSOR_LIABILITY_PENDING` (alphabetical within
    SPONSOR_* — note "ESCAPED" < "FIRED" < "PENDING").
- Strict alphabetical ordering: insert each name at its position in
  the existing sorted block. Plan §10.7 documents the specific
  positions; if the live file's existing ordering disagrees with
  plan §10.7's positions (a real possibility if v1-JM-e changed
  the block since plan-write), **adapt to live ordering** + document.

**File 3: `.claude/rules/governance-log-entry-kind-registry.md`**

- Use Edit. Replace lines 160–165 (the stub) with 5 populated rows
  per plan §10.7. Each row MUST contain:
  - `ENTRY_KIND_*` const name
  - String literal value
  - Description (one sentence)
  - "Emitting handler" column with `(pending)` marker
  - Downstream-plan citation matching plan Task 7 GOTCHA
    "registry pre-landed-const exemption":
    - PENDING → SL-d submit_jury_vote.rs
    - FIRED → SL-c sponsor_liability_grace.rs
    - ESCAPED → SL-b revoke_endorsement.rs AND SL-c
      sponsor_liability_grace.rs
    - ENDORSEMENT_REVOKED → SL-b revoke_endorsement.rs
    - RESTORATION_COMPLETED → restorative-mechanics-v1 PRD
      restoration_complete.rs
- ALSO update Acceptance invariants section at lines 187–193:
  change the count claim from `33` to `38`.

### Post-edit count invariant (verify before commit)

Per plan Task 7 GOTCHA "count invariant — verify post-commit":

```bash
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# expect 38

rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
# expect 38

rg -n '"[a-z_]+"' crates/db_schema/src/source/governance/governance_log.rs | \
  awk -F: '/ENTRY_KIND_/ {print}' | grep -oE '"[a-z_]+"' | sort | uniq -d
# expect empty (no duplicate string literals)
```

If any of these don't match expectations, DO NOT commit. Revisit
the file before continuing. If unable to reconcile, file a DQ.

### Shape G discipline (no local cargo)

- Push your worker branch to `junior/<task-slug>`. The push to
  `crates/**` (and `.claude/**`) triggers
  `cargo-validate-workspace.yml`. Migration workflow does NOT
  trigger.
- Capture the workflow run id:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  ```
- Write ONE `kind: "validate-pending"` DQ entry: `from: "impl"`,
  `kind: "validate-pending"`, `workflow_run_id: <id>`, `branch:
  "<your branch>"`, `phase_task: 7`, `result: null`, etc. `context`
  says "workspace-check for v1-SL-a task 7".
- Commit + push the DQ entry to your worker branch immediately per
  "Mid-task visibility". Two commits.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Compute next id: scan **both** live `pending`/`resolved` AND any
  `decision-queue-archive-*.json`. **Cohort A coordination:** Tasks
  1 + 2 + 3 + 6 + 7 are parallel; re-read live `decision-queue.json`
  immediately before computing next id.

### Commit shape

- ONE source-code commit (3 files together):
  - Subject: `feat(v1-SL-a): add 5 ENTRY_KIND consts (db_schema define + api shim re-export) + registry v1-SL-a section (task 7)` (verbatim per plan §13 line 1912).
  - Body: include the `HANDOVER:` YAML trailer:
    ```yaml
    HANDOVER:
    filesCreated: []
    filesModified:
      - crates/db_schema/src/source/governance/governance_log.rs
      - crates/api/api/src/governance/governance_log.rs
      - .claude/rules/governance-log-entry-kind-registry.md
    keyDecisions:
      - "added 5 ENTRY_KIND_* consts (PENDING/FIRED/ESCAPED/ENDORSEMENT_REVOKED/RESTORATION_COMPLETED)"
      - "atomic 3-file commit (schema define + api shim re-export + registry section) per plan §10.7 GOTCHA"
      - "registry rows carry (pending) + emitting-plan citations per registry Acceptance invariant 192"
      - "Acceptance invariants count: 33 → 38"
      - <strict-alphabetical insertion positions in api shim, with adapted-to-live-ordering note if applicable>
    notes: <post-edit count results — db_schema=38, api shim=38, no duplicate string literals>
    ```
- THEN the DQ-entry commit (separate).
- Files in source-code commit: ONLY the three named files.

## 5. Validation gate

Plan §15 names workspace-check (run in GH Actions on push):

- `cargo check --workspace --features full`.
- `cargo clippy --workspace --features full --no-deps -- -D warnings`.

**Cohort A barrier semantics:** workspace-check on YOUR branch may
fail because cohort A Tasks 4 and 5 haven't landed. The new
ENTRY_KIND consts you add do NOT introduce non-exhaustive matches
on their own (no `match k { ENTRY_KIND_FOO => ... }` declarations
exist for governance_log entry kinds at present). However, the
combined effect of cohort A tasks (Task 2's CaseStatus extension +
Task 3's column additions + your ENTRY_KIND additions) means
workspace-check will fail until Tasks 4 + 5 land. Plan §14 Story 1
checkpoint = workspace-check `conclusion: "success"` on **Task 5's
push**.

If workspace-check fails on a Task-7-specific error (typo in const
name, wrong string literal, broken alphabetical ordering, registry
markdown malformed, count invariant breach), advisor catch-fires.

## 6. Expected output (return to advisor)

```
## Task 7 complete — v1-SL-a 5 ENTRY_KIND consts + registry section

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): add 5 ENTRY_KIND consts ... (task 7)
  - <sha-b> chore(decision-queue): impl raised DQ #<N> — sl-a-task-7 validate-pending
**Files (atomic commit):**
  - crates/db_schema/src/source/governance/governance_log.rs (+5 const declarations)
  - crates/api/api/src/governance/governance_log.rs (+5 pub use lines, strict-alphabetical)
  - .claude/rules/governance-log-entry-kind-registry.md (5 stub→populated rows + count 33→38)
**5 ENTRY_KIND consts added:**
  - PENDING / FIRED / ESCAPED / ENDORSEMENT_REVOKED / RESTORATION_COMPLETED
**Post-edit count invariants:**
  - db_schema ENTRY_KIND_ count: 38 (PASS)
  - api shim ENTRY_KIND_ count: 38 (PASS)
  - no duplicate string literals (PASS)
**Registry rows:** all 5 carry (pending) + emitting-plan citation
**Workflow run captured:**
  - workspace-check workflow_run_id: <int> → DQ #<N>
**Expected workspace-check outcome:** dependent on Tasks 4/5 (cohort B); cohort barrier holds until Task 5 push
**Next:** advisor dispatches 1 ci-watcher; cohort A advances when all 5 task validate-pending entries resolve
```

If any pre-write check failed (line numbers diverge past adaptation,
post-edit count invariants fail, registry rows missing required
columns), replace with a DQ catch-fire entry committed + pushed to
your worker branch immediately.
