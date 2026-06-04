---
phase: m1-b
role: impl-task
n: 6
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 6
minimax_trial: NOT eligible (single Sonnet arm — Task 6 is fire-and-forget HTTP wiring with no MIRROR-sibling shape)
---

# [role:impl-task] m1-b Task 6 — read-only bridge notify wiring

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-6 bridge-notify wiring — see .claude/PRPs/briefs/m1-b-impl-6.md
```

## 2. Scope

Add `bridge_notify::notify_if_enabled` — a fire-and-forget HTTP notification to the (future) Matrix bridge, gated on `messaging_enabled` — and call it ALONGSIDE the existing private-message `plugin_hook_notification`. **One file created, two files modified.** **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself** (cargo runs on the laptop, never on the daemon).

**Produce (exactly 1 file created, 2 files modified):**

```yaml
creates:
  - crates/api/api_utils/src/bridge_notify.rs   # notify_if_enabled (read messaging_enabled → fire-and-forget POST or no-op)
modifies:
  - crates/api/api_utils/src/lib.rs              # + pub mod bridge_notify;
  - crates/api/api_utils/src/notify.rs           # call notify_if_enabled(...) alongside plugin_hook_notification at :305
requires:
  - task: 2   # SATISFIED: GovernanceMessagingConfig::read_current(pool, scope, key) -> LemmyResult<Option<Self>> merged on phase-m1-b @ cb130a357 (validated). You call read_current(pool, "instance", "messaging_enabled") and check value_bool == Some(true).
```

**Do NOT:**
- Use `?` on the bridge POST, or let a bridge error propagate. **The PM-notification path is governance-critical** — a bridge being down MUST NOT break PM delivery. Use `.ok()` / log-and-swallow on the transport result, NEVER `?`. See §4.0 (the load-bearing constraint).
- Replace the existing `plugin_hook_notification(notifications, context).await?;` at `notify.rs:305`. Your call goes **alongside** it (additive), NOT instead of it. Do NOT add a new Extism plugin hook (ADR-012 — M1 adds NO new hooks; brief §4.e of the plan).
- Add a config cache, a connection pool for the bridge, or any per-send optimization. Reading config per-PM-send is acceptable at M1 scale (plan §10.5 GOTCHA — "if hot-path cost shows in M2, add a cache then").
- Block the PM path on the HTTP call in a way that adds latency to message delivery. Fire-and-forget: issue the POST, do not await a long timeout inline (use a short timeout + swallow, or spawn — mirror how the codebase does non-blocking outbound; if unsure, a short-timeout `.ok()` is acceptable for M1).
- Touch the messaging-config handler, the routes crate, the migration, the DTOs, the Diesel model, or `mod.rs` under `governance/`. Task 6 is ONLY the three files named.
- Run `cargo check` / `cargo clippy` / any cargo or docker command. Validation is the laptop's job (write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### 3a. Handover from prior cohort (Task 2 — the read path you call)

```yaml
prior_cohort_task:
  - task: 2
    finalize_merge: cb130a357    # GovernanceMessagingConfig model on phase-m1-b (Task 2 merged earlier; tip advanced through Tasks 3/4/5)
    validated: pass
    keyDecisions:
      - "GovernanceMessagingConfig::read_current(pool: &mut DbPool<'_>, scope: &str, key: &str) -> LemmyResult<Option<Self>> — THE read entry point. For messaging_enabled: read_current(pool, \"instance\", \"messaging_enabled\")."
      - "model fields include value_bool: Option<bool>. messaging_enabled is stored as value_type=\"bool\", value_bool=Some(true|false). To get the flag: match read_current(...)? { Some(row) => row.value_bool.unwrap_or(false), None => false }."
      - "the migration (Task 1) seeds messaging_enabled=false at scope=\"instance\" — so a fresh instance reads false (clean posture, no bridge POST)."
