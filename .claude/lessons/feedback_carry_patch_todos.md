---
name: Carry-patch TODO comment format
description: When patching upstream Lemmy code to make it work in the Brehon fork, use a specific TODO form that leaves a PR placeholder for later fill-in
type: feedback
originSessionId: d8c30b13-ac92-4629-baff-664040b47be1
---
When touching upstream Lemmy code with a local carry-patch (a fix we maintain until it's upstreamed), the `TODO` comment inside the patched block must use this exact form:

```rust
// TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___
```

Leave `#___` blank. The user fills in the real PR number after filing it upstream. Do NOT invent a number, do NOT use `TBD`, do NOT drop the `#___` placeholder entirely.

The comment above the TODO should explain *why* the patch exists in one or two lines (e.g. "upstream Lemmy assumes a Unix host; `tokio::signal::unix` does not exist on windows-msvc"). The TODO itself stays short — the explanation belongs above it.

Also prefer `#[cfg(not(windows))]` over `#[cfg(unix)]` when gating Unix-only code, because it is more defensive — future non-Unix targets (WASI, Redox, etc.) still get the Unix-style behaviour unless they explicitly opt out.

**Why:** Carry-patches must be easy to spot during the weekly `upstream/main` rebase so we can cherry-pick them into an upstream PR and then delete them from our tree. A consistent `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` string is greppable, reviewable, and obviously a placeholder waiting to be filled in.

**How to apply:** Any time you edit upstream Lemmy source files (anything under `crates/` that is NOT in a new `governance/` subdirectory), ask yourself whether it is a carry-patch. If yes, commit it as a standalone `fix(...)` or `chore(...)` commit with a clear subject, include the TODO-with-placeholder above the patched block, and leave the PR number blank. Brehon-original governance code does not need this comment — it has nothing to upstream.
