---
phase: v1-AD-e
role: impl-task
task: fix-impl-1
brief_n: 1
authored: 2026-05-17
plan: .claude/PRPs/plans/v1-admin-dashboard-e.plan.md
related_dq: pr-133 CR findings cr-1, cr-4, cr-5, cr-6 (triage YAML .claude/PRPs/reviews/pr-133-findings.yaml, bucket=fix-in-pr; user gate-3 approved 2026-05-17)
canonical_sibling: ".claude/PRPs/briefs/v1-AD-e-impl-5.md (house-style + §5 validate-pending-laptop 4-command shape); cr-6 e2e mirror: crates/server/tests/e2e.rs admin_dashboard_html_forbidden_for_non_admin (e2e.rs:14914, sibling in the v1-AD-e fixtures module); cr-5 scrub mirror: crates/api/api/src/governance/admin_rule_sets.rs:55 (use crate::governance::redaction::scrub) + :329 (rule_text: scrub(&rsv.rule_text))"
mandatory_lessons_fired:
  - feedback_lemmy_error_no_std_error.md   # e2e.rs edit (cr-6) — Case A enumeration (mirror v1-AD-e fixtures sibling LemmyResult<()>)
  - feedback_async_pool_test_pattern.md    # e2e.rs edit (cr-6) — AsyncPgConnection/DbPool seed pattern
  # feedback_junior_worker_e2e_edit_hang.md referenced by advisor-orchestrator §2.4 but ABSENT from .claude/lessons/ — principle folded into §4 inline
---

# [role:impl-task] v1-AD-e fix-impl-1 — CR fix-in-PR for PR #133 (cr-4 auth-order / cr-5 ADR-015 scrub / cr-6 symmetric test / cr-1 DQ chronology) — see .claude/PRPs/briefs/v1-AD-e-fix-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-AD-e fix-impl-1 — PR #133 CR fixes: cr-4 flag-before-auth (404) / cr-5 scrub audit strings (ADR-015) / cr-6 non-admin /audit/view test / cr-1 DQ timestamp chronology`

You are the **impl-task** subagent (Sonnet 4.6). This is a **fix-in-PR**
cycle for PR #133 (`phase-v1-AD-e` → `governance-v0`). CodeRabbit posted
6 findings; the user approved the four-bucket triage at gate 3. **Four**
findings are bucketed `fix-in-pr` — you implement all four. (The other
two — cr-2 rebut, cr-3 wont-fix — need NO code; do not touch them.)

The phase branch tip already has the triage YAML
(`.claude/PRPs/reviews/pr-133-findings.yaml`). Your worktree branches
from `phase-v1-AD-e` (base_branch set by the advisor).

## §2 Scope — four fixes, ONE commit

Produce **ONE commit** touching exactly these files:

- `crates/api/api/src/governance/admin_dashboard_html.rs` (cr-4 + cr-5)
- `crates/server/tests/e2e.rs` (cr-6 — append-only)
- `.claude/decision-queue.json` (cr-1 — mechanical timestamp fix only)

### 2.1 — cr-4 (major): check feature flag BEFORE admin authorisation

**File:** `crates/api/api/src/governance/admin_dashboard_html.rs`

