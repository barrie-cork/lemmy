---
name: §15 dry-run catches YAML parse but misses invalid cargo flags inside YAML run: bodies
description: Plan §15 dry-run discipline (yamllint / gh workflow view) catches YAML parse errors but cannot catch invalid cargo flags inside the YAML's `run:` body — first-push validation is the real gate.
type: feedback
---

The plan-side §15 dry-run discipline (per `feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`) catches YAML structural errors and missing files, but cannot catch invalid cargo flags embedded as shell strings inside a `run:` body. End-to-end validation requires a real first-push run on the GitHub-hosted runner.

## Concrete failure

v1-validate-agent plan §13 Task 1 IMPLEMENT block prescribed:

```yaml
- name: cargo check workspace + features full
  run: cargo check --workspace --features full --no-deps
```

The plan's §15.1 dry-run shape was `gh workflow view --yaml ...` exit 0 OR `yamllint <path>` exit 0. Both pass — the YAML is well-formed, the file path is valid. But `cargo check` does NOT accept `--no-deps`; that flag is `cargo clippy`-only. The plan inherited `--no-deps` from clippy DoD wording in earlier phases where it was valid.

First push to `phase-v1-validate-agent` failed at workflow run `25017407659` after ~30 seconds with:

```
error: unexpected argument '--no-deps' found
```

## Why this is a class of failure, not a one-off

Plan §15 dry-run validates:
- YAML syntactic structure (parse-able)
- File existence at the path
- Tool availability (`gh workflow view --help`, `yamllint --version`)

Plan §15 dry-run does NOT validate:
- The semantic correctness of any shell command nested inside a YAML `run:` body
- Whether the cargo subcommand accepts the flags passed to it
- Whether environment-dependent behaviour (toolchain components, cached dependencies, libpq discovery) actually works on the runner

The semantic checks are runtime concerns — they only surface on a real workflow run.

## How to apply

For Shape G plans (workflow-driven validation):

1. **Plan-side dry-run remains necessary** — yamllint / gh workflow view exit 0 catches the bulk of malformed-YAML errors before push.
2. **The first push to the phase branch is the real gate.** Treat it as a probe: expect failure, capture the failure mode, fix-and-push.
3. Plan-write should **explicitly call out** that DoD validity for cargo flag combinations is unverifiable until first push. Phrase §15 entries as "expected to pass on first push; if not, fix-up commit lands before the next task."
4. Never hand-derive flag combinations from clippy DoD wording without verifying the flag exists for the subcommand. `cargo check`, `cargo clippy`, `cargo build`, `cargo test`, `cargo doc` all have distinct flag sets despite sharing many.

## Generalises to

Any DoD command that's a shell string nested inside a configuration file the planner doesn't actually execute at plan-write time:
- GitHub Actions workflow `run:` bodies
- Docker `RUN` lines in a Dockerfile
- shell stanzas in CI YAMLs (CircleCI, GitLab CI, Buildkite)
- npm `scripts:` entries the planner doesn't run via `npm run`
- Makefile recipes referenced by name

The dry-run validates the wrapper (the YAML / Dockerfile / Makefile parses) but cannot validate the wrapped command (the shell string is correct).

## Symptom to recognise

A plan ships, the phase branch is cut, the first impl-task push triggers the workflow, and the workflow fails ~30 seconds in with a flag-parsing error from cargo / docker / npm / make. The plan's §15 dry-run output was clean. The §16a Story 1 acceptance fails on first push but passes after a fix-up commit.

This is not a planner-quality problem — it's an inherent limitation of plan-side dry-run for nested-shell DoD. The retro should treat first-push fix-ups as routine, not as planner failures, and capture the specific class of fix as a watch-item for the next sub-phase under the same shape.

## Where this came from

DQ #69 (impl-self-resolved 2026-04-27, v1-validate-agent). Fix-up commit `ed049970b` dropped `--no-deps` from the cargo check step. The `--no-deps` flag remained on the cargo clippy step where it's valid and intentional (per `feedback_features_full_p_crate_incompatible.md`).
