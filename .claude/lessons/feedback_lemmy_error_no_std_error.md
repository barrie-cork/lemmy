---
name: LemmyError doesn't implement std::error::Error
description: LemmyResult<T> wraps LemmyError which can't be ?-converted into Box<dyn Error>. Recipe is case-shape dependent — uniform LemmyResult<T> throughout (Case A, preferred) or Box<dyn Error> throughout with annotated .map_err closures (Case B). Mixed shapes (Case C) are a hard refusal. Surfaced at Phase 2b task 30; case enumeration added 2026-05-09 after v1-SL-c-2 3-cycle catch-fire.
type: feedback
originSessionId: e31990ad-143c-4e12-9bfb-8b135b83d7e6
---

`LemmyError` does not implement `std::error::Error`. Tests calling Lemmy-native functions (which return `LemmyResult<T>`) cannot use bare `?` propagation against a test fn signature that returns `Result<(), Box<dyn Error>>` — `From<LemmyError> for Box<dyn Error>` is missing.

The fix is **case-shape dependent**, not a single mechanical bridge. Pick the case that matches the surrounding code shape and propagate it uniformly through the whole module.

## Case A — uniform `LemmyResult<T>` throughout (preferred, canonical-sibling shape)

**Use when:** every helper, every callee, and the test fn itself can return `LemmyResult<T>`. Specifically: helpers do their own DB/Lemmy work via `LemmyResult`-returning calls; the test fn body only calls Lemmy-native functions plus those helpers. No `Box<dyn Error>` anywhere in the module.

**Recipe:**
- Test fn signature: `async fn <name>() -> LemmyResult<()>`.
- Every helper signature: `-> LemmyResult<T>` (NOT `Result<T, Box<dyn Error>>`).
- Every `?` propagation: bare, no `.map_err`.
- Module imports: `use lemmy_utils::error::LemmyResult;`.

**Why this works:** `LemmyResult<T>` is `Result<T, LemmyError>`. Every Lemmy-native call already returns `LemmyResult<T>`, so `?` propagates `LemmyError → LemmyError` directly. No bridge. No `Box<dyn Error>` in the module body.

**Canonical reference:** `crates/server/tests/e2e.rs:11001-11924` (the v1-SL-b `mod v1_sl_b_fixtures` module). 7 `LemmyResult<T>` helpers + 9+ test fns, all `LemmyResult<()>`, all bare `?` — zero `.map_err`, zero `Box<dyn Error>` in the module.

## Case B — uniform `Box<dyn Error>` throughout, with annotated `.map_err` closures at Lemmy-native call sites

**Use when:** the test fn must return `Result<(), Box<dyn Error>>` for some external reason (e.g. `#[test]` harness compatibility before tokio integration), AND every helper also returns `Result<T, Box<dyn Error>>`. This is rare in modern Lemmy fork code — most fixtures modules can use Case A.

**Recipe:**
- Test fn signature: `async fn <name>() -> Result<(), Box<dyn Error>>`.
- Every helper signature: `-> Result<T, Box<dyn Error>>` (homogeneous).
- Diesel and other `std::error::Error`-implementing call sites: bare `?` (Diesel errors propagate cleanly into `Box<dyn Error>`).
- **Lemmy-native call sites (returning `LemmyResult<T>`):** annotated `.map_err` closure with explicit return type. Two equivalent forms:

  **Form B1** — explicit closure return type annotation:
  ```rust
  let x = some_lemmy_call().await
      .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { format!("{e}").into() })?;
  ```

  **Form B2** — explicit `Box::<...>::from`:
  ```rust
  let x = some_lemmy_call().await
      .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(format!("{e}")))?;
  ```

**Why the annotation matters:** `Result<(), Box<dyn Error>>` is **abstract** at the test fn boundary. Without an explicit closure return type, the compiler cannot infer `_` in `Into<_>` from `From<String> for _` because there's no concrete target — `Box<dyn Error>` is a trait object. Result: `error[E0283]: type annotations needed`.

