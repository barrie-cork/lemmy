# Pi Project Context — Lemmy/Brehon

This repo is an application codebase: a governance-enabled fork of Lemmy 1.0-beta. Use pi for normal code-repo tasks unless the user explicitly asks for Claude/Junior orchestration work.

## Non-negotiable Brehon constraints

From `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`, existing ADRs are append-only. Contradicting them requires a new ADR, not a quiet edit.

- Lemmy 1.0-beta fork; Extism plugin host for governance hooks.
- AGPLv3 inherited; releases must honour source-disclosure notice.
- v0 scope is exactly the 11 endpoints in `docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md` §2.
- v0 simplifications: 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub.
- Solo-dev stack: no Keycloak, OpenFGA, Vault, Kubernetes, external log signer, or blockchain anchoring.
- Auth uses Lemmy's existing JWT; optional passkey MFA via `webauthn-rs`.
- Authz is hardcoded capability checks in Rust reading `reputation_snapshot` flags.
- Governance log uses `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0.
- GDPR from day 1: pseudonymised `actor_pseudonym` table mandatory.
- Illegal content from day 1: `CaseStatus::EmergencyRemove` mandatory.
- Federation: vanilla Lemmy content federation remains compatible; governance signals are fork-only AP types.

If a requested plan contradicts these, stop and surface it to the user.

## Coding workflow constraints

- Never write Rust code without a plan file in `.claude/PRPs/plans/`.
- Planning and implementation are separate phases.
- Prefer reading canonical design docs before changing specs or API behaviour:
  - `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
  - `docs/brehon-law-inspired-network/04-data-model-and-api.md`
  - `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`
- For crate layout, slash-command history, and repo-specific references, read `.claude/brehon-reference.md` on demand.

## Dual-harness boundary

The `.claude/` directory is still used for Claude Code and custom Junior orchestration. Do not delete, rename, or simplify it as part of pi work unless the user explicitly asks.

For pi sessions:

- Treat Claude/Junior orchestration docs as optional reference material, not mandatory foreground workflow.
- Use `.claude/rules/*.md` only when a task clearly touches the rule's topic.
- Do not invoke or emulate Junior subagents unless explicitly requested.
- Do NOT read root `CLAUDE.md` unless explicitly asked. `AGENTS.md` is the pi entry point; pi prefers it over `CLAUDE.md` (verified 2026-05-04).

## Recursive Learning System parity for pi

Pi participates in the same RLS artifact loop as Claude Code where the surfaces overlap:

- Session retros and command retros write under `.claude/PRPs/reports/`.
- Durable lessons write under `.claude/lessons/feedback_*.md` or `.claude/lessons/reference_*.md` with YAML frontmatter.
- `.pi/extensions/lemmy-hooks.ts` runs a PMD HTTP reachability guard at session start.
- After pi `edit`/`write` calls to lesson files, the extension runs:
  - `.pi/hook-scripts/lesson-frontmatter-reminder.sh`
  - `.pi/hook-scripts/lesson-pmd-sync.sh`
- Use `scripts/brehon/pmd-query.sh` and `scripts/brehon/pmd-write.sh` for explicit PMD access from pi/headless contexts.
- Use `/brehon-mode harness-maintenance` for explicit skill/lesson/retro/harness metadata work; it permits `.claude/skills/`, `.claude/lessons/`, `.claude/PRPs/reports/`, and focused `.pi/` harness paths while still blocking app code.

PMD topology differs by repo/machine. For this Brehon repo, the EliteDesk checkout uses HTTP PMD at the P50/Windows laptop Tailscale endpoint (`http://100.104.171.26:11435/mcp`) with a bearer token from `/srv/brehon-fork/.env`; loopback `http://localhost:11435/mcp` is used when running on the PMD host. Do not assume a per-worktree SQLite file is live unless the relevant PMD invariant/doc says so.

## Pi session Rust quick-reference

Pi sessions optimise for short context windows and mechanical loops. The four-role
Junior model in `.claude/` does not apply here — but the cargo wrappers under
`scripts/brehon/` and the lessons under `.claude/lessons/` ARE shared infrastructure
and should be used.

