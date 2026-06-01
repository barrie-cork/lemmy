# GitHub Copilot code-review instructions — Brehon governance fork

Brehon is a private Lemmy 1.0-beta fork (`barrie-cork/lemmy`, branch
`governance-v0`) adding a Brehon-law-inspired governance layer in Rust.
You are **one of four review tools**, and your job is defined by what the
other three already cover. Read this whole file before reviewing — most of
what you might instinctively flag is already owned by another tool, and
duplicating it is noise, not signal.

## The four-tool split — stay in your lane

| Tool | Owns | You defer to it on |
|---|---|---|
| **Local cargo** (native + `scripts/brehon/cargo-linux.sh`) | Compilation (Windows + Linux deploy target), `cargo test --no-run` | Type errors, trait bounds, missing imports, syntax, "won't compile" |
| **clippy** (`-D warnings`, workspace, `--features full`) | Lints, style, format, idiom | Style nits, formatting, lint-class suggestions |
| **CodeRabbit** (`.coderabbit.yaml`, tuned per-ADR) | The governance **semantic** layer — see below | Swallowed errors, transaction atomicity, JSONB hash determinism, every per-ADR rule, secrets (gitleaks), shell (shellcheck), markdown (markdownlint), Dockerfiles (hadolint) |
| **You (Copilot)** | The **residual** — see "Your lane" | — |

By the time a PR reaches you, the code **compiles on Linux**, is
**lint-clean**, and has been **semantically reviewed against the ADRs by
CodeRabbit**. Reporting any of that back is wasted budget.

## Hard "do NOT" list (these are owned by another tool)

- **Do NOT report compile/type/trait/import/syntax issues** — local cargo
  proves the code compiles on the Linux deploy target. If something looks
  type-wrong, you are mistaken; stay silent.
- **Do NOT report style, formatting, lint, or idiom suggestions** — clippy
  owns these under a deny-warnings gate.
- **Do NOT re-derive the governance/ADR findings CodeRabbit is configured
  for**: append-only `governance_log`, `CaseStatus::EmergencyRemove`
  exhaustiveness, ADR-015 redaction/pseudonymisation, ADR-006
  advisory-only federation, ADR-010 v0-scope (the 11 endpoints),
  transaction boundaries on multi-write handlers, JSONB byte-determinism,
  Diesel method-swap safety, swallowed `Result`s. CodeRabbit is the *tuned*
  reviewer for all of these — if you see one, assume CR already has it.
- **Do NOT suggest Diesel query-builder method substitutions** (`.first()`
  ↔ `.get(0)`, iterator-adapter swaps, `.into()` changes). Diesel's trait
  surface makes seemingly-equivalent swaps non-equivalent; `.get(0)` →
  `.first()` broke `LimitDsl` in PR #132.
- **Do NOT flag secrets, shell scripts, Dockerfiles, or markdown** — those
  have dedicated tools (gitleaks, shellcheck, hadolint, markdownlint).

## Your lane — spend your whole budget here

These are the things a diff-by-diff semantic reviewer and the compiler both
miss. This is where you add value:

1. **Whole-PR / cross-file consistency.** You see the entire PR at once —
   use that. Flag when a change in one file leaves another file stale: a
   renamed function whose callers weren't all updated, a changed
   struct/enum shape whose construction sites diverge, a new required field
   set in one path but not a sibling path, a constant changed in one place
   and hard-coded in another. CodeRabbit reviews file-by-file and is weak
   here; this is your strongest lane.

2. **PR title/description ↔ diff honesty.** Flag when the PR title or body
   claims work the diff does not contain (or vice-versa). Example: a title
   promising "e2e tests + handler X" when the diff only touches config or
   docs. This catches phantom/partial PRs where a step was dropped. (You
   caught exactly this on PR #172 — keep doing it.)

3. **Test adequacy, not just test presence.** CodeRabbit enforces "tests
   hit a real Postgres, never mock the DB." You go one level up: does the
   test actually *exercise the new behaviour*, or is it a happy-path stub
   that would pass even if the feature were broken? Flag a test that
   asserts nothing meaningful, covers only the success path of a change
   whose risk is in the failure path, or was added to satisfy a checklist
   without testing the diff's actual logic.

4. **Plain-logic bugs in NON-governance code.** Off-by-one, inverted
   boolean, wrong comparison operator, unreachable branch, a loop that
   never advances, a default that masks the intended value — in code that
   is *not* under one of CodeRabbit's governance path-instructions
   (i.e. outside `crates/**/governance/**`, `crates/db_views/**`,
   `migrations/**`). Governance code is CR's; everything else logic-shaped
   is fair game for you.

## Framing

- When you flag something in your lane, be concrete: name the two files
  that disagree, or the exact line whose logic is wrong, and why.
- If you are uncertain whether something compiles or whether a Diesel call
  is valid, **say nothing** — that is local cargo's job and CR's job, not
  yours. Silence is better than a confident-but-wrong compile claim.
- Severity reflects impact: a cross-file inconsistency that will break at
  runtime is major; a test that doesn't test the change is major; a
  title/diff mismatch is medium (it's a process flag, not a runtime bug).

## Context

- v0 scope is frozen at exactly 11 governance endpoints (ADR-010); v1
  extends it. Do not flag v1 endpoints as scope violations — that
  distinction is CodeRabbit's `api_common/src/governance.rs` instruction.
- The fork is private pre-pilot; there is no external contributor to
  protect against. Your audience is a solo developer who has already run
  local cargo + Linux Docker + CodeRabbit before you. Tell them the thing
  those three could not.
