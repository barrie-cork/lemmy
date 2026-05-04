<img src="https://r2cdn.perplexity.ai/pplx-full-logo-primary-dark%402x.png" style="height:64px;margin-right:32px"/>

# Search for best practices when building with rust and using Pi Coding agent CLI

There's no dedicated "Pi + Rust best practices" repository that has emerged as a community standard yet. Drawing from the Pi extension ecosystem, the [badlogic/pi-mono](https://github.com/badlogic/pi-mono) agent's design, your Lemmy codebase, and Rust AI-agent patterns observed across community repos, here are the consolidated best practices .

## Rust-Specific Pi Agent Best Practices

The core challenge with Pi + Rust is that Rust's compile-time feedback loop is **slower and more verbose** than most other languages — `cargo check` on a cold workspace takes seconds, `cargo build` takes tens of seconds, and compiler errors are detailed multi-line messages that aggressively consume Pi's context window. Every practice below is shaped by this reality .

***

## Project Setup

**1. Write a precise `CLAUDE.md` / `GEMINI.md` at the repo root** — Pi reads these files as the agent's first context, before any conversation . For a Rust project, this file should contain:

- Exact Cargo commands (`cargo check --workspace`, `cargo test --workspace`, `cargo clippy -- -D warnings`)
- Workspace crate map — what each crate in `crates/` is responsible for
- Coding conventions (e.g., error handling style, async runtime used, ORM conventions)
- Which tests to run and how (`cargo test -p lemmy_api`, `cargo test --test api_tests`)

**2. Pin the Rust toolchain explicitly** with a `rust-toolchain.toml` so Pi never guesses the correct edition or edition-specific syntax . Your Lemmy fork already does this well.

**3. Keep `bacon.toml` configured** for background compilation — you can run `bacon` in a `pi-tmux` pane and let Pi see a live error stream without spawning redundant `cargo check` calls .

***

## Context Management (Most Critical)

Pi uses a context window; Rust compiler output is one of the worst offenders for flooding it .

**4. Install `pi-rtk-optimizer` and enable `filterBuildOutput`** — this compacts `cargo build` and `cargo clippy` output to errors/warnings only, stripping thousands of lines of "Compiling crate X..." noise .

**5. Prefer `cargo check` over `cargo build` for validation loops** — instruct Pi explicitly: "run `cargo check --workspace` first; only run `cargo build` when I confirm check passes." `check` is 5–10× faster and produces the same error output .

**6. Scope test runs to the relevant crate.** Instead of `cargo test --workspace` on every change, tell Pi to run `cargo test -p <affected_crate>`. Full workspace tests are for pre-commit validation, not iterative development .

***

## Prompting Patterns

**7. Provide the complete error message, not a paraphrase.** Rust errors include lifetime annotations, trait bounds, and suggested fixes — Pi needs the full compiler output to reason correctly. Use `/paste` (from `tmustier/pi-extensions`) to paste multi-line error dumps without context pollution .

**8. Scope each task to a single crate boundary.** Pi works best when the task is: "implement X in `crates/api/src/lib.rs`." Asking it to simultaneously modify `crates/api/`, `crates/db_schema/`, and `crates/federation/` in one prompt without using `pi-goal` (parallel agents) leads to confused context and incomplete changes .

**9. Explicitly tell Pi the error handling convention.** Rust has many error handling patterns (`thiserror`, `anyhow`, `?` propagation, `Result<_, LemmyError>`). State this in `CLAUDE.md` and repeat in prompts when switching crates .

**10. For ownership/lifetime errors, give Pi the full struct definitions.** These errors are almost always context-dependent — Pi needs to see the owning types, not just the error site .

***

## Workflow Automation

**11. Use `context-workflow` for new feature development**, not for debugging. The automated write → test → review → fix loop works well for greenfield Rust code where `cargo test` is the acceptance gate . For debugging an existing compile error, that loop is too heavy — prompt directly instead.

**12. Stage migrations separately from Rust code changes.** Diesel migration files (`migrations/*/up.sql`) should be committed and applied **before** Pi writes the corresponding Rust schema types. Pi gets confused if it's generating schema types for a migration that doesn't exist yet in the database .

**13. Run `cargo fmt --check` as a Pi post-step hook** by including it in your `CLAUDE.md` conventions. Ask Pi to always run format checks after it finishes a code block — Rust's formatter is strict, and misformatted code triggers CI failures that waste subsequent agent turns .

***

## Multi-Session / Multi-Crate

**14. Use `tab-status` + `session-recap` for parallel Lemmy crate work.** When running two Pi sessions (e.g., one for `crates/api/` and one for `crates/db_schema/`), `tab-status` shows ✅/🚧 per tab and `session-recap` gives a one-line summary on refocus .

**15. Commit working states frequently before giving Pi large refactor tasks.** Unlike Python or JS where you can undo with ctrl-Z, Rust type errors from a partially-applied refactor can cascade across crates and make it hard to recover without reverting . A clean Git state gives Pi a rollback point.

***

## Cargo-Specific Tips for Pi

| Situation | Command to give Pi |
| :-- | :-- |
| Fast syntax/type check | `cargo check -p <crate>` |
| Run only unit tests | `cargo test -p <crate> --lib` |
| Run integration tests | `cargo test --test api_tests` |
| Check all lints | `cargo clippy --workspace -- -D warnings` |
| Format check | `cargo fmt -- --check` |
| Expand macros for debugging | `cargo expand -p <crate> <module>` |
| Dependency audit | `cargo audit` |

The single highest-ROI practice is **a well-structured `CLAUDE.md`** — it front-loads context Pi needs for every turn, eliminates repeated re-explanation, and ensures Pi defaults to your project's specific Cargo conventions rather than generic Rust patterns .