### Cargo commands (always via wrappers, never raw `cargo`)

The wrappers under `scripts/brehon/cargo-*.sh` (mac/Linux) and `.bat` (Windows)
print `TOOLCHAIN_OK` + version, set repo root, and pass `$@` through. They exist
so output capture and toolchain pinning are uniform across both harnesses.

| Task                          | Command                                                                  |
| :---------------------------- | :----------------------------------------------------------------------- |
| Fast type-check, one crate    | `scripts/brehon/cargo-check.sh -p <crate>`                               |
| Type-check workspace + feats  | `scripts/brehon/cargo-check.sh --workspace --features full`              |
| Clippy, one crate             | `scripts/brehon/cargo-clippy.sh -p <crate> --no-deps -- -D warnings`     |
| Clippy, workspace             | `scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings` |
| Unit tests, one crate         | `scripts/brehon/cargo-test.sh -p <crate> --lib`                          |
| e2e integration tests         | `scripts/brehon/cargo-test.sh --test e2e -p lemmy_server`                |
| Format check                  | `cargo fmt -- --check` (no wrapper yet; add only if churn justifies)     |

Rules:

- **Always pass scope flags** (`-p <crate>` or `--workspace`). The wrappers
  intentionally do NOT default to `--workspace` — see
  `.claude/lessons/feedback_wrapper_script_flag_silence.md`.
- **Never combine `-p <crate>` with `--features full`** unless the crate
  defines `full` itself (see `feedback_features_full_p_crate_incompatible.md`).
- **Always `--no-deps` on clippy** to suppress external-crate noise.
- **Prefer `cargo check` to `cargo build`** during iteration; `build` only on
  user request or pre-PR.
- **Heavy validation belongs on GH Actions** (Shape G) not the laptop — see
  `.github/workflows/cargo-validate-*.yml`.

### Output discipline (critical for pi context windows)

Per `.claude/rules/no-cargo-output-paste.md`: do NOT paste raw cargo output
into the conversation. The wrappers + `pi-rtk-optimizer` filter noise, but the
real win is to redirect to a log and read only the tail:

```bash
scripts/brehon/cargo-check.sh -p lemmy_api > .pi/cargo-check-lemmy_api.log 2>&1
# then: read tail-50, or grep for "error\[" / "warning:"
```

For deep error inspection, open the log via `/readfiles .pi/<file>.log` rather
than echoing it to chat.

### Error-handling convention

Brehon code uses `LemmyResult<T>` (alias for `Result<T, LemmyError>`) end-to-end.
New code should:

- Return `LemmyResult<T>` from public fn signatures.
- Propagate with `?`; convert foreign errors via `From` impls already in
  `crates/utils/src/error.rs`.
- Never `unwrap()` or `expect()` outside tests — surface via `LemmyError` so
  the API layer renders a structured response.

For test fixtures, follow the pool/conn/`LemmyResult` pattern in the
`test-write` skill (`.claude/skills/test-write/SKILL.md`).

### Pi-specific tactics for Rust pain points

- **Borrow / lifetime errors**: open the full owning struct with `/readfiles
  crates/<crate>/src/<file>.rs` BEFORE proposing a fix. Lifetime errors are
  context-dependent; a paraphrased error site is rarely enough.
- **Trait-bound errors**: read the trait definition AND every `impl` for the
  concrete type. `rg "impl .* for <Type>"` first, then read the hits.
- **Schema/migration changes**: commit and apply the Diesel migration BEFORE
  generating Rust schema types. The two must round-trip; a half-applied
  migration produces phantom errors.
- **Refactors crossing crate boundaries**: stop and ask the user for a plan
  file under `.claude/PRPs/plans/`. The "no Rust without a plan" rule applies
  to pi sessions too — that's a Brehon hard constraint, not a Claude-only one.

### Four-role advisor pattern (default working pattern)

Pi sessions on the Brehon fork are the **advisor** role in the four-role
model (`advisor / planning / impl / bm`). The full model lives in
`CLAUDE.md` "Four-role model" + `.claude/rules/advisor-orchestrator.md`.
The pi harness mirrors the model here so it does not need to load CLAUDE.md
on every session.

