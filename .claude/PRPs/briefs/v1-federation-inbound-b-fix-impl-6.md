---
phase: v1-federation-inbound-b
role: impl-task
kind: fix-impl
fix_impl_n: 6
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
triggering_dq: 286
triggering_task: 6
classification: "Two clippy errors (-D warnings) on Task-6 + fix-impl-5 stack (post-fix §15 R2 cmd-2): (1) clippy::items-after-statements at publish_trust_attestation.rs:138 — `const CONFIG_KEY: &str = ...` declared AFTER statements in `check_per_actor_rate_limit` body (let-bindings at L125-132 precede the const at L138). (2) clippy::unfulfilled_lint_expectations at inbox.rs:470 — `#[expect(dead_code, ...)]` on `rate_per_actor_counts` is now unfulfilled because Task 6's `publish_trust_attestation.rs::check_per_actor_rate_limit` override CALLS `crate::governance::inbox::rate_per_actor_counts()`. Both are §G4-allowlist-equivalent mechanical fixes: (1) clippy::items-after-statements is in the clippy-test-style family (per `feedback_clippy_test_style.md`); canonical fix is to hoist the const BEFORE the first statement in the same fn body. (2) clippy::unfulfilled_lint_expectations is the SAME class as fix-impl-4's strip of `#[expect(dead_code)]` on `wrap_governance_inbound` — Task 6 became the first caller; strip the expectation. **Allowlist-equivalent mechanical fix** (zero design ambiguity; both fixes have canonical recipes)."
base: "phase-v1-federation-inbound-b @ 7c57ca31b (fix-impl-5 finalize-merge tip — publish_trust_attestation.rs E0283 resolved by dropping spurious .into(); §15 R2 cmd-1 cargo-check GREEN 3m15s; §15 R2 cmd-2 clippy FAILED with these 2 errors). Task-6 commits stay UNCHANGED — fix-impl-6 is on top."
cap: "EXACTLY 2 hunks across 2 files (≤8 lines net). **File 1: crates/apub/activities/src/governance/publish_trust_attestation.rs** — Hoist `const CONFIG_KEY: &str = \"federation.inbound.per_actor_attestation_rate_per_hour\";` from line 138 (after statements at L125-132) to BEFORE line 125 (the first statement in `check_per_actor_rate_limit` body). Idiomatic placement: immediately after the `) -> LemmyResult<()> {` opening of the fn at L121 + the leading comment block (if any). Concretely: move the single `const CONFIG_KEY` line so it appears as the FIRST line of the fn body, before `let subject_url = ...`. Delete it from line 138; insert it at the top of the fn body (right after `) -> LemmyResult<()> {`). Net diff: +1 / -1 (relocate). **File 2: crates/apub/activities/src/governance/inbox.rs** — DELETE the `#[expect(dead_code, reason = \"pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13\")]` line immediately above `pub(crate) fn rate_per_actor_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {` (currently at L470 in inbox.rs). Task 6 now calls this fn (`publish_trust_attestation.rs::check_per_actor_rate_limit` → `crate::governance::inbox::rate_per_actor_counts()`), so the dead_code expectation is unfulfilled → workspace clippy `-D warnings` fails. Net diff: -1 (strip annotation). Total: -1 line in inbox.rs, +1 / -1 (relocate) in publish_trust_attestation.rs. NEVER touch the trait, the wrap_governance_inbound (fix-impl-4 territory), the wrap call site, the impl block beyond hoisting the const, log_inbox_drop, get_inbound_config_int, current_hour_bucket, rate_per_peer_counts, helper consts, publish_sanction_notice.rs, publish_label.rs, plan, briefs, lessons. A 3rd hunk or any other-file edit → STOP + kind:blocker."
serial: "Single-task barrier fix (Task 6 §15 — the SECOND fix-impl on Task 6 after fix-impl-5 resolved E0283), strictly serial cap=1 — only in-flight Junior for this lane. §15 (DQ #286 — 2 cmds: cargo-check + cargo-clippy --no-deps -- -D warnings) is re-run by the ADVISOR on the laptop AFTER this fix lands + finalize-merge. Task 6 is COMPLETE only when this fix's §15 passes both cmds; THEN advisor mutates DQ #286 result:pass. Cohort B's remaining serial dispatch (Task 7) stays gated behind Task-6-complete per user's cap-≤2 OOM choice."
---

