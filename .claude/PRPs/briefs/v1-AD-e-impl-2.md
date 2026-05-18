---
phase: v1-AD-e
role: impl-task
task: 2
brief_n: 2
authored: 2026-05-17
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: null
canonical_sibling: ".claude/PRPs/briefs/v1-AD-e-impl-1.md (§N house-style + validate-pending-laptop §5 shape); .claude/PRPs/templates/impl-task-brief.template.md"
---

# [role:impl-task] v1-AD-e task 2 — extract gather_dashboard + dashboard HTML handler + route — see .claude/PRPs/briefs/v1-AD-e-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e task 2 — extract gather_dashboard + dashboard HTML handler + route`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 2**
from `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` §13 (lines
409–449). Engine is **maud 0.27.0** (DQ #238, installed Task 1 — present
at `crates/api/api/Cargo.toml:72` with the `actix-web` feature, so
`maud::Markup`/`String` render is available; `maud::html!` auto-escapes
all interpolated strings).

## §2 Scope

**Produce (ONE commit, 4 files):**

1. **modify** `crates/api/api/src/governance/admin_dashboard.rs` —
   behaviour-preservingly extract the data-gathering body into
   `pub(crate) async fn gather_dashboard(...) -> LemmyResult<AdminDashboardResponse>`;
   rewrite `admin_dashboard()` to `is_admin?; <setup>; Ok(Json(gather_dashboard(...).await?))`.
2. **create** `crates/api/api/src/governance/admin_dashboard_html.rs` —
   `admin_dashboard_html(context, local_user_view) -> LemmyResult<HttpResponse>`
   (auth → feature-gate-404 → gather → render → `text/html` body) +
   `render_dashboard(&AdminDashboardResponse) -> String` via
   `maud::html! { … }`.
3. **modify** `crates/api/api/src/governance/mod.rs` — add
   `pub(crate) mod admin_dashboard_html;` alphabetically adjacent to
   the `admin_dashboard` declaration.
4. **modify** `crates/api/routes/src/lib.rs` — import
   **`admin_dashboard_html::admin_dashboard_html` ONLY** (NOT
   `admin_audit_html` — that lands Task 3; importing it now would not
   compile) + add `.route("/dashboard/view", get().to(admin_dashboard_html))`
   immediately after the existing `:544` `/dashboard` JSON route.

**Do NOT** in this task:

- Add `admin_audit_html` / `render_audit` / the `/audit/view` route
  (Task 3) or the live-tail `<script>` (Task 4) or any e2e test
  (Task 5). Task 2 is dashboard-view only.
- Change `AdminDashboardResponse` or any DTO shape — the extraction is
  **byte-behaviour-preserving** (the existing JSON `/dashboard` handler
  and the v1-AD-d dashboard e2e test must still pass — VALIDATE R7).
- Add askama / templates dir / `.html` files (maud is inline macros).
- `#[allow]`-spam a clippy finding — if the render fn surfaces a lint,
  fix it properly (maud auto-escapes; no `unwrap`/`expect`).
- Guess `conn`/`pool`/`cache` types — copy them EXACTLY from
  `admin_dashboard.rs:44-47` (GOTCHA below).

**Commit message** (exactly):
`feat(api): extract gather_dashboard + add dashboard HTML handler + route (task 2)`

Commit body: list the 4 files, note "behaviour-preserving extraction
(JSON handler delegates to gather_dashboard)", cite the MIRROR line
ranges used, and a `HANDOVER:` YAML trailer (filesCreated /
filesModified / keyDecisions / notes) per the cohort-handover convention.

## §3 Required reading

In this order:

1. **Plan §13 Task 2** (`.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
   lines 409–449) — the canonical ACTION / FILES / IMPLEMENT (4 files) /
   MIRROR / GOTCHA / VALIDATE block. **This is the contract.** The
   `requires: task 1` is satisfied (maud merged into `phase-v1-AD-e`
   at `bcade7d94`; DQ #241 = pass).
2. **Plan §10.2 + §10.3 + §10.4** — the handler preamble (§10.2:
   `is_admin?` → `html_pages_enabled` read → 404-if-false → gather →
   render → `text/html`), the verbatim `gather_dashboard` extraction
   shape (§10.3), and the config-accessor for
   `governance.dashboard.html_pages_enabled` (§10.4 — use the EXISTING
   accessor, do not hand-roll a `governance_config` query).
3. **MIRROR refs (read before writing):**
   - `crates/api/api/src/governance/admin_dashboard.rs:38-64` —
     the current handler + the body you extract into `gather_dashboard`.
     **Copy `conn`/`pool`/`cache` param types from `:44-47` verbatim.**
   - `crates/api/api/src/governance/admin_audit_stream.rs:123-126` —
     the `HttpResponse … text/html` body idiom (content-type +
     `.body(...)`).
   - `crates/api/routes/src/lib.rs:544` — the existing `/dashboard`
     JSON route; insert `/dashboard/view` immediately after it.
4. **`.claude/lessons/feedback_library_add_after_shipping.md`** — the
   maud render fn is the first real *consumer* of the Task-1 dep; the
   Task 2 clippy gate (`--workspace --no-deps -D warnings`) is the
   detector for any consumer-side lint. Genuinely clean (exit 0); do
   not patch around a cascade — STOP and surface.
5. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — **always
   for cargo work.** Capture each cargo command's full output to its
   `.claude/PRPs/debug/v1-AD-e-task2-*.log`, check the exit code
   separately, THEN tail. Never pipe cargo through tail/grep when you
   need its exit status.
6. **`.claude/lessons/feedback_clippy_test_style.md`** — the workspace
   denies `unwrap`/`expect`/`#[allow]`; `LemmyResult<T>` + `?`
   throughout the new handler + render fn.
7. **`.claude/lessons/feedback_commit_hygiene_lockfiles_and_task_labels.md`**
   — single commit, `(task 2)` label in the subject; no `Cargo.lock`
   change expected (no new dep — if `Cargo.lock` changes, something is
   wrong; investigate before committing).
8. **`.claude/agents/impl-task.md` "Pre-Shape-G plans"** + the
   `impl-task-brief.template.md` §5 — Shape G is SUSPENDED until
   2026-06-01 (`project_shape_g_suspended_2026_05_16`, DQ #229). After
   pushing, write a `kind: "validate-pending-laptop"` DQ entry (NOT
   `kind: "validate-pending"`, NO `workflow_run_id`). Exact shape in §5.

## §3a Handover from prior task

```yaml
prior_task:
  task: 1
  commit: 55c373016
  filesCreated: []
  filesModified: [crates/api/api/Cargo.toml, Cargo.lock]
  keyDecisions:
    - "maud 0.27.0 resolved against actix-web 4.13.0"
    - "actix-web feature activates maud::Markup as actix Responder — no shim crate"
    - "inline single-consumer dep form mirroring sitemap-rs/totp-rs at Cargo.toml:70-71"
  notes: >
    maud merged into phase-v1-AD-e at bcade7d94; DQ #241
    validate-pending-laptop = pass (cargo-check --workspace --features
    full exit 0 7m30s; cargo-clippy -p lemmy_api --no-deps -D warnings
    exit 0 6m38s; no transitive cascade). render_dashboard uses
    `maud::html! {}` — maud auto-escapes interpolated strings; no
    manual HTML escaping needed.
```

## §4 Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-AD-e` (current tip
  `a0f886e3a` — has the maud dep + DQ #241 resolved). Finalize merges
  your worktree branch back; do NOT push to `phase-v1-AD-e` directly.
- **ONE commit**, 4 files. If the post-edit clippy fails for a
  *fixable in-scope* reason, amend/fixup into the one commit. If it
  fails for an out-of-scope transitive reason, STOP (§5).
- Mid-task DQ visibility: if you raise a `pending` blocker, **commit +
  push immediately** to your worktree branch (per
  `.claude/rules/decision-queue.md` "Mid-task visibility").
- No `answered_by: "advisor"` / `"user"` from this subagent.
  Self-resolve only as `"impl-self-resolved"`. The
  `validate-pending-laptop` entry is `from: "impl"`, `answered_by:
  null` (advisor-laptop mutates it).

### Behaviour-preservation (R7 — load-bearing)

The extraction MUST be byte-behaviour-preserving. After your edit, the
existing JSON `admin_dashboard` handler returns exactly the same
`AdminDashboardResponse` it did before — it just delegates to
`gather_dashboard`. The Task 2 VALIDATE block runs
`cargo-test --no-run -p lemmy_server --test e2e` to prove the v1-AD-d
dashboard e2e test still compiles+links (R7). Do NOT change response
field order, types, or the auth/setup sequence in `admin_dashboard()`.

### Dashboard-render specifics (GOTCHA — copy verbatim from plan §13)

- **Copy the EXACT `conn`/`pool`/`cache` types from
  `admin_dashboard.rs:44-47`** — do not guess or infer. The
  `gather_dashboard` signature uses the real param types.
- `ActiveCasesSummary.by_status` is a `BTreeMap<String,i64>` — stable
  order; render with a safe `@for (k, v) in &resp.active_cases.by_status`
  maud loop.
- `ReputationBuckets` fields are `[i64;5]` **fixed arrays** — render
  directly (R1: NO `as` casts on them).
- Feature-gate (`html_pages_enabled == false`) returns **404**
  (`HttpResponse::NotFound().finish()`), **not 403** (R-html-3). 403 is
  reserved for the non-admin auth failure (that comes from `is_admin?`).
- `render_dashboard` must render **every field of the response tree**:
  `active_cases.by_status` (BTreeMap loop), `jury_queue` 3 counts,
  `federation` 3 counts, `rule_sets.per_community` (≤100 loop),
  `reputation` 4 bucket arrays + 3 thresholds + capability/founder
  counts, `calculated_at`. (Plan §13 Task 2 IMPLEMENT file 2 of 4 is
  the field checklist.)
- maud `html! {}` **auto-escapes** all interpolated string values — do
  NOT manually HTML-escape, and do NOT use any raw/unescaped maud
  construct for user-derived strings.

### Memory-cap awareness

The daemon runs `MemoryMax=10G`. The Task 2 DoD runs
`cargo-check --workspace --features full`; the workspace is already
warm from Task 1 so the incremental recompile is small, but if it
OOM-kills (`Killed`, non-zero exit), do NOT retry blindly — file a
`kind: "blocker"` DQ with the cargo log tail.

### CC v2.1.119 sensitive-file gate (observed on Junior #282)

Writes under `.claude/**` MAY be blocked even in `bypassPermissions`
mode. This affects (a) the `.claude/PRPs/debug/*.log` validation logs,
and (b) the `kind: "validate-pending-laptop"` write to
`.claude/decision-queue.json`.

- **If a `.claude/PRPs/debug/` log write is denied:** redirect that
  cargo command's output to `<worktree-root>/<same-filename>.log`;
  note the relocation in your §6 report. The cargo *exit code* is the
  load-bearing signal.
- **If the `.claude/decision-queue.json` `validate-pending-laptop`
  write is denied:** write the intended DQ entry JSON to
  `<worktree-root>/v1-AD-e-task2-VALIDATE-PENDING.json` instead and
  STOP with a clear escalation message naming that file. The advisor
  relocates it into the canonical DQ. Do NOT silently skip the
  validate-pending handoff — the advisor must know cargo needs to run
  on the laptop.

### Submodule pre-check (per `feedback_worktree_submodules_not_auto_init.md`)

`git worktree add` does NOT init submodules. Before the first cargo
command (the Task 2 DoD compiles `lemmy_email` via
`cargo check --workspace`):

```bash
git submodule update --init --recursive
ls crates/email/translations/backend/ | head -3   # expect *.json locale files
```

If `ls` shows `*.json`, the submodule is present (no-op if the daemon
already initialised it — harmless). If `git submodule update` fails
(network), file a `kind: "blocker"` DQ — do NOT proceed to a cargo
command that will fail with the `Os { code: 3, NotFound }` error and
waste a validate-pending-laptop cycle. (Observed on Junior #286 at
v1-AD-e Task 0 — first clippy exited 101 until `git submodule
update --init`.)

### Plan-cited line numbers may have drifted

Plan cites `admin_dashboard.rs:38-64`/`:44-47`, `admin_audit_stream.rs:123-126`,
`lib.rs:544`/`lib.rs:35-44`. Verify each with `grep -n` before editing
and follow the grep output if lines moved (e.g.
`grep -n 'pub async fn admin_dashboard\|let mut conn\|let mut pool' crates/api/api/src/governance/admin_dashboard.rs`).

## §5 Validation gates (per plan §13 Task 2 VALIDATE block) — Shape-G SUSPENDED

**Shape G is suspended until 2026-06-01.** After committing + pushing
your worktree branch, write a **`kind: "validate-pending-laptop"`** DQ
entry to `.claude/decision-queue.json` (commit + push it on your
worktree branch immediately so the advisor sees it on next fetch).

Do NOT write `kind: "validate-pending"`. Do NOT capture a
`workflow_run_id`. The advisor laptop session runs the commands and
mutates the entry.

**The `commands[]` array MUST contain, verbatim (the plan §13 Task 2
VALIDATE block — three commands, in order):**

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-task2-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task2-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-task2-testnorun.log 2>&1"
```

All three EXPECT exit 0. (Note: Task 2's clippy is **`--workspace`**
`--no-deps -D warnings` — wider than Task 1's `-p lemmy_api` because
the new module + route touch `lemmy_api` AND `lemmy_api_routes`. The
`test --no-run` is the R7 behaviour-preservation proof — it must
*link*, not run.)

**`validate-pending-laptop` entry shape** (per
`.claude/rules/decision-queue.md` "kind: validate-pending-laptop" +
the impl-task template §5):

```json
{
  "id": "<max(all ids across .claude/decision-queue.json + .claude/decision-queue-archive-*.json) + 1>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-AD-e Task 2 (extract gather_dashboard + dashboard HTML handler + route) pushed on <worktree-branch> — run §13-Task-2 DoD on laptop",
  "branch": "<your worktree branch name>",
  "phase_task": 2,
  "commands": ["<the three cmd //c lines above, verbatim>"],
  "context": "gather_dashboard extracted (behaviour-preserving) + admin_dashboard_html.rs created (maud render) + mod.rs + lib.rs route /dashboard/view; commit <sha>. Pre-Shape-G validate-pending-laptop (Shape G suspended until 2026-06-01, DQ #229).",
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
`.claude/rules/decision-queue.md` "Next-id calculation MUST span both"
— the DQ #50 collision lesson). The current max id is **241** (DQ #241
resolved); the next id is **242** unless an archive holds a higher one
— check. Do NOT reuse an id.

If any local cargo gate fails when YOU run a sanity pre-check
(optional but encouraged per `feedback_local_runtime_before_push`), do
NOT push a broken tree — fix the in-scope cause first, or if it is an
out-of-scope transitive cascade, file a `kind: "blocker"` DQ instead
of the `validate-pending-laptop` entry and STOP.

## §6 Expected output (return to advisor)

```
## Task 2 complete — v1-AD-e extract gather_dashboard + dashboard HTML handler + route

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/governance/admin_dashboard.rs (extract gather_dashboard; admin_dashboard delegates)
  - crates/api/api/src/governance/admin_dashboard_html.rs (NEW — admin_dashboard_html + render_dashboard maud)
  - crates/api/api/src/governance/mod.rs (+pub(crate) mod admin_dashboard_html;)
  - crates/api/routes/src/lib.rs (+import admin_dashboard_html + .route("/dashboard/view", ...))
**MIRROR cited:** admin_dashboard.rs:<lines> (handler+gather), admin_audit_stream.rs:<lines> (html body idiom), lib.rs:<line> (route insertion)
**Behaviour-preservation:** admin_dashboard() JSON handler now delegates to gather_dashboard — response shape unchanged
**Validate handoff:** wrote DQ #<id> kind=validate-pending-laptop, from=impl, branch=<worktree-branch>, phase_task=2 (commands: cargo-check --workspace --features full ; cargo-clippy --workspace --features full --no-deps -D warnings ; cargo-test --no-run -p lemmy_server --test e2e)
**Next:** advisor-laptop runs the three DoD commands, mutates DQ #<id>; on result=pass advisor finalize-merges + queues Task 3 (audit HTML handler + route)
```

Plus any `kind: "blocker"` DQ #N reference if you hit an OOM /
transitive-cascade / sensitive-file-gate / behaviour-preservation STOP
condition.

## §7 Why this brief differs from the plan

Clean execution of plan §13 Task 2 with the Shape-G-suspended
validate-pending-laptop substitution (per
`project_shape_g_suspended_2026_05_16` + DQ #229 — the cargo command
shapes are identical to the plan's VALIDATE block; only the handoff
mechanism is `kind: "validate-pending-laptop"` instead of a
`workflow_run_id`). The only other additions are the explicit CC
v2.1.119 sensitive-file-gate fallbacks (§4) and the submodule
pre-check (§4 — observed firing on Junior #286 at v1-AD-e Task 0);
both are belt-and-braces operational guards, not scope changes. The
import-only-`admin_dashboard_html` instruction (§2 file 4) is
copied verbatim from plan §13 Task 2 IMPLEMENT "file 4 of 4" — it is
the plan's own independently-compiling guidance, not a deviation.