```

### MIRROR refs — read these EXACT locations on your base branch (phase-m1-b @ cb130a357)

`crates/api/api_utils/src/notify.rs` (the file you add ONE call to):
- **`:285` `pub fn notify_private_message(view: &PrivateMessageView, is_create: bool, context: &LemmyContext)`** — the public PM-notify entry.
- **`:290` `async fn notify_private_message_internal(view: &PrivateMessageView, ...)`** — the internal async fn.
- **`:305` `plugin_hook_notification(notifications, context).await?;`** — **THIS is the call site.** Add your `bridge_notify::notify_if_enabled(context, view).await.ok();` (or `.await.unwrap_or_default();` — log-and-swallow, never `?`) IMMEDIATELY ALONGSIDE this line (right after it, inside `notify_private_message_internal`). The `view: &PrivateMessageView` is in scope there — pass it.
- **`:19` `use lemmy_db_views_private_message::PrivateMessageView;`** — the type your `notify_if_enabled` second param uses (already imported in notify.rs; you'll import it in bridge_notify.rs).

`crates/api/api_utils/src/plugins.rs`:
- **`:48` `pub async fn plugin_hook_notification(notifications: Vec<Notification>, context: &LemmyContext) -> LemmyResult<()>`** — mirror the **fn shape** for your `notify_if_enabled` (async, takes `&LemmyContext`, returns `LemmyResult<()>`). Your fn signature per §10.5: `pub async fn notify_if_enabled(context: &LemmyContext, view: &PrivateMessageView) -> LemmyResult<()>`.

`crates/api/api_utils/src/lib.rs`:
- **`:1-8`** — the `pub mod` block (`build_response`, `claims`, `context`, `notify`, `plugins`, `request`, `send_activity`, `utils`). Add `pub mod bridge_notify;` in alphabetical position (it sorts FIRST, before `build_response`). Match the existing ordering style.

`crates/api/api_utils/src/context.rs` + `crates/api/api_utils/src/request.rs` (the HTTP client):
- **`context.rs:15`** `client: Arc<ClientWithMiddleware>` — `LemmyContext` holds a `reqwest_middleware` client. Find the accessor `LemmyContext` exposes for it (likely `context.client()` — read context.rs for the public getter). Use that client for the fire-and-forget POST. If there is no public getter, mirror how other api_utils code issues an outbound HTTP request (read `request.rs` / `send_activity` for the pattern). Do NOT construct a brand-new client if the context already exposes one.

### Plan sections
- `.claude/PRPs/plans/m1.plan.md` §13 Task 6 (ACTION / IMPLEMENT file 1-3 / MIRROR / GOTCHA / VALIDATE) — authoritative.
- `.claude/PRPs/plans/m1.plan.md` §10.5 (the `notify_if_enabled` reference implementation — read the GOTCHA box: fire-and-forget, `.ok()`/log-and-swallow, NEVER `?`, never block PM delivery).
- `.claude/PRPs/plans/m1.plan.md` §12 (NOT building in m1 — no config cache, no new hook, no M2 backplane).
- ADR-012 in `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — Extism hooks are M1's only seam; this task adds NO new hook, it rides the existing PM-notify path.

