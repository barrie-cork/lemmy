# CLAUDE.md — Brehon Fork (advisor CWD)

**Fork of:** [LemmyNet/lemmy](https://github.com/LemmyNet/lemmy) @ `d1975776a` (Lemmy 1.0-beta; last rebase 2026-04-18)
**Working branch:** `governance-v0` (v0 feature work; `main` is reserved for upstream-sync rebases)
**Active sub-phase:** `v1-SL-c-1` (in flight; trunk at `c93cf7e90`; phase branch `phase-v1-SL-c-1` cut at `477f0c55c` on 2026-05-07). Last shipped: `v1-SL-b` PR #119 merged at `9ae4c332c` on 2026-05-07.
**Rust toolchain:** `1.95` · **License:** AGPL-3.0 (see `AGPL-NOTICE.md`)

A governance-enabled fork of Lemmy 1.0-beta. v0 goal: 11 new API endpoints for a Brehon-style reputation + jury workflow, tamper-evident governance log, and outbound federation of governance signals — while staying compatible with vanilla-Lemmy content federation.

This CWD is the **persistent advisor session** for Brehon governance work. Non-Brehon ops (Docker stacks, agent-grey, midleton-market, web-archive, weekly review, NAS backups, n8n, infra) live in `C:\Users\barri\Developer\homeserver`.

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

The advisor session here drives one Brehon sub-phase end-to-end via four Junior subagents. The advisor is the fourth role — meta-oversight, never authors content.

| Role | Where it runs | Model | Triggered by |
|---|---|---|---|
| **Advisor** | This persistent CC session (laptop, brehon-fork CWD) | Opus 4.7 (1M) — set in `.claude/settings.json` | User opens session |
| **Planning** | Junior worker on EliteDesk | Opus 4.7 (1M) | `[role:planning]` task prefix |
| **Impl** | Junior worker on EliteDesk | Sonnet 4.6 | `[role:impl-task]` task prefix |
| **BM** | Junior worker on EliteDesk | Haiku 4.5 | `[role:bm-task]` task prefix |
| **ci-watcher** | Junior worker on EliteDesk (ad-hoc) | Haiku 4.5 | `[role:ci-watcher]` task prefix |

EliteDesk = Tailscale alias `homeserver`, headless Ubuntu Server 24.04 LTS; reach via `ssh homeserver`. Junior daemon: `junior@brehon-fork`. Brehon fork at `/srv/brehon-fork` on EliteDesk; this CWD on the laptop.

Model tiering enforced by the four-role tiering patch on the EliteDesk. Patch source mirrored at `homeserver/scripts/junior-server-patches/`; restore via `bash C:/Users/barri/Developer/homeserver/scripts/restore-junior-server-patches.sh` after upstream junior-src updates.

## Polling loop

~10-min `list_tasks` cadence → on transition `show_task + git fetch + read DQ` → triage → queue next. End-to-end: `/auto-phase <phase>`. Full discipline: `.claude/rules/advisor-orchestrator.md`. State machine: `.claude/rules/auto-phase.md`.

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

## Resume / state-recovery

`/start-brehon [phase]` — synthesises live state from git/gh/DQ/Junior into one-screen report. `--fast <N>` for mid-task polling. DQ pending > 0 → `/check-dq`.

## Canonical paths

- **Brief paths (this CWD on phase branch):** `.claude/PRPs/briefs/<phase>-<role>-<n>.md`
- **Active plan:** `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` (next; sibling c-2 ships after c-1 merges)
- **Bootstrap brief for next session:** `.claude/PRPs/handovers/v1-SL-c-bootstrap.md`
- **DQ:** `.claude/decision-queue.json` (on phase branch when cut)
- **Runlog:** `.claude/runlog/v1-SL-c-1-runlog.md` (will be created on phase-v1-SL-c-1 by first runlog entry)
- **Lessons (Junior reads at task-0):** `.claude/lessons/feedback_*.md` and `.claude/lessons/reference_*.md` in this repo
- **PMD index (advisor session start):** `C:\Users\barri\.claude\projects\C--Users-barri-Developer-brehon-fork\memory\MEMORY.md`
- **Plans (laptop scratch):** `C:\Users\barri\.claude\plans\*.md` (do NOT auto-delete; consult before queueing)
- **Promoted commands/skills (user scope):** `~/.claude/commands/{precheck,start-brehon,advisor-checkpoint}.md` and `~/.claude/skills/{check-dq,brehon-phase-transition}/`

## Where to look next

- **Planning / design docs / crate layout / slash commands / Monday-morning checklist / out-of-scope list:** read `.claude/brehon-reference.md` on demand.
- **Design doc hub:** `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` (§3 phase-by-phase) and `04-data-model-and-api.md` (tables, DTOs, routes).
- **ADRs + open questions:** `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`.

**Never write Rust code without a plan file in `.claude/PRPs/plans/`.** Planning and implementation are separate phases.
