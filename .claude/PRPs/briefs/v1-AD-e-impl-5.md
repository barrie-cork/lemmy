---
phase: v1-AD-e
role: impl-task
task: 5
brief_n: 5
authored: 2026-05-17
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: null
canonical_sibling: ".claude/PRPs/briefs/v1-AD-e-impl-3.md (house-style + §5 validate-pending-laptop shape); e2e sibling: crates/server/tests/e2e.rs admin-route-sweep module (~3805-4045, actix test::init_service + make_user(is_admin) + mint_jwt + call_service + status().as_u16()) AND mod v1_sl_e_fixtures (13846+, latest Case-A LemmyResult<()> discipline)"
mandatory_lessons_fired:
  - feedback_lemmy_error_no_std_error.md   # e2e.rs edit — Case A enumeration
  - feedback_async_pool_test_pattern.md    # e2e.rs edit — AsyncPgConnection/DbPool seed
  - feedback_junior_worker_e2e_edit_hang.md # e2e.rs >8000 lines — append-only, own task, never bundled
---

# [role:impl-task] v1-AD-e task 5 — e2e test (admin 200 / non-admin 403 / flag-off 404) — see .claude/PRPs/briefs/v1-AD-e-impl-5.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e task 5 — e2e test: admin 200 / non-admin 403 / flag-off 404 for /dashboard/view (+/audit/view)`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 5**
from `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` §13 (lines
512–538). This is the FINAL implementation task: a single append-only
fixtures module added to `crates/server/tests/e2e.rs` asserting the
three behaviours of the v1-AD-e HTML pages. Tasks 1–4 are complete +
validated (maud dep; gather_dashboard extract + `/dashboard/view`;
`/audit/view` route + import; AUDIT_SCRIPT pre-satisfied).

## §2 Scope

**Produce (ONE commit, 1 file — `crates/server/tests/e2e.rs`,
append-only):** ONE new `mod v1_ad_e_fixtures { … }` fixtures module
(or a single test fn appended to a new module) at the END of the file
(after `mod v1_sl_e_fixtures`, before EOF), asserting:

1. **admin → 200** — GET `/api/v4/governance/admin/dashboard/view`
   with an instance-admin JWT → assert `200`,
   `content-type: text/html; charset=utf-8`, body contains a stable
   marker (a seeded case's status string OR a fixed page heading the
   maud `render_dashboard` emits — read `admin_dashboard_html.rs` to
   pick a literal that will always be present, e.g. an `<h1>` text).
2. **non-admin → 403** — GET the same path with a NON-admin user's
   JWT → assert `403` (this is the `is_admin?` rejection, NOT the
   feature-gate).
3. **flag-off → 404** — seed
   `governance.dashboard.html_pages_enabled = false` at
   `ConfigScope::Instance` via the EXISTING config-write path (the
   same accessor/handler v1-AD config tests use — do NOT raw-INSERT
   into `governance_config`), then GET `/dashboard/view` with the
   admin JWT → assert `404` (the feature-gate, distinct from 403).
4. *(optional, encouraged — same module)* repeat the 200-admin /
   403-non-admin pair for `/api/v4/governance/admin/audit/view` (the
   Task 3 route). The flag-off-404 only needs asserting once
   (`html_pages_enabled` gates both pages identically).

**Do NOT** in this task:

- Touch any file other than `crates/server/tests/e2e.rs`.
- Modify `admin_dashboard.rs` / `admin_dashboard_html.rs` /
  `mod.rs` / `lib.rs` (Tasks 2–4, all merged + validated).
- Bundle any other task into this commit (`e2e.rs` is >14 000 lines —
  `feedback_junior_worker_e2e_edit_hang`: this is its own task, ONE
  append-only edit, never combined with handler logic).
- Edit anywhere except the END of the file. Append a new module; do
  NOT splice into an existing `mod v1_*_fixtures`.

