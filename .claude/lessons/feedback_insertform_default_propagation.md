---
name: Adding fields to an InsertForm with derive(Default) requires ..Default::default() at every caller
description: Plan GOTCHAs saying "Option<_> lets callers continue compiling" mis-describe Rust's struct-literal syntax. Enumerate caller sites before planning the extension.
type: feedback
originSessionId: 211a8c3b-c503-4ae0-afdb-bd47dba4fdc6
---
**Rule:** When a plan extends a struct (typically `*InsertForm`) with new fields, Rust requires **every caller using struct-literal syntax** to either:
1. name the new field explicitly, or
2. use `..Default::default()` (only valid if the struct derives `Default`).

Option type (`Option<T>`) does **not** make a field optional in struct-literal syntax. "Option<_> lets old callers continue compiling" is a generic design mantra that does not apply to concrete Rust syntax.

**Why:** v1-JM-a R5.1: plan §10.7 GOTCHA claimed `Option<_>` typing alone would let callers compile. Reality: the InsertForm already derived `Default`, so `..Default::default()` was the actual fix. ~10 caller sites in `e2e.rs`, 1 in `create_report.rs`, 1 in `admin_emergency_remove.rs` all needed the suffix added. Plan wording cost ~5 minutes mid-task discovery.

**How to apply:**
- **Before planning an InsertForm extension:** run `rg -l '<FormName> {' crates/` to enumerate caller sites. Count them. If >0, the plan must either:
  - (a) include a task-step that adds `..Default::default()` to every enumerated caller, or
  - (b) explicitly state "derive(Default) on the form + `..Default::default()` at callers" as the propagation mechanism.
- **In plan §10 pattern snippets for InsertForm-extension tasks:** the GOTCHA must say `adding fields to a struct-with-Default-derive requires ..Default::default() at every caller site; enumerate via rg -l '<FormName> {' crates/ before planning`. Avoid the "Option<_> auto-compat" framing — it's incorrect.
- **During impl:** if a struct-extension task reaches cargo check with "missing field X" errors, the remedy is `..Default::default()` at each call site (assuming derive(Default) is present). Don't name every field explicitly — that's churn.

**Applies to:** any Brehon plan that extends a struct with `derive(Default)`, including but not limited to `*InsertForm`. Source: v1-JM-a retro §2.2 R5.1. Retire when plan template is updated to include the pre-task grep step (§3.2 row 2).