Both handlers currently call `is_admin(&local_user_view)?;` as the
**first statement**, BEFORE the `html_pages_enabled` flag check. So a
non-admin hitting a flag-disabled page gets **403** (leaking "you are
not an admin") instead of **404** (page does not exist). Plan R-html-3
requires flag-off → **404, not 403** consistently.

**Fix (exactly CR's suggested diff — minimal, both handlers):** move
`is_admin(&local_user_view)?;` to *after* the
`if !enabled { return Ok(HttpResponse::NotFound().finish()); }` block,
in **both** `admin_dashboard_html` (~L71) **and** `admin_audit_html`
(~L245). Resulting order in each handler:

```rust
  let mut cache = ConfigCache::new();
  let mut pool = context.pool();
  let enabled = get_bool(&mut cache, &mut pool, Scope::Instance, HTML_PAGES_KEY).await?;
  if !enabled {
    return Ok(HttpResponse::NotFound().finish());
  }
  is_admin(&local_user_view)?;          // ← moved to here (was the first stmt)
  let conn = &mut get_conn(&mut pool).await?;
  …
```

Do NOT change anything else in these handlers. Do NOT alter the
`get_bool` / `HTML_PAGES_KEY` / `Scope::Instance` call.

### 2.2 — cr-5 (major, ADR-015-backed): scrub user-visible audit strings

**File:** `crates/api/api/src/governance/admin_dashboard_html.rs`,
function `audit_entry_row` (~L310–353).

`audit_entry_row` renders user-visible fields **without scrub/redaction**.
ADR-015 coding guideline (CR-cited): *"Every user-visible string in a
response or log payload is passed through scrub()/redaction before
leaving the handler."*

**Use the CANONICAL existing utility — do NOT invent one.** The codebase
already has it:

- `use crate::governance::redaction::scrub;`
  — signature `pub fn scrub(text: &str) -> String`
  — already used by `admin_rule_sets.rs:55` (import) + `:329`
    (`rule_text: scrub(&rsv.rule_text)`). Mirror that usage exactly.
- For JSON `Value` payloads there is a sibling
  `crate::governance::redaction::scrub_json` (`pub fn scrub_json(value:
  &Value) -> Value`). Use it for `previous_value` / `new_value` (those
  are `serde_json::Value`, not `&str`).

Apply scrub to every user-visible string field rendered in
`audit_entry_row`:

- `e.scope`, `e.key`, `e.reason`, `e.entry_kind` → `scrub(&e.<field>)`
  (if the field is already `&str`/`String`; if it is an enum/Display,
  format to string then `scrub(&s)` — read the actual field types in
  `AdminConfigAuditEntry` first; do not assume).
- `e.actor_pseudonym` (`Option<…>`), `e.denial_reason` (`Option<…>`) →
  scrub the inner value inside the `@if let Some(..)` arm.
- `e.previous_value` (`Option<Value>`), `e.new_value` (`Value`) →
  `scrub_json(v)` then `.to_string()` (mirror the existing
  `(v.to_string())` / `(e.new_value.to_string())` shape but feed it the
  scrubbed Value).
- Do NOT scrub `e.id`, `e.created_at` (timestamp), or the
  `e.signature.is_some()` boolean — those are not free-text PII vectors
  and CR's list does not include them.

Add the `use crate::governance::redaction::{scrub, scrub_json};` import
alongside the existing `use crate::governance::{…}` block (mirror
`admin_rule_sets.rs:52-57` grouping).

**Read `AdminConfigAuditEntry`'s actual field types FIRST** (it is in
`lemmy_api_common::governance` — the file already imports it) so you
call `scrub` (for `&str`/`String`) vs `scrub_json` (for `Value`) on the
right fields. **If a user-visible field has a type with NO applicable
canonical scrub (e.g. a numeric/enum that ADR-015 would still require
redacting and neither `scrub` nor `scrub_json` fits): STOP, file a
`kind: "blocker"` DQ** (`from: "impl"`) describing the field + type +
why no canonical scrub applies — that is a real ADR gap for user
escalation, **not** something to guess or hand-roll. Do not invent a
new scrub helper.

### 2.3 — cr-6 (major): non-admin rejection test for /audit/view

**File:** `crates/server/tests/e2e.rs` — **append-only, ONE new test fn.**

The v1-AD-e fixtures module has `admin_dashboard_html_forbidden_for_non_admin`
(e2e.rs:14914) for the dashboard route but **no symmetric non-admin test
for `/audit/view`**. Add `admin_audit_html_forbidden_for_non_admin`
mirroring the dashboard one **verbatim in structure**, changing only:

- the route/handler under test → the audit one
  (`governance::admin_dashboard_html::admin_audit_html`, the same
  function the existing `admin_audit_html_returns_html_for_admin`
  test at e2e.rs:14993 calls — read it to copy the exact invocation
  shape: direct-handler call with the non-admin `LocalUserView`).
- the test fn name → `admin_audit_html_forbidden_for_non_admin`.
- assert the **same** denial status the dashboard non-admin test
  asserts (read `admin_dashboard_html_forbidden_for_non_admin` at
  e2e.rs:14914 and mirror its expected status exactly — do NOT
  hard-code a guess; whatever `is_admin?` rejection maps to there is
  what this asserts here).

**Mirror the existing v1-AD-e fixtures module's error-shape discipline
EXACTLY** (Case A `LemmyResult<()>` outer + helpers — per
`feedback_lemmy_error_no_std_error.md`; the sibling tests
`admin_dashboard_html_forbidden_for_non_admin` /
`admin_audit_html_returns_html_for_admin` are the canonical shape — read
them and match the signature + `?`-propagation + seed-helper calls
verbatim; do NOT introduce `.map_err` bridges or a different Result
type). Use the same `admin_config_fixtures::bootstrap` /
`bootstrap_instance` / `seed_user` (non-admin) helpers the sibling
tests use (per `feedback_async_pool_test_pattern.md`).

Append the new fn **inside the existing v1-AD-e fixtures module**
adjacent to its siblings (it shares their `use`s + helpers — do NOT
create a new module; do NOT splice elsewhere). This is a **single
small append** (one fn) — low e2e-edit-hang risk, but still: ONE
function, ONE contiguous edit, no other e2e.rs change in this commit.

### 2.4 — cr-1 (low): fix DQ timestamp chronology

**File:** `.claude/decision-queue.json` (the phase-branch copy in your
worktree).

CR flagged that the newly-added DQ entries (#237/#238 region, ~lines
3962/3998) have a timestamp chronology issue (a later entry timestamped
before an earlier one, or non-monotonic `timestamp`/`resolved_at`).

**Mechanical fix ONLY:** correct the `timestamp` / `resolved_at` values
so they are chronologically monotonic for the #237/#238 entries
**without changing any other field**. Do **NOT** touch `question`,
`answer`, `answered_by`, `options`, `from`, `kind`, `id`, or `context`
on these entries — those are immutable audit content. This is purely an
ordering correction on the timestamp fields. If the "correct"
chronology is ambiguous (you cannot tell from surrounding entries what
the right order is), leave them as-is and note it in §6 output — do NOT
guess at semantic intent. Per `.claude/rules/decision-queue.md`
forward-only rule, do not rewrite any OTHER historical entry.

Write the JSON back with `json.dump(..., ensure_ascii=False, indent=2)`
(preserve non-ASCII; per `feedback_json_dump_ensure_ascii_false`).

### Do NOT in this task

- Touch any file other than the three named above.
- Address cr-2 (rebut — no code) or cr-3 (wont-fix — no code).
- Modify the triage YAML `.claude/PRPs/reviews/pr-133-findings.yaml`
  (the advisor updates `addressed_in` after your commit lands).
- Bundle anything beyond the four findings above.
- Refactor adjacent code, rename, or "clean up" — minimal diff only.
- Post any PR comment or touch `.github/`.

**Commit message** (exactly):
`fix(v1-AD-e): PR #133 CR fix-in-PR — flag-before-auth 404 (cr-4) + scrub audit strings ADR-015 (cr-5) + non-admin /audit/view test (cr-6) + DQ timestamp chronology (cr-1)`

## §3 Required reading

- `.claude/PRPs/plans/v1-admin-dashboard-e.plan.md` — §13 (R-html-3 =
  flag-off → 404 not 403; ADR-015 scrub requirement) + §18 GOTCHAs.
- `.claude/PRPs/reviews/pr-133-findings.yaml` — the triage (cr-1/4/5/6
  = fix-in-pr; cr-2 rebut, cr-3 wont-fix — do not touch the YAML).
- `crates/api/api/src/governance/admin_rule_sets.rs` lines 50–60 +
  325–332 — **canonical `scrub` usage to mirror for cr-5** (import
  grouping + call shape).
- `crates/db_schema/src/source/governance/redaction.rs` ~L74–95 —
  `scrub` / `scrub_json` signatures (read so you call the right one
  per field type).
- `crates/server/tests/e2e.rs` — read these three v1-AD-e fixtures
  tests as the cr-6 mirror + Case-A discipline source:
  `admin_dashboard_html_forbidden_for_non_admin` (~14914, the
  structural mirror), `admin_audit_html_returns_html_for_admin`
  (~14993, the exact audit-handler invocation shape),
  `admin_dashboard_html_returns_html_for_admin` (~14881, helper-call
  pattern). Match their `LemmyResult<()>` outer + `?` propagation
  verbatim.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **MANDATORY**
  (e2e.rs edit). Case A = `LemmyResult<()>` outer + helpers; the
  v1-AD-e fixtures siblings already use Case A — mirror verbatim, no
  `.map_err` bridge.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — **MANDATORY**
  (e2e.rs edit). Seed via the existing `admin_config_fixtures` helpers,
  not raw INSERT.
- `.claude/lessons/feedback_json_dump_ensure_ascii_false.md` — DQ JSON
  write discipline (cr-1).
- `.claude/rules/decision-queue.md` — forward-only / "do not rewrite
  historical entries" (bounds cr-1: timestamps only, no other field).
- `.claude/rules/circuit-breaker.md` + `.claude/rules/escalation.md` —
  the `kind:"blocker"` DQ fallback for the cr-5 "no applicable
  canonical scrub" case (escalate, do not guess).

## §4 Constraints

- **ONE commit, exactly 3 files** (`admin_dashboard_html.rs`, `e2e.rs`,
  `.claude/decision-queue.json`). A 4th file in the diff = scope breach
  → STOP + `kind:"blocker"` DQ.
- **cr-4:** minimal move of one statement in each of the two handlers.
  No other handler change. Preserve exact existing flag-check code.
- **cr-5:** use ONLY `crate::governance::redaction::{scrub, scrub_json}`
  (the canonical ADR-015 util, mirrored from `admin_rule_sets.rs`).
  Inventing/hand-rolling a scrub = hard refusal → `kind:"blocker"` DQ.
  Read `AdminConfigAuditEntry` field types before choosing scrub vs
  scrub_json. No-applicable-scrub for a required field → STOP +
  `kind:"blocker"` DQ (real ADR gap, user-escalation — NOT a guess).
- **cr-6:** ONE new test fn, appended INSIDE the existing v1-AD-e
  fixtures module, mirroring `admin_dashboard_html_forbidden_for_non_admin`
  verbatim in structure + the exact denial status it asserts. Case A
  `LemmyResult<()>` discipline mirrored verbatim (no `.map_err`, no
  alternate Result type — `feedback_lemmy_error_no_std_error.md`).
  ONE contiguous append; never bundle other e2e.rs edits (e2e.rs is
  >15 000 lines — the e2e-edit-hang principle: single small append,
  its own scope).
- **cr-1:** timestamp/`resolved_at` ordering ONLY on the #237/#238
  entries. Zero change to any other DQ field or any other entry.
  Ambiguous correct-order → leave as-is + note in §6 (no guessing).
- **Local pre-push compile sanity:** before pushing, run
  `cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-fiximpl1-testnorun.log 2>&1"`
  and `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-fiximpl1-check.log 2>&1"`.
  Both EXPECT exit 0. If either fails: fix the in-scope cause (mirror
  the canonical shapes exactly); if the failure is an out-of-scope
  cascade, file a `kind:"blocker"` DQ and STOP — do NOT push a broken
  tree, do NOT `#[allow]`-spam to bypass. (Per
  `feedback_fix_impl_pre_push_cargo_check`: local check ~30s warm vs a
  full wasted laptop-validate cycle.) Never bare `cargo` on Windows
  (libpq.dll — bat wrapper sets vcpkg PATH).
- File-ownership: you are impl-task — you MAY write `crates/**`,
  `tests/**`, `.claude/decision-queue.json`. Do NOT write
  `.claude/PRPs/reviews/**`, `.claude/PRPs/plans/**`, `docs/brehon-*/**`.

## §5 Validation gate — Shape-G SUSPENDED → validate-pending-laptop

**Shape G is suspended until 2026-06-01 (DQ #229).** After committing +
pushing your worktree branch, write a **`kind: "validate-pending-laptop"`**
DQ entry to `.claude/decision-queue.json` (commit + push on your
worktree branch immediately — the advisor-laptop reads it off your
branch). Do NOT write `kind: "validate-pending"`. Do NOT capture a
`workflow_run_id`. The advisor-laptop runs ALL commands (including the
full e2e, Docker up) and mutates the entry.

**The `commands[]` array MUST contain, verbatim — FOUR commands** (this
touches both a handler and e2e.rs, so the full e2e run is required to
prove cr-4's 404-semantics + cr-6's new test + no regression):

```
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-AD-e-fiximpl1-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-AD-e-fiximpl1-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --no-run -p lemmy_server --test e2e > .claude/PRPs/debug/v1-AD-e-fiximpl1-testnorun.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-AD-e-fiximpl1-e2e.log 2>&1"
```

First three EXPECT exit 0. The fourth (full e2e) EXPECT exit 0 with the
new `admin_audit_html_forbidden_for_non_admin` test PASSING and all
prior v1-AD-e tests still PASSING (cr-4 changes handler behaviour —
the existing `admin_html_pages_flag_off_returns_404` /
`admin_dashboard_html_forbidden_for_non_admin` /
`admin_audit_html_returns_html_for_admin` MUST still pass; if cr-4's
reorder breaks the existing admin-200 path, that's an in-scope
regression to fix in the same commit). Advisor-laptop runs the e2e
with Docker up (~32 min). **Never bare `cargo test`** (libpq.dll — bat
wrapper sets vcpkg PATH per `feedback_windows_e2e_requires_bat_wrapper`);
**never `-p lemmy_server --features full`** for the run (use
`--workspace --test e2e --features full`).

**`validate-pending-laptop` entry shape:**

```json
{
  "id": "<max(all ids across .claude/decision-queue.json + .claude/decision-queue-archive-*.json) + 1>",
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO 8601 UTC>",
  "question": "v1-AD-e fix-impl-1 (PR #133 CR fix-in-PR: cr-4/cr-5/cr-6/cr-1) pushed on <worktree-branch> — run DoD (incl full e2e) on laptop",
  "branch": "<your worktree branch name>",
  "phase_task": "fix-impl-1",
  "commands": ["<the four cmd //c lines above, verbatim>"],
  "context": "PR #133 CR fix-in-PR (user gate-3 approved). admin_dashboard_html.rs: cr-4 move is_admin after flag-check in both handlers + cr-5 scrub/scrub_json audit_entry_row fields (ADR-015, canonical crate::governance::redaction). e2e.rs: cr-6 append admin_audit_html_forbidden_for_non_admin mirroring the dashboard non-admin sibling. decision-queue.json: cr-1 #237/#238 timestamp chronology only. commit <sha>. Pre-Shape-G validate-pending-laptop (Shape G suspended until 2026-06-01, DQ #229). Needs FULL e2e (Docker) — handler behaviour changed.",
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
both"). **Current max id is 244** (phase-branch DQ; pending=0; no
archives in worktree) → next is **245** unless an archive holds higher;
check before writing. Do NOT reuse an id.

If your local `--no-run` / `cargo-check` sanity check fails (§4), do
NOT push a broken tree — fix the in-scope cause (mirror the canonical
sibling shapes exactly), or if it is an out-of-scope cascade, file a
`kind: "blocker"` DQ instead of the `validate-pending-laptop` entry
and STOP.

## §6 Expected output (return to advisor)

Report concisely:

- Commit SHA + the `git show --stat` (must be exactly 3 files:
  `admin_dashboard_html.rs`, `e2e.rs`, `.claude/decision-queue.json`).
- **cr-4:** confirmation `is_admin?` now follows the flag-check in
  BOTH handlers (quote the 2 reordered regions, ~3 lines each).
- **cr-5:** which fields you scrubbed and with which fn (`scrub` vs
  `scrub_json`), the import line added, and the `AdminConfigAuditEntry`
  field types you observed. If you filed a `kind:"blocker"` for a
  no-applicable-scrub field, say so + the DQ id.
- **cr-6:** the new test fn name, the denial status it asserts (and
  which sibling you mirrored it from), confirmation it is inside the
  existing v1-AD-e fixtures module with Case-A `LemmyResult<()>`.
- **cr-1:** the timestamp values before→after on #237/#238 (or
  "left as-is, ambiguous" + why).
- Local pre-push sanity: `cargo-check` + `--no-run` exit codes.
- The `validate-pending-laptop` DQ id written (expect 245) + your
  worktree branch name.
- Any `kind:"blocker"` DQ raised (id + one-line reason).
