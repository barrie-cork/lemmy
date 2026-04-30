# [role:impl-task] sweep-2026-04-30 C4 — reqwest_middleware skew in OAuth path (issue #83)

## 1. Dispatch line

`[role:impl-task] sweep-c4-cargo-dep-skew — see .claude/PRPs/briefs/sweep-2026-04-30-c4-cargo-dep-skew.md`

## 2. Scope

Resolve the `reqwest_middleware` version skew that breaks `cargo check -p lemmy_api_crud --features full` in the OAuth path (issue **#83**). The break is upstream-originated (not introduced by Brehon work) and was routed around at v1-AD-c by validating at `-p lemmy_api` instead of `-p lemmy_api_crud`. This blocks the first non-governance phase that touches `lemmy_api_crud` full-feature builds.

**Fix path is dep-graph-only:**

1. Read the current `reqwest-middleware` constraint in `Cargo.toml` (workspace root). Note: it's currently `reqwest-middleware = "0.5.1"` per recent inspection.
2. Read the constraint in `crates/api/api_crud/Cargo.toml` if present.
3. Cross-reference the OAuth path: identify the file in `crates/api/api_crud/` that uses `reqwest-middleware` (typically `oauth.rs` or similar) and identify its imports.
4. Identify whether the skew is between two transitive deps (e.g. workspace pins `0.5.1` but a transitive pin demands `0.4.x`) by inspecting `Cargo.lock` for the package — `grep "reqwest-middleware" Cargo.lock` will list every version present.
5. Resolve by **either** (a) bumping/lowering the workspace constraint to a version that satisfies all transitive deps, **or** (b) adding a `[patch.crates-io]` override pinning a single version, **or** (c) explicitly relaxing the version constraint (e.g. `0.5` instead of `0.5.1`) so cargo can resolve.

**Cannot run cargo on this worker.** The fix must be defensible by reading `Cargo.lock` + `Cargo.toml` + the OAuth source file and identifying the version conflict by analysis. Validation runs OFF-box via GH Actions push trigger on the worker branch.

**Out of scope:**
- Do NOT modify the OAuth plugin interface code. No `crates/api/api_crud/src/oauth.rs` edits beyond imports if the dep change forces them.
- Do NOT modify lemmy_api or other crates' Cargo.toml beyond what the dep resolution requires.
- Do NOT rebuild Cargo.lock from scratch (no `rm Cargo.lock`); make targeted edits.

**Boundaries:**
- Edit `Cargo.toml` (workspace root) and `crates/api/api_crud/Cargo.toml` if present.
- Edit `Cargo.lock` ONLY via the same source as the .toml change (i.e. allow the cargo update to flow through; do NOT hand-edit Cargo.lock entries).
- Single commit subject: `chore: pin reqwest_middleware to resolve OAuth skew in lemmy_api_crud (closes #83)`.

## 3. Required reading

- **`Cargo.toml`** (workspace root, line ~193 `reqwest-middleware = "0.5.1"`)
- **`crates/api/api_crud/Cargo.toml`** (if present)
- **`Cargo.lock`** — search for `reqwest-middleware` and any transitive `reqwest_middleware` to enumerate versions
- **`crates/api/api_crud/src/oauth.rs`** (or equivalent file using reqwest-middleware in the OAuth path)
- **`.claude/build-task4-tests.log`** (if still in repo) for the original failure log
- **`.claude/lessons/feedback_features_full_workspace_only.md`** — `--features full` only via `--workspace`
- **`.claude/lessons/pattern_cargo_feature_flag_propagation.md`** — cargo dep / feature propagation pitfalls
- **GitHub issue #83 body**: `gh issue view 83 --repo barrie-cork/lemmy --json body --jq .body`

## 4. Constraints

**HARD FORBIDS:**
- `cargo *` on the worker. NO `cargo check`, NO `cargo update`, NO `cargo build`, NO `cargo test`, NO `cargo clippy`. Validation is OFF-box only.
- Hand-editing `Cargo.lock` entries. The lock is a build artefact, not a source.
- Modifying OAuth plugin behaviour. Dep-graph-only fix.
- Bumping any other dep (no opportunistic upgrades).

**Required behaviour:**
- Single commit subject: `chore: pin reqwest_middleware to resolve OAuth skew in lemmy_api_crud (closes #83)`.
- Trailer: `Closes: barrie-cork/lemmy#83`.
- Push branch and exit.
- After push, raise a `kind: "validate-pending"` DQ entry in `.claude/decision-queue.json` with the new GH Actions run id (use `gh run list --repo barrie-cork/lemmy --branch <your-worker-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId,headBranch` after push). The advisor's polling loop will see the entry and dispatch a ci-watcher.
- DO NOT open a PR — Junior daemon's finalize-merge will land directly on `governance-v0` after the validate-pending entry resolves.

**Approach guidance:**
- Open `Cargo.lock` and grep all `name = "reqwest-middleware"` and `name = "reqwest_middleware"` (note the underscore variant) blocks to enumerate all coexisting versions.
- The skew is most likely: workspace pin `0.5.1` is incompatible with one transitive consumer (e.g. an HTTP middleware that internally pulls `0.4.x` API). The fix is usually to relax the workspace pin to `^0.4` if the consumer demands old API, or bump the consumer if it has a new release.
- If you cannot resolve by reasoning alone (i.e. the skew involves a closed-source or locked transitive), raise a `kind: "clarify"` DQ entry naming the exact two packages and their version-incompat error from the build log.

**File-locality:** `Cargo.toml` (root), `crates/api/api_crud/Cargo.toml`, `Cargo.lock`. No overlap with C1, C2, C3.

**Mid-task DQ push:** dep-skew mystery → `kind: "clarify"`, `from: "impl"`, push, continue.