# [role:impl-task] v1-federation-inbound-b fix-impl-6 — clippy fixes: hoist const + strip rate_per_actor_counts dead_code expectation

> **Provenance:** fix-impl-5 (#343, worker commit d32d2c3f2, finalize-merge 7c57ca31b) RESOLVED the E0283 type-annotations-needed at publish_trust_attestation.rs:155 (verbatim canonical-sibling mirror — drop spurious `.into()`). Advisor §15 R2 cmd-1 cargo-check GREEN (3m15s); §15 R2 cmd-2 cargo-clippy `-D warnings` FAILED with 2 errors: (a) clippy::items-after-statements at publish_trust_attestation.rs:138 (`const CONFIG_KEY` declared mid-function after let-bindings), (b) clippy::unfulfilled_lint_expectations at inbox.rs:470 (Task 6 now calls `rate_per_actor_counts`; the `#[expect(dead_code)]` annotation is unfulfilled — same class as fix-impl-4's strip on `wrap_governance_inbound`). Both are §G4-allowlist-equivalent mechanical fixes. DQ #286 stays pending until this fix's §15 (2 cmds) passes.

## §0 Pre-flight (subagent runs this FIRST, before reading anything else)

- Confirm CWD is a Junior worktree branched off `phase-v1-federation-inbound-b` (base tip `7c57ca31b`). `git merge-base --is-ancestor 7c57ca31b HEAD` MUST be true. If on `phase-v1-federation-inbound-b` itself or a non-`junior/*` branch → STOP, file `kind: "blocker"` DQ.
- Forbidden-window self-check (per `.claude/agents/impl-task.md`): `date -u +"%a %H:%M UTC"` — inside a forbidden window → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED.
- **Submodule init (MANDATORY before any in-worker cargo — per `feedback_worktree_submodules_not_auto_init`):** `git submodule update --init 2>&1` from worktree root.
- Confirm the 2 defect anchors are present:
  ```
  grep -n "^    const CONFIG_KEY: &str" crates/apub/activities/src/governance/publish_trust_attestation.rs
  grep -nB 1 "pub(crate) fn rate_per_actor_counts" crates/apub/activities/src/governance/inbox.rs
  ```
  Expected: `const CONFIG_KEY` at ~L138 (4-space indented, inside fn body, AFTER the let-statements at L125-132). The `#[expect(dead_code, reason = \"pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 ...\")]` line at L470 above `pub(crate) fn rate_per_actor_counts()` at L471. If either missing → STOP + `kind:"blocker"` (target moved).
- Confirm the `#[expect(dead_code)]` on `wrap_governance_inbound` was ALREADY stripped (fix-impl-4 work): `grep -B 1 "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs | head -3` — must NOT show `#[expect(dead_code...`. If it does → STOP + `kind:"blocker"` (fix-impl-4 reverted?).

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b fix-impl-6 — hoist CONFIG_KEY + strip rate_per_actor_counts dead_code (Task 6 §15 clippy fix)`

Actual create-task description (single line, <100 chars):

```
[role:impl-task] v1-federation-inbound-b fix-impl-6 — see .claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-6.md
```

## §2 Scope

### 2.1 The defects being fixed (the contract — Task 6 §15 R2 cmd-2 fail, verified by advisor 2026-05-19)

**Defect 1 — items-after-statements at publish_trust_attestation.rs:138**

In `crates/apub/activities/src/governance/publish_trust_attestation.rs`, inside `impl GovernanceInboundActivity for PublishTrustAttestation`'s `check_per_actor_rate_limit` method body (~L118-onwards), the const `CONFIG_KEY` is declared AFTER let-bindings:

```rust
async fn check_per_actor_rate_limit(
  &self,
  context: &Data<LemmyContext>,
) -> LemmyResult<()> {
  // Extract the attested subject URL from the untyped object stub.
  let subject_url = self                                               // L125 — STATEMENT
    .object
    .rest
    .get("subject")
    .and_then(serde_json::Value::as_str)
    .ok_or_else(|| {
      LemmyErrorType::Unknown("TrustAttestation object missing subject field".into())
    })?;                                                               // L132

  // Read the per-actor rate cap. ...
  const CONFIG_KEY: &str = "federation.inbound.per_actor_attestation_rate_per_hour";  // L138 — ITEM AFTER STATEMENTS
  let actor_cap: i64 = { ... };
  ...
}
```

clippy::items-after-statements fires because `const` is an item, and items are visible from the start of their scope — declaring them mid-function is confusing.

Compile error verbatim:
```
error: adding items after statements is confusing, since items exist from the start of the scope
   --> crates\apub\activities\src\governance\publish_trust_attestation.rs:138:5
    |
138 |     const CONFIG_KEY: &str = "federation.inbound.per_actor_attestation_rate_per_hour";
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    = note: requested on the command line with `-D clippy::items-after-statements`
```

**Defect 2 — unfulfilled_lint_expectations at inbox.rs:470**

In `crates/apub/activities/src/governance/inbox.rs`, `rate_per_actor_counts` carries `#[expect(dead_code, reason = \"pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 ...\")]`:

```rust
// L469 (blank line)
#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]  // L470
pub(crate) fn rate_per_actor_counts() -> &'static Mutex<HashMap<(String, i64), u32>> {  // L471
  static CELL: OnceLock<Mutex<HashMap<(String, i64), u32>>> = OnceLock::new();
  CELL.get_or_init(|| Mutex::new(HashMap::new()))
}
```

Task 6's `publish_trust_attestation.rs::check_per_actor_rate_limit` now CALLS this fn (via `crate::governance::inbox::rate_per_actor_counts()`), so the dead_code expectation is unfulfilled → workspace clippy `-D warnings` fails.

Compile error verbatim:
```
error: this lint expectation is unfulfilled
   --> crates\apub\activities\src\governance\inbox.rs:470:10
    |
470 | #[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 ...
    = note: pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13
    = note: `-D unfulfilled-lint-expectations` implied by `-D warnings`
```

### 2.2 The fix (the contract — verbatim mechanical recipe; apply EXACTLY)

**§G4 CANONICAL RECIPE — Defect 1 (clippy::items-after-statements):**

> | Failure signature | Auto-fix | Source lesson |
> | `clippy::items-after-statements` on a `const` declared mid-function after let-bindings | Hoist the const BEFORE the first statement in the same fn body (idiomatic: immediately after the fn opening brace, before any let-bindings). The const's value/type/visibility is unchanged. | `feedback_clippy_test_style.md` clippy-test-style family |

For publish_trust_attestation.rs `check_per_actor_rate_limit`: move the `const CONFIG_KEY` from line 138 to BEFORE line 125 (the first statement `let subject_url = ...`). Idiomatic placement: as the FIRST line of the fn body, immediately after `) -> LemmyResult<()> {` at L121. Diff (apply EXACTLY this):

1. **DELETE** the line at L138: `    const CONFIG_KEY: &str = "federation.inbound.per_actor_attestation_rate_per_hour";`
2. **INSERT** the same line as the FIRST line of the fn body, right after `) -> LemmyResult<()> {` at L121. Concretely: between L121 (the closing `) -> LemmyResult<()> {`) and L122 (the existing `// Extract the attested subject URL ...` comment).

**The comment block at L134-137** (above the OLD const location, "Read the per-actor rate cap. Mirrors ...") STAYS at its current location — it describes the per-actor-cap-read block that follows, NOT the const. Do NOT move the comment. Net result for File 1: +1 line near L121, -1 line at L138.

**§G4 CANONICAL RECIPE — Defect 2 (clippy::unfulfilled_lint_expectations):**

> | Failure signature | Auto-fix | Source lesson |
> | `clippy::unfulfilled_lint_expectations` on a `#[expect(dead_code, ...)]` annotation whose target fn now has a CALLER | DELETE the `#[expect(dead_code, ...)]` line entirely. The annotation was a pre-landed-infra placeholder; once a caller wires up, it's no longer dead, and the expectation must be stripped to satisfy `-D warnings`. (Same recipe as fix-impl-4 stripped `#[expect(dead_code)]` on `wrap_governance_inbound` when Task 5 became its first caller.) | `feedback_clippy_test_style.md` clippy-test-style family + fix-impl-4 retro |

For inbox.rs `rate_per_actor_counts`:

1. **DELETE** the entire line at L470: `#[expect(dead_code, reason = "pre-landed federation-inbound enforcement infra; callers wired by Cohort B Tasks 5-7 (impl GovernanceInboundActivity + wrap_governance_inbound call sites), which declare requires: task 4 per plan §13")]`

Nothing else changes in inbox.rs. The `pub(crate) fn rate_per_actor_counts()` declaration at L471 is unchanged; the body L472-L474 is unchanged. Net result for File 2: -1 line.

### 2.3 What is NOT in scope (the fence)

- **NEVER touch `wrap_governance_inbound`** (fix-impl-4 territory — already stripped its `#[expect(dead_code)]` + added HRTB). Confirm via §0 pre-flight grep that it has NO `#[expect(dead_code)]` — if it does, fix-impl-4 reverted and that's a separate blocker.
- **NEVER touch the `GovernanceInboundActivity` trait** (L443-461). Unchanged.
- **NEVER touch any other `#[expect(dead_code)]` site** in inbox.rs — the `wrap_governance_inbound` annotation was stripped by fix-impl-4; the `rate_per_actor_counts` annotation is THIS fix's only inbox.rs strip; no other `#[expect(dead_code)]` annotations exist in inbox.rs at base tip 7c57ca31b.
- **NEVER touch `publish_sanction_notice.rs`** (Task-5 complete + §15-green).
- **NEVER touch `publish_label.rs`** (Task-7 pending).
- **NEVER touch the wrap call site, the trait-impl block, the rate-gate body, or the imports** beyond the 2 specific hunks in §2.2.
- **NEVER touch the `.map_err` sibling at L148-150 in publish_trust_attestation.rs.** That's the canonical reference for fix-impl-5; leave it byte-for-byte.
- **NEVER touch any other file** in `crates/` or any test, schema, Cargo, plan, brief, lesson.
- **NEVER add `#[allow(...)]`** to suppress either error. Both fixes are structural (hoist + strip), not lint suppressions.

### 2.4 Verification (run BEFORE committing — pre-push cargo-check + cargo-clippy discipline)

In the worker's worktree, after applying the §2.2 edits, run a LOCAL `cargo check` AND `cargo clippy` BEFORE pushing:

```
bash scripts/brehon/cargo-check.sh --workspace --features full 2>&1 | tail -30
bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings 2>&1 | tail -30
```

Expected: BOTH `Finished \`dev\` profile [unoptimized] target(s) in <N>m<N>s` and exit 0. If EITHER fails:
- Compile error in publish_trust_attestation.rs (mis-hoisted the const, or wrong indentation) → STOP + `kind:"blocker"`.
- Clippy error on a DIFFERENT inbox.rs `#[expect(dead_code)]` site (none should exist at base tip — wrap_governance_inbound was stripped by fix-impl-4 and rate_per_actor_counts is THIS fix's target) → STOP + `kind:"blocker"`.
- Clippy error on something new in publish_trust_attestation.rs (e.g. the hoisted const now triggers a different lint — unlikely, const-at-top is idiomatic Rust) → STOP + `kind:"blocker"`.

Then grep-verify the §2.2 edits landed cleanly:

```
# Expected: const CONFIG_KEY appears EXACTLY once in publish_trust_attestation.rs, BEFORE the first let-statement
grep -n "const CONFIG_KEY:" crates/apub/activities/src/governance/publish_trust_attestation.rs
# Expected: the line number is now in the early L120s (right after fn opening), NOT L138

# Expected: 0 occurrences of the unfulfilled-lint-expectations defect annotation on rate_per_actor_counts
grep -B 1 "pub(crate) fn rate_per_actor_counts" crates/apub/activities/src/governance/inbox.rs | grep -c "expect(dead_code"
# (must return 0)

# Expected: wrap_governance_inbound still has NO #[expect(dead_code)] (fix-impl-4 unchanged)
grep -B 1 "pub(crate) async fn wrap_governance_inbound" crates/apub/activities/src/governance/inbox.rs | grep -c "expect(dead_code"
# (must return 0)
```

If ANY grep returns the wrong count → patch + re-grep before committing.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` pending entries** — DQ #286 (validate-pending-laptop, ptask=6) is the gating entry; this fix-impl unblocks it.
2. **`crates/apub/activities/src/governance/publish_trust_attestation.rs:118-160`** — Read the entire `check_per_actor_rate_limit` fn byte-for-byte. Confirm the const is at L138 (mid-fn) and identify the exact insertion point at the top of the fn body (between the fn opening `{` and the first comment/statement).
3. **`crates/apub/activities/src/governance/inbox.rs:465-475`** — Read around `rate_per_actor_counts` to confirm L470 has the `#[expect(dead_code)]` annotation and L471 is the fn declaration. fix-impl-4 already stripped the `wrap_governance_inbound` annotation at L485 (now gone) — confirm `rate_per_actor_counts` is the only remaining `#[expect(dead_code)]` in inbox.rs.
4. **`.claude/PRPs/briefs/v1-federation-inbound-b-fix-impl-4.md` §2.2 step (1)** — Read the CANONICAL RECIPE for stripping `#[expect(dead_code)]` on a pre-landed-infra fn whose first caller has just landed. The §2.2 step (1) here mirrors fix-impl-4's step (1) exactly — same recipe, different fn target.
5. **Lessons** (MANDATORY per `.claude/rules/advisor-orchestrator.md` §2.4):
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** clippy denies `#[allow]`; the §2.2 fixes are structural (hoist + strip), not suppressions.
   - `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** re-run clippy after the fix (stale pass ≠ evidence; §2.4 mandates pre-push BOTH cargo-check + cargo-clippy).
   - `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — **Why:** §2.4 worker-side discipline; saves a full ci-watcher cycle.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always).

## §3a Handover from prior cohort

```yaml
prior_tasks:
  - task: 4 (chain)
    commits: [a3757eabe, cdff6f09d, f01a1d44e, 4a60667c9, fix-impl-4 = 3bd2cfa4e]
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "fix-impl-4 added <'a> HRTB to wrap_governance_inbound + stripped its #[expect(dead_code)]. rate_per_actor_counts (L470) STILL has #[expect(dead_code)] — Task 6 was pending, so its strip was deferred. THIS fix-impl-6 strips that annotation."
  - task: 5
    commit: 92605e907 (impl) + 88f0f03c8 (DQ #285 mutate)
    filesModified: [crates/apub/activities/src/governance/publish_sanction_notice.rs]
    keyDecisions:
      - "Task 5 §15-green; trait-default rate gate (no per-actor override), so does NOT call rate_per_actor_counts."
  - task: 6 (worker) + fix-impl-5
    commits: [ec3b8643f (impl), d32d2c3f2 (fix-impl-5 drop spurious .into()), 7c57ca31b (finalize-merge)]
    filesModified: [crates/apub/activities/src/governance/publish_trust_attestation.rs]
    keyDecisions:
      - "Task 6 publish_trust_attestation.rs: wrap_governance_inbound call site + impl GovernanceInboundActivity with per-actor override calling rate_per_actor_counts/current_hour_bucket/log_inbox_drop. fix-impl-5 resolved E0283 at line 155 (drop spurious .into() in .ok_or_else closure)."
      - "§15 R2 cmd-1 cargo-check GREEN (3m15s) on tip 7c57ca31b — confirms Task-6 + fix-impl-5 compile cleanly."
      - "DEFECTS for this fix-impl-6: (a) const CONFIG_KEY at L138 fires clippy::items-after-statements (it's after let-statements at L125-132); (b) rate_per_actor_counts at inbox.rs:L470 now-unfulfilled #[expect(dead_code)] (Task 6 calls it)."
    notes: "fix-impl-6 lands on top of 7c57ca31b. Task-6 + fix-impl-5 commits stay unchanged. The 2-file 3-hunk fix unblocks DQ #286."
```

## §4 Constraints (hard rules)

- **2 commits OR 1 commit (worker's choice).** 1 commit is preferred (atomic). Subject:
  ```
  fix(v1-federation-inbound-b): hoist CONFIG_KEY + strip rate_per_actor_counts dead_code (fix-impl-6 → Task 6 §15)
  ```
  Or 2 commits if the worker prefers per-file split — same effect.
- Mid-task DQ visibility: if you raise a `pending` entry mid-task, **commit + push immediately**.
- No `answered_by: "advisor"` or `"user"`.

### Harness-gap note (per DQ #235)

If you need to write a DQ entry and the sensitive-file gate blocks it: write `FIXIMPL6_BLOCKER_DQ.json` + `FIXIMPL6_ESCALATION.md` at worktree root, commit + push, STOP. fix-impl-6 has NO `.claude/` deliverable on the happy path.

### fix-impl-6 GOTCHAs

- **Two files, two recipes, ONE atomic commit preferred.** If the worker splits into 2 commits, that's also valid; advisor's lossless finalize-merge ancestor-check handles both shapes.
- **The const-hoist preserves the value/type/visibility EXACTLY** — just moves the line. No re-naming, no scope change (it stays inside the fn body, just at the top).
- **The dead_code strip is a single-line delete** — not a "comment out", not a strip-with-leave-blank-line; just delete the entire `#[expect(...)]` line. The blank line at L469 (if present) and the fn at L471 stay; the deleted line was L470.
- **The §0 pre-flight grep confirms fix-impl-4's wrap_governance_inbound strip is still in place** — if it's NOT, that's a separate-blocker situation (fix-impl-4 was reverted somehow). Don't try to "re-fix" wrap_governance_inbound in this brief.

### Plan-cited line numbers may have drifted

Use the SYMBOL anchors: `const CONFIG_KEY:` for File 1 hoist target; `pub(crate) fn rate_per_actor_counts` for File 2 strip target.

## §5 Validation gates (advisor re-runs §15 — worker runs §2.4 pre-push)

**Worker-side:** §2.4 pre-push cargo-check + cargo-clippy. BOTH must exit 0 before the worker pushes.

**Advisor-side (after finalize-merge):** re-run DQ #286's `commands[]` verbatim on the laptop:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task6-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task6-clippy.log 2>&1"
```

On BOTH GREEN: mutate DQ #286 result:"pass".

## §6 Expected output (return to advisor)

```
## fix-impl-6 complete — hoist CONFIG_KEY + strip rate_per_actor_counts dead_code

**Commit(s):** <sha(s)> on <worktree-branch>
**Files changed:** publish_trust_attestation.rs (+1/-1 const relocate) + inbox.rs (-1 annotation strip)
**PRECON self-check:** const CONFIG_KEY was at L138 mid-fn (AFTER let-statements); rate_per_actor_counts had #[expect(dead_code)] at L470; wrap_governance_inbound had NO #[expect(dead_code)] (fix-impl-4 unchanged). After fix: const at top of fn body (~L122); #[expect(dead_code)] on rate_per_actor_counts stripped; wrap_governance_inbound still has no annotation.
**§2.4 pre-push:** cargo-check GREEN + cargo-clippy -D warnings GREEN (both 0 errors).
**Grep-verify:** all 3 grep counts at expected values.
**No new DQ raised** (existing DQ #286 stays pending; advisor mutates after §15 R3).
**Next:** advisor laptop re-runs §15 (DQ #286 cmds), mutates DQ #286 result:pass; Task 6 complete; Task 7 dispatched.
```

## §7 Why this brief differs from the plan

Plan §10.5/§10.6 prescribed the publish_trust_attestation.rs wrap + per-actor override; both fired correctly. This fix-impl chains: fix-impl-5 (E0283) → fix-impl-6 (clippy items-after-statements + unfulfilled-expectation). The dead_code strip on rate_per_actor_counts was foreseeable (annotation reason text named "Tasks 5-7" as future callers; Task 6 is the wire-up); the const hoist is a clippy::items-after-statements lint that surfaces with `-D warnings` workspace deny. Mirrors fix-impl-4's strip recipe for the SAME-CLASS unfulfilled-expectation defect. Mirrors fix-impl-3/4/5 schema per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first. The 2-file scope is justified per the §G4 "callsite-enumeration discipline" — both defects share a common surfacing condition (Task-6 first caller of pre-landed enforcement-core primitive) and a single fix-impl resolves both.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`). Brief committed on `governance-v0` first; then cherry-picked onto `phase-v1-federation-inbound-b`. §G4-allowlist-equivalent mechanical fix; no AskUserQuestion (canonical recipes; zero design ambiguity). fix-impl-6 is the SECOND fix-impl for Task 6 after fix-impl-5; fix-impl-N numbering: 1-3=Task-4 chain, 4=Task-5 HRTB, 5=Task-6 E0283, 6=Task-6 clippy._