**Commit message** (exactly):
`test(v1-AD-e): e2e — admin 200 / non-admin 403 / flag-off 404 for dashboard+audit HTML pages (task 5)`

Commit body: name the sibling module + line range you mirrored
(canonical-schema-first), the stable body marker you asserted, and a
`HANDOVER:` YAML trailer.

## §3 Required reading

**MANDATORY (file-class lesson injection — `crates/server/tests/e2e.rs`
edit, per `.claude/rules/advisor-orchestrator.md` §2.4). Read these
THREE before anything else:**

1. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — the
   error-shape contract. The e2e file uses **Case A**:
   `LemmyResult<()>` outer on the `#[tokio::test]` fn, ALL helpers
   return `LemmyResult<T>`, **NO `.map_err` bridges, NO
   `Box<dyn Error>`**. The sibling modules
   (`v1_sl_e_fixtures` line 13846+, the admin-route-sweep module
   ~3805) all use Case A verbatim. **Mirror Case A exactly** — do not
   introduce `Box<dyn std::error::Error>` or `.map_err(|e| …)?`
   bridges. Test fn signature:
   `async fn <name>() -> lemmy_utils::error::LemmyResult<()>`.
2. **`.claude/lessons/feedback_async_pool_test_pattern.md`** — the
   DB-seed pattern: `governance_fixtures::start_postgres()` +
   `db_url` + `apply_all_schema(&mut sync_conn)` +
   `build_db_pool_for_tests()` + `LemmyContext::create(...)`, then
   `&mut context.pool()` / `AsyncPgConnection` for seeds. Mirror the
   sibling's setup block verbatim.
3. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** —
   `e2e.rs` is >14 000 lines. ONE append-only edit at EOF. Do NOT
   read-modify-rewrite the whole file; do NOT make >1 edit; append a
   single new module. Multiple edits / large in-place rewrites hang
   the Junior worker.

Then, in order:

4. **Plan §13 Task 5** (`.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
   lines 512–538) — ACTION / FILES / IMPLEMENT / MIRROR / GOTCHA /
   VALIDATE. **The contract.**
5. **Canonical sibling A — the admin-route-sweep module** in
   `crates/server/tests/e2e.rs` **~lines 3805–4045**. This is the
   closest shape to Task 5: it builds the actix app via
   `test::init_service(App::new().app_data(...).wrap(SessionMiddleware)
   .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)))`,
   defines `async fn make_user(ctx, instance_id, name, is_admin) ->
   LemmyResult<(LocalUserId, PersonId)>` (admin via
   `LocalUserInsertForm::test_form_admin`, non-admin via
   `::test_form`), `async fn mint_jwt(ctx, local_user_id) ->
   LemmyResult<String>` (via `Claims::generate(...)`), then issues
   `test::TestRequest::get().uri(path).insert_header(("authorization",
   format!("Bearer {jwt}"))).to_request()` + `test::call_service` +
   `resp.status().as_u16()`. **Mirror this make_user + mint_jwt +
   request + status-assert shape verbatim** — it is the proven
   admin-JWT-vs-non-admin HTTP pattern.
6. **Canonical sibling B — `mod v1_sl_e_fixtures`** (`e2e.rs` line
   **13846+**, the most-recent fixtures module) — for the current
   `use super::*;` import block shape, the Case-A `LemmyResult<()>`
   helper signatures, and `governance_fixtures::seed_user` /
   `start_postgres` / `apply_all_schema` usage idioms. Your new module
   goes AFTER this one (EOF).
7. **`crates/api/api/src/governance/admin_dashboard_html.rs`** (on
   phase tip) — read `admin_dashboard_html` (auth → 404-gate → gather
   → render), `render_dashboard`, and `render_audit` to pick a
   **stable literal body marker** that the maud output always contains
   (e.g. an `<h1>` heading text or a fixed `<title>`), and to confirm
   the exact route paths (`/dashboard/view`, `/audit/view`) and the
   config key string (`HTML_PAGES_KEY` =
   `"governance.dashboard.html_pages_enabled"`).
8. **The config-write path used by v1-AD config tests** — grep
   `e2e.rs` for how existing v1-AD/SL tests set a `governance_config`
   row at `ConfigScope::Instance` (the config-write handler/helper).
   Use that EXACT path for the flag-off-404 case (GOTCHA: default is
   `true`, so the 200/403 cases need NO flag row; only the 404 case
   writes `false`).

## §3a Handover from prior task

```yaml
prior_tasks:
  - task: 2
    commit: ade904851
    note: "admin_dashboard_html + render_dashboard + gather_dashboard extracted; /dashboard/view route. Validated DQ #242=pass."
  - task: 3
    commit: 5afe5b378
    note: "/audit/view route + admin_audit_html import wired. admin_audit_html + render_audit pre-built by Task 2. Validated DQ #243=pass."
  - task: 4
    disposition: "PRE-SATISFIED — AUDIT_SCRIPT verified 8/8 spec-conformant on phase tip; no code change, no Junior dispatch."
