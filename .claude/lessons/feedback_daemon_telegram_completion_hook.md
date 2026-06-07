---
name: Daemon Telegram completion hook — check + recreate at session start
description: The Junior daemon's Telegram completion-ping hook is daemon state, not repo state; a restart can wipe it or leave it broken (active but last-triggered never, with a last_error). Check via list_hooks at every session start and recreate if missing or carrying a last_error.
type: feedback
---

# Daemon Telegram completion hook — check + recreate at session start

**Why:** The Junior daemon fires a Telegram ping (✅ on `done`, ❌ on
`failed`) when a task reaches a terminal state, so the advisor gets
out-of-band completion notice without polling. The hook is **daemon
state, not repo state** — it lives in the daemon's hooks store, not in
git. A daemon restart (or version drift across a restart) can wipe the
hook or leave it in a broken `BuildMessage: ModuleNotFound` state where
it shows `active` but has `last triggered: never` and carries a
`last_error`. When that happens the advisor silently loses completion
notifications.

**How to apply:** At every advisor session start, call
`mcp__junior-brehon__list_hooks`. Decide by state:

| Observed state | Action |
|---|---|
| A hook exists, `active`, no `last_error` | OK — leave it. |
| **No** completion-notify hook present | Recreate it (daemon restart wiped it). |
| Hook present but carries a `last_error` (e.g. `BuildMessage: ModuleNotFound`) | **Remove the broken one, then recreate** — a stale check_fn references a module the current daemon runtime can't resolve. Recreating regenerates the check_fn against the live runtime. |

Recreate via `mcp__junior-brehon__create_hook` with this
natural-language description (the daemon compiles it into a check_fn +
prompt; do NOT hand-author the check_fn):

> Whenever a Junior task transitions to a terminal state, send a
> Telegram notification: if the task completed successfully (status
> "done"), send a message starting with ✅ that names the task id and its
> title; if the task ended in "failed", send a message starting with ❌
> that names the task id, its title, and the failure reason. Only fire
> once per task per terminal transition.

The generated hook must respect the Telegram scope guardrail
(`.claude/rules/branch-manager.md` "Telegram scope" +
`feedback_telegram_scope_notification_only.md`): no diff content, no
secrets, no file dumps; one message per task, no batching; MCP
disconnected → silent skip + runlog note. The `create_hook` extraction
typically folds these in automatically when the description mentions
them, but verify the generated `Prompt` body includes the
silent-skip-on-disconnect clause.

**Hook ID is not stable across recreation.** The historical hook was
"ID 1"; after a remove+recreate cycle it comes back with a fresh
auto-increment id (e.g. ID 2). Cite the hook by **name/purpose**
("the Junior task-terminal completion notifier"), not by a hardcoded
numeric id — older rule/handover text that says "hook ID 1 must exist"
means "the completion-notify hook must exist," not literally id==1.

**Topology caveat (2026-06-04):** the MCP `list_hooks` is the source of
truth for hook state — it reads the live daemon the laptop MCP transport
connects to. Do NOT cross-check against an on-disk `junior.db` `hooks`
table to confirm presence: the SSH-reachable `/home/barrie/.junior/junior.db`
showed `0 hooks` while `list_hooks` correctly returned the live hook,
because the MCP-connected daemon instance and the SSH-queried DB are not
the same store (multiple `junior daemon` processes were running). Trust
`list_hooks`, not the file.

**Non-gating:** completion pings are notifications, not gating signals.
A broken or missing hook never blocks work — it only costs you the
out-of-band ✅/❌. Recreate it for convenience; never stall a phase on it.

## See also

- `.claude/rules/advisor-orchestrator.md` §1 "Telegram completion hook check"
- `feedback_telegram_scope_notification_only.md` — the five allowed ping
  shapes + never-send list this hook must honour
- `reference_brehon_consensus_telegram_topic.md` — the topic the ping
  lands in (ID 1283)
