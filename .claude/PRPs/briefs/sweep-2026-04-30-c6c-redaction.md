# [role:impl-task] sweep-2026-04-30 C6c — redaction regex hardening (issue #58)

## 1. Dispatch line

`[role:impl-task] sweep-c6c-redaction — see .claude/PRPs/briefs/sweep-2026-04-30-c6c-redaction.md`

## 2. Scope

Harden the redaction regexes in `crates/api/api/src/governance/redaction.rs` against Unicode and case-variant edge cases that defeat right-to-delete, and add focused unit tests for each. Per `IMPLEMENTATION-PLAN-v0.md` §7.1 ("leak of a username defeats right-to-delete permanently") this is **GDPR-critical**.

**Current state:**
- `redaction.rs` is 22 lines, 3 regexes + a `scrub_json` recursive function.
- 0 tests exist for redaction.

**Hardening steps:**

1. **Read the 3 regex patterns** and identify what each is meant to redact (likely: usernames, emails, instance domains).
2. **For each regex, identify failures** in:
   - **Unicode edge cases** — combining characters (e.g. `é` vs `é`), RTL marks, homoglyph attacks (Cyrillic `а` vs Latin `a`), zero-width-space injection
   - **Case-variants** — uppercase, mixed-case, underscored, dotted variants of the same logical identifier
   - **JSON nesting depth** — `scrub_json` should recurse correctly; verify by reading the recursion implementation
3. **Update each regex** to handle the gaps (likely with `(?i)` for case, `\p{L}` for Unicode letters, NFC normalisation guidance in a comment).
4. **Add unit tests** in the file's `#[cfg(test)] mod tests` block (or create one). Each gap identified above gets a test case: the input string, the redaction output, and the assertion.

**Out of scope:**
- Do NOT redesign the redaction interface (`scrub_json` signature).
- Do NOT add new dependencies. Use `regex` crate features already in scope; if NFC normalisation requires `unicode-normalization`, raise a `kind: "clarify"` DQ instead of adding the dep unilaterally.
- Do NOT modify `crates/api/api_common/` schema files for redacted types.
- Do NOT modify `crates/server/tests/e2e.rs`.

**Boundaries:**
- Edit only `crates/api/api/src/governance/redaction.rs`.
- Single commit subject: `fix(redaction): harden regexes against Unicode + case-variant attacks + tests (closes #58)`.

## 3. Required reading

- **`crates/api/api/src/governance/redaction.rs`** — full file (22 lines)
- **Files that call `scrub_json`** — `grep -rn "scrub_json\|use.*redaction" crates/api/api/src/`
- **`docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`** §7.1 (right-to-delete invariant)
- **e2e tests under `crates/server/tests/e2e.rs`** that exercise the redaction path — locate the test names via grep, but **read with `Read` offset/limit**, not the whole file
- **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — never load full e2e.rs
- **`regex` crate docs** for `\p{L}`, `(?i)`, and Unicode handling — cite via `mcp__ref-context__ref_search_documentation` if needed (e.g. "rust regex unicode classes")
- **GitHub issue #58 body**: `gh issue view 58 --repo barrie-cork/lemmy --json body --jq .body`

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` on the worker. Validation OFF-box.
- Editing any file outside `crates/api/api/src/governance/redaction.rs`.
- Adding new crate dependencies (use `regex` features only).
- Loading `crates/server/tests/e2e.rs` whole. Use `Read` with `offset`/`limit` for any e2e callsite check.

**Required behaviour:**
- Single commit subject: `fix(redaction): harden regexes against Unicode + case-variant attacks + tests (closes #58)`.
- Trailer: `Closes: barrie-cork/lemmy#58`.
- Push branch and exit.
- After push, raise `kind: "validate-pending"` DQ entry with cargo-validate-workspace run id.
- DO NOT open a PR.

**Test discipline:**
- Each regex hardening must have a corresponding unit test that fails with the OLD regex and passes with the NEW regex. Document this in the test name (e.g. `redacts_homoglyph_username`, `redacts_combining_diacritic_email`).
- Add a regression test for any existing input that the OLD regex correctly handled, to ensure no regression.

**File-locality:** `crates/api/api/src/governance/redaction.rs` only. Mutually exclusive from C6a (submit_jury_vote.rs) and C6b (apub crates).

**Mid-task DQ push:** if NFC normalisation cannot be done with current deps, raise `kind: "clarify"` asking whether to add `unicode-normalization` to `crates/api/api/Cargo.toml` or whether to limit hardening to non-Unicode-NFC cases.
