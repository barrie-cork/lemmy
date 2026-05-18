---
phase: v1-AD-e
role: impl-task
task: 3
brief_n: 3
authored: 2026-05-17
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: null
canonical_sibling: ".claude/PRPs/briefs/v1-AD-e-impl-2.md (§N house-style + validate-pending-laptop §5 shape); .claude/PRPs/templates/impl-task-brief.template.md"
scope_note: "REDUCED — Task 2 worker scope-bled the audit handler/render/script into admin_dashboard_html.rs; Task 3 is now route+import wiring + spec-conformance verification only."
---

# [role:impl-task] v1-AD-e task 3 — wire /audit/view route + import (audit handler pre-built) — see .claude/PRPs/briefs/v1-AD-e-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e task 3 — wire /audit/view route + import + verify pre-built audit handler`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 3**
from `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` §13 (lines
451–475) — **with a materially reduced scope** (see §2 + §7). The
Task 3 *handler* (`admin_audit_html`), *render* (`render_audit`,
`audit_entry_row`, `render_audit_table`) and the Task 4 *live-tail
`<script>`* (`AUDIT_SCRIPT` const) were **already implemented by the
Task 2 worker** (commit `ade904851`, scope-bleed) and are on
`phase-v1-AD-e` at HEAD. Task 3's remaining work is the **2-line
lib.rs route + import wiring** plus a **spec-conformance verification**
of the pre-built code.

## §2 Scope

**Produce (ONE commit, 1 file — `crates/api/routes/src/lib.rs` only):**

1. **modify** the governance handler import block: change
   `admin_dashboard_html::admin_dashboard_html` (currently lib.rs:42)
   to `admin_dashboard_html::{admin_dashboard_html, admin_audit_html}`.
2. **modify** the `/admin` route tree: add
   `.route("/view", get().to(admin_audit_html))` to the existing
   `scope("/audit")` service (currently lib.rs:558:
   `.service(scope("/audit").route("/stream", get().to(admin_audit_stream)))`)
   so it becomes
   `.service(scope("/audit").route("/stream", get().to(admin_audit_stream)).route("/view", get().to(admin_audit_html)))`
   — `/audit/stream` and `/audit/view` are siblings under the same
   `scope("/audit")`. (Final URL: `/api/v4/governance/admin/audit/view`.)

**Then VERIFY (read-only, no edit unless a deviation is found):** that
the pre-built `admin_audit_html` + `render_audit` + `AUDIT_SCRIPT`
(all in `crates/api/api/src/governance/admin_dashboard_html.rs`,
already on phase-v1-AD-e) conform to plan §13 Task 3 + Task 4 spec.
The verification checklist is §4 "Spec-conformance verification". If
**every** item passes, the ONLY edit is the 2-line lib.rs change. If
an item **fails**, fix that specific deviation in
`admin_dashboard_html.rs` in the same commit and note it in §6.

**Do NOT** in this task:

