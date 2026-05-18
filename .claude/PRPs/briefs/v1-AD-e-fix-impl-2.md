---
phase: v1-AD-e
role: impl-task
task: fix-impl-2
brief_n: 2
authored: 2026-05-17
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: pr-133 CR finding cr-5 (ADR-015 audit-string scrub) — fix-impl-1 (#297, commit 5d742a323) addressed it PARTIALLY (reason + denial_reason only); user gate-5 2026-05-17 chose "complete cr-5 first, then merge". This task completes the ADR-015 scrub surface.
canonical_sibling: ".claude/PRPs/briefs/v1-AD-e-fix-impl-1.md (house-style + §5 validate-pending-laptop 4-command shape); scrub mirror: crates/api/api/src/governance/admin_rule_sets.rs:55 (use crate::governance::redaction::scrub) + :329 (rule_text: scrub(&rsv.rule_text)); the partial fix already on the phase branch: admin_dashboard_html.rs audit_entry_row L323 scrub(&e.reason) + L331 scrub(d) — extend this same pattern to the remaining fields"
mandatory_lessons_fired:
  # No e2e.rs edit, no migration, no new test, no #[cfg(feature)] gate — none of the advisor-orchestrator §2.4 mechanical rows fire. Pure handler-render scrub extension mirroring an already-present in-file pattern.
---

# [role:impl-task] v1-AD-e fix-impl-2 — complete cr-5 ADR-015 scrub on PR #133 — see .claude/PRPs/briefs/v1-AD-e-fix-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e fix-impl-2 — complete cr-5 ADR-015 scrub: scope/key/entry_kind/actor_pseudonym via scrub() + previous_value/new_value via scrub_json() in audit_entry_row`

You are the **impl-task** subagent (Sonnet 4.6). This **completes** CR
finding **cr-5** on PR #133 (`phase-v1-AD-e` → `governance-v0`).
fix-impl-1 (Junior #297, commit `5d742a323`, already on the phase
branch) addressed cr-5 **partially** — it scrubbed only `e.reason`
and `e.denial_reason`. The user, at merge-confirm gate 5, decided the
remaining ADR-015 fields MUST be scrubbed before merge. This task adds
the rest. Your worktree branches from `phase-v1-AD-e` (base_branch set
by the advisor; tip already has `5d742a323`).

## §2 Scope — ONE commit, ONE file

Produce **ONE commit** touching exactly ONE file:
`crates/api/api/src/governance/admin_dashboard_html.rs`, function
**`audit_entry_row`** (~L311–335) ONLY.

### 2.1 — the canonical scrub utilities (use ONLY these — do NOT invent)

Two functions in `crate::governance::redaction` (canonical ADR-015
util, already used in this codebase — mirror `admin_rule_sets.rs:55,329`
and the partial fix already present at `admin_dashboard_html.rs:323,331`):

- `pub fn scrub(text: &str) -> String` — for `&str` / `String` fields.
- `pub fn scrub_json(value: &serde_json::Value) -> serde_json::Value`
  — for `serde_json::Value` fields. Returns a scrubbed `Value`; call
  `.to_string()` on the result to render it (mirror the existing
  `(v.to_string())` / `(e.new_value.to_string())` shape, just feed it
  the scrubbed Value).

The file's import block currently has
`use crate::governance::{ ... redaction::scrub };`. **Extend it to**
`use crate::governance::{ ... redaction::{scrub, scrub_json} };`
(keep the rest of the grouped `use` exactly as-is; just widen the
`redaction::` leaf to a `{scrub, scrub_json}` group).

### 2.2 — exact field-by-field changes in `audit_entry_row`

`AdminConfigAuditEntry` field types are **verified** (defined at
`crates/api/api_common/src/governance.rs:555–573`). Apply EXACTLY these,
mirroring the already-present `scrub(&e.reason)` / `scrub(d)` pattern:

| Line (current) | Current source | Field type | Change to |
|---|---|---|---|
| ~L316 | `td { (e.entry_kind) }` | `entry_kind: String` | `td { (scrub(&e.entry_kind)) }` |
| ~L317 | `td { (e.scope) }` | `scope: String` | `td { (scrub(&e.scope)) }` |
| ~L318 | `td { (e.key) }` | `key: String` | `td { (scrub(&e.key)) }` |
| ~L319-321 | `td { @if let Some(v) = &e.previous_value { (v.to_string()) } @else { "" } }` | `previous_value: Option<serde_json::Value>` | `td { @if let Some(v) = &e.previous_value { (scrub_json(v).to_string()) } @else { "" } }` |
| ~L322 | `td { (e.new_value.to_string()) }` | `new_value: serde_json::Value` | `td { (scrub_json(&e.new_value).to_string()) }` |
| ~L324-326 | `td { @if let Some(p) = &e.actor_pseudonym { (p) } @else { "" } }` | `actor_pseudonym: Option<String>` | `td { @if let Some(p) = &e.actor_pseudonym { (scrub(p)) } @else { "" } }` |

**Leave UNCHANGED** (NOT user-visible free-text PII vectors — CR cr-5
+ ADR-015 do not require these):

- `td { (e.id) }` — `i64`, numeric.
- `td { (e.created_at.format("%Y-%m-%d %H:%M:%S")) }` — timestamp.
- `td { @if e.signature.is_some() { "yes" } @else { "no" } }` —
  boolean display, no payload rendered.

**Already scrubbed by fix-impl-1 (#297) — DO NOT touch / DO NOT
double-scrub:**

- `td { (scrub(&e.reason)) }` (~L323) — leave exactly as-is.
- `td { @if let Some(d) = &e.denial_reason { (scrub(d)) } @else { "" } }`
  (~L331) — leave exactly as-is.

After this task, every user-visible string/value field in
`audit_entry_row` is passed through the canonical scrub/scrub_json —
ADR-015 ("every user-visible string passed through scrub()/redaction
before leaving the handler") fully satisfied for the audit page.

### 2.3 — actor_pseudonym scrub note (read before applying)

`actor_pseudonym` is the ADR-015 pseudonym (already a redacted
identity). Scrubbing it through `scrub()` is still correct here per
the CR finding + the "every user-visible string" rule (scrub is
idempotent on already-clean text — it only strips URLs/mentions/
emails; a bare pseudonym passes through unchanged). Apply the
`scrub(p)` per the table. If you observe that `scrub()` would
*mangle* a valid pseudonym (it should not — pseudonyms have no
URL/mention/email shape), STOP and file a `kind:"blocker"` DQ rather
than guessing — but this should not fire; apply as specified.

### Do NOT in this task

- Touch any file other than
  `crates/api/api/src/governance/admin_dashboard_html.rs`.
- Touch anything in that file OTHER than the `use` line and
  `audit_entry_row`'s 6 field renders listed in §2.2.
- Re-scrub `reason` / `denial_reason` (already done by #297).
- Touch the cr-4 handler-order code (already done + validated).
- Modify `e2e.rs`, the triage YAML, the plan, or
  `.claude/decision-queue.json` (except your own
  `validate-pending-laptop` entry per §5).
- Refactor, rename, or "tidy" adjacent code — minimal diff only
  (≈7 changed lines: 1 `use` widen + 6 field renders).

**Commit message** (exactly):
`fix(v1-AD-e): complete cr-5 ADR-015 scrub — scope/key/entry_kind/actor_pseudonym + scrub_json(value payloads) in audit_entry_row (PR #133)`

## §3 Required reading

- `crates/api/api/src/governance/admin_dashboard_html.rs` — read
  `audit_entry_row` (~L311-335) + the `use crate::governance::{...}`
  block + the EXISTING `scrub(&e.reason)` / `scrub(d)` lines (your
  pattern to mirror) + the `scrub` import.
- `crates/api/api/src/governance/admin_rule_sets.rs` ~L50-60 + L325-332
  — canonical `scrub` import-grouping + call shape.
- `crates/db_schema/src/source/governance/redaction.rs` ~L74-95 —
  `scrub` / `scrub_json` signatures (confirm `scrub_json(&Value)
  -> Value`; you `.to_string()` the result for rendering).
- `crates/api/api_common/src/governance.rs` L555-573 —
  `AdminConfigAuditEntry` definition (field types are authoritative;
  the §2.2 table is derived from these — re-confirm before editing).
- `.claude/PRPs/reviews/pr-133-findings.yaml` — cr-5 entry +
  its `notes:` recording the partial scope this task completes.
- `.claude/rules/circuit-breaker.md` + `.claude/rules/escalation.md` —
  `kind:"blocker"` fallback (the §2.3 pseudonym-mangle edge, which
  should not fire).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
  — ADR-015 (the binding constraint cr-5 cites; read the verbatim
  ADR-015 text so you understand WHY every field gets scrubbed).

## §4 Constraints

- **ONE commit, exactly 1 file**
  (`crates/api/api/src/governance/admin_dashboard_html.rs`). A 2nd
  file in the diff = scope breach → STOP + `kind:"blocker"` DQ.
- **Use ONLY `crate::governance::redaction::{scrub, scrub_json}`** —
  the canonical ADR-015 util, mirrored from the in-file `scrub(&e.reason)`
  pattern + `admin_rule_sets.rs`. Hand-rolling/inventing a scrub =
  hard refusal → `kind:"blocker"` DQ.
- **Exactly the 6 field renders in §2.2 + the 1 `use` widen.** Do NOT
  scrub `id`/`created_at`/`signature`. Do NOT re-touch
  `reason`/`denial_reason` (already scrubbed by #297). ≈7 changed
  lines total.
- `scrub` for `String`/`Option<String>` fields
  (entry_kind/scope/key/actor_pseudonym); `scrub_json(...).to_string()`
  for `serde_json::Value` fields (previous_value/new_value). Match
  field→fn per the §2.2 table (types verified at
  api_common/src/governance.rs:555-573).
- **Local pre-push cargo-check discipline** (per
  `feedback_fix_impl_pre_push_cargo_check`): before pushing, run
  `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-fiximpl2-check.log 2>&1"`
  — EXPECT exit 0. If it fails: fix the in-scope cause (likely a
  `scrub_json` arg/`.to_string()` shape mismatch — read the
  signature again); if out-of-scope cascade, file `kind:"blocker"`
  DQ + STOP. Do NOT push a broken tree; do NOT `#[allow]`-spam.
  Never bare `cargo` on Windows (libpq.dll — bat wrapper sets vcpkg
  PATH).
- File-ownership: impl-task — MAY write `crates/**` +
  `.claude/decision-queue.json` (your validate-pending-laptop entry
  only). Do NOT write `.claude/PRPs/reviews/**`,
  `.claude/PRPs/plans/**`, `docs/brehon-*/**`.

## §5 Validation gate — Shape-G SUSPENDED → validate-pending-laptop

**Shape G is suspended until 2026-06-01 (DQ #229).** After committing +
pushing your worktree branch, write a **`kind: "validate-pending-laptop"`**
DQ entry to `.claude/decision-queue.json` (commit + push on your
worktree branch immediately). Do NOT write `kind: "validate-pending"`.
Do NOT capture a `workflow_run_id`. The advisor-laptop runs ALL
commands (including the full e2e, Docker up) and mutates the entry.

**The `commands[]` array MUST contain, verbatim — FOUR commands**
(this is a handler-render change to a page the e2e exercises, so the
full e2e regression run is required — a scrub_json shape bug would
only surface at render/runtime):

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-fiximpl2-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-fiximpl2-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-fiximpl2-testnorun.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-AD-e-fiximpl2-e2e.log 2>&1"
```

First three EXPECT exit 0. The fourth (full e2e) EXPECT exit 0 with
ALL prior v1-AD-e tests still PASSING (the 4 admin_*_html tests +
the new cr-6 `admin_audit_html_forbidden_for_non_admin`) and
`test result: ok. … 0 failed` (scrub/scrub_json must not change
status codes or break the audit-page render — the audit
admin-200 test renders `audit_entry_row`, so a `scrub_json`
`.to_string()` shape error would fail it). advisor-laptop runs the
e2e with Docker up (~32 min). **Never bare `cargo test`**
(libpq.dll — bat wrapper sets vcpkg PATH per
`feedback_windows_e2e_requires_bat_wrapper`); **never
`-p lemmy_server --features full`** for the run (use
`--workspace --test e2e --features full`).

**`validate-pending-laptop` entry shape:**

```json
{
  "id": "<max(all ids across .claude/decision-queue.json + .claude/decision-queue-archive-*.json) + 1>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-AD-e fix-impl-2 (complete cr-5 ADR-015 scrub) pushed on <worktree-branch> — run DoD (incl full e2e) on laptop",
  "branch": "<your worktree branch name>",
  "phase_task": "fix-impl-2",
  "commands": ["<the four cmd //c lines above, verbatim>"],
  "context": "PR #133 cr-5 completion (user gate-5 chose complete-first). admin_dashboard_html.rs audit_entry_row: added scrub() to entry_kind/scope/key/actor_pseudonym + scrub_json() to previous_value/new_value (reason+denial_reason already scrubbed by #297 5d742a323). 1 file, ~7 lines. Canonical crate::governance::redaction::{scrub,scrub_json}. commit <sha>. Pre-Shape-G validate-pending-laptop (Shape G suspended until 2026-06-01, DQ #229). Full e2e required — handler-render change to the audit page the e2e exercises.",
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
both"). **Current max id is 245** (DQ #245 validate-pending-laptop
resolved=pass; no archives in worktree) → next is **246** unless an
archive holds higher; check before writing. Do NOT reuse an id.

If your local `cargo-check` sanity check fails (§4), do NOT push a
broken tree — fix the in-scope cause (re-read the `scrub_json`
signature; it returns `Value`, you must `.to_string()` it), or if
out-of-scope cascade, file a `kind: "blocker"` DQ instead of the
`validate-pending-laptop` entry and STOP.

## §6 Expected output (return to advisor)

Report concisely:

- Commit SHA + `git show --stat` (must be exactly 1 file:
  `admin_dashboard_html.rs`).
- The `git show` diff of `audit_entry_row` (the ≈7 changed lines —
  the `use` widen + 6 field renders), so the advisor can confirm
  `scrub` vs `scrub_json` is applied per the §2.2 table and
  reason/denial_reason are untouched.
- Confirmation `reason` (L323) + `denial_reason` (L331) were NOT
  re-touched (still single `scrub(...)`).
- Local pre-push `cargo-check` exit code.
- The `validate-pending-laptop` DQ id written (expect 246) + your
  worktree branch name.
- Any `kind:"blocker"` DQ raised (id + one-line reason — should be
  none).
