---
phase: v1-federation-inbound-b
role: impl-task
task: 3
brief_n: 1
authored: 2026-05-19
plan: .claude/PRPs/plans/v1-federation-inbound-b.plan.md
plan_task: "§13 Task 3 — declare ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED const + shim re-export + registry append"
parent_phase_tip: 753289320 (phase-v1-federation-inbound-b @ Task-0 audit pass, origin tip)
cohort: "Cohort A (Tasks 1-3, 3-way [P]) — dispatched in parallel"
related_dq: "276 (proceed-as-one, user-resolved), 279 (serial-phase policy), 235 (registry .claude/ write gated — advisor reconciles)"
---

# [role:impl-task] v1-federation-inbound-b Task 3 — persist_failed const + shim re-export — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-3.md

> **Clarify provenance:** parent **planning** brief clarified via `/brehon-clarify` (DQ #273/#274/#275 resolved advisor-mode; DQ #276 split-or-proceed user-resolved **proceed-as-one**). PRECON-3 (BINDING) = §15 is `validate-pending-laptop` shape (Shape G SUSPENDED until 2026-06-01 per DQ #229). Impl-task brief; no clarify-DQ gates it directly.

## §0 Pre-flight (subagent runs this before reading anything else)

- Confirm CWD branch is `phase-v1-federation-inbound-b`. If `git branch --show-current` is anything else → STOP, file `kind: "blocker"` DQ (`from: "impl"`).
- Forbidden-window self-check (per `.claude/agents/impl-task.md` task-0 discipline): `date -u +"%a %H:%M UTC"` — inside a forbidden window (Daily 02:55–04:15 / Sun 01:55–02:35 / Sun 03:55–04:30 / Wed 03:55–04:15 UTC) → exit non-zero `FORBIDDEN_WINDOW: <window>`. Shape G SUSPENDED (cargo on laptop); keep the check.

## §1 Role + dispatch

`[role:impl-task] v1-federation-inbound-b Task 3 — ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED const + api shim re-export`

```
[role:impl-task] v1-federation-inbound-b Task 3 — see .claude/PRPs/briefs/v1-federation-inbound-b-impl-3.md
```

## §2 Scope

**Produce** (one commit) — **2 `crates/` files ONLY** (the 3rd "file" in plan §13 Task 3's FILES YAML, `.claude/rules/governance-log-entry-kind-registry.md`, is an advisor-reconciled write — see §4 harness-gap; do NOT attempt it and do NOT block on it):

- `crates/db_schema/src/source/governance/governance_log.rs` — append, alphabetically within the existing `federation_inbound_*` const cluster (plan cites lines :221-229 — verify by grep):
  ```rust
  pub const ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED: &str = "federation_inbound_persist_failed";
  ```
- `crates/api/api/src/governance/governance_log.rs` — add ONE `pub use` line, alphabetically with the existing `_FEDERATION_INBOUND_*` re-exports:
  ```rust
  pub use lemmy_db_schema::source::governance::governance_log::ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED;
  ```

**Do NOT** in this task:

- Edit `.claude/rules/governance-log-entry-kind-registry.md`. That is plan §13 Task 3's 3rd FILES-YAML entry, but the Junior sensitive-file gate blocks `.claude/**` writes (DQ #235). The advisor applies the registry append + the v1-federation-inbound-a "Deferred" bullet edit + the 54→55 count bump **post-task, advisor-side**. Your job is ONLY the 2 `crates/` files. Do NOT file a blocker for the registry — its omission from your commit is EXPECTED and handled (see §4 + §6).
- Touch `error.rs` (Task 1), the 3 protocol structs (Task 2), `inbox.rs` (Task 4), any `publish_*.rs` (Cohort B), `scheduled_tasks.rs` (Task 8), `e2e.rs` (Task 9).
- Add any emitting call site. The const is DECLARED here; the live `_persist_failed` emit lands in Task 4 (`inbox.rs` receive bodies). If you find yourself editing a `receive_remote_*` body, STOP (out of scope — that is Task 4).

**Commit message** (exactly): `feat(v1-federation-inbound-b): governance_log — ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED const + api shim re-export (task 3)`

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries gating this task** — DQ #276 (BINDING — proceed-as-one), DQ #279 (serial-phase policy), DQ #235 (BINDING — the `.claude/rules/` registry edit is advisor-reconciled, NOT your scope; this is WHY your commit is 2 files not 3).
2. **Plan §10.3** — the authoritative const + shim re-export blocks (copy verbatim; this is the contract). The registry-markdown block in §10.3 is for the ADVISOR's post-task reconcile — read it for context only; do NOT apply it.
3. **Plan §13 "Task 3"** — step list (note IMPLEMENT file 3 of 3 is the registry; per §4 here it is advisor-side, so you do files 1 and 2 only).
4. **MIRROR ref** — fed-in-a's dual-file precedent at `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` §10.7 (the const-in-db_schema + pub-use-in-api shim pattern). Also **`grep -n 'ENTRY_KIND_FEDERATION_INBOUND_'`** in BOTH `crates/db_schema/src/source/governance/governance_log.rs` and `crates/api/api/src/governance/governance_log.rs` to find the exact existing cluster + the alphabetical insertion point (plan line cites :221-229 are plan-time; trust grep).
5. **Lessons** (per `.claude/rules/advisor-orchestrator.md` §2.4 — Task 3 runs `cargo-clippy.bat ... -D warnings` on new code):
   - `.claude/lessons/feedback_clippy_test_style.md` — **Why:** the workspace clippy config denies escape-hatches; the new `pub const` + `pub use` must be clippy-clean under `-D warnings` (the §5 validation runs `cargo-clippy.bat --workspace --features full --no-deps -- -D warnings`).
   - `.claude/lessons/feedback_clippy_rerun_after_fix.md` — **Why:** a stale clippy pass is not evidence; re-run after any fix.
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail for the §5 commands).

## §3a Handover from prior cohort

(none — Cohort A is the first impl cohort; Task 0 was a pure read-only probe with no `HANDOVER:` trailer.)

## §4 Constraints (hard rules)

### Branch + commit discipline

- Junior worktree off `phase-v1-federation-inbound-b`; finalize merges back; do NOT push to the phase branch directly.
- One commit (2 `crates/` files). If clippy fails on first attempt, amend/fixup — do NOT split.
- Mid-task DQ visibility: raise a `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as `"impl-self-resolved"`.

### Harness-gap note (per DQ #235 — registry edit is ADVISOR-RECONCILED, not a blocker)

**This is the load-bearing constraint for Task 3.** Plan §13 Task 3's FILES YAML lists 3 modifies, the 3rd being `.claude/rules/governance-log-entry-kind-registry.md`. The Junior Claude Code sensitive-file gate blocks ALL `.claude/**` writes (the same gate that blocked planning #330 + bm-cut #331 — DQ #235). Therefore:

- You implement ONLY the 2 `crates/` files (db_schema const + api shim re-export). This is COMPLETE for Task 3's Junior scope.
- Do **NOT** attempt to edit `.claude/rules/governance-log-entry-kind-registry.md`.
- Do **NOT** file a `kind: "blocker"` about the registry — its absence from your commit is EXPECTED and the advisor handles it post-task (the advisor applies the §10.3 registry-markdown block + removes the `federation_inbound_persist_failed` line from the v1-federation-inbound-a "Deferred" bullet + bumps the acceptance-invariants count 54→55, advisor-side, per `.claude/rules/escalation.md` + the fed-in-a precedent).
- In your §6 task-complete summary, explicitly note: "registry edit deferred to advisor per DQ #235 — 2-file commit is complete Task-3 Junior scope". This signals the advisor to do the reconcile; it is NOT an escalation/blocker.

If a DIFFERENT `.claude/` write is gated (the §5 `validate-pending-laptop` DQ entry): use the standard escalation path — write the JSON to `TASK3_VALIDATE_PENDING.json` + a short `TASK3_ESCALATION.md` at worktree root + commit both + push + STOP. Advisor transcribes per `.claude/rules/escalation.md`.

### clippy discipline

- Do NOT `#[allow]`-spam to silence clippy. A `pub const &str` + a `pub use` are trivially clippy-clean; if clippy flags something, fix the root cause (per `feedback_clippy_test_style.md`). If a genuine clippy finding can't be resolved without an allow, file a DQ pending rather than spamming.

### Plan-cited line numbers may have drifted

§10.3 is the contract for *content + placement* (alphabetical within the `federation_inbound_*` cluster). The :221-229 line cite is plan-time — `grep -n 'ENTRY_KIND_FEDERATION_INBOUND_'` in both files for the real cluster position and insert alphabetically. If `crates/db_schema/.../governance_log.rs` no longer has a `federation_inbound_*` const cluster, or the api shim no longer re-exports `_FEDERATION_INBOUND_*`, file a DQ pending entry — the plan may have drifted.

## §5 Validation gates (Shape-G suspended — validate-pending-laptop)

After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `phase_task: 3`, `branch: <your-worktree-branch>`) with `commands[]` set **verbatim** to:

```
cmd //c "scripts\brehon\cargo-check.bat --workspace --features full > .claude/PRPs/debug/fed-in-b-task3-check.log 2>&1"
cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/fed-in-b-task3-clippy.log 2>&1"
```

Do NOT write `kind: "validate-pending"` / capture a `workflow_run_id`. The advisor laptop session runs both commands and mutates the entry (`result: pass|fail`). If the DQ write is gated, use the §4 harness-gap path with `TASK3_VALIDATE_PENDING.json`.

Per `feedback_pipes_mask_exit_codes.md`: you do not run these commands yourself — your job is only to apply the 2-file edit + raise the entry with the commands verbatim.

## §6 Expected output (return to advisor)

```
## Task 3 complete — v1-federation-inbound-b persist_failed const + api shim

**Commit:** <sha> on <worktree-branch>
**Files changed (2 — registry is advisor-reconciled per DQ #235):**
  - crates/db_schema/src/source/governance/governance_log.rs  (+1 pub const ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED)
  - crates/api/api/src/governance/governance_log.rs             (+1 pub use re-export)
**Registry:** .claude/rules/governance-log-entry-kind-registry.md edit DEFERRED to advisor per DQ #235 (Junior sensitive-file gate) — 2-file commit is the complete Task-3 Junior scope; NOT a blocker
**Anchor verification:** federation_inbound_* const cluster found at db_schema:<N>; api shim re-export block at <M>; new const/use inserted alphabetically
**validate-pending-laptop DQ:** #<id> raised (commands: cargo-check.bat + cargo-clippy.bat -D warnings)
**Next:** advisor applies registry append + Deferred-bullet edit + 54→55 count (advisor-side); advisor laptop runs §15 cargo, mutates DQ #<id>; Cohort A barrier waits on all 3 tasks (1+2+3)
```

Plus any DQ #N references if you raised a blocker mid-task (NOT for the registry — that is expected-deferred, not a blocker).

## §7 Why this brief differs from the plan

Plan §13 Task 3's FILES YAML lists 3 modifies; this brief scopes Junior to **2** (the 2 `crates/` files) and explicitly defers the 3rd (`.claude/rules/governance-log-entry-kind-registry.md`) to advisor-side reconciliation per **DQ #235** (the Junior Claude Code sensitive-file gate blocks `.claude/**` writes — confirmed on planning #330 + bm-cut #331 this same phase). This is the ONLY substantive divergence and it is forced by the harness gap, not a scope change: the const+shim (the compile-relevant deliverable) is implemented by Junior exactly per §10.3; the registry (a tracked-markdown audit artifact) is applied by the advisor with the §10.3 block verbatim. Other additions: (a) §0 forbidden-window self-check, (b) §5 explicit `validate-pending-laptop` shape per PRECON-3, (c) explicit "plan line numbers may have drifted — grep for the cluster". The const + shim re-export are §10.3 verbatim — do not deviate. Mirrors the canonical sibling `.claude/PRPs/briefs/federation-inbound-a-impl-3.md` per `.claude/rules/advisor-orchestrator.md` §3.6 canonical-schema-first gate.

---

_Brief author: advisor session (canonical CWD `C:/Users/barri/Developer/brehon-fork` on `governance-v0`; per DQ #279 the canonical session drives fed-in-b impl under serial-phase policy). Brief committed on `governance-v0` before the Junior Task-3 task is queued; then propagated onto `phase-v1-federation-inbound-b` (conflict-free trunk-merge in the lane worktree) so the worker — which branches from the PHASE branch — sees it. `/precheck` re-verifies daemon-local `phase-v1-federation-inbound-b` SYNC before queue. The advisor performs the §10.3 registry append + Deferred-bullet edit + 54→55 count bump post-task per DQ #235._
