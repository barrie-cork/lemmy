---
name: Plan mirror stubs must compile + annotate pre-landed callers
description: Planner authoring a multi-task plan whose Task N adds infra and Task N+1 first-calls it MUST include a temporary unit test (or assert call site) that exercises Task N's signature so the compiler proves call-site discipline at Task N's §15, NOT Task N+1's. Strip the assert in the same task that strips #[expect(dead_code)]. Prevents the "stub passes Task N validate, breaks at Task N+1 because no compile-call exists" failure mode.
type: feedback
originSessionId: b8d4a39f-9c25-5e7a-af63-cdb123456789
---

## What this catches

The **stub-lands-but-call-site-breaks** failure mode: a Task N "stub" (a struct, fn signature, trait implementation, or migration) passes its own §15 `cargo check --workspace --features full` because nothing in the workspace calls it yet -- or the stub is annotated `#[expect(dead_code)]` to suppress the unused-item warning. Task N+1 is the first task to call the stub, and the call-site compile error pops at Task N+1's §15 instead of Task N's.

The cost: one ci-watcher cycle (5-26 min), one `fix-impl-task` Junior dispatch, and a brief revision cycle -- all for a type-shape mismatch that a single `cargo check` call at Task N's commit time would have caught.

## Recipe (planner-side)

The planner authoring a multi-task plan where Task N adds infra and Task N+1 first-calls it MUST add a **temporary compile probe** in Task N's IMPLEMENT block that exercises the stub's signature against its eventual caller. Concrete form:

```rust
// Task N -- temporary compile probe for <stub_name>
// Strip this assert in the same commit that strips #[expect(dead_code)].
#[cfg(test)]
mod task_n_compile_probe {
    use super::*;
    fn _compile_probe_<stub_name>() {
        let _: Result<_, _> = <stub_name>(<arg_type>::default());
    }
}
```

The probe lives in the **same crate as the stub**, so Task N's `cargo check --workspace --features full` exercises it. The probe does NOT need to run (it's `#[cfg(test)]` and not called from any test fn) -- it only needs to compile. The compiler catches signature drift at Task N's §15, not Task N+1's.

**Strip discipline:** the temporary probe is removed in the SAME commit that removes `#[expect(dead_code)]` -- typically Task N+1's commit. Brief §13 Task N+1 MUST include "strip `#[cfg(test)] mod task_n_compile_probe`" in its IMPLEMENT block. Do NOT leave the probe past Task N+1.

**Alternative form (for fn signatures, not struct constructors):**

```rust
#[cfg(test)]
fn _task_n_type_check() {
    let _ : fn(<ArgType>) -> Result<ReturnType, _> = <stub_name>;
}
```

Both forms compile the fn's type without calling it at runtime.

## Worked example

**Scenario:** Task N adds `pub fn process_inbound_label(payload: &InboundLabel, pool: &DbPool) -> LemmyResult<()>` with `#[expect(dead_code)]`. Task N+1 first-calls it from `receive_remote_moderation_label`.

**Without the probe:** Task N's `cargo check` passes (no caller, `#[expect(dead_code)]` suppresses the warning). Task N+1's IMPLEMENT block adds the call:
```rust
process_inbound_label(&label, pool).await?;
```
Task N+1's `cargo check` sees the first compile error -- perhaps `payload` type changed from `&InboundLabel` to `InboundLabel` (ownership mistake), or `LemmyResult<()>` changed to `Result<(), Box<dyn Error>>` (error-shape mismatch). A full ci-watcher cycle and fix-impl dispatch follow.

**With the probe (in Task N's IMPLEMENT):**
```rust
#[cfg(test)]
fn _task_n_type_check() {
    let _: fn(&InboundLabel, &DbPool) -> LemmyResult<()> = process_inbound_label;
}
```
Task N's `cargo check` catches the type mismatch immediately -- before the ci-watcher cycle, before Task N+1 is queued. The planner's brief is the right place to add this probe because the planner knows both Task N's signature and Task N+1's intended call-site shape.

## When this applies

Multi-task plans where:

- Task N adds infra (data structures, fn signatures, trait impls, migrations) that no existing call-site in the workspace exercises.
- Task N uses `#[expect(dead_code)]` or `#[allow(dead_code)]` to suppress unused-item warnings.
- Task N+1 (or later) adds the first call sites.

**Does NOT apply** to plans where Task N's stub IS called by existing code (the compiler already exercises it). Does NOT apply to plans where Task N and Task N+1 are the same task (bundle if the stub is trivial).

## Cross-link note

Cross-link `[[feedback_dead_code_shields_latent_type_errors]]` not yet promoted per session-retro-2026-05-20 change-#3; using `[[feedback_plan_stub_uniformity_with_canonical_sibling]]` as closest sibling.

## See also

- `[[feedback_plan_stub_uniformity_with_canonical_sibling]]` -- the companion lesson on Task N stubs diverging from same-file canonical sibling shape; this lesson is the planning-side corollary (prove call-site discipline at Task N, not Task N+1).