### Lessons (mandatory)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error construction: `LemmyResult<T>`; the fn RETURNS `LemmyResult<()>` (so the read can use `?`), but the **bridge POST result is swallowed** (`.ok()`), never propagated. The only `?` allowed is on the `read_current` DB read; the HTTP transport error is logged + swallowed.
- `.claude/lessons/feedback_clippy_test_style.md` — workspace clippy denies `unwrap`/`expect`/`allow_attributes` in non-test code. Use `.unwrap_or(false)` / `.ok()` / `if let` — never bare `.unwrap()` / `.expect()`. (`row.value_bool.unwrap_or(false)` is fine — that's `unwrap_or`, not `unwrap`.)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — **does NOT apply** (Task 6 does ZERO DB writes — one read + one fire-and-forget POST). Listed only so you don't reach for a transaction.
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — the laptop DoD runs `--features full --workspace` (never `-p <crate> --features full`). Gate DB/HTTP-touching code `#[cfg(feature = "full")]` if the surrounding code is so gated — mirror notify.rs's gating.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop. Do NOT run cargo.

## 4. Constraints

### 4.0 FIRE-AND-FORGET — NEVER fail the PM path (the load-bearing constraint)

Plan §10.5 GOTCHA is authoritative:

> The bridge POST is fire-and-forget — a bridge being down must NOT break PM delivery (the PM path is governance-critical). Use `.ok()` / log-and-swallow, never `?`. Reading config per-PM-send is acceptable at M1 scale.

Your `notify_if_enabled` flow:
1. Read `messaging_enabled` via `read_current(pool, "instance", "messaging_enabled")`. If `Err` (DB error) you MAY `?` this (a DB failure is already a PM-path failure). If `Ok(None)` or `Ok(Some(row))` with `value_bool != Some(true)` → **return `Ok(())` immediately (no-op, clean posture)**.
2. If enabled: issue a fire-and-forget HTTP POST to the bridge. **Swallow the transport result** — `let _ = context.client().post(url).json(&payload).send().await;` or `.send().await.ok();` with a `tracing::warn!` / `tracing::debug!` log on error. NEVER `?` the POST. NEVER let a bridge-down error escape this fn.
3. Return `Ok(())`.

The call site (`notify.rs:305`) uses `.await.ok();` (or `.unwrap_or_default()`) so even if `notify_if_enabled` itself returned an `Err`, the PM path continues. Belt-and-braces: the fn swallows internally AND the call site swallows.

### 4.1 notify_if_enabled signature + body (per §10.5)

```rust
use crate::context::LemmyContext;
use lemmy_db_views_private_message::PrivateMessageView;
use lemmy_db_schema::source::governance::governance_messaging_config::GovernanceMessagingConfig;  // confirm exact path by reading where the handler imports it
use lemmy_utils::error::LemmyResult;

/// Fire-and-forget notify to the Matrix bridge. Reads `messaging_enabled`;
/// false → no-op (clean v0 governance-only posture). True → POST to the bridge,
/// log+swallow any transport error. NEVER fails the PM path. ADR-012 (no new hook).
pub async fn notify_if_enabled(context: &LemmyContext, view: &PrivateMessageView) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let enabled = match GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled").await? {
    Some(row) => row.value_bool.unwrap_or(false),
    None => false,
  };
  if !enabled {
    return Ok(()); // clean posture — no bridge surface
  }
  // fire-and-forget POST to the bridge; swallow transport errors (NEVER ?)
  // build the payload from `view` (the PM that was just sent) — minimal: ids/actor, not body content unless §10.5 says otherwise
  // let _ = context.client().post(<bridge_url>).json(&payload).send().await;  // log on error via tracing
  Ok(())
}
```

> **Bridge URL / port:** the plan §10.5 stub uses `localhost:<bridge_port>`. M1 has no configured bridge port yet (the bridge crate is Tree A, not built here). Use a sensible M1 default — a localhost URL with a reasonable default port (read the plan / chat-plane design doc for the bridge's intended port; if a constant or env var is the obvious source, use it). **If the bridge URL/port source is genuinely ambiguous from the plan + design docs, raise a `kind: "blocker"` DQ** (§4.5) rather than inventing a magic number — but a documented default const (e.g. `const BRIDGE_NOTIFY_URL: &str = "http://localhost:<port>/brehon/notify";`) with a code comment citing §10.5 is acceptable for M1. The POST never has to succeed in M1 (no bridge running), so the URL just needs to be well-formed.

### 4.2 The call site (notify.rs:305)

Inside `notify_private_message_internal` (notify.rs:290), immediately after the existing `plugin_hook_notification(notifications, context).await?;` at line 305, add:

```rust
  crate::bridge_notify::notify_if_enabled(context, view).await.ok();  // fire-and-forget; never fails PM path (§10.5)
```

`view: &PrivateMessageView` is the fn's first param — in scope. Do NOT change the existing `plugin_hook_notification` line. Do NOT add a new hook.

### 4.3 lib.rs

Add `pub mod bridge_notify;` to `crates/api/api_utils/src/lib.rs`, alphabetically first (before `pub mod build_response;`), matching the existing ordering.

### 4.4 If something is genuinely ambiguous — raise a blocker, do not guess

If the HTTP-client accessor on `LemmyContext`, the bridge URL/port source, or the POST payload shape is not derivable from the MIRROR refs + plan §10.5 + the chat-plane design doc, raise a `kind: "blocker"` DQ entry (`from: "impl"`, `answered_by: null`), commit + push it immediately (mid-task visibility), and STOP that thread. Do NOT invent a novel client or a magic config shape. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`. (For the bridge URL specifically: a documented default const citing §10.5 is acceptable — that is NOT a blocker. Blocker only if the client accessor or payload shape is unclear.)

### 4.5 validate-pending-laptop DQ entry

After editing all three files and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 6,
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full",
    "./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e"
  ],
  "question": "Workspace check + e2e test-target compile for bridge_notify wiring — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 6 created crates/api/api_utils/src/bridge_notify.rs (notify_if_enabled: read messaging_enabled via read_current, false→no-op, true→fire-and-forget POST swallowing transport errors), added pub mod bridge_notify to lib.rs, and called notify_if_enabled alongside plugin_hook_notification at notify.rs:305 (additive, no new hook). Fire-and-forget — NEVER fails the PM path (.ok()/log-and-swallow, never ?). cargo check --workspace --features full required; e2e --no-run confirms it compiles into the test target. Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 4.6 Commit + push discipline
- **Commit subject:** `feat(api): add fire-and-forget bridge notify on PM send (task 6)`
- Sequence:
  ```
  git add crates/api/api_utils/src/bridge_notify.rs crates/api/api_utils/src/lib.rs crates/api/api_utils/src/notify.rs .claude/decision-queue.json
  git commit -m "feat(api): add fire-and-forget bridge notify on PM send (task 6)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-b task 6`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.7 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- fire-and-forget error handling → `feedback_lemmy_error_no_std_error.md` ✓ + `feedback_clippy_test_style.md` ✓ (no unwrap/expect; `.ok()`/`unwrap_or` swallow; `?` ONLY on the DB read)
- `#[cfg(feature = "full")]` gating → `feedback_features_full_workspace_only.md` ✓ + `feedback_features_full_p_crate_incompatible.md` ✓ (DoD uses `--workspace`)
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓
- single-read-no-write → `feedback_multi_write_handlers_need_transactions.md` cited as non-applicable (zero DB writes) ✓

> No `crates/server/tests/e2e.rs` edit in this task (e2e tests are Task 7) → e2e-anchor lessons do NOT fire. No migration, no multi-write → those lessons do NOT fire.

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-b-task-6
  filesCreated:
    - crates/api/api_utils/src/bridge_notify.rs
  filesModified:
    - crates/api/api_utils/src/lib.rs
    - crates/api/api_utils/src/notify.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "notify_if_enabled(context, view): reads messaging_enabled via read_current(pool, \"instance\", \"messaging_enabled\"); false/None → no-op (clean posture); true → fire-and-forget POST, transport errors swallowed (.ok()/log), NEVER ?"
    - "called alongside plugin_hook_notification at notify.rs:305 via crate::bridge_notify::notify_if_enabled(context, view).await.ok(); — additive, NO new hook (ADR-012)"
    - "bridge URL: <const/env you used> per §10.5 (M1 has no running bridge; POST never has to succeed)"
    - "pub mod bridge_notify added to lib.rs (alphabetically first)"
  notes: "validation = cargo check --workspace --features full + e2e --no-run on laptop; worker wrote validate-pending-laptop DQ + stopped. Task 7 (e2e clean-posture test) asserts bridge_notify no-ops when messaging_enabled=false."
```
