# Brief: m3-core-stage-mode fix-impl-4 (Task 4 stage.rs doc-overindent clippy)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-fix-impl4-stage-doc-overindent — see .claude/PRPs/briefs/m3-core-stage-mode-fix-impl-4.md`

## §2 Scope

**§G4 allowlist match — verbatim row (anti-paraphrase gate):**

> | `clippy::doc_overindented_list_items` warning | re-indent the doc-list continuation to clippy's suggested column (4 spaces under `///`) per the `help: try using` hint | n/a (mechanical; added 2026-06-19 m3-core-stage-mode Task 4 — sibling of `doc_lazy_continuation`) |

**Fix the ONE clippy error that failed Task 4's `validate-pending-laptop-linux` validation** (DQ `13fd7f8da89b-001`). `cargo check` PASSES and the bridge compiles; the ONLY blocker is a single doc-comment formatting lint.

**The failing lint (verbatim):**
```
error: doc list item overindented
   --> src/stage.rs:161:9
    |
161 |     ///            `std::future::ready(())` when activation already happened.
    |         ^^^^^^^^^^^ help: try using `    ` (4 spaces)
    |
    = note: `-D clippy::doc-overindented-list-items` implied by `-D warnings`
```

The `run_grace` doc comment (stage.rs ~155-162) has a two-item list:
```rust
/// Callers supply the cancel Future:
///   - Tests: `std::future::pending::<()>()` for the boundary case;
///            `std::future::ready(())` when activation already happened.
///   - Production (Task 5+): a `oneshot::Receiver<()>` that `on_activate` fires.
```
The continuation line (`std::future::ready...`) is over-indented relative to the `- Tests:` bullet. clippy wants list items + continuations aligned to a consistent 4-space indent under `///`.

**Produces (exactly 1 file edit, ONE commit):**
1. `services/bridge/src/stage.rs` — fix ONLY the doc-comment indentation in the `run_grace` doc block (~lines 159-162) so clippy's `doc_overindented_list_items` passes. The clean form clippy accepts:
   ```rust
   /// Callers supply the cancel Future:
   /// - Tests: `std::future::pending::<()>()` for the boundary case;
   ///   `std::future::ready(())` when activation already happened.
   /// - Production (Task 5+): a `oneshot::Receiver<()>` that `on_activate` fires.
   ```
   (List markers `- ` directly after `/// `; continuation indented 2 spaces under the marker. This is the standard markdown-in-rustdoc list form clippy expects. If clippy still flags after this, follow the exact `help: try using` column it prints.)

**Do NOT:**
- Change any code line, signature, test, or non-doc-comment text — this is a pure doc-comment whitespace fix.
- Add `#[allow(clippy::doc_overindented_list_items)]` — re-indent properly, do not suppress.
- Touch any other file, any `crates/**`, any migration, `Cargo.toml`/`Cargo.lock`.
- Touch the two grace tests or any of the 6 Task-3 tests.

**Branch:** forks from the **fix-impl-3 #715 worker branch** `junior/role-impl-task-m3-core-stage-mode-fix-impl3-tokio-time-feature-gate-see-claude-prps-briefs-m3-core-stage-mode-fix-i-715` (tip `8c9f0ec56`) — so the fix lands on the lineage that already has the Cargo.toml feature toggle + the committed stage.rs.

## §3 Required reading

- `services/bridge/src/stage.rs:155-165` — the `run_grace` doc comment with the over-indented list.
- **Lessons (mandatory, §2.4 file-class — `services/bridge/**`):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.

## §4 Constraints

- **ONE commit, 1 file** — `fix(rtc): re-indent run_grace doc list to satisfy clippy doc-overindent (fix-impl 4)`.
- **Re-indent, NOT suppress** — fix the whitespace; no `#[allow]`.
- **Pure doc-comment change** — no code/test/signature touched; the 8 tests are unaffected.
- **Resolve DQ `13fd7f8da89b-001`** in the same commit (move pending→resolved, `answered_by: "advisor"`, `answer` = "doc-overindent fixed per advisor §G4 allowlist 2026-06-19").
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.

## §3a Handover from prior cohort

Task 4 (#713 code @ `53b8c5612`) + fix-impl-3 (#715 Cargo.toml feature toggle @ `8c9f0ec56`) landed the 30s grace timer + the tokio `time`/`test-util` features. The bridge now `cargo check`s clean on Linux. The ONLY remaining validation blocker is a single `doc_overindented_list_items` clippy lint at `stage.rs:161` in the `run_grace` doc comment (the cancel-future callers list). This is a pure doc-comment whitespace fix; the 8 tests + all code are correct and unchanged.