key_facts_for_task_5:
  - "Routes live: GET /api/v4/governance/admin/dashboard/view → admin_dashboard_html; GET /api/v4/governance/admin/audit/view → admin_audit_html (both under scope('/admin') → scope nesting in lib.rs)."
  - "Auth order in both handlers: is_admin(&local_user_view)? FIRST (→403 for non-admin), THEN html_pages_enabled get_bool (→404 if false). 403 and 404 are distinct cases — assert the right one."
  - "Config key: governance.dashboard.html_pages_enabled, ConfigScope::Instance, default true. Read via get_bool(&mut cache, &mut pool, Scope::Instance, HTML_PAGES_KEY)."
  - "content-type on success: text/html; charset=utf-8."
  - "Mirror Case A (LemmyResult<()>); the admin-route-sweep module ~3805 is the closest existing pattern; v1_sl_e_fixtures (13846) is the most-recent module + your insertion anchor (append after it)."
```

## §4 Constraints

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-AD-e` (current tip has
  Tasks 1–4 merged: `6c7cd7621`-or-later). Finalize merges your
  worktree branch back; do NOT push to `phase-v1-AD-e` directly.
- **ONE commit, ONE file, ONE append-only edit.** No fixups that
  re-rewrite the file. If the test needs iteration, make the SMALLEST
  possible follow-up edit to the new module only.
- Mid-task DQ visibility: a `pending` blocker → **commit + push
  immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as
  `"impl-self-resolved"`. The `validate-pending-laptop` entry is
  `from: "impl"`, `answered_by: null`.

### e2e-edit-hang prevention (per `feedback_junior_worker_e2e_edit_hang`)

`e2e.rs` is >14 000 lines. The Junior worker hangs on a full-file
Edit. Use a SINGLE append: read ONLY the tail (e.g. last ~60 lines, to
see the EOF + the close of `mod v1_sl_e_fixtures`) and the two sibling
ranges cited in §3 (3805–4045, 13846+) — do NOT Read the whole file.
Append the new module with ONE Edit/Write-append operation. Do NOT
make multiple edits to e2e.rs in this task.

### Error-shape (per `feedback_lemmy_error_no_std_error.md` — Case A)

The sibling modules use `LemmyResult<()>` outer + `LemmyResult<T>`
helpers + `?` throughout, **no `.map_err` bridges, no
`Box<dyn Error>`**. Mirror Case A verbatim. A `Box<dyn Error>` outer
or a `.map_err(|e| format!("{e}").into())?` bridge is a hard
contract violation here (the file is uniformly Case A; mixing shapes
triggers the E0277 cascade the lesson documents). If `?` on a
Lemmy-native call "doesn't convert", the fix is NOT a bridge — it is
to match the sibling's exact return type (`LemmyResult<()>`), which
already works for every helper in `v1_sl_e_fixtures`.

### Memory-cap + e2e runtime

