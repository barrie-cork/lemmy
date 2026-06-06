---
phase: m2-rooms-a
role: impl-task
n: 1w
authored: 2026-06-06
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-rooms-a
task_number: 1w
requires: [1]
minimax_trial: not-eligible (workspace producer + Diesel join; >2 logical files, not MIRROR-ref-trivial)
mandatory_lessons_fired:
  - feedback_features_full_workspace_only.md            # §2.4 — workspace task; --features full, never -p <crate> --features full
  - feedback_validate_pending_laptop_write_then_stop.md # R4 — write DQ + STOP; cargo runs on laptop
  - feedback_lemmy_error_no_std_error.md                # producer is LemmyResult; the new join method returns LemmyResult and is ?-propagated
gate_provenance: "plan-approval gate 1 + re-approval gate (§3.4 DoD smoke + §3.5 watchpoints) PASSED 2026-06-06; user approved dispatch. gov-v0 @ 4f3ca3ad0."
---

# [role:impl-task] m2-rooms-a Task 1w — augment CaseTransitionEvent with juror_pseudonyms (workspace producer)

## 1. Role + dispatch line

```
[role:impl-task] m2-rooms-a task-1w juror-pseudonyms producer — see .claude/PRPs/briefs/m2-rooms-a-impl-1w.md
```

## 2. Scope

**Task 1w (one commit).** Close the T2 juror-sourcing gap (DQ `a3d0e9941441-055`,
option-a): the Matrix bridge cannot learn which jurors to invite because
`CaseTransitionEvent` carries no identities and the bridge has no workspace-DB
access. This task makes the **binary producer** resolve the jurors and push
their pseudonyms into the event payload — exactly as the M1 DM-relay pushes
`brehon_sender`/`brehon_recipient` (`services/bridge/src/relay.rs:28-36`).

**Produce (exactly 2 files — the by-case join lives as a free fn in
`bridge_notify.rs`, keeping this to 2 crates):**

```yaml
modifies:
  - crates/api/api_common/src/governance.rs    # +pub juror_pseudonyms: Vec<String> on CaseTransitionEvent + doc-comment + #[serde(default)]
  - crates/api/api_utils/src/bridge_notify.rs  # add the by-case juror-pseudonym free fn + populate juror_pseudonyms for JurySelection; Vec::new() otherwise
requires:
  - task: 1   # serial dispatch (file-disjoint from T1, keeps daemon at ≤1 worker); no logical dependency on T1's bridge changes
```