| Role | Runs on | Model | Authors |
|---|---|---|---|
| **advisor** (this session) | laptop | opus-4-7 | briefs, lessons, retros, ADRs, harness config — NEVER plan/code/PR content |
| **planning** | EliteDesk Junior daemon | opus-4-8 | plan files at `.claude/PRPs/plans/<phase>.plan.md` |
| **impl** | EliteDesk Junior daemon (lane worktree) | sonnet-4-6 | Rust code, migrations, schema regen |
| **bm** | EliteDesk Junior daemon | haiku-4-5 | git/PR/CR lifecycle, branch cuts, merges |

**What the advisor does (the rules):**

- When the user says "begin planning X" / "plan X" / "draft a plan for X":
  1. Author the brief at `.claude/PRPs/briefs/<phase>-planning-1.md` (advisor's own work).
  2. Run `/brehon-clarify <brief-path>` if scope is non-trivial.
  3. Queue `[role:planning] <slug> — see .claude/PRPs/briefs/<file>.md` on Junior.
  4. Poll. When planning lands: §3.4 DoD smoke test + §3.5 watchpoint gate, surface to user (gate 1: plan approval).
  5. After approval: queue `bm-cut` to create the phase branch. **NEVER write the plan yourself.**
- When the user says "implement Y" / "make change Y": verify a plan exists at `.claude/PRPs/plans/<phase>.plan.md`; if not, route back to planning. Author the impl-task brief (4 sections, §2.4 lesson injection, §2.4a ADR load-bearing clause). Queue `[role:impl-task]`. Poll. Run §3.9 verify gate. Surface.
- Briefs commit on `governance-v0` (planning/bm briefs) or on the phase branch (impl-task briefs, Mode A). Always reachable at the ref the worker forks from.

**What the advisor does NOT do (the rule that was violated in m2-late-2 planning, 2026-06-10):**

- ❌ Author plan files at `.claude/PRPs/plans/<phase>.plan.md` directly.
- ❌ Author Rust code at `crates/**/src/**/*.rs`, migrations, `crates/db_schema_file/src/schema.rs`.
- ❌ Open PRs / cut branches / merge directly (use `[role:bm-task]`).
- ❌ Commit source-code changes authored in the advisor CWD.

**What the advisor MAY author (the exceptions):**

- ✓ Briefs at `.claude/PRPs/briefs/<phase>-<role>-<n>.md`.
- ✓ Lessons at `.claude/lessons/{feedback,reference}_*.md` (cross-harness corpus).
- ✓ Retros at `.claude/PRPs/reports/<phase>-retro.md`.
- ✓ Verify reports at `.claude/PRPs/reports/<phase>-verify.md`.
- ✓ ADRs at `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` (gate 2 / judgment-heavy DQ applies).
- ✓ Handover files at `.claude/PRPs/handovers/<phase>-<scope>-<date>.md` (pre-compact discipline).
- ✓ Harness config: `AGENTS.md`, `.pi/PROJECT_CONTEXT.md`, `.pi/extensions/*.ts` (this file is itself such a write).

**The full rule + failure-case writeup:** `.claude/lessons/feedback_pi_advisor_role_dispatches_to_junior.md`.

### What pi sessions do NOT do (other than four-role violations)

- Do not write to `.claude/decision-queue.json`, `.claude/runlog/`, briefs,
  retros, or Junior daemon state directly — these are owned by the four-role
  model. The advisor's writes to these files go via the Junior dispatch
  surface, not direct file edits (the only exception is the lesson corpus,
  which is harness metadata).
- Do not auto-promote anything to user-scope (`~/.claude/` or `~/.pi/`).
- Do not enable `context-workflow`, `pi-goal`, or `pi-ralph-wiggum`-style
  autonomous loops without explicit user approval per turn.

## Project subagents — delegate, don't load

The main pi session is intentionally slim on GitHub Actions detail and
BM verb mechanics. Two project-scope subagents live in `.pi/agents/`
with focused harnesses:

| Agent | When to delegate |
|---|---|
| `ci-debug` | Any work on `.github/workflows/*.yml` or `.github/scripts/*.sh`; debugging a failing workflow run; designing a new gate. The agent reads `.claude/lessons/feedback_gha_pi_loop_postmortem.md` first — six load-bearing GH Actions facts the main session does not need. |
| `bm-pi` | Any Brehon Branch Manager verb (bm-cut, bm-pr, bm-poll-cr, bm-merge, etc.). The agent reads `.pi/skills/bm-task/SKILL.md` + `.claude/rules/branch-manager.md` first — the BM contract is large, slim main-session context stays clean. |

**Invocation convention:** call the `subagent` tool with
`agentScope: "both"` so project-local agents in `.pi/agents/` are
discovered alongside user-level agents:

```
subagent({ agent: "ci-debug", task: "<one-line task>", agentScope: "both" })
```

Without `agentScope: "both"` the call defaults to user-scope only and
the project agents above are invisible. The first interactive
invocation per session prompts the user to confirm project-agent use
(`confirmProjectAgents`); subsequent calls in the same session reuse
the confirmation.

**Pre-requisite (one-time, user-scope):** the `subagent` tool itself
is loaded by an extension that lives at user scope:

```
mkdir -p ~/.pi/agent/extensions/subagent
ln -sf /opt/homebrew/lib/node_modules/@mariozechner/pi-coding-agent/examples/extensions/subagent/index.ts ~/.pi/agent/extensions/subagent/index.ts
ln -sf /opt/homebrew/lib/node_modules/@mariozechner/pi-coding-agent/examples/extensions/subagent/agents.ts ~/.pi/agent/extensions/subagent/agents.ts
mkdir -p ~/.pi/agent/agents
for f in /opt/homebrew/lib/node_modules/@mariozechner/pi-coding-agent/examples/extensions/subagent/agents/*.md; do
  ln -sf "$f" ~/.pi/agent/agents/$(basename "$f")
done
```

Without this install the project agents in `.pi/agents/` exist as
ready-to-use definitions but cannot be dispatched. Install once per
machine.

**Why this shape:** previous CI-debug iterations on this repo loaded
GitHub Actions detail directly into the main pi session, where it
lived for the whole session window without ever being needed for the
80% of work that isn't CI. The 2026-05-06 adr-compliance loop also
showed a failure mode where the main session anchored on a wrong
premise (`GITHUB_EVENT_NUMBER`) and self-authored a memory note
making it canonical for future sessions. A subagent gets a fresh
window with the lesson loaded as ground truth and exits when done —
no premise leaks into the main session's memory.

## Setup decisions log (do not re-litigate)

Dual-harness baseline locked in 2026-05-04. Future pi sessions: take these as
given; surface a new ADR-style note if you genuinely need to revisit them.

| Decision | Rationale | Reference |
| :--- | :--- | :--- |
| `AGENTS.md` is the pi entry point at repo root; `CLAUDE.md` is Claude Code's | Empirical probe confirmed pi loads `AGENTS.md` and ignores `CLAUDE.md` when both exist at the same root. Clean dual-harness isolation, no `--no-context-files` workaround needed. | Commit `db413f87c`; phd-vault `PI_QUIRKS.md §17` |
| `.pi/hook-scripts/` (not `.pi/hooks/`) | Pi renamed hooks to extensions; a literal `.pi/hooks/` dir triggers a startup warning regardless of contents. | Commit `db413f87c` |
| Pi auto-commit per edit is disabled | Pi edits remain in the working tree. Follow the existing Brehon workflow for explicit review, `git add`, commit grouping, and push/PR actions. Historical `auto(pi): update <basename>` commits may exist before 2026-06-10; they are no longer produced after the extension reloads. | `.pi/extensions/lemmy-hooks.ts` `tool_result` handler |
| `raw-paste` extension enabled user-scope | Lets `/paste` arm a one-shot raw paste so multi-line Rust compiler errors stay editable. Not in repo — lives in `~/.pi/agent/settings.json`. | User-scope only |
| Cargo work goes through `scripts/brehon/cargo-*.sh` wrappers | Uniform toolchain pinning + output capture across both harnesses; matches `.claude/rules/no-cargo-output-paste.md` discipline. | See cargo table above |