- Re-author `admin_audit_html` / `render_audit` / `audit_entry_row` /
  `render_audit_table` / `AUDIT_SCRIPT` from scratch — they exist and
  (per the advisor's pre-brief read) are spec-conformant. Touch them
  ONLY to correct a specific verification failure.
- Add the e2e test (Task 5).
- Change `admin_dashboard_html` / `render_dashboard` / `gather_dashboard`
  (Task 2, already validated — DQ #242 pass).
- Add a `/audit/view` route anywhere other than inside the existing
  `scope("/audit")` (it must be a sibling of `/stream`, NOT a
  top-level `/admin` route — the plan is explicit: "adjacent to the
  existing `scope("/audit").route("/stream", …)` service").

**Commit message** (exactly):
`feat(api): wire /audit/view route + admin_audit_html import (task 3)`

Commit body: note "audit handler/render/script pre-built by Task 2
worker (scope-bleed); Task 3 = route+import wiring", list whether the
verification found ANY deviation (and what was fixed if so), and a
`HANDOVER:` YAML trailer (filesModified / keyDecisions /
verificationResult / notes).

## §3 Required reading

In this order:

1. **Plan §13 Task 3** (`.claude/PRPs/plans/v1-admin-dashboard-e.plan.md`
   lines 451–475) — ACTION / FILES / IMPLEMENT (2 files) / MIRROR /
   GOTCHA / VALIDATE. **The contract.** Note: the plan's IMPLEMENT
   "file 1 of 2" (add `admin_audit_html` + `render_audit`) is
   **already done** by Task 2's worker — your job is IMPLEMENT "file
   2 of 2" (the lib.rs wiring) + verifying file 1 matches spec.
2. **Plan §13 Task 4** (lines 477–510) — the `AUDIT_SCRIPT` /
   live-tail spec. The const already exists; you verify it against
   this. **If `AUDIT_SCRIPT` fully conforms (it does per the
   advisor's read), Task 4 is effectively pre-satisfied** — the
   advisor will assess collapsing Task 4 to a no-op verification
   after Task 3 lands. Do NOT pre-empt that decision; just verify.
3. **The pre-built code to verify** —
   `crates/api/api/src/governance/admin_dashboard_html.rs`:
   - `admin_audit_html` (~line 241–260): auth + 404-gate + reuse +
     render + `text/html`.
   - `render_audit` (~262–308) + `audit_entry_row` (~310–334).
   - `AUDIT_SCRIPT` const (~35–63).
4. **Plan §10.5 + §10.2** — the audit-handler preamble contract
   (`is_admin?` → `html_pages_enabled`-404 → `list_recent_config_changes`
   reuse → render → `text/html`) and the SSE-frame contract
   (`admin_audit_stream.rs:245-248` — `event: admin_config_changed` /
   `admin_config_change_denied`, `retry: 10000`).
5. **MIRROR refs** (read to verify the pre-built code mirrors them):
   - `crates/api/api/src/governance/admin_audit_stream.rs:245-248` —
     the `event:<kind>\ndata:<json>` frame contract `AUDIT_SCRIPT`'s
     `addEventListener` names must match.
   - `crates/api/api_common/src/governance.rs:555-573` —
     `AdminConfigAuditEntry` field list (verify `render_audit` /
     `audit_entry_row` render every field with correct Option
     handling + `signature` as bool-not-bytes).
   - `crates/api/routes/src/lib.rs:558` — the existing
     `scope("/audit").route("/stream", …)` service you extend.
6. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — **always
   for cargo work.** Capture each cargo command to its
   `.claude/PRPs/debug/v1-AD-e-task3-*.log`, check the exit code
   separately, THEN tail.
7. **`.claude/lessons/feedback_clippy_test_style.md`** — workspace
   denies `unwrap`/`expect`/`#[allow]`. (The pre-built code is
   already clippy-clean from Task 2's DoD; only a verification-fix
   could regress it — keep `LemmyResult<T>` + `?`.)
8. **`.claude/agents/impl-task.md` "Pre-Shape-G plans"** + the
   `impl-task-brief.template.md` §5 — Shape G is SUSPENDED until
   2026-06-01 (`project_shape_g_suspended_2026_05_16`, DQ #229).
   After pushing, write a `kind: "validate-pending-laptop"` DQ entry
   (NOT `kind: "validate-pending"`, NO `workflow_run_id`). Exact
   shape in §5.

## §3a Handover from prior task

```yaml
prior_task:
  task: 2
  commit: ade904851
  finalize_merge: a0a19637d
  validate_dq: 242 (result=pass; check 1m47s + clippy 5m35s + test --no-run 10m44s, all exit 0)
  filesCreated:
    - crates/api/api/src/governance/admin_dashboard_html.rs
  filesModified:
    - crates/api/api/src/governance/admin_dashboard.rs
    - crates/api/api/src/governance/mod.rs
    - crates/api/routes/src/lib.rs
  keyDecisions:
    - "gather_dashboard takes Data<LemmyContext> not DbPool (lifetime borrow conflict)"
    - "module is pub not pub(crate) — lemmy_routes is a separate crate"
    - "AUDIT_SCRIPT is a const str outside html!{} to avoid maud raw-string parse error"
    - "element IDs use id=\"...\" attribute not #\"...\" shorthand (maud parser)"
  scope_bleed: >
    Task 2 worker ALSO implemented admin_audit_html + render_audit +
    audit_entry_row + render_audit_table + AUDIT_SCRIPT (Task 3 + Task 4
    code) into admin_dashboard_html.rs, but correctly did NOT add the
    /audit/view route or the lib.rs import for admin_audit_html (those
    stay Task 3). Advisor verified (pre-brief read) the pre-built
    handler + script are spec-conformant. Task 3 is therefore reduced
    to the lib.rs route+import wiring + a verification pass.
  notes: >
    list_recent_config_changes was made pub(crate) in admin_dashboard.rs
    by the Task 2 worker specifically so admin_audit_html could reuse it.
    DQ pending=0 at Task 3 dispatch (DQ #241+#242 both resolved=pass).
```

## §4 Constraints + Spec-conformance verification

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-AD-e` (current tip
  `371ffaac8` — Task 1+2 merged, DQ #241+#242 resolved). Finalize
  merges your worktree branch back; do NOT push to `phase-v1-AD-e`
  directly.
- **ONE commit**, normally 1 file (`lib.rs`). If the verification
  finds a deviation, the fix to `admin_dashboard_html.rs` goes in the
  **same** commit (do not split).
- Mid-task DQ visibility: a `pending` blocker → **commit + push
  immediately** to your worktree branch.
- No `answered_by: "advisor"` / `"user"`. Self-resolve only as
  `"impl-self-resolved"`. The `validate-pending-laptop` entry is
  `from: "impl"`, `answered_by: null`.

### Spec-conformance verification (READ-ONLY checklist — run before editing lib.rs)

Read `crates/api/api/src/governance/admin_dashboard_html.rs` and
confirm EACH. A ✗ on any line → fix that specific thing in the same
commit + document in §6. A ✓ on all → lib.rs 2-line edit is the only
change.

**`admin_audit_html` handler (plan §13 Task 3 IMPLEMENT file 1):**
- [ ] `is_admin(&local_user_view)?` is the first auth gate.
- [ ] reads `governance.dashboard.html_pages_enabled` via the SAME
  config accessor `admin_dashboard_html` uses (`get_bool` +
  `HTML_PAGES_KEY` + `Scope::Instance`).
- [ ] flag `false` → `Ok(HttpResponse::NotFound().finish())`
  (**404, NOT 403** — R-html-3; 403 is only the `is_admin?` path).
- [ ] reuses `list_recent_config_changes(conn)` (the `pub(crate)` fn
  from `admin_dashboard.rs`) — does NOT re-query `governance_config`
  by hand.
- [ ] returns `text/html; charset=utf-8` body.
- [ ] signature `(context: Data<LemmyContext>, local_user_view:
  LocalUserView) -> LemmyResult<HttpResponse>` (mirrors
  `admin_dashboard_html`).

**`render_audit` / `audit_entry_row` (plan §13 Task 3 GOTCHA):**
- [ ] renders every `AdminConfigAuditEntry` field per
  `governance.rs:555-573`: id, created_at, entry_kind, scope, key,
  previous_value, new_value, reason, actor_pseudonym, signature,
  denial_reason.
- [ ] `signature: Option<Vec<u8>>` rendered as a **bool** (`yes`/`no`
  or signed:true/false) — **never raw bytes** into HTML.
- [ ] `previous_value` / `denial_reason` / `actor_pseudonym` are
  `Option` — rendered with `@if let Some(..)` / `@else { "" }` (no
  `.unwrap()`; None on pre-v1-AD-c rows must not panic).
- [ ] `new_value` / `previous_value` are `serde_json::Value` —
  rendered via `.to_string()` (not `{:?}`).
- [ ] maud auto-escapes string fields (no `PreEscaped`/raw on
  user-derived strings; `PreEscaped` is acceptable ONLY for the
  static `<style>` and the `AUDIT_SCRIPT` const).

**`AUDIT_SCRIPT` const (plan §13 Task 4 IMPLEMENT + GOTCHA):**
- [ ] `new EventSource("/api/v4/governance/admin/audit/stream")` —
  exact stream URL.
- [ ] uses `es.addEventListener("admin_config_changed", …)` AND
  `es.addEventListener("admin_config_change_denied", …)` — **named
  events, NOT `es.onmessage`** (the #1 SSE footgun: the server sends
  `event: <kind>` named frames; `onmessage` alone receives nothing).
- [ ] prepends rows (`insertBefore(row, tbody.firstChild)`).
- [ ] `es.onerror` shows a muted reconnecting note (EventSource
  auto-reconnects per server `retry:` — script must NOT implement
  its own backoff).
- [ ] JS field names match the serde snake_case DTO
  (`previous_value`, `new_value`, `actor_pseudonym`, `denial_reason`,
  `entry_kind`, `created_at`).
- [ ] embedded in `render_audit`'s output via `PreEscaped(AUDIT_SCRIPT)`.
- [ ] one HTML-comment or visible note documenting the per-admin SSE
  cap (second tab → 409 → reconnecting).

(Advisor pre-brief read on `371ffaac8` found ALL of the above ✓ —
this checklist is your independent re-verification, not a
formality. If your read disagrees with the advisor's, trust your
read and fix + document.)

### Plan-cited line numbers may have drifted

Verify before editing:
`grep -n 'admin_dashboard_html::\|scope("/audit")\|/stream"' crates/api/routes/src/lib.rs`.
Current (HEAD `371ffaac8`): import at lib.rs:42, `scope("/audit")`
service at lib.rs:558. Follow grep output if they moved.

### Memory-cap awareness

The daemon runs `MemoryMax=10G`. Task 3 DoD runs
`cargo-check --workspace --features full` — the workspace is fully
warm from Task 2 + a 2-line lib.rs change is a near-instant
incremental recompile. If it OOM-kills (`Killed`, non-zero), do NOT
retry blindly — file a `kind: "blocker"` DQ with the cargo log tail.

### CC v2.1.119 sensitive-file gate (observed on Junior #282)

Writes under `.claude/**` MAY be blocked even in `bypassPermissions`
mode (the `.claude/PRPs/debug/*.log` logs + the
`validate-pending-laptop` DQ write).

- **`.claude/PRPs/debug/` log write denied:** redirect that cargo
  command's output to `<worktree-root>/<same-filename>.log`; note in
  §6. Exit code is the load-bearing signal.
- **`.claude/decision-queue.json` `validate-pending-laptop` write
  denied:** write the intended entry JSON to
  `<worktree-root>/v1-AD-e-task3-VALIDATE-PENDING.json` and STOP with
  a clear escalation naming that file. Do NOT silently skip the
  handoff.

### Submodule pre-check (per `feedback_worktree_submodules_not_auto_init.md`)

Before the first cargo command (Task 3 DoD compiles `lemmy_email`):

```bash
git submodule update --init --recursive
ls crates/email/translations/backend/ | head -3   # expect *.json
```

If `git submodule update` fails (network), file a `kind: "blocker"`
DQ — do NOT proceed to a cargo command that fails with
`Os { code: 3, NotFound }`. (Observed firing on Junior #286 at
v1-AD-e Task 0.)

## §5 Validation gates (per plan §13 Task 3 VALIDATE block) — Shape-G SUSPENDED

**Shape G is suspended until 2026-06-01.** After committing + pushing
your worktree branch, write a **`kind: "validate-pending-laptop"`** DQ
entry to `.claude/decision-queue.json` (commit + push it on your
worktree branch immediately so the advisor sees it on next fetch).

Do NOT write `kind: "validate-pending"`. Do NOT capture a
`workflow_run_id`. The advisor laptop session runs the commands and
mutates the entry.

**The `commands[]` array MUST contain, verbatim (plan §13 Task 3
VALIDATE — same three-command block as Task 2):**

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-task3-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-task3-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-task3-testnorun.log 2>&1"
```

All three EXPECT exit 0. (`test --no-run` because the new
`/audit/view` route + import changes the `lemmy_routes` →
`lemmy_server` call surface; R7 confirms the e2e binary still links.)

**`validate-pending-laptop` entry shape** (per
`.claude/rules/decision-queue.md` "kind: validate-pending-laptop" +
template §5):

```json
{
  "id": "<max(all ids across .claude/decision-queue.json + .claude/decision-queue-archive-*.json) + 1>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-AD-e Task 3 (wire /audit/view route + admin_audit_html import) pushed on <worktree-branch> — run §13-Task-3 DoD on laptop",
  "branch": "<your worktree branch name>",
  "phase_task": 3,
  "commands": ["<the three cmd //c lines above, verbatim>"],
  "context": "lib.rs: +/audit/view route in scope(\"/audit\") + admin_audit_html added to governance import. Audit handler/render/script were pre-built by Task 2 (scope-bleed) + verified spec-conformant <or: + verification fixed: <what>>. commit <sha>. Pre-Shape-G validate-pending-laptop (Shape G suspended until 2026-06-01, DQ #229).",
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
both" — the DQ #50 collision lesson). Current max id is **242** (DQ
#241+#242 resolved) → next is **243** unless an archive holds higher;
check. Do NOT reuse an id.

If a local cargo gate fails when YOU run a sanity pre-check
(encouraged per `feedback_local_runtime_before_push`), do NOT push a
broken tree — fix the in-scope cause first, or if it is an
out-of-scope cascade, file a `kind: "blocker"` DQ instead of the
`validate-pending-laptop` entry and STOP.

## §6 Expected output (return to advisor)

```
## Task 3 complete — v1-AD-e wire /audit/view route + admin_audit_html import

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/routes/src/lib.rs (+admin_audit_html to governance import; +.route("/view", get().to(admin_audit_html)) in scope("/audit"))
  - [crates/api/api/src/governance/admin_dashboard_html.rs — ONLY if a verification deviation was fixed; else "not modified (pre-built code verified spec-conformant)"]
**Spec-conformance verification:** ALL ✓ (handler 404-gate + list_recent_config_changes reuse; render_audit Option/signature-bool; AUDIT_SCRIPT named-events + stream URL)  |  DEVIATION FIXED: <what + where>
**Task 4 status note:** AUDIT_SCRIPT pre-built + spec-conformant + embedded — Task 4 (audit live-tail <script>) appears pre-satisfied; advisor to confirm Task 4 collapses to verification.
**Validate handoff:** wrote DQ #<id> kind=validate-pending-laptop, from=impl, branch=<worktree-branch>, phase_task=3 (commands: cargo-check --workspace --features full ; cargo-clippy --workspace --features full --no-deps -D warnings ; cargo-test --no-run -p lemmy_server --test e2e)
**Next:** advisor-laptop runs the three DoD commands, mutates DQ #<id>; on result=pass advisor finalize-merges + assesses Task 4 (likely verification-only) then Task 5 (e2e test)
```

Plus any `kind: "blocker"` DQ #N reference if you hit an OOM /
verification-deviation-that-needs-a-design-decision /
sensitive-file-gate STOP condition.

## §7 Why this brief differs from the plan

**Materially reduced scope vs plan §13 Task 3** — and this is the
load-bearing deviation, called out per
`feedback_advisor_instruction_mismatch_stop_and_ask` discipline:

The plan §13 Task 3 IMPLEMENT has two files: (file 1) add
`admin_audit_html` + `render_audit` to `admin_dashboard_html.rs`;
(file 2) wire the lib.rs route + import. **The Task 2 Junior worker
(commit `ade904851`) already implemented file 1 in full** —
`admin_audit_html`, `render_audit`, `audit_entry_row`,
`render_audit_table`, AND the Task 4 `AUDIT_SCRIPT` const — as
scope-bleed beyond its Task 2 brief (which said "do NOT add
admin_audit_html — Task 3"). The worker correctly stopped short of
the lib.rs route/import (those stay Task 3). The advisor verified the
pre-built code on `371ffaac8` against plan §13 Task 3 + Task 4 spec
(handler 404-gate, `list_recent_config_changes` reuse, Option
handling, `signature`-as-bool, `AUDIT_SCRIPT` named-events + correct
stream URL) and found it **spec-conformant**.

Therefore Task 3 is reduced to: (a) the 2-line lib.rs route+import
wiring (plan file 2 of 2, unchanged), and (b) an independent
spec-conformance re-verification of the pre-built file 1 (§4
checklist) — with authority to fix a specific deviation in the same
commit if the worker's re-read disagrees with the advisor's.

**Consequence for Task 4:** the plan's Task 4 (finalise the audit
live-tail `<script>`) is `AUDIT_SCRIPT`, which already exists,
already conforms, and is already embedded via
`PreEscaped(AUDIT_SCRIPT)` in `render_audit`. **Task 4 is almost
certainly pre-satisfied** — the advisor will assess collapsing Task 4
to a no-op verification (or a tiny touch-up if Task 3's verification
surfaces a `<script>` deviation) once Task 3's DoD is green. This
brief does NOT pre-decide Task 4; it flags it (§6 "Task 4 status
note") for the advisor.

The only other additions are the standard Shape-G-suspended
`validate-pending-laptop` substitution (§5), the CC v2.1.119
sensitive-file-gate fallbacks (§4), and the submodule pre-check (§4
— observed firing on Junior #286 at v1-AD-e Task 0). The
three-command DoD block (check + clippy --workspace --no-deps -D
warnings + test --no-run -p lemmy_server --test e2e) is copied
verbatim from plan §13 Task 3 VALIDATE.