**Bare `.map_err(|e| format!("{e}").into())` does NOT work** when the outer return is abstract `Box<dyn Error>`. That was the historical Phase 2b recipe and works only when the outer return is a concrete error type whose `From<String>` impl is unambiguous (rare for `Box<dyn Error>`; common for crate-local error enums).

## Case C — mixed shapes (HARD REFUSAL — re-plan, do not bridge)

**Symptom:** test fn returns `LemmyResult<()>` but helpers return `Result<T, Box<dyn Error>>`, OR vice versa.

**Why this fails:** `dyn Error` (trait object inside `Box<dyn Error>`) lacks the `Send + Sync + Sized + 'static` bounds that `LemmyError` requires for `?` to work in either direction. No mechanical bridge resolves this — bridging in one direction breaks the other. The compile errors will cascade across every helper-call site (E0277 trait bound failures, often 8-12 sites at once).

**Fix:** type-shape uniformity is mandatory across a single e2e module. Pick Case A (preferred) or Case B and propagate it through every helper + test fn. Do NOT attempt to bridge mid-module.

If you encounter this case mid-cycle (e.g. after a partial fix), surface to user as a re-plan signal — the recipe family was wrong-shaped, mechanical fixes will not converge.

## Symptoms recognised

- **Phase 2b task 30 smoke tests** (original surfacing) — Case B with crate-local error enum outer (worked). Mechanical recipe `.map_err(|e| format!("{e}").into())` was sufficient because outer was concrete.
- **v1-SL-c-2 cycles 1-3 catch-fire** (2026-05-09, workflow runs `25582548670` / `25595869651` / `25603848858`):
  - Cycle 1: Plan §13 stub prescribed `Result<(), Box<dyn Error>>` outer with no bridges → 1× E0277 (`LemmyError: std::error::Error`).
  - Cycle 2: fix-impl-1 brief flipped outer to `LemmyResult<()>` and **forbade** helper updates → 11× E0277 (Case C — Send/Sync/Sized cascade).
  - Cycle 3: fix-impl-2 brief reverted outer to `Result<(), Box<dyn Error>>` + bare `.map_err(|e| format!("{e}").into())?` → 1× E0283 (no annotated closure return type; abstract trait object outer).
  - Resolution: re-planned to Case A (uniform `LemmyResult<T>` throughout, mirroring v1-SL-b sibling). Case enumeration added to this lesson; §G4 row split into 4a/4b/4c.

## How to apply

When authoring a brief that will edit `crates/server/tests/e2e.rs` (or any test module under `crates/*/tests/**`):

1. Read the existing v1-SL-* fixtures modules in the same file. If one matches the test class you're authoring, **mirror its case verbatim** — same outer return, same helper return, same `?` propagation pattern. This is the canonical-schema-first gate.
2. If no canonical sibling exists, default to Case A. Lemmy 1.0-beta fork code already uses `LemmyResult<T>` everywhere; mixing in `Box<dyn Error>` is a Phase 2b legacy artefact, not a current convention.
3. Verify the case in Required reading citations: include this lesson's path AND the canonical sibling's path with line numbers.
4. **Never write a fix-impl brief that prescribes Case C** (mixed shapes). The §G4 classifier row 4c will refuse the auto-fix and surface to user for re-plan.

## Generalises to

Any Rust test module that mixes a fork-local error type that doesn't implement `std::error::Error` with `std::error::Error`-implementing callees. The failure mode is the same: `?` propagation cannot bridge across the impl boundary without an explicit recipe choice.

## See also

- `.claude/rules/advisor-orchestrator.md` §5.3 §G4 classifier (rows 4a/4b/4c).
- `.claude/lessons/feedback_async_pool_test_pattern.md` — companion lesson on the AsyncPgConnection + DbPool pattern that often co-occurs with the error-shape choice.
- `.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md` — the v1-SL-c-2 case-enumeration RCA.
- v1-SL-b canonical reference module: `crates/server/tests/e2e.rs:11001-11924`.
