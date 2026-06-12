# CLAUDE.md — Brehon Fork (advisor CWD)

```yaml
project:
  fork_of: "LemmyNet/lemmy"            # Lemmy 1.0-beta base; rebase SHA in git log
  working_branch: "governance-v0"      # main = upstream-sync rebases only
  rust_toolchain: "1.95"
  license: "AGPL-3.0"
  cwd_role: "persistent advisor session"
  non_brehon_ops_cwd: "C:/Users/barri/Developer/homeserver"
# active_sub_phase / phase_branch / trunk_sha / last_shipped: derive from git, do not hardcode here
```

## Hard constraints (do NOT re-litigate)

From `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`, the 15 ADRs are append-only. Contradicting them requires a new ADR, not a quiet edit.

- **Lemmy 1.0-beta fork**; Extism plugin host for governance hooks (ADR-012)
- **AGPLv3** inherited (ADR-011) — every release honours the source-disclosure notice
- **v0 scope = exactly the 11 endpoints** in [05 §2](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md); nothing else
- **v0 simplifications** (mandatory): 5-juror panels, quorum 3, simple majority, outbound-only federation, local hash chain, reputation-decay stub
- **Solo-dev stack:** NO Keycloak, NO OpenFGA, NO Vault, NO Kubernetes, NO external log signer, NO blockchain anchoring
- **Auth:** Lemmy's existing JWT; optional passkey MFA via `webauthn-rs`
- **Authz:** hardcoded capability checks in Rust reading `reputation_snapshot` flags
- **Governance log:** `sha2` hash chain via Postgres triggers; `rs_merkle` + `ed25519-dalek` for signing; key in `.env` for v0
- **GDPR from day 1:** pseudonymised `actor_pseudonym` table mandatory (ADR-015)
- **Illegal content from day 1:** `CaseStatus::EmergencyRemove` mandatory (ADR-013)
- **Federation:** content-level with vanilla Lemmy works; governance signals are fork-only AP types (ADR-014)

If a plan contradicts any of these, STOP and surface to the user. Do not silently fix in the plan body.

## Four-role model

Advisor = meta-oversight only, never authors content. Junior workers run on EliteDesk (`ssh homeserver`, daemon `junior@brehon-fork`, repo at `/srv/brehon-fork`).

```yaml
roles:
  advisor:  { runs_on: "laptop (this session)", model: "opus-4-7",  task_prefix: null,            authors_content: false }
  planning: { runs_on: "EliteDesk/Junior",      model: "opus-4-8",  task_prefix: "[role:planning]"  }
  impl:     { runs_on: "EliteDesk/Junior",      model: "sonnet-4-6", task_prefix: "[role:impl-task]" }
  bm:       { runs_on: "EliteDesk/Junior",      model: "haiku-4-5", task_prefix: "[role:bm-task]"   }
  ci_watcher: { runs_on: "EliteDesk/Junior",    model: "haiku-4-5", task_prefix: "[role:ci-watcher]" }
infra:
  elitedesk_alias: "homeserver"
  junior_daemon: "junior@brehon-fork"
  brehon_repo_path: "/srv/brehon-fork"
  tiering_patch_restore: "bash C:/Users/barri/Developer/homeserver/scripts/restore-junior-server-patches.sh"
```

## Polling loop

~10-min `list_tasks` cadence → on transition `show_task + git fetch + read DQ` → triage → queue next. End-to-end: `/auto-phase <phase>`. Full discipline: `.claude/rules/advisor-orchestrator.md`. State machine: `.claude/refs/auto-phase.md` (lazy-loaded by skill body).

## Mandatory user gates (never skip)

1. **Plan approval** — after planning task ships, advisor runs DoD smoke test (every §15 command literally) + watchpoint-specificity gate → surface to user → wait.
2. **Judgment-heavy DQ** — ADR-affecting / scope-changing / visible-to-others-impact entries → surface → wait.
3. **CR triage approval** — after `bm-poll-cr` + `bm-triage` draft → surface four-bucket counts → wait.
4. **Merge confirm** — before `bm-merge` → surface → wait.
5. **Phase 2 e2e — local vs dispatch** — never auto-pick after PR #105.
6. **Retro sign-off** — author retro per `feedback_retro_not_report` + `feedback_four_role_retro_signals` → wait → then `/brehon-phase-transition`.

