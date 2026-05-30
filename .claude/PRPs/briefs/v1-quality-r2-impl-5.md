# Brief: v1-quality-r2 impl-task 5 — wrap 13 LEMMY_DATABASE_URL setter sites with EnvVarGuard

## 1. Role + dispatch line

`[role:impl-task]` v1-quality-r2 task 5 — LEMMY_DATABASE_URL EnvVarGuard wrap 13 sites (closes #160) — see `.claude/PRPs/briefs/v1-quality-r2-impl-5.md`

## 2. Scope

**Goal:** implement Task 5 (C4-B) from `.claude/PRPs/plans/v1-quality-r2.plan.md`.

Replace every `unsafe { std::env::set_var("LEMMY_DATABASE_URL", ...) }` block in `e2e.rs` (outside `boot_context()`) with `let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", ...);`. 13 sites total. Also add `use super::EnvVarGuard;` to `governance_fixtures` and `admin_config_fixtures` (those two modules lack `use super::*`; `v1_ship_3_fixtures` already has `use super::*` so EnvVarGuard is visible there without explicit import after T4's hoist).

**Prerequisite:** T4 must be merged into `phase-v1-quality-r2` BEFORE this task starts (T4 hoists `EnvVarGuard` to test-crate root). Worker forks from the phase branch tip which will include T4's commit. Verify: `grep -n 'struct EnvVarGuard' crates/server/tests/e2e.rs` should return a line number < 200 (top-of-file, before `mod governance_fixtures {`). If it returns ~17256 (inside the module), T4 has NOT landed — stop and raise `kind: "blocker"` DQ.

**Out of scope:**
- Site 14 (line ~17508, inside `boot_context()`) — handled by T4, NOT T5.
- Do NOT touch T4's callsite update lines or `use super::EnvVarGuard;` inside `mod v1_rt_r3_fixtures`.
- Do NOT edit any module other than the 13 enumerated sites + the two `use` block additions.
- Do NOT add `use super::EnvVarGuard;` to modules that already have `use super::*;`.

**Commit subject:** `refactor(e2e): wrap 13 LEMMY_DATABASE_URL setter sites with EnvVarGuard (closes #160, task 5)`

**Single file:** `crates/server/tests/e2e.rs` only.

## 2.1 Pre-locate anchor check (R11)

Per `feedback_fix_impl_pre_locate_e2e_anchors.md`: before any Edit, locate each `old_string` verbatim in the current file. If ANY anchor does not match byte-for-byte, STOP — raise `kind: "blocker"` DQ. Do NOT fuzzy-match.

All anchors below were pre-located against `origin/phase-v1-quality-r2` tip at the time of brief authorship. T4 will shift line numbers when it merges — use text anchors, not line numbers.

**T4 prerequisite check (run before any Edit):**
```python
import re, sys
content = open('crates/server/tests/e2e.rs').read()
m = re.search(r'^struct EnvVarGuard \{', content, re.MULTILINE)
if not m:
    print('FAIL: EnvVarGuard struct not found at crate root — T4 may not have landed'); sys.exit(1)
first_mod = re.search(r'^mod \w+_fixtures \{', content, re.MULTILINE)
if m.start() > first_mod.start():
    print(f'FAIL: EnvVarGuard at line {content[:m.start()].count(chr(10))+1} is AFTER first mod at line {content[:first_mod.start()].count(chr(10))+1} — T4 not landed'); sys.exit(1)
print('OK: EnvVarGuard hoisted to crate root (T4 landed)')
```

## 2.2 `use super::EnvVarGuard;` additions (2 edits)

### Edit USE-1 — governance_fixtures use block

**old_string**:
```
mod governance_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use actix_web::web::Data;
```

**new_string**:
```
mod governance_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::EnvVarGuard;
  use actix_web::web::Data;
```

### Edit USE-2 — admin_config_fixtures use block

**old_string**:
```
mod admin_config_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use actix_web::web::Data;
```

**new_string**:
```
mod admin_config_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::EnvVarGuard;
  use actix_web::web::Data;
```

## 2.3 Verbatim setter-site replacements (13 edits)

**Replacement template (for sites with `&db_url`):**

old_string pattern:
```
  unsafe {
    std::env::set_var("LEMMY_DATABASE_URL", &db_url);
  }
```
new_string pattern:
```
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

For sites with SAFETY comments, the comment is REMOVED. The justification lives at the hoisted `EnvVarGuard::set` fn.

### Site 1 (line ~811, governance_fixtures, `&db_url`)

**old_string** (3 surrounding lines for uniqueness):
```
    let db_url = db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 2 (line ~2549, governance_fixtures test scope, `&db_url`)

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

> **NOTE for Sites 2–10:** many of these share the same 3-line pattern `let db_url = governance_fixtures::db_url(host_port); unsafe { std::env::set_var(...) }`. Expand the anchor with 1-2 surrounding lines (the line before `let db_url` or the lines after the unsafe block) to ensure uniqueness. Use `replace_all: false` (default). If an anchor is still non-unique, include the enclosing `async fn` name as additional context.

---

### Site 3 (line ~3329, governance_fixtures, `&db_url`)

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
```

> **Anchor note:** site 3 is unique because the line after the unsafe block is `{` immediately (no blank line), followed by `let mut sync_conn`. Use these trailing lines as the anchor extension if needed.

---

### Site 4 (line ~4101, `&db_url`, blank line after)

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

> **NOTE for sites 4–7:** if the 6-line anchor (db_url assignment + unsafe block + blank + `{`) appears multiple times, extend with the test function name from 3-5 lines above.

---

### Site 5 (line ~4448)

Same pattern as site 4. Use function-name context to disambiguate if needed.

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 6 (line ~4779)

Same pattern. Disambiguate via function name if needed.

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 7 (line ~4912)

Same pattern. Disambiguate via function name if needed.

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 8 (line ~5049, two-DB pattern, `&url_a`)

This site has a SAFETY comment and uses `url_a` not `db_url`.

**old_string**:
```
  // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
  unsafe {
    std::env::set_var("LEMMY_DATABASE_URL", &url_a);
  }
  let pool_a: ActualDbPool = build_db_pool_for_tests();
```

**new_string**:
```
  let _g_db_url_a = EnvVarGuard::set("LEMMY_DATABASE_URL", &url_a);
  let pool_a: ActualDbPool = build_db_pool_for_tests();
```

---

### Site 9 (line ~5098, two-DB pattern, `&url_b`)

This site has a SAFETY comment and uses `url_b`.

**old_string**:
```
  // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
  unsafe {
    std::env::set_var("LEMMY_DATABASE_URL", &url_b);
  }
  let pool_b: ActualDbPool = build_db_pool_for_tests();
```

**new_string**:
```
  let _g_db_url_b = EnvVarGuard::set("LEMMY_DATABASE_URL", &url_b);
  let pool_b: ActualDbPool = build_db_pool_for_tests();
```

> **Note for sites 8+9:** use `_g_db_url_a` and `_g_db_url_b` (not `_g_db_url`) to avoid name collision within the same function scope where both `url_a` and `url_b` are set.

---

### Site 10 (line ~5676, `&db_url`)

Same standard pattern. Disambiguate via function name if needed.

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 11 (line ~5879, `&db_url`)

Same standard pattern. Disambiguate via function name if needed.

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 12 (line ~6147, admin_config_fixtures, `&db_url`, uses `super::governance_fixtures`)

**old_string**:
```
    let db_url = super::governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
```

**new_string**:
```
    let db_url = super::governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
```

---

### Site 13 (line ~16823, v1_ship_3_fixtures, `&db_url`, no blank line after)

**old_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
```

**new_string**:
```
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
```

> **Anchor note:** site 13 is in `mod v1_ship_3_fixtures` and unique because of the `super::governance_fixtures` context and the no-blank-line-after-unsafe pattern (same as site 3's `{` continuation).

---

### Do NOT touch site 14 (boot_context, line ~17508)

The site inside `async fn boot_context()` is handled by Task 4. Do not edit any line near `17508`.

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r2.plan.md` §10.2 (EnvVarGuard callsite usage shape), §10.1 (EnvVarGuard struct — verify it's at crate root in T4's commit)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — e2e.rs error shape
- `.claude/lessons/feedback_async_pool_test_pattern.md` — async pool pattern
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate anchors; stop on mismatch

**Mandatory MIRROR ref:** T4's commit on `phase-v1-quality-r2` — read it to confirm the hoisted `EnvVarGuard` location and `use super::EnvVarGuard;` in `mod v1_rt_r3_fixtures` before editing.

## 4. Constraints

1. **T4 prerequisite guard:** run the Python check in §2.1 before any Edit. If it fails, raise `kind: "blocker"` DQ.
2. **Pre-push cargo check:** run `bash scripts/brehon/cargo-check.sh --workspace --features full` before pushing. Non-zero → fix in same commit (if in-scope) or raise `kind: "blocker"` DQ.
3. **`_g_db_url` not `_`:** using bare `_` drops the guard immediately. Use `_g_db_url` (or `_g_db_url_a`/`_g_db_url_b` for the two-DB sites).
4. **SAFETY comments removed:** when the `unsafe` block had a `// SAFETY:` comment, remove it with the block. The justification now lives at the hoisted `EnvVarGuard::set` fn in `impl EnvVarGuard`.
5. **`use super::EnvVarGuard;` conditional:** only add it to modules that lack `use super::*;`. From the pre-located audit: add to `governance_fixtures` and `admin_config_fixtures`; skip for `v1_ship_3_fixtures` (already has `use super::*`).
6. **R11 anchor discipline:** if ANY `old_string` fails to match verbatim, STOP — do not fuzzy-match — raise `kind: "blocker"` DQ.
7. **DQ mid-task push:** if you raise a `kind: "blocker"` DQ entry, commit + push `decision-queue.json` immediately.
8. **validate-pending-laptop DQ:** after successful push, write a `kind: "validate-pending-laptop"` DQ entry with commands:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --test e2e --no-run --workspace --features full`
   - Python audit: `python3 -c "import subprocess; r = subprocess.run(['grep', '-c', 'unsafe.*set_var.*LEMMY_DATABASE_URL', 'crates/server/tests/e2e.rs'], capture_output=True); count=int(r.stdout.strip()); print(f'FAIL: {count} unsafe LEMMY_DATABASE_URL sites remain' if count > 0 else 'OK: no unsafe LEMMY_DATABASE_URL sites outside boot_context')"`

**Mandatory file-class lessons fired:** `feedback_lemmy_error_no_std_error.md` (e2e.rs edit), `feedback_async_pool_test_pattern.md` (e2e.rs edit), `feedback_fix_impl_pre_locate_e2e_anchors.md` (≥2 edits to e2e.rs).