**This is a WORKSPACE task** (`crates/**`, Lemmy toolchain). Write a
`validate-pending-laptop` DQ entry with
`commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`,
commit + push, then **STOP**. Do NOT run cargo on the daemon. (This is the
opposite toolchain from T1's bridge task — see §3 R8 note.)

**Do NOT:**
- Touch any file under `services/bridge/`, `crates/db_schema/migrations/`, `docs/`, or any test file.
- Add a workspace migration (the `jury_assignment` + `actor_pseudonym` tables already exist from v1-JM-b — NO schema change).
- Put a real username, email, or display-name into `juror_pseudonyms` — ONLY `actor_pseudonym.pseudonym` values (ADR-015). If you cannot resolve a juror to a pseudonym, omit it (do not fall back to a real identity).
- Reuse `crates/api/api/src/governance/actor_pseudonym_helper.rs` — it is in the `lemmy_api` crate, which `lemmy_api_utils` does NOT depend on (confirmed at the gate). It is unreachable from the producer.
- Add a 3rd crate's worth of changes (no edit to `crates/db_schema/`) — the join goes as a free fn in `bridge_notify.rs` using the `lemmy_db_schema` / `lemmy_db_schema_file` schema types the crate already imports.
- Commit to `governance-v0` — commit to your task branch only.
- Run any cargo command.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT files/lines before writing

**`crates/api/api_utils/src/bridge_notify.rs:53-90`** (the producer
`governance_case_after_transition`) — the function you are extending. It
ALREADY holds `let pool = &mut context.pool();` (line 60) and ALREADY does an
async DB read (`GovernanceMessagingConfig::read_current`, lines 60-66) before
constructing the payload at line 69. Your new juror fetch follows that exact
shape: an `.await?`-ed read, then set the field on the struct literal.

**`crates/api/api/src/governance/actor_pseudonym_helper.rs:26-30`** — the
Diesel pattern to MIRROR for the join (you CANNOT call this fn — wrong crate
— but you copy its shape):
```rust
actor_pseudonym::table
    .filter(actor_pseudonym::person_id.eq(person_id))
    .select(actor_pseudonym::pseudonym)
```
Your free fn does the same with an `.inner_join(jury_assignment::table)`
filtered by `jury_assignment::case_id.eq(case_id)`, selecting
`actor_pseudonym::pseudonym`, loading `Vec<String>`.

**`crates/db_schema/src/source/governance/jury_assignment.rs:21-24`** +
**`actor_pseudonym.rs:20-21`** — the two source models being joined
(`jury_assignment.case_id → person_id`, `actor_pseudonym.person_id →
pseudonym: String`).

**`services/bridge/src/relay.rs:28-36`** — the M1 precedent: `BridgeNotifyPayload`
carries `brehon_sender`/`brehon_recipient` as `String` — the binary computes
identities and pushes them in; the bridge never GETs back. Your field is the
same idea for the juror list.

**`crates/api/api_common/src/governance.rs:860-866`** (the `CaseTransitionEvent`
struct — `target_type` at line 865 is the last field, add `juror_pseudonyms`
after it) and **`:872-875`** (the `BridgeNotifyPayload` enum it lives in).

### Plan section
`.claude/PRPs/plans/m2-rooms-a.plan.md` §"Task 1w: Augment CaseTransitionEvent
with juror_pseudonyms" — the full IMPLEMENT block (files 1-3), the dep-direction
resolution, and the §5.2 ceiling note.

### Lessons (mandatory)
- `.claude/lessons/feedback_features_full_workspace_only.md` — this is a workspace task; validate with `--workspace --features full`. NEVER `-p lemmy_server --features full` (no `full` feature on that crate).
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then STOP; the laptop advisor runs cargo.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — the producer is `LemmyResult<()>`; your new join fn returns `LemmyResult<Vec<String>>` and is `?`-propagated. Follow the case-A/B/C error idiom if you wrap a `diesel` error (use `.map_err` with the annotated closure shape from the lesson, or rely on the existing `LemmyResult` `From<diesel::result::Error>` if one exists in this codebase — check how `actor_pseudonym_helper.rs` propagates its query error and mirror that exactly).

**R8 note (this task is the INVERSE of T1):** T1 was a `services/bridge/`
task (anyhow, axum, NO `--features full`). **Task 1w IS a Lemmy-workspace
task** — `LemmyResult`, `LemmyContext`, Diesel, `--features full` ALL apply.
Do NOT carry the bridge-context "do not use Lemmy lessons" rule here; it is
reversed for this task.

## 4. Implementation

### 4.1 `crates/api/api_common/src/governance.rs` — add the field

In the `CaseTransitionEvent` struct (the one whose doc-comment says "Integer +
enum fields only — no usernames/emails"), add after `target_type` (the last
field):

```rust
  /// Pre-resolved pseudonymous juror handles for jury-bound transitions
  /// (e.g. JurySelection). Empty for non-jury transitions. These are
  /// `actor_pseudonym.pseudonym` values ONLY — never real usernames/emails
  /// (ADR-015); the binary resolves to pseudonyms so the bridge never sees a
  /// real identity. The bridge renders each as `Juror-<suffix>`.
  #[serde(default)]
  pub juror_pseudonyms: Vec<String>,
```

Update the struct's top doc-comment so the "no usernames/emails" line still
reads true: the field carries *pseudonyms*, not real identities — adjust the
wording to "Integer + enum fields plus pre-resolved pseudonyms — no real
usernames/emails (the bridge owns the `Juror-<suffix>` rendering)."

`#[serde(default)]` lets the field default to empty on deserialize if a
producer ever omits it (forward-compat).

### 4.2 `crates/api/api_utils/src/bridge_notify.rs` — the join fn + population

**(a) Add a private async free fn** that does the join. MIRROR
`actor_pseudonym_helper.rs:26-30`'s Diesel shape. Signature shape (adapt
types to what the codebase uses — `DbPool`, `ModerationCaseId`, etc.):

```rust
async fn fetch_juror_pseudonyms(
  pool: &mut DbPool<'_>,
  case_id: ModerationCaseId,
) -> LemmyResult<Vec<String>> {
  use lemmy_db_schema_file::schema::{actor_pseudonym, jury_assignment};
  // mirror actor_pseudonym_helper.rs error propagation exactly
  let conn = &mut get_conn(pool).await?;          // use this codebase's conn-acquisition idiom
  let rows = jury_assignment::table
    .inner_join(actor_pseudonym::table.on(actor_pseudonym::person_id.eq(jury_assignment::person_id)))
    .filter(jury_assignment::case_id.eq(case_id))
    .select(actor_pseudonym::pseudonym)
    .load::<String>(conn)
    .await?;
  Ok(rows)
}
```
GOTCHA: confirm the exact `inner_join`/`.on(...)` syntax against how other
governance handlers join in this codebase (some use explicit `.on`, some rely
on declared `joinable!` associations). Read one existing two-table governance
join before writing this — do NOT guess the association. If
`lemmy_db_schema_file::schema` does not declare a `joinable!` between
`jury_assignment` and `actor_pseudonym`, use the explicit `.on(...)` form
above.