The daemon runs `MemoryMax=10G`. **The Task 5 DoD includes the REAL
e2e run** (not just `--no-run`) because Task 5 IS the test — it must
actually execute against testcontainers Postgres (Docker required,
~26 min). The Junior worker does NOT run the full e2e itself (watchdog
+ memory); it writes the `validate-pending-laptop` entry and the
ADVISOR-LAPTOP runs the e2e (Docker up on the laptop) per §5. The
worker SHOULD run `cargo-test --no-run -p lemmy_server --test e2e`
locally as a compile sanity check before pushing (encouraged per
`feedback_local_runtime_before_push`) but must NOT attempt the full
e2e run on the daemon.

### CC v2.1.119 sensitive-file gate (observed on Junior #282/#289)

Writes under `.claude/**` MAY be blocked. This affects (a) the
`.claude/PRPs/debug/*.log` logs, (b) the `validate-pending-laptop`
write to `.claude/decision-queue.json`.

- **`.claude/PRPs/debug/` log write denied:** redirect to
  `<worktree-root>/<same-filename>.log`; note in §6.
- **`.claude/decision-queue.json` write denied:** write the intended
  entry JSON to `<worktree-root>/v1-AD-e-task5-VALIDATE-PENDING.json`
  and STOP with a clear escalation naming that file (Junior #289 hit
  exactly this — the advisor relocates it into the canonical DQ). Do
  NOT silently skip the handoff.

### Submodule pre-check (per `feedback_worktree_submodules_not_auto_init.md`)

Before any cargo command:

```bash
git submodule update --init --recursive
ls crates/email/translations/backend/ | head -3   # expect *.json
```