## Branch Manager (BM) verbs

BM dispatched as `[role:bm-task]`; advisor queues, never authors. Verb catalog + file-ownership: `.claude/rules/branch-manager.md`.

## Shape G — GH-Actions-side cargo validation

Cargo runs on GH Actions (not EliteDesk). JM-e onward = pure Shape G. Pre-Shape-G (≤v1-JM-d) runs on laptop via validate-pending-laptop handler. Details: `.claude/rules/advisor-orchestrator.md`.

## Pre-queue git pre-flight (mandatory)

Junior workers branch from the **committed HEAD** of the trunk branch in `/srv/brehon-fork`. Before every real-work `mcp__junior-brehon__create_task`: run `/precheck` (user-scope command, registered at `~/.claude/commands/precheck.md`). Smoke / diagnostic tasks must branch off a throwaway branch first to isolate contamination.

## Lane worktree bootstrap (mandatory after `git worktree add`)

`git worktree add` does NOT init submodules or copy gitignored files. The `crates/email/translations` submodule directory exists as an empty gitlink — any cargo command touching `lemmy_email` fails with `Os { code: 3, kind: NotFound }` until initialized. Run before first cargo or advisor session in any new lane worktree:

```bash
git submodule update --init --recursive          # fixes lemmy_email build.rs
cp C:/Users/barri/Developer/brehon-fork/.mcp.json        .mcp.json
cp C:/Users/barri/Developer/brehon-fork/.env             .env
cp C:/Users/barri/Developer/brehon-fork/.claude/settings.local.json .claude/settings.local.json
```

Full checklist: `feedback_phase_lane_worktree_bootstrap_checklist.md`.

## Resume / state-recovery

- **`brehon-state-status` agent** (`.claude/agents/brehon-state-status.md`) — dispatch at session start (and any mid-session state check) to get a "where are we?" synthesis **without** burning main-conversation context: it runs the probe battery (CWD/branch/worktrees/stash, origin+daemon-local SHAs + drift, DQ pending, in-flight Junior tasks, open PRs) and returns one screen — raw output stays in the subagent. Read-only, Haiku-pinned. Prefer this over inline probing.
- **`/start-brehon [phase]`** — when you need state loaded **into** this session to act on it. `--fast <N>` for mid-task polling. DQ pending > 0 → `/check-dq`.

## Canonical paths

```yaml
paths:
  # patterns — phase name is a variable, derive current value from git branch
  briefs:        ".claude/PRPs/briefs/<phase>-<role>-<n>.md"
  active_plan:   ".claude/PRPs/plans/<phase>.plan.md"       # glob for current phase
  bootstrap:     ".claude/PRPs/handovers/<phase>-bootstrap.md"
  dq:            ".claude/decision-queue.json"
  runlog:        ".claude/runlog/<phase>-runlog.md"
  # stable paths — never change
  lessons:       ".claude/lessons/{feedback,reference}_*.md"
  pmd_index:     "C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md"
  scratch_plans: "C:/Users/barri/.claude/plans/*.md"        # do NOT auto-delete
  user_commands: "~/.claude/commands/{precheck,start-brehon,advisor-checkpoint}.md"
  user_skills:   "~/.claude/skills/{check-dq,brehon-phase-transition}/"
```

## Where to look next

- **Planning / design docs / crate layout / slash commands / Monday-morning checklist / out-of-scope list:** read `.claude/brehon-reference.md` on demand.
- **Live work hub (current state, what remains):** `.claude/PRPs/v1-roadmap.json` — lane/sub-phase tracker (status, PRs, retros, `what_remains`); driven by `/roadmap-next` + `/auto-roadmap`. v1 PRDs at `.claude/PRPs/prds/v1-*.prd.md`; v2 at `v2-messaging-rtc.prd.md`.
- **Canonical schema / DTO / route reference:** `docs/brehon-law-inspired-network/04-data-model-and-api.md` (LIVING — current v0 base + all merged v1 schema; tables, enums, Diesel models, DTOs, routes, jury-vote lifecycle, federation; derived from live code, CODE WINS on discrepancy).
- **ADRs + open questions:** `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`.

**Never write Rust code without a plan file in `.claude/PRPs/plans/`.** Planning and implementation are separate phases.
