---
name: Commit hygiene — generated siblings + prospective task labels
description: Stage lockfiles and other generated-from-manifest siblings alongside their source edits; use `feat(scope): task N — <summary>` from task 1 of each phase so labelling is prospective not retroactive
type: feedback
originSessionId: 98b0fb05-df8a-4d08-a152-27b3f01323e4
---
Two commit-time disciplines that came out of Phase 1 and should be standard from Phase 2 onward.

**(1) Cargo.lock belongs with the Cargo.toml edit that caused it.** When adding dev-deps or main deps, `cargo check` or the test compile rewrites `Cargo.lock` to include the new transitive resolution. If you only `git add crates/<name>/Cargo.toml` and forget the lockfile, the lockfile drifts into a separate follow-up commit that has nothing to do with the original change. This happened in Phase 1 task 13 — I staged only `crates/server/Cargo.toml` + `crates/server/tests/e2e.rs` and left `Cargo.lock` uncommitted for three more commits, until the final-gate `git status` caught it. The fix was a separate `chore(deps): update Cargo.lock` commit, which is idiomatic but a waste compared to staging together.

**Why**: reproducibility. A lockfile-less dev-dep commit can check out cleanly on the author's machine (lockfile already resolved) but fail on a reviewer's machine when their cargo re-resolves and picks different transitive versions. The two files are a unit.

**How to apply**:
- After any commit that adds or modifies a `[dependencies]` or `[dev-dependencies]` section, run `git diff --stat HEAD` before committing to catch sibling files that were auto-rewritten (Cargo.lock, generated schema.rs, ts-rs exports, etc.). If any sibling is dirty, stage it with the original edit, not as a follow-up.
- This extends to `diesel print-schema` → `crates/db_schema_file/src/schema.rs`, `diesel_ltree.patch` → schema.rs, any codegen output. The pattern is: "one logical change, one commit, all generated siblings included."

**(2) Commit message convention for phase work**: `feat(scope): task N — <summary>` from task 1 of each phase, prospectively. Phase 1 task commit messages varied — some had `task 14` explicitly (`ba7b4400b feat(governance): task 14 — hash-chain trigger test`), others used content-only wording (`810d42ae0 feat(migrations): add governance core tables`). Both are readable, but the former makes Phase 5 git archaeology trivial and the latter requires cross-referencing against the plan file. Retrofitting task numbers to existing commits via `git commit --amend` would rewrite hashes that are already canonical in Phase 1 notes across homeserver + fork, so Phase 1 stays as-is.

**Why**: phases ship 10–15 commits each and Phase 2-6 archaeology will need to answer "which task landed this file?" quickly. `feat(scope): task N — <summary>` answers it in one grep.

**How to apply**: from Phase 2 task 1 onward, use the `feat(<lemmy-scope>): task N — <short summary>` template. Example: `feat(db_views): task 5 — governance_case view with juror join`. `<scope>` matches the Lemmy convention (`migrations`, `db-schema`, `db-schema-file`, `db-views-<name>`, `api`, `apub`, etc.) not a free-form descriptor. Don't retrofit Phase 1 — the cost of the hash rewrite is higher than the archaeology benefit.