Network failure on `git submodule update` → file a `kind: "blocker"`
DQ; do NOT proceed to a cargo command that fails with
`Os { code: 3, NotFound }`. (Observed on Junior #286 Task 0.)

### Plan-cited line numbers may have drifted

`e2e.rs` line ranges (3805–4045 admin-sweep, 13846 v1_sl_e) are from
the advisor's read on phase tip `6c7cd7621`. Verify before mirroring:
`grep -n 'mod v1_sl_e_fixtures\|async fn make_user\|async fn mint_jwt\|test::init_service' crates/server/tests/e2e.rs`. Follow grep output
if they moved.

## §5 Validation gates (per plan §13 Task 5 VALIDATE block) — Shape-G SUSPENDED

**Shape G is suspended until 2026-06-01.** After committing + pushing
your worktree branch, write a **`kind: "validate-pending-laptop"`** DQ
entry to `.claude/decision-queue.json` (commit + push on your worktree
branch immediately).

Do NOT write `kind: "validate-pending"`. Do NOT capture a
`workflow_run_id`. The advisor-laptop runs the commands (including the
full e2e — Docker up) and mutates the entry.

**The `commands[]` array MUST contain, verbatim — FOUR commands (Task
5 adds the real e2e run; per plan §13 Task 5 VALIDATE + §15.4):**

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-task5-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task5-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-task5-testnorun.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-AD-e-task5-e2e.log 2>&1"
```

First three EXPECT exit 0. The fourth (full e2e) EXPECT exit 0 with
the new v1-AD-e test(s) PASSING — advisor-laptop runs it with Docker
up (~26 min). **Never bare `cargo test`** (libpq.dll — bat wrapper
sets vcpkg PATH per `feedback_windows_e2e_requires_bat_wrapper`);
**never `-p lemmy_server --features full`** for the run (use
`--workspace --test e2e --features full`).

**`validate-pending-laptop` entry shape:**

```json
{
  "id": "<max(all ids across .claude/decision-queue.json + .claude/decision-queue-archive-*.json) + 1>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-AD-e Task 5 (e2e admin 200 / non-admin 403 / flag-off 404) pushed on <worktree-branch> — run §13-Task-5 DoD (incl full e2e) on laptop",
  "branch": "<your worktree branch name>",
  "phase_task": 5,
  "commands": ["<the four cmd //c lines above, verbatim>"],
  "context": "One append-only fixtures module added to crates/server/tests/e2e.rs: /dashboard/view + /audit/view admin-200 / non-admin-403 / flag-off-404. Mirrored sibling <module> (e2e.rs:<lines>). commit <sha>. Pre-Shape-G validate-pending-laptop (Shape G suspended until 2026-06-01, DQ #229). Task 5 needs the FULL e2e run (Docker) — advisor-laptop runs it.",
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
both"). Current max id is **243** (DQ #241/#242/#243 resolved) → next
is **244** unless an archive holds higher; check. Do NOT reuse an id.

If your local `--no-run` compile sanity check fails, do NOT push a
broken tree — fix the in-scope cause first (mirror the sibling Case-A
shape exactly), or if it is an out-of-scope cascade, file a
`kind: "blocker"` DQ instead of the `validate-pending-laptop` entry
and STOP.

## §6 Expected output (return to advisor)

```
## Task 5 complete — v1-AD-e e2e (admin 200 / non-admin 403 / flag-off 404)

**Commit:** <sha> on <worktree-branch>
**Files changed:** crates/server/tests/e2e.rs (+1 append-only fixtures module `mod v1_ad_e_fixtures`, N test fn(s))
**Mirrored sibling:** <module name> (e2e.rs:<line range>) — Case A LemmyResult<()> verbatim
**Body marker asserted:** "<the stable literal from render_dashboard you matched>"
**Cases:** /dashboard/view admin→200(text/html) | non-admin→403 | flag-off→404 [; /audit/view admin→200 | non-admin→403]
**Local --no-run sanity:** exit 0 (full e2e deferred to advisor-laptop per §5)
**Validate handoff:** wrote DQ #<id> kind=validate-pending-laptop, from=impl, branch=<worktree-branch>, phase_task=5 (4 commands incl full e2e run)
**Next:** advisor-laptop runs check+clippy+test--no-run+FULL e2e (Docker up, ~26min), mutates DQ #<id>; on result=pass advisor finalize-merges → /brehon-verify → CR/PR cycle → merge → retro
```

Plus any `kind: "blocker"` DQ #N reference if you hit an
error-shape-cascade / sensitive-file-gate / submodule STOP condition.

## §7 Why this brief differs from the plan

Clean execution of plan §13 Task 5 with the standard Shape-G-suspended
`validate-pending-laptop` substitution (§5) — **but the commands[]
array adds a FOURTH command: the real full e2e run**
(`cargo-test --workspace --test e2e --features full`). Plan §13 Task 5
VALIDATE specifies the full e2e run (Task 5 IS the e2e test — a
`--no-run` link check is necessary but not sufficient to prove the
three assertions pass). Under Shape-G-suspended, the advisor-laptop
(Docker up) executes that run per the validate-pending-laptop handler;
the Junior worker does NOT run full e2e on the daemon (watchdog +
`MemoryMax=10G`). This mirrors the prior v1-SL-*/v1-JM-* e2e-task
pattern (`validate-pending-laptop-e2e` style) — kept as
`validate-pending-laptop` with the e2e command inline for kind
simplicity.

The other additions are standard operational guards observed firing
this phase: CC v2.1.119 sensitive-file-gate fallbacks (§4 — Junior
#289 hit the DQ-write block), the submodule pre-check (§4 — Junior
#286 Task 0), and the e2e-edit-hang single-append discipline (§4 —
`feedback_junior_worker_e2e_edit_hang`, e2e.rs >14 000 lines). The
mandatory file-class lessons (`feedback_lemmy_error_no_std_error.md`,
`feedback_async_pool_test_pattern.md`,
`feedback_junior_worker_e2e_edit_hang.md`) are injected in §3 per
`.claude/rules/advisor-orchestrator.md` §2.4 (e2e.rs edit → all three
fire mechanically). No scope deviation — Task 5 is exactly the plan's
e2e task.
