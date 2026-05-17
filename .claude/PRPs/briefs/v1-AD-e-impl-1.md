---
phase: v1-AD-e
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-16
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: 238
canonical_sibling: ".claude/PRPs/briefs/sl-e-impl-2.md (§N house-style); .claude/PRPs/templates/impl-task-brief.template.md §5 (validate-pending-laptop shape)"
---

# [role:impl-task] v1-AD-e task 1 — add maud engine dependency (isolated commit) — see .claude/PRPs/briefs/v1-AD-e-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e task 1 — add maud engine dependency (isolated commit)`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 1**
from `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` §13 (lines
378–407). DQ #238 (resolved, `answered_by: user`) selected **maud**
over askama — install **maud only**, no `templates/` directory, no
askama crates.

## §2 Scope

**Produce (ONE commit):**

- `crates/api/api/Cargo.toml` — add the maud dependency to
  `[dependencies]`, inline single-consumer form with the `actix-web`
  feature (mirror `sitemap-rs`/`totp-rs` at lines 70–71).
- `Cargo.lock` — the regenerated resolution from `cargo add` /
  `cargo check`. Co-commit with the `Cargo.toml` change (per
  `feedback_commit_hygiene_lockfiles_and_task_labels`).

**Do NOT** in this task:

- Add askama / askama_web / askama_actix (DQ #238 = maud; askama was
  explicitly NOT chosen).
- Create any `templates/` directory or any `.html` file (maud is
  inline `html! {}` macros — zero new non-Rust files).
- Write any handler, route, module, or `mod.rs` line (Tasks 2–4).
- Touch `crates/server/tests/e2e.rs` (Task 5).
- `#[allow]`-spam to silence a clippy cascade — if the dependency-
  tree rebuild surfaces a transitive lint, STOP and surface to
  advisor (per `feedback_library_add_after_shipping.md` — the clippy
  run below is the *detector*).

**Commit message** (exactly): `feat(api): add maud HTML engine dependency (task 1)`

Record the exact resolved maud version (whatever `cargo add` selects
compatible with `actix-web 4.13.0`) in the commit body.

## §3 Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entry DQ #238** — the
   engine decision. Question: "pick the template engine"; Answer:
   "(a) maud — 1 dep, 0 new files (inline `html! {}` macros), native
   actix Responder … askama NOT chosen. Task 1 installs maud only;
   no templates/ dir." `answered_by: user`. This is the contract for
   *what* dep to add.