**(b) Populate the field** in `governance_case_after_transition`, gated on
`new_status`. The C2.1 jury room is provisioned on the transition INTO jury
selection, so:

```rust
let juror_pseudonyms = if matches!(new_status, CaseStatus::JurySelection) {
  fetch_juror_pseudonyms(pool, case.id).await?
} else {
  Vec::new()
};
let payload = BridgeNotifyPayload::CaseTransition(CaseTransitionEvent {
  case_id: case.id.0,
  old_status,
  new_status,
  community_id: case.community_id.map(|c| c.0),
  target_type: case.target_type,
  juror_pseudonyms,
});
```

GOTCHA: place the fetch AFTER the `if !enabled { return Ok(()); }` early
return (lines 64-66) — no point fetching when messaging is disabled. The
fetch adds ONE indexed read on `case_id`, only for `JurySelection` transitions
— R3 (non-blocking notify) still holds; the notify is already
fire-and-forget. Do NOT add the fetch for every transition.

NOTE (scope boundary): only `JurySelection` triggers the fetch in THIS task.
The appeal-jury room (C2.4) is T3 scope; if T3 needs the appeal jury list,
the field already exists to populate then — do not add appeal handling here.

## 5. Validate-pending-laptop entry (write-then-stop, R4)

After implementing + committing the 2 files on your task branch:

1. Generate a new DQ id: `bash scripts/brehon/dq-v3-new-entry.sh`
2. Write the fragment to `/tmp/dq-t1w-frag.json`:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "m2-rooms-a Task 1w validate-pending-laptop: cargo check --workspace --features full",
  "options": ["pass", "fail"],
  "context": "Task 1w complete: CaseTransitionEvent gains juror_pseudonyms (serde default), bridge_notify.rs adds fetch_juror_pseudonyms free fn (jury_assignment INNER JOIN actor_pseudonym) + populates for JurySelection. Workspace toolchain. Waiting for laptop --workspace --features full check.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "1w",
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```
3. Append: `bash scripts/brehon/dq-v3-append-fragment.sh /tmp/dq-t1w-frag.json --pending`
4. `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-1w" && git push origin <your-branch>`
5. **STOP. Do not run cargo. The laptop advisor runs the command.**

## 6. Commit message (Task 1w)

```
feat(api): augment CaseTransitionEvent with juror_pseudonyms producer (task 1w)
```

Optional body trailer (per the m2-core-hook lesson about scoping trailers to
producer context): if you note anything about the serialization, state the
consumer — e.g. `LESSON: juror_pseudonyms is consumed by the Matrix bridge's
invite loop (relay.rs-style), not a log line — keep it a plain Vec<String>`.

## 7. DQ blocker protocol

If you hit an unexpected obstacle, write a `kind: "blocker"` DQ entry via
`bash scripts/brehon/dq-v3-append-fragment.sh <frag> --pending`, commit + push
on your task branch, and stop. Do not guess. Specific blockers to watch:
- **No `joinable!` association** between `jury_assignment` and `actor_pseudonym`
  AND the explicit `.on(...)` form doesn't compile → blocker (do not invent a
  schema change).
- **`ModerationCaseId` / `case.id` type mismatch** in the filter → read the
  newtype's `.eq` usage in an existing governance query and mirror; if still
  stuck, blocker.
- **`get_conn` / pool-acquisition idiom** differs from what's shown → mirror
  `actor_pseudonym_helper.rs`'s exact acquisition; if it uses a different
  helper, use that.

## HANDOVER

```yaml
HANDOVER:
  task: m2-rooms-a-impl-1w
  filesModified:
    - crates/api/api_common/src/governance.rs
    - crates/api/api_utils/src/bridge_notify.rs
  keyDecisions:
    - "juror_pseudonyms: Vec<String> with #[serde(default)] on CaseTransitionEvent"
    - "by-case join (jury_assignment INNER JOIN actor_pseudonym) as a free fn in bridge_notify.rs — kept to 2 crates (api_common+api_utils); actor_pseudonym_helper.rs unreachable from api_utils"
    - "populated only for new_status == JurySelection (C2.1 trigger); appeal jury (C2.4) deferred to T3"
    - "ADR-015: pseudonyms only, never real identities"
    - "validate-pending-laptop DQ written (--workspace --features full); waiting for laptop check"
  nextTask: "T2 impl-task (room_provisioner.rs) — requires T1w validate pass; T2 consumes event.juror_pseudonyms"
```
