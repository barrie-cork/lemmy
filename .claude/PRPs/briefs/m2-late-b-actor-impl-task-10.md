---
role: impl-task
task_number: 10
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G
---

# impl-task brief — m2-late-b-actor Task 10: WIRE routes in `crates/api/routes/src/lib.rs`

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 10 of 13
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Task 9 merged (handler functions must exist before route wiring).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-10 route-wiring link-endpoints — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-10.md
```

---

## 2. Scope

**Produce:**
- 3 new route registrations in `crates/api/routes/src/lib.rs` under `scope("/governance")`
- 3 new handler imports in the `governance::{...}` use block
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Touch `services/bridge/**` (Tasks 11-12)
- Touch `crates/server/tests/e2e/**` (Task 13)
- Run cargo yourself

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 10"
2. `crates/api/routes/src/lib.rs` lines 31-55 — **MIRROR**: the `governance::{...}` import block; add the 3 new handler imports alphabetically
3. `crates/api/routes/src/lib.rs` lines 481-510 — **MIRROR**: the `scope("/governance")` wiring block; the 3 new routes go here
4. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

Read `crates/api/routes/src/lib.rs`. Two changes are needed.

**Change 1: Add 3 imports to the `governance::{...}` use block (around lines 31-55)**

The import block has entries like `room_event_handler::handle_room_event`. Add the 3 new handler imports alphabetically. `actor_app_link` sorts before `admin_*`, `bridge_*`, `decline_*`, etc. — insert near the top of the governance import list:

```rust
    actor_app_link::{link_actor, link_confirm, revoke_link},
```

Insert this line immediately after the opening `governance::{` line (it sorts first — `actor_` < `accept_` alphabetically is debatable; `actor` comes before `admin`, `bridge`, `decline` etc. Check the exact alphabetical position against the existing entries and insert at the right spot).

**Change 2: Add 3 routes under `scope("/governance")` (around lines 481-510)**

The governance scope currently starts with `.route("/room-event", ...)`. Add the 3 link routes. Per the plan:

```rust
  .route("/link", get().to(link_actor))
  .route("/link/confirm", post().to(link_confirm))
  .route("/link/revoke", post().to(revoke_link))
```

Place them logically — after `/room-event` and `/bridge/messaging-status` (the other non-JWT routes) or grouped with user-facing routes. A reasonable position is after `.route("/bridge/messaging-status", ...)` and before `.route("/report", ...)`.

**GOTCHA — `/link/confirm` is bearer-authed, not JWT:**
- `/link/confirm` uses `bridge_auth::verify_bridge_secret` inside the handler (not a middleware)
- It still sits under the governance scope with `wrap(rate_limit.post())` — that's correct (matches `handle_room_event` which also uses bearer auth via its own middleware)
- No special outer middleware needed for `/link/confirm` — the handler itself gates on bearer

**GOTCHA — import routing:**
- The handler fns `link_actor`, `link_confirm`, `revoke_link` are exported from `crates/api/api/src/governance/mod.rs` via `pub use actor_app_link::{link_actor, link_confirm, revoke_link};` (added by Task 9)
- Import path: `lemmy_api::governance::actor_app_link::{link_actor, link_confirm, revoke_link}` — but in this file the use block uses the short form `governance::{...}` (relative to `lemmy_api::`)
- Exact form: `actor_app_link::{link_actor, link_confirm, revoke_link},` inside the existing `governance::{...}` block

**GOTCHA — `get()` import:**
- `get()` from actix-web is already imported (used by `get_case`, `list_cases` etc.) — do NOT re-import

### validate-pending-laptop (MANDATORY)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["cargo check -p lemmy_routes"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 10
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(routes): wire /link, /link/confirm, /link/revoke routes under governance scope (task 10)
```

Stage only: `crates/api/routes/src/lib.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `actor_app_link::{link_actor, link_confirm, revoke_link}` import present in the governance use block
- [ ] `.route("/link", get().to(link_actor))` present under `scope("/governance")`
- [ ] `.route("/link/confirm", post().to(link_confirm))` present under `scope("/governance")`
- [ ] `.route("/link/revoke", post().to(revoke_link))` present under `scope("/governance")`
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-10
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/api/routes/src/lib.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "3 routes under scope('/governance'); /link/confirm bearer-gated in handler, not middleware"
    - "imports via actor_app_link::{...} in governance use block"
  notes: "Task 10 of 13. Tasks 11+12 (bridge, Linux-only) follow."
```