2. **Plan §13 Task 1** (`.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
   lines 378–407) — the canonical ACTION / FILES / IMPLEMENT / MIRROR
   / GOTCHA / VALIDATE block.
3. **Plan §5** (engine-parameterised note, lines ~86–100) — maud
   recommendation rationale + the "Task 1 installs whichever engine
   the DQ resolves" parameterisation.
4. **MIRROR ref:** `crates/api/api/Cargo.toml:70-71` —
   `sitemap-rs = "0.4.0"` and
   `totp-rs = { version = "5.7.1", features = ["gen_secret", "otpauth"] }`.
   These are the inline single-consumer dependency precedent. Add
   maud in the **same idiom** (NOT a `[dependencies.maud]` table
   block; NOT workspace-inherited — this crate is the sole consumer).
5. **`.claude/lessons/feedback_library_add_after_shipping.md`** —
   **MANDATORY (plan §2 binds this to Task 1).** A new workspace dep
   triggers a dependency-tree rebuild that can surface an unrelated
   transitive clippy lint. The post-add clippy run (§5 gate 2) is
   the detector, not optional. If a cascade appears, STOP — do not
   patch around it.
6. **`.claude/lessons/feedback_commit_hygiene_lockfiles_and_task_labels.md`**
   — `Cargo.lock` co-commits with the `Cargo.toml` change in the
   same task commit; the commit subject carries the `(task 1)` label.
7. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — **always
   for cargo work.** Capture cargo-check / cargo-clippy full output
   to a log file, check the exit code separately, THEN tail. Never
   pipe cargo through tail/grep when you need its exit status.
8. **`.claude/lessons/feedback_clippy_test_style.md`** — the
   workspace denies `unwrap`/`expect`/`#[allow]`; the clippy gate
   must be genuinely clean (exit 0).
9. **`.claude/agents/impl-task.md` "Pre-Shape-G plans"** + the
   `impl-task-brief.template.md` §5 — Shape G is SUSPENDED until
   2026-06-01 (`project_shape_g_suspended_2026_05_16`). After
   pushing, you write a `kind: "validate-pending-laptop"` DQ entry
   (NOT `kind: "validate-pending"`, NOT a `workflow_run_id`). See §5
   below for the exact entry shape.

## §3a Handover from prior task

`(none — prior task was Task 0, a read-only pre-flight barrier with
no commit and no key decisions to propagate.)`

## §4 Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-AD-e`. Finalize
  merges your worktree branch back; do NOT push to `phase-v1-AD-e`
  directly.
- **ONE commit.** Cargo.toml + Cargo.lock in a single
  `feat(api): add maud HTML engine dependency (task 1)` commit. If
  the post-add clippy fails on the first attempt for a *fixable*
  in-scope reason, amend/fixup into the one commit; do not split. If
  it fails for an out-of-scope transitive reason, STOP (see §5).
- Mid-task DQ visibility: if you raise a `pending` blocker, **commit
  + push immediately** to your worktree branch (per
  `.claude/rules/decision-queue.md` "Mid-task visibility").
- No `answered_by: "advisor"` or `"user"` from this subagent.
  Self-resolve only as `"impl-self-resolved"`. The
  `validate-pending-laptop` entry you write is `from: "impl"`,
  `answered_by: null` (the advisor-laptop mutates it).

### Dependency-add specifics

- **maud only.** `cargo add maud --features actix-web` from the
  `crates/api/api/` directory (or hand-edit `[dependencies]` then
  `cargo check` to regenerate the lock). The `actix-web` feature
  gives `Markup: Responder` directly — **no integration shim crate
  needed** (this is the maud advantage over askama; do NOT add
  `askama_web` / `askama_actix`).
- Pin = whatever `cargo add` resolves compatible with
  `actix-web 4.13.0`. Record the exact version string in the commit
  body (e.g. "maud 0.27.0 resolved against actix-web 4.13.0").
- Inline form, mirroring `Cargo.toml:70-71`:
  `maud = { version = "<resolved>", features = ["actix-web"] }`.
- The dependency-tree rebuild (per
  `feedback_library_add_after_shipping.md`) is expected to recompile
  a large chunk of the workspace — that is normal, not an error.
  Only a non-zero clippy/check exit is a failure.

### Memory-cap awareness

The daemon runs `MemoryMax=10G`. `cargo check --workspace --features
full` after a fresh dep add is memory-heavy. If it OOM-kills (you see
`Killed` in the log, non-zero exit), do NOT retry blindly — file a
`kind: "blocker"` DQ with the cargo log tail; the advisor decides.

### CC v2.1.119 sensitive-file gate (observed on Junior #282)

Writes under `.claude/**` MAY be blocked even in `bypassPermissions`
mode. This affects: (a) the `.claude/PRPs/debug/*.log` validation
logs, and (b) the `kind: "validate-pending-laptop"` write to
`.claude/decision-queue.json`.

- **If a `.claude/PRPs/debug/` log write is denied:** redirect that
  cargo command's output to `<worktree-root>/<same-filename>.log`
  instead; note the relocation in your §6 report. The cargo *exit
  code* is the load-bearing signal.
- **If the `.claude/decision-queue.json` `validate-pending-laptop`
  write is denied:** write the intended DQ entry JSON to
  `<worktree-root>/v1-AD-e-task1-VALIDATE-PENDING.json` instead and
  STOP with a clear escalation message naming that file. The
  advisor will relocate it into the canonical DQ on
  `phase-v1-AD-e`. Do NOT silently skip the validate-pending handoff
  because the write failed — the advisor must know cargo needs to
  run on the laptop.

### Submodule pre-check (per `feedback_worktree_submodules_not_auto_init.md`)

`git worktree add` does NOT init submodules. The Lemmy fork has a
submodule at `crates/email/translations`; an empty
`translations/backend/` makes `lemmy_email`'s `build.rs` fail with
`Os { code: 3, kind: NotFound }` → `cargo check --workspace` exits
101. Your Task 1 DoD runs `cargo-check --workspace --features full`,
which compiles `lemmy_email`. **Before the first cargo command**, run:

```bash
git submodule update --init --recursive
ls crates/email/translations/backend/ | head -3   # expect *.json locale files
```

If `ls` shows `*.json` files, the submodule is present (no-op if the
daemon already initialised it — harmless). If `git submodule update`
fails (network), file a `kind: "blocker"` DQ — do NOT proceed to a
cargo command that will fail with the NotFound error and waste a
validate-pending-laptop cycle. (Observed on the advisor's
`brehon-fork-ad-e` lane worktree during the v1-AD-e pre-phase audit
2026-05-16 — fixed there; this guard is belt-and-braces for the
daemon-side per-task worktree.)

### Plan-cited line numbers may have drifted

Plan cites `Cargo.toml:70-71`. Verify with `grep -n "sitemap-rs\|totp-rs"
crates/api/api/Cargo.toml` before editing; follow the grep output if
the lines moved.

## §5 Validation gates (per plan §13 Task 1 VALIDATE block) — Shape-G SUSPENDED

**Shape G is suspended until 2026-06-01.** After committing + pushing
your worktree branch, write a **`kind: "validate-pending-laptop"`** DQ
entry to `.claude/decision-queue.json` (commit + push it on your
worktree branch immediately so the advisor sees it on next fetch).

Do NOT write `kind: "validate-pending"`. Do NOT capture a
`workflow_run_id`. The advisor laptop session runs the commands and
mutates the entry.

**The `commands[]` array MUST contain, verbatim (the plan §15 /
§13-Task-1 DoD lines for Task 1):**

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-task1-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat -p lemmy_api --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task1-clippy.log 2>&1"
```

Both EXPECT exit 0. (Note the plan's Task 1 VALIDATE block uses
`-p lemmy_api` for the clippy narrowing — this is the dependency-
cascade detector per `feedback_library_add_after_shipping.md`; it is
intentionally crate-narrowed, not `--workspace`, for Task 1
specifically. The workspace clippy comes at later tasks per §15.2.)

**`validate-pending-laptop` entry shape** (per
`.claude/rules/decision-queue.md` "kind: validate-pending-laptop" +
the impl-task template §5):

```json
{
  "id": <max(all ids across decision-queue.json + decision-queue-archive-*.json) + 1>,
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-AD-e Task 1 (add maud dep) pushed on <worktree-branch> — run §15 DoD on laptop",
  "branch": "<your worktree branch name>",
  "phase_task": 1,
  "commands": ["<the two cmd //c lines above, verbatim>"],
  "context": "maud <resolved-version> added to crates/api/api/Cargo.toml + Cargo.lock regenerated; commit <sha>. Pre-Shape-G validate-pending-laptop (Shape G suspended until 2026-06-01, DQ #229).",
  "answer": null,
  "answered_by": null,
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "resolved_at": null
}
```

Compute `id` as `max(...) + 1` across BOTH
`.claude/decision-queue.json` AND every
`.claude/decision-queue-archive-*.json` (per
`.claude/rules/decision-queue.md` "Next-id calculation MUST span
both" — the DQ #50 collision lesson). Do NOT reuse an id.

If any local cargo gate fails when YOU run a sanity pre-check
(optional but encouraged per `feedback_local_runtime_before_push`),
do NOT push a broken tree — fix the in-scope cause first, or if it
is an out-of-scope transitive cascade, file a `kind: "blocker"` DQ
instead of the `validate-pending-laptop` entry and STOP.

## §6 Expected output (return to advisor)

```
## Task 1 complete — v1-AD-e add maud HTML engine dependency

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/Cargo.toml (+maud = { version = "<resolved>", features = ["actix-web"] })
  - Cargo.lock (regenerated — maud + transitive deps)
**maud version resolved:** <exact version> (against actix-web 4.13.0)
**Validate handoff:** wrote DQ #<id> kind=validate-pending-laptop, from=impl, branch=<worktree-branch>, phase_task=1 (commands: cargo-check --workspace --features full ; cargo-clippy -p lemmy_api --features full --no-deps -D warnings)
**Next:** advisor-laptop runs the two DoD commands, mutates DQ #<id>; on result=pass advisor queues Task 2 (extract gather_dashboard + dashboard HTML handler + route)
```

Plus any `kind: "blocker"` DQ #N reference if you hit an OOM /
transitive-cascade / sensitive-file-gate STOP condition.

## §7 Why this brief differs from the plan

Clean execution of plan §13 Task 1 with the Shape-G-suspended
validate-pending-laptop substitution (per
`project_shape_g_suspended_2026_05_16` + DQ #229 — Shape G's
`workflow_run_id` / `kind: "validate-pending"` path is replaced by
`kind: "validate-pending-laptop"` with `commands[]`; the cargo
command shapes are otherwise identical to the plan's §15). The only
other additions are the explicit CC v2.1.119 sensitive-file-gate
fallbacks (§4) — observed on Junior #282 at bm-cut; the branch +
push remain the load-bearing deliverables and are unaffected by that
gate.
