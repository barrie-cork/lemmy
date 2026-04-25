---
name: edit-mechanical
description: Apply mechanical Rust edits with rg-enumerate-first discipline (R5.1 prevention).
user-invocable: true
---

# `/edit-mechanical` — disciplined mechanical edits with caller enumeration

Apply small, well-defined, repeatable Rust edits across multiple call sites in a single bounded pass. Designed to prevent the R5.1 class of bug (PR #92 cr-12, JM-b Event 2): "I added a field with `Default::default()` propagation; tests broke because three callers used explicit struct literals." This skill makes the enumeration step a precondition, not an afterthought.

## When to invoke

- Adding a field to a struct with `derive(Default)` and propagating to N call sites
- Renaming an enum variant or trait method across the workspace
- Adding a `#[expect(lint, reason="...")]` attribute to N sites with the same shape
- Replacing a deprecated API call across known sites
- Mechanical lint fixes (e.g., `i64::try_from(x).unwrap_or(i64::MAX)` for `len()-as-i64`, `bool::then_some` swaps)

Skip when:
- The edit requires reasoning about types/borrows beyond the rename surface (use main thread, Opus + xhigh)
- The edit is in `crates/server/tests/e2e.rs` AND adds new test cases (use `/test-write`)
- The edit is in `migrations/**` (out of scope; migrations need explicit plan amendment)

## Inputs

The user/plan provides:

- **The mechanical change** — exactly what's being changed. One of:
  - "Add field `<name>: <type>` to struct `<FormName>`, default `<value>`"
  - "Rename `<old>` → `<new>` in enum `<EnumName>`"
  - "Add `#[expect(<lint>, reason=\"<r>\")]` to fn `<F>` in `<file>:<line>` (and any siblings matching pattern)"
  - Free-form: a one-sentence description that names exactly which symbol changes how
- **Scope ceiling** — `crates/`, `crates/api/`, `tests/` etc. Defaults to `crates/` if unspecified
- **Forbidden zones** — overrides from the user. Always-forbidden: `migrations/**`, `Cargo.toml`, `Cargo.lock`

If the change cannot be expressed as one mechanical pattern, STOP and ask the user to split it. This skill does ONE pattern at a time.

## Procedure (HARD ORDERING — do not skip steps)

### Step 1 — Enumerate

Run the canonical enumeration grep BEFORE any edit. For struct field propagation:

```bash
# Caller-site enumeration for InsertForm-style propagation
rg -l '<FormName> \{' crates/ | sort
```

For enum variant renames:

```bash
rg -l '<EnumName>::<OldVariant>' crates/ | sort
```

For attribute additions matching a shape:

```bash
rg -l '<pattern>' crates/ | sort
```

**Capture the file list as the work plan.** Output it as the first item in the report.

### Step 2 — Classify each site

For each enumerated file, determine which propagation shape it uses. For struct-field propagation specifically:

- **Sites using `..Default::default()`** — auto-compatible if the field has `Default`. No edit needed; record as "auto-compat."
- **Sites using explicit struct literals** — edit needed. Record as "needs-propagation."
- **Sites in `tests/` using fixture builders** — check the builder. If builder uses `..Default::default()`, sites are auto-compat. Else needs-propagation.

Output the classification before editing.

### Step 3 — Edit

Now edit. For each "needs-propagation" site:

- Use Edit with sufficient surrounding context to disambiguate (per Edit tool docs — `old_string` must be unique)
- Add the new field/variant/attribute matching the project's existing style at that site
- Do NOT reformat surrounding code; do NOT add/remove blank lines; do NOT touch unrelated lines

### Step 4 — Verify enumeration was complete

After edits, re-run the enumeration grep from Step 1 with the new field/symbol included:

```bash
rg -l '<FormName> \{' crates/ | sort > /tmp/post-edit.txt
diff /tmp/pre-edit.txt /tmp/post-edit.txt
```

Expect: identical file lists. New files appearing means a caller was created mid-edit (race) — STOP. Files disappearing means an edit broke a file's parse — STOP and re-read.

### Step 5 — Compile-check

Pair-invoke `/cargo-validate check -p <crate>` (per-crate first, narrow scope), then `/cargo-validate check --workspace --features full` if the change crosses crate boundaries. Don't report done until both exit 0.

### Step 6 — Report

```
pattern: <one line — what changed>
scope:   crates/...

enumeration (Step 1):
  N files matched
  - <file 1>
  - <file 2>
  ...

classification (Step 2):
  auto-compat:      M sites
  needs-propagation: K sites
  forbidden:         X sites (migrations/, Cargo.toml — skipped)

edits (Step 3):
  K sites edited

verification (Step 4):
  diff pre/post enumeration: identical (or: <details>)

compile-check (Step 5):
  /cargo-validate check -p <crate>     exit 0
  /cargo-validate check --workspace    exit 0

commit suggestion: chore(<scope>): <one-line> — propagated <field> to N call sites
```

## What this skill never does

- Edit `migrations/**`, `Cargo.toml`, or `Cargo.lock`
- Make multiple unrelated mechanical changes in one invocation (one pattern per invocation; ask user to split)
- Skip the enumeration step (Step 1)
- Skip the verification step (Step 4) — that's the R5.1-prevention gate
- Use Bash for `sed`/`awk` mass-edits — always Edit tool, one site at a time, for review-friendly diffs
- Reformat surrounding code or "while-I'm-here" cleanup
- Commit. Caller's job, with the suggested subject from Step 6
- Run tests. Use `/cargo-validate` or the cargo-runner subagent

## R5.1 — the bug class this skill exists to prevent

JM-a R5.1 (PR #92 cr-12 root cause, repeated as JM-b Event 2 at `cb7b6bdb2`):

> Adding a field to a struct with `derive(Default)` is NOT auto-compatible with all callers. Sites using `..Default::default()` propagate; sites using explicit struct literals do NOT. The drift-fix at `cb7b6bdb2` claimed `..Default::default()` propagation in the commit body but didn't run the enumeration — three test sites in `crates/server/tests/e2e.rs` (lines 917, 3170, 3811) used explicit literals and broke at compile time.

The Step 1 enumeration step IS the fix. Do NOT skip it under any time pressure.

## Related rules (auto-loaded)

- `.claude/rules/pm-plugin-hooks-stable.md` — if the symbol being renamed is a PM-path hook string, STOP (those are stability-guaranteed)
- `.claude/rules/decision-queue.md` — if the change requires a plan amendment, file a DQ instead

## Memory references

- `feedback_insertform_default_propagation.md` — R5.1 root description; primary motivator for this skill
- `feedback_carry_patch_todos.md` — when to mark a fork-only edit as upstream-bound TODO
- `feedback_clippy_test_style.md` — never use `#[allow]`; use `#[expect(reason="...")]`
- `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` — specific lint-fix idiom
- `feedback_pg_advisory_xact_lock_void_decode.md` — example of mechanical fix with subtle gotcha (this skill doesn't write that one, but the pattern is canonical)
