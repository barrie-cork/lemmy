---
name: dtolnay/rust-toolchain ignores `components:` input when rust-toolchain.toml pins a channel
description: When a repo has rust-toolchain.toml pinning a specific Rust channel, the `components:` input on dtolnay/rust-toolchain@master is silently ignored. Add an explicit `rustup component add` step.
type: feedback
---

Empirical finding from v1-validate-agent Task 1 (2026-04-27): the `dtolnay/rust-toolchain@master` GitHub Action does NOT honour the `components:` action input when the repository has a `rust-toolchain.toml` file pinning the channel.

## What fails

Workflow YAML using the action's intended pattern:

```yaml
- uses: dtolnay/rust-toolchain@master
  with:
    components: clippy
```

When `rust-toolchain.toml` exists and contains:

```toml
[toolchain]
channel = "1.95"
```

The action installs the pinned `1.95` toolchain (correct) but does NOT install `clippy` (incorrect). Subsequent `cargo clippy` steps fail with:

```
error: 'cargo-clippy' is not installed for the toolchain '1.95-x86_64-unknown-linux-gnu'.
       To install, run `rustup component add clippy --toolchain 1.95-x86_64-unknown-linux-gnu`.
```

## Why it slips through

The `components:` input is documented as the canonical way to add components to the installed toolchain. Standard cache + checkout shape from existing workflows (e.g. `cargo-test-e2e.yml`) doesn't reveal the bug if those workflows don't actually invoke clippy. v1-validate-agent's `cargo-validate-workspace.yml` was the first Brehon workflow to need clippy on the pinned `1.95` toolchain, and the bug surfaced on first push.

## How to apply

For any workflow YAML that invokes `cargo clippy` (or any other component beyond `rustc` + `cargo`) on a repo with `rust-toolchain.toml` pinning a channel:

1. Use `dtolnay/rust-toolchain@master` for the channel install (it reads `rust-toolchain.toml` correctly).
2. **Add an explicit `rustup component add` step** that derives the toolchain dynamically from `rustup show active-toolchain` — this tracks `rust-toolchain.toml` automatically across channel bumps and runner-target changes:

```yaml
- uses: dtolnay/rust-toolchain@master
  with:
    components: clippy   # NB: silently ignored when rust-toolchain.toml exists; kept for documentation

- name: Install clippy for active toolchain
  run: |
    toolchain="$(rustup show active-toolchain | awk '{print $1}')"
    rustup component add clippy --toolchain "$toolchain"
```

The redundant `with: components:` line is a documentation marker for future readers; the `rustup component add` line is what actually installs.

**Anti-pattern to avoid:** hard-coding the toolchain version + triple in the `rustup component add` line (e.g. `rustup component add clippy --toolchain 1.95-x86_64-unknown-linux-gnu`). This breaks silently when `rust-toolchain.toml` is bumped to 1.96 (clippy installs into the wrong toolchain; the actual active one still has none) or when the runner target changes (e.g. `aarch64-unknown-linux-gnu` for arm64 runners). Always derive from `rustup show active-toolchain`. CR finding cr-4 on PR #104 caught the hard-coded form.

## Generalises to

- `rustfmt` for `cargo fmt --check` workflows.
- `rust-src` for cross-compiled or `-Z` flag work.
- Any other `rustup` component the action can technically install but won't when a `rust-toolchain.toml` pin is present.
- The same pattern likely affects competing actions (`actions-rs/toolchain`, `actions-rust-lang/setup-rust-toolchain`) when they read a toolchain file — verify on first use.

## Symptom to recognise

CI fails on the first cargo invocation that uses the missing component, with a clear `'cargo-X' is not installed for the toolchain` error. The `with: components:` line in the YAML appears correct on inspection. Searching for the bug in the action's repo issues will surface it (this is a known behaviour, not a regression).

## Where this came from

v1-validate-agent commit `000f86093` (Task 2 fix-up). Workflow run `25017554049` failed with the missing-clippy error after `cargo check` passed; the fix added the explicit `rustup component add` step. The plan's §13 Task 1 IMPLEMENT block did not anticipate this — surfaced for retro §5 watch-items as a planning-side miss.
